use crate::services::{config_store, ffmpeg_locator, frame_extractor, video_meta};
use tauri::AppHandle;

#[tauri::command]
pub fn inspect_video_meta(app: AppHandle, path: String) -> Result<video_meta::VideoMeta, String> {
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

/// 为 LOGO 编辑器抽取视频中指定时间点的一帧 PNG，返回文件绝对路径。
/// 前端通过 `convertFileSrc` 将该路径转换为 webview 可加载的 URL。
#[tauri::command]
pub fn extract_video_frame(
    app: AppHandle,
    path: String,
    time_seconds: f64,
) -> Result<String, String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("视频路径为空".to_string());
    }
    let config = config_store::load(&app)?;
    let status = ffmpeg_locator::detect(&config);
    let ffmpeg_path = status
        .ffmpeg_path
        .ok_or_else(|| "ffmpeg 未配置".to_string())?;
    let cache_dir = frame_extractor::frame_cache_dir(&app)?;
    let frame_path =
        frame_extractor::extract_frame(&ffmpeg_path, trimmed, time_seconds, &cache_dir)?;
    Ok(frame_path.to_string_lossy().to_string())
}

/// 清空抽帧缓存（编辑器关闭时调用，可选）
#[tauri::command]
pub fn clear_frame_cache(app: AppHandle) -> Result<(), String> {
    let dir = frame_extractor::frame_cache_dir(&app)?;
    frame_extractor::cleanup_frame_cache(&dir);
    Ok(())
}
