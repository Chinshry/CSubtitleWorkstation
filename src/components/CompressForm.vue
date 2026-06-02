<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import type { CompressJob, VideoEncodePreset } from '../types'
import { isWindows } from '../stores/platformStore'
import { avsStatus, initAvsStatus } from '../stores/avsStore'
import { analyzeSubtitle, type SubtitleAnalysisResult } from '../api/compress'
import { useEncoderOptions } from '../composables/useEncoderOptions'
import { useToast } from '../composables/useToast'
import EncodeSettingsFields from './EncodeSettingsFields.vue'
import AppSelect from './AppSelect.vue'
import InfoHint from './InfoHint.vue'

const job = defineModel<CompressJob>({ required: true })

const props = defineProps<{
  encodePresets?: VideoEncodePreset[]
  selectedEncodePresetId?: string
  /** 配置 LOGO 按钮是否禁用（一般在视频未就绪时禁用） */
  logoButtonDisabled?: boolean
  /** 配置 LOGO 按钮禁用原因（用于 tooltip） */
  logoButtonDisabledReason?: string
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
const advancedOpen = ref(false)
const presetMenuOpen = ref(false)
const toast = useToast()
let subtitleAnalyzeSeq = 0

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

// 字幕分析：检测是否包含特效标签，自动勾选 AVS；同时把结果透传给上层用于色彩矩阵匹配
async function analyzeSubtitleForEffects() {
  const subtitlePath = job.value.subtitlePath?.trim()
  const seq = ++subtitleAnalyzeSeq
  if (!subtitlePath) {
    avsAutoEnabledReason.value = ''
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
    avsAutoEnabledReason.value = `检测到字幕特效（${result.detectedTags.join('、')}），已自动启用 AVS 压制`
    job.value.useAvs = true
  } else {
    avsAutoEnabledReason.value = ''
  }
  // 矩阵信息无论平台都需要透传，banner 判定逻辑在外层
  emit('subtitle-analyzed', result)
}

// 监听字幕路径变化
watch(() => job.value.subtitlePath, analyzeSubtitleForEffects, { immediate: false })

onMounted(async () => {
  document.addEventListener('mousedown', closePresetMenuOnOutside)

  // 加载支持的编码器列表
  try {
    await loadEncoderOptions()
  } catch (err) {
    console.error('Failed to get supported encoders:', err)
  }

  if (isWindows.value) {
    await initAvsStatus()
  }
  syncAvsAvailability()
})

onBeforeUnmount(() => {
  document.removeEventListener('mousedown', closePresetMenuOnOutside)
})
// 压制预设提示：解释作用 + 引导用户去哪儿管理预设
const presetTip = '一键应用一组编码器、质量值、最大码率组合。\n\n在左侧侧边栏「预设」页面新增、修改或删除预设。\n应用预设后，下方质量值/最大码率/编码器仍可手动微调，不会回写到预设本身。'

// 已配置 LOGO 的摘要文案
// 直接给百分比对用户不直观，改成「方位（九宫格） + 像素尺寸」
const logoSummary = computed(() => {
  const layout = job.value.logoLayout
  if (!layout || !layout.path) return ''
  const video_name = logoBasename(layout.path)
  return `已配置：${video_name} · ${describeLogoPosition(layout)} · ${describeLogoSize(layout)}`
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
  toast.success(presetName ? `已套用到当前参数：${presetName}` : '已套用预设', 2500)
}

function describeEncodePreset(preset: VideoEncodePreset): string {
  return [
    preset.encoder,
    `质量 ${preset.crf}`,
    describePresetBitrate(preset.maxBitrate),
  ].join(' · ')
}

function describePresetBitrate(maxBitrate: number | undefined): string {
  if (maxBitrate === undefined) return '不限码率'
  if (maxBitrate === 0) return '自动码率'
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
    args.push('-maxrate', '原视频码率+1000k', '-bufsize', '2倍最大码率')
  }
  const custom = preset.customVideoArgs?.trim()
  if (custom) args.push(custom)
  return args.join(' ')
}
</script>

<template>
  <section class="panel">
    <div class="panel-heading compress-heading">
      <div>
        <h2>压制参数</h2>
      </div>
      <div v-if="encodePresetOptions.length" class="preset-picker">
        <button
          type="button"
          class="secondary preset-menu-trigger"
          v-tooltip="selectedEncodePresetName ? `上次套用：${selectedEncodePresetName}` : '选择一个压制预设并套用到当前参数'"
          @click="togglePresetMenu"
        >
          套用预设
        </button>
        <InfoHint
          placement="left"
          title="压制预设"
          body="一键应用一组编码器、质量值、最大码率组合。应用后下方参数仍可手动微调，不会回写到预设本身。"
          :items="['在左侧「预设」页面新增、修改或删除预设。', '适合把常用平台规格保存成固定方案。']"
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
        <EncodeSettingsFields v-model="job" :encoder-options="encoderOptions">
          <template #encoder-trailing>
            <button type="button" class="secondary advanced-toggle" @click="advancedOpen = !advancedOpen">
              {{ advancedOpen ? '隐藏附加参数' : '显示附加参数' }}
            </button>
          </template>
        </EncodeSettingsFields>

        <div v-if="advancedOpen" class="advanced-panel">
          <label class="custom-args-field">
            <span>
              附加 ffmpeg 视频参数
            </span>
            <textarea
              v-model="job.customVideoArgs"
              rows="3"
              spellcheck="false"
            ></textarea>
          </label>
          <p class="advanced-note">
            这些参数会追加到视频编码参数后；输入、滤镜、编码器、音频和输出路径仍由工作站管理。
          </p>
        </div>
      </section>

      <section class="form-section options-section">
        <div class="switch-row-wrap">
          <label class="switch-row">
            <input v-model="job.needLogo" type="checkbox" />
            <span class="switch"></span>
            <span>压制 LOGO</span>
            <InfoHint
              placement="right"
              title="压制 LOGO"
              body="在视频画面上叠加一张 LOGO 图片，可视化设置图片、位置与大小。"
              :items="['点击「配置 LOGO」进入编辑器。', '关闭开关时，已保存的 LOGO 布局会保留，但不会参与压制。']"
            />
          </label>
          <label class="switch-row">
            <input v-model="job.needYadif" type="checkbox" />
            <span class="switch"></span>
            <span>使用反交错压制</span>
            <InfoHint
              title="反交错压制"
              command="-vf yadif"
              body="把交错信号合成连续画面，消除横向锯齿或梳状伪影。"
              :items="['TV 录制、转录、DV、磁带数字化素材常见隔行，建议开启。', '网络发布视频通常已经是逐行扫描，一般不需要开启。']"
            />
          </label>
          <label class="switch-row" :class="{ 'switch-row-disabled': avsToggleDisabled }">
            <input
              v-model="job.useAvs"
              type="checkbox"
              :disabled="avsToggleDisabled"
            />
            <span class="switch"></span>
            <span>AVS 压制模式</span>
            <span v-if="avsAutoEnabledReason" class="avs-hint" :data-tip="`${detectedTagsDisplay.join('、')}`">检测到特殊标签</span>
            <InfoHint
              placement="left"
              title="AVS 压制模式"
              command="AviSynth+ + VSFilterMod TextSubMod"
              body="启用 AviSynth+ 脚本作为 ffmpeg 输入，字幕由 VSFilterMod 渲染；LOGO overlay 与 yadif 仍然有效。"
              :items="['仅 Windows 支持，需要本机安装 AviSynth+，且 ffmpeg 启用 --enable-avisynth。', '适合复杂 ASS 特效字幕；不勾选则走 ffmpeg filter 模式。']"
            />
          </label>
        </div>

        <div v-if="job.needLogo" class="logo-config-row">
          <button
            type="button"
            class="secondary logo-config-btn"
            :class="{ disabled: logoButtonDisabled }"
            :disabled="logoButtonDisabled"
            v-tooltip="logoButtonDisabled ? logoButtonDisabledReason : '打开 LOGO 编辑器，可视化设置图片、位置与大小'"
            @click="onOpenLogoEditor"
          >
            {{ job.logoLayout ? '重新配置 LOGO' : '配置 LOGO' }}
          </button>
          <span v-if="logoSummary" class="logo-summary">{{ logoSummary }}</span>
          <span v-else class="logo-summary muted">尚未配置 LOGO</span>
          <div class="logo-layer-control" :class="{ 'logo-layer-disabled': logoLayerDisabled }" v-tooltip="logoLayerTip">
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
      </section>
    </div>
  </section>
</template>

<style scoped>
.logo-config-row {
  align-items: center;
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  margin-top: 14px;
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
