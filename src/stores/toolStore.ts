import { ref } from 'vue'

export type ToolId = 'proofread' | 'text-conversion' | 'cc-subtitle' | 'subtitle-format' | 'media-remux'

export const activeTool = ref<ToolId>('proofread')
