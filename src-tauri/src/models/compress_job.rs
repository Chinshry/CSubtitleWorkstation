use serde::{Deserialize, Serialize};

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
}
