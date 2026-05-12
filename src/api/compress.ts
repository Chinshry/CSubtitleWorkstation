import { invoke } from '@tauri-apps/api/core'
import type { CompressJob } from '../types'

export function previewFfmpegCommand(job: CompressJob) {
  return invoke<string[]>('preview_ffmpeg_command', { job })
}

export function startCompress(job: CompressJob) {
  return invoke<void>('start_compress', { job })
}

export function cancelCompress(jobId: string) {
  return invoke<void>('cancel_compress', { jobId })
}
