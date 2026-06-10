<script setup lang="ts">
import { open, save } from '@tauri-apps/plugin-dialog'
import { computed, onUnmounted, ref, watch } from 'vue'
import { loadConfig, saveConfig } from '../api/config'
import RuleDictionaryModal from '../components/RuleDictionaryModal.vue'
import {
  organizeCcSubtitleText,
  readCcSubtitleFile,
  saveCcSubtitleFile,
  saveCcSubtitleToPath,
  type CcReplacementRule,
  type CcSubtitleResult
} from '../api/ccSubtitle'
import { useToast } from '../composables/useToast'
import { globalDragActive, pendingDrop, pushDiag } from '../stores/dropStore'
import type { AppConfig } from '../types'
import { parseRuleDictionary } from '../utils/ruleDictionary'

const sourceText = ref('')
const result = ref<CcSubtitleResult | null>(null)
const pendingFilePath = ref('')
const replacementEnabled = ref(true)
const replacementDictionary = ref('')
const dictionaryOpen = ref(false)
const busy = ref(false)
const statusText = ref('')
const appConfig = ref<AppConfig | null>(null)
const toast = useToast()
let organizeTimer: ReturnType<typeof setTimeout> | null = null
let dictionarySaveTimer: ReturnType<typeof setTimeout> | null = null
let organizeSeq = 0
let dictionaryLoaded = false
const dictionaryLoadPromise = loadReplacementDictionary()

void dictionaryLoadPromise

const resultText = computed(() => result.value?.text ?? '')
const sourceCount = computed(() => Array.from(sourceText.value).length)
const resultCount = computed(() => Array.from(resultText.value).length)
const hasResult = computed(() => Boolean(resultText.value))
const replacementRules = computed<CcReplacementRule[]>(() => {
  return parseRuleDictionary(replacementDictionary.value)
    .map((rule) => ({ replacement: rule.target, pattern: rule.pattern }))
})
const activeReplacementRules = computed(() => replacementEnabled.value ? replacementRules.value : [])

async function loadReplacementDictionary() {
  try {
    const config = await loadConfig()
    appConfig.value = config
    replacementDictionary.value = config.ccSubtitleReplacementDictionary ?? ''
  } catch (err) {
    statusText.value = String(err)
  } finally {
    dictionaryLoaded = true
  }
}

function scheduleSaveReplacementDictionary() {
  if (!dictionaryLoaded) return
  if (dictionarySaveTimer) clearTimeout(dictionarySaveTimer)
  dictionarySaveTimer = setTimeout(() => {
    void saveReplacementDictionary()
  }, 600)
}

async function saveReplacementDictionary() {
  try {
    const base = appConfig.value ?? await loadConfig()
    if (base.ccSubtitleReplacementDictionary === replacementDictionary.value) return
    const next: AppConfig = {
      ...base,
      ccSubtitleReplacementDictionary: replacementDictionary.value
    }
    appConfig.value = next
    await saveConfig(next)
  } catch (err) {
    statusText.value = String(err)
  }
}

function scheduleOrganize() {
  if (organizeTimer) clearTimeout(organizeTimer)
  if (!sourceText.value.trim()) {
    result.value = null
    statusText.value = ''
    return
  }
  organizeTimer = setTimeout(() => {
    void organizeCurrentText()
  }, 180)
}

async function organizeCurrentText() {
  const seq = ++organizeSeq
  busy.value = true
  try {
    await dictionaryLoadPromise
    const next = await organizeCcSubtitleText(sourceText.value, activeReplacementRules.value)
    if (seq !== organizeSeq) return
    result.value = next
    statusText.value = `已整理：改动 ${next.changedLines} 行，新增 ${next.insertedLines} 行，自定义词库命中 ${next.replacementCount} 处`
  } catch (err) {
    if (seq !== organizeSeq) return
    statusText.value = String(err)
    toast.error('CC 字幕整理失败', 2200)
  } finally {
    if (seq === organizeSeq) busy.value = false
  }
}

async function chooseFile() {
  if (busy.value) return
  const selected = await open({
    multiple: false,
    filters: [
      { name: 'ASS subtitles', extensions: ['ass', 'ssa'] },
      { name: 'Text files', extensions: ['txt'] },
      { name: 'All files', extensions: ['*'] }
    ]
  })
  if (typeof selected === 'string') {
    await loadFile(selected)
  }
}

async function loadFile(path: string) {
  busy.value = true
  statusText.value = '正在读取字幕文件...'
  try {
    await dictionaryLoadPromise
    const text = await readCcSubtitleFile(path)
    pendingFilePath.value = path
    sourceText.value = text
    pushDiag(`cc subtitle loaded: ${path}`)
    await organizeCurrentText()
  } catch (err) {
    statusText.value = String(err)
    toast.error('读取字幕失败', 2200)
  } finally {
    busy.value = false
  }
}

async function copyResult() {
  if (!resultText.value) return
  try {
    await navigator.clipboard.writeText(resultText.value)
    toast.success('已复制', 1600)
  } catch {
    toast.error('复制失败', 2200)
  }
}

function clearText() {
  sourceText.value = ''
  result.value = null
  pendingFilePath.value = ''
  statusText.value = ''
}

async function exportPendingFile() {
  if (!pendingFilePath.value || !resultText.value || busy.value) return
  busy.value = true
  try {
    const saved = await saveCcSubtitleFile(pendingFilePath.value, resultText.value, '_cc整理', false)
    statusText.value = `已导出：${saved.outputPath}`
    toast.success('已导出', 1800)
  } catch (err) {
    const message = String(err)
    if (!message.startsWith('OUTPUT_EXISTS:')) {
      statusText.value = message
      toast.error('导出失败', 2200)
      return
    }

    const outputPath = message.slice('OUTPUT_EXISTS:'.length)
    const shouldOverwrite = window.confirm(`输出文件已存在：\n${outputPath}\n\n是否覆盖？`)
    if (!shouldOverwrite) {
      statusText.value = '已取消导出，未覆盖现有文件。'
      return
    }

    const saved = await saveCcSubtitleFile(pendingFilePath.value, resultText.value, '_cc整理', true)
    statusText.value = `已覆盖：${saved.outputPath}`
    toast.success('已覆盖导出', 1800)
  } finally {
    busy.value = false
  }
}

async function exportAs() {
  if (!resultText.value || busy.value) return
  const outputPath = await save({
    title: '导出 CC 字幕整理结果',
    defaultPath: buildDefaultExportPath(pendingFilePath.value),
    filters: [
      { name: 'ASS subtitles', extensions: ['ass', 'ssa'] },
      { name: 'Text files', extensions: ['txt'] },
      { name: 'All files', extensions: ['*'] }
    ]
  })
  if (!outputPath) return

  busy.value = true
  try {
    const saved = await saveCcSubtitleToPath(outputPath, resultText.value)
    statusText.value = `已导出：${saved.outputPath}`
    toast.success('已导出', 1800)
  } catch (err) {
    statusText.value = String(err)
    toast.error('导出失败', 2200)
  } finally {
    busy.value = false
  }
}

function buildDefaultExportPath(sourcePath: string) {
  if (!sourcePath) return 'cc-subtitle.ass'
  const separatorIndex = Math.max(sourcePath.lastIndexOf('/'), sourcePath.lastIndexOf('\\'))
  const directory = separatorIndex >= 0 ? sourcePath.slice(0, separatorIndex + 1) : ''
  const fileName = separatorIndex >= 0 ? sourcePath.slice(separatorIndex + 1) : sourcePath
  const dotIndex = fileName.lastIndexOf('.')
  const stem = dotIndex > 0 ? fileName.slice(0, dotIndex) : fileName
  const extension = dotIndex > 0 ? fileName.slice(dotIndex) : '.ass'
  return `${directory}${stem}_cc整理${extension}`
}

watch([sourceText, replacementEnabled], scheduleOrganize)

watch(replacementDictionary, () => {
  scheduleSaveReplacementDictionary()
  scheduleOrganize()
})

watch(pendingDrop, (drop) => {
  if (drop?.target !== 'tools') return
  if (drop.tool !== 'cc-subtitle') return
  const path = drop?.subtitlePath || drop?.textPath
  if (!path) return
  pendingDrop.value = null
  void loadFile(path)
}, { immediate: true })

onUnmounted(() => {
  if (organizeTimer) clearTimeout(organizeTimer)
  if (dictionarySaveTimer) clearTimeout(dictionarySaveTimer)
})
</script>

<template>
  <section class="cc-subtitle-workspace">
    <div v-if="globalDragActive" class="drop-overlay">松开以读取 ASS / SSA 字幕</div>

    <section class="panel cc-panel">
      <div class="panel-heading cc-heading">
        <div>
          <h2>CC 字幕整理</h2>
          <p>把 Web CC 的说话人标签拆成花字行，并将台词整理为听轴行。</p>
        </div>
        <div class="cc-actions">
          <label class="switch-row">
            <input v-model="replacementEnabled" type="checkbox" />
            <span class="switch"></span>
            <span>启用自定义词库</span>
          </label>
          <button type="button" class="secondary" @click="dictionaryOpen = true">自定义词库</button>
          <button type="button" class="secondary" :disabled="busy" @click="chooseFile">选择字幕</button>
        </div>
      </div>

      <div class="cc-grid">
        <div class="cc-field">
          <span class="field-head">
            <strong>输入</strong>
            <span class="field-tools">
              <small>{{ sourceCount }} 字</small>
              <button class="field-tool" type="button" :disabled="!sourceText || busy" @click="clearText">清空</button>
            </span>
          </span>
          <textarea
            v-model="sourceText"
            spellcheck="false"
            placeholder="粘贴 ASS / SSA 内容，或拖入字幕文件"
          ></textarea>
        </div>

        <div class="cc-field">
          <span class="field-head">
            <strong>结果</strong>
            <span class="field-tools">
              <small>{{ resultCount }} 字</small>
              <button class="field-tool" type="button" :disabled="!hasResult" @click="copyResult">复制</button>
            </span>
          </span>
          <pre class="cc-result">{{ resultText }}</pre>
        </div>
      </div>

      <div class="cc-footer">
        <span>{{ statusText || '整理规则：说话人标签转花字，台词转听轴，清理省略号、换行和多余空格。' }}</span>
        <div class="actions">
          <button type="button" class="secondary" :disabled="!hasResult || busy" @click="exportAs">另存为</button>
          <button type="button" :disabled="!pendingFilePath || !hasResult || busy" @click="exportPendingFile">导出到原目录</button>
        </div>
      </div>
    </section>

    <RuleDictionaryModal
      v-model:open="dictionaryOpen"
      v-model="replacementDictionary"
      title="自定义词库"
      description="维护 CC 说话人和台词里的名称规则，整理字幕时会把命中的文本替换为标准写法。"
      target-label="标准写法"
      pattern-label="匹配规则"
      target-placeholder="例如 章昊"
      pattern-placeholder="例如 (?i)ZHANG\\s*HAO"
      raw-placeholder="[&quot;章昊&quot;] = &quot;(?i)ZHANG\\s*HAO&quot;"
      ariaLabel="CC 字幕自定义词库"
    />
  </section>
</template>

<style scoped>
.cc-subtitle-workspace {
  min-height: 0;
  position: relative;
}

.cc-panel {
  display: grid;
  gap: 14px;
}

.cc-heading {
  align-items: center;
  margin-bottom: 0;
}

.cc-actions {
  align-items: center;
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  justify-content: flex-end;
}

.cc-grid {
  display: grid;
  gap: 14px;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  min-height: 480px;
}

.cc-field {
  display: grid;
  gap: 8px;
  grid-template-rows: auto minmax(0, 1fr);
  min-height: 0;
}

.field-head {
  align-items: center;
  display: flex;
  justify-content: space-between;
  gap: 12px;
}

.field-tools {
  align-items: center;
  display: inline-flex;
  gap: 8px;
}

.field-tools small {
  color: #667582;
  font-size: 12px;
}

.field-tool {
  background: #eef2f6;
  border: 1px solid #dce5ec;
  color: #43515c;
  font-size: 12px;
  min-height: 28px;
  padding: 0 10px;
}

.field-tool.primary {
  background: #176b87;
  border-color: #176b87;
  color: #fff;
}

.cc-field textarea,
.cc-result,
.cc-dictionary-dialog textarea {
  background: #f9fbfc;
  border: 1px solid #d6dee5;
  border-radius: 8px;
  color: #18202a;
  font-family: "Cascadia Code", Consolas, "Microsoft YaHei", monospace;
  font-size: 12.5px;
  line-height: 1.55;
  margin: 0;
  min-height: 0;
  overflow: auto;
  padding: 12px;
  resize: none;
  white-space: pre;
  width: 100%;
}

.cc-result {
  user-select: text;
}

.cc-footer {
  align-items: center;
  border-top: 1px solid #e4ebf0;
  color: #667582;
  display: flex;
  gap: 14px;
  justify-content: space-between;
  padding-top: 14px;
}

.cc-footer > span {
  font-size: 13px;
  overflow-wrap: anywhere;
}

.cc-dictionary-modal {
  align-items: center;
  background: rgba(15, 23, 32, 0.42);
  display: flex;
  inset: 0;
  justify-content: center;
  padding: 28px;
  position: fixed;
  z-index: 90;
}

.cc-dictionary-dialog {
  background: #fff;
  border: 1px solid #dce3e8;
  border-radius: 8px;
  box-shadow: 0 20px 60px rgba(15, 23, 32, 0.22);
  display: grid;
  gap: 14px;
  max-height: min(780px, calc(100vh - 56px));
  max-width: min(980px, 100%);
  overflow: auto;
  padding: 18px;
  width: 980px;
}

.cc-dictionary-dialog-head,
.cc-dictionary-toolbar,
.cc-dictionary-dialog-foot {
  align-items: center;
  display: flex;
  gap: 14px;
  justify-content: space-between;
}

.cc-dictionary-dialog-head p,
.cc-dictionary-dialog-foot span,
.replacement-empty {
  color: #667582;
  font-size: 13px;
  line-height: 1.55;
}

.cc-dictionary-tabs {
  background: #edf2f5;
  border-radius: 8px;
  display: inline-flex;
  padding: 3px;
}

.cc-dictionary-tabs button {
  background: transparent;
  color: #52616c;
  min-height: 30px;
  padding: 0 12px;
}

.cc-dictionary-tabs button.active {
  background: #176b87;
  color: #fff;
}

.replacement-entry-editor {
  display: grid;
  gap: 8px;
}

.replacement-entry-head,
.replacement-entry-row {
  display: grid;
  gap: 10px;
  grid-template-columns: minmax(160px, 0.42fr) minmax(240px, 1fr) auto;
}

.replacement-entry-head {
  color: #667582;
  font-size: 12px;
  font-weight: 700;
}

.replacement-entry-list {
  display: grid;
  gap: 8px;
  max-height: 360px;
  overflow: auto;
}

.replacement-entry-row {
  align-items: start;
  background: #f8fafb;
  border: 1px solid #e3e9ee;
  border-radius: 8px;
  padding: 10px;
}

.replacement-entry-row.invalid {
  border-color: #f2b8b5;
}

.replacement-entry-row label {
  display: grid;
  gap: 5px;
}

.replacement-entry-row label > span {
  color: #667582;
  display: none;
  font-size: 12px;
}

.replacement-entry-row small {
  color: #b45309;
  grid-column: 1 / -1;
}

.cc-dictionary-dialog textarea {
  min-height: 360px;
}

@media (max-width: 1080px) {
  .cc-grid {
    grid-template-columns: 1fr;
  }

  .cc-heading,
  .cc-footer,
  .cc-dictionary-dialog-head,
  .cc-dictionary-toolbar,
  .cc-dictionary-dialog-foot {
    align-items: stretch;
    flex-direction: column;
  }

  .cc-actions,
  .cc-footer .actions {
    justify-content: flex-start;
  }

  .replacement-entry-head {
    display: none;
  }

  .replacement-entry-row {
    grid-template-columns: 1fr;
  }

  .replacement-entry-row label > span {
    display: inline;
  }
}
</style>
