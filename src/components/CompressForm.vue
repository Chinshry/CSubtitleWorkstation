<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import type { CompressJob, QuickProcessSettings, VideoEncodePreset } from '../types'
import { isWindows } from '../stores/platformStore'
import { avsStatus, lavChecking, lavStatusLoaded } from '../stores/avsStore'
import { analyzeSubtitle, type SubtitleAnalysisResult } from '../api/compress'
import { useEncoderOptions } from '../composables/useEncoderOptions'
import { useToast } from '../composables/useToast'
import EncodeSettingsFields from './EncodeSettingsFields.vue'
import AppSelect from './AppSelect.vue'
import InfoHint from './InfoHint.vue'
import { useI18n } from '../i18n'

const job = defineModel<CompressJob>({ required: true })

const props = defineProps<{
  encodePresets?: VideoEncodePreset[]
  selectedEncodePresetId?: string
  /** 配置 LOGO 按钮是否禁用（一般在视频未就绪时禁用） */
  logoButtonDisabled?: boolean
  /** 配置 LOGO 按钮禁用原因（用于 tooltip） */
  logoButtonDisabledReason?: string
  /** 当前视频编码（来自 VideoMeta）。VP9 在 AVS fallback 下经 DirectShow 解码，依赖 64 位 LAV Filters */
  videoCodec?: string
}>()

const emit = defineEmits<{
  (e: 'open-logo-editor'): void
  (e: 'update:selected-encode-preset-id', value: string): void
  (e: 'apply-encode-preset', value?: string): void
  (e: 'subtitle-analyzing', value: boolean): void
  /** 字幕被分析后，把后端结果透传给上层（HomeView 用来做色彩矩阵匹配） */
  (e: 'subtitle-analyzed', result: SubtitleAnalysisResult | null): void
}>()

// 支持的编码器列表和自动启用 AVS 的原因
const { encoderOptions, loadEncoderOptions } = useEncoderOptions()
const avsAutoEnabledReason = ref<string>('')
const detectedTagsDisplay = ref<string[]>([])
const advancedOpen = ref(false)
const presetMenuOpen = ref(false)
const toast = useToast()
const { t } = useI18n()
let subtitleAnalyzeSeq = 0

const defaultQuickProcess: QuickProcessSettings = {
  enabled: false,
  transform: 'none' as const,
  rotation: 'none' as const,
  mirror: 'none' as const,
  scale: 'none' as const,
  customScale: '',
  frameRate: undefined,
  videoBitrateKbps: undefined,
}

const quickProcess = computed(() => job.value.quickProcess ?? defaultQuickProcess)

const quickEnabled = computed({
  get() {
    return quickProcess.value.enabled
  },
  set(value: boolean) {
    updateQuickProcess({ enabled: value })
  }
})

const quickRotation = computed({
  get() {
    return quickProcess.value.rotation ?? legacyRotationFromTransform(quickProcess.value.transform)
  },
  set(value: string | number) {
    updateQuickProcess({ rotation: String(value) as QuickProcessSettings['rotation'], transform: 'none' })
  }
})

const quickMirror = computed({
  get() {
    return quickProcess.value.mirror ?? legacyMirrorFromTransform(quickProcess.value.transform)
  },
  set(value: string | number) {
    updateQuickProcess({ mirror: String(value) as QuickProcessSettings['mirror'], transform: 'none' })
  }
})

const quickScale = computed({
  get() {
    return quickProcess.value.scale
  },
  set(value: string | number) {
    updateQuickProcess({ scale: String(value) as QuickProcessSettings['scale'] })
  }
})

const quickCustomScale = computed({
  get() {
    return quickProcess.value.customScale
  },
  set(value: string) {
    updateQuickProcess({ customScale: value })
  }
})

const quickFrameRate = computed<number | undefined>({
  get() {
    return quickProcess.value.frameRate
  },
  set(value) {
    updateQuickProcess({ frameRate: normalizePositiveNumber(value) })
  }
})

const quickVideoBitrate = computed<number | undefined>({
  get() {
    return quickProcess.value.videoBitrateKbps
  },
  set(value) {
    updateQuickProcess({ videoBitrateKbps: normalizePositiveInteger(value) })
  }
})

const quickSummary = computed(() => {
  if (!quickProcess.value.enabled) return t('compressForm.quick.off')
  const parts: string[] = []
  const rotation = quickRotationOptions.value.find((item) => item.value === quickRotation.value)
  const mirror = quickMirrorOptions.value.find((item) => item.value === quickMirror.value)
  const scale = quickScaleOptions.value.find((item) => item.value === quickProcess.value.scale)
  if (rotation && rotation.value !== 'none') parts.push(rotation.label)
  if (mirror && mirror.value !== 'none') parts.push(mirror.label)
  if (scale && scale.value !== 'none') {
    parts.push(scale.value === 'custom'
      ? t('compressForm.quick.customScaleSummary', { value: quickProcess.value.customScale || t('compressForm.quick.custom') })
      : scale.label)
  }
  if (quickProcess.value.frameRate) parts.push(`${quickProcess.value.frameRate} fps`)
  if (quickProcess.value.videoBitrateKbps) parts.push(`${quickProcess.value.videoBitrateKbps} Kbps`)
  return parts.length ? parts.join(' · ') : t('compressForm.quick.noneSelected')
})

const quickRotationOptions = computed<Array<{ value: QuickProcessSettings['rotation'], label: string }>>(() => [
  { value: 'none', label: t('compressForm.quick.rotation.none') },
  { value: 'rotate_cw', label: t('compressForm.quick.rotation.cw') },
  { value: 'rotate_ccw', label: t('compressForm.quick.rotation.ccw') },
  { value: 'rotate_180', label: t('compressForm.quick.rotation.rotate180') },
])

const quickMirrorOptions = computed<Array<{ value: QuickProcessSettings['mirror'], label: string }>>(() => [
  { value: 'none', label: t('compressForm.quick.mirror.none') },
  { value: 'hflip', label: t('compressForm.quick.mirror.hflip') },
  { value: 'vflip', label: t('compressForm.quick.mirror.vflip') },
])

const quickScaleOptions = computed<Array<{ value: QuickProcessSettings['scale'], label: string }>>(() => [
  { value: 'none', label: t('compressForm.quick.scale.none') },
  { value: 'landscape_4k', label: t('compressForm.quick.scale.landscape4k') },
  { value: 'landscape_1080', label: t('compressForm.quick.scale.landscape1080') },
  { value: 'landscape_720', label: t('compressForm.quick.scale.landscape720') },
  { value: 'portrait_1080', label: t('compressForm.quick.scale.portrait1080') },
  { value: 'portrait_720', label: t('compressForm.quick.scale.portrait720') },
  { value: 'custom', label: t('compressForm.quick.custom') },
])

const encodePresetOptions = computed(() => {
  return (props.encodePresets ?? []).map((preset) => ({
    value: preset.id,
    label: preset.name,
    description: describeEncodePreset(preset),
    title: buildEncodePresetCommandSummary(preset),
  }))
})

const selectedEncodePresetModel = computed({
  get() {
    return props.selectedEncodePresetId ?? encodePresetOptions.value[0]?.value ?? ''
  },
  set(value: string | number) {
    emit('update:selected-encode-preset-id', String(value))
  }
})

const selectedEncodePresetName = computed(() => {
  return encodePresetOptions.value.find((item) => item.value === selectedEncodePresetModel.value)?.label ?? ''
})

// 副标题：仅 Windows 提示可启用 AVS，其它平台说明走 filter 模式
const avsHint = computed(() => {
  if (avsAutoEnabledReason.value) {
    return avsAutoEnabledReason.value
  }
  return ''
})

// AVS 开关的完整提示
const avsToggleTip = computed(() => {
  return t('compressForm.avs.toggleTip')
})

// 当前视频是否 VP9：VP9 在 AVS fallback 下经 DirectShow 解码链读取，依赖 64 位 LAV Filters
const isVp9 = computed(() => props.videoCodec?.toLowerCase() === 'vp9')

// AVS 环境本身是否就绪（仅 Windows + AviSynth/demuxer 检测通过），不含 LAV
const avsEnvUnavailable = computed(() => !isWindows.value || !avsStatus.value?.available)

// VP9 视频且 LAV 检测结果尚未就绪：解码支持未知，暂时锁住 AVS 开关（仅环境就绪时才有意义）
const lavResolvingForVp9 = computed(
  () => !avsEnvUnavailable.value && isVp9.value && (lavChecking.value || !lavStatusLoaded.value)
)

// AVS 开关启用条件：仅 Windows + 检测通过；VP9 视频的 LAV 检测尚未出结果时暂时禁用
const avsToggleDisabled = computed(() => avsEnvUnavailable.value || lavResolvingForVp9.value)
const avsToggleDisabledTip = computed(() => {
  if (!isWindows.value) return t('compressForm.avs.windowsOnly')
  const s = avsStatus.value
  if (!s) return t('compressForm.avs.checking')
  if (avsEnvUnavailable.value) return s.message ?? t('compressForm.avs.unavailable')
  if (lavResolvingForVp9.value) return t('compressForm.avs.lavResolving')
  return ''
})

// VP9 视频 + LAV 检测中：解码支持未知，显示中性「检测中」提示
const lavCheckingHint = computed(() => lavResolvingForVp9.value)
const lavCheckingTip = computed(() => t('compressForm.avs.lavCheckingTip'))

// VP9 解码依赖：仅 VP9 视频在 AVS fallback 下需要 64 位 LAV Filters，缺失时仅提示（不禁用）。
// 读 avsStatus（已叠加调试 mock），故「模拟 LAV 缺失」与真机缺失都会触发。
const lavMissingHint = computed(
  () =>
    isVp9.value &&
    !avsEnvUnavailable.value &&
    lavStatusLoaded.value &&
    !lavChecking.value &&
    avsStatus.value?.lavFiltersInstalled === false
)
const lavMissingTip = computed(() => t('compressForm.avs.lavMissingTip'))

// 平台不支持或 mock 切换导致已勾选但不可用时，强制关掉
function syncAvsAvailability() {
  if (avsToggleDisabled.value && job.value.useAvs) {
    job.value.useAvs = false
  }
}

// AVS 状态变化时（含调试 mock 切换）自动同步
watch(avsToggleDisabled, syncAvsAvailability)
watch(() => job.value.subtitlePath, (path) => {
  if (!path?.trim() && job.value.useAvs) {
    job.value.useAvs = false
  }
})

// 字幕分析：检测是否包含特效标签，自动勾选 AVS；同时把结果透传给上层用于色彩矩阵匹配
async function analyzeSubtitleForEffects() {
  const subtitlePath = job.value.subtitlePath?.trim()
  const seq = ++subtitleAnalyzeSeq
  if (!subtitlePath) {
    avsAutoEnabledReason.value = ''
    detectedTagsDisplay.value = []
    emit('subtitle-analyzing', false)
    emit('subtitle-analyzed', null)
    return
  }

  let result: SubtitleAnalysisResult | null = null
  emit('subtitle-analyzing', true)
  emit('subtitle-analyzed', null)
  try {
    result = await analyzeSubtitle(subtitlePath)
  } catch (err) {
    if (seq !== subtitleAnalyzeSeq) return
    console.error('Failed to analyze subtitle:', err)
    avsAutoEnabledReason.value = ''
    detectedTagsDisplay.value = []
    emit('subtitle-analyzed', null)
    return
  } finally {
    if (seq === subtitleAnalyzeSeq) {
      emit('subtitle-analyzing', false)
    }
  }

  if (seq !== subtitleAnalyzeSeq) {
    return
  }

  // 仅在 Windows 且 AVS 环境可用时自动勾选 AVS；其它平台只是检测信息
  if (result.hasEffects && isWindows.value && avsStatus.value?.available) {
    detectedTagsDisplay.value = result.detectedTags
    avsAutoEnabledReason.value = t('compressForm.avs.autoEnabled', { tags: result.detectedTags.join('、') })
    job.value.useAvs = true
  } else {
    avsAutoEnabledReason.value = ''
    detectedTagsDisplay.value = []
  }
  // 矩阵信息无论平台都需要透传，banner 判定逻辑在外层
  emit('subtitle-analyzed', result)
}

// 监听字幕路径变化
watch(() => job.value.subtitlePath, analyzeSubtitleForEffects, { immediate: false })

onMounted(() => {
  document.addEventListener('mousedown', closePresetMenuOnOutside)
  void loadEncoderOptions().catch((err) => {
    console.error('Failed to get supported encoders:', err)
  })
  syncAvsAvailability()
})

onBeforeUnmount(() => {
  document.removeEventListener('mousedown', closePresetMenuOnOutside)
})
// 压制预设提示：解释作用 + 引导用户去哪儿管理预设
const presetTip = computed(() => t('compressForm.preset.tip'))

// 已配置 LOGO 的摘要文案
// 直接给百分比对用户不直观，改成「方位（九宫格） + 像素尺寸」
const logoSummary = computed(() => {
  const layout = job.value.logoLayout
  if (!layout || !layout.path) return ''
  const video_name = logoBasename(layout.path)
  return t('compressForm.logo.summary', {
    name: video_name,
    position: describeLogoPosition(layout),
    size: describeLogoSize(layout)
  })
})

// LOGO 层级：AVS 模式下 VSFilterMod TextSubMod 把字幕烧进 AVS 输出，
// ffmpeg 滤镜无法再插入到字幕之下，故 AVS 启用时 LOGO 强制在字幕上、禁用切换。
const logoLayerDisabled = computed(() => !!job.value.useAvs)
// AppSelect 仅接受 string | number，做一层 bool ↔ string 适配。
// AVS 模式下视觉上锁定为 'top'，不写回 job.logoOnTop（保留用户上次选择）
const logoLayerValue = computed<'top' | 'bottom'>({
  get() {
    if (logoLayerDisabled.value) return 'top'
    return job.value.logoOnTop ? 'top' : 'bottom'
  },
  set(v) {
    if (logoLayerDisabled.value) return
    job.value.logoOnTop = v === 'top'
  }
})

function describeLogoPosition(layout: NonNullable<typeof job.value.logoLayout>): string {
  // LOGO 中心点占视频画面的百分比（取中心点更符合"摆在哪个角落"的语感）
  const cx = layout.xPct + layout.wPct / 2
  const cy = layout.yPct + layout.hPct / 2
  const horiz = cx < 0.34 ? 'left' : cx < 0.67 ? 'center' : 'right'
  const vert = cy < 0.34 ? 'top' : cy < 0.67 ? 'middle' : 'bottom'
  if (horiz === 'center' && vert === 'middle') return t('compressForm.logo.position.center')
  if (horiz === 'center') return vert === 'top' ? t('compressForm.logo.position.topCenter') : t('compressForm.logo.position.bottomCenter')
  if (vert === 'middle') return horiz === 'left' ? t('compressForm.logo.position.leftCenter') : t('compressForm.logo.position.rightCenter')
  return t(`compressForm.logo.position.${vert}${horiz[0].toUpperCase()}${horiz.slice(1)}`)
}

function describeLogoSize(layout: NonNullable<typeof job.value.logoLayout>): string {
  const vw = job.value.videoWidth
  const vh = job.value.videoHeight
  if (vw && vh && vw > 0 && vh > 0) {
    const w = Math.round(layout.wPct * vw)
    const h = Math.round(layout.hPct * vh)
    return t('compressForm.logo.sizePixels', { width: w, height: h })
  }
  // 视频分辨率未就绪时退回到百分比
  return `${(layout.wPct * 100).toFixed(1)}% × ${(layout.hPct * 100).toFixed(1)}%`
}

function logoBasename(p: string): string {
  if (!p) return ''
  const idx = Math.max(p.lastIndexOf('\\'), p.lastIndexOf('/'))
  return idx >= 0 ? p.slice(idx + 1) : p
}

function legacyRotationFromTransform(transform?: QuickProcessSettings['transform']): QuickProcessSettings['rotation'] {
  if (transform === 'rotate_cw' || transform === 'rotate_cw_flip') return 'rotate_cw'
  if (transform === 'rotate_ccw' || transform === 'rotate_ccw_flip') return 'rotate_ccw'
  if (transform === 'rotate_180') return 'rotate_180'
  return 'none'
}

function legacyMirrorFromTransform(transform?: QuickProcessSettings['transform']): QuickProcessSettings['mirror'] {
  if (transform === 'rotate_cw_flip' || transform === 'rotate_ccw_flip' || transform === 'vflip') return 'vflip'
  if (transform === 'hflip') return 'hflip'
  return 'none'
}

function onOpenLogoEditor() {
  if (props.logoButtonDisabled) return
  emit('open-logo-editor')
}

function togglePresetMenu() {
  presetMenuOpen.value = !presetMenuOpen.value
}

function closePresetMenuOnOutside(event: MouseEvent) {
  const target = event.target as HTMLElement | null
  if (target?.closest('.preset-picker')) return
  presetMenuOpen.value = false
}

function applyEncodePresetOption(presetId: string, presetName: string) {
  emit('update:selected-encode-preset-id', presetId)
  emit('apply-encode-preset', presetId)
  presetMenuOpen.value = false
  toast.success(presetName ? t('compressForm.preset.appliedNamed', { name: presetName }) : t('compressForm.preset.applied'), 2500)
}

function describeEncodePreset(preset: VideoEncodePreset): string {
  return [
    preset.encoder,
    t('compressForm.preset.qualitySummary', { crf: preset.crf }),
    describePresetBitrate(preset.maxBitrate),
  ].join(' · ')
}

function describePresetBitrate(maxBitrate: number | undefined): string {
  if (maxBitrate === undefined) return t('compressForm.preset.unlimitedBitrate')
  if (maxBitrate === 0) return t('compressForm.preset.autoBitrate')
  return `${maxBitrate} Kbps`
}

function buildEncodePresetCommandSummary(preset: VideoEncodePreset): string {
  const args = ['-c:v', preset.encoder]
  if (preset.encoder === 'h264_nvenc') {
    args.push('-rc', 'vbr', '-cq', String(preset.crf), '-b:v', '0')
  } else if (preset.encoder === 'h264_amf') {
    args.push('-rc', 'cqp', '-qp_i', String(preset.crf), '-qp_p', String(preset.crf), '-qp_b', String(preset.crf))
  } else if (preset.encoder === 'h264_videotoolbox') {
    if (typeof preset.maxBitrate === 'number' && preset.maxBitrate > 0) {
      args.push('-b:v', `${preset.maxBitrate}k`)
    }
  } else {
    args.push('-crf', String(preset.crf))
  }
  if (typeof preset.maxBitrate === 'number' && preset.maxBitrate > 0) {
    args.push('-maxrate', `${preset.maxBitrate}k`, '-bufsize', `${preset.maxBitrate * 2}k`)
  } else if (preset.maxBitrate === 0) {
    args.push('-maxrate', t('compressForm.preset.sourceBitratePlus'), '-bufsize', t('compressForm.preset.doubleMaxBitrate'))
  }
  const custom = preset.customVideoArgs?.trim()
  if (custom) args.push(custom)
  return args.join(' ')
}

function updateQuickProcess(patch: Partial<QuickProcessSettings>) {
  job.value = {
    ...job.value,
    quickProcess: {
    ...defaultQuickProcess,
    ...job.value.quickProcess,
    ...patch,
    }
  }
}

function normalizePositiveNumber(value: number | undefined): number | undefined {
  if (typeof value !== 'number' || !Number.isFinite(value) || value <= 0) return undefined
  return Math.round(value * 1000) / 1000
}

function normalizePositiveInteger(value: number | undefined): number | undefined {
  if (typeof value !== 'number' || !Number.isFinite(value) || value <= 0) return undefined
  return Math.round(value)
}
</script>

<template>
  <section class="panel">
    <div class="panel-heading compress-heading">
      <div>
        <h2>{{ t('compressForm.title') }}</h2>
      </div>
      <div v-if="encodePresetOptions.length" class="preset-picker">
        <button
          type="button"
          class="secondary preset-menu-trigger"
          v-tooltip="selectedEncodePresetName ? t('compressForm.preset.lastApplied', { name: selectedEncodePresetName }) : t('compressForm.preset.applyTip')"
          @click="togglePresetMenu"
        >
          {{ t('compressForm.preset.apply') }}
        </button>
        <InfoHint
          placement="left"
          :title="t('compressForm.preset.title')"
          :body="t('compressForm.preset.body')"
          :items="[t('compressForm.preset.itemManage'), t('compressForm.preset.itemUseCase')]"
        />
        <div v-if="presetMenuOpen" class="preset-menu">
          <button
            v-for="option in encodePresetOptions"
            :key="option.value"
            type="button"
            class="preset-menu-option"
            v-tooltip="option.title"
            @click="applyEncodePresetOption(String(option.value), option.label)"
          >
            <span>{{ option.label }}</span>
            <small>{{ option.description }}</small>
          </button>
        </div>
      </div>
    </div>

    <div class="compress-sections">
      <section class="form-section settings-section">
        <div class="quick-process-panel" :class="{ active: quickEnabled }">
          <label class="switch-row quick-process-switch">
            <input v-model="quickEnabled" type="checkbox" />
            <span class="switch"></span>
            <span>{{ t('compressForm.quick.title') }}</span>
            <small v-if="quickEnabled">{{ quickSummary }}</small>
            <InfoHint
              placement="right"
              :title="t('compressForm.quick.title')"
              :body="t('compressForm.quick.body')"
              :items="[t('compressForm.quick.itemUseCase'), t('compressForm.quick.itemReuse')]"
            />
          </label>

          <div v-if="quickEnabled" class="quick-process-grid">
            <label class="quick-field-rotate">
              <span class="quick-field-label">
                {{ t('compressForm.quick.rotate') }}
                <InfoHint
                  placement="right"
                  :title="t('compressForm.quick.rotate')"
                  command="transpose / hflip,vflip"
                  :body="t('compressForm.quick.rotateBody')"
                  :items="[t('compressForm.quick.rotateItemUseCase'), t('compressForm.quick.rotateItem180')]"
                />
              </span>
              <AppSelect v-model="quickRotation" :options="quickRotationOptions" />
            </label>
            <label class="quick-field-mirror">
              <span class="quick-field-label">
                {{ t('compressForm.quick.mirrorLabel') }}
                <InfoHint
                  :title="t('compressForm.quick.mirrorLabel')"
                  command="hflip / vflip"
                  :body="t('compressForm.quick.mirrorBody')"
                  :items="[t('compressForm.quick.mirrorItemH'), t('compressForm.quick.mirrorItemV')]"
                />
              </span>
              <AppSelect v-model="quickMirror" :options="quickMirrorOptions" />
            </label>
            <label class="quick-field-scale">
              <span class="quick-field-label">
                {{ t('compressForm.quick.scaleLabel') }}
                <InfoHint
                  :title="t('compressForm.quick.scaleLabel')"
                  command="scale"
                  :body="t('compressForm.quick.scaleBody')"
                  :items="[t('compressForm.quick.scaleItemLandscape'), t('compressForm.quick.scaleItemPortrait')]"
                />
              </span>
              <AppSelect v-model="quickScale" :options="quickScaleOptions" />
            </label>
            <label v-if="quickScale === 'custom'" class="quick-field-custom-scale">
              <span class="quick-field-label">
                {{ t('compressForm.quick.customScale') }}
                <InfoHint
                  :title="t('compressForm.quick.customScale')"
                  :command="t('compressForm.quick.customScaleCommand')"
                  :body="t('compressForm.quick.customScaleBody')"
                  :items="[t('compressForm.quick.customScaleItemHeight'), t('compressForm.quick.customScaleItemWidth')]"
                />
              </span>
              <input
                v-model.trim="quickCustomScale"
                type="text"
                spellcheck="false"
                :placeholder="t('compressForm.quick.customScalePlaceholder')"
              />
            </label>
            <label class="quick-field-fps">
              <span class="quick-field-label">
                {{ t('compressForm.quick.frameRate') }}
                <InfoHint
                  :title="t('compressForm.quick.frameRate')"
                  command="fps"
                  :body="t('compressForm.quick.frameRateBody')"
                  :items="[t('compressForm.quick.frameRateItemEmpty'), t('compressForm.quick.frameRateItemValues')]"
                />
              </span>
              <span class="quick-inline-input">
                <input v-model.number="quickFrameRate" type="number" min="1" max="240" :placeholder="t('compressForm.quick.noChange')" />
                <em>fps</em>
              </span>
            </label>
            <label class="quick-field-bitrate">
              <span class="quick-field-label">
                {{ t('compressForm.quick.videoBitrate') }}
                <InfoHint
                  :title="t('compressForm.quick.videoBitrate')"
                  command="-b:v"
                  :body="t('compressForm.quick.videoBitrateBody')"
                  :items="[t('compressForm.quick.videoBitrateItemEmpty'), t('compressForm.quick.videoBitrateItemValue')]"
                />
              </span>
              <span class="quick-inline-input">
                <input v-model.number="quickVideoBitrate" type="number" min="1" :placeholder="t('compressForm.quick.noChange')" />
                <em>Kbps</em>
              </span>
            </label>
          </div>
        </div>

        <EncodeSettingsFields v-model="job" :encoder-options="encoderOptions">
          <template #encoder-trailing>
            <button type="button" class="secondary advanced-toggle" @click="advancedOpen = !advancedOpen">
              {{ advancedOpen ? t('compressForm.advanced.hide') : t('compressForm.advanced.show') }}
            </button>
          </template>
        </EncodeSettingsFields>

        <div v-if="advancedOpen" class="advanced-panel">
          <label class="custom-args-field">
            <span>
              {{ t('compressForm.advanced.label') }}
            </span>
            <textarea
              v-model="job.customVideoArgs"
              rows="3"
              spellcheck="false"
            ></textarea>
          </label>
          <p class="advanced-note">
            {{ t('compressForm.advanced.note') }}
          </p>
        </div>
      </section>

      <section class="form-section options-section">
        <div class="compress-option-rows">
          <div class="compress-option-row">
            <label class="switch-row">
              <input v-model="job.needYadif" type="checkbox" />
              <span class="switch"></span>
              <span>{{ t('compressForm.options.yadif') }}</span>
              <InfoHint
                :title="t('compressForm.options.yadifTitle')"
                command="-vf yadif"
                :body="t('compressForm.options.yadifBody')"
                :items="[t('compressForm.options.yadifItemInterlaced'), t('compressForm.options.yadifItemProgressive')]"
              />
            </label>

          <label class="switch-row">
            <input v-model="job.needLogo" type="checkbox" />
            <span class="switch"></span>
            <span>{{ t('compressForm.logo.title') }}</span>
            <InfoHint
              placement="right"
              :title="t('compressForm.logo.title')"
              :body="t('compressForm.logo.body')"
              :items="[t('compressForm.logo.itemConfigure'), t('compressForm.logo.itemKeepLayout')]"
            />
          </label>

            <div v-if="job.needLogo" class="logo-config-inline">
              <button
                type="button"
                class="secondary logo-config-btn"
                :class="{ disabled: logoButtonDisabled }"
                :disabled="logoButtonDisabled"
                v-tooltip="logoButtonDisabled ? logoButtonDisabledReason : t('compressForm.logo.openTip')"
                @click="onOpenLogoEditor"
              >
                {{ job.logoLayout ? t('compressForm.logo.reconfigure') : t('compressForm.logo.configure') }}
              </button>
              <span v-if="logoSummary" class="logo-summary">{{ logoSummary }}</span>
              <span v-else class="logo-summary muted">{{ t('compressForm.logo.notConfigured') }}</span>
            </div>
          </div>

          <div v-if="job.subtitlePath?.trim()" class="compress-option-row subtitle-option-row">
          <label class="switch-row" :class="{ 'switch-row-disabled': avsToggleDisabled }">
            <input
              v-model="job.useAvs"
              type="checkbox"
              :disabled="avsToggleDisabled"
            />
            <span class="switch"></span>
            <span>{{ t('compressForm.avs.title') }}</span>
            <span v-if="avsAutoEnabledReason" class="avs-hint" :data-tip="`${detectedTagsDisplay.join('、')}`">{{ t('compressForm.avs.detectedSpecialTags') }}</span>
            <span v-if="lavCheckingHint" class="avs-checking" :data-tip="lavCheckingTip" tabindex="0">{{ t('compressForm.avs.lavChecking') }}</span>
            <span v-else-if="lavMissingHint" class="avs-warn" :data-tip="lavMissingTip" tabindex="0">{{ t('compressForm.avs.lavMissing') }}</span>
            <InfoHint
              placement="left"
              :title="t('compressForm.avs.title')"
              command="AviSynth+ + VSFilterMod TextSubMod"
              :body="t('compressForm.avs.body')"
              :items="[t('compressForm.avs.itemWindows'), t('compressForm.avs.itemUseCase')]"
            />
          </label>

            <div v-if="job.needLogo" class="logo-layer-control" :class="{ 'logo-layer-disabled': logoLayerDisabled }">
            <span class="logo-layer-label">
              {{ t('compressForm.logo.layer') }}
              <InfoHint
                placement="right"
                :title="t('compressForm.logo.layer')"
                :body="t('compressForm.logo.layerBody')"
                :items="[
                  t('compressForm.logo.layerItemBottom'),
                  t('compressForm.logo.layerItemTop'),
                  t('compressForm.logo.layerItemAvs')
                ]"
              />
            </span>
            <AppSelect
              v-model="logoLayerValue"
              class="logo-layer-select"
              :disabled="logoLayerDisabled"
              :options="[
                { value: 'bottom', label: t('compressForm.logo.layerBottom'), title: t('compressForm.logo.layerBottomTitle') },
                { value: 'top', label: t('compressForm.logo.layerTop'), title: t('compressForm.logo.layerTopTitle') }
              ]"
            />
            </div>
          </div>
        </div>
      </section>
    </div>
  </section>
</template>

<style scoped>
.compress-option-rows {
  display: grid;
  gap: 12px;
  padding: 6px 2px;
}

.compress-option-row,
.logo-config-inline {
  align-items: center;
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
}

.compress-option-row {
  gap: 28px;
}

.subtitle-option-row {
  border-top: 1px solid #e3e9ed;
  margin-top: 4px;
  padding-top: 16px;
}

.compress-sections {
  display: grid;
  gap: 0;
}

.form-section {
  border-bottom: 1px solid #e3e9ed;
  padding-bottom: 16px;
  margin-bottom: 16px;
}

.form-section:last-child {
  border-bottom: none;
  padding-bottom: 0;
  margin-bottom: 0;
}

.settings-section {
  border-bottom: none;
  padding-bottom: 0;
  margin-bottom: 12px;
}

.section-heading {
  color: #43515c;
  font-size: 12.5px;
  font-weight: 600;
  margin-bottom: 12px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
.advanced-toggle {
  align-self: center;
  min-height: 38px;
  padding: 0 14px;
  white-space: nowrap;
}
.advanced-panel {
  background: #f8fafb;
  border: 1px solid #e3e9ed;
  border-radius: 6px;
  padding: 12px;
  margin-top: 12px;
}
.compress-heading {
  align-items: center;
}

.preset-picker {
  align-items: center;
  display: flex;
  flex: 0 0 auto;
  gap: 8px;
  position: relative;
}

.preset-menu-trigger {
  min-height: 34px;
  padding: 0 14px;
}

.preset-menu {
  background: #fff;
  border: 1px solid #e3e9ed;
  border-radius: 8px;
  box-shadow: 0 12px 28px rgba(15, 23, 42, 0.14);
  min-width: 280px;
  padding: 6px;
  position: absolute;
  right: 0;
  top: calc(100% + 6px);
  z-index: 20;
  max-width: min(360px, calc(100vw - 32px));
}

.preset-menu-option {
  background: transparent;
  border: 0;
  border-radius: 6px;
  color: #18202a;
  cursor: pointer;
  display: block;
  padding: 8px 10px;
  text-align: left;
  width: 100%;
}

.preset-menu-option:hover,
.preset-menu-option:focus-visible {
  background: #f2f7f9;
  outline: none;
}

.preset-menu-option span,
.preset-menu-option small {
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.preset-menu-option span {
  font-size: 13px;
  font-weight: 600;
}

.preset-menu-option small {
  color: #7a8894;
  font-size: 11px;
  line-height: 1.35;
  margin-top: 3px;
}
.custom-args-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.custom-args-field > span {
  align-items: center;
  color: #43515c;
  display: inline-flex;
  font-size: 12.5px;
  font-weight: 600;
  gap: 6px;
}
.custom-args-field textarea {
  background: #fff;
  border: 1px solid #d6e0e6;
  border-radius: 6px;
  color: #18202a;
  font: 12.5px/1.5 ui-monospace, SFMono-Regular, Consolas, "Liberation Mono", monospace;
  min-height: 74px;
  padding: 8px 10px;
  resize: vertical;
}
.advanced-note {
  color: #687682;
  font-size: 12px;
  line-height: 1.5;
  margin: 8px 0 0;
}
.quick-process-panel {
  background: #f8fafb;
  border: 1px solid #e3e9ed;
  border-radius: 6px;
  margin-bottom: 12px;
  padding: 10px 12px;
}
.quick-process-panel.active {
  background: #f4fafc;
  border-color: #b7d8e3;
}
.quick-process-switch {
  margin: 0;
}
.quick-process-switch small {
  color: #667582;
  font-size: 12px;
  margin-left: 4px;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.quick-process-grid {
  display: grid;
  gap: 10px;
  grid-template-columns: repeat(14, minmax(0, 1fr));
  margin-top: 12px;
}
.quick-process-grid label {
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-width: 0;
}
.quick-field-rotate,
.quick-field-mirror,
.quick-field-fps {
  grid-column: span 2;
}
.quick-field-scale,
.quick-field-custom-scale {
  grid-column: span 3;
}
.quick-field-bitrate {
  grid-column: span 2;
}
.quick-process-grid label > span:first-child {
  color: #43515c;
  font-size: 12.5px;
  font-weight: 600;
}
.quick-process-grid .quick-field-label {
  align-items: center;
  display: inline-flex;
  gap: 6px;
}
.quick-process-grid input {
  background: #fff;
  border: 1px solid #d6e0e6;
  border-radius: 6px;
  color: #18202a;
  min-height: 36px;
  min-width: 0;
  padding: 0 10px;
}
.quick-inline-input {
  align-items: center;
  display: flex;
  gap: 6px;
}
.quick-inline-input input {
  flex: 1;
}
.quick-inline-input em {
  color: #667582;
  font-size: 12px;
  font-style: normal;
}
.logo-config-btn {
  min-height: 34px;
  padding: 0 14px;
}
.logo-config-btn.disabled {
  background: #f4f6f8;
  border-color: #e3e9ed;
  color: #9aa7b1;
  cursor: not-allowed;
}
.logo-summary {
  color: #176b87;
  font-size: 12.5px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.logo-summary.muted {
  color: #9aa7b1;
}
.logo-layer-control {
  align-items: center;
  display: flex;
  gap: 8px;
  min-width: 0;
  flex-wrap: wrap;
}
.logo-layer-label {
  color: #4a5560;
  font-size: 12.5px;
  white-space: nowrap;
}
.logo-layer-select {
  min-width: 0;
  flex: 1 1 180px;
  max-width: 240px;
}
.logo-layer-disabled .logo-layer-label {
  color: #9aa7b1;
}
.avs-hint {
  position: relative;
  display: inline-block;
  flex-shrink: 0;
  margin-left: 8px;
  padding: 2px 8px;
  background: #fff3cd;
  border: 1px solid #ffc107;
  border-radius: 3px;
  font-size: 12px;
  color: #856404;
  cursor: help;
  white-space: nowrap;
}
.avs-warn {
  position: relative;
  display: inline-block;
  flex-shrink: 0;
  margin-left: 8px;
  padding: 2px 8px;
  background: #fde8e8;
  border: 1px solid #f56565;
  border-radius: 3px;
  font-size: 12px;
  color: #c53030;
  cursor: help;
  white-space: nowrap;
}
.avs-checking {
  position: relative;
  display: inline-block;
  flex-shrink: 0;
  margin-left: 8px;
  padding: 2px 8px;
  background: #e8f0fe;
  border: 1px solid #90b4f0;
  border-radius: 3px;
  font-size: 12px;
  color: #2c5282;
  cursor: help;
  white-space: nowrap;
}
/* 复用 .hint::after 的暗卡片 tooltip 风格 */
.avs-hint::after,
.avs-warn::after,
.avs-checking::after {
  background: #1e293b;
  border-radius: 8px;
  box-shadow: 0 8px 24px rgba(15, 23, 42, 0.35);
  color: #f1f5f9;
  content: attr(data-tip);
  font: 400 12.5px/1.6 system-ui, "Segoe UI", "Microsoft YaHei", sans-serif;
  left: 50%;
  letter-spacing: 0.1px;
  max-width: 360px;
  opacity: 0;
  padding: 8px 12px;
  pointer-events: none;
  position: absolute;
  bottom: calc(100% + 8px);
  transform: translateX(-50%) translateY(-4px);
  transition: opacity 0.15s ease, transform 0.15s ease;
  visibility: hidden;
  white-space: pre-line;
  width: max-content;
  z-index: 50;
}
.avs-hint:hover::after,
.avs-hint:focus::after,
.avs-warn:hover::after,
.avs-warn:focus::after,
.avs-checking:hover::after,
.avs-checking:focus::after {
  opacity: 1;
  transform: translateX(-50%) translateY(0);
  visibility: visible;
}
.detected-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-left: 8px;
}
.tag {
  display: inline-block;
  background: #e8f4f8;
  border: 1px solid #b3d9e8;
  border-radius: 3px;
  padding: 2px 8px;
  font-size: 11px;
  color: #176b87;
  font-weight: 500;
}

@media (max-width: 1120px) {
  .quick-process-grid {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }
  .quick-process-grid label {
    grid-column: span 1;
  }
}

@media (max-width: 760px) {
  .quick-process-grid {
    grid-template-columns: 1fr;
  }
  .quick-process-grid label {
    grid-column: span 1;
  }
}

</style>
