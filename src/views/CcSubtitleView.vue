<script setup lang="ts">
import { open, save } from '@tauri-apps/plugin-dialog'
import { computed, onUnmounted, ref, watch } from 'vue'
import { loadCcSubtitleConfig, saveCcSubtitleConfig } from '../api/toolConfig'
import AppSelect from '../components/AppSelect.vue'
import RuleDictionaryModal from '../components/RuleDictionaryModal.vue'
import {
  organizeCcSubtitleText,
  readCcSubtitleFile,
  saveCcSubtitleToPath,
  type CcReplacementRule,
  type CcSubtitleResult
} from '../api/ccSubtitle'
import { useToast } from '../composables/useToast'
import { globalDragActive, pendingDrop, pushDiag } from '../stores/dropStore'
import type { CcStyleProfile, CcSubtitleConfig } from '../types'
import { parseRuleDictionary, serializeValidRuleDictionary } from '../utils/ruleDictionary'
import { useI18n } from '../i18n'

const INPUT_PREVIEW_LIMIT = 200_000
const RESULT_PREVIEW_LIMIT = 200_000

const sourceText = ref('')
const result = ref<CcSubtitleResult | null>(null)
const pendingFilePath = ref('')
const replacementEnabled = ref(true)
const replacementDictionary = ref('')
const styleProfiles = ref<CcStyleProfile[]>([])
const selectedStyleProfileId = ref('')
const activeStyleNames = ref<string[]>([])
const assHeaderTemplate = ref('')
const screenStyleName = ref('')
const speakStyleName = ref('')
const styleProfileOpen = ref(false)
const styleProfileNameOpen = ref(false)
const styleProfileNameMode = ref<'create' | 'rename'>('create')
const styleProfileNameInput = ref('')
const pendingStyleProfileAssHeader = ref('')
const dictionaryOpen = ref(false)
const busy = ref(false)
const organizing = ref(false)
const statusText = ref('')
const toolConfig = ref<CcSubtitleConfig | null>(null)
const ccGrid = ref<HTMLDivElement | null>(null)
const sourcePanePercent = ref(50)
const resizing = ref(false)
const toast = useToast()
const { t } = useI18n()
let organizeTimer: ReturnType<typeof setTimeout> | null = null
let dictionarySaveTimer: ReturnType<typeof setTimeout> | null = null
let organizeSeq = 0
let organizeInFlight = false
let organizeAgain = false
let dictionaryLoaded = false
let dictionaryLoadPromise: Promise<void> | null = null

const resultText = computed(() => result.value?.text ?? '')
const sourceCount = computed(() => sourceText.value.length)
const resultCount = computed(() => resultText.value.length)
const sourcePreviewTruncated = computed(() => sourceText.value.length > INPUT_PREVIEW_LIMIT)
const resultPreviewTruncated = computed(() => resultText.value.length > RESULT_PREVIEW_LIMIT)
const sourceEditorText = computed({
  get: () => previewText(sourceText.value, INPUT_PREVIEW_LIMIT),
  set: (value: string) => {
    sourceText.value = value
  }
})
const resultPreviewText = computed(() => previewText(resultText.value, RESULT_PREVIEW_LIMIT))
const hasResult = computed(() => Boolean(resultText.value))
const styleProfileOptions = computed(() => styleProfiles.value.map((profile) => ({
  value: profile.id,
  label: profile.name
})))
const currentStyleProfile = computed(() => {
  return styleProfiles.value.find((profile) => profile.id === selectedStyleProfileId.value) ?? null
})
const styleProfileSummary = computed(() => {
  if (!currentStyleProfile.value) return t('ccSubtitle.style.noProfile')
  if (!styleReady.value) return t('ccSubtitle.style.chooseStyles')
  return t('ccSubtitle.style.summary', { speak: speakStyleName.value, screen: screenStyleName.value })
})
const styleOptions = computed(() => activeStyleNames.value.map((style) => ({ value: style, label: style })))
const replacementRuleCount = computed(() => replacementRules.value.length)
const styleReady = computed(() => {
  return Boolean(currentStyleProfile.value) &&
    activeStyleNames.value.includes(screenStyleName.value) &&
    activeStyleNames.value.includes(speakStyleName.value)
})
const replacementRules = computed<CcReplacementRule[]>(() => {
  return parseRuleDictionary(replacementDictionary.value, { validatePattern: true })
    .map((rule) => ({ replacement: rule.target, pattern: rule.pattern }))
})
const activeReplacementRules = computed(() => replacementEnabled.value ? replacementRules.value : [])
const ccGridStyle = computed(() => ({
  '--source-pane-percent': `${sourcePanePercent.value}%`,
  '--result-pane-percent': `${100 - sourcePanePercent.value}%`
}))

async function loadReplacementDictionary() {
  try {
    const config = await loadCcSubtitleConfig()
    toolConfig.value = config
    replacementDictionary.value = config.replacementDictionary ?? ''
    styleProfiles.value = normalizeStyleProfiles(config.styleProfiles ?? [])
    selectedStyleProfileId.value = styleProfiles.value.some((profile) => profile.id === config.styleProfileId)
      ? config.styleProfileId
      : styleProfiles.value[0]?.id ?? ''
    applySelectedStyleProfile()
  } catch (err) {
    statusText.value = String(err)
  } finally {
    dictionaryLoaded = true
  }
}

function ensureReplacementDictionaryLoaded() {
  if (!dictionaryLoadPromise) {
    dictionaryLoadPromise = loadReplacementDictionary()
  }
  return dictionaryLoadPromise
}

function openDictionary() {
  dictionaryOpen.value = true
  void ensureReplacementDictionaryLoaded()
}

function openStyleProfileDialog() {
  styleProfileOpen.value = true
  void ensureReplacementDictionaryLoaded()
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
    const base = toolConfig.value ?? await loadCcSubtitleConfig()
    const validDictionary = serializeValidRuleDictionary(replacementDictionary.value, { validatePattern: true })
    if (
      base.replacementDictionary === validDictionary &&
      (base.styleProfileId ?? '') === selectedStyleProfileId.value &&
      JSON.stringify(base.styleProfiles ?? []) === JSON.stringify(styleProfiles.value)
    ) return
    const next: CcSubtitleConfig = {
      ...base,
      replacementDictionary: validDictionary,
      styleProfileId: selectedStyleProfileId.value,
      styleProfiles: styleProfiles.value
    }
    toolConfig.value = next
    await saveCcSubtitleConfig(next)
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
  if (!styleReady.value) {
    result.value = null
    statusText.value = t('ccSubtitle.status.needStyle')
    return
  }
  organizeTimer = setTimeout(() => {
    void organizeCurrentText()
  }, 500)
}

async function organizeCurrentText() {
  if (organizeInFlight) {
    organizeAgain = true
    return
  }
  const seq = ++organizeSeq
  organizeInFlight = true
  organizing.value = true
  try {
    await ensureReplacementDictionaryLoaded()
    if (!styleReady.value) {
      result.value = null
      statusText.value = t('ccSubtitle.status.needStyle')
      return
    }
    const next = await organizeCcSubtitleText(
      sourceText.value,
      activeReplacementRules.value,
      screenStyleName.value,
      speakStyleName.value,
      assHeaderTemplate.value
    )
    if (seq !== organizeSeq) return
    result.value = next
    statusText.value = ''
  } catch (err) {
    if (seq !== organizeSeq) return
    statusText.value = String(err)
    toast.error(t('ccSubtitle.toast.organizeFailed'), 2200)
  } finally {
    organizing.value = false
    organizeInFlight = false
    if (organizeAgain) {
      organizeAgain = false
      scheduleOrganize()
    }
  }
}

async function createStyleProfileFromAss() {
  if (busy.value) return
  void ensureReplacementDictionaryLoaded()
  const selected = await open({
    multiple: false,
    filters: [
      { name: 'ASS subtitles', extensions: ['ass', 'ssa'] },
      { name: 'All files', extensions: ['*'] }
    ]
  })
  if (typeof selected !== 'string') return

  busy.value = true
  try {
    const text = await readCcSubtitleFile(selected)
    const styles = parseAssStyleNames(text)
    if (!styles.length) {
      statusText.value = t('ccSubtitle.status.noStylesParsed')
      toast.error(t('ccSubtitle.toast.noStylesParsed'), 2200)
      return
    }
    pendingStyleProfileAssHeader.value = extractReusableAssHeader(text)
    styleProfileNameInput.value = basenameWithoutExtension(selected)
    styleProfileNameMode.value = 'create'
    styleProfileNameOpen.value = true
  } catch (err) {
    statusText.value = String(err)
    toast.error(t('ccSubtitle.toast.readStyleFailed'), 2200)
  } finally {
    busy.value = false
  }
}

function renameCurrentStyleProfile() {
  const current = currentStyleProfile.value
  if (!current) return
  pendingStyleProfileAssHeader.value = ''
  styleProfileNameInput.value = current.name
  styleProfileNameMode.value = 'rename'
  styleProfileNameOpen.value = true
}

function closeStyleProfileNameDialog() {
  styleProfileNameOpen.value = false
  pendingStyleProfileAssHeader.value = ''
  styleProfileNameInput.value = ''
}

function confirmStyleProfileName() {
  const name = styleProfileNameInput.value.trim()
  if (!name) return

  if (styleProfileNameMode.value === 'create') {
    const assHeader = pendingStyleProfileAssHeader.value
    if (!assHeader.trim()) return
    const profile = createStyleProfile(name, assHeader)
    styleProfiles.value = [...styleProfiles.value, profile]
    selectedStyleProfileId.value = profile.id
    applySelectedStyleProfile()
    result.value = null
    statusText.value = t('ccSubtitle.status.profileCreated', { name })
    toast.success(t('ccSubtitle.toast.profileCreated'), 1600)
    scheduleOrganize()
  } else {
    const current = currentStyleProfile.value
    if (!current) return
    if (name !== current.name) {
      updateCurrentStyleProfile({ name })
    }
  }

  scheduleSaveReplacementDictionary()
  closeStyleProfileNameDialog()
}

function deleteCurrentStyleProfile() {
  const current = currentStyleProfile.value
  if (!current) return
  if (!confirm(t('ccSubtitle.confirmDeleteProfile', { name: current.name }))) return
  const next = styleProfiles.value.filter((profile) => profile.id !== current.id)
  styleProfiles.value = next
  selectedStyleProfileId.value = next[0]?.id ?? ''
  applySelectedStyleProfile()
  result.value = null
  scheduleSaveReplacementDictionary()
  scheduleOrganize()
}

function applySelectedStyleProfile() {
  const current = currentStyleProfile.value
  assHeaderTemplate.value = current?.assHeader ?? ''
  activeStyleNames.value = parseAssStyleNames(assHeaderTemplate.value)
  screenStyleName.value = current && activeStyleNames.value.includes(current.screenStyleName)
    ? current.screenStyleName
    : ''
  speakStyleName.value = current && activeStyleNames.value.includes(current.speakStyleName)
    ? current.speakStyleName
    : ''
}

function updateCurrentStyleProfile(patch: Partial<CcStyleProfile>) {
  const id = selectedStyleProfileId.value
  styleProfiles.value = styleProfiles.value.map((profile) =>
    profile.id === id ? { ...profile, ...patch } : profile
  )
}

function createStyleProfile(name: string, assHeader: string): CcStyleProfile {
  return {
    id: crypto.randomUUID(),
    name,
    assHeader,
    screenStyleName: '',
    speakStyleName: ''
  }
}

function normalizeStyleProfiles(profiles: CcStyleProfile[]) {
  return profiles
    .map((profile) => ({
      id: profile.id || crypto.randomUUID(),
      name: profile.name?.trim() || t('ccSubtitle.style.untitled'),
      assHeader: profile.assHeader ?? '',
      screenStyleName: profile.screenStyleName ?? '',
      speakStyleName: profile.speakStyleName ?? ''
    }))
    .filter((profile) => profile.assHeader.trim())
}

function basenameWithoutExtension(path: string) {
  const fileName = path.slice(Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\')) + 1)
  const dotIndex = fileName.lastIndexOf('.')
  return dotIndex > 0 ? fileName.slice(0, dotIndex) : fileName
}

function describeStyleProfile(profile: CcStyleProfile) {
  const speak = profile.speakStyleName || t('ccSubtitle.style.speakUnset')
  const screen = profile.screenStyleName || t('ccSubtitle.style.screenUnset')
  return `${speak} / ${screen}`
}

async function loadFile(path: string) {
  if (!styleReady.value) {
    statusText.value = t('ccSubtitle.status.needStyleBeforeImport')
    toast.error(t('ccSubtitle.toast.needStyle'), 2200)
    return
  }
  busy.value = true
  statusText.value = t('ccSubtitle.status.readingSubtitle')
  try {
    await ensureReplacementDictionaryLoaded()
    const text = await readCcSubtitleFile(path)
    pendingFilePath.value = path
    sourceText.value = text
    pushDiag(`cc subtitle loaded: ${path}`)
    await organizeCurrentText()
  } catch (err) {
    statusText.value = String(err)
    toast.error(t('ccSubtitle.toast.readSubtitleFailed'), 2200)
  } finally {
    busy.value = false
  }
}

function parseAssStyleNames(text: string) {
  const names: string[] = []
  let inStyles = false
  let nameIndex = 0

  for (const rawLine of text.split(/\r?\n/)) {
    const line = rawLine.trim()
    if (!line) continue
    if (line.startsWith('[') && line.endsWith(']')) {
      inStyles = line.toLowerCase() === '[v4+ styles]'
      continue
    }
    if (!inStyles) continue
    if (line.toLowerCase().startsWith('format:')) {
      const fields = line.slice(line.indexOf(':') + 1).split(',').map((field) => field.trim().toLowerCase())
      const nextNameIndex = fields.indexOf('name')
      nameIndex = nextNameIndex >= 0 ? nextNameIndex : 0
      continue
    }
    if (!line.toLowerCase().startsWith('style:')) continue

    const value = line.slice(line.indexOf(':') + 1).trim()
    const fields = value.split(',')
    const name = fields[nameIndex]?.trim()
    if (name) names.push(name)
  }

  return uniqueStyleNames(names)
}

function extractReusableAssHeader(text: string) {
  const lines = text.replace(/\r\n/g, '\n').replace(/\r/g, '\n').split('\n')
  const kept: string[] = []
  let skipSection = false

  for (const line of lines) {
    const trimmed = line.trim()
    if (trimmed.startsWith('[') && trimmed.endsWith(']')) {
      const section = trimmed.toLowerCase()
      if (section === '[events]') break
      skipSection = section === '[aegisub project garbage]'
    }
    if (!skipSection) kept.push(line)
  }

  while (kept.length && !kept[kept.length - 1].trim()) {
    kept.pop()
  }
  return kept.join('\n')
}

function uniqueStyleNames(values: string[]) {
  return Array.from(new Set(values.map((value) => value.trim()).filter(Boolean)))
}

function previewText(text: string, limit: number) {
  if (text.length <= limit) return text
  return `${text.slice(0, limit)}\n\n${t('ccSubtitle.previewTruncated', { count: formatCount(text.length - limit) })}`
}

function formatCount(count: number) {
  return count.toLocaleString('zh-CN')
}

function clampPanePercent(percent: number, width: number) {
  const leftMin = Math.min(360, width * 0.48)
  const rightMin = Math.min(320, width * 0.42)
  const minPercent = (leftMin / width) * 100
  const maxPercent = ((width - rightMin) / width) * 100
  return Math.min(maxPercent, Math.max(minPercent, percent))
}

function updatePaneSplit(clientX: number) {
  const grid = ccGrid.value
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

async function copyResult() {
  if (!resultText.value) return
  try {
    await navigator.clipboard.writeText(resultText.value)
    toast.success(t('common.copied'), 1600)
  } catch {
    toast.error(t('common.copyFailed'), 2200)
  }
}

function clearText() {
  organizeSeq += 1
  organizeAgain = false
  sourceText.value = ''
  result.value = null
  pendingFilePath.value = ''
  statusText.value = ''
}

async function exportAs() {
  if (!resultText.value || busy.value) return
  const outputPath = await save({
    title: t('ccSubtitle.dialog.exportTitle'),
    defaultPath: buildDefaultExportPath(pendingFilePath.value),
    filters: [
      { name: 'ASS subtitles', extensions: ['ass', 'ssa'] },
      { name: 'All files', extensions: ['*'] }
    ]
  })
  if (!outputPath) return

  busy.value = true
  try {
    const saved = await saveCcSubtitleToPath(outputPath, resultText.value)
    statusText.value = t('ccSubtitle.status.exported', { path: saved.outputPath })
    toast.success(t('ccSubtitle.toast.exported'), 1800)
  } catch (err) {
    statusText.value = String(err)
    toast.error(t('ccSubtitle.toast.exportFailed'), 2200)
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
  return `${directory}${stem}${t('ccSubtitle.outputSuffix')}.ass`
}

watch([sourceText, replacementEnabled, screenStyleName, speakStyleName], scheduleOrganize)

watch(replacementDictionary, () => {
  scheduleSaveReplacementDictionary()
  scheduleOrganize()
})

watch([screenStyleName, speakStyleName], () => {
  updateCurrentStyleProfile({
    screenStyleName: screenStyleName.value,
    speakStyleName: speakStyleName.value
  })
  scheduleSaveReplacementDictionary()
})

watch(selectedStyleProfileId, () => {
  applySelectedStyleProfile()
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
  stopPaneResize()
})
</script>

<template>
  <section class="cc-subtitle-workspace">
    <div v-if="globalDragActive" class="drop-overlay">{{ t('ccSubtitle.dropOverlay') }}</div>

    <section class="panel cc-panel">
      <div class="cc-toolbar">
        <div>
          <p v-if="statusText" class="cc-status-summary">{{ statusText }}</p>
        </div>
        <div class="cc-actions">
          <section class="cc-summary-group style-summary-group" :aria-label="t('ccSubtitle.style.title')">
            <div>
              <strong>{{ currentStyleProfile?.name ?? t('ccSubtitle.style.noProfile') }}</strong>
              <span>{{ styleProfileSummary }}</span>
            </div>
            <button type="button" class="secondary summary-action" @click="openStyleProfileDialog">{{ t('ccSubtitle.style.configure') }}</button>
          </section>

          <section class="cc-summary-group dictionary-group" :aria-label="t('ccSubtitle.dictionary.title')">
            <div>
              <strong>{{ t('ccSubtitle.dictionary.title') }}</strong>
              <span>{{ t('ccSubtitle.dictionary.summary', { count: replacementRuleCount, status: replacementEnabled ? t('ccSubtitle.dictionary.enabled') : t('ccSubtitle.dictionary.disabled') }) }}</span>
            </div>
            <div class="dictionary-row">
              <label class="switch-row">
                <input v-model="replacementEnabled" type="checkbox" />
                <span class="switch"></span>
                <span>{{ t('ccSubtitle.dictionary.enable') }}</span>
              </label>
              <button type="button" class="secondary summary-action" @click="openDictionary">{{ t('common.edit') }}</button>
            </div>
          </section>
        </div>
      </div>

      <div
        ref="ccGrid"
        class="cc-grid"
        :class="{ disabled: !styleReady, resizing }"
        :style="ccGridStyle"
      >
        <div class="cc-field">
          <span class="field-head">
            <strong>{{ t('ccSubtitle.input') }}</strong>
            <span class="field-tools">
              <small>{{ t('ccSubtitle.charCount', { count: formatCount(sourceCount) }) }}</small>
              <small v-if="sourcePreviewTruncated">{{ t('ccSubtitle.previewLimit', { count: formatCount(INPUT_PREVIEW_LIMIT) }) }}</small>
              <button class="field-tool" type="button" :disabled="!styleReady || !sourceText || busy" @click="clearText">{{ t('common.clear') }}</button>
            </span>
          </span>
          <textarea
            v-model="sourceEditorText"
            spellcheck="false"
            readonly
            aria-readonly="true"
            :placeholder="t('ccSubtitle.inputPlaceholder')"
          ></textarea>
        </div>

        <button
          type="button"
          class="pane-resizer"
          role="separator"
          aria-orientation="vertical"
          :aria-label="t('ccSubtitle.resizeLabel')"
          @pointerdown.prevent="startPaneResize"
        ></button>

        <div class="cc-field">
          <span class="field-head">
            <strong>{{ t('ccSubtitle.result') }}</strong>
            <span class="field-tools">
              <small v-if="organizing">{{ t('ccSubtitle.organizing') }}</small>
              <small>{{ t('ccSubtitle.charCount', { count: formatCount(resultCount) }) }}</small>
              <small v-if="resultPreviewTruncated">{{ t('ccSubtitle.previewLimit', { count: formatCount(RESULT_PREVIEW_LIMIT) }) }}</small>
              <button class="field-tool" type="button" :disabled="!styleReady || !hasResult" @click="copyResult">{{ t('common.copy') }}</button>
              <button class="field-tool primary" type="button" :disabled="!styleReady || !hasResult || busy || organizing" @click="exportAs">{{ t('common.export') }}</button>
            </span>
          </span>
          <pre class="cc-result">{{ resultPreviewText }}</pre>
        </div>
      </div>

    </section>

    <RuleDictionaryModal
      v-model:open="dictionaryOpen"
      v-model="replacementDictionary"
      :title="t('ccSubtitle.dictionary.modalTitle')"
      :description="t('ccSubtitle.dictionary.description')"
      :target-label="t('ccSubtitle.dictionary.targetLabel')"
      :pattern-label="t('ccSubtitle.dictionary.patternLabel')"
      :target-placeholder="t('ccSubtitle.dictionary.targetPlaceholder')"
      :pattern-placeholder="t('ccSubtitle.dictionary.patternPlaceholder')"
      :raw-placeholder="t('ccSubtitle.dictionary.rawPlaceholder')"
      :ariaLabel="t('ccSubtitle.dictionary.ariaLabel')"
    />

    <div v-if="styleProfileOpen" class="style-profile-modal app-modal-active" role="presentation" @click.self="styleProfileOpen = false">
      <section class="style-profile-dialog" role="dialog" aria-modal="true" aria-labelledby="style-profile-title">
        <header class="style-profile-dialog-head">
          <div>
            <h2 id="style-profile-title">{{ t('ccSubtitle.style.title') }}</h2>
            <p>{{ t('ccSubtitle.style.description') }}</p>
          </div>
          <button type="button" class="secondary close-action" @click="styleProfileOpen = false">{{ t('common.close') }}</button>
        </header>

        <div v-if="styleProfiles.length" class="style-profile-manager">
          <aside class="style-profile-list" :aria-label="t('ccSubtitle.style.listLabel')">
            <div class="style-profile-list-toolbar">
              <button type="button" class="style-profile-list-create" :disabled="busy" @click="createStyleProfileFromAss">
                <svg class="button-icon" viewBox="0 0 24 24" aria-hidden="true">
                  <path d="M12 5v14M5 12h14" />
                </svg>
                {{ t('common.create') }}
              </button>
            </div>
            <div
              v-for="profile in styleProfiles"
              :key="profile.id"
              class="style-profile-list-row"
              :class="{ active: profile.id === selectedStyleProfileId }"
            >
              <button
                type="button"
                class="style-profile-list-item"
                @click="selectedStyleProfileId = profile.id"
              >
                <strong>{{ profile.name }}</strong>
                <span>{{ describeStyleProfile(profile) }}</span>
                <small>{{ t('ccSubtitle.style.styleCount', { count: parseAssStyleNames(profile.assHeader).length }) }}</small>
              </button>
              <div class="style-profile-list-actions">
                <button
                  type="button"
                  class="style-profile-mini-action"
                  v-tooltip="t('common.rename')"
                  :aria-label="t('common.rename')"
                  :disabled="busy"
                  @click.stop="selectedStyleProfileId = profile.id; renameCurrentStyleProfile()"
                >
                  <svg class="mini-icon" viewBox="0 0 24 24" aria-hidden="true">
                    <path d="M12 20h9" />
                    <path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4Z" />
                  </svg>
                </button>
                <button
                  type="button"
                  class="style-profile-mini-action danger"
                  v-tooltip="t('common.delete')"
                  :aria-label="t('common.delete')"
                  :disabled="busy"
                  @click.stop="selectedStyleProfileId = profile.id; deleteCurrentStyleProfile()"
                >
                  <svg class="mini-icon" viewBox="0 0 24 24" aria-hidden="true">
                    <path d="M18 6 6 18M6 6l12 12" />
                  </svg>
                </button>
              </div>
            </div>
          </aside>

          <div class="style-profile-editor">
            <div class="style-profile-editor-head">
              <div>
                <strong>{{ currentStyleProfile?.name }}</strong>
                <span>{{ t('ccSubtitle.style.styleCount', { count: activeStyleNames.length }) }}</span>
              </div>
            </div>

            <div class="style-profile-fields">
              <label class="style-select style-select-control">
                <span>{{ t('ccSubtitle.style.speakStyle') }}</span>
                <AppSelect
                  v-model="speakStyleName"
                  :options="styleOptions"
                  :placeholder="t('common.unselected')"
                  :disabled="!activeStyleNames.length || busy"
                />
              </label>
              <label class="style-select style-select-control">
                <span>{{ t('ccSubtitle.style.screenStyle') }}</span>
                <AppSelect
                  v-model="screenStyleName"
                  :options="styleOptions"
                  :placeholder="t('common.unselected')"
                  :disabled="!activeStyleNames.length || busy"
                />
              </label>
            </div>
          </div>
        </div>

        <div v-else class="style-profile-empty">
          <strong>{{ t('ccSubtitle.style.noProfile') }}</strong>
          <p>{{ t('ccSubtitle.style.emptyDescription') }}</p>
          <button type="button" :disabled="busy" @click="createStyleProfileFromAss">{{ t('ccSubtitle.style.create') }}</button>
        </div>
      </section>
    </div>

    <div
      v-if="styleProfileNameOpen"
      class="style-profile-name-modal app-modal-active"
      role="presentation"
      @click.self="closeStyleProfileNameDialog"
    >
      <section class="style-profile-name-dialog" role="dialog" aria-modal="true" aria-labelledby="style-profile-name-title">
        <h2 id="style-profile-name-title">{{ styleProfileNameMode === 'create' ? t('ccSubtitle.style.create') : t('ccSubtitle.style.rename') }}</h2>
        <label class="style-profile-name-field">
          <span>{{ t('ccSubtitle.style.nameLabel') }}</span>
          <input
            v-model="styleProfileNameInput"
            type="text"
            autocomplete="off"
            @keydown.enter.prevent="confirmStyleProfileName"
          />
        </label>
        <div class="style-profile-name-actions">
          <button type="button" class="secondary" @click="closeStyleProfileNameDialog">{{ t('common.cancel') }}</button>
          <button type="button" :disabled="!styleProfileNameInput.trim()" @click="confirmStyleProfileName">{{ t('common.confirm') }}</button>
        </div>
      </section>
    </div>
  </section>
</template>

<style scoped>
.cc-subtitle-workspace {
  display: grid;
  height: 100%;
  min-height: 0;
  position: relative;
}

.cc-panel {
  box-sizing: border-box;
  display: grid;
  gap: 14px;
  grid-template-rows: auto minmax(0, 1fr);
  height: 100%;
  min-height: 0;
}

.cc-toolbar {
  align-items: flex-start;
  display: flex;
  gap: 16px;
  justify-content: space-between;
}

.cc-actions {
  align-items: stretch;
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  justify-content: flex-end;
}

.cc-summary-group {
  align-items: center;
  background: #f8fafb;
  border: 1px solid #dce5ec;
  border-radius: 8px;
  display: flex;
  gap: 18px;
  justify-content: space-between;
  min-height: 58px;
  padding: 10px 16px;
}

.style-summary-group {
  min-width: 340px;
}

.dictionary-group {
  min-width: 280px;
}

.cc-summary-group > div {
  display: grid;
  gap: 3px;
  min-width: 0;
}

.cc-summary-group strong {
  color: #102030;
  font-size: 15px;
  font-weight: 800;
  line-height: 1.25;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.cc-summary-group span {
  color: #667582;
  font-size: 12px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dictionary-row {
  align-items: center;
  display: flex;
  flex: 0 0 auto;
  gap: 6px;
}

.dictionary-group .switch-row {
  gap: 6px;
  font-size: 12px;
}

.dictionary-group .switch {
  height: 20px;
  padding: 2px;
  width: 36px;
}

.dictionary-group .switch::after {
  height: 16px;
  width: 16px;
}

.dictionary-group .switch-row input:checked + .switch::after {
  transform: translateX(16px);
}

.compact-action {
  font-size: 13px;
  min-height: 32px;
  padding: 0 12px;
}

.summary-action {
  border-radius: 7px;
  font-size: 12px;
  font-weight: 700;
  min-height: 28px;
  min-width: auto;
  padding: 0 10px;
  white-space: nowrap;
}

.danger-lite:not(:disabled) {
  color: #b42318;
}

.style-select {
  align-items: center;
  color: #667582;
  display: inline-flex;
  gap: 6px;
  font-size: 12px;
}

.style-select-control {
  min-width: 240px;
}

.style-select-control > span {
  flex: 0 0 auto;
  line-height: 1.2;
}

.style-select-control :deep(.app-select) {
  width: 160px;
}

.style-select-control :deep(.app-select-trigger) {
  background: #eef2f6;
  border-color: #dce5ec;
  border-radius: 8px;
  font-size: 13px;
  min-height: 32px;
  padding: 0 10px;
}

.style-profile-modal {
  align-items: center;
  background: rgba(15, 23, 32, 0.42);
  display: flex;
  inset: 0;
  justify-content: center;
  padding: 28px;
  position: fixed;
  z-index: 90;
}

.style-profile-dialog {
  background: #fff;
  border: 1px solid #dce3e8;
  border-radius: 8px;
  box-shadow: 0 20px 60px rgba(15, 23, 32, 0.22);
  display: grid;
  gap: 16px;
  max-height: min(680px, calc(100vh - 56px));
  max-width: min(860px, 100%);
  overflow: auto;
  padding: 18px;
  width: 860px;
}

.style-profile-dialog-head {
  align-items: flex-start;
  display: flex;
  gap: 16px;
  justify-content: space-between;
}

.style-profile-dialog-head h2 {
  margin: 0;
}

.style-profile-dialog-head p {
  color: #667582;
  font-size: 13px;
  line-height: 1.5;
  margin: 5px 0 0;
}

.close-action {
  flex: 0 0 auto;
}

.style-profile-manager {
  display: grid;
  gap: 14px;
  grid-template-columns: minmax(170px, 220px) minmax(0, 1fr);
}

.style-profile-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.style-profile-list-toolbar {
  display: flex;
}

.style-profile-list-row {
  position: relative;
}

.style-profile-list-item,
.style-profile-list-create {
  box-sizing: border-box;
  width: 100%;
}

.style-profile-list-create {
  align-items: center;
  background: #f8fafb;
  border: 1px solid #d8e2e8;
  border-radius: 6px;
  color: #176b87;
  cursor: pointer;
  display: flex;
  gap: 6px;
  font-size: 13px;
  font-weight: 800;
  justify-content: center;
  line-height: 1.25;
  min-height: 42px;
  padding: 0 14px;
  text-align: center;
}

.style-profile-list-create:hover:not(:disabled) {
  background: #f2f8fa;
  border-color: #8bb4c2;
}

.button-icon,
.mini-icon {
  fill: none;
  stroke: currentColor;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.button-icon {
  flex: 0 0 auto;
  height: 16px;
  stroke-width: 2.4;
  width: 16px;
}

.style-profile-list-item {
  background: #f8fafb;
  border: 1px solid #e3e9ed;
  border-radius: 6px;
  color: #18202a;
  display: grid;
  gap: 4px;
  min-height: 82px;
  padding: 8px 66px 8px 12px;
  text-align: left;
}

.style-profile-list-row.active .style-profile-list-item {
  background: #e8f4f8;
  border-color: #a8c8d2;
  color: #0f5268;
}

.style-profile-list-item strong,
.style-profile-list-item span,
.style-profile-list-item small {
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.style-profile-list-item strong {
  font-size: 14px;
  line-height: 1.25;
}

.style-profile-list-item span {
  color: #667582;
  font-size: 12px;
  line-height: 1.25;
}

.style-profile-list-item small {
  background: #eef3f6;
  border-radius: 999px;
  color: #5f6f7b;
  display: inline-block;
  font-size: 11px;
  font-weight: 650;
  justify-self: start;
  line-height: 1;
  margin-top: 5px;
  max-width: 100%;
  padding: 4px 7px;
  vertical-align: top;
}

.style-profile-list-row.active .style-profile-list-item small {
  background: #d7e9ef;
  color: #176b87;
}

.style-profile-list-actions {
  display: flex;
  gap: 4px;
  position: absolute;
  right: 7px;
  top: 50%;
  transform: translateY(-50%);
  z-index: 3;
}

.style-profile-mini-action {
  align-items: center;
  background: rgba(255, 255, 255, 0.92);
  border: 1px solid #d8e2e8;
  border-radius: 6px;
  color: #43515c;
  cursor: pointer;
  display: inline-flex;
  justify-content: center;
  min-height: 26px;
  opacity: 0;
  padding: 0;
  pointer-events: none;
  transition: opacity 0.12s ease, background 0.12s ease, color 0.12s ease, border-color 0.12s ease;
  width: 26px;
}

.style-profile-list-row:hover .style-profile-mini-action,
.style-profile-list-row:focus-within .style-profile-mini-action {
  opacity: 1;
  pointer-events: auto;
}

.style-profile-mini-action:hover:not(:disabled) {
  background: #f4f8fa;
  border-color: #176b87;
  color: #176b87;
}

.style-profile-mini-action.danger:hover:not(:disabled) {
  background: rgba(220, 38, 38, 0.08);
  border-color: #dc2626;
  color: #dc2626;
}

.style-profile-mini-action:disabled {
  cursor: not-allowed;
  opacity: 0.45;
}

.style-profile-list-row:hover .style-profile-mini-action:disabled,
.style-profile-list-row:focus-within .style-profile-mini-action:disabled {
  opacity: 0.45;
}

.mini-icon {
  height: 14px;
  stroke-width: 2.2;
  width: 14px;
}

.style-profile-editor {
  background: #f8fafb;
  border: 1px solid #dce5ec;
  border-radius: 8px;
  display: grid;
  gap: 16px;
  padding: 14px;
}

.style-profile-editor-head {
  align-items: center;
  display: flex;
  gap: 12px;
  justify-content: space-between;
}

.style-profile-editor-head > div:first-child {
  display: grid;
  gap: 4px;
  min-width: 0;
}

.style-profile-editor-head strong,
.style-profile-editor-head span {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.style-profile-editor-head span {
  color: #667582;
  font-size: 12px;
}

.style-profile-fields {
  align-items: center;
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.style-profile-empty {
  align-items: center;
  background: #f8fafb;
  border: 1px dashed #b9c9d3;
  border-radius: 8px;
  color: #24313c;
  display: grid;
  gap: 9px;
  justify-items: center;
  padding: 32px 18px;
  text-align: center;
}

.style-profile-empty p {
  color: #667582;
  font-size: 13px;
  line-height: 1.5;
  margin: 0;
}

.style-profile-name-modal {
  align-items: center;
  background: rgba(15, 23, 32, 0.42);
  display: flex;
  inset: 0;
  justify-content: center;
  padding: 24px;
  position: fixed;
  z-index: 100;
}

.style-profile-name-dialog {
  background: #fff;
  border: 1px solid #dce3e8;
  border-radius: 8px;
  box-shadow: 0 18px 54px rgba(15, 23, 32, 0.24);
  display: grid;
  gap: 14px;
  max-width: min(520px, 100%);
  padding: 18px;
  width: 520px;
}

.style-profile-name-dialog h2 {
  font-size: 18px;
  margin: 0;
}

.style-profile-name-field {
  color: #24313c;
  display: grid;
  gap: 7px;
  font-size: 13px;
}

.style-profile-name-field input {
  background: #fff;
  border: 1px solid #cfd9e0;
  border-radius: 8px;
  box-sizing: border-box;
  color: #102033;
  font-size: 15px;
  min-height: 40px;
  outline: none;
  padding: 0 11px;
}

.style-profile-name-field input:focus {
  border-color: #1a7895;
  box-shadow: 0 0 0 3px rgba(26, 120, 149, 0.16);
}

.style-profile-name-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
}

.cc-grid {
  display: grid;
  gap: 4px;
  grid-template-columns:
    minmax(280px, calc(var(--source-pane-percent, 50%) - 6px))
    12px
    minmax(300px, calc(var(--result-pane-percent, 50%) - 6px));
  height: 100%;
  min-height: 0;
}

.cc-field {
  display: grid;
  gap: 8px;
  grid-template-rows: auto minmax(0, 1fr);
  min-height: 0;
  min-width: 0;
}

.pane-resizer {
  align-self: stretch;
  background: transparent;
  border: 0;
  border-radius: 6px;
  cursor: col-resize;
  margin: 42px 0 0;
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
.cc-grid.resizing .pane-resizer::before {
  background: #176b87;
  opacity: 1;
  width: 3px;
}

.pane-resizer:focus-visible {
  outline: 2px solid rgba(23, 107, 135, 0.36);
  outline-offset: 2px;
}

.field-head {
  align-items: center;
  display: flex;
  justify-content: space-between;
  gap: 12px;
  min-height: 34px;
}

.field-head strong {
  color: #102030;
  font-size: 15px;
  font-weight: 800;
  line-height: 1.2;
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
  outline: none;
  overflow: auto;
  padding: 12px;
  resize: none;
  width: 100%;
}

.cc-field textarea,
.cc-result {
  overflow-x: hidden;
  overflow-wrap: anywhere;
  white-space: pre-wrap;
  word-break: break-word;
}

.cc-dictionary-dialog textarea {
  white-space: pre;
}

.cc-field textarea:read-only {
  background: #f9fbfc;
  caret-color: transparent;
  color: #18202a;
  cursor: default;
}

.cc-field textarea:focus {
  border-color: #d6dee5;
  box-shadow: none;
}

.cc-field textarea::placeholder {
  color: #667582;
  opacity: 1;
}

.cc-field textarea::selection,
.cc-result::selection {
  background: #fff0a8;
  color: #18202a;
}

.cc-field textarea,
.cc-result {
  box-sizing: border-box;
  height: 100%;
}

.cc-result {
  user-select: text;
}

.cc-status-summary {
  color: #667582;
  font-size: 13px;
  line-height: 1.5;
  margin: 6px 0 0;
  overflow-wrap: anywhere;
}

.cc-status-summary {
  margin-top: 2px;
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

@media (max-width: 960px) {
  .cc-grid {
    grid-template-columns: 1fr;
  }

  .pane-resizer {
    display: none;
  }

  .cc-toolbar,
  .cc-dictionary-dialog-head,
  .cc-dictionary-toolbar,
  .cc-dictionary-dialog-foot {
    align-items: stretch;
    flex-direction: column;
  }

  .cc-actions {
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
