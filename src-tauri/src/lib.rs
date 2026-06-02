mod commands;
mod models;
mod services;

use std::collections::{HashMap, HashSet};
use std::env;
use std::process::ChildStdin;
use std::sync::Mutex;
use tauri::Manager;

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
    // macOS: 加载 shell 环境变量，使 Finder/Dock 启动的应用能找到 Homebrew 工具
    #[cfg(target_os = "macos")]
    load_shell_env();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .manage(AppState::default())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(|app| {
            if let Err(err) = services::temp_cleanup::cleanup_transient_dirs(app.handle()) {
                eprintln!("Failed to cleanup transient cache dirs on startup: {err}");
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::config::load_config,
            commands::config::save_config,
            commands::config::export_encode_presets,
            commands::config::import_encode_presets,
            commands::encoder::get_supported_encoders,
            commands::ffmpeg::detect_ffmpeg,
            commands::ffmpeg::set_ffmpeg_path,
            commands::ffmpeg::reset_ffmpeg_to_system,
            commands::avs::detect_avs,
            commands::compress::preview_ffmpeg_command,
            commands::compress::analyze_subtitle,
            commands::compress::validate_output_parent_dir,
            commands::compress::inspect_avs_staging_plan,
            commands::compress::start_compress,
            commands::compress::cancel_compress,
            commands::video::inspect_video_meta,
            commands::video::extract_video_frame,
            commands::video::clear_frame_cache,
            commands::updater::get_current_app_version
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(target_os = "macos")]
fn load_shell_env() {
    // 尝试从 ~/.zprofile 或 ~/.bash_profile 加载环境变量
    let home = match env::var("HOME") {
        Ok(h) => h,
        Err(_) => return,
    };

    let zprofile = std::path::PathBuf::from(&home).join(".zprofile");
    let bash_profile = std::path::PathBuf::from(&home).join(".bash_profile");

    let profile_path = if zprofile.exists() {
        zprofile
    } else if bash_profile.exists() {
        bash_profile
    } else {
        return;
    };

    // 执行 shell 脚本来获取环境变量
    if let Ok(output) = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!(
            "source '{}' && echo \"$PATH\"",
            profile_path.display()
        ))
        .output()
    {
        if output.status.success() {
            if let Ok(path_str) = String::from_utf8(output.stdout) {
                let path = path_str.trim();
                if !path.is_empty() {
                    env::set_var("PATH", path);
                }
            }
        }
    }
}
