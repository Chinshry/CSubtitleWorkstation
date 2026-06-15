import { invoke } from '@tauri-apps/api/core'
import type { EncoderInfo } from '../types'

export function getSupportedEncoders() {
  return invoke<EncoderInfo[]>('get_supported_encoders')
}
