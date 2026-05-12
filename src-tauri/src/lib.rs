mod commands;
mod models;
mod services;

use std::collections::HashMap;
use std::process::Child;
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct AppState {
    pub jobs: Mutex<HashMap<String, Arc<Mutex<Child>>>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            commands::config::load_config,
            commands::config::save_config,
            commands::ffmpeg::detect_ffmpeg,
            commands::ffmpeg::set_ffmpeg_path,
            commands::ffmpeg::reset_ffmpeg_to_system,
            commands::avs::detect_avs,
            commands::compress::preview_ffmpeg_command,
            commands::compress::start_compress,
            commands::compress::cancel_compress,
            commands::video::inspect_video_meta,
            commands::updater::get_current_app_version,
            commands::updater::check_app_update
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
