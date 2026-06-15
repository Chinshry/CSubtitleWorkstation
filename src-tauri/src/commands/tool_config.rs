use crate::models::tool_config::{CcSubtitleConfig, ProofreadConfig, TextConversionConfig};
use crate::services::tool_config_store;
use tauri::AppHandle;

const TOOLBOX_FILE: &str = "toolbox.yaml";

#[tauri::command]
pub fn load_text_conversion_config(app: AppHandle) -> Result<TextConversionConfig, String> {
    tool_config_store::load_text_conversion(&app, TOOLBOX_FILE)
}

#[tauri::command]
pub fn save_text_conversion_config(
    app: AppHandle,
    config: TextConversionConfig,
) -> Result<(), String> {
    tool_config_store::save_text_conversion(&app, TOOLBOX_FILE, &config)
}

#[tauri::command]
pub fn load_proofread_config(app: AppHandle) -> Result<ProofreadConfig, String> {
    tool_config_store::load_proofread(&app, TOOLBOX_FILE)
}

#[tauri::command]
pub fn save_proofread_config(app: AppHandle, config: ProofreadConfig) -> Result<(), String> {
    tool_config_store::save_proofread(&app, TOOLBOX_FILE, &config)
}

#[tauri::command]
pub fn load_cc_subtitle_config(app: AppHandle) -> Result<CcSubtitleConfig, String> {
    tool_config_store::load_cc_subtitle(&app, TOOLBOX_FILE)
}

#[tauri::command]
pub fn save_cc_subtitle_config(app: AppHandle, config: CcSubtitleConfig) -> Result<(), String> {
    tool_config_store::save_cc_subtitle(&app, TOOLBOX_FILE, &config)
}
