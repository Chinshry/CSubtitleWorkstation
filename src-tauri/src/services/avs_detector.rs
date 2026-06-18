use crate::models::avs_status::{AvsStatus, LavFiltersStatus};
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

/// 平台是否可能支持 AVS（仅 Windows）
pub fn is_supported_platform() -> bool {
    cfg!(windows)
}

/// 综合检测 AVS 可用性
pub fn detect(ffmpeg_path: Option<&str>) -> AvsStatus {
    let supported = is_supported_platform();

    if !supported {
        return AvsStatus {
            supported_platform: false,
            ffmpeg_demuxer_available: false,
            avisynth_installed: false,
            avisynth_version: None,
            avisynth_install_path: None,
            avisynth_dll_path: None,
            lav_filters_installed: false,
            lav_filters_version: None,
            lav_filters_install_path: None,
            lav_filters_x64_available: false,
            lav_filters_directshow_registered: false,
            available: false,
            message: Some("AVS 压制仅支持 Windows".to_string()),
        };
    }

    let demuxer_available = ffmpeg_path.map(check_ffmpeg_demuxer).unwrap_or(false);

    let detected = detect_avisynth();
    let avisynth_installed = detected.installed;
    let available = demuxer_available && avisynth_installed;

    let message = if available {
        None
    } else if !demuxer_available && !avisynth_installed {
        Some("缺少 ffmpeg avisynth demuxer 与 AviSynth+ 安装".to_string())
    } else if !demuxer_available {
        Some("当前 ffmpeg 未启用 avisynth demuxer（需 --enable-avisynth 构建版本，例如 Gyan.dev full 版）".to_string())
    } else {
        Some("未检测到 AviSynth+，请先安装 AviSynth+ 运行环境".to_string())
    };

    AvsStatus {
        supported_platform: true,
        ffmpeg_demuxer_available: demuxer_available,
        avisynth_installed,
        avisynth_version: detected.version,
        avisynth_install_path: detected.install_path,
        avisynth_dll_path: detected.dll_path,
        lav_filters_installed: false,
        lav_filters_version: None,
        lav_filters_install_path: None,
        lav_filters_x64_available: false,
        lav_filters_directshow_registered: false,
        available,
        message,
    }
}

pub fn detect_lav_filters_status() -> LavFiltersStatus {
    if !is_supported_platform() {
        return LavFiltersStatus::default();
    }

    let lav = detect_lav_filters();
    LavFiltersStatus {
        lav_filters_installed: lav.installed,
        lav_filters_version: lav.version,
        lav_filters_install_path: lav.install_path,
        lav_filters_x64_available: lav.x64_available,
        lav_filters_directshow_registered: lav.directshow_registered,
    }
}

/// 通过 `ffmpeg -hide_banner -demuxers` 输出搜索 avisynth 关键字
fn check_ffmpeg_demuxer(ffmpeg_path: &str) -> bool {
    let mut cmd = Command::new(ffmpeg_path);
    cmd.args(["-hide_banner", "-demuxers"]);
    no_window(&mut cmd);
    let Ok(output) = cmd.output() else {
        return false;
    };
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    // 输出形如："  D  avisynth        AviSynth script"
    text.lines().any(|line| {
        let lower = line.to_ascii_lowercase();
        // 避免误命中如 'avi' 这类前缀，要求单词边界
        lower.split_whitespace().any(|tok| tok == "avisynth")
    })
}

#[derive(Debug, Default)]
struct DetectedAvisynth {
    installed: bool,
    /// AviSynth.dll FileVersion，如 "3.7.3.0"
    version: Option<String>,
    /// 注册表 HKLM\SOFTWARE\AviSynth 默认值（AviSynth+ 安装目录，例如 "D:\AviSynth+"）
    install_path: Option<String>,
    /// 实际被加载的 AviSynth.dll 路径
    dll_path: Option<String>,
}

#[derive(Debug, Default)]
struct DetectedLavFilters {
    installed: bool,
    version: Option<String>,
    install_path: Option<String>,
    x64_available: bool,
    directshow_registered: bool,
}

/// Windows: 优先检 system32\AviSynth.dll；再尝试注册表 HKLM\SOFTWARE\AviSynth
#[cfg(windows)]
fn detect_avisynth() -> DetectedAvisynth {
    use std::path::PathBuf;
    let mut out = DetectedAvisynth::default();

    // 1) SystemRoot/System32/AviSynth.dll —— AviSynth+ 默认安装位置
    if let Some(system_root) = std::env::var_os("SystemRoot") {
        let candidates = [
            PathBuf::from(&system_root)
                .join("System32")
                .join("AviSynth.dll"),
            PathBuf::from(&system_root)
                .join("SysWOW64")
                .join("AviSynth.dll"),
        ];
        for path in candidates {
            if path.exists() {
                out.installed = true;
                out.dll_path = Some(path.to_string_lossy().to_string());
                out.version = read_dll_file_version(&path);
                break;
            }
        }
    }

    // 2) 注册表 HKLM\SOFTWARE\AviSynth：默认值 = 安装目录（不是版本号）
    let install_path = read_registry_install_path();
    if install_path.is_some() {
        out.installed = true;
        out.install_path = install_path;
    }

    out
}

#[cfg(not(windows))]
fn detect_avisynth() -> DetectedAvisynth {
    DetectedAvisynth::default()
}

/// 注册表 HKLM\SOFTWARE\AviSynth 默认值 → 安装目录路径
#[cfg(windows)]
fn detect_lav_filters() -> DetectedLavFilters {
    let mut out = DetectedLavFilters::default();

    for path in common_lav_install_paths() {
        if path.exists() {
            out.installed = true;
            out.install_path = Some(path.to_string_lossy().to_string());
            out.x64_available = path.join("x64").exists();
            break;
        }
    }

    let splitter_path = query_registered_lav_filter_path(LAV_SPLITTER_CLSID);
    let video_path = query_registered_lav_filter_path(LAV_VIDEO_DECODER_CLSID);
    out.directshow_registered = splitter_path.is_some() && video_path.is_some();

    if out.directshow_registered {
        out.installed = true;
        if out.install_path.is_none() {
            out.install_path = splitter_path
                .as_deref()
                .and_then(lav_install_dir_from_registered_filter);
        }
        if !out.x64_available {
            out.x64_available = splitter_path
                .as_deref()
                .map(is_registered_filter_x64)
                .unwrap_or(false)
                || video_path
                    .as_deref()
                    .map(is_registered_filter_x64)
                    .unwrap_or(false);
        }
    }

    out
}

#[cfg(not(windows))]
fn detect_lav_filters() -> DetectedLavFilters {
    DetectedLavFilters::default()
}

#[cfg(windows)]
const LAV_SPLITTER_CLSID: &str = r"{B98D13E7-55DB-4385-A33D-09FD1BA26338}";
#[cfg(windows)]
const LAV_VIDEO_DECODER_CLSID: &str = r"{EE30215D-164F-4A92-A4EB-9D4C13390F9F}";

#[cfg(windows)]
fn query_registered_lav_filter_path(clsid: &str) -> Option<String> {
    for root in [
        r"HKCR\CLSID",
        r"HKLM\SOFTWARE\Classes\CLSID",
        r"HKLM\SOFTWARE\WOW6432Node\Classes\CLSID",
    ] {
        let key = format!(r"{root}\{clsid}\InprocServer32");
        let mut cmd = Command::new(system_tool_path("reg.exe"));
        cmd.args(["query", &key, "/ve"]);
        no_window(&mut cmd);
        let Ok(output) = cmd.output() else { continue };
        if !output.status.success() {
            continue;
        }
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            let trimmed = line.trim();
            if let Some(idx) = trimmed.find("REG_SZ") {
                let value = trimmed[idx + "REG_SZ".len()..].trim();
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }
    }
    None
}

#[cfg(windows)]
fn lav_install_dir_from_registered_filter(path: &str) -> Option<String> {
    let path = Path::new(path);
    let parent = path.parent()?;
    if parent
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case("x64") || name.eq_ignore_ascii_case("x86"))
    {
        return parent.parent().map(|dir| dir.to_string_lossy().to_string());
    }
    Some(parent.to_string_lossy().to_string())
}

#[cfg(windows)]
fn is_registered_filter_x64(path: &str) -> bool {
    Path::new(path)
        .parent()
        .and_then(|dir| dir.file_name())
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case("x64"))
}

#[cfg(windows)]
fn read_registry_install_path() -> Option<String> {
    for key in [
        "HKLM\\SOFTWARE\\AviSynth",
        "HKLM\\SOFTWARE\\WOW6432Node\\AviSynth",
    ] {
        let mut cmd = Command::new(system_tool_path("reg.exe"));
        cmd.args(["query", key, "/ve"]);
        no_window(&mut cmd);
        let Ok(output) = cmd.output() else { continue };
        if !output.status.success() {
            continue;
        }
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            let trimmed = line.trim();
            if let Some(idx) = trimmed.find("REG_SZ") {
                let value = trimmed[idx + "REG_SZ".len()..].trim();
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }
    }
    None
}

/// 通过 PowerShell 读取 dll 的 FileVersion；失败返回 None
#[cfg(windows)]
fn read_dll_file_version(dll: &Path) -> Option<String> {
    let script = format!(
        "(Get-Item -LiteralPath '{}').VersionInfo.ProductVersion",
        dll.display().to_string().replace('\'', "''")
    );
    let mut cmd = Command::new(powershell_path());
    cmd.args(["-NoProfile", "-NonInteractive", "-Command", &script]);
    no_window(&mut cmd);
    let output = cmd.output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

#[cfg(windows)]
fn common_lav_install_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(program_files) = std::env::var_os("ProgramFiles") {
        paths.push(PathBuf::from(program_files).join("LAV Filters"));
    }
    if let Some(program_files_x86) = std::env::var_os("ProgramFiles(x86)") {
        paths.push(PathBuf::from(program_files_x86).join("LAV Filters"));
    }
    paths.push(PathBuf::from(r"C:\Program Files\LAV Filters"));
    paths.push(PathBuf::from(r"C:\Program Files (x86)\LAV Filters"));
    paths
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
fn powershell_path() -> PathBuf {
    let system32 = std::env::var_os("SystemRoot")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Windows"))
        .join("System32");

    let powershell = system32
        .join("WindowsPowerShell")
        .join("v1.0")
        .join("powershell.exe");
    if powershell.exists() {
        powershell
    } else {
        system32.join("powershell.exe")
    }
}

#[allow(dead_code)]
pub fn ensure_dll_dir(_dir: &Path) {
    // 占位：未来若要把内置 DLL 解压到独立目录可在此实现
}
