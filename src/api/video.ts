import { invoke } from '@tauri-apps/api/core'
import type { VideoMeta } from '../types'

export function inspectVideoMeta(path: string) {
  return invoke<VideoMeta>('inspect_video_meta', { path })
}

export function extractVideoFrame(path: string, timeSeconds: number) {
  return invoke<string>('extract_video_frame', { path, timeSeconds })
}

export function clearFrameCache() {
  return invoke<void>('clear_frame_cache')
}
