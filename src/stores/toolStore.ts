import { ref } from 'vue'
import type { MediaToolMode } from '../api/mediaTool'

export type TextToolId = 'proofread' | 'text-conversion' | 'cc-subtitle' | 'subtitle-format'
export type MediaToolId = 'media-remux' | 'media-concat-ts' | 'media-cover' | 'media-merge-av'
export type ToolId = TextToolId | MediaToolId

export const mediaToolModeByToolId: Record<MediaToolId, MediaToolMode> = {
  'media-remux': 'remuxToMp4',
  'media-concat-ts': 'concatTsToMp4',
  'media-cover': 'addCoverToMp4',
  'media-merge-av': 'mergeAudioVideo'
}

export const mediaToolIdByMode: Record<MediaToolMode, MediaToolId> = {
  remuxToMp4: 'media-remux',
  concatTsToMp4: 'media-concat-ts',
  addCoverToMp4: 'media-cover',
  mergeAudioVideo: 'media-merge-av'
}

export function isMediaToolId(tool: ToolId): tool is MediaToolId {
  return tool in mediaToolModeByToolId
}

export const activeTool = ref<ToolId>('proofread')
export const activeMediaToolMode = ref<MediaToolMode>('remuxToMp4')
