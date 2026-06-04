use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AvsStatus {
    /// 当前平台是否可能支持 AVS（仅 Windows = true）
    pub supported_platform: bool,
    /// ffmpeg 是否包含 avisynth demuxer（构建时带 --enable-avisynth）
    pub ffmpeg_demuxer_available: bool,
    /// 系统是否安装 AviSynth+/AviSynth（Windows 注册表或 system32\AviSynth.dll）
    pub avisynth_installed: bool,
    /// AviSynth.dll 文件版本号（从 dll FileVersion 读取，例如 "3.7.3.0"）
    pub avisynth_version: Option<String>,
    /// 注册表中记录的 AviSynth+ 安装目录（例如 "D:\AviSynth+"）
    pub avisynth_install_path: Option<String>,
    /// 实际被 ffmpeg 加载的 AviSynth.dll 路径（例如 "C:\Windows\System32\AviSynth.dll"）
    pub avisynth_dll_path: Option<String>,
    /// 系统是否安装 LAV Filters（Windows 注册表或 system32\LAVFilters.dll）
    pub lav_filters_installed: bool,
    /// LAV Filters 的版本号（从 dll FileVersion 读取，例如 "0.7.17.0"）
    pub lav_filters_version: Option<String>,
    /// 注册表中记录的 LAV Filters 安装目录（例如 "D:\LAVFilters"）
    pub lav_filters_install_path: Option<String>,
    /// 系统是否安装 LAV Filters x64（Windows 注册表或 system32\LAVFilters64.dll）
    pub lav_filters_x64_available: bool,
    /// 注册表中记录的 LAV Filters x64 安装目录（例如 "D:\LAVFilters64"）
    pub lav_filters_directshow_registered: bool,
    /// 综合判断：能否开启 AVS 模式
    pub available: bool,
    /// 错误信息
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LavFiltersStatus {
    pub lav_filters_installed: bool,
    pub lav_filters_version: Option<String>,
    pub lav_filters_install_path: Option<String>,
    pub lav_filters_x64_available: bool,
    pub lav_filters_directshow_registered: bool,
}

impl Default for AvsStatus {
    fn default() -> Self {
        Self {
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
            message: None,
        }
    }
}
