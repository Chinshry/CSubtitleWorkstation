use crate::models::app_config::{AppConfig, FfmpegMode};
use crate::models::ffmpeg_status::{FfmpegSource, FfmpegStatus};
use std::collections::HashSet;
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

    let path_result = inspect_path(Path::new(system_ffmpeg_name()), FfmpegSource::SystemPath);
    if path_result.available {
        return path_result;
    }

    for candidate in fallback_ffmpeg_candidates() {
        if !candidate.exists() {
            continue;
        }
        let status = inspect_path(&candidate, FfmpegSource::SystemPath);
        if status.available {
            return status;
        }
    }

    path_result
}

pub fn inspect_path(path: &Path, source: FfmpegSource) -> FfmpegStatus {
    let mut cmd = Command::new(path);
    cmd.arg("-version");
    no_window(&mut cmd);
    match cmd.output() {
        Ok(output) if output.status.success() => {
            let text = String::from_utf8_lossy(&output.stdout);
            let version = text.lines().next().map(|line| line.trim().to_string());

            let (ffprobe_path, ffprobe_version) = detect_ffprobe(path);
            let (subtitle_filter_available, ass_filter_available) = detect_subtitle_filters(path);
            let available = cfg!(target_os = "windows") || subtitle_filter_available;
            let message = if available {
                None
            } else {
                Some(subtitle_filter_missing_message())
            };

            FfmpegStatus {
                available,
                source,
                ffmpeg_path: Some(path_to_string(path)),
                ffmpeg_version: version,
                ffprobe_path,
                ffprobe_version,
                subtitle_filter_available,
                ass_filter_available,
                message,
            }
        }
        Ok(output) => FfmpegStatus {
            available: false,
            source,
            ffmpeg_path: Some(path_to_string(path)),
            ffmpeg_version: None,
            ffprobe_path: None,
            ffprobe_version: None,
            subtitle_filter_available: false,
            ass_filter_available: false,
            message: Some(format!("ffmpeg 返回非零状态: {}", output.status)),
        },
        Err(err) => FfmpegStatus {
            available: false,
            source: FfmpegSource::NotFound,
            ffmpeg_path: Some(path_to_string(path)),
            ffmpeg_version: None,
            ffprobe_path: None,
            ffprobe_version: None,
            subtitle_filter_available: false,
            ass_filter_available: false,
            message: Some(format!("未找到可用 ffmpeg: {err}")),
        },
    }
}

fn detect_subtitle_filters(ffmpeg_path: &Path) -> (bool, bool) {
    let mut cmd = Command::new(ffmpeg_path);
    cmd.args(["-hide_banner", "-filters"]);
    no_window(&mut cmd);
    let Ok(output) = cmd.output() else {
        return (false, false);
    };
    if !output.status.success() {
        return (false, false);
    }
    let text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    parse_subtitle_filters(&text)
}

fn parse_subtitle_filters(text: &str) -> (bool, bool) {
    let has_filter = |name: &str| {
        text.lines()
            .filter_map(|line| line.split_whitespace().nth(1))
            .any(|filter| filter == name)
    };
    (has_filter("subtitles"), has_filter("ass"))
}

fn subtitle_filter_missing_message() -> String {
    if cfg!(target_os = "macos") {
        "当前 ffmpeg 缺少 subtitles/libass filter，macOS 请安装 ffmpeg-full，并在本工具中选择 /opt/homebrew/opt/ffmpeg-full/bin/ffmpeg。".to_string()
    } else {
        "当前 ffmpeg 缺少 subtitles/libass filter，无法压制 ASS 字幕。请安装包含 libass/subtitles filter 的 ffmpeg full 构建。".to_string()
    }
}

fn detect_ffprobe(ffmpeg_path: &Path) -> (Option<String>, Option<String>) {
    let probe_name = system_ffprobe_name();

    if let Some(dir) = ffmpeg_path.parent() {
        let candidate = dir.join(probe_name);
        if candidate.exists() {
            if let Some(version) = run_version(&candidate) {
                return (Some(path_to_string(&candidate)), Some(version));
            }
        }
    }

    let bare = Path::new(probe_name);
    if let Some(version) = run_version(bare) {
        return (Some(probe_name.to_string()), Some(version));
    }

    for dir in system_path_dirs() {
        let candidate = dir.join(probe_name);
        if !candidate.exists() {
            continue;
        }
        if let Some(version) = run_version(&candidate) {
            return (Some(path_to_string(&candidate)), Some(version));
        }
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
        return path
            .join(system_ffmpeg_name())
            .to_string_lossy()
            .to_string();
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

fn fallback_ffmpeg_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    let mut seen = HashSet::new();

    for dir in system_path_dirs() {
        push_path(&mut candidates, &mut seen, dir.join(system_ffmpeg_name()));
    }

    for path in well_known_ffmpeg_paths() {
        push_path(&mut candidates, &mut seen, PathBuf::from(path));
    }

    candidates
}

fn well_known_ffmpeg_paths() -> &'static [&'static str] {
    #[cfg(target_os = "macos")]
    {
        &[
            "/opt/homebrew/opt/ffmpeg-full/bin/ffmpeg",
            "/usr/local/opt/ffmpeg-full/bin/ffmpeg",
            "/opt/homebrew/bin/ffmpeg",
            "/usr/local/bin/ffmpeg",
            "/opt/local/bin/ffmpeg",
        ]
    }
    #[cfg(target_os = "linux")]
    {
        &[
            "/usr/bin/ffmpeg",
            "/usr/local/bin/ffmpeg",
            "/snap/bin/ffmpeg",
        ]
    }
    #[cfg(target_os = "windows")]
    {
        &[
            r"C:\ffmpeg\bin\ffmpeg.exe",
            r"C:\Program Files\ffmpeg\bin\ffmpeg.exe",
            r"C:\Program Files (x86)\ffmpeg\bin\ffmpeg.exe",
            r"D:\ffmpeg\bin\ffmpeg.exe",
        ]
    }
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn system_path_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let mut seen = HashSet::new();

    if let Some(path) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path) {
            push_dir(&mut dirs, &mut seen, dir);
        }
    }

    #[cfg(windows)]
    {
        for raw in windows_registry_path_values() {
            for item in raw.split(';').map(str::trim).filter(|item| !item.is_empty()) {
                push_dir(
                    &mut dirs,
                    &mut seen,
                    PathBuf::from(expand_windows_env_vars(item)),
                );
            }
        }
    }

    dirs
}

fn push_path(paths: &mut Vec<PathBuf>, seen: &mut HashSet<String>, path: PathBuf) {
    let key = path.to_string_lossy().to_ascii_lowercase();
    if seen.insert(key) {
        paths.push(path);
    }
}

fn push_dir(dirs: &mut Vec<PathBuf>, seen: &mut HashSet<String>, dir: PathBuf) {
    let key = dir.to_string_lossy().trim_end_matches('\\').to_ascii_lowercase();
    if !key.is_empty() && seen.insert(key) {
        dirs.push(dir);
    }
}

#[cfg(windows)]
fn windows_registry_path_values() -> Vec<String> {
    [
        (
            r"HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment",
            "Path",
        ),
        (r"HKCU\Environment", "Path"),
    ]
    .into_iter()
    .filter_map(|(key, name)| query_registry_value(key, name))
    .collect()
}

#[cfg(windows)]
fn query_registry_value(key: &str, value_name: &str) -> Option<String> {
    let mut cmd = Command::new(system_tool_path("reg.exe"));
    cmd.args(["query", key, "/v", value_name]);
    no_window(&mut cmd);
    let output = cmd.output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    registry_value_from_query_text(&text, value_name)
}

#[cfg(windows)]
fn registry_value_from_query_text(text: &str, value_name: &str) -> Option<String> {
    for line in text.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with(value_name) {
            continue;
        }
        for marker in ["REG_EXPAND_SZ", "REG_SZ"] {
            if let Some(idx) = trimmed.find(marker) {
                let value = trimmed[idx + marker.len()..].trim();
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }
    }
    None
}

#[cfg(windows)]
fn system_tool_path(name: &str) -> PathBuf {
    std::env::var_os("SystemRoot")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Windows"))
        .join("System32")
        .join(name)
}

#[cfg(windows)]
fn expand_windows_env_vars(input: &str) -> String {
    let mut out = input.to_string();
    for name in [
        "SystemRoot",
        "WINDIR",
        "ProgramFiles",
        "ProgramFiles(x86)",
        "USERPROFILE",
    ] {
        let pattern = format!("%{name}%");
        if let Ok(value) = std::env::var(name) {
            out = out.replace(&pattern, &value);
        }
    }
    out
}
