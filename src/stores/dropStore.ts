import { ref } from 'vue'

export type PendingDrop = {
  videoPath?: string
  subtitlePath?: string
  raw: string[]
  receivedAt: number
}

export const pendingDrop = ref<PendingDrop | null>(null)
export const globalDragActive = ref(false)

export function pushDiag(line: string) {
  // 仅写到 devtools 控制台，不再渲染到 UI
  console.log('[diag]', line)
}
