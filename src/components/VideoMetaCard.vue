<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import type { OutputNameTemplate, VideoMeta } from '../types'
import { globalDragActive } from '../stores/dropStore'
import { validateOutputParentDir } from '../api/compress'
import { useToast } from '../composables/useToast'

const props = defineProps<{
  meta: VideoMeta | null
  loading: boolean
  error: string
  videoPath: string
  subtitlePath: string
  outputTemplates?: OutputNameTemplate[]
  selectedOutputTemplateId?: string
}>()

const emit = defineEmits<{
  (e: 'clear-video'): void
  (e: 'clear-subtitle'): void
  (e: 'pick-video', path: string): void
  (e: 'pick-subtitle', path: string): void
  (e: 'update:selectedOutputTemplateId', value: string): void
  (e: 'apply-output-template'): void
}>()

const outputPath = defineModel<string>('outputPath', { default: '' })

const isEditingOutput = ref(false)
const draftOutputPath = ref('')
const isTemplateMenuOpen = ref(false)
const templateMenuRef = ref<HTMLElement | null>(null)
const toast = useToast()

const hasAnyPath = computed(
  () => !!props.videoPath || !!props.subtitlePath
)

const isEmptyState = computed(
  () => !hasAnyPath.value && !props.meta && !props.loading && !props.error
)

async function chooseVideo() {
  const sel = await open({
    multiple: false,
    directory: false,
    title: '选择视频文件',
    filters: [
      { name: '视频', extensions: ['mp4', 'mkv', 'mov', 'ts', 'm4v', 'flv', 'avi', 'webm', 'wmv', 'mpg', 'mpeg', '3gp', 'mts', 'm2ts'] }
    ]
  })
  if (typeof sel === 'string') emit('pick-video', sel)
}

async function chooseSubtitle() {
  const sel = await open({
    multiple: false,
    directory: false,
    title: '选择字幕文件',
    filters: [
      { name: '字幕', extensions: ['ass', 'ssa', 'srt', 'vtt', 'sub'] }
    ]
  })
  if (typeof sel === 'string') emit('pick-subtitle', sel)
}

// dropzone 多选：一次选视频 + 字幕，按扩展名自动分发
function beginOutputEdit() {
  draftOutputPath.value = outputPath.value
  isEditingOutput.value = true
}

async function saveOutputEdit() {
  const next = draftOutputPath.value.trim()
  try {
    await validateOutputParentDir(next)
    outputPath.value = next
    isEditingOutput.value = false
  } catch (error) {
    toast.error(formatError(error))
  }
}

function formatError(error: unknown) {
  return error instanceof Error ? error.message : String(error)
}

async function chooseFiles() {
  const sel = await open({
    multiple: true,
    directory: false,
    title: '选择视频和字幕文件',
    filters: [
      {
        name: '视频和字幕',
        extensions: [
          'mp4', 'mkv', 'mov', 'ts', 'm4v', 'flv', 'avi', 'webm', 'wmv', 'mpg', 'mpeg', '3gp', '3g2', 'rm', 'rmvb', 'vob', 'mts', 'm2ts',
          'ass', 'ssa', 'srt', 'vtt', 'sub'
        ]
      }
    ]
  })
  if (!sel) return
  const arr = Array.isArray(sel) ? sel : [sel]
  for (const p of arr) {
    const lower = p.toLowerCase()
    if (/\.(mp4|mkv|mov|ts|m4v|flv|avi|webm|wmv|mpg|mpeg|3gp|3g2|rm|rmvb|vob|mts|m2ts)$/.test(lower)) {
      emit('pick-video', p)
    } else if (/\.(ass|ssa|srt|vtt|sub)$/.test(lower)) {
      emit('pick-subtitle', p)
    }
  }
}

const DASH = '—'

type Field = { label: string; value: string; title?: string }

function formatBytes(bytes?: number): string {
  if (!bytes || bytes <= 0) return DASH
  const mb = bytes / 1024 / 1024
  if (mb < 1024) return `${mb.toFixed(1)} MB`
  return `${(mb / 1024).toFixed(2)} GB`
}

const resolutionField = computed(() => {
  const m = props.meta
  if (!m || !m.width || !m.height) return null
  const tags: string[] = []
  if (m.dar) tags.push(`DAR ${m.dar}`)
  if (m.sar && m.sar !== '1:1') tags.push(`SAR ${m.sar}`)
  return {
    value: `宽 ${m.width} × 高 ${m.height}`,
    title: tags.length
      ? `画面像素尺寸（${tags.join('，')}）`
      : '画面像素尺寸（宽×高）'
  }
})

const durationText = computed(() => {
  const m = props.meta
  if (!m) return DASH
  const t = m.durationText ?? '—'
  if (typeof m.startSeconds === 'number' && m.startSeconds > 0) {
    return `${t}（起始 ${m.startSeconds.toFixed(2)}s）`
  }
  return t
})

const videoFields = computed<Field[]>(() => {
  const m = props.meta
  if (!m) return []
  const fields: Field[] = []
  const res = resolutionField.value
  if (res) {
    fields.push({
      label: '分辨率',
      value: res.value,
      title: res.title
    })
  }
  if (m.videoCodec) {
    fields.push({
      label: '编码',
      value: m.videoProfile ? `${m.videoCodec} ${m.videoProfile}` : m.videoCodec,
      title: '视频编码器与 Profile：决定压缩算法与档次（如 h264 High、hevc Main10）'
    })
  }
  if (m.pixelFormat) {
    fields.push({
      label: '像素',
      value: m.pixelFormat,
      title: '像素格式（色彩采样/位深）：常见 yuv420p 为 8bit 4:2:0，yuv420p10le 为 10bit'
    })
  }
  const rate = m.videoBitrateKbps ?? m.overallBitrateKbps
  if (rate) {
    fields.push({
      label: '码率',
      value: `${rate} kbps`,
      title: '视频码率：每秒数据量，越高画质越好、文件越大'
    })
  }
  if (m.fps) {
    fields.push({
      label: '帧率',
      value: `${m.fps} fps`,
      title: '帧率：每秒画面数（fps）'
    })
  }
  if (m.frameRateMode) {
    fields.push({
      label: '帧模式',
      value: m.frameRateMode,
      title:
        m.frameRateMode === 'CFR'
          ? '恒定帧率（CFR）：每帧间隔均匀。ffprobe 中 r_frame_rate ≈ avg_frame_rate'
          : '可变帧率（VFR）：帧间隔不均匀，常见于屏幕录制 / 部分网络视频。压制时如需稳定帧率可考虑重映射。'
    })
  }
  if (m.totalFrames) {
    fields.push({
      label: '总帧数',
      value: `${m.totalFramesEstimated ? '约 ' : ''}${m.totalFrames.toLocaleString()}`,
      title: m.totalFramesEstimated
        ? '估算总帧数（ffprobe 未提供 nb_frames，按时长 × 帧率计算；未进行耗时逐帧统计）'
        : '容器中记录的总帧数（来自 ffprobe nb_frames）'
    })
  }
  if (m.colorSpace) {
    fields.push({
      label: '色域',
      value: m.colorSpace,
      title: '色彩空间/原色：常见 bt709（SDR）、bt2020（HDR）'
    })
  }
  if (m.colorRange) {
    fields.push({
      label: '色范围',
      value: m.colorRange,
      title: '亮度范围：tv/limited 为 16–235，pc/full 为 0–255'
    })
  }
  return fields
})

const audioFields = computed<Field[]>(() => {
  const m = props.meta
  if (!m || !m.audioCodec) return []
  const fields: Field[] = []
  fields.push({
    label: '编码',
    value: m.audioProfile ? `${m.audioCodec} ${m.audioProfile}` : m.audioCodec,
    title: '音频编码器与 Profile：如 aac LC（低复杂度）、HE-AAC、ac3、eac3、opus'
  })
  if (m.audioSampleRate) {
    const khz = (m.audioSampleRate / 1000).toString().replace(/\.0$/, '')
    fields.push({
      label: '采样率',
      value: `${khz} kHz`,
      title: '采样率：每秒采样次数，常见 44.1 kHz / 48 kHz'
    })
  }
  if (m.audioChannels) {
    fields.push({
      label: '声道',
      value: m.audioChannels,
      title: '声道布局：mono 单声道 / stereo 立体声 / 5.1 环绕 / 7.1 等'
    })
  }
  if (m.audioBitrateKbps) {
    fields.push({
      label: '码率',
      value: `${m.audioBitrateKbps} kbps`,
      title: '音频码率：每秒数据量'
    })
  }
  return fields
})

const fileText = computed(() => {
  const m = props.meta
  if (!m) return DASH
  return formatBytes(m.fileSizeBytes)
})

function friendlyContainer(raw: string): string {
  const tags = raw.toLowerCase().split(',').map((t) => t.trim()).filter(Boolean)
  const has = (...keys: string[]) => keys.some((k) => tags.includes(k))
  if (has('mov', 'mp4', 'm4a', '3gp', '3g2', 'mj2')) return 'MP4 / MOV'
  if (has('matroska', 'webm')) return 'Matroska / WebM'
  if (has('mpegts')) return 'MPEG-TS'
  if (has('avi')) return 'AVI'
  if (has('asf', 'wmv', 'asf_o')) return 'ASF / WMV'
  if (has('flv')) return 'FLV'
  if (has('ogg', 'ogv')) return 'Ogg'
  if (has('mpeg', 'mpegvideo', 'mpegps')) return 'MPEG-PS'
  if (has('rm', 'rmvb')) return 'RealMedia'
  return raw
}

const containerField = computed(() => {
  const m = props.meta
  if (!m || !m.format) return null
  return {
    value: friendlyContainer(m.format),
    title: `ffmpeg demuxer：${m.format}（同一 demuxer 处理的所有兼容扩展名都会列出）`
  }
})

const hasVideoFields = computed(() => videoFields.value.length > 0)
const hasAudioFields = computed(() => audioFields.value.length > 0)
const templateOptions = computed(() => props.outputTemplates ?? [])

const selectedTemplateId = computed(() => props.selectedOutputTemplateId ?? templateOptions.value[0]?.id ?? '')
const selectedTemplateName = computed(() => {
  return templateOptions.value.find((tpl) => tpl.id === selectedTemplateId.value)?.name ?? '命名模板'
})

function toggleTemplateMenu() {
  isTemplateMenuOpen.value = !isTemplateMenuOpen.value
}

function applyTemplate(id: string) {
  emit('update:selectedOutputTemplateId', id)
  emit('apply-output-template')
  isTemplateMenuOpen.value = false
}

function closeTemplateMenuOnOutside(event: MouseEvent) {
  const target = event.target
  if (!(target instanceof Node)) return
  if (!templateMenuRef.value?.contains(target)) {
    isTemplateMenuOpen.value = false
  }
}

function closeTemplateMenuOnEscape(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    isTemplateMenuOpen.value = false
  }
}

onMounted(() => {
  document.addEventListener('mousedown', closeTemplateMenuOnOutside)
  document.addEventListener('keydown', closeTemplateMenuOnEscape)
})

onBeforeUnmount(() => {
  document.removeEventListener('mousedown', closeTemplateMenuOnOutside)
  document.removeEventListener('keydown', closeTemplateMenuOnEscape)
})
</script>

<template>
  <section
    class="panel video-meta-card"
    :class="{ 'drag-target': globalDragActive, 'is-empty': isEmptyState }"
  >
    <!-- 空状态：dropzone 占位 -->
    <div v-if="isEmptyState" class="dropzone">
      <div class="dropzone-icon">⬇︎</div>
      <div class="dropzone-title">拖入视频开始处理</div>
      <div class="dropzone-sub">
        <span class="dropzone-note">支持单视频处理；需要字幕压制时可同时拖入字幕</span>
        <br />
        视频：mp4 / mkv / mov / ts / m4v / flv / avi / webm / wmv / mpg / 3gp / mts
        <br />
        字幕可选：ass / ssa / srt / vtt / sub
      </div>
      <div class="dropzone-actions">
        <button class="secondary" @click="chooseFiles">选择文件</button>
      </div>
    </div>

    <!-- 正常状态：路径行 + 视频信息 -->
    <template v-else>
      <div class="meta-head">
        <h2>视频信息</h2>
        <span v-if="loading" class="meta-status loading">解析中…</span>
        <span v-else-if="error" class="meta-status error" v-tooltip="error">{{ error }}</span>
      </div>

      <dl class="path-grid">
        <div>
          <dt>视频</dt>
          <dd class="path-row">
            <span class="path-text readonly" v-tooltip="videoPath || '未导入'">{{ videoPath || '—' }}</span>
            <button
              v-if="videoPath"
              class="path-action"
              data-tip="清除并重新拖入"
              @click="emit('clear-video')"
              aria-label="清除视频"
            >✕</button>
            <button
              v-else
              class="path-action"
              data-tip="选择视频文件"
              @click="chooseVideo"
              aria-label="选择视频"
            >+</button>
          </dd>
        </div>
        <div>
          <dt>字幕</dt>
          <dd class="path-row">
            <span class="path-text readonly" v-tooltip="subtitlePath || '未导入'">{{ subtitlePath || '—' }}</span>
            <button
              v-if="subtitlePath"
              class="path-action"
              data-tip="清除并重新拖入"
              @click="emit('clear-subtitle')"
              aria-label="清除字幕"
            >✕</button>
            <button
              v-else
              class="path-action"
              data-tip="选择字幕文件"
              @click="chooseSubtitle"
              aria-label="选择字幕"
            >+</button>
          </dd>
        </div>
        <div>
          <dt>输出</dt>
          <dd class="path-row">
            <input
              v-if="isEditingOutput"
              v-model="draftOutputPath"
              class="path-input"
              placeholder="例如：E:\path\to\output.mp4"
              @blur="saveOutputEdit"
              @keyup.enter="saveOutputEdit"
              @keyup.esc="isEditingOutput = false"
            />
            <span v-else class="path-text" v-tooltip="outputPath || '未设置'">{{ outputPath || '—' }}</span>
            <div v-if="templateOptions.length" ref="templateMenuRef" class="path-template-menu">
              <button
                class="path-action"
                :class="{ active: isTemplateMenuOpen }"
                data-tip="套用命名模板"
                type="button"
                @click="toggleTemplateMenu"
                :aria-label="`套用命名模板：${selectedTemplateName}`"
                :aria-expanded="isTemplateMenuOpen"
                aria-haspopup="menu"
              >
                <span aria-hidden="true" class="template-action-icon"></span>
              </button>
              <div v-if="isTemplateMenuOpen" class="template-popover" role="menu">
                <button
                  v-for="tpl in templateOptions"
                  :key="tpl.id"
                  class="template-popover-item"
                  :class="{ selected: tpl.id === selectedTemplateId }"
                  type="button"
                  role="menuitem"
                  v-tooltip="tpl.pattern"
                  @click="applyTemplate(tpl.id)"
                >
                  <span>{{ tpl.name }}</span>
                  <span v-if="tpl.id === selectedTemplateId" aria-hidden="true">✓</span>
                </button>
              </div>
            </div>
            <button
              class="path-action"
              data-tip="编辑输出路径"
              type="button"
              @click="isEditingOutput ? (isEditingOutput = false) : beginOutputEdit()"
              aria-label="编辑输出"
            >✎</button>
          </dd>
        </div>
      </dl>

      <dl v-if="meta && !error" class="meta-grid">
        <div class="wide">
          <dt>文件</dt>
          <dd class="kv-line">
            <span class="kv" v-tooltip="'文件大小'">
              <em>大小</em>
              <span>{{ fileText }}</span>
            </span>
            <span class="kv" v-tooltip="'媒体时长（起始时间若不为 0 会在括号中标出）'">
              <em>时长</em>
              <span>{{ durationText }}</span>
            </span>
            <span class="kv" v-if="containerField" v-tooltip="containerField.title">
              <em>容器</em>
              <span>{{ containerField.value }}</span>
            </span>
          </dd>
        </div>
        <div class="wide" v-if="hasVideoFields">
          <dt>视频</dt>
          <dd class="kv-line">
            <span class="kv" v-for="f in videoFields" :key="f.label" v-tooltip="f.title">
              <em>{{ f.label }}</em>
              <span>{{ f.value }}</span>
            </span>
          </dd>
        </div>
        <div class="wide" v-if="hasAudioFields">
          <dt>音频</dt>
          <dd class="kv-line">
            <span class="kv" v-for="f in audioFields" :key="f.label" v-tooltip="f.title">
              <em>{{ f.label }}</em>
              <span>{{ f.value }}</span>
            </span>
          </dd>
        </div>
      </dl>
    </template>
  </section>
</template>
