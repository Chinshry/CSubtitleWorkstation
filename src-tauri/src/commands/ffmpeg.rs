use crate::models::app_config::FfmpegMode;
use crate::models::ffmpeg_status::FfmpegStatus;
use crate::services::{config_store, ffmpeg_locator};
use std::path::Path;
use tauri::AppHandle;

#[tauri::command]
pub async fn detect_ffmpeg(app: AppHandle) -> Result<FfmpegStatus, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let config = config_store::load(&app)?;
        Ok(ffmpeg_locator::detect(&config))
    })
    .await
    .map_err(|err| format!("ffmpeg detection task failed: {err}"))?
}

#[tauri::command]
pub async fn set_ffmpeg_path(app: AppHandle, path: String) -> Result<FfmpegStatus, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let ffmpeg_path = ffmpeg_locator::normalize_user_path(&path);
        let status = ffmpeg_locator::inspect_path(
            Path::new(&ffmpeg_path),
            crate::models::ffmpeg_status::FfmpegSource::CustomPath,
        );
        if !status.available {
            return Ok(status);
        }

        let mut config = config_store::load(&app)?;
        config.ffmpeg_mode = FfmpegMode::Custom;
        config.ffmpeg_path = Some(ffmpeg_path);
        config_store::save(&app, &config)?;
        Ok(status)
    })
    .await
    .map_err(|err| format!("ffmpeg path update task failed: {err}"))?
}

#[tauri::command]
pub async fn reset_ffmpeg_to_system(app: AppHandle) -> Result<FfmpegStatus, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let mut config = config_store::load(&app)?;
        config.ffmpeg_mode = FfmpegMode::System;
        config.ffmpeg_path = None;
        config_store::save(&app, &config)?;
        Ok(ffmpeg_locator::detect(&config))
    })
    .await
    .map_err(|err| format!("ffmpeg reset task failed: {err}"))?
}
