use serde::{Deserialize, Serialize};

use crate::models::app_config::LogoLayout;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompressJob {
    pub id: String,
    pub video_path: String,
    pub subtitle_path: String,
    pub output_path: String,
    pub crf: u8,
    pub max_bitrate: Option<i32>,
    pub need_logo: bool,
    pub need_yadif: bool,
    pub encoder: String,
    #[serde(default)]
    pub logo_dir: Option<String>,
    #[serde(default)]
    pub use_avs: bool,
    /// 可视化编辑器输出的 LOGO 布局（百分比）。need_logo=true 且本字段存在时使用；
    /// 不存在则不叠加 LOGO（不再回退到 ASS 行解析）。
    #[serde(default)]
    pub logo_layout: Option<LogoLayout>,
    /// 前端从 inspect_video_meta 获取的"显示尺寸"（已应用 rotation）。
    /// command_builder 优先用它换算 LOGO overlay 像素，避免再走没 rotation 知识的文本解析。
    #[serde(default)]
    pub video_width: Option<i32>,
    #[serde(default)]
    pub video_height: Option<i32>,
}
