<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { loadConfig, saveConfig } from '../api/config'
import { globalDragActive, pendingDrop, pushDiag } from '../stores/dropStore'
import type { AppConfig } from '../types'
import {
  convertChineseText,
  readPlainTextFile,
  saveConvertedTextFile,
  type ChineseConversionMode,
  type CustomConversionRule
} from '../api/textConversion'

type DiffSegment = {
  text: string
  changed: boolean
}

const mode = ref<ChineseConversionMode>('t2s')
const sourceText = ref('')
const resultText = ref('')
const customDictionary = ref('')
const fileStatus = ref('')
const fileBusy = ref(false)
const textBusy = ref(false)
const appConfig = ref<AppConfig | null>(null)
const sourceTextarea = ref<HTMLTextAreaElement | null>(null)
const resultPreview = ref<HTMLDivElement | null>(null)
const conversionTextHeight = ref(320)
const pendingTextFilePath = ref('')

let convertTimer: ReturnType<typeof setTimeout> | null = null
let dictionarySaveTimer: ReturnType<typeof setTimeout> | null = null
let sourceResizeObserver: ResizeObserver | null = null
let convertSeq = 0
let scrollSyncing = false
let customDictionaryLoaded = false
const customDictionaryLoadPromise = loadCustomDictionary()

void customDictionaryLoadPromise

const modeLabel = computed(() => (mode.value === 's2t' ? '简体转繁体' : '繁体转简体'))
const outputSuffix = computed(() => (mode.value === 's2t' ? '_繁体' : '_简体'))
const sourceCount = computed(() => Array.from(sourceText.value).length)
const resultCount = computed(() => Array.from(resultText.value).length)
const customRules = computed(() => parseCustomDictionary(customDictionary.value))
const resultDiffSegments = computed(() => diffResult(sourceText.value, resultText.value))

function parseCustomDictionary(text: string): CustomConversionRule[] {
  return text
    .replace(/\r\n/g, '\n')
    .split('\n')
    .map((line) => line.trim())
    .filter((line) => line && !line.startsWith('#'))
    .map(parseDictionaryLine)
    .filter((rule): rule is CustomConversionRule => Boolean(rule))
}

function parseDictionaryLine(line: string): CustomConversionRule | null {
  const separators = ['->', '=>', '=', '\t']
  for (const separator of separators) {
    const index = line.indexOf(separator)
    if (index <= 0) continue
    const from = line.slice(0, index).trim()
    const to = line.slice(index + separator.length).trim()
    if (from && to) return { from, to }
  }
  return null
}

function scheduleConvert() {
  if (convertTimer) clearTimeout(convertTimer)
  if (!sourceText.value) {
    resultText.value = ''
    fileStatus.value = ''
    textBusy.value = false
    return
  }
  textBusy.value = true
  convertTimer = setTimeout(() => {
    void convertCurrentText()
  }, 220)
}

async function convertCurrentText() {
  const seq = ++convertSeq
  const text = sourceText.value
  const rules = customRules.value
  try {
    const converted = await convertChineseText(text, mode.value, rules)
    if (seq !== convertSeq) return
    resultText.value = converted
    fileStatus.value = `已完成${modeLabel.value}`
  } catch (err) {
    if (seq !== convertSeq) return
    fileStatus.value = String(err)
  } finally {
    if (seq === convertSeq) textBusy.value = false
  }
}

async function convertTextNow(text: string) {
  return convertChineseText(text, mode.value, customRules.value)
}

async function loadCustomDictionary() {
  try {
    const config = await loadConfig()
    appConfig.value = config
    customDictionary.value = config.textConversionCustomDictionary ?? ''
  } catch (err) {
    fileStatus.value = String(err)
  } finally {
    customDictionaryLoaded = true
  }
}

function scheduleSaveCustomDictionary() {
  if (!customDictionaryLoaded) return
  if (dictionarySaveTimer) clearTimeout(dictionarySaveTimer)
  dictionarySaveTimer = setTimeout(() => {
    void saveCustomDictionary()
  }, 600)
}

async function saveCustomDictionary() {
  try {
    const base = appConfig.value ?? await loadConfig()
    if (base.textConversionCustomDictionary === customDictionary.value) return
    const next: AppConfig = {
      ...base,
      textConversionCustomDictionary: customDictionary.value
    }
    appConfig.value = next
    await saveConfig(next)
  } catch (err) {
    fileStatus.value = String(err)
  }
}

function setMode(nextMode: ChineseConversionMode) {
  if (mode.value === nextMode) return
  mode.value = nextMode
}

async function copyResult() {
  if (!resultText.value) return
  await navigator.clipboard.writeText(resultText.value)
  fileStatus.value = '转换结果已复制'
}

function clearText() {
  sourceText.value = ''
  resultText.value = ''
  fileStatus.value = ''
  pendingTextFilePath.value = ''
}

function syncSourceTextareaHeight() {
  if (!sourceTextarea.value) return
  conversionTextHeight.value = Math.round(sourceTextarea.value.getBoundingClientRect().height)
}

function syncScroll(source: HTMLElement | null, target: HTMLElement | null) {
  if (!source || !target || scrollSyncing) return
  const sourceMax = source.scrollHeight - source.clientHeight
  const targetMax = target.scrollHeight - target.clientHeight
  if (sourceMax <= 0 || targetMax <= 0) return

  scrollSyncing = true
  target.scrollTop = (source.scrollTop / sourceMax) * targetMax
  requestAnimationFrame(() => {
    scrollSyncing = false
  })
}

function syncResultScrollToSource() {
  syncScroll(sourceTextarea.value, resultPreview.value)
}

function syncSourceScrollToResult() {
  syncScroll(resultPreview.value, sourceTextarea.value)
}

async function convertDroppedTextFile(path: string) {
  fileBusy.value = true
  fileStatus.value = '正在读取 TXT 文件...'
  try {
    await customDictionaryLoadPromise
    const text = await readPlainTextFile(path)
    const converted = await convertTextNow(text)
    pendingTextFilePath.value = path
    sourceText.value = text
    resultText.value = converted
    fileStatus.value = '已读取 TXT 文件。可先补充自定义词库，确认结果后再导出。'
    pushDiag(`text conversion loaded: ${path}`)
  } catch (err) {
    fileStatus.value = String(err)
    pushDiag(`text conversion failed: ${String(err)}`)
  } finally {
    fileBusy.value = false
  }
}

async function exportPendingTextFile() {
  if (!pendingTextFilePath.value || fileBusy.value) return
  fileBusy.value = true
  fileStatus.value = '正在导出转换结果...'
  try {
    const text = textBusy.value ? await convertTextNow(sourceText.value) : resultText.value
    const saved = await saveConvertedTextFile(
      pendingTextFilePath.value,
      text,
      outputSuffix.value,
      false
    )
    resultText.value = text
    fileStatus.value = `已输出：${saved.outputPath}`
    pushDiag(`text conversion saved: ${saved.outputPath}`)
  } catch (err) {
    const message = String(err)
    if (!message.startsWith('OUTPUT_EXISTS:')) {
      fileStatus.value = message
      pushDiag(`text conversion export failed: ${message}`)
      return
    }

    const outputPath = message.slice('OUTPUT_EXISTS:'.length)
    const shouldOverwrite = window.confirm(`输出文件已存在：\n${outputPath}\n\n是否覆盖？`)
    if (!shouldOverwrite) {
      fileStatus.value = '已取消导出，未覆盖现有文件。'
      return
    }

    const text = textBusy.value ? await convertTextNow(sourceText.value) : resultText.value
    const saved = await saveConvertedTextFile(
      pendingTextFilePath.value,
      text,
      outputSuffix.value,
      true
    )
    resultText.value = text
    fileStatus.value = `已覆盖：${saved.outputPath}`
    pushDiag(`text conversion overwritten: ${saved.outputPath}`)
  } finally {
    fileBusy.value = false
  }
}

function diffResult(source: string, result: string): DiffSegment[] {
  if (!result) return []
  const sourceChars = Array.from(source)
  const resultChars = Array.from(result)
  const sourceLength = sourceChars.length
  const resultLength = resultChars.length
  const table = Array.from({ length: sourceLength + 1 }, () => Array(resultLength + 1).fill(0) as number[])

  for (let i = sourceLength - 1; i >= 0; i--) {
    for (let j = resultLength - 1; j >= 0; j--) {
      table[i][j] = sourceChars[i] === resultChars[j]
        ? table[i + 1][j + 1] + 1
        : Math.max(table[i + 1][j], table[i][j + 1])
    }
  }

  const segments: DiffSegment[] = []
  let i = 0
  let j = 0
  while (j < resultLength) {
    let changed = true
    if (i < sourceLength && sourceChars[i] === resultChars[j]) {
      changed = false
      i++
      j++
    } else if (i < sourceLength && table[i + 1][j] >= table[i][j + 1]) {
      i++
      continue
    } else {
      j++
    }

    const text = resultChars[j - 1]
    const last = segments[segments.length - 1]
    if (last && last.changed === changed) {
      last.text += text
    } else {
      segments.push({ text, changed })
    }
  }

  return segments
}

watch([sourceText, mode, customDictionary], scheduleConvert)

watch(customDictionary, scheduleSaveCustomDictionary)

watch(resultText, () => {
  void nextTick(syncResultScrollToSource)
})

watch(pendingDrop, (drop) => {
  const path = drop?.textPath
  if (!path) return
  void convertDroppedTextFile(path)
}, { immediate: true })

onMounted(() => {
  syncSourceTextareaHeight()
  if (!sourceTextarea.value || typeof ResizeObserver === 'undefined') return
  sourceResizeObserver = new ResizeObserver(syncSourceTextareaHeight)
  sourceResizeObserver.observe(sourceTextarea.value)
})

onUnmounted(() => {
  if (convertTimer) clearTimeout(convertTimer)
  if (dictionarySaveTimer) clearTimeout(dictionarySaveTimer)
  sourceResizeObserver?.disconnect()
})
</script>

<template>
  <section class="text-conversion-workspace">
    <div v-if="globalDragActive" class="drop-overlay">松开以转换 TXT 文件</div>

    <section class="panel text-conversion-panel">
      <div class="panel-heading text-conversion-heading">
        <div>
          <h2>繁简字转换</h2>
          <p>粘贴文本后自动转换，或拖入 TXT 文件后在同目录输出结果。</p>
        </div>
        <div class="conversion-mode" role="group" aria-label="转换方向">
          <button :class="{ active: mode === 't2s' }" @click="setMode('t2s')">繁体 → 简体</button>
          <button :class="{ active: mode === 's2t' }" @click="setMode('s2t')">简体 → 繁体</button>
        </div>
      </div>

      <div
        class="conversion-grid"
        :style="{ '--conversion-text-height': `${conversionTextHeight}px` }"
      >
        <label class="conversion-field">
          <span class="field-head">
            <strong>输入</strong>
            <button class="field-tool" type="button" :disabled="!sourceText" @click="clearText">清空</button>
          </span>
          <textarea
            ref="sourceTextarea"
            v-model="sourceText"
            spellcheck="false"
            @scroll="syncResultScrollToSource"
            placeholder="在这里粘贴要转换的文本"
          ></textarea>
          <small>{{ sourceCount }} 字</small>
        </label>

        <div class="conversion-field">
          <span class="field-head">
            <strong>结果</strong>
            <button class="field-tool" type="button" :disabled="!resultText" @click="copyResult">复制</button>
          </span>
          <div
            ref="resultPreview"
            class="result-preview"
            :class="{ empty: !resultText }"
            aria-live="polite"
            @scroll="syncSourceScrollToResult"
          >
            <template v-if="resultText">
              <span
                v-for="(segment, index) in resultDiffSegments"
                :key="index"
                :class="{ changed: segment.changed }"
              >{{ segment.text }}</span>
            </template>
            <span v-else class="placeholder">{{ textBusy ? '正在转换...' : '转换结果会显示在这里' }}</span>
          </div>
          <small>{{ resultCount }} 字</small>
        </div>
      </div>

      <div class="custom-dictionary">
        <div class="custom-dictionary-head">
          <div>
            <strong>自定义词库</strong>
            <span>每行一条：原词 = 目标词，也支持 原词 -&gt; 目标词。</span>
          </div>
          <small>{{ customRules.length }} 条规则</small>
        </div>
        <textarea
          v-model="customDictionary"
          spellcheck="false"
          placeholder="俐落 = 利落&#10;急遽 = 急剧"
        ></textarea>
      </div>

      <div class="text-drop-box" :class="{ active: globalDragActive, busy: fileBusy || textBusy }">
        <div class="text-drop-icon" aria-hidden="true">
          <svg viewBox="0 0 24 24" width="28" height="28" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
            <path d="M14 2H7a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V7z" />
            <path d="M14 2v5h5" />
            <path d="M9 13h6" />
            <path d="M9 17h4" />
          </svg>
        </div>
        <div>
          <strong>{{ fileBusy ? '正在处理文件' : textBusy ? '正在转换文本' : '拖入 TXT 文件' }}</strong>
          <span>{{ fileStatus || '拖入 TXT 后先预览转换结果；补充自定义词库后再导出到原文件同目录。' }}</span>
        </div>
        <button
          class="export-file-button"
          type="button"
          :disabled="!pendingTextFilePath || fileBusy || !resultText"
          @click="exportPendingTextFile"
        >导出结果</button>
      </div>
    </section>
  </section>
</template>

<style scoped>
.text-conversion-workspace {
  grid-auto-rows: minmax(0, 1fr);
  position: relative;
}

.text-conversion-panel {
  display: grid;
  gap: 16px;
  min-height: calc(100vh - 158px);
}

.text-conversion-heading {
  align-items: center;
}

.conversion-mode {
  align-items: center;
  background: #edf1f4;
  border: 1px solid #d6e0e7;
  border-radius: 8px;
  display: flex;
  gap: 3px;
  padding: 3px;
}

.conversion-mode button {
  background: transparent;
  color: #43515c;
  min-height: 32px;
  padding: 0 12px;
}

.conversion-mode button.active {
  background: #176b87;
  color: #fff;
}

.conversion-grid {
  --conversion-text-height: 320px;
  align-items: start;
  display: grid;
  gap: 16px;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
}

.conversion-field {
  display: grid;
  gap: 8px;
  grid-template-rows: auto auto auto;
  min-width: 0;
}

.field-head,
.custom-dictionary-head {
  align-items: center;
  color: #24313c;
  display: flex;
  justify-content: space-between;
  gap: 10px;
}

.field-head strong,
.custom-dictionary-head strong {
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

.conversion-field textarea,
.custom-dictionary textarea,
.result-preview {
  background: #fbfcfd;
  border: 1px solid #d8e2e8;
  border-radius: 8px;
  color: #18202a;
  font: 14px/1.7 "Microsoft YaHei", "Segoe UI", sans-serif;
  outline: none;
  padding: 12px;
  width: 100%;
}

.conversion-field textarea,
.result-preview {
  box-sizing: border-box;
  height: var(--conversion-text-height);
  min-height: 120px;
}

.conversion-field textarea {
  resize: vertical;
}

.conversion-field textarea:focus,
.custom-dictionary textarea:focus {
  border-color: #176b87;
  box-shadow: 0 0 0 3px rgba(23, 107, 135, 0.12);
}

.result-preview {
  background: #f6f8fa;
  overflow: auto;
  white-space: pre-wrap;
  word-break: break-word;
}

.result-preview .changed {
  background: #fff0a8;
  border-radius: 3px;
  box-shadow: 0 0 0 1px rgba(188, 137, 0, 0.16);
}

.result-preview .placeholder {
  color: #8996a1;
}

.conversion-field small,
.custom-dictionary-head small {
  color: #667582;
  font-size: 12px;
  text-align: right;
}

.custom-dictionary {
  display: grid;
  gap: 8px;
}

.custom-dictionary-head span {
  color: #667582;
  display: block;
  font-size: 12px;
  font-weight: 400;
  margin-top: 3px;
}

.custom-dictionary textarea {
  min-height: 86px;
  resize: vertical;
}

.text-drop-box {
  align-items: center;
  background: #f8fafb;
  border: 1px dashed #b7c8d4;
  border-radius: 8px;
  color: #43515c;
  display: flex;
  gap: 12px;
  min-height: 78px;
  padding: 14px;
}

.text-drop-box > div:nth-child(2) {
  flex: 1 1 auto;
  min-width: 0;
}

.text-drop-box.active {
  background: #edf8fb;
  border-color: #176b87;
  color: #0f5268;
}

.text-drop-box.busy {
  opacity: 0.78;
}

.text-drop-icon {
  align-items: center;
  background: #e7eef3;
  border-radius: 8px;
  color: #176b87;
  display: flex;
  flex: 0 0 auto;
  height: 44px;
  justify-content: center;
  width: 44px;
}

.export-file-button {
  background: #176b87;
  color: #fff;
  flex: 0 0 auto;
  min-height: 34px;
  padding: 0 14px;
}

.export-file-button:disabled {
  background: #dce4e9;
  color: #7a8790;
}

.text-drop-box strong,
.text-drop-box span {
  display: block;
}

.text-drop-box span {
  color: #667582;
  font-size: 13px;
  line-height: 1.5;
  margin-top: 4px;
  overflow-wrap: anywhere;
}

@media (max-width: 920px) {
  .conversion-grid {
    grid-template-columns: minmax(0, 1fr);
  }

  .text-conversion-heading {
    align-items: flex-start;
  }

  .text-drop-box {
    align-items: stretch;
    flex-wrap: wrap;
  }

  .export-file-button {
    width: 100%;
  }
}
</style>
