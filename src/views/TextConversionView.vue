<script setup lang="ts">
import { save } from '@tauri-apps/plugin-dialog'
import { computed, nextTick, onUnmounted, ref, watch } from 'vue'
import { loadConfig, saveConfig } from '../api/config'
import RuleDictionaryModal from '../components/RuleDictionaryModal.vue'
import { useToast } from '../composables/useToast'
import { globalDragActive, pendingDrop, pushDiag } from '../stores/dropStore'
import type { AppConfig } from '../types'
import { parseRuleDictionary, serializeValidRuleDictionary } from '../utils/ruleDictionary'
import {
  convertChineseText,
  readPlainTextFile,
  saveConvertedTextFile,
  saveConvertedTextToPath,
  type ChineseConversionMode,
  type CustomConversionRule
} from '../api/textConversion'

type DiffSegment = {
  text: string
  changed: boolean
}

const DIFF_HIGHLIGHT_CHAR_LIMIT = 12000

const mode = ref<ChineseConversionMode>('t2s')
const sourceText = ref('')
const resultText = ref('')
const customDictionary = ref('')
const fileStatus = ref('')
const fileBusy = ref(false)
const textBusy = ref(false)
const dictionaryOpen = ref(false)
const appConfig = ref<AppConfig | null>(null)
const sourceTextarea = ref<HTMLTextAreaElement | null>(null)
const resultPreview = ref<HTMLDivElement | null>(null)
const sourceLineNumbers = ref<HTMLDivElement | null>(null)
const resultLineNumbers = ref<HTMLDivElement | null>(null)
const pendingTextFilePath = ref('')
const toast = useToast()

let convertTimer: ReturnType<typeof setTimeout> | null = null
let dictionarySaveTimer: ReturnType<typeof setTimeout> | null = null
let convertSeq = 0
let convertInFlight = false
let convertAgain = false
let scrollSyncing = false
let customDictionaryLoaded = false
const customDictionaryLoadPromise = loadCustomDictionary()

void customDictionaryLoadPromise

const modeLabel = computed(() => (mode.value === 's2t' ? '简体转繁体' : '繁体转简体'))
const outputSuffix = computed(() => (mode.value === 's2t' ? '_繁体' : '_简体'))
const sourceCount = computed(() => Array.from(sourceText.value).length)
const resultCount = computed(() => Array.from(resultText.value).length)
const customRules = computed<CustomConversionRule[]>(() => {
  return parseRuleDictionary(customDictionary.value)
    .map((rule) => ({ from: rule.pattern, to: rule.target }))
})
const resultDiffSegments = computed(() => diffResult(sourceText.value, resultText.value))
const sourceLines = computed(() => buildLineNumbers(sourceText.value))
const resultLines = computed(() => buildLineNumbers(resultText.value))

function buildLineNumbers(text: string) {
  const count = text ? text.split(/\r\n|\r|\n/).length : 1
  return Array.from({ length: count }, (_, index) => index + 1)
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
  }, 500)
}

async function convertCurrentText() {
  if (convertInFlight) {
    convertAgain = true
    return
  }
  const seq = ++convertSeq
  const text = sourceText.value
  const rules = customRules.value
  convertInFlight = true
  textBusy.value = true
  try {
    const converted = await convertChineseText(text, mode.value, rules)
    if (seq !== convertSeq) return
    resultText.value = converted
    fileStatus.value = `已完成${modeLabel.value}`
  } catch (err) {
    if (seq !== convertSeq) return
    fileStatus.value = String(err)
  } finally {
    textBusy.value = false
    convertInFlight = false
    if (convertAgain) {
      convertAgain = false
      scheduleConvert()
    }
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
    const validDictionary = serializeValidRuleDictionary(customDictionary.value, { validatePattern: false })
    if (base.textConversionCustomDictionary === validDictionary) return
    const next: AppConfig = {
      ...base,
      textConversionCustomDictionary: validDictionary
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
  try {
    await navigator.clipboard.writeText(resultText.value)
    fileStatus.value = '转换结果已复制'
    toast.success('已复制', 1800)
  } catch (err) {
    fileStatus.value = String(err)
    toast.error('复制失败', 2200)
  }
}

function clearText() {
  convertSeq += 1
  convertAgain = false
  sourceText.value = ''
  resultText.value = ''
  fileStatus.value = ''
  pendingTextFilePath.value = ''
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
  if (sourceTextarea.value && sourceLineNumbers.value) {
    sourceLineNumbers.value.scrollTop = sourceTextarea.value.scrollTop
  }
  if (resultPreview.value && resultLineNumbers.value) {
    resultLineNumbers.value.scrollTop = resultPreview.value.scrollTop
  }
}

function syncSourceScrollToResult() {
  syncScroll(resultPreview.value, sourceTextarea.value)
  if (resultPreview.value && resultLineNumbers.value) {
    resultLineNumbers.value.scrollTop = resultPreview.value.scrollTop
  }
  if (sourceTextarea.value && sourceLineNumbers.value) {
    sourceLineNumbers.value.scrollTop = sourceTextarea.value.scrollTop
  }
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

async function convertDroppedTextFile(path: string) {
  fileBusy.value = true
  fileStatus.value = '正在读取文本文件...'
  try {
    await customDictionaryLoadPromise
    const text = await readPlainTextFile(path)
    const converted = await convertTextNow(text)
    pendingTextFilePath.value = path
    sourceText.value = text
    resultText.value = converted
    fileStatus.value = '已读取文本文件。可先补充自定义词库，确认结果后再导出。'
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

async function exportTextFileAs() {
  if ((!sourceText.value && !resultText.value) || fileBusy.value) return
  fileBusy.value = true
  fileStatus.value = '正在导出转换结果...'
  try {
    const text = textBusy.value || !resultText.value ? await convertTextNow(sourceText.value) : resultText.value
    const outputPath = await save({
      title: '导出转换结果',
      defaultPath: buildDefaultExportPath(pendingTextFilePath.value, outputSuffix.value, 'converted-text.txt'),
      filters: [
        { name: 'Text and subtitles', extensions: ['txt', 'ass', 'ssa', 'srt', 'vtt', 'sub'] },
        { name: 'All files', extensions: ['*'] }
      ]
    })
    if (!outputPath) return
    const saved = await saveConvertedTextToPath(outputPath, text)
    resultText.value = text
    fileStatus.value = `已导出：${saved.outputPath}`
    pushDiag(`text conversion saved: ${saved.outputPath}`)
  } catch (err) {
    const message = String(err)
    fileStatus.value = message
    pushDiag(`text conversion export failed: ${message}`)
  } finally {
    fileBusy.value = false
  }
}

function diffResult(source: string, result: string): DiffSegment[] {
  if (!result) return []
  if (source.length > DIFF_HIGHLIGHT_CHAR_LIMIT || result.length > DIFF_HIGHLIGHT_CHAR_LIMIT) {
    return [{ text: result, changed: false }]
  }
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
  if (drop?.target !== 'tools') return
  if (drop.tool !== 'text-conversion') return
  const path = drop?.textPath
  if (!path) return
  pendingDrop.value = null
  void convertDroppedTextFile(path)
}, { immediate: true })

onUnmounted(() => {
  if (convertTimer) clearTimeout(convertTimer)
  if (dictionarySaveTimer) clearTimeout(dictionarySaveTimer)
})
</script>

<template>
  <section class="text-conversion-workspace">
    <div v-if="globalDragActive" class="drop-overlay">松开以转换文本或字幕文件</div>

    <section class="panel text-conversion-panel">
      <div class="panel-heading text-conversion-heading">
        <div>
          <h2>繁简字转换</h2>
          <p>使用 zhconv 转换繁简文本，自定义词库会优先保护和替换指定词条。</p>
        </div>
        <div class="heading-tools">
          <button type="button" class="dictionary-button" @click="dictionaryOpen = true">自定义词库</button>
          <div class="conversion-mode" role="group" aria-label="转换方向">
            <button :class="{ active: mode === 't2s' }" @click="setMode('t2s')">繁体 → 简体</button>
            <button :class="{ active: mode === 's2t' }" @click="setMode('s2t')">简体 → 繁体</button>
          </div>
        </div>
      </div>

      <div class="conversion-grid">
        <div class="conversion-field">
          <span class="field-head">
            <strong>输入</strong>
            <span class="field-tools">
              <small>{{ sourceCount }} 字</small>
              <button class="field-tool" type="button" :disabled="!sourceText" @click="clearText">清空</button>
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
              @scroll="syncResultScrollToSource"
              placeholder="粘贴要处理的文本&#10;可拖入 TXT / ASS / SSA / SRT / VTT / SUB 文件"
            ></textarea>
          </div>
        </div>

        <div class="conversion-field">
          <span class="field-head">
            <strong>结果</strong>
            <span class="field-tools">
              <small>{{ resultCount }} 字</small>
              <button class="field-tool" type="button" :disabled="!resultText" @click="copyResult">复制</button>
              <button
                class="field-tool primary"
                type="button"
                :disabled="(!sourceText && !resultText) || fileBusy"
                @click="exportTextFileAs"
              >导出</button>
            </span>
          </span>
          <div class="text-editor-shell">
            <div ref="resultLineNumbers" class="line-numbers" aria-hidden="true">
              <span v-for="line in resultLines" :key="line">{{ line }}</span>
            </div>
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
          </div>
        </div>
      </div>

      <RuleDictionaryModal
        v-model:open="dictionaryOpen"
        v-model="customDictionary"
        title="自定义词库"
        description="维护繁简转换后仍需固定的词条，转换时会先匹配规则再输出标准写法。"
        target-label="标准写法"
        pattern-label="匹配规则(支持正则)"
        target-placeholder="例如 利落"
        pattern-placeholder="例如 俐落"
        raw-placeholder="&quot;利落&quot; = &quot;俐落&quot;"
        ariaLabel="繁简转换自定义词库"
        :validate-pattern="false"
        :supports-capture="false"
      />
    </section>
  </section>
</template>

<style scoped>
.text-conversion-workspace {
  display: grid;
  height: 100%;
  min-height: 0;
  position: relative;
}

.text-conversion-panel {
  box-sizing: border-box;
  display: grid;
  gap: 10px;
  grid-template-rows: auto minmax(0, 1fr);
  height: 100%;
  min-height: 0;
}

.text-conversion-heading {
  align-items: center;
  margin-bottom: 0;
}

.heading-tools {
  align-items: center;
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  justify-content: flex-end;
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

.dictionary-button {
  background: #e5eaee;
  color: #24313c;
  min-height: 32px;
  padding: 0 12px;
}

.conversion-grid {
  align-items: stretch;
  display: grid;
  gap: 14px;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  height: 100%;
  min-height: 0;
}

.conversion-field {
  display: grid;
  gap: 8px;
  grid-template-rows: auto minmax(0, 1fr);
  min-height: 0;
  min-width: 0;
}

.field-head {
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

.field-head strong {
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

.field-tool:disabled {
  background: #dce4e9;
  color: #7a8790;
}

.text-editor-shell {
  background: #fbfcfd;
  border: 1px solid #d8e2e8;
  border-radius: 8px;
  display: grid;
  grid-template-columns: 44px minmax(0, 1fr);
  height: 100%;
  min-height: 320px;
  overflow: hidden;
}

.line-numbers {
  background: linear-gradient(#f2f5f7, #f2f5f7) 0 0 / 100% calc(100% - 1px) no-repeat;
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

.conversion-field textarea,
.result-preview {
  background: transparent;
  border: 0;
  color: #18202a;
  font: 14px/1.7 "Microsoft YaHei", "Segoe UI", sans-serif;
  outline: none;
  padding: 12px;
  width: 100%;
}

.conversion-field textarea,
.result-preview {
  box-sizing: border-box;
  height: 100%;
  min-height: 0;
}

.conversion-field textarea {
  resize: none;
}

.conversion-field textarea:focus {
  box-shadow: none;
}

.text-editor-shell:focus-within {
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

.field-tools small,
.dictionary-dialog-foot span {
  color: #667582;
  font-size: 12px;
}

@media (max-width: 920px) {
  .conversion-grid {
    grid-template-columns: minmax(0, 1fr);
  }

  .text-conversion-heading {
    align-items: flex-start;
  }

  .heading-tools {
    justify-content: flex-start;
  }

  .field-tools {
    flex-wrap: wrap;
    justify-content: flex-end;
  }
}
</style>

<style>
.dictionary-modal {
  align-items: center;
  background: rgba(15, 23, 32, 0.38);
  display: flex;
  inset: 0;
  justify-content: center;
  padding: 28px;
  position: fixed;
  z-index: 120;
}

.dictionary-dialog {
  background: #fff;
  border: 1px solid #d8e2e8;
  border-radius: 8px;
  box-shadow: 0 22px 64px rgba(15, 23, 32, 0.22);
  color: #24313c;
  display: grid;
  gap: 14px;
  max-width: min(720px, 100%);
  padding: 18px;
  width: 720px;
}

.dictionary-dialog-head,
.dictionary-dialog-foot {
  align-items: center;
  display: flex;
  gap: 12px;
  justify-content: space-between;
}

.dictionary-dialog-head h2 {
  color: #102030;
  font-size: 17px;
  margin: 0;
}

.dictionary-dialog-head p {
  color: #667582;
  font-size: 13px;
  margin: 5px 0 0;
}

.dictionary-dialog textarea {
  background: #fbfcfd;
  border: 1px solid #d8e2e8;
  border-radius: 8px;
  color: #18202a;
  font: 14px/1.7 "Microsoft YaHei", "Segoe UI", sans-serif;
  min-height: 300px;
  outline: none;
  padding: 12px;
  resize: none;
  width: 100%;
}

.dictionary-dialog textarea:focus {
  border-color: #176b87;
  box-shadow: 0 0 0 3px rgba(23, 107, 135, 0.12);
}

.dictionary-dialog-foot span {
  color: #667582;
  font-size: 12px;
  overflow-wrap: anywhere;
}

@media (max-width: 760px) {
  .dictionary-modal {
    align-items: stretch;
    padding: 16px;
  }

  .dictionary-dialog {
    align-content: start;
    width: 100%;
  }

  .dictionary-dialog-head,
  .dictionary-dialog-foot {
    align-items: stretch;
    flex-direction: column;
  }
}
</style>
