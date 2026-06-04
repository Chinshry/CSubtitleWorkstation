use crate::models::avs_status::{AvsStatus, LavFiltersStatus};
use std::path::Path;
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
    use std::path::Path;

    let mut out = DetectedLavFilters::default();
    let uninstall_roots = [
        r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        r"HKLM\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
    ];

    for root in uninstall_roots {
        let Some((version, install_path)) = query_lav_uninstall_entry(root) else {
            continue;
        };
        out.installed = true;
        out.version = version;
        out.install_path = install_path;
        break;
    }

    if let Some(path) = out.install_path.as_deref() {
        out.x64_available = Path::new(path).join("x64").exists();
    }

    let has_splitter = registry_text_contains(r"HKCR\CLSID", "LAV Splitter");
    let has_video = registry_text_contains(r"HKCR\CLSID", "LAV Video Decoder");
    out.directshow_registered = has_splitter && has_video;
    if out.directshow_registered {
        out.installed = true;
    }

    out
}

#[cfg(not(windows))]
fn detect_lav_filters() -> DetectedLavFilters {
    DetectedLavFilters::default()
}

#[cfg(windows)]
fn query_lav_uninstall_entry(root: &str) -> Option<(Option<String>, Option<String>)> {
    let mut list_cmd = Command::new("reg");
    list_cmd.args(["query", root]);
    no_window(&mut list_cmd);
    let list_output = list_cmd.output().ok()?;
    if !list_output.status.success() {
        return None;
    }

    let list_text = String::from_utf8_lossy(&list_output.stdout);
    for key in list_text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        let mut cmd = Command::new("reg");
        cmd.args(["query", key]);
        no_window(&mut cmd);
        let Ok(output) = cmd.output() else { continue };
        if !output.status.success() {
            continue;
        }
        let text = String::from_utf8_lossy(&output.stdout);
        if !text.contains("LAV Filters") {
            continue;
        }
        let version = registry_value_from_text(&text, "DisplayVersion");
        let install_path = registry_value_from_text(&text, "InstallLocation");
        return Some((version, install_path));
    }
    None
}

#[cfg(windows)]
fn registry_value_from_text(text: &str, value_name: &str) -> Option<String> {
    for line in text.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with(value_name) {
            continue;
        }
        let Some(idx) = trimmed.find("REG_SZ") else {
            continue;
        };
        let value = trimmed[idx + "REG_SZ".len()..].trim();
        if !value.is_empty() {
            return Some(value.to_string());
        }
    }
    None
}

#[cfg(windows)]
fn registry_text_contains(root: &str, needle: &str) -> bool {
    let mut cmd = Command::new("reg");
    cmd.args(["query", root, "/f", needle, "/s"]);
    no_window(&mut cmd);
    let Ok(output) = cmd.output() else {
        return false;
    };
    if !output.status.success() {
        return false;
    }
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    text.contains(needle)
}

#[cfg(windows)]
fn read_registry_install_path() -> Option<String> {
    for key in [
        "HKLM\\SOFTWARE\\AviSynth",
        "HKLM\\SOFTWARE\\WOW6432Node\\AviSynth",
    ] {
        let mut cmd = Command::new("reg");
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
    let mut cmd = Command::new("powershell");
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

#[allow(dead_code)]
pub fn ensure_dll_dir(_dir: &Path) {
    // 占位：未来若要把内置 DLL 解压到独立目录可在此实现
}
