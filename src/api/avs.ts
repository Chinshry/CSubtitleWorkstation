import { invoke } from '@tauri-apps/api/core'
import type { AvsStatus } from '../types'

export function detectAvs() {
  return invoke<AvsStatus>('detect_avs')
}
