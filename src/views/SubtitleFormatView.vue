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
import { useI18n } from '../i18n'

const inputPath = ref('')
const outputPath = ref('')
const targetFormat = ref<SubtitleTargetFormat>('ass')
const command = ref<string[]>([])
const logs = ref<string[]>([])
const running = ref(false)
const jobFailed = ref(false)
const showCommandPreview = ref(false)
let previewTimer: ReturnType<typeof setTimeout> | null = null
const { t } = useI18n()

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
  if (ffmpegChecking.value) return t('subtitleFormat.disabled.checking')
  if (ffmpegStatus.value && !ffmpegStatus.value.available) return t('subtitleFormat.disabled.ffmpegUnavailable')
  if (!inputPath.value.trim()) return t('subtitleFormat.disabled.input')
  if (!outputPath.value.trim()) return t('subtitleFormat.disabled.output')
  if (outputConflictsWithInput.value) return t('subtitleFormat.disabled.conflict')
  return t('subtitleFormat.ready')
})

const canRun = computed(() => runDisabledTip.value === t('subtitleFormat.ready'))
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
  const file = t('subtitleFormat.outputFile', { stem: parts.stem, format: targetFormat.value })
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
    title: t('subtitleFormat.dialog.inputTitle'),
    multiple: false,
    filters: [{ name: t('subtitleFormat.dialog.subtitleFilter'), extensions: ['ass', 'ssa', 'srt', 'vtt', 'sub'] }]
  })
  if (typeof selected === 'string') {
    inputPath.value = selected
    applyAutoOutput()
  }
}

async function pickOutputPath() {
  if (running.value) return
  const selected = await save({
    title: t('subtitleFormat.dialog.outputTitle'),
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
    logs.value = result.logs.length ? result.logs : [t('subtitleFormat.outputLog', { path: result.outputPath })]
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
      <strong>{{ t('ffmpegPanel.checkingTitle') }}</strong>
      <span>{{ t('ffmpegPanel.checkingSubtitle') }}</span>
    </div>
    <div v-else-if="ffmpegStatus && !ffmpegStatus.available" class="ffmpeg-missing">
      <strong>{{ ffmpegStatus.ffmpegPath ? t('ffmpegPanel.incomplete') : t('ffmpegPanel.missing') }}</strong>
      <span>{{ ffmpegStatus.message ?? t('ffmpegPanel.missingHelp') }}</span>
      <button class="secondary" @click="refreshFfmpeg">{{ t('ffmpegPanel.refresh') }}</button>
    </div>

    <section class="panel subtitle-format-panel" :class="{ 'is-empty': !inputPath, 'drag-target': globalDragActive }">
      <div v-if="!inputPath" class="tool-dropzone">
        <div class="dropzone-icon">⬇︎</div>
        <div class="dropzone-title">{{ t('subtitleFormat.dropzone.title') }}</div>
        <div class="dropzone-sub">
          <span class="dropzone-note">{{ t('subtitleFormat.dropzone.note') }}</span>
          <br />
          {{ t('subtitleFormat.dropzone.description') }}
        </div>
        <div class="dropzone-actions">
          <button class="secondary" type="button" @click="pickInputFile">{{ t('subtitleFormat.dropzone.choose') }}</button>
        </div>
      </div>

      <div v-else class="subtitle-format-grid">
        <PathPickerField
          v-model="inputPath"
          :label="t('subtitleFormat.inputLabel')"
          :placeholder="t('subtitleFormat.inputPlaceholder')"
          :disabled="running"
          compact
          compact-action="clear"
          @pick="pickInputFile"
          @clear="clearInput"
        />

        <label class="format-field">
          <span>{{ t('subtitleFormat.targetFormat') }}</span>
          <AppSelect
            v-model="targetFormat"
            :disabled="running"
            :title="t('subtitleFormat.targetFormatTitle')"
            :options="formatOptions"
          />
        </label>

        <PathPickerField
          v-model="outputPath"
          :label="t('subtitleFormat.outputLabel')"
          :placeholder="t('subtitleFormat.outputPlaceholder')"
          :disabled="running || !inputPath"
          compact
          compact-action="edit"
          @pick="pickOutputPath"
        />
      </div>

      <p v-if="outputConflictsWithInput" class="form-warning">
        {{ t('subtitleFormat.conflictWarning') }}
      </p>

      <div class="tool-note">
        <strong>{{ t('subtitleFormat.noteTitle') }}</strong>
        <span>{{ t('subtitleFormat.noteAss') }}</span>
        <span>{{ t('subtitleFormat.noteSimplify') }}</span>
      </div>
    </section>

    <CommandPreviewCard v-if="command.length && showCommandPreview" :command="command" />

    <CommandTaskActions
      v-model:preview-open="showCommandPreview"
      :command="command"
      :running="running"
      :can-run="canRun"
      :start-label="t('subtitleFormat.start')"
      :cancel-label="t('subtitleFormat.cancel')"
      :running-label="t('subtitleFormat.running')"
      :cancelable="false"
      :preview-disabled-tip="t('common.commandPreviewDisabledTip')"
      :run-disabled-tip="runDisabledTip"
      @run="runJob"
    />

    <JobLogPanel
      :title="t('subtitleFormat.progressTitle')"
      :idle-title="t('subtitleFormat.idleTitle')"
      :idle-tip="t('subtitleFormat.idleTip')"
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
