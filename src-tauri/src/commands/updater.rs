use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;

#[tauri::command]
pub fn get_current_app_version(app: AppHandle) -> String {
    app.package_info().version.to_string()
}

#[tauri::command]
pub async fn check_app_update(app: AppHandle) -> Result<Option<String>, String> {
    let update = app
        .updater()
        .map_err(|err| format!("更新器初始化失败: {err}"))?
        .check()
        .await
        .map_err(|err| format!("检查更新失败: {err}"))?;

    Ok(update.map(|item| item.version.to_string()))
}
