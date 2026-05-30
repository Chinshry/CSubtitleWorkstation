#[tauri::command]
pub fn get_current_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
