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
  /** ASS [Events] 段 Effect 字段命中的 Banner 滚动横幅；非空意味需要 AVS 压制 */
  bannerHits: Array<{
    line: number
    raw: string
  }>
}

export function previewFfmpegCommand(job: CompressJob) {
  return invoke<string[]>('preview_ffmpeg_command', { job: normalizeCompressJob(job) })
}

export function analyzeSubtitle(subtitlePath: string) {
  return invoke<SubtitleAnalysisResult>('analyze_subtitle', { subtitlePath })
}

export function validateOutputParentDir(outputPath: string) {
  return invoke<void>('validate_output_parent_dir', { outputPath })
}

export interface AvsStagingPlan {
  required: boolean
  sourceSizeBytes: number
  sourceSizeLabel: string
  tempPath: string
  reason: string
}

export function inspectAvsStagingPlan(job: CompressJob) {
  return invoke<AvsStagingPlan | null>('inspect_avs_staging_plan', { job: normalizeCompressJob(job) })
}

export function startCompress(job: CompressJob) {
  return invoke<void>('start_compress', { job: normalizeCompressJob(job) })
}

function normalizeCompressJob(job: CompressJob): CompressJob {
  const rawCrf = (job as CompressJob & { crf?: unknown }).crf
  const crf = rawCrf === null || rawCrf === undefined
    ? null
    : typeof rawCrf === 'number' && Number.isFinite(rawCrf)
      ? Math.min(51, Math.max(0, Math.round(rawCrf)))
      : null

  return {
    ...job,
    crf,
    quickProcess: job.quickProcess
      ? {
          ...job.quickProcess,
          transform: job.quickProcess.transform ?? 'none',
          rotation: job.quickProcess.rotation ?? 'none',
          mirror: job.quickProcess.mirror ?? 'none',
          scale: job.quickProcess.scale ?? 'none',
          customScale: job.quickProcess.customScale ?? '',
        }
      : undefined,
  }
}

export function cancelCompress(jobId: string) {
  return invoke<void>('cancel_compress', { jobId })
}
