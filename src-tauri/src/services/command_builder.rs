use crate::models::compress_job::CompressJob;
use crate::services::ass_logo;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn build_preview(ffmpeg_path: &str, job: &CompressJob) -> Result<Vec<String>, String> {
    build_with_options(ffmpeg_path, job, None)
}

/// 构建 ffmpeg 命令；`avs_input_override` 在 AVS 模式压制时由调用方传入实际写好的 input.avs 路径。
/// 预览时传 None：useAvs 仍用 video_path 作为输入占位，避免每次预览都触发文件 IO。
pub fn build_with_options(
    ffmpeg_path: &str,
    job: &CompressJob,
    avs_input_override: Option<&str>,
) -> Result<Vec<String>, String> {
    let video_info = inspect_video(ffmpeg_path, &job.video_path).unwrap_or_default();

    // AVS 模式：ffmpeg 输入改为 .avs 脚本；字幕由脚本内部 TextSubMod 渲染，不再加 subtitles filter
    let input_path: String = if job.use_avs {
        avs_input_override
            .map(|s| s.to_string())
            .unwrap_or_else(|| "<avs script>".to_string())
    } else {
        job.video_path.clone()
    };

    let mut args = vec![
        ffmpeg_path.to_string(),
        "-hide_banner".to_string(),
        "-i".to_string(),
        input_path,
    ];

    let mut filters = Vec::new();

    // LOGO overlay 与 AVS 共存：AVS 处理字幕，LOGO 仍由 ffmpeg movie+overlay 叠加（与原 BAT 行为一致）
    if job.need_logo && subtitle_is_ass(&job.subtitle_path) {
        if let Some(logo) = ass_logo::parse_ass_logo(
            &job.subtitle_path,
            video_info.width,
            video_info.height,
            job.logo_dir.as_deref(),
        )? {
            if job.use_avs {
                filters.push(format!(
                    "movie='{}',scale={}:{}[wm];[in][wm]overlay={}:{}",
                    escape_filter_path(&logo.image_path),
                    logo.width,
                    logo.height,
                    logo.position_x,
                    logo.position_y,
                ));
            } else {
                filters.push(format!(
                    "movie='{}',scale={}:{}[wm];[in][wm]overlay={}:{},subtitles='{}'",
                    escape_filter_path(&logo.image_path),
                    logo.width,
                    logo.height,
                    logo.position_x,
                    logo.position_y,
                    escape_filter_path(&job.subtitle_path)
                ));
            }
        } else if !job.use_avs {
            filters.push(subtitle_filter(&job.subtitle_path));
        }
    } else if !job.use_avs && !job.subtitle_path.trim().is_empty() {
        filters.push(subtitle_filter(&job.subtitle_path));
    }

    if job.need_yadif {
        filters.push("yadif".to_string());
    }

    if !filters.is_empty() {
        args.push("-vf".to_string());
        args.push(filters.join(","));
    }

    args.extend([
        "-c:v".to_string(),
        job.encoder.clone(),
        "-preset".to_string(),
        "veryfast".to_string(),
        "-crf".to_string(),
        job.crf.to_string(),
    ]);

    if let Some(max_bitrate) = job.max_bitrate {
        // 语义：留空(None) = 不限制；0 = 视频码率 + 1000；其他正数 = 直接使用该值（Kbps）
        let resolved: Option<i32> = match max_bitrate {
            v if v < 0 => None, // 兼容旧值：负数视为不限制
            0 => video_info.bitrate_kbps.map(|kb| kb + 1000),
            v => Some(v),
        };
        if let Some(kb) = resolved {
            if kb > 0 {
                args.push("-maxrate".to_string());
                args.push(format!("{kb}k"));
                args.push("-bufsize".to_string());
                args.push(format!("{}k", kb * 2));
            }
        }
    }

    let output_path = normalize_output_path(&job.video_path, &job.output_path);

    args.extend([
        "-c:a".to_string(),
        "aac".to_string(),
        output_path,
        "-y".to_string(),
    ]);

    Ok(args)
}

#[derive(Default)]
pub struct VideoInfo {
    pub duration_seconds: Option<f64>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub bitrate_kbps: Option<i32>,
}

pub fn inspect_video(ffmpeg_path: &str, video_path: &str) -> Result<VideoInfo, String> {
    let output = Command::new(ffmpeg_path)
        .arg("-i")
        .arg(video_path)
        .output()
        .map_err(|err| format!("Failed to run ffmpeg inspect: {err}"))?;
    let text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let size = parse_video_size(&text);
    Ok(VideoInfo {
        duration_seconds: parse_duration(&text),
        width: size.map(|item| item.0),
        height: size.map(|item| item.1),
        bitrate_kbps: parse_bitrate_kbps(&text),
    })
}

pub fn parse_duration(text: &str) -> Option<f64> {
    let marker = "Duration: ";
    let start = text.find(marker)? + marker.len();
    let value = text[start..].split(',').next()?.trim();
    parse_timestamp(value)
}

#[allow(dead_code)]
pub fn parse_progress_time(line: &str) -> Option<f64> {
    if let Some(value) = line.strip_prefix("out_time_ms=") {
        return value.trim().parse::<f64>().ok().map(|ms| ms / 1_000_000.0);
    }
    if let Some(value) = line.strip_prefix("out_time=") {
        return parse_timestamp(value.trim());
    }

    let marker = "time=";
    let start = line.find(marker)? + marker.len();
    let value = line[start..].split_whitespace().next()?.trim();
    parse_timestamp(value)
}

fn parse_timestamp(value: &str) -> Option<f64> {
    let mut parts = value.split(':');
    let hours = parts.next()?.parse::<f64>().ok()?;
    let minutes = parts.next()?.parse::<f64>().ok()?;
    let seconds = parts.next()?.parse::<f64>().ok()?;
    Some(hours * 3600.0 + minutes * 60.0 + seconds)
}

// 从 ffmpeg -i 的输出中解析整体码率，例如 "bitrate: 3402 kb/s"
fn parse_bitrate_kbps(text: &str) -> Option<i32> {
    let marker = "bitrate:";
    let start = text.find(marker)? + marker.len();
    let value = text[start..].split_whitespace().next()?.trim();
    value.parse::<i32>().ok()
}

#[derive(Debug, Default, Clone)]
pub struct ProgressFields {
    pub frame: Option<u64>,
    pub fps: Option<f64>,
    pub size_kb: Option<u64>,
    pub current_seconds: Option<f64>,
    pub bitrate_kbps: Option<f64>,
    pub speed: Option<f64>,
}

// 识别 ffmpeg stderr 持续刷新的进度行，例如：
// "frame=  100 fps= 30 q=22.0 size=    1024kB time=00:00:04.12 bitrate=2034.1kbits/s speed=1.02x"
pub fn is_ffmpeg_progress_line(line: &str) -> bool {
    line.contains("time=") && (line.contains("frame=") || line.contains("size="))
}

pub fn parse_ffmpeg_progress(line: &str) -> ProgressFields {
    // ffmpeg 会用多个空格右对齐数值，如 "size=    1024kB"。先把所有空白压缩成单个空格，
    // 再把 "= " 折叠为 "="，这样 split_whitespace 后每个 key=value 都是一个 token。
    let collapsed: String = line.split_whitespace().collect::<Vec<_>>().join(" ");
    let normalized = collapsed.replace("= ", "=");
    let mut out = ProgressFields::default();
    for token in normalized.split_whitespace() {
        let Some((k, v)) = token.split_once('=') else {
            continue;
        };
        let v = v.trim();
        match k.trim() {
            "frame" => out.frame = v.parse::<u64>().ok(),
            "fps" => out.fps = v.parse::<f64>().ok(),
            // stream-copy 时 ffmpeg 把最终值打印为 Lsize=…
            "size" | "Lsize" => {
                out.size_kb = parse_size_to_kb(v);
            }
            "time" => out.current_seconds = parse_timestamp(v),
            "bitrate" => {
                // "2034.1kbits/s"
                let num = v.trim_end_matches("kbits/s").trim_end_matches("kbit/s");
                out.bitrate_kbps = num.parse::<f64>().ok();
            }
            "speed" => {
                // "1.02x"
                let num = v.trim_end_matches('x');
                out.speed = num.parse::<f64>().ok();
            }
            _ => {}
        }
    }
    out
}

// 解析 ffmpeg 进度行里的 size 字段，统一返回 KB（1KB = 1024B）
fn parse_size_to_kb(v: &str) -> Option<u64> {
    let trimmed = v.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("N/A") {
        return None;
    }
    // 依次尝试更长的后缀，避免 "kB" 提前命中 "B"
    let (num_str, mul_kb) = if let Some(prefix) = trimmed
        .strip_suffix("GiB")
        .or_else(|| trimmed.strip_suffix("GB"))
    {
        (prefix, 1024u64 * 1024)
    } else if let Some(prefix) = trimmed
        .strip_suffix("MiB")
        .or_else(|| trimmed.strip_suffix("MB"))
    {
        (prefix, 1024u64)
    } else if let Some(prefix) = trimmed
        .strip_suffix("KiB")
        .or_else(|| trimmed.strip_suffix("kB"))
        .or_else(|| trimmed.strip_suffix("KB"))
    {
        (prefix, 1u64)
    } else if let Some(prefix) = trimmed.strip_suffix('B') {
        // 纯字节
        return prefix.trim().parse::<u64>().ok().map(|b| b / 1024);
    } else {
        // 没单位，按字节解释
        return trimmed.parse::<u64>().ok().map(|b| b / 1024);
    };
    // 兼容小数：1.5MiB → 1.5 * 1024 ≈ 1536KB
    if let Ok(n) = num_str.trim().parse::<f64>() {
        return Some((n * mul_kb as f64).round() as u64);
    }
    None
}

fn parse_video_size(text: &str) -> Option<(i32, i32)> {
    for line in text.lines().filter(|line| line.contains("Video:")) {
        for raw in line.split(|c: char| c == ',' || c.is_whitespace()) {
            let token = raw.trim();
            let Some((left, right)) = token.split_once('x') else {
                continue;
            };
            let width = left.parse::<i32>().ok()?;
            let height_text: String = right.chars().take_while(|c| c.is_ascii_digit()).collect();
            let height = height_text.parse::<i32>().ok()?;
            if width >= 100 && height >= 100 {
                return Some((width, height));
            }
        }
    }
    None
}

fn subtitle_filter(path: &str) -> String {
    format!("subtitles='{}'", escape_filter_path(path))
}

fn subtitle_is_ass(path: &str) -> bool {
    path.to_ascii_lowercase().ends_with(".ass")
}

pub fn normalize_output_path(video_path: &str, output_path: &str) -> String {
    let trimmed = output_path.trim();
    let video = Path::new(video_path);
    let stem = video
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("output");
    let filename = format!("{stem} output.mp4");

    if trimmed.is_empty() {
        return video
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(filename)
            .to_string_lossy()
            .to_string();
    }

    let output = Path::new(trimmed);
    if trimmed.ends_with('\\') || trimmed.ends_with('/') || output.is_dir() {
        return output.join(filename).to_string_lossy().to_string();
    }

    if output.extension().is_none() {
        return PathBuf::from(output).join(filename).to_string_lossy().to_string();
    }

    trimmed.to_string()
}

fn escape_filter_path(path: &str) -> String {
    path.replace('\\', "\\\\")
        .replace(':', "\\:")
        .replace('[', "\\[")
        .replace(']', "\\]")
        // ffmpeg filter 单引号字符串中不能用 \' 表示单引号；必须闭合后写 \' 再重新打开。
        // 例：'a'\''b' => a'b
        .replace('\'', "'\\''")
}
