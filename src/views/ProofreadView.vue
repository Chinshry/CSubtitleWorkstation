<script setup lang="ts">
import { save } from '@tauri-apps/plugin-dialog'
import { computed, nextTick, onUnmounted, ref, watch } from 'vue'
import { loadConfig, saveConfig } from '../api/config'
import RuleDictionaryModal from '../components/RuleDictionaryModal.vue'
import { useToast } from '../composables/useToast'
import { globalDragActive, pendingDrop, pushDiag } from '../stores/dropStore'
import {
  proofreadText,
  readProofreadFile,
  saveProofreadFile,
  saveProofreadToPath,
  type ProofreadIssue,
  type ProofreadTermRule
} from '../api/proofread'
import type { AppConfig } from '../types'
import { parseRuleDictionary } from '../utils/ruleDictionary'

const sourceText = ref('')
const issues = ref<ProofreadIssue[]>([])
const selectedIssueId = ref('')
const statusText = ref('')
const busy = ref(false)
const pendingFilePath = ref('')
const termDictionary = ref('')
const dictionaryOpen = ref(false)
const appConfig = ref<AppConfig | null>(null)
const proofreadGrid = ref<HTMLDivElement | null>(null)
const sourceTextarea = ref<HTMLTextAreaElement | null>(null)
const sourceLineNumbers = ref<HTMLDivElement | null>(null)
const sourcePanePercent = ref(58)
const resizing = ref(false)
const toast = useToast()
let proofreadTimer: ReturnType<typeof setTimeout> | null = null
let dictionarySaveTimer: ReturnType<typeof setTimeout> | null = null
let proofreadSeq = 0
let termDictionaryLoaded = false
const termDictionaryLoadPromise = loadTermDictionary()

void termDictionaryLoadPromise

const issueCount = computed(() => issues.value.length)
const sourceCount = computed(() => Array.from(sourceText.value).length)
const sourceLines = computed(() => buildLineNumbers(sourceText.value))
const termRules = computed<ProofreadTermRule[]>(() => {
  return parseRuleDictionary(termDictionary.value)
    .map((rule) => ({ canonical: rule.target, pattern: rule.pattern }))
})
const proofreadGridStyle = computed(() => ({
  '--source-pane-percent': `${sourcePanePercent.value}%`,
  '--issue-pane-percent': `${100 - sourcePanePercent.value}%`
}))

type HighlightedContext = {
  before: string
  original: string
  suggestion: string
  after: string
}

type IssueRange = {
  start: number
  end: number
  lineIndex: number
}

function buildLineNumbers(text: string) {
  const count = text ? text.split(/\r\n|\r|\n/).length : 1
  return Array.from({ length: count }, (_, index) => index + 1)
}

async function loadTermDictionary() {
  try {
    const config = await loadConfig()
    appConfig.value = config
    termDictionary.value = config.proofreadTermDictionary ?? ''
  } catch (err) {
    statusText.value = String(err)
  } finally {
    termDictionaryLoaded = true
  }
}

function scheduleSaveTermDictionary() {
  if (!termDictionaryLoaded) return
  if (dictionarySaveTimer) clearTimeout(dictionarySaveTimer)
  dictionarySaveTimer = setTimeout(() => {
    void saveTermDictionary()
  }, 600)
}

async function saveTermDictionary() {
  try {
    const base = appConfig.value ?? await loadConfig()
    if (base.proofreadTermDictionary === termDictionary.value) return
    const next: AppConfig = {
      ...base,
      proofreadTermDictionary: termDictionary.value
    }
    appConfig.value = next
    await saveConfig(next)
  } catch (err) {
    statusText.value = String(err)
  }
}

function getHighlightedContext(issue: ProofreadIssue): HighlightedContext {
  const index = issue.context.indexOf(issue.original)
  if (index < 0) {
    return {
      before: issue.context,
      original: issue.original,
      suggestion: issue.suggestion,
      after: ''
    }
  }
  return {
    before: issue.context.slice(0, index),
    original: issue.original,
    suggestion: issue.suggestion,
    after: issue.context.slice(index + issue.original.length)
  }
}

function getLineRanges(text: string) {
  const ranges: Array<{ start: number; end: number }> = []
  const linePattern = /[^\r\n]*(?:\r\n|\r|\n|$)/g
  let match: RegExpExecArray | null
  while ((match = linePattern.exec(text)) !== null) {
    const value = match[0]
    const start = match.index
    if (!value && start === text.length) break
    const end = start + value.replace(/\r\n|\r|\n$/, '').length
    ranges.push({ start, end })
  }
  return ranges.length ? ranges : [{ start: 0, end: 0 }]
}

function getLineIndexAtPosition(text: string, position: number) {
  const ranges = getLineRanges(text)
  const index = ranges.findIndex((range) => position >= range.start && position <= range.end)
  return index >= 0 ? index : Math.max(0, ranges.length - 1)
}

function findIssueRangeByContext(issue: ProofreadIssue): IssueRange | null {
  const context = issue.context.trim()
  if (!context) return null

  let searchFrom = 0
  let bestRange: IssueRange | null = null
  let bestDistance = Number.POSITIVE_INFINITY
  while (searchFrom <= sourceText.value.length) {
    const contextStart = sourceText.value.indexOf(context, searchFrom)
    if (contextStart < 0) break

    const originalIndex = context.indexOf(issue.original)
    if (originalIndex >= 0) {
      const start = contextStart + originalIndex
      const end = start + issue.original.length
      const distance = Math.abs(start - issue.start)
      if (distance < bestDistance) {
        bestDistance = distance
        bestRange = {
          start,
          end,
          lineIndex: getLineIndexAtPosition(sourceText.value, start)
        }
      }
    }
    searchFrom = contextStart + context.length
  }

  return bestRange
}

function resolveIssueRange(issue: ProofreadIssue): IssueRange {
  const contextRange = findIssueRangeByContext(issue)
  if (contextRange) return contextRange

  const lineIndex = Math.max(0, issue.line - 1)
  const lineRange = getLineRanges(sourceText.value)[lineIndex]
  if (!lineRange) {
    return {
      start: Math.min(sourceText.value.length, issue.start),
      end: Math.min(sourceText.value.length, issue.end),
      lineIndex
    }
  }

  const lineText = sourceText.value.slice(lineRange.start, lineRange.end)
  const directStart = Math.min(sourceText.value.length, issue.start)
  const directEnd = Math.min(sourceText.value.length, issue.end)
  if (
    directStart >= lineRange.start &&
    directEnd <= lineRange.end &&
    sourceText.value.slice(directStart, directEnd) === issue.original
  ) {
    return {
      start: directStart,
      end: directEnd,
      lineIndex
    }
  }

  const originalIndex = lineText.indexOf(issue.original)
  if (originalIndex >= 0) {
    return {
      start: lineRange.start + originalIndex,
      end: lineRange.start + originalIndex + issue.original.length,
      lineIndex
    }
  }

  const issueLength = Math.max(1, issue.end - issue.start)
  const start = Math.min(lineRange.end, Math.max(lineRange.start, directStart))
  return {
    start,
    end: Math.min(lineRange.end, start + issueLength),
    lineIndex
  }
}

function getTextareaSelectionTop(textarea: HTMLTextAreaElement, position: number) {
  const style = window.getComputedStyle(textarea)
  const mirror = document.createElement('div')
  const marker = document.createElement('span')
  const mirroredProperties = [
    'borderTopWidth',
    'borderRightWidth',
    'borderBottomWidth',
    'borderLeftWidth',
    'boxSizing',
    'fontFamily',
    'fontSize',
    'fontStyle',
    'fontVariant',
    'fontWeight',
    'letterSpacing',
    'lineHeight',
    'paddingTop',
    'paddingRight',
    'paddingBottom',
    'paddingLeft',
    'textIndent',
    'textTransform',
    'width',
    'wordSpacing'
  ] as const

  mirroredProperties.forEach((property) => {
    mirror.style[property] = style[property]
  })
  mirror.style.height = 'auto'
  mirror.style.left = '-9999px'
  mirror.style.overflow = 'hidden'
  mirror.style.position = 'absolute'
  mirror.style.top = '0'
  mirror.style.visibility = 'hidden'
  mirror.style.width = `${textarea.clientWidth}px`
  mirror.style.whiteSpace = 'pre-wrap'
  mirror.style.wordBreak = style.wordBreak
  mirror.style.overflowWrap = 'break-word'
  mirror.style.wordWrap = 'break-word'

  mirror.textContent = sourceText.value.slice(0, position)
  marker.textContent = sourceText.value.slice(position, position + 1) || '.'
  mirror.appendChild(marker)
  document.body.appendChild(mirror)
  const top = marker.offsetTop
  mirror.remove()
  return top
}

function scrollTextareaToRange(textarea: HTMLTextAreaElement, range: IssueRange) {
  const targetTop = getTextareaSelectionTop(textarea, range.start)
  textarea.scrollTop = Math.max(0, targetTop - textarea.clientHeight * 0.35)
  syncSourceLineScroll()
}

function clampPanePercent(percent: number, width: number) {
  const leftMin = Math.min(360, width * 0.48)
  const rightMin = Math.min(320, width * 0.42)
  const minPercent = (leftMin / width) * 100
  const maxPercent = ((width - rightMin) / width) * 100
  return Math.min(maxPercent, Math.max(minPercent, percent))
}

function updatePaneSplit(clientX: number) {
  const grid = proofreadGrid.value
  if (!grid) return
  const rect = grid.getBoundingClientRect()
  const nextPercent = ((clientX - rect.left) / rect.width) * 100
  sourcePanePercent.value = clampPanePercent(nextPercent, rect.width)
}

function stopPaneResize() {
  resizing.value = false
  document.body.style.cursor = ''
  document.body.style.userSelect = ''
  window.removeEventListener('pointermove', onPaneResizeMove)
  window.removeEventListener('pointerup', stopPaneResize)
}

function onPaneResizeMove(event: PointerEvent) {
  if (!resizing.value) return
  updatePaneSplit(event.clientX)
}

function startPaneResize(event: PointerEvent) {
  resizing.value = true
  document.body.style.cursor = 'col-resize'
  document.body.style.userSelect = 'none'
  updatePaneSplit(event.clientX)
  window.addEventListener('pointermove', onPaneResizeMove)
  window.addEventListener('pointerup', stopPaneResize)
}

function scheduleProofread() {
  if (proofreadTimer) clearTimeout(proofreadTimer)
  if (!sourceText.value.trim()) {
    issues.value = []
    selectedIssueId.value = ''
    statusText.value = ''
    return
  }
  proofreadTimer = setTimeout(() => {
    void runProofread()
  }, 180)
}

async function runProofread() {
  const seq = ++proofreadSeq
  busy.value = true
  try {
    await termDictionaryLoadPromise
    const nextIssues = await proofreadText(sourceText.value, termRules.value)
    if (seq !== proofreadSeq) return
    issues.value = nextIssues
    selectedIssueId.value = nextIssues[0]?.id ?? ''
    statusText.value = nextIssues.length ? `发现 ${nextIssues.length} 个疑似问题` : '没有发现的地得疑似问题'
  } catch (err) {
    if (seq !== proofreadSeq) return
    statusText.value = String(err)
  } finally {
    if (seq === proofreadSeq) busy.value = false
  }
}

function clearText() {
  sourceText.value = ''
  issues.value = []
  selectedIssueId.value = ''
  statusText.value = ''
  pendingFilePath.value = ''
}

function selectIssue(issue: ProofreadIssue) {
  selectedIssueId.value = issue.id
  void nextTick(() => {
    const textarea = sourceTextarea.value
    if (!textarea) return
    const range = resolveIssueRange(issue)
    textarea.focus()
    textarea.setSelectionRange(range.start, range.end)
    scrollTextareaToRange(textarea, range)
  })
}

function syncSourceLineScroll() {
  if (!sourceTextarea.value || !sourceLineNumbers.value) return
  sourceLineNumbers.value.scrollTop = sourceTextarea.value.scrollTop
}

function buildDefaultExportPath(sourcePath: string, suffix: string, fallbackName: string) {
  if (!sourcePath) return fallbackName
  const separatorIndex = Math.max(sourcePath.lastIndexOf('/'), sourcePath.lastIndexOf('\\'))
  const directory = separatorIndex >= 0 ? sourcePath.slice(0, separatorIndex + 1) : ''
  const fileName = separatorIndex >= 0 ? sourcePath.slice(separatorIndex + 1) : sourcePath
  const dotIndex = fileName.lastIndexOf('.')
  const stem = dotIndex > 0 ? fileName.slice(0, dotIndex) : fileName
  const extension = dotIndex > 0 ? fileName.slice(dotIndex) : ''
  return `${directory}${stem}${suffix}${extension || '.txt'}`
}

function applyIssue(issue: ProofreadIssue) {
  const range = resolveIssueRange(issue)
  sourceText.value = `${sourceText.value.slice(0, range.start)}${issue.suggestion}${sourceText.value.slice(range.end)}`
  statusText.value = `已应用：${issue.original} → ${issue.suggestion}`
  void runProofread()
}

function ignoreIssue(issue: ProofreadIssue) {
  issues.value = issues.value.filter((item) => item.id !== issue.id)
  selectedIssueId.value = issues.value[0]?.id ?? ''
  statusText.value = '已忽略当前提示'
}

async function copyText() {
  if (!sourceText.value) return
  try {
    await navigator.clipboard.writeText(sourceText.value)
    statusText.value = '校对文本已复制'
    toast.success('已复制', 1800)
  } catch (err) {
    statusText.value = String(err)
    toast.error('复制失败', 2200)
  }
}

async function loadDroppedFile(path: string) {
  busy.value = true
  statusText.value = '正在读取文件...'
  try {
    const text = await readProofreadFile(path)
    pendingFilePath.value = path
    sourceText.value = text
    await runProofread()
    pushDiag(`proofread loaded: ${path}`)
  } catch (err) {
    statusText.value = String(err)
    pushDiag(`proofread failed: ${String(err)}`)
  } finally {
    busy.value = false
  }
}

async function exportFile() {
  if (!pendingFilePath.value || busy.value) return
  busy.value = true
  statusText.value = '正在导出校对结果...'
  try {
    const saved = await saveProofreadFile(pendingFilePath.value, sourceText.value, '_校对', false)
    statusText.value = `已输出：${saved.outputPath}`
    pushDiag(`proofread saved: ${saved.outputPath}`)
  } catch (err) {
    const message = String(err)
    if (!message.startsWith('OUTPUT_EXISTS:')) {
      statusText.value = message
      return
    }
    const outputPath = message.slice('OUTPUT_EXISTS:'.length)
    const shouldOverwrite = window.confirm(`输出文件已存在：\n${outputPath}\n\n是否覆盖？`)
    if (!shouldOverwrite) {
      statusText.value = '已取消导出，未覆盖现有文件。'
      return
    }
    const saved = await saveProofreadFile(pendingFilePath.value, sourceText.value, '_校对', true)
    statusText.value = `已覆盖：${saved.outputPath}`
    pushDiag(`proofread overwritten: ${saved.outputPath}`)
  } finally {
    busy.value = false
  }
}

async function exportFileAs() {
  if (!sourceText.value || busy.value) return
  busy.value = true
  statusText.value = '正在导出校对结果...'
  try {
    const outputPath = await save({
      title: '导出校对结果',
      defaultPath: buildDefaultExportPath(pendingFilePath.value, '_校对', 'proofread-text.txt'),
      filters: [
        { name: 'Text and subtitles', extensions: ['txt', 'ass', 'ssa', 'srt', 'vtt', 'sub'] },
        { name: 'All files', extensions: ['*'] }
      ]
    })
    if (!outputPath) return
    const saved = await saveProofreadToPath(outputPath, sourceText.value)
    statusText.value = `已导出：${saved.outputPath}`
    pushDiag(`proofread saved: ${saved.outputPath}`)
  } catch (err) {
    const message = String(err)
    statusText.value = message
    pushDiag(`proofread export failed: ${message}`)
  } finally {
    busy.value = false
  }
}

watch(sourceText, scheduleProofread)

watch(termDictionary, () => {
  scheduleSaveTermDictionary()
  scheduleProofread()
})

watch(pendingDrop, (drop) => {
  if (drop?.target !== 'tools') return
  if (drop.tool !== 'proofread') return
  const path = drop?.textPath
  if (!path) return
  pendingDrop.value = null
  void loadDroppedFile(path)
}, { immediate: true })

onUnmounted(() => {
  if (proofreadTimer) clearTimeout(proofreadTimer)
  if (dictionarySaveTimer) clearTimeout(dictionarySaveTimer)
  stopPaneResize()
})
</script>

<template>
  <section class="proofread-workspace">
    <div v-if="globalDragActive" class="drop-overlay">松开以校对文本或字幕文件</div>

    <section class="panel proofread-panel">
      <div class="panel-heading proofread-heading">
        <div>
          <h2>字幕校对</h2>
          <p>使用jieba-rs分词与词性标注，检查“的 / 地 / 得”疑似误用；检查自定义词库专有名词写法。</p>
        </div>
        <button type="button" class="dictionary-button" @click="dictionaryOpen = true">自定义词库</button>
      </div>

      <div
        ref="proofreadGrid"
        class="proofread-grid"
        :class="{ resizing }"
        :style="proofreadGridStyle"
      >
        <div class="proofread-source">
          <span class="field-head">
            <strong>文本</strong>
            <span class="field-tools">
              <small>{{ sourceCount }} 字</small>
              <button type="button" class="field-tool" :disabled="!sourceText" @click="copyText">复制</button>
              <button type="button" class="field-tool" :disabled="!sourceText" @click="clearText">清空</button>
            </span>
          </span>
          <div class="text-editor-shell">
            <div ref="sourceLineNumbers" class="line-numbers" aria-hidden="true">
              <span v-for="line in sourceLines" :key="line">{{ line }}</span>
            </div>
            <textarea
              ref="sourceTextarea"
              v-model="sourceText"
              spellcheck="false"
              placeholder="粘贴要处理的文本&#10;可拖入 TXT / ASS / SSA / SRT / VTT / SUB 文件"
              @scroll="syncSourceLineScroll"
            ></textarea>
          </div>
        </div>

        <button
          type="button"
          class="pane-resizer"
          role="separator"
          aria-orientation="vertical"
          aria-label="调整文本和问题列表宽度"
          @pointerdown.prevent="startPaneResize"
        ></button>

        <div class="issue-list">
          <div class="issue-list-head">
            <div>
              <strong>疑似问题</strong>
            </div>
            <div class="issue-toolbar">
              <small>{{ busy ? '检查中...' : `${issueCount} 项` }}</small>
              <button type="button" :disabled="!sourceText || busy" @click="exportFileAs">导出</button>
            </div>
          </div>
          <div v-if="issues.length" class="issue-items">
            <div
              v-for="issue in issues"
              :key="issue.id"
              role="button"
              tabindex="0"
              class="issue-item"
              :class="{ active: selectedIssueId === issue.id }"
              @click="selectIssue(issue)"
              @keydown.enter.prevent="selectIssue(issue)"
              @keydown.space.prevent="selectIssue(issue)"
            >
              <span class="issue-copy">
                <span class="issue-sentence">
                  <span>{{ getHighlightedContext(issue).before }}</span>
                  <mark class="original">{{ getHighlightedContext(issue).original }}</mark>
                  <span>{{ getHighlightedContext(issue).after }}</span>
                </span>
                <span class="issue-sentence">
                  <span>{{ getHighlightedContext(issue).before }}</span>
                  <mark class="suggestion">{{ getHighlightedContext(issue).suggestion }}</mark>
                  <span>{{ getHighlightedContext(issue).after }}</span>
                </span>
                <small>{{ issue.reason }}</small>
              </span>
              <span class="issue-actions">
                <button type="button" @click.stop="applyIssue(issue)">采纳</button>
                <button type="button" class="secondary" @click.stop="ignoreIssue(issue)">忽略</button>
              </span>
            </div>
          </div>
          <div v-else class="empty-issues">
            {{ sourceText ? '暂未发现疑似问题' : '校对结果会显示在这里' }}
          </div>
        </div>
      </div>

      <RuleDictionaryModal
        v-model:open="dictionaryOpen"
        v-model="termDictionary"
        title="自定义词库"
        description="维护专有名词、艺人名和固定译名，校对时会按匹配规则提示统一写法。"
        target-label="标准写法"
        pattern-label="匹配规则"
        target-placeholder="例如 ZEROBASEONE"
        pattern-placeholder="例如 (?i)ZE[EROBASN]{7,12}"
        raw-placeholder="[&quot;ZEROBASEONE&quot;] = &quot;(?i)ZE[EROBASN]{7,12}&quot;"
        ariaLabel="字幕校对自定义词库"
      />

    </section>
  </section>
</template>

<style scoped>
.proofread-workspace {
  display: grid;
  min-height: 0;
  position: relative;
}

.proofread-panel {
  display: grid;
  gap: 10px;
  grid-template-rows: auto minmax(0, 1fr);
  min-height: 0;
}

.proofread-heading {
  align-items: center;
  margin-bottom: 0;
}

.dictionary-button {
  background: #e5eaee;
  color: #24313c;
  min-height: 32px;
  padding: 0 12px;
}

.proofread-grid {
  align-items: stretch;
  display: grid;
  grid-template-columns:
    minmax(280px, calc(var(--source-pane-percent, 58%) - 6px))
    12px
    minmax(300px, calc(var(--issue-pane-percent, 42%) - 6px));
  min-height: 0;
}

.pane-resizer {
  align-self: stretch;
  background: transparent;
  border: 0;
  border-radius: 6px;
  cursor: col-resize;
  margin: 28px 0 0;
  min-height: 320px;
  min-width: 12px;
  padding: 0;
  position: relative;
  touch-action: none;
}

.pane-resizer::before {
  background: #cbd8e0;
  border-radius: 999px;
  content: "";
  inset: 0 auto 0 5px;
  opacity: 0.72;
  position: absolute;
  transition: background 0.16s ease, opacity 0.16s ease, width 0.16s ease;
  width: 2px;
}

.pane-resizer:hover::before,
.pane-resizer:focus-visible::before,
.proofread-grid.resizing .pane-resizer::before {
  background: #176b87;
  opacity: 1;
  width: 3px;
}

.pane-resizer:focus-visible {
  outline: 2px solid rgba(23, 107, 135, 0.36);
  outline-offset: 2px;
}

.proofread-source,
.issue-list {
  display: grid;
  gap: 8px;
  grid-template-rows: auto auto;
  min-width: 0;
  min-height: 0;
}

.field-head,
.issue-list-head {
  align-items: center;
  color: #24313c;
  display: flex;
  justify-content: space-between;
  gap: 10px;
}

.field-tools {
  align-items: center;
  display: flex;
  gap: 8px;
}

.issue-list-head > div:first-child {
  display: grid;
  gap: 2px;
  min-width: 0;
}

.field-head strong,
.issue-list-head strong {
  font-size: 13px;
  font-weight: 700;
}

.field-tool {
  background: #e5eaee;
  color: #24313c;
  font-size: 13px;
  min-height: 28px;
  padding: 0 10px;
}

.field-tool.primary {
  background: #176b87;
  color: #fff;
}

.text-editor-shell {
  background: #fbfcfd;
  border: 1px solid #d8e2e8;
  border-radius: 8px;
  display: grid;
  grid-template-columns: 44px minmax(0, 1fr);
  height: max(360px, calc(100vh - 252px));
  min-height: 320px;
  overflow: hidden;
}

.line-numbers {
  background: #f2f5f7;
  border-right: 1px solid #d8e2e8;
  color: #8a98a3;
  font: 12px/1.7 "Microsoft YaHei", "Segoe UI", sans-serif;
  overflow: hidden;
  padding: 12px 8px;
  text-align: right;
  user-select: none;
}

.line-numbers span {
  display: block;
  height: 23.8px;
}

.proofread-source textarea {
  background: transparent;
  border: 0;
  color: #18202a;
  font: 14px/1.7 "Microsoft YaHei", "Segoe UI", sans-serif;
  height: 100%;
  min-height: 0;
  outline: none;
  padding: 12px;
  resize: none;
  width: 100%;
}

.proofread-source textarea:focus {
  box-shadow: none;
}

.text-editor-shell:focus-within {
  border-color: #176b87;
  box-shadow: 0 0 0 3px rgba(23, 107, 135, 0.12);
}

.proofread-source textarea::selection {
  background: #fff0a8;
  color: #18202a;
}

.field-tools small,
.issue-toolbar small {
  color: #667582;
  font-size: 12px;
  text-align: right;
}

.issue-toolbar {
  align-items: center;
  display: flex;
  flex: 0 0 auto;
  gap: 8px;
}

.issue-toolbar button {
  background: #e5eaee;
  color: #24313c;
  font-size: 13px;
  min-height: 28px;
  padding: 0 10px;
}

.issue-toolbar button:disabled {
  background: #dce4e9;
  color: #7a8790;
}

.issue-items,
.empty-issues {
  background: #f6f8fa;
  border: 1px solid #d8e2e8;
  border-radius: 8px;
  height: max(360px, calc(100vh - 252px));
  min-height: 320px;
  overflow: auto;
  padding: 8px;
}

.issue-item {
  align-items: start;
  background: #fff;
  border: 1px solid #e2e8ee;
  border-radius: 6px;
  color: #24313c;
  cursor: pointer;
  display: flex;
  gap: 14px;
  justify-content: space-between;
  min-height: 92px;
  padding: 12px;
  text-align: left;
  width: 100%;
}

.issue-item + .issue-item {
  margin-top: 8px;
}

.issue-item.active {
  background: #fffdf1;
  border-color: #d9b94c;
  box-shadow: inset 3px 0 0 #d9b94c;
}

.issue-item:focus-visible {
  outline: 2px solid #176b87;
  outline-offset: 2px;
}

.issue-copy {
  display: grid;
  gap: 5px;
  min-width: 0;
}

.issue-sentence {
  color: #102030;
  display: flex;
  flex-wrap: wrap;
  font-size: 14px;
  gap: 2px;
  line-height: 1.6;
}

.issue-sentence mark {
  background: #ffe2df;
  border-radius: 3px;
  color: #b42318;
  font-weight: 700;
  padding: 0 2px;
  text-decoration: line-through;
  text-decoration-thickness: 1.5px;
}

.issue-sentence mark.suggestion {
  background: #dff6e8;
  color: #177245;
  text-decoration: none;
}

.issue-copy small {
  border-top: 1px solid #edf1f4;
  color: #667582;
  font-size: 11px;
  line-height: 1.3;
  padding-top: 6px;
}

.empty-issues {
  align-items: center;
  color: #8996a1;
  display: flex;
  justify-content: center;
}

.issue-actions,
.issue-toolbar {
  display: flex;
  flex: 0 0 auto;
  gap: 8px;
}

.issue-actions button {
  background: #176b87;
  color: #fff;
  min-height: 34px;
  padding: 0 12px;
}

.issue-actions {
  align-self: center;
  flex-direction: column;
}

.issue-actions button.secondary {
  background: #e5eaee;
  color: #24313c;
}

@media (max-width: 960px) {
  .proofread-grid {
    grid-template-columns: minmax(0, 1fr);
  }

  .pane-resizer {
    display: none;
  }

  .proofread-heading,
  .issue-list-head {
    align-items: stretch;
    flex-direction: column;
  }

  .issue-item {
    flex-direction: column;
  }

  .issue-actions {
    align-self: stretch;
    flex-direction: row;
  }
}
</style>
