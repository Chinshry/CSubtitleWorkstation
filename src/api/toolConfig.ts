import { invoke } from '@tauri-apps/api/core'
import type { CcSubtitleConfig, ProofreadConfig, TextConversionConfig } from '../types'

export function loadTextConversionConfig() {
  return invoke<TextConversionConfig>('load_text_conversion_config')
}

export function saveTextConversionConfig(config: TextConversionConfig) {
  return invoke<void>('save_text_conversion_config', { config })
}

export function loadProofreadConfig() {
  return invoke<ProofreadConfig>('load_proofread_config')
}

export function saveProofreadConfig(config: ProofreadConfig) {
  return invoke<void>('save_proofread_config', { config })
}

export function loadCcSubtitleConfig() {
  return invoke<CcSubtitleConfig>('load_cc_subtitle_config')
}

export function saveCcSubtitleConfig(config: CcSubtitleConfig) {
  return invoke<void>('save_cc_subtitle_config', { config })
}
