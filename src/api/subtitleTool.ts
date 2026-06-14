import { invoke } from '@tauri-apps/api/core'

export type SubtitleTargetFormat = 'ass' | 'ssa' | 'srt' | 'vtt'

export interface SubtitleFormatJob {
  inputPath: string
  outputPath: string
  targetFormat: SubtitleTargetFormat
}

export interface SubtitleFormatResult {
  outputPath: string
  logs: string[]
}

export function previewSubtitleFormatCommand(job: SubtitleFormatJob) {
  return invoke<string[]>('preview_subtitle_format_command', { job })
}

export function convertSubtitleFormat(job: SubtitleFormatJob) {
  return invoke<SubtitleFormatResult>('convert_subtitle_format', { job })
}
