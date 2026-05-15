use crate::models::app_config::FfmpegMode;
use crate::models::ffmpeg_status::FfmpegStatus;
use crate::services::{config_store, ffmpeg_locator};
use std::path::Path;
use tauri::AppHandle;

#[tauri::command]
pub fn detect_ffmpeg(app: AppHandle) -> Result<FfmpegStatus, String> {
    let config = config_store::load(&app)?;
    Ok(ffmpeg_locator::detect(&config))
}

#[tauri::command]
pub fn set_ffmpeg_path(app: AppHandle, path: String) -> Result<FfmpegStatus, String> {
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
}

#[tauri::command]
pub fn reset_ffmpeg_to_system(app: AppHandle) -> Result<FfmpegStatus, String> {
    let mut config = config_store::load(&app)?;
    config.ffmpeg_mode = FfmpegMode::System;
    config.ffmpeg_path = None;
    config_store::save(&app, &config)?;
    Ok(ffmpeg_locator::detect(&config))
}
