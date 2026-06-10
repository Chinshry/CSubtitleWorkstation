import { ref } from 'vue'

export type ToolId = 'proofread' | 'text-conversion' | 'cc-subtitle'

export const activeTool = ref<ToolId>('proofread')
