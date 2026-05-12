use crate::services::{config_store, ffmpeg_locator, video_meta};
use tauri::AppHandle;

#[tauri::command]
pub fn inspect_video_meta(
    app: AppHandle,
    path: String,
) -> Result<video_meta::VideoMeta, String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("Path is empty.".to_string());
    }
    let config = config_store::load(&app)?;
    let status = ffmpeg_locator::detect(&config);
    let ffmpeg_path = status
        .ffmpeg_path
        .ok_or_else(|| "ffmpeg is not configured.".to_string())?;
    video_meta::inspect(&ffmpeg_path, status.ffprobe_path.as_deref(), trimmed)
}
