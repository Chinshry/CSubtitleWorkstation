import { invoke } from '@tauri-apps/api/core'
import type { CompressJob } from '../types'

export interface SubtitleAnalysisResult {
  hasEffects: boolean
  detectedTags: string[]
  /** ASS [Script Info] 段中 YCbCr Matrix 的原始字符串，例如 "TV.709"、"PC.601"；缺失为 undefined */
  assMatrix?: string
  missingImgPaths: Array<{
    path: string
    resolvedPath: string
    line: number
    tag: string
  }>
  missingFonts: Array<{
    font: string
    source: string
    line?: number
  }>
  missingStyles: Array<{
    style: string
    line: number
  }>
}

export function previewFfmpegCommand(job: CompressJob) {
  return invoke<string[]>('preview_ffmpeg_command', { job })
}

export function analyzeSubtitle(subtitlePath: string) {
  return invoke<SubtitleAnalysisResult>('analyze_subtitle', { subtitlePath })
}

export function validateOutputParentDir(outputPath: string) {
  return invoke<void>('validate_output_parent_dir', { outputPath })
}

export function startCompress(job: CompressJob) {
  return invoke<void>('start_compress', { job })
}

export function cancelCompress(jobId: string) {
  return invoke<void>('cancel_compress', { jobId })
}
