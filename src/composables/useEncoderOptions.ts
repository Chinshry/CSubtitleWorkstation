import { computed, ref } from 'vue'
import { getSupportedEncoders } from '../api/encoder'
import type { EncoderInfo } from '../types'
import { readCachedEncoderOptions, writeCachedEncoderOptions } from '../utils/environmentCache'
import { useI18n } from '../i18n'

export type EncoderOption = {
  value: string | number
  label: string
}

const ENCODER_LABEL_KEYS: Record<string, string> = {
  libx264: 'encoderOptions.libx264',
  libx265: 'encoderOptions.libx265',
  h264_nvenc: 'encoderOptions.h264Nvenc',
  h264_amf: 'encoderOptions.h264Amf',
  h264_videotoolbox: 'encoderOptions.h264Videotoolbox',
}

const FALLBACK_ENCODER_VALUES = [
  'libx264',
  'libx265',
  'h264_nvenc',
  'h264_amf',
  'h264_videotoolbox',
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
  const { t } = useI18n()

  function encoderLabel(name: string, fallback: string) {
    const key = ENCODER_LABEL_KEYS[name]
    return key ? t(key) : fallback
  }

  const encoderOptions = computed<EncoderOption[]>(() => {
    if (!supportedEncoders.value.length) {
      return FALLBACK_ENCODER_VALUES.map((value) => ({
        value,
        label: encoderLabel(value, value),
      }))
    }
    return supportedEncoders.value
      .filter((encoder) => encoder.supported)
      .map((encoder) => ({
        value: encoder.name,
        label: encoderLabel(encoder.name, encoder.label),
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
