import { invoke } from '@tauri-apps/api/core'

export function getCurrentAppVersion() {
  return invoke<string>('get_current_app_version')
}

export function checkAppUpdate() {
  return invoke<string | null>('check_app_update')
}
