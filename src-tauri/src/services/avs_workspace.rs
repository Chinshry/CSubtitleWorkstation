use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

/// AVS 工作目录：内置 DLL + 临时 input.avs 输出位置
pub struct AvsWorkspace {
    pub dll_dir: PathBuf,
    pub temp_dir: PathBuf,
}

impl AvsWorkspace {
    pub fn vsfiltermod_path(&self) -> PathBuf {
        self.dll_dir.join("VSFilterMod.dll")
    }
    pub fn lsmashsource_path(&self) -> PathBuf {
        self.dll_dir.join("LSMASHSource.dll")
    }
}

/// 解析资源目录中的 DLL 位置；temp 目录使用 app_local_data_dir/avs-temp
pub fn resolve(app: &AppHandle) -> Result<AvsWorkspace, String> {
    let resource_root = app
        .path()
        .resource_dir()
        .map_err(|err| format!("获取 resource_dir 失败: {err}"))?;
    let dll_dir = resource_root.join("resources").join("avs");
    let dll_dir = if dll_dir.exists() {
        dll_dir
    } else {
        // 开发模式下 resource_dir 已直接指向 src-tauri，再次兜底尝试
        let fallback = resource_root.join("avs");
        if fallback.exists() {
            fallback
        } else {
            return Err(format!(
                "未找到 AVS DLL 资源目录，尝试位置: {}",
                dll_dir.display()
            ));
        }
    };

    let data_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|err| format!("获取 app_local_data_dir 失败: {err}"))?;
    let temp_dir = data_dir.join("avs-temp");
    fs::create_dir_all(&temp_dir).map_err(|err| format!("创建 AVS 临时目录失败: {err}"))?;

    Ok(AvsWorkspace { dll_dir, temp_dir })
}

/// 生成 input.avs 内容（AviSynth+ 语法，使用绝对路径避免 cwd 依赖）
pub fn build_avs_script(
    vsfiltermod: &Path,
    lsmash: &Path,
    video_path: &str,
    subtitle_path: &str,
) -> String {
    let vs_str = vsfiltermod.to_string_lossy();
    let ls_str = lsmash.to_string_lossy();
    format!(
        "LoadPlugin(\"{vs}\")\n\
         LoadPlugin(\"{ls}\")\n\
         Video=LWLibavVideoSource(\"{video}\")\n\
         Audio=LWLibavAudioSource(\"{video}\")\n\
         AudioDub(Video,Audio)\n\
         TextSubMod(\"{sub}\")\n",
        vs = escape_avs_string(strip_extended_path_prefix(&vs_str)),
        ls = escape_avs_string(strip_extended_path_prefix(&ls_str)),
        video = escape_avs_string(strip_extended_path_prefix(video_path)),
        sub = escape_avs_string(strip_extended_path_prefix(subtitle_path)),
    )
}

/// AviSynth 字符串只需要转义反斜杠与引号；脚本里我们已经用双引号包裹
fn escape_avs_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// 剥离 Windows 扩展长度路径前缀 `\\?\`。
///
/// 背景：Tauri `resource_dir()` / `Path::canonicalize` 在 Windows 上常返回带 `\\?\`
/// 前缀的 PathBuf。该前缀让 Win32 API 解除 MAX_PATH 限制，但 AviSynth 的 `LoadPlugin`
/// 等函数会把 `\\?\E:\foo` 错误解析成 `C:/?/E:/foo`，导致 DLL/资源加载失败。
/// 因此仅在写入 AVS 文本脚本这一边界做剥离；PathBuf 自身保持原样。
///
/// 未处理 `\\?\UNC\` 前缀：本项目所有传入路径（内置 DLL、本地视频、本地字幕）
/// 均位于本地磁盘，不会出现 UNC verbatim。如未来引入网络路径再扩展。
fn strip_extended_path_prefix(path: &str) -> &str {
    path.strip_prefix(r"\\?\").unwrap_or(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_keeps_normal_paths() {
        assert_eq!(
            strip_extended_path_prefix("E:\\foo\\bar.dll"),
            "E:\\foo\\bar.dll"
        );
        assert_eq!(strip_extended_path_prefix("/tmp/x.dll"), "/tmp/x.dll");
        assert_eq!(strip_extended_path_prefix(""), "");
    }

    #[test]
    fn strip_removes_verbatim_prefix() {
        assert_eq!(
            strip_extended_path_prefix(r"\\?\E:\Project\foo.dll"),
            r"E:\Project\foo.dll"
        );
    }

    #[test]
    fn build_script_outputs_clean_dll_paths() {
        let vs = PathBuf::from(r"\\?\E:\Project\resources\avs\VSFilterMod.dll");
        let ls = PathBuf::from(r"\\?\E:\Project\resources\avs\LSMASHSource.dll");
        let script = build_avs_script(&vs, &ls, r"E:\video.mp4", r"E:\sub.ass");
        assert!(script.contains(r#"LoadPlugin("E:\\Project\\resources\\avs\\VSFilterMod.dll")"#));
        assert!(script.contains(r#"LoadPlugin("E:\\Project\\resources\\avs\\LSMASHSource.dll")"#));
        assert!(!script.contains(r"\\?\"));
    }
}

/// 写入 input.avs 并返回脚本绝对路径
pub fn write_script(workspace: &AvsWorkspace, content: &str) -> Result<PathBuf, String> {
    let script_path = workspace.temp_dir.join("input.avs");
    fs::write(&script_path, content).map_err(|err| format!("写入 AVS 脚本失败: {err}"))?;
    Ok(script_path)
}
