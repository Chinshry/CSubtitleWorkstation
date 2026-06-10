import { invoke } from '@tauri-apps/api/core'

export type CcSubtitleResult = {
  text: string
  changedLines: number
  insertedLines: number
  replacementCount: number
}

export type SavedCcSubtitleFile = {
  outputPath: string
}

export type CcReplacementRule = {
  replacement: string
  pattern: string
}

export function organizeCcSubtitleText(text: string, replacementRules: CcReplacementRule[] = []) {
  return invoke<CcSubtitleResult>('organize_cc_subtitle_text', { text, replacementRules })
}

export function readCcSubtitleFile(path: string) {
  return invoke<string>('read_cc_subtitle_file', { path })
}

export function saveCcSubtitleFile(path: string, text: string, suffix: string, overwrite = false) {
  return invoke<SavedCcSubtitleFile>('save_cc_subtitle_file', { path, text, suffix, overwrite })
}

export function saveCcSubtitleToPath(path: string, text: string) {
  return invoke<SavedCcSubtitleFile>('save_cc_subtitle_to_path', { path, text })
}
