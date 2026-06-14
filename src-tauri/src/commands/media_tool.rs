use crate::commands::compress::CompressStatus;
use crate::services::{command_builder, config_store, ffmpeg_locator, temp_cleanup};
use crate::{AppState, JobHandle};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use tauri::{AppHandle, Emitter, Manager, State};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaToolJob {
    pub id: String,
    pub mode: MediaToolMode,
    pub input_path: String,
    pub cover_path: Option<String>,
    pub audio_path: Option<String>,
    pub output_path: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MediaToolMode {
    RemuxToMp4,
    ConcatTsToMp4,
    AddCoverToMp4,
    MergeAudioVideo,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TsSegment {
    pub path: String,
    pub name: String,
    pub size_bytes: u64,
}

#[tauri::command]
pub fn list_ts_segments(folder_path: String) -> Result<Vec<TsSegment>, String> {
    let folder = Path::new(folder_path.trim());
    if !folder.is_dir() {
        return Err("请选择包含 TS 分片的文件夹。".to_string());
    }

    let mut segments = Vec::new();
    for entry in fs::read_dir(folder).map_err(|err| format!("读取分片目录失败: {err}"))? {
        let entry = entry.map_err(|err| format!("读取分片目录失败: {err}"))?;
        let path = entry.path();
        if !path.is_file() || !is_ts_like(&path) {
            continue;
        }
        let size_bytes = entry.metadata().map(|meta| meta.len()).unwrap_or(0);
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_string();
        segments.push(TsSegment {
            path: path.to_string_lossy().to_string(),
            name,
            size_bytes,
        });
    }

    segments.sort_by(|a, b| natural_cmp(&a.name, &b.name));
    Ok(segments)
}

#[tauri::command]
pub fn preview_media_tool_command(
    app: AppHandle,
    job: MediaToolJob,
) -> Result<Vec<String>, String> {
    let config = config_store::load(&app)?;
    let status = ffmpeg_locator::detect(&config);
    let ffmpeg_path = status
        .ffmpeg_path
        .ok_or_else(|| "ffmpeg is not configured.".to_string())?;
    build_media_tool_command(&ffmpeg_path, &job, "<temp concat list>")
}

#[tauri::command]
pub async fn start_media_tool(
    app: AppHandle,
    state: State<'_, AppState>,
    job: MediaToolJob,
) -> Result<(), String> {
    {
        let mut preparing = state
            .preparing_jobs
            .lock()
            .map_err(|_| "Preparing job state is poisoned.".to_string())?;
        preparing.insert(job.id.clone());
    }

    let job_id = job.id.clone();
    let app_for_cleanup = app.clone();
    let result =
        match tauri::async_runtime::spawn_blocking(move || start_media_tool_blocking(app, job))
            .await
        {
            Ok(result) => result,
            Err(err) => Err(format!("Media tool startup task failed: {err}")),
        };

    finish_preparing_job(&app_for_cleanup, &job_id);
    result
}

#[tauri::command]
pub fn cancel_media_tool(
    app: AppHandle,
    state: State<AppState>,
    job_id: String,
) -> Result<(), String> {
    let handle = {
        let mut jobs = state
            .jobs
            .lock()
            .map_err(|_| "Job state is poisoned.".to_string())?;
        jobs.remove(&job_id)
    };

    let Some(mut handle) = handle else {
        let is_preparing = {
            let preparing = state
                .preparing_jobs
                .lock()
                .map_err(|_| "Preparing job state is poisoned.".to_string())?;
            preparing.contains(&job_id)
        };
        if is_preparing {
            let mut cancelled = state
                .cancelled_jobs
                .lock()
                .map_err(|_| "Cancelled jobs state is poisoned.".to_string())?;
            cancelled.insert(job_id);
            let _ = app.emit("media-tool-log", "已收到取消请求。");
            return Ok(());
        }
        return Err("No running media tool job matched the requested id.".to_string());
    };

    {
        let mut cancelled = state
            .cancelled_jobs
            .lock()
            .map_err(|_| "Cancelled jobs state is poisoned.".to_string())?;
        cancelled.insert(job_id);
    }

    if let Some(mut stdin) = handle.stdin.take() {
        let _ = stdin.write_all(b"q\n");
        let _ = stdin.flush();
    }

    let pid = handle.pid;
    let _ = app.emit(
        "media-tool-log",
        "已发送取消信号：等待 ffmpeg 写入文件尾后退出。",
    );
    thread::spawn(move || {
        thread::sleep(std::time::Duration::from_secs(10));
        let _ = kill_process_tree(pid);
    });
    Ok(())
}

fn start_media_tool_blocking(app: AppHandle, job: MediaToolJob) -> Result<(), String> {
    validate_media_job(&job)?;

    let config = config_store::load(&app)?;
    let status = ffmpeg_locator::detect(&config);
    if !status.available {
        return Err(status
            .message
            .unwrap_or_else(|| "No available ffmpeg was found.".to_string()));
    }
    let ffmpeg_path = status
        .ffmpeg_path
        .ok_or_else(|| "ffmpeg is not configured.".to_string())?;

    let duration_seconds = match job.mode {
        MediaToolMode::RemuxToMp4
        | MediaToolMode::AddCoverToMp4
        | MediaToolMode::MergeAudioVideo => {
            command_builder::inspect_video(&ffmpeg_path, &job.input_path)
                .ok()
                .and_then(|info| info.duration_seconds)
        }
        MediaToolMode::ConcatTsToMp4 => None,
    };

    let temp_dir = temp_cleanup::filter_temp_dir(&app)?.join(sanitize_job_id(&job.id));
    fs::create_dir_all(&temp_dir).map_err(|err| format!("创建临时目录失败: {err}"))?;

    let concat_list = if matches!(job.mode, MediaToolMode::ConcatTsToMp4) {
        let segments = list_ts_segments(job.input_path.clone())?;
        if segments.is_empty() {
            return Err("所选文件夹中没有可合并的 TS / M2TS / MTS 分片。".to_string());
        }
        let list_path = temp_dir.join("concat-list.txt");
        write_concat_list(&list_path, &segments)?;
        let _ = app.emit(
            "media-tool-log",
            format!("已生成分片列表：{} 个文件。", segments.len()),
        );
        Some(list_path.to_string_lossy().to_string())
    } else {
        None
    };

    let concat_list_arg = concat_list.as_deref().unwrap_or("");
    let command = build_media_tool_command(&ffmpeg_path, &job, concat_list_arg)?;

    if is_job_cancelled(&app, &job.id) {
        cleanup_temp_dir(&app, &temp_dir);
        clear_cancelled_job(&app, &job.id);
        let _ = app.emit("media-tool-log", "Media tool exited.");
        return Ok(());
    }

    app.emit("media-tool-log", format!("Command: {}", command.join(" ")))
        .map_err(|err| format!("Failed to emit log event: {err}"))?;

    ensure_output_parent_dir(&job.output_path)?;

    let mut builder = Command::new(&command[0]);
    builder
        .args(&command[1..])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        builder.creation_flags(CREATE_NO_WINDOW);
    }

    let mut child = builder
        .spawn()
        .map_err(|err| format!("Failed to start ffmpeg: {err}"))?;
    let stderr = child.stderr.take();
    let stdout = child.stdout.take();
    let stdin = child.stdin.take();
    let pid = child.id();

    {
        let state = app.state::<AppState>();
        let mut jobs = state
            .jobs
            .lock()
            .map_err(|_| "Job state is poisoned.".to_string())?;
        jobs.insert(job.id.clone(), JobHandle { pid, stdin });
    }

    if let Some(stderr) = stderr {
        let app_for_stderr = app.clone();
        let job_id = job.id.clone();
        thread::spawn(move || {
            read_lines_split(stderr, |line| {
                emit_log_and_progress(&app_for_stderr, &job_id, line, duration_seconds);
            });
        });
    }

    if let Some(stdout) = stdout {
        let app_for_stdout = app.clone();
        let job_id = job.id.clone();
        thread::spawn(move || {
            read_lines_split(stdout, |line| {
                emit_log_and_progress(&app_for_stdout, &job_id, line, duration_seconds);
            });
        });
    }

    let app_for_wait = app.clone();
    let job_id_for_wait = job.id.clone();
    thread::spawn(move || {
        let status = child.wait();
        let was_cancelled = if let Some(state) = app_for_wait.try_state::<AppState>() {
            let mut jobs = if let Ok(j) = state.jobs.lock() {
                j
            } else {
                return;
            };
            jobs.remove(&job_id_for_wait);

            let mut cancelled = if let Ok(c) = state.cancelled_jobs.lock() {
                c
            } else {
                return;
            };
            let was_cancelled = cancelled.contains(&job_id_for_wait);
            cancelled.remove(&job_id_for_wait);
            was_cancelled
        } else {
            false
        };

        match status {
            Ok(status) if status.success() => {
                if was_cancelled {
                    let _ = app_for_wait.emit("media-tool-log", "Media tool exited.");
                } else {
                    let _ = app_for_wait.emit(
                        "media-tool-status",
                        CompressStatus {
                            job_id: job_id_for_wait.clone(),
                            status_line: "Media tool completed.".to_string(),
                            percent: Some(100.0),
                            current_seconds: duration_seconds,
                            duration_seconds,
                            size_kb: None,
                            bitrate_kbps: None,
                            speed: None,
                            fps: None,
                        },
                    );
                    let _ = app_for_wait.emit("media-tool-log", "Media tool completed.");
                }
            }
            Ok(status) => {
                let _ = app_for_wait.emit(
                    "media-tool-log",
                    format!("❌ Media tool failed with exit code: {status}"),
                );
                let _ = app_for_wait.emit(
                    "media-tool-log",
                    "如果错误与 codec、tag、header 或 muxer 相关，通常表示当前音视频流不兼容 MP4 容器；请到压制页重新编码后再输出 MP4。",
                );
            }
            Err(err) => {
                let _ = app_for_wait.emit(
                    "media-tool-log",
                    format!("❌ Failed to wait for ffmpeg: {err}"),
                );
            }
        }
        cleanup_temp_dir(&app_for_wait, &temp_dir);
    });

    Ok(())
}

fn build_media_tool_command(
    ffmpeg_path: &str,
    job: &MediaToolJob,
    concat_list_path: &str,
) -> Result<Vec<String>, String> {
    validate_media_job(job)?;
    let mut args = vec![ffmpeg_path.to_string(), "-hide_banner".to_string()];
    match job.mode {
        MediaToolMode::RemuxToMp4 => {
            args.extend(["-i".to_string(), job.input_path.clone()]);
            args.extend([
                "-map".to_string(),
                "0".to_string(),
                "-c".to_string(),
                "copy".to_string(),
            ]);
            if needs_aac_adtstoasc(&job.input_path) {
                args.extend(["-bsf:a".to_string(), "aac_adtstoasc".to_string()]);
            }
        }
        MediaToolMode::ConcatTsToMp4 => {
            if concat_list_path.trim().is_empty() {
                return Err("TS 分片合并缺少 concat list。".to_string());
            }
            args.extend([
                "-f".to_string(),
                "concat".to_string(),
                "-safe".to_string(),
                "0".to_string(),
                "-i".to_string(),
                concat_list_path.to_string(),
                "-c".to_string(),
                "copy".to_string(),
                "-bsf:a".to_string(),
                "aac_adtstoasc".to_string(),
            ]);
        }
        MediaToolMode::AddCoverToMp4 => {
            let cover_path = job
                .cover_path
                .as_deref()
                .ok_or_else(|| "请选择封面图片。".to_string())?;
            args.extend([
                "-i".to_string(),
                job.input_path.clone(),
                "-i".to_string(),
                cover_path.to_string(),
                "-map".to_string(),
                "0".to_string(),
                "-map".to_string(),
                "1:v:0".to_string(),
                "-c".to_string(),
                "copy".to_string(),
                "-disposition:v:1".to_string(),
                "attached_pic".to_string(),
            ]);
        }
        MediaToolMode::MergeAudioVideo => {
            let audio_path = job
                .audio_path
                .as_deref()
                .ok_or_else(|| "请选择音频文件。".to_string())?;
            args.extend([
                "-i".to_string(),
                job.input_path.clone(),
                "-i".to_string(),
                audio_path.to_string(),
                "-map".to_string(),
                "0:v:0".to_string(),
                "-map".to_string(),
                "1:a:0".to_string(),
                "-c".to_string(),
                "copy".to_string(),
                "-shortest".to_string(),
            ]);
        }
    }
    args.extend([job.output_path.clone(), "-y".to_string()]);
    Ok(args)
}

fn validate_media_job(job: &MediaToolJob) -> Result<(), String> {
    if job.id.trim().is_empty() {
        return Err("任务 ID 不能为空。".to_string());
    }
    if job.input_path.trim().is_empty() {
        return Err("输入路径不能为空。".to_string());
    }
    if job.output_path.trim().is_empty() {
        return Err("输出路径不能为空。".to_string());
    }
    if !job.output_path.to_ascii_lowercase().ends_with(".mp4") {
        return Err("封装转换第一版只输出 MP4 文件。".to_string());
    }
    if output_matches_source(job) {
        return Err("输出路径不能和输入文件相同。".to_string());
    }
    match job.mode {
        MediaToolMode::RemuxToMp4 => {
            if !Path::new(&job.input_path).is_file() {
                return Err("请选择要转为 MP4 封装的视频文件。".to_string());
            }
        }
        MediaToolMode::ConcatTsToMp4 => {
            if !Path::new(&job.input_path).is_dir() {
                return Err("请选择包含 TS 分片的文件夹。".to_string());
            }
        }
        MediaToolMode::AddCoverToMp4 => {
            if !Path::new(&job.input_path).is_file() {
                return Err("请选择要添加封面的 MP4 视频文件。".to_string());
            }
            let cover_path = job.cover_path.as_deref().unwrap_or("").trim();
            if cover_path.is_empty() {
                return Err("请选择封面图片。".to_string());
            }
            if !Path::new(cover_path).is_file() || !is_cover_image(Path::new(cover_path)) {
                return Err("封面图片仅支持 JPG / JPEG / PNG。".to_string());
            }
            if !matches!(
                Path::new(&job.input_path)
                    .extension()
                    .and_then(|value| value.to_str())
                    .map(|ext| ext.to_ascii_lowercase())
                    .as_deref(),
                Some("mp4") | Some("m4v") | Some("mov")
            ) {
                return Err("添加封面第一版仅支持 MP4 / M4V / MOV 输入。".to_string());
            }
        }
        MediaToolMode::MergeAudioVideo => {
            if !Path::new(&job.input_path).is_file() {
                return Err("请选择要合并音频的视频文件。".to_string());
            }
            let audio_path = job.audio_path.as_deref().unwrap_or("").trim();
            if audio_path.is_empty() {
                return Err("请选择音频来源文件。".to_string());
            }
            if !Path::new(audio_path).is_file() || !is_audio_source_file(Path::new(audio_path)) {
                return Err("音频来源仅支持常见音频文件，或带音轨的视频文件。".to_string());
            }
        }
    }
    Ok(())
}

fn output_matches_source(job: &MediaToolJob) -> bool {
    let output = comparable_path(&job.output_path);
    if output.is_empty() {
        return false;
    }
    let mut sources = vec![job.input_path.as_str()];
    if let Some(path) = job.cover_path.as_deref() {
        sources.push(path);
    }
    if let Some(path) = job.audio_path.as_deref() {
        sources.push(path);
    }
    sources
        .into_iter()
        .filter(|path| !path.trim().is_empty())
        .any(|path| comparable_path(path) == output)
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

fn write_concat_list(path: &Path, segments: &[TsSegment]) -> Result<(), String> {
    let mut content = String::new();
    for segment in segments {
        content.push_str("file '");
        content.push_str(&segment.path.replace('\\', "/").replace('\'', "'\\''"));
        content.push_str("'\n");
    }
    fs::write(path, content).map_err(|err| format!("写入 concat list 失败: {err}"))
}

fn needs_aac_adtstoasc(input_path: &str) -> bool {
    Path::new(input_path)
        .extension()
        .and_then(|value| value.to_str())
        .map(|ext| matches!(ext.to_ascii_lowercase().as_str(), "ts" | "m2ts" | "mts"))
        .unwrap_or(false)
}

fn is_ts_like(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .map(|ext| matches!(ext.to_ascii_lowercase().as_str(), "ts" | "m2ts" | "mts"))
        .unwrap_or(false)
}

fn is_cover_image(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .map(|ext| matches!(ext.to_ascii_lowercase().as_str(), "jpg" | "jpeg" | "png"))
        .unwrap_or(false)
}

fn is_audio_file(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .map(|ext| {
            matches!(
                ext.to_ascii_lowercase().as_str(),
                "m4a" | "aac" | "mp3" | "wav" | "flac" | "ac3" | "eac3" | "opus" | "ogg"
            )
        })
        .unwrap_or(false)
}

fn is_video_file(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .map(|ext| {
            matches!(
                ext.to_ascii_lowercase().as_str(),
                "mp4"
                    | "mkv"
                    | "mov"
                    | "ts"
                    | "m4v"
                    | "flv"
                    | "avi"
                    | "webm"
                    | "wmv"
                    | "mpg"
                    | "mpeg"
                    | "3gp"
                    | "3g2"
                    | "rm"
                    | "rmvb"
                    | "vob"
                    | "mts"
                    | "m2ts"
                    | "ogv"
                    | "ogg"
                    | "divx"
                    | "asf"
                    | "f4v"
                    | "hevc"
                    | "h265"
            )
        })
        .unwrap_or(false)
}

fn is_audio_source_file(path: &Path) -> bool {
    is_audio_file(path) || is_video_file(path)
}

fn natural_cmp(a: &str, b: &str) -> Ordering {
    let mut ia = 0;
    let mut ib = 0;
    let ba = a.as_bytes();
    let bb = b.as_bytes();
    while ia < ba.len() && ib < bb.len() {
        if ba[ia].is_ascii_digit() && bb[ib].is_ascii_digit() {
            let sa = ia;
            let sb = ib;
            while ia < ba.len() && ba[ia].is_ascii_digit() {
                ia += 1;
            }
            while ib < bb.len() && bb[ib].is_ascii_digit() {
                ib += 1;
            }
            let na = a[sa..ia].trim_start_matches('0');
            let nb = b[sb..ib].trim_start_matches('0');
            let na = if na.is_empty() { "0" } else { na };
            let nb = if nb.is_empty() { "0" } else { nb };
            match na.len().cmp(&nb.len()).then_with(|| na.cmp(nb)) {
                Ordering::Equal => continue,
                other => return other,
            }
        }
        let ca = ba[ia].to_ascii_lowercase();
        let cb = bb[ib].to_ascii_lowercase();
        match ca.cmp(&cb) {
            Ordering::Equal => {
                ia += 1;
                ib += 1;
            }
            other => return other,
        }
    }
    ba.len().cmp(&bb.len())
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

fn is_job_cancelled(app: &AppHandle, job_id: &str) -> bool {
    app.try_state::<AppState>()
        .and_then(|state| {
            state
                .cancelled_jobs
                .lock()
                .ok()
                .map(|cancelled| cancelled.contains(job_id))
        })
        .unwrap_or(false)
}

fn clear_cancelled_job(app: &AppHandle, job_id: &str) {
    if let Some(state) = app.try_state::<AppState>() {
        if let Ok(mut cancelled) = state.cancelled_jobs.lock() {
            cancelled.remove(job_id);
        }
    }
}

fn finish_preparing_job(app: &AppHandle, job_id: &str) {
    if let Some(state) = app.try_state::<AppState>() {
        if let Ok(mut preparing) = state.preparing_jobs.lock() {
            preparing.remove(job_id);
        }
    }
}

fn cleanup_temp_dir(app: &AppHandle, dir: &Path) {
    if !dir.exists() {
        return;
    }
    if let Err(err) = fs::remove_dir_all(dir) {
        let _ = app.emit(
            "media-tool-log",
            format!("Warning: failed to cleanup temp files: {err}"),
        );
    }
}

fn sanitize_job_id(job_id: &str) -> String {
    job_id
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '-' || *ch == '_')
        .collect()
}

fn read_lines_split<R: Read, F: FnMut(&str)>(mut reader: R, mut on_line: F) {
    let mut buf = [0u8; 4096];
    let mut acc: Vec<u8> = Vec::with_capacity(512);
    loop {
        match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                for &b in &buf[..n] {
                    if b == b'\r' || b == b'\n' {
                        if !acc.is_empty() {
                            let line = String::from_utf8_lossy(&acc).to_string();
                            on_line(line.trim_end_matches(['\r', '\n']));
                            acc.clear();
                        }
                    } else {
                        acc.push(b);
                    }
                }
            }
            Err(_) => break,
        }
    }
    if !acc.is_empty() {
        let line = String::from_utf8_lossy(&acc).to_string();
        on_line(line.trim_end_matches(['\r', '\n']));
    }
}

fn emit_log_and_progress(app: &AppHandle, job_id: &str, line: &str, duration_seconds: Option<f64>) {
    let trimmed = line.trim_end();
    if command_builder::is_ffmpeg_progress_line(trimmed) {
        let fields = command_builder::parse_ffmpeg_progress(trimmed);
        let percent = match (fields.current_seconds, duration_seconds) {
            (Some(cur), Some(dur)) if dur > 0.0 => Some((cur / dur * 100.0).clamp(0.0, 100.0)),
            _ => None,
        };
        let _ = app.emit(
            "media-tool-status",
            CompressStatus {
                job_id: job_id.to_string(),
                status_line: trimmed.to_string(),
                percent,
                current_seconds: fields.current_seconds,
                duration_seconds,
                size_kb: fields.size_kb,
                bitrate_kbps: fields.bitrate_kbps,
                speed: fields.speed,
                fps: fields.fps,
            },
        );
        return;
    }

    if is_log_noise(trimmed) {
        return;
    }

    let _ = app.emit("media-tool-log", trimmed.to_string());
}

fn is_log_noise(line: &str) -> bool {
    const SUBSTRS: &[&str] = &[
        "Past duration",
        "Last message repeated",
        "deprecated pixel format",
    ];
    let trimmed = line.trim_start();
    SUBSTRS.iter().any(|s| trimmed.contains(s))
}

#[cfg(windows)]
fn kill_process_tree(pid: u32) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let mut cmd = Command::new("taskkill");
    cmd.args(["/F", "/T", "/PID"]).arg(pid.to_string());
    cmd.creation_flags(CREATE_NO_WINDOW);
    let output = cmd
        .output()
        .map_err(|err| format!("启动 taskkill 失败: {err}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not found") || stderr.contains("找不到") || stderr.contains("不存在")
        {
            return Ok(());
        }
        return Err(format!("taskkill 失败: {stderr}"));
    }
    Ok(())
}

#[cfg(not(windows))]
fn kill_process_tree(pid: u32) -> Result<(), String> {
    let status = Command::new("kill")
        .arg("-9")
        .arg(pid.to_string())
        .status()
        .map_err(|err| format!("启动 kill 失败: {err}"))?;
    if !status.success() {
        return Ok(());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn job(mode: MediaToolMode, input_path: &Path) -> MediaToolJob {
        MediaToolJob {
            id: "job-1".to_string(),
            mode,
            input_path: input_path.to_string_lossy().to_string(),
            cover_path: None,
            audio_path: None,
            output_path: r"E:\out.mp4".to_string(),
        }
    }

    fn cover_job(input_path: &Path, cover_path: &Path) -> MediaToolJob {
        MediaToolJob {
            id: "job-cover".to_string(),
            mode: MediaToolMode::AddCoverToMp4,
            input_path: input_path.to_string_lossy().to_string(),
            cover_path: Some(cover_path.to_string_lossy().to_string()),
            audio_path: None,
            output_path: r"E:\out.mp4".to_string(),
        }
    }

    fn merge_job(input_path: &Path, audio_path: &Path) -> MediaToolJob {
        MediaToolJob {
            id: "job-merge".to_string(),
            mode: MediaToolMode::MergeAudioVideo,
            input_path: input_path.to_string_lossy().to_string(),
            cover_path: None,
            audio_path: Some(audio_path.to_string_lossy().to_string()),
            output_path: r"E:\out.mp4".to_string(),
        }
    }

    fn unique_temp_path(name: &str) -> std::path::PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("csubtitle-media-tool-{suffix}-{name}"))
    }

    #[test]
    fn remux_ts_adds_aac_bitstream_filter() {
        let input = unique_temp_path("in.ts");
        fs::write(&input, b"fake").unwrap();
        let command =
            build_media_tool_command("ffmpeg", &job(MediaToolMode::RemuxToMp4, &input), "")
                .unwrap();
        assert!(command.iter().any(|arg| arg == "-bsf:a"));
        assert!(command.iter().any(|arg| arg == "aac_adtstoasc"));
        let _ = fs::remove_file(input);
    }

    #[test]
    fn remux_mkv_only_stream_copies() {
        let input = unique_temp_path("in.mkv");
        fs::write(&input, b"fake").unwrap();
        let command =
            build_media_tool_command("ffmpeg", &job(MediaToolMode::RemuxToMp4, &input), "")
                .unwrap();
        assert!(command.windows(2).any(|pair| pair == ["-c", "copy"]));
        assert!(!command.iter().any(|arg| arg == "-bsf:a"));
        let _ = fs::remove_file(input);
    }

    #[test]
    fn concat_uses_concat_demuxer() {
        let input_dir = unique_temp_path("segments");
        fs::create_dir_all(&input_dir).unwrap();
        let command = build_media_tool_command(
            "ffmpeg",
            &job(MediaToolMode::ConcatTsToMp4, &input_dir),
            r"E:\temp\list.txt",
        )
        .unwrap();
        assert!(command.windows(2).any(|pair| pair == ["-f", "concat"]));
        assert!(command.windows(2).any(|pair| pair == ["-safe", "0"]));
        assert!(command.windows(2).any(|pair| pair == ["-c", "copy"]));
        let _ = fs::remove_dir_all(input_dir);
    }

    #[test]
    fn natural_sort_handles_numbered_segments() {
        let mut names = vec!["10.ts", "2.ts", "1.ts", "001.ts"];
        names.sort_by(|a, b| natural_cmp(a, b));
        assert_eq!(names, vec!["1.ts", "001.ts", "2.ts", "10.ts"]);
    }

    #[test]
    fn add_cover_maps_image_as_attached_picture() {
        let input = unique_temp_path("in.mp4");
        let cover = unique_temp_path("cover.jpg");
        fs::write(&input, b"fake").unwrap();
        fs::write(&cover, b"fake").unwrap();
        let command = build_media_tool_command("ffmpeg", &cover_job(&input, &cover), "").unwrap();
        assert!(command.windows(2).any(|pair| pair == ["-map", "0"]));
        assert!(command.windows(2).any(|pair| pair == ["-map", "1:v:0"]));
        assert!(command
            .windows(2)
            .any(|pair| pair == ["-disposition:v:1", "attached_pic"]));
        assert!(command.windows(2).any(|pair| pair == ["-c", "copy"]));
        let _ = fs::remove_file(input);
        let _ = fs::remove_file(cover);
    }

    #[test]
    fn merge_audio_video_maps_video_and_external_audio() {
        let input = unique_temp_path("in.mp4");
        let audio = unique_temp_path("audio.m4a");
        fs::write(&input, b"fake").unwrap();
        fs::write(&audio, b"fake").unwrap();
        let command = build_media_tool_command("ffmpeg", &merge_job(&input, &audio), "").unwrap();
        assert!(command.windows(2).any(|pair| pair == ["-map", "0:v:0"]));
        assert!(command.windows(2).any(|pair| pair == ["-map", "1:a:0"]));
        assert!(command.windows(2).any(|pair| pair == ["-c", "copy"]));
        assert!(command.iter().any(|arg| arg == "-shortest"));
        let _ = fs::remove_file(input);
        let _ = fs::remove_file(audio);
    }

    #[test]
    fn merge_audio_video_accepts_video_as_audio_source() {
        let input = unique_temp_path("in.mp4");
        let audio_source = unique_temp_path("audio-source.mkv");
        fs::write(&input, b"fake").unwrap();
        fs::write(&audio_source, b"fake").unwrap();
        let command =
            build_media_tool_command("ffmpeg", &merge_job(&input, &audio_source), "").unwrap();
        assert!(command.windows(2).any(|pair| pair == ["-map", "0:v:0"]));
        assert!(command.windows(2).any(|pair| pair == ["-map", "1:a:0"]));
        let _ = fs::remove_file(input);
        let _ = fs::remove_file(audio_source);
    }

    #[test]
    fn validate_rejects_output_same_as_input() {
        let input = unique_temp_path("in.mp4");
        fs::write(&input, b"fake").unwrap();
        let mut job = job(MediaToolMode::RemuxToMp4, &input);
        job.output_path = input.to_string_lossy().to_string();
        let err = validate_media_job(&job).unwrap_err();
        assert!(err.contains("输出路径不能和输入文件相同"));
        let _ = fs::remove_file(input);
    }
}
