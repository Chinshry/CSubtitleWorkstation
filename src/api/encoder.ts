import { invoke } from '@tauri-apps/api/core'

export interface EncoderInfo {
  name: string
  label: string
  supported: boolean
}

export function getSupportedEncoders() {
  return invoke<EncoderInfo[]>('get_supported_encoders')
}
