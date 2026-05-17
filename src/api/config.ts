import { invoke } from '@tauri-apps/api/core'
import type { AppConfig, VideoEncodePreset } from '../types'
import { notifyConfigChanged } from '../stores/configStore'

export function loadConfig() {
  return invoke<AppConfig>('load_config')
}

export async function saveConfig(config: AppConfig) {
  await invoke<void>('save_config', { config })
  notifyConfigChanged()
}

export function exportEncodePresets(path: string, presets: VideoEncodePreset[]) {
  return invoke<void>('export_encode_presets', { path, presets })
}

export function importEncodePresets(path: string) {
  return invoke<VideoEncodePreset[]>('import_encode_presets', { path })
}
