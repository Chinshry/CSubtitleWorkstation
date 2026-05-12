use crate::models::app_config::AppConfig;
use crate::services::config_store;
use tauri::AppHandle;

#[tauri::command]
pub fn load_config(app: AppHandle) -> Result<AppConfig, String> {
    config_store::load(&app)
}

#[tauri::command]
pub fn save_config(app: AppHandle, config: AppConfig) -> Result<(), String> {
    config_store::save(&app, &config)
}
