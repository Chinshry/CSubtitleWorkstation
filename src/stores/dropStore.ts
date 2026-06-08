import { ref } from 'vue'
import type { ToolId } from './toolStore'

export type PendingDrop = {
  target: 'home' | 'tools'
  tool?: ToolId
  videoPath?: string
  subtitlePath?: string
  textPath?: string
  raw: string[]
  receivedAt: number
}

export const pendingDrop = ref<PendingDrop | null>(null)
export const globalDragActive = ref(false)

export function pushDiag(line: string) {
  // 仅写到 devtools 控制台，不再渲染到 UI
  console.log('[diag]', line)
}
