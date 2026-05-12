use crate::models::avs_status::AvsStatus;
use crate::services::{avs_detector, config_store, ffmpeg_locator};
use tauri::AppHandle;

#[tauri::command]
pub fn detect_avs(app: AppHandle) -> Result<AvsStatus, String> {
    let config = config_store::load(&app)?;
    let ffmpeg_status = ffmpeg_locator::detect(&config);
    let ffmpeg_path = ffmpeg_status.ffmpeg_path.as_deref();
    Ok(avs_detector::detect(ffmpeg_path))
}
