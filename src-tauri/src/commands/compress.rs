use crate::models::compress_job::CompressJob;
use crate::services::{
    avs_detector, avs_workspace, command_builder, config_store, encoder_detector, ffmpeg_locator,
    subtitle_analyzer, temp_cleanup,
};
use crate::{AppState, JobHandle};
use serde::Serialize;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompressStatus {
    pub job_id: String,
    pub status_line: String,
    pub percent: Option<f64>,
    pub current_seconds: Option<f64>,
    pub duration_seconds: Option<f64>,
    pub size_kb: Option<u64>,
    pub bitrate_kbps: Option<f64>,
    pub speed: Option<f64>,
    pub fps: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleAnalysisResult {
    pub has_effects: bool,
    pub detected_tags: Vec<String>,
    /// ASS [Script Info] 段的 YCbCr Matrix 原始值，前端用于与视频 color_space/color_range 比对
    pub ass_matrix: Option<String>,
    pub missing_img_paths: Vec<subtitle_analyzer::SubtitleResourceIssue>,
    pub missing_fonts: Vec<subtitle_analyzer::SubtitleFontIssue>,
    pub missing_styles: Vec<subtitle_analyzer::SubtitleStyleIssue>,
    /// ASS [Events] 段 Effect 字段命中的 Banner 滚动横幅（携带行号 / 原始字符串）
    pub banner_hits: Vec<subtitle_analyzer::SubtitleBannerHit>,
}

#[tauri::command]
pub fn preview_ffmpeg_command(app: AppHandle, job: CompressJob) -> Result<Vec<String>, String> {
    let config = config_store::load(&app)?;
    let status = ffmpeg_locator::detect(&config);
    let ffmpeg_path = status
        .ffmpeg_path
        .ok_or_else(|| "ffmpeg is not configured.".to_string())?;
    command_builder::build_preview(&ffmpeg_path, &job)
}

#[tauri::command]
pub fn analyze_subtitle(subtitle_path: String) -> Result<SubtitleAnalysisResult, String> {
    let analysis = subtitle_analyzer::analyze_subtitle(&subtitle_path)?;
    Ok(SubtitleAnalysisResult {
        has_effects: analysis.has_effects,
        detected_tags: analysis.detected_tags,
        ass_matrix: analysis.ass_matrix,
        missing_img_paths: analysis.missing_img_paths,
        missing_fonts: analysis.missing_fonts,
        missing_styles: analysis.missing_styles,
        banner_hits: analysis.banner_hits,
    })
}

#[tauri::command]
pub fn validate_output_parent_dir(output_path: String) -> Result<(), String> {
    let trimmed = output_path.trim();
    if trimmed.is_empty() {
        return Err("输出路径不能为空".to_string());
    }

    let output = Path::new(trimmed);
    let parent = if trimmed.ends_with('\\') || trimmed.ends_with('/') || output.extension().is_none()
    {
        output
    } else {
        output
            .parent()
            .ok_or_else(|| "无法识别输出目录".to_string())?
    };

    if !parent.exists() {
        return Err(format!("输出目录不存在: {}", parent.display()));
    }
    if !parent.is_dir() {
        return Err(format!("输出目录不是文件夹: {}", parent.display()));
    }
    Ok(())
}

#[tauri::command]
pub fn start_compress(
    app: AppHandle,
    state: State<AppState>,
    job: CompressJob,
) -> Result<(), String> {
    // 验证编码器是否在当前平台支持
    if !encoder_detector::is_encoder_supported(&job.encoder) {
        return Err(format!("编码器 {} 在当前平台不支持。", job.encoder));
    }

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

    if !job.use_avs && !job.subtitle_path.trim().is_empty() && !status.subtitle_filter_available {
        return Err(
            "当前 ffmpeg 缺少 subtitles/libass filter，无法使用 ffmpeg filter 模式压制 ASS 字幕。Windows 可改用 AVS 兼容模式，或选择包含 libass/subtitles filter 的 ffmpeg full 构建。"
                .to_string(),
        );
    }

    // inspect_video 失败不再静默——把错误透出到日志，便于排查
    let video_info = match command_builder::inspect_video(&ffmpeg_path, &job.video_path) {
        Ok(info) => info,
        Err(err) => {
            let _ = app.emit("compress-log", format!("inspect_video failed: {err}"));
            Default::default()
        }
    };
    if video_info.duration_seconds.is_none() {
        let _ = app.emit(
            "compress-log",
            "Warning: failed to detect video duration; progress percent will stay 0.".to_string(),
        );
    }
    let duration_seconds = video_info.duration_seconds;

    // 字幕统一 stage 到 ASCII 临时路径：
    // - 非 AVS 模式：避开 ffmpeg subtitles filter 对路径中半角单引号的解析问题
    // - AVS 模式：VSFilterMod 的 TextSubMod 内部用 Win32 ANSI API 打开文件，含
    //   中文 / `@@` 等非 ASCII 字符的路径会报 "Can't open"，必须先复制到 ASCII 路径
    let mut command_job = job.clone();
    let temp_dir = job_temp_dir(&app, &command_job.id)?;
    let subtitle_temp_path = if !command_job.subtitle_path.trim().is_empty() {
        Some(stage_subtitle_to_ascii(
            &temp_dir,
            &command_job.subtitle_path,
        )?)
    } else {
        None
    };
    if let Some(path) = &subtitle_temp_path {
        command_job.subtitle_path = path.clone();
    }

    // AVS 模式：先检测环境，再写入 input.avs 临时脚本
    let avs_script_path: Option<String> = if command_job.use_avs {
        let avs_status = avs_detector::detect(Some(&ffmpeg_path));
        if !avs_status.available {
            return Err(avs_status
                .message
                .unwrap_or_else(|| "AVS 环境不可用".to_string()));
        }
        let workspace = avs_workspace::resolve(&app)?;
        let script = avs_workspace::build_avs_script(
            &workspace.vsfiltermod_path(),
            &workspace.lsmashsource_path(),
            &command_job.video_path,
            &command_job.subtitle_path,
        );
        let script_path = avs_workspace::write_script(&workspace, &script)?;
        let path_str = script_path.to_string_lossy().to_string();
        let _ = app.emit("compress-log", format!("AVS script written: {path_str}"));
        Some(path_str)
    } else {
        None
    };

    let command = command_builder::build_with_options(
        &ffmpeg_path,
        &command_job,
        avs_script_path.as_deref(),
    )?;

    app.emit("compress-log", format!("Command: {}", command.join(" ")))
        .map_err(|err| format!("Failed to emit log event: {err}"))?;

    if command.len() < 2 {
        return Err("Generated ffmpeg command is incomplete.".to_string());
    }
    ensure_output_parent_dir(&command)?;

    // 安全 spawn：
    // - stdin 用 piped，cancel_compress 通过写 b"q\n" 让 ffmpeg 优雅退出（写文件尾，部分输出可播放）。
    // - Windows 下禁止弹出黑窗。
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
            // ffmpeg 在 stderr 用 \r 刷新进度行，必须按 \r/\n 同时拆分
            read_lines_split(stderr, |line| {
                emit_log_and_progress(&app_for_stderr, &job_id, line, duration_seconds);
            });
        });
    }

    if let Some(stdout) = stdout {
        let app_for_stdout = app.clone();
        let job_id = job.id.clone();
        thread::spawn(move || {
            // -progress pipe:1 输出本身是 \n 分隔，但统一走 split 实现更稳
            read_lines_split(stdout, |line| {
                emit_log_and_progress(&app_for_stdout, &job_id, line, duration_seconds);
            });
        });
    }

    let app_for_wait = app.clone();
    let job_id_for_wait = job.id.clone();
    let duration_for_wait = duration_seconds;
    thread::spawn(move || {
        // wait 线程独占 child，不再放进 Mutex；cancel_compress 通过 pid + OS 调用终止进程，
        // 不会与本线程互相阻塞，彻底避开死锁。
        let status = child.wait();

        // wait 返回 → 进程已结束（正常退出或被 cancel kill 掉），从 jobs 表中移除该 id
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
                // 如果任务被取消，不发送 100% 完成状态，让前端保持当前进度
                if was_cancelled {
                    let _ = app_for_wait.emit("compress-log", "Compression exited.");
                } else {
                    // ffmpeg 最后一行 progress 通常 current_seconds 比 duration 略短，
                    // 前端百分比会卡在 99.9。这里在正常退出后补发一次 100% 状态，
                    // 把进度推到完成态；前端无需识别"完成事件"特殊类型。
                    let _ = app_for_wait.emit(
                        "compress-status",
                        CompressStatus {
                            job_id: job_id_for_wait.clone(),
                            status_line: "Compression completed.".to_string(),
                            percent: Some(100.0),
                            current_seconds: duration_for_wait,
                            duration_seconds: duration_for_wait,
                            size_kb: None,
                            bitrate_kbps: None,
                            speed: None,
                            fps: None,
                        },
                    );
                    let _ = app_for_wait.emit("compress-log", "Compression completed.");
                }
            }
            Ok(status) => {
                let _ = app_for_wait.emit(
                    "compress-log",
                    format!("❌ Compression failed with exit code: {status}"),
                );
            }
            Err(err) => {
                let _ = app_for_wait.emit(
                    "compress-log",
                    format!("❌ Failed to wait for ffmpeg: {err}"),
                );
            }
        }
        cleanup_temp_dir(&app_for_wait, &temp_dir);
    });

    Ok(())
}

#[tauri::command]
pub fn cancel_compress(
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
        return Err("No running job matched the requested id.".to_string());
    };

    let pid = handle.pid;

    // 标记任务为已取消，wait 线程会检查此标记避免发送 100% 完成状态
    {
        let mut cancelled = state
            .cancelled_jobs
            .lock()
            .map_err(|_| "Cancelled jobs state is poisoned.".to_string())?;
        cancelled.insert(job_id.clone());
    }

    // 优雅退出路径：通过 stdin 给 ffmpeg 写 'q'，让它停止编码、写完文件尾、自然退出。
    // 这与命令行 Ctrl+C 等效，已编码部分仍然可播放。
    let mut graceful = false;
    if let Some(mut stdin) = handle.stdin.take() {
        if stdin.write_all(b"q\n").is_ok() && stdin.flush().is_ok() {
            graceful = true;
        }
        // 显式 drop 关闭 stdin，向 ffmpeg 传递 EOF
        drop(stdin);
    }

    if graceful {
        let _ = app.emit(
            "compress-log",
            "已发送取消信号：等待 ffmpeg 写入文件尾后退出（已压制部分仍可播放）。",
        );
        // 兜底：超时仍未退出则强制结束，避免僵尸进程。
        // kill_process_tree 对"找不到 pid"已做幂等处理，进程正常退出后调用是安全的。
        thread::spawn(move || {
            thread::sleep(Duration::from_secs(10));
            let _ = kill_process_tree(pid);
        });
        Ok(())
    } else {
        let _ = app.emit(
            "compress-log",
            "无法通过 stdin 通知 ffmpeg，将直接结束进程（输出文件可能不完整）。",
        );
        kill_process_tree(pid)
    }
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
        // pid 已退出 / 不存在时 taskkill 返回非零；属于幂等行为，不视为错误
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
        // 进程已退出时 kill 返回非零，不视为错误
        return Ok(());
    }
    Ok(())
}

fn job_temp_dir(app: &AppHandle, job_id: &str) -> Result<PathBuf, String> {
    let dir = temp_cleanup::filter_temp_dir(app)?.join(sanitize_job_id(job_id));
    fs::create_dir_all(&dir).map_err(|err| format!("创建任务临时目录失败: {err}"))?;
    Ok(dir)
}

fn ensure_output_parent_dir(command: &[String]) -> Result<(), String> {
    let Some(output_path) = command
        .iter()
        .rposition(|arg| arg == "-y")
        .and_then(|index| index.checked_sub(1))
        .and_then(|index| command.get(index))
    else {
        return Ok(());
    };
    let Some(parent) = Path::new(output_path).parent() else {
        return Ok(());
    };
    if parent.as_os_str().is_empty() {
        return Ok(());
    }
    fs::create_dir_all(parent).map_err(|err| format!("创建输出目录失败: {err}"))
}

fn stage_subtitle_to_ascii(dir: &Path, subtitle_path: &str) -> Result<String, String> {
    let src = Path::new(subtitle_path);
    let ext = src
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("ass")
        .to_ascii_lowercase();
    let safe_ext = match ext.as_str() {
        "ass" | "srt" | "vtt" | "sub" => ext.as_str(),
        _ => "ass",
    };

    let staged = dir.join(format!("subtitle.{safe_ext}"));
    fs::copy(src, &staged).map_err(|err| format!("复制字幕到临时路径失败: {err}"))?;
    Ok(staged.to_string_lossy().to_string())
}

fn cleanup_temp_dir(app: &AppHandle, dir: &Path) {
    if !dir.exists() {
        return;
    }
    if let Err(err) = fs::remove_dir_all(dir) {
        let _ = app.emit(
            "compress-log",
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

// 按 \r 与 \n 同时拆分；空行跳过；EOF 时把残余 buffer 也发一次。
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

    // 1) ffmpeg 终端式进度行：用 \r 持续刷新的整行（含 time=… 与 frame=/size=）。
    //    解析所有字段后发到 compress-status，前端在专门的"状态行"区刷新展示，不进日志。
    if command_builder::is_ffmpeg_progress_line(trimmed) {
        let fields = command_builder::parse_ffmpeg_progress(trimmed);
        let percent = match (fields.current_seconds, duration_seconds) {
            (Some(cur), Some(dur)) if dur > 0.0 => Some((cur / dur * 100.0).clamp(0.0, 100.0)),
            _ => None,
        };
        let _ = app.emit(
            "compress-status",
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

    // 2) 其他持续噪声：编码器告警等
    if is_log_noise(trimmed) {
        return;
    }

    let _ = app.emit("compress-log", trimmed.to_string());
}

fn is_log_noise(line: &str) -> bool {
    // 子串匹配：编码器在运行时持续输出的提示，不需要展示
    const SUBSTRS: &[&str] = &[
        "VBV underflow",
        "Past duration",
        "Last message repeated",
        "deprecated pixel format",
    ];
    let trimmed = line.trim_start();
    SUBSTRS.iter().any(|s| trimmed.contains(s))
}
