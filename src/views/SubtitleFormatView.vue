<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from 'vue'
import { open, save } from '@tauri-apps/plugin-dialog'
import {
  convertSubtitleFormat,
  previewSubtitleFormatCommand,
  type SubtitleTargetFormat
} from '../api/subtitleTool'
import { globalDragActive, pendingDrop } from '../stores/dropStore'
import { ffmpegChecking, ffmpegStatus, refreshFfmpegStatus } from '../stores/ffmpegStore'
import AppSelect from '../components/AppSelect.vue'
import CommandPreviewCard from '../components/CommandPreviewCard.vue'
import CommandTaskActions from '../components/CommandTaskActions.vue'
import JobLogPanel from '../components/JobLogPanel.vue'
import PathPickerField from '../components/PathPickerField.vue'

const inputPath = ref('')
const outputPath = ref('')
const targetFormat = ref<SubtitleTargetFormat>('ass')
const command = ref<string[]>([])
const logs = ref<string[]>([])
const running = ref(false)
const jobFailed = ref(false)
const showCommandPreview = ref(false)
let previewTimer: ReturnType<typeof setTimeout> | null = null

const formatOptions: { value: SubtitleTargetFormat; label: string }[] = [
  { value: 'srt', label: 'SRT' },
  { value: 'ass', label: 'ASS' },
  { value: 'ssa', label: 'SSA' },
  { value: 'vtt', label: 'VTT' }
]

const outputConflictsWithInput = computed(() => (
  normalizePathForCompare(inputPath.value) !== '' &&
  normalizePathForCompare(inputPath.value) === normalizePathForCompare(outputPath.value)
))

const runDisabledTip = computed(() => {
  if (ffmpegChecking.value) return '正在检测 ffmpeg'
  if (ffmpegStatus.value && !ffmpegStatus.value.available) return '请先在设置页配置可用的 ffmpeg'
  if (!inputPath.value.trim()) return '请选择输入字幕'
  if (!outputPath.value.trim()) return '请选择输出字幕路径'
  if (outputConflictsWithInput.value) return '输出路径不能和输入字幕相同'
  return '可以开始转换'
})

const canRun = computed(() => runDisabledTip.value === '可以开始转换')
const failed = computed(() => jobFailed.value)
const completed = computed(() => !running.value && logs.value.length > 0 && !failed.value)
const progressPercent = computed(() => {
  if (running.value) return 35
  if (completed.value) return 100
  return 0
})
function createJob() {
  return {
    inputPath: inputPath.value,
    outputPath: outputPath.value,
    targetFormat: targetFormat.value
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

function normalizePathForCompare(path: string) {
  return path.trim().replace(/[\\/]+/g, '\\').toLowerCase()
}

function isSubtitlePath(path: string) {
  return /\.(ass|ssa|srt|vtt|sub)$/i.test(path)
}

function outputForInput(path: string) {
  if (!path.trim()) return ''
  const parts = splitPath(path)
  const file = `${parts.stem} 转换.${targetFormat.value}`
  return parts.dir ? `${parts.dir}${parts.sep}${file}` : file
}

function applyAutoOutput() {
  if (!inputPath.value.trim()) return
  outputPath.value = outputForInput(inputPath.value)
}

function clearInput() {
  inputPath.value = ''
  outputPath.value = ''
  command.value = []
}

function applyDroppedPaths(paths: string[], subtitlePath?: string) {
  if (running.value) return
  const path = subtitlePath || paths.find(isSubtitlePath)
  if (!path) return
  inputPath.value = path
  applyAutoOutput()
}

async function pickInputFile() {
  if (running.value) return
  const selected = await open({
    title: '选择要转换格式的字幕',
    multiple: false,
    filters: [{ name: '字幕', extensions: ['ass', 'ssa', 'srt', 'vtt', 'sub'] }]
  })
  if (typeof selected === 'string') {
    inputPath.value = selected
    applyAutoOutput()
  }
}

async function pickOutputPath() {
  if (running.value) return
  const selected = await save({
    title: '选择输出字幕路径',
    defaultPath: outputPath.value || outputForInput(inputPath.value) || `subtitle.${targetFormat.value}`,
    filters: [{ name: targetFormat.value.toUpperCase(), extensions: [targetFormat.value] }]
  })
  if (typeof selected === 'string') {
    const suffix = `.${targetFormat.value}`
    outputPath.value = selected.toLowerCase().endsWith(suffix) ? selected : `${selected}${suffix}`
  }
}

async function previewCommand() {
  if (!inputPath.value.trim() || !outputPath.value.trim() || outputConflictsWithInput.value) {
    command.value = []
    return
  }
  try {
    command.value = await previewSubtitleFormatCommand(createJob())
  } catch {
    command.value = []
  }
}

async function runJob() {
  if (!canRun.value) return
  running.value = true
  jobFailed.value = false
  logs.value = []
  try {
    await previewCommand()
    const result = await convertSubtitleFormat(createJob())
    logs.value = result.logs.length ? result.logs : [`已输出：${result.outputPath}`]
  } catch (error) {
    jobFailed.value = true
    logs.value = formatError(error).split('\n').filter(Boolean)
    if (!logs.value.length) logs.value = ['Subtitle format conversion failed']
  } finally {
    running.value = false
  }
}

async function refreshFfmpeg() {
  try {
    await refreshFfmpegStatus()
  } catch (error) {
    logs.value.push(formatError(error))
  }
}

watch(targetFormat, () => {
  if (inputPath.value) applyAutoOutput()
})

watch([inputPath, outputPath, targetFormat], () => {
  if (running.value) return
  if (previewTimer) clearTimeout(previewTimer)
  previewTimer = setTimeout(() => {
    void previewCommand()
  }, 250)
})

watch(pendingDrop, (drop) => {
  if (!drop) return
  if (drop.target !== 'tools' || drop.tool !== 'subtitle-format') return
  applyDroppedPaths(drop.raw, drop.subtitlePath)
  pendingDrop.value = null
}, { immediate: true })

onUnmounted(() => {
  if (previewTimer) clearTimeout(previewTimer)
})
</script>

<template>
  <section class="subtitle-format-workspace">
    <div v-if="ffmpegChecking" class="ffmpeg-missing ffmpeg-checking">
      <strong>正在检测 ffmpeg 环境</strong>
      <span>正在检测 ffmpeg / ffprobe，请稍候。</span>
    </div>
    <div v-else-if="ffmpegStatus && !ffmpegStatus.available" class="ffmpeg-missing">
      <strong>{{ ffmpegStatus.ffmpegPath ? 'ffmpeg 功能不完整' : '未检测到 ffmpeg' }}</strong>
      <span>{{ ffmpegStatus.message ?? '请前往左侧「设置」面板配置 ffmpeg 路径，或安装后将其加入系统 PATH。' }}</span>
      <button class="secondary" @click="refreshFfmpeg">重新检测</button>
    </div>

    <section class="panel subtitle-format-panel" :class="{ 'is-empty': !inputPath, 'drag-target': globalDragActive }">
      <div v-if="!inputPath" class="tool-dropzone">
        <div class="dropzone-icon">⬇︎</div>
        <div class="dropzone-title">拖入字幕开始转换</div>
        <div class="dropzone-sub">
          <span class="dropzone-note">支持 ASS / SSA / SRT / VTT / SUB</span>
          <br />
          选择目标格式后会自动生成输出路径；转换到 SRT / VTT 时会简化不支持的样式。
        </div>
        <div class="dropzone-actions">
          <button class="secondary" type="button" @click="pickInputFile">选择字幕</button>
        </div>
      </div>

      <div v-else class="subtitle-format-grid">
        <PathPickerField
          v-model="inputPath"
          label="输入字幕"
          placeholder="选择 ass / ssa / srt / vtt / sub 字幕文件"
          :disabled="running"
          compact
          compact-action="clear"
          @pick="pickInputFile"
          @clear="clearInput"
        />

        <label class="format-field">
          <span>目标格式</span>
          <AppSelect
            v-model="targetFormat"
            :disabled="running"
            title="选择输出字幕格式"
            :options="formatOptions"
          />
        </label>

        <PathPickerField
          v-model="outputPath"
          label="输出字幕"
          placeholder="选择输出字幕路径"
          :disabled="running || !inputPath"
          compact
          compact-action="edit"
          @pick="pickOutputPath"
        />
      </div>

      <p v-if="outputConflictsWithInput" class="form-warning">
        输出路径不能和输入字幕相同，请选择一个新文件。
      </p>

      <div class="tool-note">
        <strong>处理说明</strong>
        <span>ASS / SSA 保留样式能力更强；SRT / VTT 更通用，但只能表达基础文本和时间轴。</span>
        <span>如果源字幕包含复杂定位、特效、字体样式，转换成 SRT / VTT 后这些信息会按目标格式能力被简化。</span>
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
      running-label="转换中..."
      :cancelable="false"
      preview-disabled-tip="选择输入和输出后自动生成命令"
      :run-disabled-tip="runDisabledTip"
      @run="runJob"
    />

    <JobLogPanel
      title="转换进度"
      idle-title="尚未开始转换"
      idle-tip="选择输入、目标格式和输出路径后点击上方「开始转换」。"
      :lines="logs"
      :command="command"
      :percent="progressPercent"
      :running="running"
      :completed="completed"
      :failed="failed"
    />
  </section>
</template>

<style scoped>
.subtitle-format-workspace {
  align-content: start;
  display: grid;
  gap: 12px;
  grid-auto-rows: max-content;
  min-height: 0;
  position: relative;
}

.subtitle-format-panel {
  display: grid;
  gap: 16px;
}

.subtitle-format-panel.is-empty {
  padding: 32px 22px;
}

.subtitle-format-panel.drag-target {
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

.subtitle-format-panel.drag-target .tool-dropzone {
  background: rgba(23, 107, 135, 0.08);
  border-color: #176b87;
  color: #0f5268;
}

.subtitle-format-panel.drag-target .dropzone-title {
  color: #0f5268;
}

.subtitle-format-grid {
  display: grid;
  gap: 14px;
  grid-template-columns: 1fr;
}

.format-field {
  align-items: center;
  display: grid;
  gap: 16px;
  grid-template-columns: 76px 100px;
}

.subtitle-format-grid :deep(.path-picker-field) {
  align-items: center;
  display: grid;
  gap: 16px;
  grid-template-columns: 76px minmax(0, 1fr);
}

.format-field > span,
.subtitle-format-grid :deep(.path-picker-field > span) {
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

.subtitle-format-grid :deep(.path-picker-control) {
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

.tool-note {
  background: #f6fafc;
  border: 1px solid #dbe7ee;
  border-radius: 8px;
  color: #536474;
  display: grid;
  font-size: 13px;
  gap: 8px;
  padding: 12px;
}

.tool-note strong {
  color: #102030;
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


@media (max-width: 920px) {
  .format-field,
  .subtitle-format-grid :deep(.path-picker-field) {
    grid-template-columns: 1fr;
  }

  .format-field > span,
  .subtitle-format-grid :deep(.path-picker-field > span) {
    justify-self: start;
    min-width: 76px;
  }
}
</style>
