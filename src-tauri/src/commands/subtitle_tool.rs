use crate::services::{config_store, ffmpeg_locator};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use tauri::AppHandle;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleFormatJob {
    pub input_path: String,
    pub output_path: String,
    pub target_format: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleFormatResult {
    pub output_path: String,
    pub logs: Vec<String>,
}

#[tauri::command]
pub fn preview_subtitle_format_command(
    app: AppHandle,
    job: SubtitleFormatJob,
) -> Result<Vec<String>, String> {
    let config = config_store::load(&app)?;
    let status = ffmpeg_locator::detect(&config);
    let ffmpeg_path = status
        .ffmpeg_path
        .ok_or_else(|| "ffmpeg is not configured.".to_string())?;
    build_subtitle_format_command(&ffmpeg_path, &job)
}

#[tauri::command]
pub async fn convert_subtitle_format(
    app: AppHandle,
    job: SubtitleFormatJob,
) -> Result<SubtitleFormatResult, String> {
    tauri::async_runtime::spawn_blocking(move || convert_subtitle_format_blocking(app, job))
        .await
        .map_err(|err| format!("字幕格式转换任务启动失败: {err}"))?
}

fn convert_subtitle_format_blocking(
    app: AppHandle,
    job: SubtitleFormatJob,
) -> Result<SubtitleFormatResult, String> {
    let config = config_store::load(&app)?;
    let status = ffmpeg_locator::detect(&config);
    if !status.available {
        return Err(status
            .message
            .unwrap_or_else(|| "未检测到可用的 ffmpeg。".to_string()));
    }
    let ffmpeg_path = status
        .ffmpeg_path
        .ok_or_else(|| "ffmpeg is not configured.".to_string())?;
    validate_subtitle_format_job(&job)?;
    ensure_output_parent_dir(&job.output_path)?;
    let target = normalize_target_format(&job.target_format)?;

    if is_ttml_path(Path::new(&job.input_path)) {
        if target != "srt" {
            return Err("TTML 输入目前仅支持转换为 SRT。".to_string());
        }
        let srt_text = ttml_file_to_srt(&job.input_path)?;
        fs::write(&job.output_path, srt_text).map_err(|err| format!("写入 SRT 字幕失败: {err}"))?;
        return Ok(SubtitleFormatResult {
            output_path: job.output_path,
            logs: vec![
                "Converted TTML to SRT internally.".to_string(),
                "Subtitle format conversion completed.".to_string(),
            ],
        });
    }

    let command = build_subtitle_format_command(&ffmpeg_path, &job)?;
    run_subtitle_format_command(command, job.output_path)
}

fn build_subtitle_format_command(
    ffmpeg_path: &str,
    job: &SubtitleFormatJob,
) -> Result<Vec<String>, String> {
    validate_subtitle_format_job(job)?;
    let target = normalize_target_format(&job.target_format)?;
    if is_ttml_path(Path::new(&job.input_path)) {
        if target != "srt" {
            return Err("TTML 输入目前仅支持转换为 SRT。".to_string());
        }
        return Ok(vec![
            "internal-ttml-to-srt".to_string(),
            job.input_path.clone(),
            job.output_path.clone(),
        ]);
    }
    Ok(build_subtitle_format_command_for_input(
        ffmpeg_path,
        &job.input_path,
        &job.output_path,
    ))
}

fn build_subtitle_format_command_for_input(
    ffmpeg_path: &str,
    input_path: &str,
    output_path: &str,
) -> Vec<String> {
    vec![
        ffmpeg_path.to_string(),
        "-hide_banner".to_string(),
        "-y".to_string(),
        "-i".to_string(),
        input_path.to_string(),
        output_path.to_string(),
    ]
}

fn run_subtitle_format_command(
    command: Vec<String>,
    output_path: String,
) -> Result<SubtitleFormatResult, String> {
    let output = Command::new(&command[0])
        .args(&command[1..])
        .output()
        .map_err(|err| format!("启动 ffmpeg 失败: {err}"))?;

    let mut logs = Vec::new();
    logs.push(format!("Command: {}", command.join(" ")));
    collect_process_output(&mut logs, &output.stdout);
    collect_process_output(&mut logs, &output.stderr);

    if !output.status.success() {
        logs.push(format!("字幕格式转换失败: {}", output.status));
        return Err(logs.join("\n"));
    }

    logs.push("Subtitle format conversion completed.".to_string());
    Ok(SubtitleFormatResult { output_path, logs })
}
fn validate_subtitle_format_job(job: &SubtitleFormatJob) -> Result<(), String> {
    if job.input_path.trim().is_empty() {
        return Err("请选择输入字幕。".to_string());
    }
    if job.output_path.trim().is_empty() {
        return Err("请选择输出路径。".to_string());
    }
    let input = Path::new(job.input_path.trim());
    if !input.is_file() || !is_supported_input(input) {
        return Err("输入字幕仅支持 ASS / SSA / SRT / VTT / TTML / SUB。".to_string());
    }
    let target = normalize_target_format(&job.target_format)?;
    if is_ttml_path(input) && target != "srt" {
        return Err("TTML 输入目前仅支持转换为 SRT。".to_string());
    }
    if !job
        .output_path
        .to_ascii_lowercase()
        .ends_with(&format!(".{target}"))
    {
        return Err(format!("输出文件扩展名必须是 .{target}。"));
    }
    if comparable_path(&job.input_path) == comparable_path(&job.output_path) {
        return Err("输出路径不能和输入字幕相同。".to_string());
    }
    Ok(())
}

fn normalize_target_format(value: &str) -> Result<&'static str, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "ass" => Ok("ass"),
        "ssa" => Ok("ssa"),
        "srt" => Ok("srt"),
        "vtt" => Ok("vtt"),
        _ => Err("目标格式仅支持 ASS / SSA / SRT / VTT。".to_string()),
    }
}
fn is_supported_input(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .map(|ext| {
            matches!(
                ext.to_ascii_lowercase().as_str(),
                "ass" | "ssa" | "srt" | "vtt" | "ttml" | "sub"
            )
        })
        .unwrap_or(false)
}

fn is_ttml_path(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("ttml"))
}

fn ttml_file_to_srt(path: &str) -> Result<String, String> {
    let raw = fs::read_to_string(path).map_err(|err| format!("读取 TTML 字幕失败: {err}"))?;
    let mut output = String::new();
    let mut count = 0usize;

    for captures in ttml_p_regex().captures_iter(&raw) {
        let attrs = captures.get(1).map(|value| value.as_str()).unwrap_or("");
        let body = captures.get(2).map(|value| value.as_str()).unwrap_or("");
        let Some(start) = ttml_attr(attrs, "begin").and_then(|value| parse_ttml_time(&value))
        else {
            continue;
        };
        let end = ttml_attr(attrs, "end")
            .and_then(|value| parse_ttml_time(&value))
            .or_else(|| {
                ttml_attr(attrs, "dur")
                    .and_then(|value| parse_ttml_time(&value))
                    .map(|dur| start + dur)
            });
        let Some(end) = end else {
            continue;
        };
        let text = clean_ttml_text(body);
        if text.trim().is_empty() || end <= start {
            continue;
        }
        count += 1;
        output.push_str(&format!(
            "{}\n{} --> {}\n{}\n\n",
            count,
            format_srt_time(start),
            format_srt_time(end),
            text
        ));
    }

    if count == 0 {
        return Err("未能从 TTML 中解析到可转换的字幕段落。".to_string());
    }
    Ok(output)
}

fn ttml_attr(attrs: &str, name: &str) -> Option<String> {
    let pattern = format!(r#"\b{}\s*=\s*[\"']([^\"']+)[\"']"#, regex::escape(name));
    Regex::new(&pattern)
        .ok()?
        .captures(attrs)
        .and_then(|captures| captures.get(1).map(|value| value.as_str().to_string()))
}

fn parse_ttml_time(value: &str) -> Option<f64> {
    let value = value.trim();
    if let Some(raw) = value.strip_suffix("ms") {
        return raw.trim().parse::<f64>().ok().map(|time| time / 1000.0);
    }
    if let Some(raw) = value.strip_suffix('s') {
        return raw.trim().parse::<f64>().ok();
    }

    let parts = value.split(':').collect::<Vec<_>>();
    let (hours, minutes, seconds) = match parts.as_slice() {
        [minutes, seconds] => (
            0.0,
            minutes.parse::<f64>().ok()?,
            seconds.parse::<f64>().ok()?,
        ),
        [hours, minutes, seconds] => (
            hours.parse::<f64>().ok()?,
            minutes.parse::<f64>().ok()?,
            seconds.parse::<f64>().ok()?,
        ),
        _ => return None,
    };
    Some(hours * 3600.0 + minutes * 60.0 + seconds)
}

fn format_srt_time(seconds: f64) -> String {
    let total_millis = (seconds.max(0.0) * 1000.0).round() as u64;
    let millis = total_millis % 1000;
    let total_seconds = total_millis / 1000;
    let seconds = total_seconds % 60;
    let total_minutes = total_seconds / 60;
    let minutes = total_minutes % 60;
    let hours = total_minutes / 60;
    format!("{hours:02}:{minutes:02}:{seconds:02},{millis:03}")
}

fn clean_ttml_text(value: &str) -> String {
    let with_breaks = ttml_br_regex().replace_all(value, "\n");
    let without_tags = ttml_tag_regex().replace_all(&with_breaks, "");
    decode_xml_entities(&without_tags)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn decode_xml_entities(value: &str) -> String {
    value
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&#10;", "\n")
        .replace("&#xA;", "\n")
}

fn ttml_p_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"(?is)<p\b([^>]*)>(.*?)</p>"#).expect("TTML paragraph regex should be valid")
    })
}

fn ttml_br_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"(?is)<br\s*/?>"#).expect("TTML br regex should be valid"))
}

fn ttml_tag_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"(?is)</?[^>]+>"#).expect("TTML tag regex should be valid"))
}
fn ensure_output_parent_dir(output_path: &str) -> Result<(), String> {
    let Some(parent) = Path::new(output_path).parent() else {
        return Ok(());
    };
    if parent.as_os_str().is_empty() {
        return Ok(());
    }
    fs::create_dir_all(parent).map_err(|err| format!("创建输出目录失败: {err}"))
}

fn comparable_path(path: &str) -> String {
    let raw = Path::new(path.trim());
    let normalized: PathBuf = fs::canonicalize(raw).unwrap_or_else(|_| {
        raw.parent()
            .and_then(|parent| fs::canonicalize(parent).ok())
            .map(|parent| {
                raw.file_name()
                    .map(|name| parent.join(name))
                    .unwrap_or(parent)
            })
            .unwrap_or_else(|| raw.to_path_buf())
    });
    let value = normalized.to_string_lossy().replace('/', "\\");
    #[cfg(windows)]
    {
        value.to_ascii_lowercase()
    }
    #[cfg(not(windows))]
    {
        value
    }
}

fn collect_process_output(logs: &mut Vec<String>, bytes: &[u8]) {
    for line in String::from_utf8_lossy(bytes).lines() {
        let line = line.trim();
        if !line.is_empty() && !is_log_noise(line) {
            logs.push(line.to_string());
        }
    }
}

fn is_log_noise(line: &str) -> bool {
    line.contains("size=N/A") || line.contains("video:0kB")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_path(name: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("csubtitle-subtitle-format-{suffix}-{name}"))
    }

    #[test]
    fn command_uses_ffmpeg_input_and_output() {
        let input = unique_temp_path("in.srt");
        fs::write(&input, b"1\n00:00:00,000 --> 00:00:01,000\nHi\n").unwrap();
        let output = unique_temp_path("out.ass");
        let job = SubtitleFormatJob {
            input_path: input.to_string_lossy().to_string(),
            output_path: output.to_string_lossy().to_string(),
            target_format: "ass".to_string(),
        };
        let command = build_subtitle_format_command("ffmpeg", &job).unwrap();
        assert_eq!(command[0], "ffmpeg");
        assert!(command
            .windows(2)
            .any(|pair| pair == ["-i", job.input_path.as_str()]));
        assert!(command.iter().any(|arg| arg == &job.output_path));
        let _ = fs::remove_file(input);
    }

    #[test]
    fn validate_rejects_output_same_as_input() {
        let input = unique_temp_path("same.srt");
        fs::write(&input, b"fake").unwrap();
        let job = SubtitleFormatJob {
            input_path: input.to_string_lossy().to_string(),
            output_path: input.to_string_lossy().to_string(),
            target_format: "srt".to_string(),
        };
        let err = validate_subtitle_format_job(&job).unwrap_err();
        assert!(err.contains("输出路径不能和输入字幕相同"));
        let _ = fs::remove_file(input);
    }

    #[test]
    fn validate_accepts_ttml_input() {
        let input = unique_temp_path("in.ttml");
        fs::write(&input, br#"<tt><body><div><p begin="00:00:00.000" end="00:00:01.000">Hi</p></div></body></tt>"#).unwrap();
        let output = unique_temp_path("out.srt");
        let job = SubtitleFormatJob {
            input_path: input.to_string_lossy().to_string(),
            output_path: output.to_string_lossy().to_string(),
            target_format: "srt".to_string(),
        };

        validate_subtitle_format_job(&job).unwrap();
        let _ = fs::remove_file(input);
    }

    #[test]
    fn parses_ttml_to_srt() {
        let input = unique_temp_path("parse.ttml");
        fs::write(
            &input,
            br#"<tt><body><div><p begin="00:00:01.500" dur="2s">Hello<br/>world &amp; all</p></div></body></tt>"#,
        )
        .unwrap();

        let srt = ttml_file_to_srt(&input.to_string_lossy()).unwrap();
        assert!(srt.contains("00:00:01,500 --> 00:00:03,500"));
        assert!(srt.contains("Hello\nworld & all"));
        let _ = fs::remove_file(input);
    }

    #[test]
    fn preview_ttml_input_uses_internal_step() {
        let input = unique_temp_path("preview.ttml");
        fs::write(&input, br#"<tt><body><div><p begin="00:00:00.000" end="00:00:01.000">Hi</p></div></body></tt>"#).unwrap();
        let output = unique_temp_path("preview.srt");
        let job = SubtitleFormatJob {
            input_path: input.to_string_lossy().to_string(),
            output_path: output.to_string_lossy().to_string(),
            target_format: "srt".to_string(),
        };

        let command = build_subtitle_format_command("ffmpeg", &job).unwrap();
        assert_eq!(
            command,
            vec![
                "internal-ttml-to-srt",
                job.input_path.as_str(),
                job.output_path.as_str()
            ]
        );
        let _ = fs::remove_file(input);
    }

    #[test]
    fn validate_rejects_ttml_non_srt_target() {
        let input = unique_temp_path("reject.ttml");
        fs::write(&input, br#"<tt><body><div><p begin="00:00:00.000" end="00:00:01.000">Hi</p></div></body></tt>"#).unwrap();
        let output = unique_temp_path("reject.ass");
        let job = SubtitleFormatJob {
            input_path: input.to_string_lossy().to_string(),
            output_path: output.to_string_lossy().to_string(),
            target_format: "ass".to_string(),
        };

        let err = validate_subtitle_format_job(&job).unwrap_err();
        assert!(err.contains("TTML 输入目前仅支持转换为 SRT"));
        let _ = fs::remove_file(input);
    }
}
