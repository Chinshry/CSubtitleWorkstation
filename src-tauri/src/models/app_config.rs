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
            default_logo_dir: Some(
                "E:\\Project\\CBash\\VIDEO_COMPRESSION\\@@压制工作站\\res\\logo".to_string(),
            ),
            default_use_avs: false,
        }
    }
}
