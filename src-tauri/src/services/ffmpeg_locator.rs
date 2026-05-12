use crate::models::app_config::{AppConfig, FfmpegMode};
use crate::models::ffmpeg_status::{FfmpegSource, FfmpegStatus};
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

pub fn detect(config: &AppConfig) -> FfmpegStatus {
    match config.ffmpeg_mode {
        FfmpegMode::Custom => {
            if let Some(path) = &config.ffmpeg_path {
                return inspect_path(Path::new(path), FfmpegSource::CustomPath);
            }
        }
        FfmpegMode::System => {}
    }

    inspect_path(Path::new(system_ffmpeg_name()), FfmpegSource::SystemPath)
}

pub fn inspect_path(path: &Path, source: FfmpegSource) -> FfmpegStatus {
    let mut cmd = Command::new(path);
    cmd.arg("-version");
    no_window(&mut cmd);
    match cmd.output() {
        Ok(output) if output.status.success() => {
            let text = String::from_utf8_lossy(&output.stdout);
            let version = text.lines().next().map(|line| line.trim().to_string());

            // 同目录或 PATH 中探测 ffprobe
            let (ffprobe_path, ffprobe_version) = detect_ffprobe(path);

            FfmpegStatus {
                available: true,
                source,
                ffmpeg_path: Some(path_to_string(path)),
                ffmpeg_version: version,
                ffprobe_path,
                ffprobe_version,
                message: None,
            }
        }
        Ok(output) => FfmpegStatus {
            available: false,
            source,
            ffmpeg_path: Some(path_to_string(path)),
            ffmpeg_version: None,
            ffprobe_path: None,
            ffprobe_version: None,
            message: Some(format!("ffmpeg 返回非零状态: {}", output.status)),
        },
        Err(err) => FfmpegStatus {
            available: false,
            source: FfmpegSource::NotFound,
            ffmpeg_path: Some(path_to_string(path)),
            ffmpeg_version: None,
            ffprobe_path: None,
            ffprobe_version: None,
            message: Some(format!("未找到可用 ffmpeg: {err}")),
        },
    }
}

// 找 ffprobe：优先 ffmpeg 同目录，否则尝试 PATH 中的 ffprobe
fn detect_ffprobe(ffmpeg_path: &Path) -> (Option<String>, Option<String>) {
    let probe_name = system_ffprobe_name();

    // 1) ffmpeg 同目录
    if let Some(dir) = ffmpeg_path.parent() {
        let candidate = dir.join(probe_name);
        if candidate.exists() {
            if let Some(version) = run_version(&candidate) {
                return (Some(path_to_string(&candidate)), Some(version));
            }
        }
    }

    // 2) PATH 上的 ffprobe（适用于 system_path 模式或散装安装）
    let bare = Path::new(probe_name);
    if let Some(version) = run_version(bare) {
        return (Some(probe_name.to_string()), Some(version));
    }

    (None, None)
}

fn run_version(path: &Path) -> Option<String> {
    let mut cmd = Command::new(path);
    cmd.arg("-version");
    no_window(&mut cmd);
    let output = cmd.output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    text.lines().next().map(|line| line.trim().to_string())
}

pub fn normalize_user_path(raw_path: &str) -> String {
    let path = PathBuf::from(raw_path);
    if path.is_dir() {
        return path.join(system_ffmpeg_name()).to_string_lossy().to_string();
    }
    raw_path.to_string()
}

pub fn system_ffmpeg_name() -> &'static str {
    if cfg!(windows) {
        "ffmpeg.exe"
    } else {
        "ffmpeg"
    }
}

pub fn system_ffprobe_name() -> &'static str {
    if cfg!(windows) {
        "ffprobe.exe"
    } else {
        "ffprobe"
    }
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}
