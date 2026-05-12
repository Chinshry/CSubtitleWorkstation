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
    fs::create_dir_all(&temp_dir)
        .map_err(|err| format!("创建 AVS 临时目录失败: {err}"))?;

    Ok(AvsWorkspace { dll_dir, temp_dir })
}

/// 生成 input.avs 内容（AviSynth+ 语法，使用绝对路径避免 cwd 依赖）
pub fn build_avs_script(
    vsfiltermod: &Path,
    lsmash: &Path,
    video_path: &str,
    subtitle_path: &str,
) -> String {
    format!(
        "LoadPlugin(\"{vs}\")\n\
         LoadPlugin(\"{ls}\")\n\
         Video=LWLibavVideoSource(\"{video}\")\n\
         Audio=LWLibavAudioSource(\"{video}\")\n\
         AudioDub(Video,Audio)\n\
         TextSubMod(\"{sub}\")\n",
        vs = escape_avs_string(&vsfiltermod.to_string_lossy()),
        ls = escape_avs_string(&lsmash.to_string_lossy()),
        video = escape_avs_string(video_path),
        sub = escape_avs_string(subtitle_path),
    )
}

/// AviSynth 字符串只需要转义反斜杠与引号；脚本里我们已经用双引号包裹
fn escape_avs_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// 写入 input.avs 并返回脚本绝对路径
pub fn write_script(workspace: &AvsWorkspace, content: &str) -> Result<PathBuf, String> {
    let script_path = workspace.temp_dir.join("input.avs");
    fs::write(&script_path, content)
        .map_err(|err| format!("写入 AVS 脚本失败: {err}"))?;
    Ok(script_path)
}
