use crate::models::avs_status::AvsStatus;
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
            available: false,
            message: Some("AVS 压制仅支持 Windows".to_string()),
        };
    }

    let demuxer_available = ffmpeg_path
        .map(check_ffmpeg_demuxer)
        .unwrap_or(false);

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
        available,
        message,
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

/// Windows: 优先检 system32\AviSynth.dll；再尝试注册表 HKLM\SOFTWARE\AviSynth
#[cfg(windows)]
fn detect_avisynth() -> DetectedAvisynth {
    use std::path::PathBuf;
    let mut out = DetectedAvisynth::default();

    // 1) SystemRoot/System32/AviSynth.dll —— AviSynth+ 默认安装位置
    if let Some(system_root) = std::env::var_os("SystemRoot") {
        let candidates = [
            PathBuf::from(&system_root).join("System32").join("AviSynth.dll"),
            PathBuf::from(&system_root).join("SysWOW64").join("AviSynth.dll"),
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
