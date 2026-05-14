mod commands;
mod models;
mod services;

use std::collections::{HashMap, HashSet};
use std::process::ChildStdin;
use std::sync::Mutex;

/// 运行中的压制任务句柄。
/// - `pid`：ffmpeg 进程 id，用于兜底强制终止。
/// - `stdin`：ffmpeg 的标准输入。取消时写入 b"q\n" 触发 ffmpeg 优雅退出（写完文件尾、关流），
///    模拟命令行 Ctrl+C 行为，保证已编码的部分输出仍然可播放。
/// 不保存 Child 本身是为避免 wait 线程长期持锁，与 cancel_compress 形成死锁。
pub struct JobHandle {
    pub pid: u32,
    pub stdin: Option<ChildStdin>,
}

#[derive(Default)]
pub struct AppState {
    pub jobs: Mutex<HashMap<String, JobHandle>>,
    pub cancelled_jobs: Mutex<HashSet<String>>,
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
            commands::encoder::get_supported_encoders,
            commands::ffmpeg::detect_ffmpeg,
            commands::ffmpeg::set_ffmpeg_path,
            commands::ffmpeg::reset_ffmpeg_to_system,
            commands::avs::detect_avs,
            commands::compress::preview_ffmpeg_command,
            commands::compress::analyze_subtitle,
            commands::compress::start_compress,
            commands::compress::cancel_compress,
            commands::video::inspect_video_meta,
            commands::video::extract_video_frame,
            commands::video::clear_frame_cache,
            commands::updater::get_current_app_version,
            commands::updater::check_app_update
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
