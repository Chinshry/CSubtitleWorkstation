<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import type { CompressJob } from '../types'
import { isWindows } from '../stores/platformStore'
import { avsStatus, initAvsStatus } from '../stores/avsStore'
import { getSupportedEncoders, type EncoderInfo } from '../api/encoder'
import { analyzeSubtitle } from '../api/compress'
import AppSelect from './AppSelect.vue'

const job = defineModel<CompressJob>({ required: true })

const props = defineProps<{
  /** 配置 LOGO 按钮是否禁用（一般在视频未就绪时禁用） */
  logoButtonDisabled?: boolean
  /** 配置 LOGO 按钮禁用原因（用于 tooltip） */
  logoButtonDisabledReason?: string
}>()

const emit = defineEmits<{
  (e: 'open-logo-editor'): void
}>()

// 支持的编码器列表和自动启用 AVS 的原因
const supportedEncoders = ref<EncoderInfo[]>([])
const avsAutoEnabledReason = ref<string>('')

// 从 avsAutoEnabledReason 中提取检测到的标签
const detectedTagsDisplay = computed(() => {
  if (!avsAutoEnabledReason.value) return []
  const match = avsAutoEnabledReason.value.match(/检测到字幕特效（(.+?)）/)
  if (!match || !match[1]) return []
  return match[1].split('、')
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
  let tip = '启用 AviSynth+ 脚本作为 ffmpeg 输入，字幕由 VSFilterMod 的 TextSubMod 渲染。\n'
  tip += '仅 Windows 支持；需要本机安装 AviSynth+ 且 ffmpeg 启用了 --enable-avisynth（如 Gyan.dev full 版）。\n'
  tip += '启用后 LOGO overlay 与 yadif 仍然有效，但 ffmpeg subtitles 滤镜会被跳过。\n\n'
  tip += '可启用 AVS 压制模式；不勾选则走 ffmpeg filter 模式。'
  return tip
})

// AVS 开关启用条件：仅 Windows + 检测通过；缺失依赖时禁用并给出提示
const avsToggleDisabled = computed(() => !isWindows.value || !avsStatus.value?.available)
const avsToggleDisabledTip = computed(() => {
  if (!isWindows.value) return 'AVS 压制仅 Windows 支持'
  const s = avsStatus.value
  if (!s) return '正在检测 AVS 环境…'
  if (s.available) return ''
  return s.message ?? 'AVS 环境不可用'
})

// 平台不支持或 mock 切换导致已勾选但不可用时，强制关掉
function syncAvsAvailability() {
  if (avsToggleDisabled.value && job.value.useAvs) {
    job.value.useAvs = false
  }
}

// AVS 状态变化时（含调试 mock 切换）自动同步
watch(avsToggleDisabled, syncAvsAvailability)

// 字幕分析：检测是否包含特效标签，自动勾选 AVS
async function analyzeSubtitleForEffects() {
  const subtitlePath = job.value.subtitlePath?.trim()
  if (!subtitlePath || !isWindows.value || !avsStatus.value?.available) {
    avsAutoEnabledReason.value = ''
    return
  }

  try {
    const result = await analyzeSubtitle(subtitlePath)
    if (result.hasEffects) {
      avsAutoEnabledReason.value = `检测到字幕特效（${result.detectedTags.join('、')}），已自动启用 AVS 压制`
      job.value.useAvs = true
    } else {
      avsAutoEnabledReason.value = ''
    }
  } catch (err) {
    console.error('Failed to analyze subtitle:', err)
    avsAutoEnabledReason.value = ''
  }
}

// 监听字幕路径变化
watch(() => job.value.subtitlePath, analyzeSubtitleForEffects, { immediate: false })

onMounted(async () => {
  // 加载支持的编码器列表
  try {
    supportedEncoders.value = await getSupportedEncoders()
  } catch (err) {
    console.error('Failed to get supported encoders:', err)
  }

  if (isWindows.value) {
    await initAvsStatus()
  }
  syncAvsAvailability()
})

// 最大码率三选一：none(留空) / auto(=0) / custom(>0)
type BitrateMode = 'none' | 'auto' | 'custom'

// 生成编码器选项，仅显示当前平台支持的编码器
const encoderOptions = computed(() => {
  const encoderDescriptions: Record<string, string> = {
    libx264: 'CPU libx264（CPU 软编，兼容性最好，支持 AVS）',
    h264_nvenc: 'NVIDIA h264_nvenc（N 卡硬编，压制更快，不支持 AVS）',
    h264_amf: 'AMD h264_amf（A 卡硬编，速度快，不支持 AVS）',
    h264_videotoolbox: 'macOS h264_videotoolbox（Apple Silicon/Intel 硬编，不支持 AVS）'
  }

  if (supportedEncoders.value.length > 0) {
    return supportedEncoders.value
      .filter(e => e.supported)
      .map(e => ({
        value: e.name,
        label: encoderDescriptions[e.name] || e.label
      }))
  }
  // 后端未返回时使用默认选项
  return [
    { value: 'libx264', label: 'CPU libx264（CPU 软编，兼容性最好，支持 AVS）' },
    { value: 'h264_nvenc', label: 'NVIDIA h264_nvenc（N 卡硬编，压制更快，不支持 AVS）' },
    { value: 'h264_amf', label: 'AMD h264_amf（A 卡硬编，速度快，不支持 AVS）' },
    { value: 'h264_videotoolbox', label: 'macOS h264_videotoolbox（Apple Silicon/Intel 硬编，不支持 AVS）' }
  ]
})

const bitrateMode = computed<BitrateMode>({
  get(): BitrateMode {
    const v = job.value.maxBitrate
    if (v === undefined || v === null || (typeof v === 'number' && v < 0)) return 'none'
    if (v === 0) return 'auto'
    return 'custom'
  },
  set(mode: BitrateMode) {
    if (mode === 'none') job.value.maxBitrate = undefined
    else if (mode === 'auto') job.value.maxBitrate = 0
    else {
      const cur = job.value.maxBitrate
      if (!cur || cur <= 0) job.value.maxBitrate = 3000
    }
  }
})

const customBitrate = computed<number | undefined>({
  get() {
    const v = job.value.maxBitrate
    return typeof v === 'number' && v > 0 ? v : undefined
  },
  set(v) {
    if (typeof v === 'number' && v > 0) job.value.maxBitrate = v
  }
})

// 已配置 LOGO 的摘要文案
// 直接给百分比对用户不直观，改成「方位（九宫格） + 像素尺寸」
const logoSummary = computed(() => {
  const layout = job.value.logoLayout
  if (!layout || !layout.path) return ''
  const name = logoBasename(layout.path)
  return `已配置：${name} · ${describeLogoPosition(layout)} · ${describeLogoSize(layout)}`
})

// LOGO 层级：AVS 模式下 VSFilterMod TextSubMod 把字幕烧进 AVS 输出，
// ffmpeg 滤镜无法再插入到字幕之下，故 AVS 启用时 LOGO 强制在字幕上、禁用切换。
const logoLayerDisabled = computed(() => !!job.value.useAvs)
const logoLayerTip = computed(() =>
  logoLayerDisabled.value
    ? 'AVS 模式下字幕由 AVS 脚本渲染，LOGO 仅能叠加在字幕之上'
    : '“字幕在上 LOGO 在下”=LOGO 会被字幕遮挡；“LOGO 在上 字幕在下”=LOGO 完整覆盖字幕'
)
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
  const horiz = cx < 0.34 ? '左' : cx < 0.67 ? '中' : '右'
  const vert = cy < 0.34 ? '上' : cy < 0.67 ? '中' : '下'
  if (horiz === '中' && vert === '中') return '画面中央'
  if (horiz === '中') return vert === '上' ? '顶部居中' : '底部居中'
  if (vert === '中') return horiz === '左' ? '左侧居中' : '右侧居中'
  return `${horiz}${vert}角` // 左上角 / 右上角 / 左下角 / 右下角
}

function describeLogoSize(layout: NonNullable<typeof job.value.logoLayout>): string {
  const vw = job.value.videoWidth
  const vh = job.value.videoHeight
  if (vw && vh && vw > 0 && vh > 0) {
    const w = Math.round(layout.wPct * vw)
    const h = Math.round(layout.hPct * vh)
    return `${w} × ${h} 像素`
  }
  // 视频分辨率未就绪时退回到百分比
  return `${(layout.wPct * 100).toFixed(1)}% × ${(layout.hPct * 100).toFixed(1)}%`
}

function logoBasename(p: string): string {
  if (!p) return ''
  const idx = Math.max(p.lastIndexOf('\\'), p.lastIndexOf('/'))
  return idx >= 0 ? p.slice(idx + 1) : p
}

function onOpenLogoEditor() {
  if (props.logoButtonDisabled) return
  emit('open-logo-editor')
}
</script>

<template>
  <section class="panel">
    <div class="panel-heading">
      <div>
        <h2>压制参数</h2>
      </div>
    </div>

    <div class="form-grid">
      <div class="param-row wide">
        <label class="crf-cell">
          <span>
            CRF
            <span
              class="hint tip-right"
              :data-tip="`对应命令：-crf ${job.crf}\n\n常量码率因子，数值越小画质越好、文件越大。\nlibx264 / libx265 推荐 18 – 28：18 视觉无损，23 默认，28 偏低质量。\n硬件编码器（nvenc / amf / videotoolbox）的 CRF 含义略有不同，仅作近似画质参考。`"
            ></span>
          </span>
          <input v-model.number="job.crf" type="number" min="0" max="51" />
        </label>
        <label class="bitrate-cell">
          <span>
            最大码率（Kbps）
            <span
              class="hint tip-right"
              data-tip="对应命令：-maxrate {值}k -bufsize {值×2}k

限制视频码率峰值，防止画面剧烈变化时码率失控。
不限制：完全跟随 CRF。
自动：取原视频码率 + 1000 Kbps。
自定义：按填写的 Kbps 直接生效。"
            ></span>
          </span>
          <div class="bitrate-control">
            <AppSelect
              v-model="bitrateMode"
              class="bitrate-select"
              :options="[
                { value: 'none', label: '不限制' },
                { value: 'auto', label: '自动（视频原码率 + 1000 Kbps）' },
                { value: 'custom', label: '自定义' }
              ]"
            />
            <input
              v-if="bitrateMode === 'custom'"
              v-model.number="customBitrate"
              type="number"
              min="1"
              class="bitrate-input"
              placeholder="Kbps"
            />
          </div>
        </label>
        <label class="wide encoder-cell">
          <span>
            编码器（GraphicsType）
            <span
              class="hint"
              :data-tip="`对应命令：-c:v ${job.encoder}\n\nlibx264：CPU 软编，兼容性最好、画质稳定，支持 AVS。\nh264_nvenc：NVIDIA 显卡硬编，速度快，不支持 AVS。\nh264_amf：AMD 显卡硬编，速度快，不支持 AVS。\nh264_videotoolbox：macOS 硬编，不支持 AVS。`"
            ></span>
          </span>
          <AppSelect
            v-model="job.encoder"
            :options="encoderOptions"
          />
        </label>
      </div>

      <div class="switch-row-wrap wide">
        <label class="switch-row">
          <input v-model="job.needLogo" type="checkbox" />
          <span class="switch"></span>
          <span>压制 LOGO</span>
          <span class="hint tip-right" data-tip="在视频画面上叠加一张 LOGO 图片。
点击右侧「配置 LOGO」按钮可视化设置图片、位置与大小。
开关关闭时即使已配置布局也不会叠加。"></span>
        </label>
        <label class="switch-row">
          <input v-model="job.needYadif" type="checkbox" />
          <span class="switch"></span>
          <span>使用反交错压制</span>
          <span class="hint" data-tip="对应命令：-vf yadif

把交错信号合成连续画面，消除横向锯齿/梳状伪影。
TV 录制、转录、DV、磁带数字化等素材容易出现隔行，需要开启。
网络发布的视频通常已经是逐行扫描，不需要开启。"></span>
        </label>
        <label class="switch-row" :class="{ 'switch-row-disabled': avsToggleDisabled }" :title="avsToggleTip">
          <input
            v-model="job.useAvs"
            type="checkbox"
            :disabled="avsToggleDisabled"
          />
          <span class="switch"></span>
          <span>AVS 压制模式</span>
          <span v-if="avsAutoEnabledReason" class="avs-hint" :data-tip="`${detectedTagsDisplay.join('、')}`">检测到特殊标签</span>
          <span class="hint" :data-tip="avsToggleTip"></span>
        </label>
      </div>

      <div v-if="job.needLogo" class="logo-config-row wide">
        <button
          type="button"
          class="secondary logo-config-btn"
          :class="{ disabled: logoButtonDisabled }"
          :disabled="logoButtonDisabled"
          :title="logoButtonDisabled ? logoButtonDisabledReason : '打开 LOGO 编辑器，可视化设置图片、位置与大小'"
          @click="onOpenLogoEditor"
        >
          {{ job.logoLayout ? '重新配置 LOGO' : '配置 LOGO' }}
        </button>
        <span v-if="logoSummary" class="logo-summary">{{ logoSummary }}</span>
        <span v-else class="logo-summary muted">尚未配置 LOGO</span>
        <div class="logo-layer-control" :class="{ 'logo-layer-disabled': logoLayerDisabled }" :title="logoLayerTip">
          <span class="logo-layer-label">LOGO 层级</span>
          <AppSelect
            v-model="logoLayerValue"
            class="logo-layer-select"
            :disabled="logoLayerDisabled"
            :options="[
              { value: 'bottom', label: '字幕在上 LOGO 在下', title: 'LOGO 会被字幕遮挡' },
              { value: 'top', label: 'LOGO 在上 字幕在下', title: 'LOGO 完整覆盖字幕' }
            ]"
          />
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.logo-config-row {
  align-items: center;
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
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
  margin-left: auto;
}
.logo-layer-label {
  color: #4a5560;
  font-size: 12.5px;
  white-space: nowrap;
}
.logo-layer-select {
  min-width: 200px;
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
/* 复用 .hint::after 的暗卡片 tooltip 风格 */
.avs-hint::after {
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
.avs-hint:focus::after {
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

</style>
