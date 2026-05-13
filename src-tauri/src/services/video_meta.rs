use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::process::Command;

#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoMeta {
    // 文件
    pub file_size_bytes: Option<u64>,
    pub created_at: Option<String>,

    // 容器
    pub duration_seconds: Option<f64>,
    pub duration_text: Option<String>,
    pub start_seconds: Option<f64>,
    pub overall_bitrate_kbps: Option<i32>,
    pub format: Option<String>,

    // 视频流
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub sar: Option<String>,
    pub dar: Option<String>,
    pub video_codec: Option<String>,
    pub video_profile: Option<String>,
    pub pixel_format: Option<String>,
    pub color_range: Option<String>,
    pub color_space: Option<String>,
    pub fps: Option<f64>,
    pub tbr: Option<f64>,
    pub video_bitrate_kbps: Option<i32>,
    pub total_frames: Option<u64>,
    pub frame_rate_mode: Option<String>, // "CFR" | "VFR"

    // 音频流
    pub audio_codec: Option<String>,
    pub audio_profile: Option<String>,
    pub audio_sample_rate: Option<i32>,
    pub audio_channels: Option<String>,
    pub audio_bitrate_kbps: Option<i32>,
}

#[cfg(windows)]
fn no_window(builder: &mut Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    builder.creation_flags(CREATE_NO_WINDOW);
}
#[cfg(not(windows))]
fn no_window(_: &mut Command) {}

/// 用 ffprobe 拿结构化元数据；ffprobe_path 为 None 时回退到 ffmpeg -i 文本解析。
pub fn inspect(
    ffmpeg_path: &str,
    ffprobe_path: Option<&str>,
    video_path: &str,
) -> Result<VideoMeta, String> {
    let mut meta = VideoMeta::default();

    // 文件大小
    if let Ok(md) = fs::metadata(video_path) {
        meta.file_size_bytes = Some(md.len());
    }

    if let Some(probe) = ffprobe_path {
        if inspect_with_ffprobe(probe, video_path, &mut meta).is_ok() {
            return Ok(meta);
        }
    }

    // 回退：ffmpeg -i 文本解析（旧实现，精度较差）
    inspect_with_ffmpeg(ffmpeg_path, video_path, &mut meta)?;
    Ok(meta)
}

fn inspect_with_ffprobe(
    ffprobe_path: &str,
    video_path: &str,
    meta: &mut VideoMeta,
) -> Result<(), String> {
    let mut cmd = Command::new(ffprobe_path);
    cmd.args([
        "-v",
        "error",
        "-hide_banner",
        "-show_streams",
        "-show_format",
        "-of",
        "json",
    ])
    .arg(video_path);
    no_window(&mut cmd);
    let output = cmd
        .output()
        .map_err(|err| format!("Failed to run ffprobe: {err}"))?;
    if !output.status.success() {
        return Err(format!("ffprobe exited with status: {}", output.status));
    }
    let text = String::from_utf8_lossy(&output.stdout).into_owned();
    let json: Value =
        serde_json::from_str(&text).map_err(|err| format!("ffprobe json parse error: {err}"))?;

    // format 段
    if let Some(format) = json.get("format") {
        meta.format = format
            .get("format_name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        meta.duration_seconds = format
            .get("duration")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok());
        if let Some(dur) = meta.duration_seconds {
            meta.duration_text = Some(format_duration(dur));
        }
        meta.start_seconds = format
            .get("start_time")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok());
        meta.overall_bitrate_kbps = format
            .get("bit_rate")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<i64>().ok())
            .map(|bps| (bps / 1000) as i32);
        if let Some(tags) = format.get("tags") {
            for key in ["creation_time", "date", "DATE"] {
                if let Some(v) = tags.get(key).and_then(|v| v.as_str()) {
                    let day = v.split('T').next().unwrap_or(v).split(' ').next().unwrap_or(v);
                    meta.created_at = Some(day.to_string());
                    break;
                }
            }
        }
    }

    // streams 段
    if let Some(streams) = json.get("streams").and_then(|v| v.as_array()) {
        for stream in streams {
            let codec_type = stream
                .get("codec_type")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            match codec_type {
                "video" if meta.video_codec.is_none() => fill_video_stream(stream, meta),
                "audio" if meta.audio_codec.is_none() => fill_audio_stream(stream, meta),
                _ => {}
            }
        }
    }

    Ok(())
}

fn fill_video_stream(s: &Value, meta: &mut VideoMeta) {
    meta.video_codec = s.get("codec_name").and_then(|v| v.as_str()).map(|s| s.to_string());
    meta.video_profile = s.get("profile").and_then(|v| v.as_str()).map(|s| s.to_string());
    meta.pixel_format = s.get("pix_fmt").and_then(|v| v.as_str()).map(|s| s.to_string());
    meta.color_range = s.get("color_range").and_then(|v| v.as_str()).map(|s| s.to_string());
    meta.color_space = s.get("color_space").and_then(|v| v.as_str()).map(|s| s.to_string());
    let raw_w = s.get("width").and_then(|v| v.as_i64()).map(|n| n as i32);
    let raw_h = s.get("height").and_then(|v| v.as_i64()).map(|n| n as i32);
    // 手机竖屏视频常存为 1920x1080 + rotation=90，ffmpeg 解码后会自动旋转为 1080x1920。
    // 这里把 width/height 修正为旋转后的"显示尺寸"，使后续 overlay 像素换算与前端预览一致。
    if is_rotated_90(s) {
        meta.width = raw_h;
        meta.height = raw_w;
    } else {
        meta.width = raw_w;
        meta.height = raw_h;
    }
    meta.sar = s.get("sample_aspect_ratio").and_then(|v| v.as_str()).map(|s| s.to_string());
    meta.dar = s.get("display_aspect_ratio").and_then(|v| v.as_str()).map(|s| s.to_string());

    let r_rate = s.get("r_frame_rate").and_then(|v| v.as_str()).and_then(parse_rational);
    let a_rate = s.get("avg_frame_rate").and_then(|v| v.as_str()).and_then(parse_rational);

    // 帧率显示用 avg（更接近实际播放帧率）；若无则用 r
    meta.fps = a_rate.or(r_rate).map(round2);
    meta.tbr = r_rate.map(round2);

    // 帧率模式判定：r 与 avg 都有效时比较
    meta.frame_rate_mode = match (r_rate, a_rate) {
        (Some(r), Some(a)) if r > 0.0 && a > 0.0 => {
            let max = r.max(a);
            let diff_ratio = (r - a).abs() / max;
            if diff_ratio < 0.001 {
                Some("CFR".to_string())
            } else {
                Some("VFR".to_string())
            }
        }
        _ => None,
    };

    meta.video_bitrate_kbps = s
        .get("bit_rate")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<i64>().ok())
        .map(|bps| (bps / 1000) as i32);

    meta.total_frames = s
        .get("nb_frames")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<u64>().ok());
}

fn fill_audio_stream(s: &Value, meta: &mut VideoMeta) {
    meta.audio_codec = s.get("codec_name").and_then(|v| v.as_str()).map(|s| s.to_string());
    meta.audio_profile = s.get("profile").and_then(|v| v.as_str()).map(|s| s.to_string());
    meta.audio_sample_rate = s
        .get("sample_rate")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<i32>().ok());
    meta.audio_channels = s
        .get("channel_layout")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            s.get("channels")
                .and_then(|v| v.as_i64())
                .map(|n| format!("{n} channels"))
        });
    meta.audio_bitrate_kbps = s
        .get("bit_rate")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<i64>().ok())
        .map(|bps| (bps / 1000) as i32);
}

fn parse_rational(s: &str) -> Option<f64> {
    let mut parts = s.split('/');
    let n: f64 = parts.next()?.parse().ok()?;
    let d: f64 = parts.next()?.parse().ok()?;
    if d == 0.0 {
        return None;
    }
    Some(n / d)
}

/// ffprobe JSON stream 中的 rotation 信息可能位于：
/// - tags.rotate（旧版 mov 容器）："90" / "180" / "270"
/// - side_data_list[].rotation（新版 ffprobe，displaymatrix）：数值，例如 -90
/// 任一通道命中 ±90 / ±270 都视为需要交换宽高。
fn is_rotated_90(s: &Value) -> bool {
    let Some(deg) = extract_rotation_degrees(s) else {
        return false;
    };
    let norm = ((deg.round() as i64) % 360 + 360) % 360;
    norm == 90 || norm == 270
}

fn extract_rotation_degrees(s: &Value) -> Option<f64> {
    if let Some(v) = s.get("tags").and_then(|t| t.get("rotate")) {
        if let Some(s) = v.as_str() {
            if let Ok(r) = s.parse::<f64>() {
                return Some(r);
            }
        }
        if let Some(r) = v.as_f64() {
            return Some(r);
        }
    }
    if let Some(list) = s.get("side_data_list").and_then(|v| v.as_array()) {
        for sd in list {
            if let Some(r) = sd.get("rotation").and_then(|v| v.as_f64()) {
                return Some(r);
            }
            if let Some(r) = sd.get("rotation").and_then(|v| v.as_i64()) {
                return Some(r as f64);
            }
            if let Some(s) = sd.get("rotation").and_then(|v| v.as_str()) {
                if let Ok(r) = s.parse::<f64>() {
                    return Some(r);
                }
            }
        }
    }
    None
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

fn format_duration(seconds: f64) -> String {
    let total = seconds as u64;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    let ms = ((seconds - total as f64) * 100.0).round() as u32;
    format!("{:02}:{:02}:{:02}.{:02}", h, m, s, ms)
}

// ============ 回退：ffmpeg -i 文本解析（精度较差，仅在没有 ffprobe 时启用） ============

fn inspect_with_ffmpeg(
    ffmpeg_path: &str,
    video_path: &str,
    meta: &mut VideoMeta,
) -> Result<(), String> {
    let mut cmd = Command::new(ffmpeg_path);
    cmd.arg("-hide_banner").arg("-i").arg(video_path);
    no_window(&mut cmd);
    let output = cmd
        .output()
        .map_err(|err| format!("Failed to run ffmpeg inspect: {err}"))?;
    let text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    parse_container(&text, meta);
    parse_video_stream(&text, meta);
    parse_audio_stream(&text, meta);
    parse_creation_date(&text, meta);
    Ok(())
}

fn parse_container(text: &str, meta: &mut VideoMeta) {
    if let Some(line) = text.lines().find(|l| l.trim_start().starts_with("Input #0,")) {
        if let Some(rest) = line.trim_start().strip_prefix("Input #0,") {
            if let Some(fmt) = rest.split(", from").next() {
                let fmt = fmt.trim().trim_end_matches(',').trim();
                if !fmt.is_empty() {
                    meta.format = Some(fmt.to_string());
                }
            }
        }
    }
    if let Some(line) = text.lines().find(|l| l.trim_start().starts_with("Duration:")) {
        let line = line.trim_start();
        if let Some(rest) = line.strip_prefix("Duration:") {
            for part in rest.split(',') {
                let part = part.trim();
                if let Some(v) = part.strip_prefix("start:") {
                    meta.start_seconds = v.trim().parse::<f64>().ok();
                } else if let Some(v) = part.strip_prefix("bitrate:") {
                    let num = v.trim().split_whitespace().next().unwrap_or("");
                    meta.overall_bitrate_kbps = num.parse::<i32>().ok();
                } else if !part.is_empty() && meta.duration_text.is_none() {
                    meta.duration_text = Some(part.to_string());
                    meta.duration_seconds = parse_timestamp(part);
                }
            }
        }
    }
}

fn parse_video_stream(text: &str, meta: &mut VideoMeta) {
    let Some(line) = text.lines().find(|l| l.contains("Video:")) else { return };
    let line = line.trim();
    if let Some(after) = line.split("Video:").nth(1) {
        let after = after.trim();
        let mut iter = after.splitn(2, ',');
        if let Some(head) = iter.next() {
            let head = head.trim();
            let codec = head.split_whitespace().next().unwrap_or("").to_string();
            if !codec.is_empty() {
                meta.video_codec = Some(codec);
            }
            if let Some(start) = head.find('(') {
                if let Some(end) = head[start + 1..].find(')') {
                    let profile = &head[start + 1..start + 1 + end];
                    if !profile.is_empty() && !profile.starts_with("avc") && !profile.contains('/')
                    {
                        meta.video_profile = Some(profile.to_string());
                    }
                }
            }
        }
    }

    // 像素格式 + 色域 + 色范围
    if let Some(pix) = extract_pixel_format(line) {
        meta.pixel_format = Some(pix.format);
        meta.color_range = pix.color_range;
        meta.color_space = pix.color_space;
    }

    // 分辨率 + DAR + SAR
    if let Some((w, h)) = parse_dimensions(line) {
        meta.width = Some(w);
        meta.height = Some(h);
    }
    if let Some(sar) = extract_bracketed_kv(line, "SAR ") {
        meta.sar = Some(sar);
    }
    if let Some(dar) = extract_bracketed_kv(line, "DAR ") {
        meta.dar = Some(dar);
    }

    for part in line.split(',') {
        let p = part.trim();
        if p.ends_with("kb/s") || p.ends_with("kbit/s") {
            let num = p.split_whitespace().next().unwrap_or("");
            if let Ok(n) = num.parse::<i32>() {
                meta.video_bitrate_kbps = Some(n);
            }
        } else if let Some(rest) = p.strip_suffix(" fps") {
            meta.fps = rest.trim().parse::<f64>().ok();
        } else if let Some(rest) = p.strip_suffix(" tbr") {
            meta.tbr = rest.trim().parse::<f64>().ok();
        }
    }

    // 文本解析得到的 width/height 仍是 codec 帧尺寸，需要按 rotation 修正为显示尺寸。
    if let Some(deg) = parse_rotation_from_ffmpeg_text(text) {
        let norm = ((deg.round() as i64) % 360 + 360) % 360;
        if norm == 90 || norm == 270 {
            if let (Some(w), Some(h)) = (meta.width, meta.height) {
                meta.width = Some(h);
                meta.height = Some(w);
            }
        }
    }
}

/// 从 `ffmpeg -i` stderr 中找 rotation：
/// - 优先 Side data 的 `displaymatrix: rotation of -90.00 degrees`
/// - 回退到 metadata 的 `rotate          : 90`
fn parse_rotation_from_ffmpeg_text(text: &str) -> Option<f64> {
    for line in text.lines() {
        let l = line.trim();
        if let Some(idx) = l.find("rotation of") {
            let rest = &l[idx + "rotation of".len()..];
            let num = rest.split_whitespace().next().unwrap_or("");
            if let Ok(r) = num.parse::<f64>() {
                return Some(r);
            }
        }
    }
    for line in text.lines() {
        let l = line.trim();
        if l.to_ascii_lowercase().starts_with("rotate") {
            if let Some(after_key) = l.splitn(2, ':').nth(1) {
                let v = after_key.trim();
                if !v.is_empty() {
                    if let Ok(r) = v.parse::<f64>() {
                        return Some(r);
                    }
                }
            }
        }
    }
    None
}

struct PixInfo {
    format: String,
    color_range: Option<String>,
    color_space: Option<String>,
}

fn extract_pixel_format(line: &str) -> Option<PixInfo> {
    let after = line.split("Video:").nth(1)?;
    for raw in after.split(',') {
        let token = raw.trim();
        let head = token.split('(').next().unwrap_or("").trim();
        if head.starts_with("yuv")
            || head.starts_with("nv")
            || head.starts_with("gray")
            || head.starts_with("rgb")
            || head.starts_with("bgr")
            || head.starts_with("p0")
        {
            let mut info = PixInfo {
                format: head.to_string(),
                color_range: None,
                color_space: None,
            };
            if let Some(start) = token.find('(') {
                if let Some(end) = token[start + 1..].find(')') {
                    let inner = &token[start + 1..start + 1 + end];
                    for piece in inner.split(',') {
                        let piece = piece.trim();
                        match piece {
                            "tv" | "pc" | "limited" | "full" => {
                                info.color_range = Some(piece.to_string());
                            }
                            _ if piece.starts_with("bt") || piece.starts_with("smpte") => {
                                info.color_space = Some(piece.to_string());
                            }
                            _ => {}
                        }
                    }
                }
            }
            return Some(info);
        }
    }
    None
}

fn extract_bracketed_kv(line: &str, key: &str) -> Option<String> {
    let start = line.find('[')?;
    let end = line[start..].find(']')?;
    let inner = &line[start + 1..start + end];
    let k = inner.find(key)? + key.len();
    let value = inner[k..]
        .split_whitespace()
        .next()?
        .trim_end_matches(']')
        .to_string();
    Some(value)
}

fn parse_dimensions(line: &str) -> Option<(i32, i32)> {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        while i < bytes.len() && !bytes[i].is_ascii_digit() {
            i += 1;
        }
        let s = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if s == i {
            break;
        }
        let n1 = std::str::from_utf8(&bytes[s..i])
            .ok()
            .and_then(|t| t.parse::<i32>().ok())
            .unwrap_or(0);
        if i < bytes.len() && bytes[i] == b'x' {
            let xs = i + 1;
            let mut xe = xs;
            while xe < bytes.len() && bytes[xe].is_ascii_digit() {
                xe += 1;
            }
            if xe > xs {
                let n2 = std::str::from_utf8(&bytes[xs..xe])
                    .ok()
                    .and_then(|t| t.parse::<i32>().ok())
                    .unwrap_or(0);
                if n1 >= 100 && n2 >= 100 {
                    return Some((n1, n2));
                }
                i = xe;
            }
        }
    }
    None
}

fn parse_audio_stream(text: &str, meta: &mut VideoMeta) {
    let Some(line) = text.lines().find(|l| l.contains("Audio:")) else { return };
    let line = line.trim();
    if let Some(after) = line.split("Audio:").nth(1) {
        let after = after.trim();
        if let Some(head) = after.split(',').next() {
            let head = head.trim();
            let codec = head.split_whitespace().next().unwrap_or("").to_string();
            if !codec.is_empty() {
                meta.audio_codec = Some(codec);
            }
        }
        for part in after.split(',') {
            let p = part.trim();
            if let Some(rest) = p.strip_suffix(" Hz") {
                meta.audio_sample_rate = rest.trim().parse::<i32>().ok();
            } else if matches!(p, "mono" | "stereo" | "5.0" | "5.1" | "7.1" | "quad")
                || p.ends_with(" channels")
            {
                meta.audio_channels = Some(p.to_string());
            } else if p.ends_with("kb/s") || p.ends_with("kbit/s") {
                let num = p.split_whitespace().next().unwrap_or("");
                if let Ok(n) = num.parse::<i32>() {
                    meta.audio_bitrate_kbps = Some(n);
                }
            }
        }
    }
}

fn parse_creation_date(text: &str, meta: &mut VideoMeta) {
    for line in text.lines() {
        let l = line.trim();
        for key in ["date            :", "creation_time   :", "date:", "creation_time:"] {
            if let Some(rest) = l.strip_prefix(key) {
                let v = rest.trim();
                if !v.is_empty() {
                    let day = v.split('T').next().unwrap_or(v).split(' ').next().unwrap_or(v);
                    meta.created_at = Some(day.to_string());
                    return;
                }
            }
        }
    }
}

fn parse_timestamp(value: &str) -> Option<f64> {
    let mut parts = value.split(':');
    let h = parts.next()?.parse::<f64>().ok()?;
    let m = parts.next()?.parse::<f64>().ok()?;
    let s = parts.next()?.parse::<f64>().ok()?;
    Some(h * 3600.0 + m * 60.0 + s)
}
