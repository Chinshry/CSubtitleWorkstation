use tauri::AppHandle;

#[tauri::command]
pub fn get_current_app_version(app: AppHandle) -> String {
    app.package_info().version.to_string()
}
