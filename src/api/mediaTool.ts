import { invoke } from '@tauri-apps/api/core'

export type MediaToolMode = 'remuxToMp4' | 'concatTsToMp4' | 'addCoverToMp4' | 'mergeAudioVideo'

export interface MediaToolJob {
  id: string
  mode: MediaToolMode
  inputPath: string
  coverPath?: string
  audioPath?: string
  outputPath: string
}

export interface TsSegment {
  path: string
  name: string
  sizeBytes: number
}

export function listTsSegments(folderPath: string) {
  return invoke<TsSegment[]>('list_ts_segments', { folderPath })
}

export function previewMediaToolCommand(job: MediaToolJob) {
  return invoke<string[]>('preview_media_tool_command', { job })
}

export function startMediaTool(job: MediaToolJob) {
  return invoke<void>('start_media_tool', { job })
}

export function cancelMediaTool(jobId: string) {
  return invoke<void>('cancel_media_tool', { jobId })
}
