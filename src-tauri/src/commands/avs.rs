use crate::models::avs_status::{AvsStatus, LavFiltersStatus};
use crate::services::{avs_detector, config_store, ffmpeg_locator};
use tauri::AppHandle;

#[tauri::command]
pub async fn detect_avs(app: AppHandle) -> Result<AvsStatus, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let config = config_store::load(&app)?;
        let ffmpeg_status = ffmpeg_locator::detect(&config);
        let ffmpeg_path = ffmpeg_status.ffmpeg_path.as_deref();
        Ok(avs_detector::detect(ffmpeg_path))
    })
    .await
    .map_err(|err| format!("AVS detection task failed: {err}"))?
}

#[tauri::command]
pub async fn detect_lav_filters() -> Result<LavFiltersStatus, String> {
    tauri::async_runtime::spawn_blocking(avs_detector::detect_lav_filters_status)
        .await
        .map_err(|err| format!("LAV Filters detection task failed: {err}"))
}
