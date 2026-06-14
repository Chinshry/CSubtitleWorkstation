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
  type TsSegment
} from '../api/mediaTool'
import { globalDragActive, pendingDrop } from '../stores/dropStore'
import { ffmpegChecking, ffmpegStatus, initFfmpegStatus, refreshFfmpegStatus } from '../stores/ffmpegStore'
import type { CompressStatus } from '../types'
import CommandPreviewCard from '../components/CommandPreviewCard.vue'
import CommandTaskActions from '../components/CommandTaskActions.vue'
import JobLogPanel from '../components/JobLogPanel.vue'
import PathPickerField from '../components/PathPickerField.vue'

const mode = ref<MediaToolMode>('remuxToMp4')
const inputPath = ref('')
const coverPath = ref('')
const audioPath = ref('')
const outputPath = ref('')
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

const segmentTotalBytes = computed(() => (
  segments.value.reduce((sum, item) => sum + item.sizeBytes, 0)
))

const visibleSegments = computed(() => segments.value.slice(0, 12))
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
  if (!ffmpegStatus.value?.available) return '请先在设置页配置可用的 ffmpeg'
  if (!inputPath.value.trim()) return mode.value === 'concatTsToMp4' ? '请选择分片目录' : '请选择输入视频'
  if (mode.value === 'addCoverToMp4' && !coverPath.value.trim()) return '请选择封面图片'
  if (mode.value === 'mergeAudioVideo' && !audioPath.value.trim()) return '请选择音频来源'
  if (!outputPath.value.trim()) return '请选择输出 MP4 路径'
  if (outputConflictsWithSource.value) return '输出路径不能和输入文件相同'
  if (mode.value === 'concatTsToMp4') {
    if (segmentsLoading.value) return '正在读取分片列表'
    if (!segments.value.length) return '所选目录中没有可合并的 TS / M2TS / MTS 分片'
  }
  return '可以开始转换'
})
const canRun = computed(() => runDisabledTip.value === '可以开始转换')
const dragHint = computed(() => (
  mode.value === 'addCoverToMp4'
    ? '松开以读取视频或封面图片'
    : mode.value === 'mergeAudioVideo'
      ? '松开以读取视频或音频来源文件'
    : mode.value === 'concatTsToMp4'
    ? '松开以读取 TS 分片目录或分片文件'
    : '松开以读取视频文件'
))

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

function createJob(): MediaToolJob {
  return {
    id: jobId.value,
    mode: mode.value,
    inputPath: inputPath.value,
    coverPath: coverPath.value || undefined,
    audioPath: audioPath.value || undefined,
    outputPath: outputPath.value
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
  if (mode.value === next) return
  mode.value = next
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
  const file = mode.value === 'concatTsToMp4'
    ? `${parts.stem || 'segments'} 合并.mp4`
    : mode.value === 'addCoverToMp4'
      ? `${parts.stem} 添加封面.mp4`
      : mode.value === 'mergeAudioVideo'
        ? `${parts.stem} 合并音频.mp4`
      : `${parts.stem} MP4封装.mp4`
  return parts.dir ? `${parts.dir}${parts.sep}${file}` : file
}

function applyAutoOutput() {
  if (!inputPath.value.trim()) return
  outputPath.value = outputForInput(inputPath.value)
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
    title: mode.value === 'addCoverToMp4' ? '选择要添加封面的 MP4 视频' : '选择要转为 MP4 封装的视频文件',
    multiple: false,
    filters: [
      {
        name: '视频',
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
    title: '选择封面图片',
    multiple: false,
    filters: [{ name: '封面图片', extensions: ['jpg', 'jpeg', 'png'] }]
  })
  if (typeof selected === 'string') {
    coverPath.value = selected
  }
}

async function pickAudioFile() {
  if (running.value) return
  const selected = await open({
    title: '选择要合并的音频来源文件',
    multiple: false,
    filters: [
      {
        name: '音频或视频',
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
    title: '选择 TS 分片所在文件夹',
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
  const selected = await save({
    title: '选择输出 MP4 文件',
    defaultPath: outputPath.value || outputForInput(inputPath.value) || 'output.mp4',
    filters: [{ name: 'MP4 视频', extensions: ['mp4'] }]
  })
  if (typeof selected === 'string') {
    outputPath.value = selected.toLowerCase().endsWith('.mp4') ? selected : `${selected}.mp4`
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
      segmentError.value = '所选文件夹中没有 TS / M2TS / MTS 分片。'
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
      logs.value.push(`${msg}\n当前音视频流可能不兼容 MP4 容器；请到压制页重新编码后再输出 MP4。`)
    } else {
      logs.value.push(msg)
    }
  }
}

async function cancelJob() {
  try {
    await cancelMediaTool(jobId.value)
    cancelled.value = true
    logs.value.push('已发送取消请求')
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

watch([mode, inputPath, coverPath, audioPath, outputPath, segments], () => {
  if (running.value) return
  if (previewTimer) clearTimeout(previewTimer)
  previewTimer = setTimeout(() => {
    void previewCommand()
  }, 300)
}, { deep: true })

watch(pendingDrop, (drop) => {
  if (!drop) return
  if (drop.target !== 'tools' || drop.tool !== 'media-remux') return
  applyDroppedPaths(drop.raw, drop.videoPath)
  pendingDrop.value = null
})

onMounted(async () => {
  void initFfmpegStatus()
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
  if (pendingDrop.value?.target === 'tools' && pendingDrop.value.tool === 'media-remux') {
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
    <div v-if="globalDragActive" class="drop-overlay">{{ dragHint }}</div>

    <div v-if="ffmpegChecking" class="ffmpeg-missing ffmpeg-checking">
      <strong>正在检测 ffmpeg 环境</strong>
      <span>正在检测 ffmpeg / ffprobe，请稍候。</span>
    </div>
    <div v-else-if="ffmpegStatus && !ffmpegStatus.available" class="ffmpeg-missing">
      <strong>{{ ffmpegStatus.ffmpegPath ? 'ffmpeg 功能不完整' : '未检测到 ffmpeg' }}</strong>
      <span>{{ ffmpegStatus.message ?? '请前往左侧「设置」面板配置 ffmpeg 路径，或安装后将其加入系统 PATH。' }}</span>
      <button class="secondary" @click="refreshFfmpeg">重新检测</button>
    </div>

    <section class="panel media-tool-panel">
      <div class="media-tool-heading">
        <div>
          <h2>封装转换</h2>
          <p>默认只复制音视频流，不重新编码；不兼容 MP4 的素材请回到压制页重新编码。</p>
        </div>
        <div class="mode-tabs" role="tablist" aria-label="封装转换模式">
          <button
            type="button"
            :class="{ active: mode === 'remuxToMp4' }"
            :disabled="running"
            role="tab"
            :aria-selected="mode === 'remuxToMp4'"
            @click="setMode('remuxToMp4')"
          >
            单文件转 MP4
          </button>
          <button
            type="button"
            :class="{ active: mode === 'concatTsToMp4' }"
            :disabled="running"
            role="tab"
            :aria-selected="mode === 'concatTsToMp4'"
            @click="setMode('concatTsToMp4')"
          >
            TS 分片合并
          </button>
          <button
            type="button"
            :class="{ active: mode === 'addCoverToMp4' }"
            :disabled="running"
            role="tab"
            :aria-selected="mode === 'addCoverToMp4'"
            @click="setMode('addCoverToMp4')"
          >
            添加封面
          </button>
          <button
            type="button"
            :class="{ active: mode === 'mergeAudioVideo' }"
            :disabled="running"
            role="tab"
            :aria-selected="mode === 'mergeAudioVideo'"
            @click="setMode('mergeAudioVideo')"
          >
            合并音视频
          </button>
        </div>
      </div>

      <div class="media-tool-grid" :class="{ 'has-extra-input': mode === 'addCoverToMp4' || mode === 'mergeAudioVideo' }">
        <PathPickerField
          v-model="inputPath"
          :label="mode === 'concatTsToMp4' ? '分片目录' : '输入视频'"
          :placeholder="mode === 'concatTsToMp4' ? '选择包含 .ts / .m2ts / .mts 的文件夹' : mode === 'addCoverToMp4' ? '选择 mp4 / m4v / mov 视频文件' : mode === 'mergeAudioVideo' ? '选择要保留画面的视频文件' : '选择 mkv / mov / ts / flv 等视频文件'"
          :disabled="running"
          @pick="mode === 'concatTsToMp4' ? pickSegmentFolder() : pickInputFile()"
        />

        <PathPickerField
          v-if="mode === 'mergeAudioVideo'"
          v-model="audioPath"
          label="输入音频"
          placeholder="选择音频或视频文件"
          :disabled="running"
          @pick="pickAudioFile"
        />

        <PathPickerField
          v-if="mode === 'addCoverToMp4'"
          v-model="coverPath"
          label="封面图片"
          placeholder="选择 jpg / png 封面图片"
          :disabled="running"
          @pick="pickCoverFile"
        />

        <PathPickerField
          v-model="outputPath"
          label="输出 MP4"
          placeholder="选择输出位置"
          :disabled="running || !inputPath"
          @pick="pickOutputPath"
        />
      </div>
      <p v-if="outputConflictsWithSource" class="form-warning">
        输出路径不能和输入文件相同，请选择一个新的 MP4 文件。
      </p>
      <div v-if="mode === 'remuxToMp4'" class="tool-note">
        <strong>处理说明</strong>
        <span>将视频换成 MP4 容器，视频流和音频流默认原样复制，不重新编码。</span>
        <span>TS / M2TS / MTS 输入会自动整理 AAC 音频封装头，让它符合 MP4 规范；这不会改变音质。</span>
      </div>

      <div v-else-if="mode === 'addCoverToMp4'" class="tool-note">
        <strong>处理说明</strong>
        <span>在 MP4 中写入一张封面图，适合让播放器和文件管理器显示自定义封面。</span>
        <span>原视频和音频会原样复制，不重新编码；封面图会作为封面流写入文件。</span>
      </div>

      <div v-else-if="mode === 'mergeAudioVideo'" class="tool-note">
        <strong>处理说明</strong>
        <span>保留输入视频的画面，并把音频来源文件的第一条音轨作为输出文件的主音轨。</span>
        <span>视频和音频默认原样复制，不重新编码；输出会按较短的一路结束，避免尾部空跑。</span>
      </div>

      <div v-else class="segments-panel">
        <div class="segments-head">
          <strong>分片顺序预览</strong>
          <span v-if="segmentsLoading">读取中...</span>
          <span v-else-if="segments.length">{{ segments.length }} 个文件 · {{ formatBytes(segmentTotalBytes) }}</span>
          <span v-else>尚未读取到分片</span>
        </div>
        <p v-if="segmentError" class="segment-error">{{ segmentError }}</p>
        <ol v-else-if="visibleSegments.length" class="segment-list">
          <li v-for="item in visibleSegments" :key="item.path">
            <span>{{ item.name }}</span>
            <em>{{ formatBytes(item.sizeBytes) }}</em>
          </li>
        </ol>
        <p v-if="segments.length > visibleSegments.length" class="muted">
          仅显示前 {{ visibleSegments.length }} 个；实际会按当前排序合并全部 {{ segments.length }} 个分片。
        </p>
        <p class="muted">
          合并输出 MP4 时会自动整理 TS 分片中 AAC 音频的封装头，不重新编码。
        </p>
      </div>
    </section>

    <CommandPreviewCard v-if="command.length && showCommandPreview" :command="command" />

    <CommandTaskActions
      v-model:preview-open="showCommandPreview"
      :command="command"
      :running="running"
      :can-run="canRun"
      start-label="开始转换"
      cancel-label="取消转换"
      preview-disabled-tip="选择输入和输出后自动生成命令"
      :run-disabled-tip="runDisabledTip"
      @run="runJob"
      @cancel="cancelJob"
    />
    <JobLogPanel
      title="转换进度"
      idle-title="尚未开始转换"
      idle-tip="选择输入和输出后点击上方「开始转换」按钮"
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

.media-tool-heading {
  align-items: flex-start;
  display: flex;
  gap: 16px;
  justify-content: space-between;
}

.media-tool-heading h2 {
  color: #102030;
  font-size: 18px;
  margin: 0;
}

.media-tool-heading p {
  color: #667582;
  font-size: 13px;
  margin: 6px 0 0;
}

.mode-tabs {
  background: #eef3f6;
  border: 1px solid #d6e0e7;
  border-radius: 8px;
  display: inline-flex;
  flex: 0 0 auto;
  gap: 4px;
  padding: 4px;
}

.mode-tabs button {
  background: transparent;
  border: 0;
  color: #536474;
  font-size: 13px;
  font-weight: 750;
  line-height: 1;
  min-height: 34px;
  padding: 0 12px;
}

.mode-tabs button.active {
  background: #176b87;
  color: #fff;
}

.media-tool-grid {
  display: grid;
  gap: 12px;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
}

.media-tool-grid.has-extra-input {
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr) minmax(0, 1fr);
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
  .media-tool-heading {
    align-items: stretch;
    flex-direction: column;
  }

  .mode-tabs {
    width: fit-content;
  }

  .media-tool-grid {
    grid-template-columns: 1fr;
  }

  .media-tool-grid.has-extra-input {
    grid-template-columns: 1fr;
  }
}
</style>
