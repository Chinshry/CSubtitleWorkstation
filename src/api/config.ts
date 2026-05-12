import { invoke } from '@tauri-apps/api/core'
import type { AppConfig } from '../types'

export function loadConfig() {
  return invoke<AppConfig>('load_config')
}

export function saveConfig(config: AppConfig) {
  return invoke<void>('save_config', { config })
}
