<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { open, save } from '@tauri-apps/plugin-dialog'
import {
  convertSubtitleFormat,
  previewSubtitleFormatCommand,
  type SubtitleTargetFormat
} from '../api/subtitleTool'
import { globalDragActive, pendingDrop } from '../stores/dropStore'
import { ffmpegChecking, ffmpegStatus, initFfmpegStatus, refreshFfmpegStatus } from '../stores/ffmpegStore'
import AppSelect from '../components/AppSelect.vue'
import CommandPreviewCard from '../components/CommandPreviewCard.vue'
import CommandTaskActions from '../components/CommandTaskActions.vue'
import PathPickerField from '../components/PathPickerField.vue'

const inputPath = ref('')
const outputPath = ref('')
const targetFormat = ref<SubtitleTargetFormat>('srt')
const command = ref<string[]>([])
const logs = ref<string[]>([])
const running = ref(false)
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
  if (!ffmpegStatus.value?.available) return '请先在设置页配置可用的 ffmpeg'
  if (!inputPath.value.trim()) return '请选择输入字幕'
  if (!outputPath.value.trim()) return '请选择输出字幕路径'
  if (outputConflictsWithInput.value) return '输出路径不能和输入字幕相同'
  return '可以开始转换'
})

const canRun = computed(() => runDisabledTip.value === '可以开始转换')
const statusText = computed(() => {
  if (running.value) return '转换中…'
  if (!logs.value.length) return '待开始'
  return logs.value.some((line) => line.includes('❌')) ? '已失败' : '已完成'
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
  logs.value = []
  try {
    await previewCommand()
    const result = await convertSubtitleFormat(createJob())
    logs.value = result.logs.length ? result.logs : [`已输出：${result.outputPath}`]
  } catch (error) {
    logs.value = formatError(error).split('\n').filter(Boolean)
    if (!logs.value.some((line) => line.includes('❌'))) {
      logs.value.push('❌ 字幕格式转换失败')
    }
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
})

onMounted(() => {
  void initFfmpegStatus()
  if (pendingDrop.value?.target === 'tools' && pendingDrop.value.tool === 'subtitle-format') {
    const drop = pendingDrop.value
    applyDroppedPaths(drop.raw, drop.subtitlePath)
    pendingDrop.value = null
  }
})

onUnmounted(() => {
  if (previewTimer) clearTimeout(previewTimer)
})
</script>

<template>
  <section class="subtitle-format-workspace">
    <div v-if="globalDragActive" class="drop-overlay">松开以读取字幕文件</div>

    <div v-if="ffmpegChecking" class="ffmpeg-missing ffmpeg-checking">
      <strong>正在检测 ffmpeg 环境</strong>
      <span>正在检测 ffmpeg / ffprobe，请稍候。</span>
    </div>
    <div v-else-if="ffmpegStatus && !ffmpegStatus.available" class="ffmpeg-missing">
      <strong>{{ ffmpegStatus.ffmpegPath ? 'ffmpeg 功能不完整' : '未检测到 ffmpeg' }}</strong>
      <span>{{ ffmpegStatus.message ?? '请前往左侧「设置」面板配置 ffmpeg 路径，或安装后将其加入系统 PATH。' }}</span>
      <button class="secondary" @click="refreshFfmpeg">重新检测</button>
    </div>

    <section class="panel subtitle-format-panel">
      <div class="subtitle-format-heading">
        <div>
          <h2>字幕格式转换</h2>
          <p>使用 ffmpeg 在 ASS / SSA / SRT / VTT 之间转换；转到 SRT / VTT 时会丢弃原格式不支持的样式和特效。</p>
        </div>
      </div>

      <div class="subtitle-format-grid">
        <PathPickerField
          v-model="inputPath"
          label="输入字幕"
          placeholder="选择 ass / ssa / srt / vtt / sub 字幕文件"
          :disabled="running"
          @pick="pickInputFile"
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
      running-label="转换中…"
      :cancelable="false"
      preview-disabled-tip="选择输入和输出后自动生成命令"
      :run-disabled-tip="runDisabledTip"
      @run="runJob"
    />

    <section class="panel subtitle-format-log">
      <div class="panel-heading">
        <div class="heading-title">
          <h2>转换结果</h2>
          <span class="status-badge">{{ statusText }}</span>
        </div>
      </div>
      <div v-if="logs.length" class="log-lines">
        <p v-for="(line, index) in logs" :key="index">{{ line }}</p>
      </div>
      <div v-else class="idle-result">
        <p>尚未开始转换</p>
        <span>选择输入、目标格式和输出路径后点击上方「开始转换」。</span>
      </div>
    </section>
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

.subtitle-format-heading h2 {
  color: #102030;
  font-size: 18px;
  margin: 0;
}

.subtitle-format-heading p {
  color: #667582;
  font-size: 13px;
  margin: 6px 0 0;
}

.subtitle-format-grid {
  display: grid;
  gap: 12px;
  grid-template-columns: minmax(0, 1fr) 160px minmax(0, 1fr);
}

.format-field {
  display: grid;
  gap: 6px;
}

.format-field > span {
  color: #4d5b66;
  font-size: 13px;
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

.subtitle-format-log {
  display: grid;
  gap: 12px;
  min-height: 180px;
}

.log-lines {
  background: #0f1720;
  border-radius: 8px;
  color: #d7e7ef;
  display: grid;
  font-family: "Cascadia Mono", "JetBrains Mono", Consolas, monospace;
  font-size: 12px;
  gap: 4px;
  max-height: 260px;
  overflow: auto;
  padding: 12px;
}

.log-lines p {
  margin: 0;
  white-space: pre-wrap;
  word-break: break-all;
}

.idle-result {
  align-items: center;
  color: #8794a0;
  display: grid;
  justify-items: center;
  min-height: 120px;
}

.idle-result p {
  color: #102030;
  font-weight: 750;
  margin: 0;
}

.idle-result span {
  font-size: 13px;
}

@media (max-width: 920px) {
  .subtitle-format-grid {
    grid-template-columns: 1fr;
  }
}
</style>
