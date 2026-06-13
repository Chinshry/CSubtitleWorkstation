import { ref } from 'vue'

export type ToolId = 'proofread' | 'text-conversion' | 'cc-subtitle' | 'media-remux'

export const activeTool = ref<ToolId>('proofread')
