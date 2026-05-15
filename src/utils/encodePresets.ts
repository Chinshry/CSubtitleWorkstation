import type { AppConfig, CompressJob, VideoEncodePreset } from '../types'

export const DEFAULT_ENCODE_PRESETS: VideoEncodePreset[] = [
  {
    id: 'balanced-x264',
    name: 'x264 平衡',
    encoder: 'libx264',
    crf: 18,
    customVideoArgs: '-preset slow -profile:v high -pix_fmt yuv420p',
    isDefault: true,
  },
  {
    id: 'fast-nvenc',
    name: 'NVENC 快速',
    encoder: 'h264_nvenc',
    crf: 19,
    customVideoArgs: '-rc vbr -cq 19 -b:v 0 -spatial-aq 1 -temporal-aq 1',
  },
  {
    id: 'hevc-small',
    name: 'x265 体积优先',
    encoder: 'libx265',
    crf: 22,
    customVideoArgs: '-preset medium -pix_fmt yuv420p -x265-params aq-mode=1:psy-rd=2.0',
  },
]

export function normalizeEncodePresets(config?: AppConfig | null): VideoEncodePreset[] {
  const saved = Array.isArray(config?.encodePresets) ? config.encodePresets : []
  const savedById = new Map(saved.map((item) => [item.id, item]))
  const raw = [
    ...DEFAULT_ENCODE_PRESETS.map((preset) => savedById.get(preset.id) ?? preset),
    ...saved.filter((preset) => !DEFAULT_ENCODE_PRESETS.some((item) => item.id === preset.id)),
  ]
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
  if (!presets.length) return DEFAULT_ENCODE_PRESETS
  if (!presets.some((item) => item.isDefault)) {
    return presets.map((item, index) => ({ ...item, isDefault: index === 0 }))
  }
  return presets
}

export function getDefaultEncodePreset(config?: AppConfig | null): VideoEncodePreset {
  const presets = normalizeEncodePresets(config)
  return presets.find((item) => item.id === config?.defaultEncodePresetId)
    ?? presets.find((item) => item.isDefault)
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
