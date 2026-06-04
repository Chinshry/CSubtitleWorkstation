import { invoke } from '@tauri-apps/api/core'
import type { AvsStatus, LavFiltersStatus } from '../types'

export function detectAvs() {
  return invoke<AvsStatus>('detect_avs')
}

export function detectLavFilters() {
  return invoke<LavFiltersStatus>('detect_lav_filters')
}
