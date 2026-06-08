import { ref } from 'vue'

export type ToolId = 'proofread' | 'text-conversion'

export const activeTool = ref<ToolId>('proofread')
