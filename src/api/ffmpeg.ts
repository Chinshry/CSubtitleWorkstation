import { invoke } from '@tauri-apps/api/core'
import type { FfmpegStatus } from '../types'

export function detectFfmpeg() {
  return invoke<FfmpegStatus>('detect_ffmpeg')
}

export function setFfmpegPath(path: string) {
  return invoke<FfmpegStatus>('set_ffmpeg_path', { path })
}

export function resetFfmpegToSystem() {
  return invoke<FfmpegStatus>('reset_ffmpeg_to_system')
}
