<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import type { OutputNameTemplate, VideoMeta } from '../types'
import { globalDragActive } from '../stores/dropStore'
import { validateOutputParentDir } from '../api/compress'
import { useToast } from '../composables/useToast'
import { useI18n } from '../i18n'

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
const { t } = useI18n()

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
    title: t('videoMeta.dialog.chooseVideo'),
    filters: [
      { name: t('videoMeta.dialog.videoFilter'), extensions: ['mp4', 'mkv', 'mov', 'ts', 'm4v', 'flv', 'avi', 'webm', 'wmv', 'mpg', 'mpeg', '3gp', 'mts', 'm2ts'] }
    ]
  })
  if (typeof sel === 'string') emit('pick-video', sel)
}

async function chooseSubtitle() {
  const sel = await open({
    multiple: false,
    directory: false,
    title: t('videoMeta.dialog.chooseSubtitle'),
    filters: [
      { name: t('videoMeta.dialog.subtitleFilter'), extensions: ['ass', 'ssa', 'srt', 'vtt', 'sub'] }
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
    title: t('videoMeta.dialog.chooseVideoAndSubtitle'),
    filters: [
      {
        name: t('videoMeta.dialog.videoSubtitleFilter'),
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

const dashText = computed(() => t('videoMeta.dash'))

type Field = { label: string; value: string; title?: string }

function formatBytes(bytes?: number): string {
  if (!bytes || bytes <= 0) return dashText.value
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
    value: t('videoMeta.dimensions', { width: m.width, height: m.height }),
    title: tags.length
      ? t('videoMeta.dimensionTitle', { tags: tags.join('，') })
      : t('videoMeta.dimensionTitlePlain')
  }
})

const durationText = computed(() => {
  const m = props.meta
  if (!m) return dashText.value
  const text = m.durationText ?? dashText.value
  if (typeof m.startSeconds === 'number' && m.startSeconds > 0) {
    return t('videoMeta.durationWithStart', { duration: text, start: m.startSeconds.toFixed(2) })
  }
  return text
})

const videoFields = computed<Field[]>(() => {
  const m = props.meta
  if (!m) return []
  const fields: Field[] = []
  const res = resolutionField.value
  if (res) {
    fields.push({
      label: t('videoMeta.fields.resolution'),
      value: res.value,
      title: res.title
    })
  }
  if (m.videoCodec) {
    fields.push({
      label: t('videoMeta.fields.codec'),
      value: m.videoProfile ? `${m.videoCodec} ${m.videoProfile}` : m.videoCodec,
      title: t('videoMeta.tips.videoCodec')
    })
  }
  if (m.pixelFormat) {
    fields.push({
      label: t('videoMeta.fields.pixel'),
      value: m.pixelFormat,
      title: t('videoMeta.tips.pixelFormat')
    })
  }
  const rate = m.videoBitrateKbps ?? m.overallBitrateKbps
  if (rate) {
    fields.push({
      label: t('videoMeta.fields.bitrate'),
      value: `${rate} kbps`,
      title: t('videoMeta.tips.videoBitrate')
    })
  }
  if (m.fps) {
    fields.push({
      label: t('videoMeta.fields.fps'),
      value: `${m.fps} fps`,
      title: t('videoMeta.tips.fps')
    })
  }
  if (m.frameRateMode) {
    fields.push({
      label: t('videoMeta.fields.frameMode'),
      value: m.frameRateMode,
      title:
        m.frameRateMode === 'CFR'
          ? t('videoMeta.tips.cfr')
          : t('videoMeta.tips.vfr')
    })
  }
  if (m.totalFrames) {
    fields.push({
      label: t('videoMeta.fields.totalFrames'),
      value: `${m.totalFramesEstimated ? t('videoMeta.approx') : ''}${m.totalFrames.toLocaleString()}`,
      title: m.totalFramesEstimated
        ? t('videoMeta.tips.estimatedFrames')
        : t('videoMeta.tips.exactFrames')
    })
  }
  if (m.colorSpace) {
    fields.push({
      label: t('videoMeta.fields.colorSpace'),
      value: m.colorSpace,
      title: t('videoMeta.tips.colorSpace')
    })
  }
  if (m.colorRange) {
    fields.push({
      label: t('videoMeta.fields.colorRange'),
      value: m.colorRange,
      title: t('videoMeta.tips.colorRange')
    })
  }
  return fields
})

const audioFields = computed<Field[]>(() => {
  const m = props.meta
  if (!m || !m.audioCodec) return []
  const fields: Field[] = []
  fields.push({
    label: t('videoMeta.fields.codec'),
    value: m.audioProfile ? `${m.audioCodec} ${m.audioProfile}` : m.audioCodec,
    title: t('videoMeta.tips.audioCodec')
  })
  if (m.audioSampleRate) {
    const khz = (m.audioSampleRate / 1000).toString().replace(/\.0$/, '')
    fields.push({
      label: t('videoMeta.fields.sampleRate'),
      value: `${khz} kHz`,
      title: t('videoMeta.tips.sampleRate')
    })
  }
  if (m.audioChannels) {
    fields.push({
      label: t('videoMeta.fields.channels'),
      value: m.audioChannels,
      title: t('videoMeta.tips.channels')
    })
  }
  if (m.audioBitrateKbps) {
    fields.push({
      label: t('videoMeta.fields.bitrate'),
      value: `${m.audioBitrateKbps} kbps`,
      title: t('videoMeta.tips.audioBitrate')
    })
  }
  return fields
})

const fileText = computed(() => {
  const m = props.meta
  if (!m) return dashText.value
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
    title: t('videoMeta.tips.demuxer', { format: m.format })
  }
})

const hasVideoFields = computed(() => videoFields.value.length > 0)
const hasAudioFields = computed(() => audioFields.value.length > 0)
const templateOptions = computed(() => props.outputTemplates ?? [])

const selectedTemplateId = computed(() => props.selectedOutputTemplateId ?? templateOptions.value[0]?.id ?? '')
const selectedTemplateName = computed(() => {
  return templateOptions.value.find((tpl) => tpl.id === selectedTemplateId.value)?.name ?? t('videoMeta.defaultTemplate')
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
      <div class="dropzone-title">{{ t('videoMeta.empty.title') }}</div>
      <div class="dropzone-sub">
        <span class="dropzone-note">{{ t('videoMeta.empty.note') }}</span>
        <br />
        {{ t('videoMeta.empty.video') }}
        <br />
        {{ t('videoMeta.empty.subtitle') }}
      </div>
      <div class="dropzone-actions">
        <button class="secondary" @click="chooseFiles">{{ t('videoMeta.empty.choose') }}</button>
      </div>
    </div>

    <!-- 正常状态：路径行 + 视频信息 -->
    <template v-else>
      <div class="meta-head">
        <h2>{{ t('videoMeta.title') }}</h2>
        <span v-if="loading" class="meta-status loading">{{ t('videoMeta.parsing') }}</span>
        <span v-else-if="error" class="meta-status error" v-tooltip="error">{{ error }}</span>
      </div>

      <dl class="path-grid">
        <div>
          <dt>{{ t('videoMeta.video') }}</dt>
          <dd class="path-row">
            <span class="path-text readonly" v-tooltip="videoPath || t('videoMeta.notImported')">{{ videoPath || dashText }}</span>
            <button
              v-if="videoPath"
              class="path-action"
              :data-tip="t('videoMeta.clearAndReimport')"
              @click="emit('clear-video')"
              :aria-label="t('videoMeta.clearVideo')"
            >✕</button>
            <button
              v-else
              class="path-action"
              :data-tip="t('videoMeta.chooseVideo')"
              @click="chooseVideo"
              :aria-label="t('videoMeta.chooseVideo')"
            >+</button>
          </dd>
        </div>
        <div>
          <dt>{{ t('videoMeta.subtitle') }}</dt>
          <dd class="path-row">
            <span class="path-text readonly" v-tooltip="subtitlePath || t('videoMeta.notImported')">{{ subtitlePath || dashText }}</span>
            <button
              v-if="subtitlePath"
              class="path-action"
              :data-tip="t('videoMeta.clearAndReimport')"
              @click="emit('clear-subtitle')"
              :aria-label="t('videoMeta.clearSubtitle')"
            >✕</button>
            <button
              v-else
              class="path-action"
              :data-tip="t('videoMeta.chooseSubtitle')"
              @click="chooseSubtitle"
              :aria-label="t('videoMeta.chooseSubtitle')"
            >+</button>
          </dd>
        </div>
        <div>
          <dt>{{ t('videoMeta.output') }}</dt>
          <dd class="path-row">
            <input
              v-if="isEditingOutput"
              v-model="draftOutputPath"
              class="path-input"
              :placeholder="t('videoMeta.outputPlaceholder')"
              @blur="saveOutputEdit"
              @keyup.enter="saveOutputEdit"
              @keyup.esc="isEditingOutput = false"
            />
            <span v-else class="path-text" v-tooltip="outputPath || t('videoMeta.unset')">{{ outputPath || dashText }}</span>
            <div v-if="templateOptions.length" ref="templateMenuRef" class="path-template-menu">
              <button
                class="path-action"
                :class="{ active: isTemplateMenuOpen }"
                :data-tip="t('videoMeta.applyTemplate')"
                type="button"
                @click="toggleTemplateMenu"
                :aria-label="t('videoMeta.applyTemplateLabel', { name: selectedTemplateName })"
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
              :data-tip="t('videoMeta.editOutputPath')"
              type="button"
              @click="isEditingOutput ? (isEditingOutput = false) : beginOutputEdit()"
              :aria-label="t('videoMeta.editOutput')"
            >✎</button>
          </dd>
        </div>
      </dl>

      <dl v-if="meta && !error" class="meta-grid">
        <div class="wide">
          <dt>{{ t('videoMeta.file') }}</dt>
          <dd class="kv-line">
            <span class="kv" v-tooltip="t('videoMeta.fileSize')">
              <em>{{ t('videoMeta.size') }}</em>
              <span>{{ fileText }}</span>
            </span>
            <span class="kv" v-tooltip="t('videoMeta.durationTip')">
              <em>{{ t('videoMeta.duration') }}</em>
              <span>{{ durationText }}</span>
            </span>
            <span class="kv" v-if="containerField" v-tooltip="containerField.title">
              <em>{{ t('videoMeta.container') }}</em>
              <span>{{ containerField.value }}</span>
            </span>
          </dd>
        </div>
        <div class="wide" v-if="hasVideoFields">
          <dt>{{ t('videoMeta.video') }}</dt>
          <dd class="kv-line">
            <span class="kv" v-for="f in videoFields" :key="f.label" v-tooltip="f.title">
              <em>{{ f.label }}</em>
              <span>{{ f.value }}</span>
            </span>
          </dd>
        </div>
        <div class="wide" v-if="hasAudioFields">
          <dt>{{ t('videoMeta.audio') }}</dt>
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
