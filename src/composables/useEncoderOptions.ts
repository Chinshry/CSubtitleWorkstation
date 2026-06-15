import { computed, ref } from 'vue'
import { getSupportedEncoders } from '../api/encoder'
import type { EncoderInfo } from '../types'
import { readCachedEncoderOptions, writeCachedEncoderOptions } from '../utils/environmentCache'

export type EncoderOption = {
  value: string | number
  label: string
}

const ENCODER_LABELS: Record<string, string> = {
  libx264: 'CPU libx264（H.264，兼容性最好，支持 AVS）',
  libx265: 'CPU libx265（H.265/HEVC，体积更小，速度较慢）',
  h264_nvenc: 'NVIDIA h264_nvenc（显卡硬编，速度快，不支持 AVS）',
  h264_amf: 'AMD h264_amf（显卡硬编，速度快，不支持 AVS）',
  h264_videotoolbox: 'macOS h264_videotoolbox（Apple 硬编，不支持 AVS）',
}

const FALLBACK_ENCODER_OPTIONS: EncoderOption[] = [
  { value: 'libx264', label: ENCODER_LABELS.libx264 },
  { value: 'libx265', label: ENCODER_LABELS.libx265 },
  { value: 'h264_nvenc', label: ENCODER_LABELS.h264_nvenc },
  { value: 'h264_amf', label: ENCODER_LABELS.h264_amf },
  { value: 'h264_videotoolbox', label: ENCODER_LABELS.h264_videotoolbox },
]

const supportedEncoders = ref<EncoderInfo[]>([])
let initPromise: Promise<void> | null = null

export async function initEncoderOptions(): Promise<void> {
  if (supportedEncoders.value.length) return
  if (initPromise) return initPromise
  initPromise = (async () => {
    const cached = await readCachedEncoderOptions()
    if (cached?.length) {
      supportedEncoders.value = cached
      return
    }
    const next = await getSupportedEncoders()
    supportedEncoders.value = next
    await writeCachedEncoderOptions(next)
  })().finally(() => {
    initPromise = null
  })
  return initPromise
}

export function useEncoderOptions() {
  const encoderOptions = computed<EncoderOption[]>(() => {
    if (!supportedEncoders.value.length) return FALLBACK_ENCODER_OPTIONS
    return supportedEncoders.value
      .filter((encoder) => encoder.supported)
      .map((encoder) => ({
        value: encoder.name,
        label: ENCODER_LABELS[encoder.name] || encoder.label,
      }))
  })

  async function loadEncoderOptions() {
    await initEncoderOptions()
  }

  return {
    encoderOptions,
    loadEncoderOptions,
    supportedEncoders,
  }
}
