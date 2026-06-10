use crate::models::app_config::AppConfig;
use crate::models::app_config::{OutputNameTemplate, VideoEncodePreset};
use crate::services::config_store;
use serde::{Deserialize, Serialize};
use std::fs;
use tauri::AppHandle;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EncodePresetBundle {
    version: u8,
    encode_presets: Vec<VideoEncodePreset>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OutputTemplateBundle {
    version: u8,
    output_templates: Vec<OutputNameTemplate>,
}

#[tauri::command]
pub fn load_config(app: AppHandle) -> Result<AppConfig, String> {
    config_store::load(&app)
}

#[tauri::command]
pub fn save_config(app: AppHandle, config: AppConfig) -> Result<(), String> {
    config_store::save(&app, &config)
}

#[tauri::command]
pub fn export_encode_presets(path: String, presets: Vec<VideoEncodePreset>) -> Result<(), String> {
    let bundle = EncodePresetBundle {
        version: 1,
        encode_presets: presets,
    };
    let text = serde_json::to_string_pretty(&bundle)
        .map_err(|err| format!("Failed to serialize encode presets: {err}"))?;
    fs::write(path, text).map_err(|err| format!("Failed to write encode presets: {err}"))
}

#[tauri::command]
pub fn import_encode_presets(path: String) -> Result<Vec<VideoEncodePreset>, String> {
    let text =
        fs::read_to_string(path).map_err(|err| format!("Failed to read encode presets: {err}"))?;
    let value: serde_json::Value = serde_json::from_str(&text)
        .map_err(|err| format!("Failed to parse encode presets JSON: {err}"))?;

    if value.is_array() {
        return serde_json::from_value(value)
            .map_err(|err| format!("Invalid encode preset array: {err}"));
    }

    let bundle: EncodePresetBundle = serde_json::from_value(value)
        .map_err(|err| format!("Invalid encode preset bundle: {err}"))?;
    Ok(bundle.encode_presets)
}

#[tauri::command]
pub fn export_output_templates(
    path: String,
    templates: Vec<OutputNameTemplate>,
) -> Result<(), String> {
    let bundle = OutputTemplateBundle {
        version: 1,
        output_templates: templates,
    };
    let text = serde_json::to_string_pretty(&bundle)
        .map_err(|err| format!("Failed to serialize output templates: {err}"))?;
    fs::write(path, text).map_err(|err| format!("Failed to write output templates: {err}"))
}

#[tauri::command]
pub fn import_output_templates(path: String) -> Result<Vec<OutputNameTemplate>, String> {
    let text =
        fs::read_to_string(path).map_err(|err| format!("Failed to read output templates: {err}"))?;
    let value: serde_json::Value = serde_json::from_str(&text)
        .map_err(|err| format!("Failed to parse output templates JSON: {err}"))?;

    if value.is_array() {
        return serde_json::from_value(value)
            .map_err(|err| format!("Invalid output template array: {err}"));
    }

    let bundle: OutputTemplateBundle = serde_json::from_value(value)
        .map_err(|err| format!("Invalid output template bundle: {err}"))?;
    Ok(bundle.output_templates)
}
