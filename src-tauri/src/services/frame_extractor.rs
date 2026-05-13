use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use tauri::{AppHandle, Manager};

/// 抽帧缓存目录名：app_local_data_dir() 下
const FRAME_DIR_NAME: &str = "logo-editor-frames";

static FRAME_SEQ: AtomicU64 = AtomicU64::new(0);

#[cfg(windows)]
fn no_window(builder: &mut Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    builder.creation_flags(CREATE_NO_WINDOW);
}
#[cfg(not(windows))]
fn no_window(_: &mut Command) {}

pub fn frame_cache_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_local_data_dir()
        .map_err(|err| format!("获取 app_local_data_dir 失败: {err}"))?
        .join(FRAME_DIR_NAME);
    fs::create_dir_all(&dir).map_err(|err| format!("创建抽帧缓存目录失败: {err}"))?;
    Ok(dir)
}

/// 从视频抽取一帧到本地 PNG 文件并返回绝对路径。
/// 时间戳单位为秒；clamp 到 [0, +inf)。
pub fn extract_frame(
    ffmpeg_path: &str,
    video_path: &str,
    time_seconds: f64,
    cache_dir: &Path,
) -> Result<PathBuf, String> {
    let ts = if time_seconds.is_finite() && time_seconds > 0.0 {
        time_seconds
    } else {
        0.0
    };
    let seq = FRAME_SEQ.fetch_add(1, Ordering::Relaxed);
    let filename = format!("frame-{seq}.png");
    let output_path = cache_dir.join(filename);

    // -ss 在 -i 之前：input-seek，快但不精确；够预览用。
    // -frames:v 1：只输出一帧。
    // -an：忽略音频流，避免某些容器错误。
    let mut cmd = Command::new(ffmpeg_path);
    cmd.args([
        "-hide_banner",
        "-loglevel",
        "error",
        "-ss",
    ])
    .arg(format!("{ts:.3}"))
    .arg("-i")
    .arg(video_path)
    .args(["-frames:v", "1", "-an", "-y"])
    .arg(&output_path);
    no_window(&mut cmd);

    let output = cmd
        .output()
        .map_err(|err| format!("启动 ffmpeg 抽帧失败: {err}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("ffmpeg 抽帧失败: {stderr}"));
    }
    if !output_path.exists() {
        return Err("ffmpeg 抽帧未生成输出文件".to_string());
    }
    Ok(output_path)
}

/// 清空抽帧缓存目录中所有旧文件（保留目录本身）。
/// 编辑器打开时调用一次，避免临时文件无限累积。
pub fn cleanup_frame_cache(cache_dir: &Path) {
    let Ok(entries) = fs::read_dir(cache_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            let _ = fs::remove_file(path);
        }
    }
}
