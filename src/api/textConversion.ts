import { invoke } from '@tauri-apps/api/core'

export type SavedTextConversion = {
  outputPath: string
}

export type ChineseConversionMode = 's2t' | 't2s'

export type CustomConversionRule = {
  from: string
  to: string
}

export function convertChineseText(
  text: string,
  mode: ChineseConversionMode,
  customRules: CustomConversionRule[] = []
) {
  return invoke<string>('convert_chinese_text', { text, mode, customRules })
}

export function readPlainTextFile(path: string) {
  return invoke<string>('read_plain_text_file', { path })
}

export function saveConvertedTextFile(
  path: string,
  text: string,
  suffix: string,
  overwrite = false
) {
  return invoke<SavedTextConversion>('save_converted_text_file', { path, text, suffix, overwrite })
}
