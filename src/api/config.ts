import { invoke } from '@tauri-apps/api/core'
import type { AppConfig, VideoEncodePreset } from '../types'

export function loadConfig() {
  return invoke<AppConfig>('load_config')
}

export function saveConfig(config: AppConfig) {
  return invoke<void>('save_config', { config })
}

export function exportEncodePresets(path: string, presets: VideoEncodePreset[]) {
  return invoke<void>('export_encode_presets', { path, presets })
}

export function importEncodePresets(path: string) {
  return invoke<VideoEncodePreset[]>('import_encode_presets', { path })
}
