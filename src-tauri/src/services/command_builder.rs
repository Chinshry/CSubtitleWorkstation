use crate::models::app_config::LogoLayout;
use crate::models::compress_job::{CompressJob, QuickProcessSettings};
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(windows)]
fn no_window(builder: &mut Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    builder.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
fn no_window(_: &mut Command) {}

pub fn build_preview(ffmpeg_path: &str, job: &CompressJob) -> Result<Vec<String>, String> {
    build_with_options(ffmpeg_path, job, None, None)
}

/// 构建 ffmpeg 命令；`avs_input_override` 在 AVS 模式压制时由调用方传入实际写好的 input.avs 路径。
/// 预览时传 None：useAvs 仍用 video_path 作为输入占位，避免每次预览都触发文件 IO。
pub fn build_with_options(
    ffmpeg_path: &str,
    job: &CompressJob,
    avs_input_override: Option<&str>,
    avs_audio_input_override: Option<&str>,
) -> Result<Vec<String>, String> {
    let video_info = inspect_video(ffmpeg_path, &job.video_path).unwrap_or_default();
    // LOGO overlay 像素换算优先使用前端传入的"显示尺寸"（inspect_video_meta 已应用 rotation）。
    // ffmpeg -i 文本解析拿到的是 codec 帧尺寸，旋转视频会算错，仅作为元数据缺失时的兜底。
    let display_width = job.video_width.or(video_info.width);
    let display_height = job.video_height.or(video_info.height);

    let subtitle_path = job.subtitle_path.trim();
    let use_avs = job.use_avs && !subtitle_path.is_empty();

    // AVS 模式：ffmpeg 输入改为 .avs 脚本；字幕由脚本内部 TextSubMod 渲染，不再加 subtitles filter
    let input_path: String = if use_avs {
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
    if use_avs {
        if let Some(audio_input) = avs_audio_input_override {
            args.extend([
                "-i".to_string(),
                audio_input.to_string(),
                "-map".to_string(),
                "0:v:0".to_string(),
                "-map".to_string(),
                "1:a:0?".to_string(),
            ]);
        }
    }

    let has_subtitle = !use_avs && !subtitle_path.is_empty();
    let mut pre_filters = Vec::new();

    if job.need_yadif {
        pre_filters.push("yadif".to_string());
    }

    if let Some(quick) = quick_process_if_enabled(job) {
        pre_filters.extend(build_quick_process_filters(quick)?);
    }

    let filter_graph = build_video_filter_graph(
        job,
        &pre_filters,
        subtitle_path,
        has_subtitle,
        display_width,
        display_height,
    );

    if let Some(filter_graph) = filter_graph {
        args.push("-vf".to_string());
        args.push(filter_graph);
    }

    let custom_video_args = parse_custom_video_args(job.custom_video_args.as_deref())?;

    args.extend(["-c:v".to_string(), job.encoder.clone()]);

    args.extend(build_quality_args(
        &job.encoder,
        job.crf,
        &custom_video_args,
    ));

    if let Some(max_bitrate) = job.max_bitrate {
        // 语义：留空(None) = 不限制；0 = 视频码率 + 1000；其他正数 = 直接使用该值（Kbps）
        let resolved: Option<i32> = match max_bitrate {
            v if v < 0 => return Err("最大码率不能为负数".to_string()),
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

    args.extend(custom_video_args);

    if let Some(quick) = quick_process_if_enabled(job) {
        args.extend(build_quick_process_output_args(quick)?);
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

fn quick_process_if_enabled(job: &CompressJob) -> Option<&QuickProcessSettings> {
    job.quick_process
        .as_ref()
        .filter(|settings| settings.enabled)
}

fn build_video_filter_graph(
    job: &CompressJob,
    pre_filters: &[String],
    subtitle_path: &str,
    has_subtitle: bool,
    logo_width: Option<i32>,
    logo_height: Option<i32>,
) -> Option<String> {
    let has_logo = build_logo_overlay(job, logo_width, logo_height).is_some();
    if !has_logo {
        let mut filters = pre_filters.to_vec();
        if has_subtitle {
            filters.insert(0, subtitle_filter(subtitle_path));
        }
        return (!filters.is_empty()).then(|| filters.join(","));
    }

    let mut segments = Vec::new();
    let mut current_label = "[in]".to_string();

    if has_subtitle && job.logo_on_top {
        let subtitle_arg = subtitle_filter_arg_for_platform(subtitle_path, cfg!(windows));
        segments.push(format!("{current_label}subtitles={subtitle_arg}[sub]"));
        current_label = "[sub]".to_string();
    }

    let logo_output_label = if has_subtitle && !job.logo_on_top || !pre_filters.is_empty() {
        Some("[logo]")
    } else {
        None
    };
    let logo_overlay = build_logo_overlay_for_platform_with_labels(
        job,
        logo_width,
        logo_height,
        cfg!(windows),
        &current_label,
        logo_output_label,
    )?;
    segments.push(logo_overlay);
    if logo_output_label.is_some() {
        current_label = "[logo]".to_string();
    }

    if has_subtitle && !job.logo_on_top {
        let subtitle_arg = subtitle_filter_arg_for_platform(subtitle_path, cfg!(windows));
        if pre_filters.is_empty() {
            segments.push(format!("{current_label}subtitles={subtitle_arg}"));
        } else {
            segments.push(format!("{current_label}subtitles={subtitle_arg}[subbed]"));
            current_label = "[subbed]".to_string();
        }
    }

    if !pre_filters.is_empty() {
        segments.push(format!("{current_label}{}", pre_filters.join(",")));
    }

    Some(segments.join(";"))
}

fn build_quick_process_filters(settings: &QuickProcessSettings) -> Result<Vec<String>, String> {
    let mut filters = Vec::new();

    let rotation = normalized_choice(&settings.rotation);
    let mirror = normalized_choice(&settings.mirror);
    if is_none_choice(&rotation) && is_none_choice(&mirror) {
        match normalized_choice(&settings.transform).as_str() {
            "" | "none" => {}
            "rotate_cw" => filters.push("transpose=1".to_string()),
            "rotate_ccw" => filters.push("transpose=2".to_string()),
            "rotate_180" => filters.push("hflip,vflip".to_string()),
            "rotate_cw_flip" => filters.push("transpose=3".to_string()),
            "rotate_ccw_flip" => filters.push("transpose=0".to_string()),
            "hflip" => filters.push("hflip".to_string()),
            "vflip" => filters.push("vflip".to_string()),
            other => return Err(format!("未知的视频处理画面变换: {other}")),
        }
    } else {
        match rotation.as_str() {
            "" | "none" => {}
            "rotate_cw" => filters.push("transpose=1".to_string()),
            "rotate_ccw" => filters.push("transpose=2".to_string()),
            "rotate_180" => filters.push("hflip,vflip".to_string()),
            other => return Err(format!("未知的视频处理旋转选项: {other}")),
        }
        match mirror.as_str() {
            "" | "none" => {}
            "hflip" => filters.push("hflip".to_string()),
            "vflip" => filters.push("vflip".to_string()),
            other => return Err(format!("未知的视频处理镜像选项: {other}")),
        }
    }

    if let Some(scale) = quick_process_scale_filter(settings)? {
        filters.push(scale);
    }

    Ok(filters)
}

fn is_none_choice(value: &str) -> bool {
    value.is_empty() || value == "none"
}

fn quick_process_scale_filter(settings: &QuickProcessSettings) -> Result<Option<String>, String> {
    let filter = match normalized_choice(&settings.scale).as_str() {
        "" | "none" => return Ok(None),
        "landscape_4k" => "scale=-1:2160".to_string(),
        "landscape_1080" => "scale=-1:1080".to_string(),
        "landscape_720" => "scale=-1:720".to_string(),
        "portrait_1080" => "scale=1080:-1".to_string(),
        "portrait_720" => "scale=720:-1".to_string(),
        "custom" => {
            let value = settings.custom_scale.trim();
            if !is_valid_custom_scale(value) {
                return Err(
                    "自定义缩放必须填写为 宽:高，例如 1920:1080、-1:1080 或 1080:-1。".to_string(),
                );
            }
            format!("scale={value}")
        }
        other => return Err(format!("未知的视频处理缩放预设: {other}")),
    };
    Ok(Some(filter))
}

fn build_quick_process_output_args(settings: &QuickProcessSettings) -> Result<Vec<String>, String> {
    let mut args = Vec::new();
    if let Some(fps) = settings.frame_rate {
        if !(fps > 0.0 && fps <= 240.0) {
            return Err("视频处理帧率必须大于 0，且不超过 240。".to_string());
        }
        args.extend(["-r".to_string(), trim_float(fps)]);
    }
    if let Some(kbps) = settings.video_bitrate_kbps {
        if kbps <= 0 {
            return Err("视频处理码率必须大于 0。".to_string());
        }
        args.extend(["-b:v".to_string(), format!("{kbps}k")]);
    }
    Ok(args)
}

fn normalized_choice(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn is_valid_custom_scale(value: &str) -> bool {
    let Some((left, right)) = value.split_once(':') else {
        return false;
    };
    is_valid_scale_part(left) && is_valid_scale_part(right)
}

fn is_valid_scale_part(value: &str) -> bool {
    if value == "-1" || value == "-2" {
        return true;
    }
    value
        .parse::<i32>()
        .is_ok_and(|parsed| (1..=8192).contains(&parsed))
}

fn trim_float(value: f64) -> String {
    let rounded = (value * 1000.0).round() / 1000.0;
    let mut text = format!("{rounded:.3}");
    while text.contains('.') && text.ends_with('0') {
        text.pop();
    }
    if text.ends_with('.') {
        text.pop();
    }
    text
}

fn parse_custom_video_args(raw: Option<&str>) -> Result<Vec<String>, String> {
    let Some(raw) = raw.map(str::trim).filter(|v| !v.is_empty()) else {
        return Ok(Vec::new());
    };
    let tokens = split_command_line(raw)?;
    validate_custom_video_args(&tokens)?;
    Ok(tokens)
}

fn split_command_line(raw: &str) -> Result<Vec<String>, String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut chars = raw.chars().peekable();
    let mut quote: Option<char> = None;
    let mut escaped = false;

    while let Some(ch) = chars.next() {
        if escaped {
            current.push(ch);
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if let Some(q) = quote {
            if ch == q {
                quote = None;
            } else {
                current.push(ch);
            }
            continue;
        }
        match ch {
            '\'' | '"' => quote = Some(ch),
            c if c.is_whitespace() => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
                while matches!(chars.peek(), Some(c) if c.is_whitespace()) {
                    chars.next();
                }
            }
            _ => current.push(ch),
        }
    }

    if escaped {
        current.push('\\');
    }
    if quote.is_some() {
        return Err("高级视频参数中的引号没有闭合。".to_string());
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    Ok(tokens)
}

fn validate_custom_video_args(tokens: &[String]) -> Result<(), String> {
    const BANNED: &[&str] = &[
        "-i",
        "-vf",
        "-filter:v",
        "-filter_complex",
        "-c:v",
        "-codec:v",
        "-c:a",
        "-codec:a",
        "-map",
        "-y",
        "-n",
        "-progress",
        "-hide_banner",
    ];
    for token in tokens {
        if !token.starts_with('-') {
            continue;
        }
        let key = token.split('=').next().unwrap_or(token);
        if BANNED.iter().any(|b| key.eq_ignore_ascii_case(b)) {
            return Err(format!(
                "高级视频参数不允许包含 {key}。输入、滤镜、编码器、音频和输出路径由本工具管理。"
            ));
        }
    }
    Ok(())
}

fn build_quality_args(encoder: &str, crf: Option<u8>, custom_video_args: &[String]) -> Vec<String> {
    let Some(crf) = crf else {
        return Vec::new();
    };
    match encoder {
        "h264_nvenc" => {
            let mut args = Vec::new();
            if !has_custom_video_option(custom_video_args, "-rc") {
                args.extend(["-rc".to_string(), "vbr".to_string()]);
            }
            if !has_custom_video_option(custom_video_args, "-cq") {
                args.extend(["-cq".to_string(), crf.to_string()]);
            }
            if !has_custom_video_option(custom_video_args, "-b:v") {
                args.extend(["-b:v".to_string(), "0".to_string()]);
            }
            args
        }
        "h264_amf" => {
            let mut args = Vec::new();
            if !has_custom_video_option(custom_video_args, "-rc") {
                args.extend(["-rc".to_string(), "cqp".to_string()]);
            }
            if !has_custom_video_option(custom_video_args, "-qp_i") {
                args.extend(["-qp_i".to_string(), crf.to_string()]);
            }
            if !has_custom_video_option(custom_video_args, "-qp_p") {
                args.extend(["-qp_p".to_string(), crf.to_string()]);
            }
            if !has_custom_video_option(custom_video_args, "-qp_b") {
                args.extend(["-qp_b".to_string(), crf.to_string()]);
            }
            args
        }
        "h264_videotoolbox" => Vec::new(),
        _ => {
            if has_custom_video_option(custom_video_args, "-crf") {
                Vec::new()
            } else {
                vec!["-crf".to_string(), crf.to_string()]
            }
        }
    }
}

fn has_custom_video_option(tokens: &[String], option: &str) -> bool {
    tokens.iter().any(|token| {
        token
            .split_once('=')
            .map(|(key, _)| key)
            .unwrap_or(token)
            .eq_ignore_ascii_case(option)
    })
}

/// 把 LogoLayout 百分比换算为像素并构造 `movie=...,scale=W:H[wm];[in][wm]overlay=X:Y` 滤镜片段。
/// 当 need_logo=false、未保存布局、或视频分辨率未知时返回 None。
fn build_logo_overlay(
    job: &CompressJob,
    video_width: Option<i32>,
    video_height: Option<i32>,
) -> Option<String> {
    build_logo_overlay_for_platform(job, video_width, video_height, cfg!(windows))
}

fn build_logo_overlay_for_platform(
    job: &CompressJob,
    video_width: Option<i32>,
    video_height: Option<i32>,
    windows: bool,
) -> Option<String> {
    build_logo_overlay_for_platform_with_labels(
        job,
        video_width,
        video_height,
        windows,
        "[in]",
        None,
    )
}

fn build_logo_overlay_for_platform_with_labels(
    job: &CompressJob,
    video_width: Option<i32>,
    video_height: Option<i32>,
    windows: bool,
    input_label: &str,
    output_label: Option<&str>,
) -> Option<String> {
    if !job.need_logo {
        return None;
    }
    let layout = job.logo_layout.as_ref()?;
    let trimmed_path = layout.path.trim();
    if trimmed_path.is_empty() {
        return None;
    }
    let (vw, vh) = match (video_width, video_height) {
        (Some(w), Some(h)) if w > 0 && h > 0 => (w as f64, h as f64),
        // 预览阶段视频分辨率未知时按 1920x1080 估算，仅用于预览展示。
        _ => (1920.0, 1080.0),
    };
    let layout = clamp_layout(layout);
    let w = (layout.w_pct * vw).round().max(1.0) as i32;
    let h = (layout.h_pct * vh).round().max(1.0) as i32;
    let x = (layout.x_pct * vw).round() as i32;
    let y = (layout.y_pct * vh).round() as i32;
    Some(format!(
        "movie={},format=rgba,scale={}:{}:flags=lanczos,format=rgba[wm];{}[wm]overlay={}:{}{}",
        movie_filter_arg_for_platform(trimmed_path, windows),
        w,
        h,
        input_label,
        x,
        y,
        output_label.unwrap_or("")
    ))
}

fn clamp_layout(layout: &LogoLayout) -> LogoLayout {
    LogoLayout {
        path: layout.path.clone(),
        x_pct: layout.x_pct.clamp(-1.0, 1.0),
        y_pct: layout.y_pct.clamp(-1.0, 1.0),
        w_pct: layout.w_pct.clamp(0.001, 1.0),
        h_pct: layout.h_pct.clamp(0.001, 1.0),
    }
}

#[derive(Default)]
pub struct VideoInfo {
    pub duration_seconds: Option<f64>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub bitrate_kbps: Option<i32>,
    pub fps: Option<f64>,
    pub codec_name: Option<String>,
}

pub fn inspect_video(ffmpeg_path: &str, video_path: &str) -> Result<VideoInfo, String> {
    let mut cmd = Command::new(ffmpeg_path);
    cmd.arg("-i").arg(video_path);
    no_window(&mut cmd);
    let output = cmd
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
        fps: parse_video_fps(&text),
        codec_name: parse_video_codec_name(&text),
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
    // ffmpeg 可能输出小数单位：1.5MiB → 1.5 * 1024 ≈ 1536KB
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

pub fn parse_video_fps(text: &str) -> Option<f64> {
    for line in text.lines().filter(|line| line.contains("Video:")) {
        let mut previous: Option<&str> = None;
        for token in line.split_whitespace() {
            let token = token.trim_end_matches(',');
            if token == "fps" {
                return previous?.parse::<f64>().ok();
            }
            previous = Some(token);
        }
    }
    None
}

pub fn parse_video_codec_name(text: &str) -> Option<String> {
    for line in text.lines().filter(|line| line.contains("Video:")) {
        let start = line.find("Video:")? + "Video:".len();
        let codec = line[start..]
            .trim_start()
            .split(|c: char| c == ',' || c == '(' || c.is_whitespace())
            .next()?
            .trim();
        if !codec.is_empty() {
            return Some(codec.to_ascii_lowercase());
        }
    }
    None
}

fn subtitle_filter(path: &str) -> String {
    subtitle_filter_for_platform(path, cfg!(windows))
}

fn subtitle_filter_for_platform(path: &str, windows: bool) -> String {
    format!(
        "subtitles={}",
        subtitle_filter_arg_for_platform(path, windows)
    )
}

fn subtitle_filter_arg_for_platform(path: &str, windows: bool) -> String {
    if windows {
        format!("'{}'", escape_filter_path_for_platform(path, windows))
    } else {
        format!(
            "filename={}",
            escape_filter_path_for_platform(path, windows)
        )
    }
}

fn movie_filter_arg_for_platform(path: &str, windows: bool) -> String {
    if windows {
        format!("'{}'", escape_filter_path_for_platform(path, windows))
    } else {
        format!(
            "filename={}",
            escape_filter_path_for_platform(path, windows)
        )
    }
}

pub fn normalize_output_path(video_path: &str, output_path: &str) -> String {
    let trimmed = output_path.trim();
    let video = Path::new(video_path);
    let stem = video
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("output");
    let filename = format!("{stem} 中字.mp4");

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
        return PathBuf::from(output)
            .join(filename)
            .to_string_lossy()
            .to_string();
    }

    trimmed.to_string()
}

fn escape_filter_path_for_platform(path: &str, windows: bool) -> String {
    let escaped = if windows {
        path.replace('\\', "\\\\")
    } else {
        path.replace('\\', "/").replace(' ', "\\ ")
    };
    escaped
        .replace(':', "\\:")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace(',', "\\,")
        .replace(';', "\\;")
        // ffmpeg filter 单引号字符串中不能用 \' 表示单引号；必须闭合后写 \' 再重新打开。
        // 例：'a'\''b' => a'b
        .replace('\'', "'\\''")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_filter_paths_with_macos_spaces() {
        assert_eq!(
            subtitle_filter_for_platform(
                "/Users/tester/Library/Application Support/app/filter-temp/subtitle.ass",
                false,
            ),
            "subtitles=filename=/Users/tester/Library/Application\\ Support/app/filter-temp/subtitle.ass"
        );

        assert_eq!(
            escape_filter_path_for_platform(
                "/Users/tester/Pictures/Sample Folder.bundle/logo.jpeg",
                false,
            ),
            "/Users/tester/Pictures/Sample\\ Folder.bundle/logo.jpeg"
        );
    }

    #[test]
    fn escapes_filter_paths_with_windows_drive_letters() {
        assert_eq!(
            subtitle_filter_for_platform(
                r"C:\Users\tester\AppData\Local\app\filter-temp\job\subtitle.ass",
                true,
            ),
            "subtitles='C\\:\\\\Users\\\\tester\\\\AppData\\\\Local\\\\app\\\\filter-temp\\\\job\\\\subtitle.ass'"
        );

        assert_eq!(
            escape_filter_path_for_platform(r"E:\sample\project\res\logo\logo.png", true,),
            "E\\:\\\\sample\\\\project\\\\res\\\\logo\\\\logo.png"
        );
    }

    #[test]
    fn escapes_commas_and_semicolons_in_filter_paths() {
        // 路径含 , 和 ; 时应被转义为 \, 和 \;，避免破坏 filter 链结构
        assert_eq!(
            escape_filter_path_for_platform("/Users/tester/Movies/show,part;1/clip.mp4", false,),
            "/Users/tester/Movies/show\\,part\\;1/clip.mp4"
        );
    }

    #[test]
    fn logo_overlay_uses_windows_quoted_movie_path() {
        let job = CompressJob {
            id: "test".to_string(),
            video_path: r"E:\video.mp4".to_string(),
            subtitle_path: String::new(),
            output_path: String::new(),
            crf: Some(18),
            max_bitrate: None,
            need_logo: true,
            need_yadif: false,
            encoder: "libx264".to_string(),
            custom_video_args: None,
            logo_dir: None,
            use_avs: false,
            logo_layout: Some(LogoLayout {
                path: r"E:\sample\project\res\logo\logo.png".to_string(),
                x_pct: 0.02,
                y_pct: 0.02,
                w_pct: 0.2,
                h_pct: 0.1,
            }),
            logo_on_top: false,
            video_width: None,
            video_height: None,
            quick_process: None,
        };

        assert_eq!(
            build_logo_overlay_for_platform(&job, Some(1920), Some(1080), true).as_deref(),
            Some("movie='E\\:\\\\sample\\\\project\\\\res\\\\logo\\\\logo.png',format=rgba,scale=384:108:flags=lanczos,format=rgba[wm];[in][wm]overlay=38:22")
        );
    }

    #[test]
    fn logo_overlay_uses_macos_explicit_movie_filename_option() {
        let job = CompressJob {
            id: "test".to_string(),
            video_path: "/tmp/video.mp4".to_string(),
            subtitle_path: String::new(),
            output_path: String::new(),
            crf: Some(18),
            max_bitrate: None,
            need_logo: true,
            need_yadif: false,
            encoder: "libx264".to_string(),
            custom_video_args: None,
            logo_dir: None,
            use_avs: false,
            logo_layout: Some(LogoLayout {
                path: "/Users/tester/Pictures/Sample Folder.bundle/logo.jpeg".to_string(),
                x_pct: 0.02,
                y_pct: 0.02,
                w_pct: 0.2,
                h_pct: 0.1,
            }),
            logo_on_top: false,
            video_width: None,
            video_height: None,
            quick_process: None,
        };

        assert_eq!(
            build_logo_overlay_for_platform(&job, Some(1920), Some(1080), false).as_deref(),
            Some("movie=filename=/Users/tester/Pictures/Sample\\ Folder.bundle/logo.jpeg,format=rgba,scale=384:108:flags=lanczos,format=rgba[wm];[in][wm]overlay=38:22")
        );
    }

    #[cfg(windows)]
    #[test]
    fn windows_logo_and_subtitle_filter_uses_quoted_paths() {
        let job = CompressJob {
            id: "test".to_string(),
            video_path: r"E:\video.mp4".to_string(),
            subtitle_path: r"C:\Users\tester\AppData\Local\app\filter-temp\job\subtitle.ass"
                .to_string(),
            output_path: r"E:\out.mp4".to_string(),
            crf: Some(18),
            max_bitrate: None,
            need_logo: true,
            need_yadif: false,
            encoder: "libx264".to_string(),
            custom_video_args: None,
            logo_dir: None,
            use_avs: false,
            logo_layout: Some(LogoLayout {
                path: r"E:\sample\project\res\logo\logo.png".to_string(),
                x_pct: 0.02,
                y_pct: 0.02,
                w_pct: 0.2,
                h_pct: 0.1,
            }),
            logo_on_top: false,
            video_width: Some(1920),
            video_height: Some(1080),
            quick_process: None,
        };
        let command =
            build_with_options("definitely-missing-ffmpeg.exe", &job, None, None).unwrap();
        let vf_index = command.iter().position(|arg| arg == "-vf").unwrap();

        assert_eq!(
            command.get(vf_index + 1).map(String::as_str),
            Some("movie='E\\:\\\\sample\\\\project\\\\res\\\\logo\\\\logo.png',format=rgba,scale=384:108:flags=lanczos,format=rgba[wm];[in][wm]overlay=38:22[logo];[logo]subtitles='C\\:\\\\Users\\\\tester\\\\AppData\\\\Local\\\\app\\\\filter-temp\\\\job\\\\subtitle.ass'")
        );
    }

    #[test]
    fn avs_audio_override_adds_second_input_and_maps_streams() {
        let job = CompressJob {
            id: "test".to_string(),
            video_path: r"E:\video.mkv".to_string(),
            subtitle_path: r"E:\sub.ass".to_string(),
            output_path: r"E:\out.mp4".to_string(),
            crf: Some(18),
            max_bitrate: None,
            need_logo: false,
            need_yadif: false,
            encoder: "libx264".to_string(),
            custom_video_args: None,
            logo_dir: None,
            use_avs: true,
            logo_layout: None,
            logo_on_top: false,
            video_width: None,
            video_height: None,
            quick_process: None,
        };
        let command = build_with_options(
            "definitely-missing-ffmpeg.exe",
            &job,
            Some(r"C:\temp\input.avs"),
            Some(r"E:\video.mkv"),
        )
        .unwrap();

        assert_eq!(
            &command[0..10],
            &[
                "definitely-missing-ffmpeg.exe",
                "-hide_banner",
                "-i",
                r"C:\temp\input.avs",
                "-i",
                r"E:\video.mkv",
                "-map",
                "0:v:0",
                "-map",
                "1:a:0?",
            ]
        );
    }

    #[test]
    fn parses_video_fps_and_codec_name() {
        let text =
            "Stream #0:0: Video: vp9 (Profile 0), yuv420p(tv), 2160x3840, 29.97 fps, 29.97 tbr";
        assert_eq!(parse_video_codec_name(text).as_deref(), Some("vp9"));
        assert_eq!(parse_video_fps(text), Some(29.97));
    }

    #[test]
    fn quick_process_builds_filters_and_output_args() {
        let job = CompressJob {
            id: "test".to_string(),
            video_path: r"E:\video.mp4".to_string(),
            subtitle_path: String::new(),
            output_path: r"E:\out.mp4".to_string(),
            crf: Some(18),
            max_bitrate: None,
            need_logo: false,
            need_yadif: true,
            encoder: "libx264".to_string(),
            custom_video_args: None,
            logo_dir: None,
            use_avs: false,
            logo_layout: None,
            logo_on_top: false,
            video_width: None,
            video_height: None,
            quick_process: Some(QuickProcessSettings {
                enabled: true,
                transform: String::new(),
                rotation: "none".to_string(),
                mirror: "hflip".to_string(),
                scale: "landscape_1080".to_string(),
                custom_scale: String::new(),
                frame_rate: Some(30.0),
                video_bitrate_kbps: Some(4000),
            }),
        };
        let command =
            build_with_options("definitely-missing-ffmpeg.exe", &job, None, None).unwrap();
        let vf_index = command.iter().position(|arg| arg == "-vf").unwrap();

        assert_eq!(
            command.get(vf_index + 1).map(String::as_str),
            Some("yadif,hflip,scale=-1:1080")
        );
        assert!(command
            .windows(2)
            .any(|pair| pair[0] == "-r" && pair[1] == "30"));
        assert!(command
            .windows(2)
            .any(|pair| pair[0] == "-b:v" && pair[1] == "4000k"));
    }

    #[test]
    fn quick_process_rotate_180_uses_flip_pair() {
        let settings = QuickProcessSettings {
            enabled: true,
            transform: String::new(),
            rotation: "rotate_180".to_string(),
            mirror: "none".to_string(),
            scale: "none".to_string(),
            custom_scale: String::new(),
            frame_rate: None,
            video_bitrate_kbps: None,
        };

        assert_eq!(
            build_quick_process_filters(&settings).unwrap(),
            vec!["hflip,vflip".to_string()]
        );
    }

    #[cfg(windows)]
    #[test]
    fn logo_overlay_runs_before_quick_process() {
        let job = CompressJob {
            id: "test".to_string(),
            video_path: r"E:\video.mp4".to_string(),
            subtitle_path: String::new(),
            output_path: r"E:\out.mp4".to_string(),
            crf: Some(18),
            max_bitrate: None,
            need_logo: true,
            need_yadif: false,
            encoder: "libx264".to_string(),
            custom_video_args: None,
            logo_dir: None,
            use_avs: false,
            logo_layout: Some(LogoLayout {
                path: r"E:\sample\project\res\logo\logo.png".to_string(),
                x_pct: 0.02,
                y_pct: 0.02,
                w_pct: 0.2,
                h_pct: 0.1,
            }),
            logo_on_top: false,
            video_width: Some(1920),
            video_height: Some(1080),
            quick_process: Some(QuickProcessSettings {
                enabled: true,
                transform: String::new(),
                rotation: "none".to_string(),
                mirror: "hflip".to_string(),
                scale: "landscape_720".to_string(),
                custom_scale: String::new(),
                frame_rate: None,
                video_bitrate_kbps: None,
            }),
        };
        let command =
            build_with_options("definitely-missing-ffmpeg.exe", &job, None, None).unwrap();
        let vf_index = command.iter().position(|arg| arg == "-vf").unwrap();

        assert_eq!(
            command.get(vf_index + 1).map(String::as_str),
            Some("movie='E\\:\\\\sample\\\\project\\\\res\\\\logo\\\\logo.png',format=rgba,scale=384:108:flags=lanczos,format=rgba[wm];[in][wm]overlay=38:22[logo];[logo]hflip,scale=-1:720")
        );
    }

    #[test]
    fn avs_without_subtitle_falls_back_to_video_input() {
        let job = CompressJob {
            id: "test".to_string(),
            video_path: r"E:\video.mp4".to_string(),
            subtitle_path: String::new(),
            output_path: r"E:\out.mp4".to_string(),
            crf: Some(18),
            max_bitrate: None,
            need_logo: false,
            need_yadif: false,
            encoder: "libx264".to_string(),
            custom_video_args: None,
            logo_dir: None,
            use_avs: true,
            logo_layout: None,
            logo_on_top: false,
            video_width: None,
            video_height: None,
            quick_process: None,
        };
        let command =
            build_with_options("definitely-missing-ffmpeg.exe", &job, None, None).unwrap();

        assert_eq!(command.get(3).map(String::as_str), Some(r"E:\video.mp4"));
        assert!(!command.iter().any(|arg| arg == "<avs script>"));
    }

    #[test]
    fn empty_quality_omits_crf_args() {
        let job = CompressJob {
            id: "test".to_string(),
            video_path: r"E:\video.mp4".to_string(),
            subtitle_path: String::new(),
            output_path: r"E:\out.mp4".to_string(),
            crf: None,
            max_bitrate: None,
            need_logo: false,
            need_yadif: false,
            encoder: "libx264".to_string(),
            custom_video_args: None,
            logo_dir: None,
            use_avs: false,
            logo_layout: None,
            logo_on_top: false,
            video_width: None,
            video_height: None,
            quick_process: None,
        };
        let command =
            build_with_options("definitely-missing-ffmpeg.exe", &job, None, None).unwrap();

        assert!(!command.iter().any(|arg| arg == "-crf"));
    }
}
