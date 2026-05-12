import { invoke } from '@tauri-apps/api/core'
import type { VideoMeta } from '../types'

export function inspectVideoMeta(path: string) {
  return invoke<VideoMeta>('inspect_video_meta', { path })
}
