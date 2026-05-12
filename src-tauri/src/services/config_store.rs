use crate::models::app_config::AppConfig;
use std::{fs, path::PathBuf};
use tauri::{AppHandle, Manager};

const CONFIG_FILE: &str = "config.json";

pub fn config_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|err| format!("无法获取配置目录: {err}"))?;
    fs::create_dir_all(&dir).map_err(|err| format!("无法创建配置目录: {err}"))?;
    Ok(dir.join(CONFIG_FILE))
}

pub fn load(app: &AppHandle) -> Result<AppConfig, String> {
    let path = config_path(app)?;
    if !path.exists() {
        return Ok(AppConfig::default());
    }

    let raw = fs::read_to_string(&path).map_err(|err| format!("无法读取配置: {err}"))?;
    serde_json::from_str(&raw).map_err(|err| format!("配置格式错误: {err}"))
}

pub fn save(app: &AppHandle, config: &AppConfig) -> Result<(), String> {
    let path = config_path(app)?;
    let raw = serde_json::to_string_pretty(config).map_err(|err| format!("无法序列化配置: {err}"))?;
    fs::write(path, raw).map_err(|err| format!("无法保存配置: {err}"))
}
