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
            output_name_template: "{name} 中字.mp4".to_string(),
            check_update_on_startup: true,
            default_logo_dir: None,
            default_use_avs: false,
            recent_logos: Vec::new(),
            last_logo_layout: None,
            logo_layouts: Vec::new(),
        }
    }
}
