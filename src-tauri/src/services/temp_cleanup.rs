use std::fs;
use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager};

const FILTER_TEMP_DIR: &str = "filter-temp";
const LOGO_FRAME_DIR: &str = "logo-editor-frames";
const AVS_TEMP_DIR: &str = "avs-temp";

pub fn cleanup_transient_dirs(app: &AppHandle) -> Result<(), String> {
    let base = app
        .path()
        .app_local_data_dir()
        .map_err(|err| format!("获取 app_local_data_dir 失败: {err}"))?;

    let targets = [
        base.join(FILTER_TEMP_DIR),
        base.join(LOGO_FRAME_DIR),
        base.join(AVS_TEMP_DIR),
    ];

    let mut errors = Vec::new();
    for target in targets {
        if let Err(err) = remove_dir_if_exists(&target) {
            errors.push(format!("{}: {err}", target.display()));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(format!("清理临时缓存失败: {}", errors.join("; ")))
    }
}

pub fn filter_temp_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_local_data_dir()
        .map_err(|err| format!("获取 app_local_data_dir 失败: {err}"))
        .map(|dir| dir.join(FILTER_TEMP_DIR))
}

fn remove_dir_if_exists(path: &Path) -> Result<(), std::io::Error> {
    match fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err),
    }
}
