<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { open, save } from '@tauri-apps/plugin-dialog'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import {
  cancelMediaTool,
  listTsSegments,
  previewMediaToolCommand,
  startMediaTool,
  type MediaToolJob,
  type MediaToolMode,
  type MediaOutputFormat,
  type TsSegment
} from '../api/mediaTool'
import { globalDragActive, pendingDrop } from '../stores/dropStore'
import { ffmpegChecking, ffmpegStatus, refreshFfmpegStatus } from '../stores/ffmpegStore'
import {
  activeMediaToolMode,
  activeTool,
  isMediaToolId,
  mediaToolIdByMode,
  mediaToolModeByToolId
} from '../stores/toolStore'
import type { CompressStatus } from '../types'
import AppSelect from '../components/AppSelect.vue'
import CommandPreviewCard from '../components/CommandPreviewCard.vue'
import CommandTaskActions from '../components/CommandTaskActions.vue'
import JobLogPanel from '../components/JobLogPanel.vue'
import PathPickerField from '../components/PathPickerField.vue'
import { useI18n } from '../i18n'

const { t } = useI18n()

const mode = ref<MediaToolMode>(activeMediaToolMode.value)
const inputPath = ref('')
const coverPath = ref('')
const audioPath = ref('')
const outputPath = ref('')
const outputFormat = ref<MediaOutputFormat>('mp4')
const command = ref<string[]>([])
const logs = ref<string[]>([])
const showCommandPreview = ref(false)
const segments = ref<TsSegment[]>([])
const segmentsLoading = ref(false)
const segmentError = ref('')
const running = ref(false)
const cancelled = ref(false)
const percent = ref(0)
const statusLine = ref('')
const currentSeconds = ref(0)
const durationSeconds = ref(0)
const sizeKb = ref(0)
const speed = ref(0)
const fps = ref(0)
const bitrateKbps = ref(0)
const elapsedSeconds = ref(0)
const smoothSpeed = ref(0)
const startedAt = ref<number | null>(null)
const jobId = ref(crypto.randomUUID())
const unlisteners: UnlistenFn[] = []
let elapsedTicker: ReturnType<typeof setInterval> | null = null
let previewTimer: ReturnType<typeof setTimeout> | null = null
let segmentTimer: ReturnType<typeof setTimeout> | null = null

const outputFormatOptions: { value: MediaOutputFormat; label: string }[] = [
  { value: 'mp4', label: 'MP4' },
  { value: 'ts', label: 'TS' }
]

const segmentTotalBytes = computed(() => (
  segments.value.reduce((sum, item) => sum + item.sizeBytes, 0)
))

const visibleSegments = computed(() => segments.value.slice(0, 12))
const activeOutputFormat = computed<MediaOutputFormat>(() => (
  mode.value === 'concatTsToMp4' ? outputFormat.value : 'mp4'
))
const outputFormatLabel = computed(() => activeOutputFormat.value.toUpperCase())
const sourcePaths = computed(() => [
  inputPath.value,
  mode.value === 'addCoverToMp4' ? coverPath.value : '',
  mode.value === 'mergeAudioVideo' ? audioPath.value : ''
].filter((path) => path.trim()))
const outputConflictsWithSource = computed(() => {
  const output = normalizePathForCompare(outputPath.value)
  if (!output) return false
  return sourcePaths.value.some((path) => normalizePathForCompare(path) === output)
})
const runDisabledTip = computed(() => {
  if (ffmpegChecking.value) return t('mediaTool.disabled.checking')
  if (ffmpegStatus.value && !ffmpegStatus.value.available) return t('mediaTool.disabled.ffmpegUnavailable')
  if (!inputPath.value.trim()) return mode.value === 'concatTsToMp4' ? t('mediaTool.disabled.segmentFolder') : t('mediaTool.disabled.inputVideo')
  if (mode.value === 'addCoverToMp4' && !coverPath.value.trim()) return t('mediaTool.disabled.cover')
  if (mode.value === 'mergeAudioVideo' && !audioPath.value.trim()) return t('mediaTool.disabled.audio')
  if (!outputPath.value.trim()) return t('mediaTool.disabled.output', { format: outputFormatLabel.value })
  if (outputConflictsWithSource.value) return t('mediaTool.disabled.conflict')
  if (mode.value === 'concatTsToMp4') {
    if (segmentsLoading.value) return t('mediaTool.disabled.loadingSegments')
    if (!segments.value.length) return t('mediaTool.disabled.noSegments')
  }
  return t('mediaTool.ready')
})
const canRun = computed(() => runDisabledTip.value === t('mediaTool.ready'))
const remainingSeconds = computed(() => {
  const dur = durationSeconds.value
  const cur = currentSeconds.value
  const sp = smoothSpeed.value
  if (!dur || !sp || sp <= 0 || cur >= dur) return 0
  return Math.max(0, (dur - cur) / sp)
})

const etaSeconds = computed(() => (
  remainingSeconds.value ? elapsedSeconds.value + remainingSeconds.value : 0
))

const modeDescription = computed(() => (
  mode.value === 'concatTsToMp4'
    ? t('mediaTool.description.concatTsToMp4', { format: outputFormatLabel.value })
    : mode.value === 'addCoverToMp4'
      ? t('mediaTool.description.addCoverToMp4')
      : mode.value === 'mergeAudioVideo'
        ? t('mediaTool.description.mergeAudioVideo')
        : t('mediaTool.description.remuxToMp4')
))

function createJob(): MediaToolJob {
  return {
    id: jobId.value,
    mode: mode.value,
    inputPath: inputPath.value,
    coverPath: coverPath.value || undefined,
    audioPath: audioPath.value || undefined,
    outputPath: outputPath.value,
    outputFormat: activeOutputFormat.value
  }
}

function resetProgress() {
  percent.value = 0
  statusLine.value = ''
  currentSeconds.value = 0
  durationSeconds.value = 0
  sizeKb.value = 0
  speed.value = 0
  fps.value = 0
  bitrateKbps.value = 0
  elapsedSeconds.value = 0
  smoothSpeed.value = 0
  startedAt.value = null
}

function setMode(next: MediaToolMode, reset = true) {
  if (running.value) return
  const nextTool = mediaToolIdByMode[next]
  if (isMediaToolId(activeTool.value) && activeTool.value !== nextTool) {
    activeTool.value = nextTool
  }
  if (mode.value === next) return
  mode.value = next
  activeMediaToolMode.value = next
  if (next !== 'concatTsToMp4') {
    outputFormat.value = 'mp4'
  }
  if (!reset) return
  coverPath.value = ''
  audioPath.value = ''
  if (next === 'concatTsToMp4' && inputPath.value && !isTsPath(inputPath.value)) {
    inputPath.value = ''
  }
  if (next === 'addCoverToMp4' && inputPath.value && !/\.(mp4|m4v|mov)$/i.test(inputPath.value)) {
    inputPath.value = ''
  }
  if (inputPath.value) {
    applyAutoOutput()
  } else {
    outputPath.value = ''
  }
  command.value = []
  showCommandPreview.value = false
  segments.value = []
  segmentError.value = ''
}

function syncModeFromActiveTool() {
  if (!isMediaToolId(activeTool.value)) return
  setMode(mediaToolModeByToolId[activeTool.value])
}

function startElapsedTicker() {
  stopElapsedTicker()
  elapsedTicker = setInterval(() => {
    if (startedAt.value !== null) {
      elapsedSeconds.value = (Date.now() - startedAt.value) / 1000
    }
  }, 500)
}

function stopElapsedTicker() {
  if (elapsedTicker) {
    clearInterval(elapsedTicker)
    elapsedTicker = null
  }
}

function formatError(error: unknown): string {
  if (typeof error === 'string') return error
  const msg = (error as { message?: unknown })?.message
  if (typeof msg === 'string') return msg
  try {
    return JSON.stringify(error)
  } catch {
    return String(error)
  }
}

function splitPath(path: string) {
  const sep = path.includes('\\') ? '\\' : '/'
  const idx = Math.max(path.lastIndexOf('\\'), path.lastIndexOf('/'))
  const dir = idx >= 0 ? path.slice(0, idx) : ''
  const file = idx >= 0 ? path.slice(idx + 1) : path
  const dot = file.lastIndexOf('.')
  const stem = dot > 0 ? file.slice(0, dot) : file
  return { dir, sep, stem }
}

function isVideoPath(path: string) {
  return /\.(mp4|mkv|mov|ts|m4v|flv|avi|webm|wmv|mpg|mpeg|3gp|3g2|rm|rmvb|vob|mts|m2ts|ogv|ogg|divx|asf|f4v|hevc|h265)$/i.test(path)
}

function isTsPath(path: string) {
  return /\.(ts|m2ts|mts)$/i.test(path)
}

function isCoverPath(path: string) {
  return /\.(jpe?g|png)$/i.test(path)
}

function isAudioPath(path: string) {
  return /\.(m4a|aac|mp3|wav|flac|ac3|eac3|opus|ogg)$/i.test(path)
}

function parentDir(path: string) {
  const idx = Math.max(path.lastIndexOf('\\'), path.lastIndexOf('/'))
  return idx >= 0 ? path.slice(0, idx) : ''
}

function sameParent(paths: string[]) {
  if (!paths.length) return ''
  const first = parentDir(paths[0])
  return paths.every((path) => parentDir(path) === first) ? first : ''
}

function normalizePathForCompare(path: string) {
  return path.trim().replace(/[\\/]+/g, '\\').toLowerCase()
}

function outputForInput(path: string) {
  if (!path.trim()) return ''
  const parts = splitPath(path)
  const extension = activeOutputFormat.value
  const file = mode.value === 'concatTsToMp4'
    ? t('mediaTool.outputName.concatTsToMp4', { stem: parts.stem || 'segments', extension })
    : mode.value === 'addCoverToMp4'
      ? t('mediaTool.outputName.addCoverToMp4', { stem: parts.stem })
      : mode.value === 'mergeAudioVideo'
        ? t('mediaTool.outputName.mergeAudioVideo', { stem: parts.stem })
      : t('mediaTool.outputName.remuxToMp4', { stem: parts.stem })
  return parts.dir ? `${parts.dir}${parts.sep}${file}` : file
}

function withOutputExtension(path: string, extension = activeOutputFormat.value) {
  if (!path.trim()) return path
  return path.replace(/\.(mp4|ts)$/i, '') + `.${extension}`
}

function applyAutoOutput() {
  if (!inputPath.value.trim()) return
  outputPath.value = outputForInput(inputPath.value)
}

function clearInput() {
  inputPath.value = ''
  coverPath.value = ''
  audioPath.value = ''
  outputPath.value = ''
  command.value = []
  showCommandPreview.value = false
  segments.value = []
  segmentError.value = ''
}

function applyDroppedPaths(paths: string[], videoPath?: string) {
  if (running.value) return
  if (!paths.length && !videoPath) return
  const coverFile = paths.find(isCoverPath)
  const audioFile = paths.find(isAudioPath)
  const videoFiles = paths.filter(isVideoPath)
  const droppedVideo = videoPath || videoFiles[0]
  const videoAudioSource = videoFiles.find((path) => path !== droppedVideo)
  const audioSourceFile = audioFile || videoAudioSource
  if (mode.value === 'mergeAudioVideo') {
    if (droppedVideo) {
      inputPath.value = droppedVideo
      applyAutoOutput()
    }
    if (audioSourceFile) {
      audioPath.value = audioSourceFile
    }
    return
  }

  if (audioSourceFile && droppedVideo) {
    setMode('mergeAudioVideo', false)
    inputPath.value = droppedVideo
    audioPath.value = audioSourceFile
    applyAutoOutput()
    return
  }

  if (audioFile) {
    setMode('mergeAudioVideo', false)
    audioPath.value = audioFile
    return
  }

  if (mode.value === 'addCoverToMp4') {
    if (droppedVideo) {
      inputPath.value = droppedVideo
      applyAutoOutput()
    }
    if (coverFile) {
      coverPath.value = coverFile
    }
    return
  }

  if (coverFile && droppedVideo) {
    setMode('addCoverToMp4', false)
    inputPath.value = droppedVideo
    coverPath.value = coverFile
    applyAutoOutput()
    return
  }

  if (coverFile) {
    setMode('addCoverToMp4', false)
    coverPath.value = coverFile
    return
  }

  const tsFiles = paths.filter(isTsPath)
  if (tsFiles.length > 1) {
    const folder = sameParent(tsFiles)
    if (folder) {
      setMode('concatTsToMp4', false)
      inputPath.value = folder
      applyAutoOutput()
      return
    }
  }

  const first = videoPath || paths.find(isVideoPath) || paths[0]
  if (!first) return

  if (mode.value === 'concatTsToMp4' && isTsPath(first)) {
    const folder = parentDir(first)
    if (folder) {
      inputPath.value = folder
      applyAutoOutput()
      return
    }
  }

  if (!isVideoPath(first)) {
    setMode('concatTsToMp4', false)
    inputPath.value = first
    applyAutoOutput()
    return
  }

  setMode('remuxToMp4', false)
  inputPath.value = first
  applyAutoOutput()
}

async function pickInputFile() {
  if (running.value) return
  const selected = await open({
    title: mode.value === 'addCoverToMp4' ? t('mediaTool.dialog.inputCoverVideo') : t('mediaTool.dialog.inputVideo'),
    multiple: false,
    filters: [
      {
        name: t('mediaTool.dialog.videoFilter'),
        extensions: mode.value === 'addCoverToMp4'
          ? ['mp4', 'm4v', 'mov']
          : ['mp4', 'mkv', 'mov', 'm4v', 'ts', 'm2ts', 'mts', 'flv', 'avi', 'webm', 'wmv', 'mpg', 'mpeg', '3gp']
      }
    ]
  })
  if (typeof selected === 'string') {
    inputPath.value = selected
    applyAutoOutput()
  }
}

async function pickCoverFile() {
  if (running.value) return
  const selected = await open({
    title: t('mediaTool.dialog.cover'),
    multiple: false,
    filters: [{ name: t('mediaTool.dialog.coverFilter'), extensions: ['jpg', 'jpeg', 'png'] }]
  })
  if (typeof selected === 'string') {
    coverPath.value = selected
  }
}

async function pickAudioFile() {
  if (running.value) return
  const selected = await open({
    title: t('mediaTool.dialog.audio'),
    multiple: false,
    filters: [
      {
        name: t('mediaTool.dialog.audioVideoFilter'),
        extensions: [
          'm4a', 'aac', 'mp3', 'wav', 'flac', 'ac3', 'eac3', 'opus', 'ogg',
          'mp4', 'mkv', 'mov', 'm4v', 'ts', 'm2ts', 'mts', 'flv', 'avi', 'webm', 'wmv', 'mpg', 'mpeg', '3gp'
        ]
      }
    ]
  })
  if (typeof selected === 'string') {
    audioPath.value = selected
  }
}

async function pickSegmentFolder() {
  if (running.value) return
  const selected = await open({
    title: t('mediaTool.dialog.segmentFolder'),
    directory: true,
    multiple: false
  })
  if (typeof selected === 'string') {
    inputPath.value = selected
    applyAutoOutput()
  }
}

async function pickOutputPath() {
  if (running.value) return
  const extension = activeOutputFormat.value
  const selected = await save({
    title: t('mediaTool.dialog.output', { format: extension.toUpperCase() }),
    defaultPath: outputPath.value || outputForInput(inputPath.value) || `output.${extension}`,
    filters: [{ name: t('mediaTool.dialog.outputFilter', { format: extension.toUpperCase() }), extensions: [extension] }]
  })
  if (typeof selected === 'string') {
    outputPath.value = selected.toLowerCase().endsWith(`.${extension}`) ? selected : `${selected}.${extension}`
  }
}

async function refreshFfmpeg() {
  try {
    await refreshFfmpegStatus()
  } catch (error) {
    logs.value.push(formatError(error))
  }
}

async function refreshSegments() {
  segments.value = []
  segmentError.value = ''
  const path = inputPath.value.trim()
  if (mode.value !== 'concatTsToMp4' || !path) return
  segmentsLoading.value = true
  try {
    segments.value = await listTsSegments(path)
    if (!segments.value.length) {
      segmentError.value = t('mediaTool.segments.noSegments')
    }
  } catch (error) {
    segmentError.value = formatError(error)
  } finally {
    segmentsLoading.value = false
  }
}

async function previewCommand() {
  if (!inputPath.value.trim() || !outputPath.value.trim()) {
    command.value = []
    return
  }
  if (mode.value === 'addCoverToMp4' && !coverPath.value.trim()) {
    command.value = []
    return
  }
  if (mode.value === 'mergeAudioVideo' && !audioPath.value.trim()) {
    command.value = []
    return
  }
  try {
    command.value = await previewMediaToolCommand(createJob())
  } catch {
    command.value = []
  }
}

async function runJob() {
  if (!canRun.value) return
  logs.value = []
  resetProgress()
  cancelled.value = false
  running.value = true
  jobId.value = crypto.randomUUID()
  statusLine.value = 'Preparing media tool job...'
  try {
    await previewCommand()
    await startMediaTool(createJob())
  } catch (error) {
    running.value = false
    stopElapsedTicker()
    const msg = formatError(error)
    if (/codec|Invalid data|not currently supported|Could not write header/i.test(msg)) {
      logs.value.push(`${msg}\n${t('mediaTool.errors.incompatibleContainer', { format: outputFormatLabel.value })}`)
    } else {
      logs.value.push(msg)
    }
  }
}

async function cancelJob() {
  try {
    await cancelMediaTool(jobId.value)
    cancelled.value = true
    logs.value.push(t('mediaTool.cancelRequested'))
  } catch (error) {
    logs.value.push(formatError(error))
    running.value = false
    stopElapsedTicker()
  }
}

watch([mode, inputPath], () => {
  if (segmentTimer) clearTimeout(segmentTimer)
  segmentTimer = setTimeout(() => {
    void refreshSegments()
  }, 250)
})

watch(activeTool, () => {
  syncModeFromActiveTool()
})

watch([mode, inputPath, coverPath, audioPath, outputPath, outputFormat, segments], () => {
  if (running.value) return
  if (previewTimer) clearTimeout(previewTimer)
  previewTimer = setTimeout(() => {
    void previewCommand()
  }, 300)
}, { deep: true })

watch(outputFormat, (format) => {
  if (mode.value !== 'concatTsToMp4') return
  if (outputPath.value.trim()) {
    outputPath.value = withOutputExtension(outputPath.value, format)
  } else if (inputPath.value.trim()) {
    applyAutoOutput()
  }
})

watch(pendingDrop, (drop) => {
  if (!drop) return
  if (drop.target !== 'tools' || !drop.tool || !isMediaToolId(drop.tool)) return
  applyDroppedPaths(drop.raw, drop.videoPath)
  pendingDrop.value = null
})

onMounted(async () => {
  syncModeFromActiveTool()
  unlisteners.push(
    await listen<string>('media-tool-log', (event) => {
      logs.value.push(event.payload)
      if (/Media tool (completed|failed|exited)/.test(event.payload)) {
        running.value = false
        if (startedAt.value !== null) {
          elapsedSeconds.value = (Date.now() - startedAt.value) / 1000
        }
        stopElapsedTicker()
      }
    })
  )
  unlisteners.push(
    await listen<CompressStatus>('media-tool-status', (event) => {
      if (event.payload.jobId !== jobId.value) return
      const p = event.payload
      if (startedAt.value === null) {
        startedAt.value = Date.now()
        startElapsedTicker()
      }
      statusLine.value = p.statusLine ?? ''
      if (typeof p.percent === 'number') percent.value = p.percent
      if (typeof p.currentSeconds === 'number') currentSeconds.value = p.currentSeconds
      if (typeof p.durationSeconds === 'number') durationSeconds.value = p.durationSeconds
      if (typeof p.sizeKb === 'number') sizeKb.value = p.sizeKb
      if (typeof p.speed === 'number') {
        speed.value = p.speed
        if (p.speed > 0) {
          smoothSpeed.value = smoothSpeed.value > 0
            ? smoothSpeed.value * 0.7 + p.speed * 0.3
            : p.speed
        }
      }
      if (typeof p.fps === 'number') fps.value = p.fps
      if (typeof p.bitrateKbps === 'number') bitrateKbps.value = p.bitrateKbps
    })
  )
  if (
    pendingDrop.value?.target === 'tools' &&
    pendingDrop.value.tool &&
    isMediaToolId(pendingDrop.value.tool)
  ) {
    const drop = pendingDrop.value
    applyDroppedPaths(drop.raw, drop.videoPath)
    pendingDrop.value = null
  }
})

onUnmounted(() => {
  for (const unlisten of unlisteners) unlisten()
  stopElapsedTicker()
  if (previewTimer) clearTimeout(previewTimer)
  if (segmentTimer) clearTimeout(segmentTimer)
})

function formatBytes(bytes: number) {
  if (!bytes || bytes <= 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let value = bytes
  let unit = units[0]
  for (const next of units.slice(1)) {
    if (value < 1024) break
    value /= 1024
    unit = next
  }
  return unit === 'B' ? `${bytes} B` : `${value.toFixed(2)} ${unit}`
}
</script>

<template>
  <section class="media-remux-workspace">
    <div v-if="ffmpegChecking" class="ffmpeg-missing ffmpeg-checking">
      <strong>{{ t('ffmpegPanel.checkingTitle') }}</strong>
      <span>{{ t('ffmpegPanel.checkingSubtitle') }}</span>
    </div>
    <div v-else-if="ffmpegStatus && !ffmpegStatus.available" class="ffmpeg-missing">
      <strong>{{ ffmpegStatus.ffmpegPath ? t('ffmpegPanel.incomplete') : t('ffmpegPanel.missing') }}</strong>
      <span>{{ ffmpegStatus.message ?? t('ffmpegPanel.missingHelp') }}</span>
      <button class="secondary" @click="refreshFfmpeg">{{ t('ffmpegPanel.refresh') }}</button>
    </div>

    <section class="panel media-tool-panel" :class="{ 'is-empty': !inputPath, 'drag-target': globalDragActive }">
      <div v-if="!inputPath" class="tool-dropzone">
        <div class="dropzone-icon">⬇︎</div>
        <div class="dropzone-title">{{ mode === 'concatTsToMp4' ? t('mediaTool.dropzone.concatTitle') : t('mediaTool.dropzone.videoTitle') }}</div>
        <div class="dropzone-sub">
          <span class="dropzone-note">
            {{ mode === 'concatTsToMp4' ? t('mediaTool.dropzone.concatNote') : mode === 'addCoverToMp4' ? t('mediaTool.dropzone.coverNote') : t('mediaTool.dropzone.videoNote') }}
          </span>
          <br />
          {{ mode === 'concatTsToMp4' ? t('mediaTool.dropzone.concatDescription') : mode === 'mergeAudioVideo' ? t('mediaTool.dropzone.mergeDescription') : mode === 'addCoverToMp4' ? t('mediaTool.dropzone.coverDescription') : t('mediaTool.dropzone.remuxDescription') }}
        </div>
        <div class="dropzone-actions">
          <button class="secondary" type="button" @click="mode === 'concatTsToMp4' ? pickSegmentFolder() : pickInputFile()">
            {{ mode === 'concatTsToMp4' ? t('mediaTool.chooseSegmentFolder') : t('mediaTool.chooseVideo') }}
          </button>
        </div>
      </div>

      <div v-else class="media-tool-grid" :class="{ 'has-extra-input': mode === 'addCoverToMp4' || mode === 'mergeAudioVideo', 'has-format': mode === 'concatTsToMp4' }">
        <PathPickerField
          v-model="inputPath"
          :label="mode === 'concatTsToMp4' ? t('mediaTool.input.segmentFolder') : t('mediaTool.input.video')"
          :placeholder="mode === 'concatTsToMp4' ? t('mediaTool.input.segmentPlaceholder') : mode === 'addCoverToMp4' ? t('mediaTool.input.coverVideoPlaceholder') : mode === 'mergeAudioVideo' ? t('mediaTool.input.mergeVideoPlaceholder') : t('mediaTool.input.videoPlaceholder')"
          :disabled="running"
          compact
          compact-action="clear"
          @pick="mode === 'concatTsToMp4' ? pickSegmentFolder() : pickInputFile()"
          @clear="clearInput"
        />

        <PathPickerField
          v-if="mode === 'mergeAudioVideo'"
          v-model="audioPath"
          :label="t('mediaTool.input.audio')"
          :placeholder="t('mediaTool.input.audioPlaceholder')"
          :disabled="running"
          compact
          compact-action="clear"
          @pick="pickAudioFile"
        />

        <PathPickerField
          v-if="mode === 'addCoverToMp4'"
          v-model="coverPath"
          :label="t('mediaTool.input.cover')"
          :placeholder="t('mediaTool.input.coverPlaceholder')"
          :disabled="running"
          compact
          compact-action="clear"
          @pick="pickCoverFile"
        />

        <label v-if="mode === 'concatTsToMp4'" class="format-field">
          <span>{{ t('mediaTool.outputFormat') }}</span>
          <AppSelect
            v-model="outputFormat"
            class="output-format-select"
            :disabled="running"
            :title="t('mediaTool.outputFormatTitle')"
            :options="outputFormatOptions"
          />
        </label>

        <PathPickerField
          v-model="outputPath"
          :label="t('mediaTool.outputLabel', { format: outputFormatLabel })"
          :placeholder="t('mediaTool.outputPlaceholder')"
          :disabled="running || !inputPath"
          compact
          compact-action="edit"
          @pick="pickOutputPath"
        />
      </div>
      <p v-if="outputConflictsWithSource" class="form-warning">
        {{ t('mediaTool.conflictWarning', { format: outputFormatLabel }) }}
      </p>
      <div v-if="mode === 'remuxToMp4'" class="tool-note">
        <strong>{{ t('mediaTool.noteTitle') }}</strong>
        <span>{{ t('mediaTool.notes.remux1') }}</span>
        <span>{{ t('mediaTool.notes.remux2') }}</span>
      </div>

      <div v-else-if="mode === 'addCoverToMp4'" class="tool-note">
        <strong>{{ t('mediaTool.noteTitle') }}</strong>
        <span>{{ t('mediaTool.notes.cover1') }}</span>
        <span>{{ t('mediaTool.notes.cover2') }}</span>
      </div>

      <div v-else-if="mode === 'mergeAudioVideo'" class="tool-note">
        <strong>{{ t('mediaTool.noteTitle') }}</strong>
        <span>{{ t('mediaTool.notes.merge1') }}</span>
        <span>{{ t('mediaTool.notes.merge2') }}</span>
      </div>

      <div v-else class="segments-panel">
        <div class="segments-head">
          <strong>{{ t('mediaTool.segments.title') }}</strong>
          <span v-if="segmentsLoading">{{ t('mediaTool.segments.loading') }}</span>
          <span v-else-if="segments.length">{{ t('mediaTool.segments.count', { count: segments.length, size: formatBytes(segmentTotalBytes) }) }}</span>
          <span v-else>{{ t('mediaTool.segments.empty') }}</span>
        </div>
        <p v-if="segmentError" class="segment-error">{{ segmentError }}</p>
        <ol v-else-if="visibleSegments.length" class="segment-list">
          <li v-for="item in visibleSegments" :key="item.path">
            <span>{{ item.name }}</span>
            <em>{{ formatBytes(item.sizeBytes) }}</em>
          </li>
        </ol>
        <p v-if="segments.length > visibleSegments.length" class="muted">
          {{ t('mediaTool.segments.truncated', { visible: visibleSegments.length, total: segments.length }) }}
        </p>
        <p class="muted">
          {{ t('mediaTool.segments.note') }}
        </p>
      </div>
    </section>

    <CommandPreviewCard v-if="command.length && showCommandPreview" :command="command" />

    <CommandTaskActions
      v-model:preview-open="showCommandPreview"
      :command="command"
      :running="running"
      :can-run="canRun"
      :start-label="t('mediaTool.start')"
      :cancel-label="t('mediaTool.cancel')"
      :preview-disabled-tip="t('mediaTool.previewDisabledTip')"
      :run-disabled-tip="runDisabledTip"
      @run="runJob"
      @cancel="cancelJob"
    />
    <JobLogPanel
      :title="t('mediaTool.progressTitle')"
      :idle-title="t('mediaTool.idleTitle')"
      :idle-tip="t('mediaTool.idleTip')"
      :lines="logs"
      :command="command"
      :percent="percent"
      :status-line="statusLine"
      :current-seconds="currentSeconds"
      :duration-seconds="durationSeconds"
      :size-kb="sizeKb"
      :speed="speed"
      :fps="fps"
      :bitrate-kbps="bitrateKbps"
      :elapsed-seconds="elapsedSeconds"
      :eta-seconds="etaSeconds"
      :remaining-seconds="remainingSeconds"
      :running="running"
      :cancelled="cancelled"
    />
  </section>
</template>

<style scoped>
.media-remux-workspace {
  align-content: start;
  display: grid;
  gap: 12px;
  grid-auto-rows: max-content;
  min-height: 0;
  position: relative;
}

.media-tool-panel {
  display: grid;
  gap: 16px;
}

.media-tool-panel.is-empty {
  padding: 32px 22px;
}

.media-tool-panel.drag-target {
  background: #eef7fa;
  border-color: #176b87;
  box-shadow: 0 0 0 3px rgba(23, 107, 135, 0.15);
}

.tool-dropzone {
  align-items: center;
  border: 2px dashed #b8c8d2;
  border-radius: 10px;
  color: #43515c;
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 38px 20px;
  text-align: center;
  transition: border-color 0.18s ease, background 0.18s ease;
}

.media-tool-panel.drag-target .tool-dropzone {
  background: rgba(23, 107, 135, 0.08);
  border-color: #176b87;
  color: #0f5268;
}

.media-tool-panel.drag-target .dropzone-title {
  color: #0f5268;
}

.media-tool-grid {
  display: grid;
  gap: 14px;
  grid-template-columns: 1fr;
}

.media-tool-grid.has-extra-input {
  grid-template-columns: 1fr;
}

.media-tool-grid.has-format {
  grid-template-columns: 1fr;
  max-width: 1320px;
  width: 100%;
}

.format-field {
  align-items: center;
  display: grid;
  gap: 16px;
  grid-template-columns: 76px 100px;
}

.media-tool-grid :deep(.path-picker-field) {
  align-items: center;
  display: grid;
  gap: 16px;
  grid-template-columns: 76px minmax(0, 1fr);
}

.format-field > span,
.media-tool-grid :deep(.path-picker-field > span) {
  background: #176b87;
  border-radius: 4px;
  box-shadow: 0 1px 2px rgba(23, 107, 135, 0.18);
  color: #fff;
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.5px;
  line-height: 1.4;
  padding: 4px 8px;
  text-align: center;
}

.media-tool-grid :deep(.path-picker-control) {
  gap: 8px;
}

.format-field :deep(.app-select) {
  width: 100px;
}

.format-field :deep(.app-select-trigger) {
  gap: 4px;
  padding: 0 7px;
}

.format-field :deep(.app-select-panel) {
  padding: 5px;
}

.format-field :deep(.app-select-option) {
  padding: 7px 6px;
}

.tool-note,
.segments-panel {
  background: #f6fafc;
  border: 1px solid #dbe7ee;
  border-radius: 8px;
  color: #536474;
  display: grid;
  font-size: 13px;
  gap: 8px;
  padding: 12px;
}

.tool-note strong,
.segments-head strong {
  color: #102030;
}

.tool-note code {
  background: #e9f1f5;
  border-radius: 4px;
  color: #0f5268;
  padding: 1px 4px;
}

.form-warning {
  background: #fff7ed;
  border: 1px solid #fed7aa;
  border-radius: 8px;
  color: #9a3412;
  font-size: 13px;
  margin: -4px 0 0;
  padding: 10px 12px;
}

.segments-head {
  align-items: center;
  display: flex;
  justify-content: space-between;
}

.segment-error {
  color: #b42318;
  margin: 0;
}

.segment-list {
  display: grid;
  gap: 4px;
  margin: 0;
  padding-left: 22px;
}

.segment-list li {
  color: #102030;
  display: list-item;
}

.segment-list span {
  word-break: break-all;
}

.segment-list em {
  color: #7b8a96;
  float: right;
  font-style: normal;
  margin-left: 12px;
}

@media (max-width: 920px) {
  .format-field,
  .media-tool-grid :deep(.path-picker-field) {
    grid-template-columns: 1fr;
  }

  .format-field > span,
  .media-tool-grid :deep(.path-picker-field > span) {
    justify-self: start;
    min-width: 76px;
  }
}
</style>
