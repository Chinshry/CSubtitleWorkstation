use crate::services::{config_store, ffmpeg_locator};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
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
    let command = build_subtitle_format_command(&ffmpeg_path, &job)?;
    ensure_output_parent_dir(&job.output_path)?;

    let output = Command::new(&command[0])
        .args(&command[1..])
        .output()
        .map_err(|err| format!("启动 ffmpeg 失败: {err}"))?;

    let mut logs = Vec::new();
    logs.push(format!("Command: {}", command.join(" ")));
    collect_process_output(&mut logs, &output.stdout);
    collect_process_output(&mut logs, &output.stderr);

    if !output.status.success() {
        logs.push(format!("❌ 字幕格式转换失败: {}", output.status));
        return Err(logs.join("\n"));
    }

    logs.push("Subtitle format conversion completed.".to_string());
    Ok(SubtitleFormatResult {
        output_path: job.output_path,
        logs,
    })
}

fn build_subtitle_format_command(
    ffmpeg_path: &str,
    job: &SubtitleFormatJob,
) -> Result<Vec<String>, String> {
    validate_subtitle_format_job(job)?;
    Ok(vec![
        ffmpeg_path.to_string(),
        "-hide_banner".to_string(),
        "-y".to_string(),
        "-i".to_string(),
        job.input_path.clone(),
        job.output_path.clone(),
    ])
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
        return Err("输入字幕仅支持 ASS / SSA / SRT / VTT / SUB。".to_string());
    }
    let target = normalize_target_format(&job.target_format)?;
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
                "ass" | "ssa" | "srt" | "vtt" | "sub"
            )
        })
        .unwrap_or(false)
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
}
