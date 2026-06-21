import type { AppConfig, CompressJob, VideoEncodePreset } from '../types'
import { t } from '../i18n'

const DEFAULT_ENCODE_PRESET_BLUEPRINTS: Array<Omit<VideoEncodePreset, 'name'> & { nameKey: string }> = [
  {
    id: 'balanced-x264',
    nameKey: 'encodePresets.balancedX264',
    encoder: 'libx264',
    crf: 18,
    customVideoArgs: '-preset slow -profile:v high -pix_fmt yuv420p',
  },
  {
    id: 'fast-nvenc',
    nameKey: 'encodePresets.fastNvenc',
    encoder: 'h264_nvenc',
    crf: 19,
    customVideoArgs: '-spatial-aq 1 -temporal-aq 1',
  },
  {
    id: 'fast-amf',
    nameKey: 'encodePresets.fastAmf',
    encoder: 'h264_amf',
    crf: 20,
    customVideoArgs: '-quality balanced -pix_fmt yuv420p',
  },
  {
    id: 'fast-videotoolbox',
    nameKey: 'encodePresets.fastVideotoolbox',
    encoder: 'h264_videotoolbox',
    crf: 20,
    maxBitrate: 6000,
    customVideoArgs: '-profile:v high -pix_fmt yuv420p',
  },
  {
    id: 'hevc-small',
    nameKey: 'encodePresets.hevcSmall',
    encoder: 'libx265',
    crf: 22,
    customVideoArgs: '-preset medium -pix_fmt yuv420p -x265-params aq-mode=1:psy-rd=2.0',
  },
]

export function createDefaultEncodePresets(): VideoEncodePreset[] {
  return DEFAULT_ENCODE_PRESET_BLUEPRINTS.map(({ nameKey, ...preset }) => ({
    ...preset,
    name: t(nameKey),
  }))
}

export function normalizeEncodePresets(config?: AppConfig | null): VideoEncodePreset[] {
  const defaults = createDefaultEncodePresets()
  const raw = config?.encodePresets?.length ? config.encodePresets : defaults
  const seen = new Set<string>()
  const presets = raw
    .filter((item): item is VideoEncodePreset => !!item?.id && !!item?.name)
    .map((item) => ({
      ...item,
      crf: clampCrf(item.crf),
      maxBitrate: normalizeBitrate(item.maxBitrate),
      customVideoArgs: item.customVideoArgs ?? '',
    }))
    .filter((item) => {
      if (seen.has(item.id)) return false
      seen.add(item.id)
      return true
    })
  if (!presets.length) return defaults
  return presets
}

export function getDefaultEncodePreset(config?: AppConfig | null): VideoEncodePreset {
  const presets = normalizeEncodePresets(config)
  return presets.find((item) => item.id === config?.defaultEncodePresetId)
    ?? presets[0]
}

export function applyEncodePresetToJob(job: CompressJob, preset: VideoEncodePreset) {
  job.encoder = preset.encoder
  job.crf = clampCrf(preset.crf)
  job.maxBitrate = normalizeBitrate(preset.maxBitrate)
  job.customVideoArgs = preset.customVideoArgs ?? ''
}

function clampCrf(value: number): number {
  if (!Number.isFinite(value)) return 18
  return Math.min(51, Math.max(0, Math.round(value)))
}

function normalizeBitrate(value: number | undefined): number | undefined {
  if (typeof value !== 'number' || !Number.isFinite(value)) return undefined
  if (value < 0) return undefined
  return Math.round(value)
}
