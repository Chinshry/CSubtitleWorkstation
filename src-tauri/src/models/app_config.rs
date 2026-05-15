use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub ffmpeg_mode: FfmpegMode,
    pub ffmpeg_path: Option<String>,
    pub default_crf: u8,
    pub default_need_logo: bool,
    pub default_need_yadif: bool,
    pub default_encoder: String,
    pub output_name_template: String,
    #[serde(default)]
    pub output_templates: Vec<OutputNameTemplate>,
    #[serde(default)]
    pub default_output_template_id: Option<String>,
    #[serde(default)]
    pub encode_presets: Vec<VideoEncodePreset>,
    #[serde(default)]
    pub default_encode_preset_id: Option<String>,
    pub check_update_on_startup: bool,
    #[serde(default)]
    pub default_logo_dir: Option<String>,
    #[serde(default)]
    pub default_use_avs: bool,
    /// 最近使用过的 LOGO 图片，按 last_used_at 倒序，最多保留 10 项
    #[serde(default)]
    pub recent_logos: Vec<RecentLogo>,
    /// 上次保存的 LOGO 布局，按百分比存储以适配不同分辨率视频。
    /// 仅在 logo_layouts 未命中桶时作为 fallback 使用。
    #[serde(default)]
    pub last_logo_layout: Option<LogoLayout>,
    /// 按 (分辨率桶, LOGO 图路径) 区分的布局记忆。
    /// 桶 key 例如 "1080p-landscape" / "1080p-portrait" / "720p-landscape" / "4k-portrait"。
    /// 非常见分辨率（不在 720p/1080p/4K 横竖屏内）不写入此列表。
    #[serde(default)]
    pub logo_layouts: Vec<LogoLayoutEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoEncodePreset {
    pub id: String,
    pub name: String,
    pub encoder: String,
    pub crf: u8,
    #[serde(default)]
    pub max_bitrate: Option<i32>,
    #[serde(default)]
    pub custom_video_args: Option<String>,
    #[serde(default)]
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentLogo {
    pub path: String,
    /// Unix 毫秒时间戳
    pub last_used_at: i64,
    /// 用户自定义昵称；为空/缺省时前端回退到 path 的文件名。
    /// 老配置文件不含此字段；序列化时若 None 也省略，避免污染存量配置。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputNameTemplate {
    pub id: String,
    pub name: String,
    pub pattern: String,
    pub output_dir_mode: OutputDirMode,
    #[serde(default)]
    pub fixed_output_dir: Option<String>,
    #[serde(default)]
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OutputDirMode {
    SameAsVideo,
    Fixed,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogoLayout {
    pub path: String,
    /// 左上角横坐标占视频宽度的百分比 [0.0, 1.0]
    pub x_pct: f64,
    /// 左上角纵坐标占视频高度的百分比 [0.0, 1.0]
    pub y_pct: f64,
    /// LOGO 宽度占视频宽度的百分比
    pub w_pct: f64,
    /// LOGO 高度占视频高度的百分比
    pub h_pct: f64,
}

/// 按 (分辨率桶, LOGO 图路径) 维度独立记忆的 LOGO 布局条目。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogoLayoutEntry {
    /// 分辨率桶 key，由前端 resolveBucket 计算。
    pub bucket: String,
    /// LOGO 图绝对路径
    pub path: String,
    pub x_pct: f64,
    pub y_pct: f64,
    pub w_pct: f64,
    pub h_pct: f64,
    /// Unix 毫秒时间戳，用于排序与展示
    pub last_used_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FfmpegMode {
    System,
    Custom,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ffmpeg_mode: FfmpegMode::System,
            ffmpeg_path: None,
            default_crf: 18,
            default_need_logo: true,
            default_need_yadif: false,
            default_encoder: "libx264".to_string(),
            output_name_template: "{video_name} output.mp4".to_string(),
            output_templates: vec![OutputNameTemplate {
                id: "default".to_string(),
                name: "默认".to_string(),
                pattern: "{video_name} output.mp4".to_string(),
                output_dir_mode: OutputDirMode::SameAsVideo,
                fixed_output_dir: None,
                is_default: true,
            }],
            default_output_template_id: Some("default".to_string()),
            encode_presets: vec![
                VideoEncodePreset {
                    id: "balanced-x264".to_string(),
                    name: "x264 平衡".to_string(),
                    encoder: "libx264".to_string(),
                    crf: 18,
                    max_bitrate: None,
                    custom_video_args: Some("-preset slow -profile:v high -pix_fmt yuv420p".to_string()),
                    is_default: true,
                },
                VideoEncodePreset {
                    id: "fast-nvenc".to_string(),
                    name: "NVENC 快速".to_string(),
                    encoder: "h264_nvenc".to_string(),
                    crf: 19,
                    max_bitrate: None,
                    custom_video_args: Some("-spatial-aq 1 -temporal-aq 1".to_string()),
                    is_default: false,
                },
                VideoEncodePreset {
                    id: "hevc-small".to_string(),
                    name: "x265 体积优先".to_string(),
                    encoder: "libx265".to_string(),
                    crf: 22,
                    max_bitrate: None,
                    custom_video_args: Some("-preset medium -pix_fmt yuv420p -x265-params aq-mode=1:psy-rd=2.0".to_string()),
                    is_default: false,
                },
            ],
            default_encode_preset_id: Some("balanced-x264".to_string()),
            check_update_on_startup: true,
            default_logo_dir: None,
            default_use_avs: false,
            recent_logos: Vec::new(),
            last_logo_layout: None,
            logo_layouts: Vec::new(),
        }
    }
}
