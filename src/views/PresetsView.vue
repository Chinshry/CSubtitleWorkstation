<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { open, save } from '@tauri-apps/plugin-dialog'
import {
  exportEncodePresets,
  exportOutputTemplates,
  importEncodePresets,
  importOutputTemplates,
  loadConfig,
  saveConfig,
} from '../api/config'
import type { AppConfig, OutputNameTemplate, VideoEncodePreset } from '../types'
import { DEFAULT_ENCODE_PRESETS, normalizeEncodePresets } from '../utils/encodePresets'
import { useEncoderOptions } from '../composables/useEncoderOptions'
import { useToast } from '../composables/useToast'
import { currentVideoPath } from '../stores/currentJobStore'
import AppSelect from '../components/AppSelect.vue'
import EncodeSettingsFields, { type EncodeSettingsModel } from '../components/EncodeSettingsFields.vue'
import {
  DEFAULT_OUTPUT_TEMPLATE,
  TEMPLATE_VARIABLES,
  normalizeOutputTemplates,
  renderOutputName,
} from '../utils/outputTemplates'

const appConfig = ref<AppConfig | null>(null)
const outputTemplates = ref<OutputNameTemplate[]>([])
const selectedTemplateId = ref('default')
const encodePresets = ref<VideoEncodePreset[]>([])
const selectedEncodePresetId = ref('balanced-x264')
const draggedTemplateId = ref<string | null>(null)
const draggedEncodePresetId = ref<string | null>(null)
const hoveredTemplateId = ref<string | null>(null)
const hoveredEncodePresetId = ref<string | null>(null)
const patternInputRef = ref<HTMLInputElement | null>(null)
const patternCursor = ref<number | null>(null)
const { encoderOptions, loadEncoderOptions } = useEncoderOptions()
const toast = useToast()

const outputDirModeOptions = [
  {
    value: 'sameAsVideo',
    label: '跟随视频目录',
    description: '输出到源视频所在文件夹',
  },
  {
    value: 'fixed',
    label: '固定目录',
    description: '始终输出到你选择的文件夹',
  },
]

const selectedTemplate = computed(() => {
  return outputTemplates.value.find((item) => item.id === selectedTemplateId.value) ?? outputTemplates.value[0]
})

const selectedEncodePreset = computed(() => {
  return encodePresets.value.find((item) => item.id === selectedEncodePresetId.value) ?? encodePresets.value[0]
})

const selectedEncodeSettings = computed<EncodeSettingsModel>({
  get() {
    return selectedEncodePreset.value ?? {
      encoder: 'libx264',
      crf: 18,
      maxBitrate: undefined,
    }
  },
  set(value) {
    updateSelectedEncodePreset({
      encoder: value.encoder,
      crf: value.crf,
      maxBitrate: value.maxBitrate,
    })
  },
})

const selectedOutputDirMode = computed({
  get() {
    const mode = selectedTemplate.value?.outputDirMode
    return mode === 'fixed' ? 'fixed' : 'sameAsVideo'
  },
  set(value: string | number) {
    void setOutputDirMode(String(value))
  },
})

const templatePreview = computed(() => {
  const tpl = selectedTemplate.value
  if (!tpl) return ''
  return renderOutputName(tpl.pattern, {
    id: 'preview',
    videoPath: currentVideoPath.value || 'E:\\Videos\\预览测试.mp4',
    subtitlePath: '',
    outputPath: '',
    crf: 18,
    needLogo: false,
    needYadif: false,
    encoder: 'libx264',
  }, { height: 1080 })
})

async function loadPresetConfig() {
  appConfig.value = await loadConfig()
  outputTemplates.value = normalizeOutputTemplates(appConfig.value)
  selectedTemplateId.value = appConfig.value.defaultOutputTemplateId
    || outputTemplates.value.find((item) => item.isDefault)?.id
    || outputTemplates.value[0]?.id
    || 'default'
  encodePresets.value = normalizeEncodePresets(appConfig.value)
  selectedEncodePresetId.value = appConfig.value.defaultEncodePresetId
    || encodePresets.value[0]?.id
    || 'balanced-x264'
}

async function persistOutputTemplates(message = '命名模板已保存') {
  if (!appConfig.value) return
  const hasEmptyFixedDir = outputTemplates.value.some(
    (item) => item.outputDirMode === 'fixed' && !item.fixedOutputDir?.trim()
  )
  if (hasEmptyFixedDir) {
    toast.error('固定目录不能为空，请先选择目录')
    return
  }
  const defaultId = outputTemplates.value.find((item) => item.isDefault)?.id ?? selectedTemplateId.value
  const previous = appConfig.value
  const next: AppConfig = {
    ...previous,
    outputTemplates: outputTemplates.value,
    defaultOutputTemplateId: defaultId,
  }
  try {
    await saveConfig(next)
    appConfig.value = next
    toast.success(message)
  } catch (err) {
    // 保存失败：回滚内存到磁盘上的旧状态，避免 UI 显示已保存但实际未落盘
    outputTemplates.value = normalizeOutputTemplates(previous)
    if (!outputTemplates.value.some((item) => item.id === selectedTemplateId.value)) {
      selectedTemplateId.value = previous.defaultOutputTemplateId
        || outputTemplates.value[0]?.id
        || 'default'
    }
    toast.error(`保存失败：${err instanceof Error ? err.message : String(err)}`)
  }
}

function newTemplate() {
  const tpl: OutputNameTemplate = {
    id: crypto.randomUUID(),
    name: `模板 ${outputTemplates.value.length + 1}`,
    pattern: DEFAULT_OUTPUT_TEMPLATE.pattern,
    outputDirMode: 'sameAsVideo',
    isDefault: false,
  }
  outputTemplates.value = [...outputTemplates.value, tpl]
  selectedTemplateId.value = tpl.id
  void persistOutputTemplates('已新建命名模板')
}

function duplicateTemplate(id = selectedTemplateId.value) {
  const source = outputTemplates.value.find((item) => item.id === id)
  if (!source) return
  const tpl: OutputNameTemplate = {
    ...source,
    id: crypto.randomUUID(),
    name: `${source.name} 副本`,
    isDefault: false,
  }
  outputTemplates.value = [...outputTemplates.value, tpl]
  selectedTemplateId.value = tpl.id
  void persistOutputTemplates('已复制命名模板')
}

function deleteTemplate(id = selectedTemplateId.value) {
  const tpl = outputTemplates.value.find((item) => item.id === id)
  if (!tpl || tpl.id === 'default') return
  if (!confirm(`删除模板「${tpl.name}」？此操作不会影响已经生成的输出路径。`)) return
  outputTemplates.value = outputTemplates.value.filter((item) => item.id !== tpl.id)
  if (selectedTemplateId.value === tpl.id) {
    selectedTemplateId.value = outputTemplates.value.find((item) => item.isDefault)?.id
      ?? outputTemplates.value[0]?.id
      ?? 'default'
  }
  void persistOutputTemplates('已删除命名模板')
}

function setDefaultTemplate() {
  const id = selectedTemplateId.value
  outputTemplates.value = outputTemplates.value.map((item) => ({
    ...item,
    isDefault: item.id === id,
  }))
  void persistOutputTemplates('已设为默认命名模板')
}

function updateSelectedTemplate(patch: Partial<OutputNameTemplate>) {
  const id = selectedTemplateId.value
  outputTemplates.value = outputTemplates.value.map((item) =>
    item.id === id ? { ...item, ...patch } : item
  )
}

function insertVariable(key: string) {
  const tpl = selectedTemplate.value
  if (!tpl) return
  const input = patternInputRef.value
  const start = input?.selectionStart ?? patternCursor.value ?? tpl.pattern.length
  const end = input?.selectionEnd ?? patternCursor.value ?? start
  const next = `${tpl.pattern.slice(0, start)}${key}${tpl.pattern.slice(end)}`
  const cursor = start + key.length
  updateSelectedTemplate({ pattern: next })
  patternCursor.value = cursor
  requestAnimationFrame(() => {
    patternInputRef.value?.focus()
    patternInputRef.value?.setSelectionRange(cursor, cursor)
  })
}

async function setOutputDirMode(value: string) {
  if (value === 'sameAsVideo') {
    updateSelectedTemplate({ outputDirMode: value })
    return
  }
  if (value === 'fixed') {
    if (selectedTemplate.value?.fixedOutputDir?.trim()) {
      updateSelectedTemplate({ outputDirMode: 'fixed' })
      return
    }
    const fixedOutputDir = await chooseFixedOutputDirPath()
    if (!fixedOutputDir) {
      toast.error('固定目录不能为空，请先选择目录')
      return
    }
    updateSelectedTemplate({ outputDirMode: 'fixed', fixedOutputDir })
  }
}

async function chooseFixedOutputDir() {
  const path = await chooseFixedOutputDirPath()
  if (!path) return
  updateSelectedTemplate({
    outputDirMode: 'fixed',
    fixedOutputDir: path,
  })
}

async function chooseFixedOutputDirPath() {
  const selected = await open({
    title: '选择固定输出目录',
    directory: true,
    multiple: false,
  })
  const path = Array.isArray(selected) ? selected[0] : selected
  return typeof path === 'string' && path ? path : null
}

function rememberPatternCursor(event: Event) {
  const input = event.target as HTMLInputElement
  patternCursor.value = input.selectionStart ?? input.value.length
}

async function persistEncodePresets(message = '压制预设已保存') {
  if (!appConfig.value) return
  const selectedId = encodePresets.value.some((item) => item.id === selectedEncodePresetId.value)
    ? selectedEncodePresetId.value
    : encodePresets.value[0]?.id
  const previous = appConfig.value
  const next: AppConfig = {
    ...previous,
    encodePresets: encodePresets.value,
    defaultEncodePresetId: selectedId,
  }
  try {
    await saveConfig(next)
    appConfig.value = next
    toast.success(message)
  } catch (err) {
    // 保存失败：回滚内存到磁盘上的旧状态，避免 UI 显示已保存但实际未落盘
    encodePresets.value = normalizeEncodePresets(previous)
    if (!encodePresets.value.some((item) => item.id === selectedEncodePresetId.value)) {
      selectedEncodePresetId.value = previous.defaultEncodePresetId
        || encodePresets.value[0]?.id
        || 'balanced-x264'
    }
    toast.error(`保存失败：${err instanceof Error ? err.message : String(err)}`)
  }
}

function startTemplateDrag(id: string, event: PointerEvent) {
  event.preventDefault()
  draggedTemplateId.value = id
  hoveredTemplateId.value = id
  document.addEventListener('pointermove', moveTemplateDrag)
  document.addEventListener('pointerup', finishTemplateDrag, { once: true })
}

function moveTemplateDrag(event: PointerEvent) {
  hoveredTemplateId.value = findRowIdAtPoint(event, 'template')
}

function finishTemplateDrag() {
  document.removeEventListener('pointermove', moveTemplateDrag)
  const draggedId = draggedTemplateId.value
  const targetId = hoveredTemplateId.value
  if (draggedId && targetId && draggedId !== targetId) {
    outputTemplates.value = reorderById(outputTemplates.value, draggedId, targetId)
    void persistOutputTemplates('已调整命名模板顺序')
  }
  draggedTemplateId.value = null
  hoveredTemplateId.value = null
}

function cancelTemplateDrag() {
  document.removeEventListener('pointermove', moveTemplateDrag)
  draggedTemplateId.value = null
  hoveredTemplateId.value = null
}

async function exportOutputTemplateFile() {
  const path = await save({
    title: '导出输出命名模板',
    defaultPath: 'output-templates.json',
    filters: [{ name: 'JSON', extensions: ['json'] }],
  })
  if (!path) return
  await exportOutputTemplates(path, outputTemplates.value)
  toast.success('输出命名模板已导出')
}

async function importOutputTemplateFile() {
  const selected = await open({
    title: '导入输出命名模板',
    multiple: false,
    filters: [{ name: 'JSON', extensions: ['json'] }],
  })
  const path = Array.isArray(selected) ? selected[0] : selected
  if (!path) return
  const imported = sanitizeImportedOutputTemplates(await importOutputTemplates(path))
  if (!imported.length) {
    toast.warning('未找到可导入的输出命名模板')
    return
  }
  const importedById = new Map(imported.map((item) => [item.id, item]))
  outputTemplates.value = [
    ...outputTemplates.value.map((item) => importedById.get(item.id) ?? item),
    ...imported.filter((item) => !outputTemplates.value.some((current) => current.id === item.id)),
  ]
  selectedTemplateId.value = imported[0].id
  void persistOutputTemplates(`已导入 ${imported.length} 个输出命名模板`)
}

async function exportEncodePresetFile() {
  const path = await save({
    title: '导出压制预设',
    defaultPath: 'encode-presets.json',
    filters: [{ name: 'JSON', extensions: ['json'] }],
  })
  if (!path) return
  await exportEncodePresets(path, encodePresets.value)
  toast.success('压制预设已导出')
}

async function importEncodePresetFile() {
  const selected = await open({
    title: '导入压制预设',
    multiple: false,
    filters: [{ name: 'JSON', extensions: ['json'] }],
  })
  const path = Array.isArray(selected) ? selected[0] : selected
  if (!path) return
  const imported = sanitizeImportedEncodePresets(await importEncodePresets(path))
  if (!imported.length) {
    toast.warning('未找到可导入的压制预设')
    return
  }
  const importedById = new Map(imported.map((item) => [item.id, item]))
  encodePresets.value = [
    ...encodePresets.value.map((item) => importedById.get(item.id) ?? item),
    ...imported.filter((item) => !encodePresets.value.some((current) => current.id === item.id)),
  ]
  selectedEncodePresetId.value = imported[0].id
  void persistEncodePresets(`已导入 ${imported.length} 个压制预设`)
}

function newEncodePreset() {
  const tpl: VideoEncodePreset = {
    id: crypto.randomUUID(),
    name: `压制预设 ${encodePresets.value.length + 1}`,
    encoder: 'libx264',
    crf: 18,
    customVideoArgs: '-preset slow -profile:v high -pix_fmt yuv420p',
    isDefault: false,
  }
  encodePresets.value = [...encodePresets.value, tpl]
  selectedEncodePresetId.value = tpl.id
  void persistEncodePresets('已新建压制预设')
}

function duplicateEncodePreset(id = selectedEncodePresetId.value) {
  const source = encodePresets.value.find((item) => item.id === id)
  if (!source) return
  const tpl: VideoEncodePreset = {
    ...source,
    id: crypto.randomUUID(),
    name: `${source.name} 副本`,
    isDefault: false,
  }
  encodePresets.value = [...encodePresets.value, tpl]
  selectedEncodePresetId.value = tpl.id
  void persistEncodePresets('已复制压制预设')
}

function deleteEncodePreset(id = selectedEncodePresetId.value) {
  const tpl = encodePresets.value.find((item) => item.id === id)
  if (!tpl || encodePresets.value.length <= 1) return
  if (!confirm(`删除压制预设「${tpl.name}」？`)) return
  encodePresets.value = encodePresets.value.filter((item) => item.id !== tpl.id)
  if (selectedEncodePresetId.value === tpl.id) {
    selectedEncodePresetId.value = encodePresets.value[0]?.id
      ?? 'balanced-x264'
  }
  void persistEncodePresets('已删除压制预设')
}

function startEncodePresetDrag(id: string, event: PointerEvent) {
  event.preventDefault()
  draggedEncodePresetId.value = id
  hoveredEncodePresetId.value = id
  document.addEventListener('pointermove', moveEncodePresetDrag)
  document.addEventListener('pointerup', finishEncodePresetDrag, { once: true })
}

function moveEncodePresetDrag(event: PointerEvent) {
  hoveredEncodePresetId.value = findRowIdAtPoint(event, 'encode')
}

function finishEncodePresetDrag() {
  document.removeEventListener('pointermove', moveEncodePresetDrag)
  const draggedId = draggedEncodePresetId.value
  const targetId = hoveredEncodePresetId.value
  if (draggedId && targetId && draggedId !== targetId) {
    encodePresets.value = reorderById(encodePresets.value, draggedId, targetId)
    void persistEncodePresets('已调整压制预设顺序')
  }
  draggedEncodePresetId.value = null
  hoveredEncodePresetId.value = null
}

function cancelEncodePresetDrag() {
  document.removeEventListener('pointermove', moveEncodePresetDrag)
  draggedEncodePresetId.value = null
  hoveredEncodePresetId.value = null
}

function resetEncodePresets() {
  if (!confirm('恢复内置压制预设将重置 5 个内置预设（x264 平衡 / NVENC 快速 / AMF 快速 / Apple 快速 / x265 体积优先）的参数，您自定义新增的预设不会被删除。是否继续？')) return
  const builtInIds = new Set(DEFAULT_ENCODE_PRESETS.map((item) => item.id))
  const customPresets = encodePresets.value.filter((item) => !builtInIds.has(item.id))
  encodePresets.value = [
    ...DEFAULT_ENCODE_PRESETS.map((item) => ({ ...item })),
    ...customPresets,
  ]
  if (!encodePresets.value.some((item) => item.id === selectedEncodePresetId.value)) {
    selectedEncodePresetId.value = encodePresets.value[0].id
  }
  void persistEncodePresets('已恢复内置压制预设（自定义预设保留）')
}

function updateSelectedEncodePreset(patch: Partial<VideoEncodePreset>) {
  const id = selectedEncodePresetId.value
  encodePresets.value = encodePresets.value.map((item) =>
    item.id === id ? { ...item, ...patch } : item
  )
}

function reorderById<T extends { id: string }>(items: T[], draggedId: string, targetId: string): T[] {
  const draggedIndex = items.findIndex((item) => item.id === draggedId)
  const targetIndex = items.findIndex((item) => item.id === targetId)
  if (draggedIndex < 0 || targetIndex < 0) return items
  const next = [...items]
  const [dragged] = next.splice(draggedIndex, 1)
  next.splice(targetIndex, 0, dragged)
  return next
}

function findRowIdAtPoint(event: PointerEvent, type: 'encode' | 'template') {
  const element = document.elementFromPoint(event.clientX, event.clientY)
  const row = element?.closest<HTMLElement>(`[data-${type}-id]`)
  return row?.dataset[type === 'encode' ? 'encodeId' : 'templateId'] ?? null
}

function sanitizeImportedOutputTemplates(items: OutputNameTemplate[]): OutputNameTemplate[] {
  const seen = new Set<string>()
  return items
    .filter((item) =>
      item &&
      typeof item.id === 'string' &&
      item.id.trim() &&
      typeof item.name === 'string' &&
      typeof item.pattern === 'string'
    )
    .filter((item) => {
      if (seen.has(item.id)) return false
      seen.add(item.id)
      return true
    })
    .map((item) => {
      const fixedOutputDir = typeof item.fixedOutputDir === 'string' && item.fixedOutputDir.trim()
        ? item.fixedOutputDir
        : undefined
      const outputDirMode: OutputNameTemplate['outputDirMode'] =
        item.outputDirMode === 'fixed' && fixedOutputDir ? 'fixed' : 'sameAsVideo'
      return {
        id: item.id,
        name: item.name.trim() || '未命名模板',
        pattern: item.pattern.trim() || DEFAULT_OUTPUT_TEMPLATE.pattern,
        outputDirMode,
        fixedOutputDir,
        isDefault: false,
      }
    })
}

function sanitizeImportedEncodePresets(items: VideoEncodePreset[]): VideoEncodePreset[] {
  const seen = new Set<string>()
  return items
    .filter((item) => item && typeof item.id === 'string' && typeof item.name === 'string')
    .filter((item) => {
      if (seen.has(item.id)) return false
      seen.add(item.id)
      return true
    })
    .map((item) => ({
      id: item.id,
      name: item.name,
      encoder: isSupportedPresetEncoder(item.encoder) ? item.encoder : 'libx264',
      crf: Number.isFinite(item.crf) ? Math.min(51, Math.max(0, Math.round(item.crf))) : 18,
      maxBitrate: typeof item.maxBitrate === 'number' && Number.isFinite(item.maxBitrate)
        ? Math.max(0, Math.round(item.maxBitrate))
        : undefined,
      customVideoArgs: item.customVideoArgs ?? '',
    }))
}

function isSupportedPresetEncoder(value: string): value is VideoEncodePreset['encoder'] {
  return value === 'libx264'
    || value === 'libx265'
    || value === 'h264_nvenc'
    || value === 'h264_amf'
    || value === 'h264_videotoolbox'
}

onMounted(async () => {
  await loadPresetConfig()
  try {
    await loadEncoderOptions()
  } catch (err) {
    console.error('Failed to get supported encoders:', err)
  }
})

onBeforeUnmount(() => {
  cancelTemplateDrag()
  cancelEncodePresetDrag()
})
</script>

<template>
  <main class="workspace">
    <section class="panel template-panel encode-section">
      <div class="panel-heading">
        <div>
          <h2>压制预设</h2>
          <p>管理常用编码器、CRF、最大码率和高级 ffmpeg 视频参数；压制页可直接选择并应用。</p>
        </div>
        <div class="panel-heading-actions">
          <button class="secondary" @click="importEncodePresetFile">批量导入</button>
          <button class="secondary" @click="exportEncodePresetFile">批量导出</button>
          <button class="secondary" @click="resetEncodePresets">恢复内置</button>
        </div>
      </div>

      <div class="template-manager" v-if="selectedEncodePreset">
        <aside class="template-list">
          <div class="template-list-toolbar">
            <button
              type="button"
              class="template-list-create"
              @click="newEncodePreset"
            >
              <svg class="button-icon" viewBox="0 0 24 24" aria-hidden="true">
                <path d="M12 5v14M5 12h14" />
              </svg>
              新建
            </button>
          </div>
          <div
            v-for="preset in encodePresets"
            :key="preset.id"
            class="template-list-row"
            :class="{
              active: preset.id === selectedEncodePresetId,
              dragging: preset.id === draggedEncodePresetId,
              over: preset.id === hoveredEncodePresetId && preset.id !== draggedEncodePresetId,
            }"
            :data-encode-id="preset.id"
          >
            <i
              class="drag-handle"
              aria-hidden="true"
              @click.stop
              @pointerdown.stop="startEncodePresetDrag(preset.id, $event)"
            >⋮⋮</i>
            <button
              type="button"
              class="template-list-item"
              @click="selectedEncodePresetId = preset.id"
            >
              <strong>{{ preset.name }}</strong>
              <small>{{ preset.encoder }}</small>
            </button>
            <div class="template-list-actions">
              <button
                type="button"
                class="mini-action"
                v-tooltip="'复制'"
                aria-label="复制"
                @click.stop="duplicateEncodePreset(preset.id)"
              >
                <svg class="mini-icon" viewBox="0 0 24 24" aria-hidden="true">
                  <path d="M8 8h10v12H8z" />
                  <path d="M6 16H4V4h12v2" />
                </svg>
              </button>
              <button
                type="button"
                class="mini-action danger"
                v-tooltip="'删除'"
                aria-label="删除"
                :disabled="encodePresets.length <= 1"
                @click.stop="deleteEncodePreset(preset.id)"
              >
                <svg class="mini-icon" viewBox="0 0 24 24" aria-hidden="true">
                  <path d="M18 6 6 18M6 6l12 12" />
                </svg>
              </button>
            </div>
          </div>
        </aside>

        <div class="template-editor">
          <label>
            <span>预设名称</span>
            <input
              :value="selectedEncodePreset.name"
              @input="updateSelectedEncodePreset({ name: ($event.target as HTMLInputElement).value })"
            />
          </label>

          <EncodeSettingsFields
            v-model="selectedEncodeSettings"
            :encoder-options="encoderOptions"
          />

          <label>
            <span>高级 ffmpeg 视频参数</span>
            <textarea
              :value="selectedEncodePreset.customVideoArgs ?? ''"
              rows="3"
              spellcheck="false"
              placeholder="-preset slow -profile:v high -level 4.1 -pix_fmt yuv420p"
              @input="updateSelectedEncodePreset({ customVideoArgs: ($event.target as HTMLTextAreaElement).value })"
            ></textarea>
          </label>

          <div class="template-preview">
            <span>应用后会覆盖</span>
            <code>
              编码器 {{ selectedEncodePreset.encoder }} / CRF {{ selectedEncodePreset.crf }} /
              码率 {{ selectedEncodePreset.maxBitrate === undefined ? '不限制' : selectedEncodePreset.maxBitrate === 0 ? '自动' : `${selectedEncodePreset.maxBitrate} Kbps` }}
            </code>
          </div>

          <div class="actions left">
            <button @click="persistEncodePresets()">保存预设</button>
          </div>
        </div>
      </div>
    </section>

    <section class="panel template-panel output-section">
      <div class="panel-heading">
        <div>
          <h2>输出命名模板</h2>
          <p>建立常用命名规则，在压制页选择模板后可一键套用到输出路径。</p>
        </div>
        <div class="panel-heading-actions">
          <button class="secondary" @click="importOutputTemplateFile">批量导入</button>
          <button class="secondary" @click="exportOutputTemplateFile">批量导出</button>
        </div>
      </div>

      <div class="template-manager" v-if="selectedTemplate">
        <aside class="template-list">
          <div class="template-list-toolbar">
            <button
              type="button"
              class="template-list-create"
              @click="newTemplate"
            >
              <svg class="button-icon" viewBox="0 0 24 24" aria-hidden="true">
                <path d="M12 5v14M5 12h14" />
              </svg>
              新建
            </button>
          </div>
          <div
            v-for="tpl in outputTemplates"
            :key="tpl.id"
            class="template-list-row"
            :class="{
              active: tpl.id === selectedTemplateId,
              dragging: tpl.id === draggedTemplateId,
              over: tpl.id === hoveredTemplateId && tpl.id !== draggedTemplateId,
            }"
            :data-template-id="tpl.id"
          >
            <i
              class="drag-handle"
              aria-hidden="true"
              @click.stop
              @pointerdown.stop="startTemplateDrag(tpl.id, $event)"
            >⋮⋮</i>
            <button
              type="button"
              class="template-list-item"
              @click="selectedTemplateId = tpl.id"
            >
              <strong class="output-template-title">
                <span>{{ tpl.name }}</span>
              </strong>
            </button>
            <div class="template-list-actions">
              <button
                type="button"
                class="mini-action"
                v-tooltip="'复制'"
                aria-label="复制"
                @click.stop="duplicateTemplate(tpl.id)"
              >
                <svg class="mini-icon" viewBox="0 0 24 24" aria-hidden="true">
                  <path d="M8 8h10v12H8z" />
                  <path d="M6 16H4V4h12v2" />
                </svg>
              </button>
              <button
                type="button"
                class="mini-action danger"
                v-tooltip="'删除'"
                aria-label="删除"
                :disabled="tpl.id === 'default'"
                @click.stop="deleteTemplate(tpl.id)"
              >
                <svg class="mini-icon" viewBox="0 0 24 24" aria-hidden="true">
                  <path d="M18 6 6 18M6 6l12 12" />
                </svg>
              </button>
            </div>
          </div>
        </aside>

        <div class="template-editor">
          <label>
            <span>模板名称</span>
            <input
              :value="selectedTemplate.name"
              @input="updateSelectedTemplate({ name: ($event.target as HTMLInputElement).value })"
            />
          </label>

          <label>
            <span>文件名模板</span>
            <input
              ref="patternInputRef"
              :value="selectedTemplate.pattern"
              @input="updateSelectedTemplate({ pattern: ($event.target as HTMLInputElement).value })"
              @click="rememberPatternCursor"
              @keyup="rememberPatternCursor"
              @select="rememberPatternCursor"
              @focus="rememberPatternCursor"
            />
          </label>

          <div class="variable-toolbar" aria-label="可插入的文件名变量">
            <div class="variable-toolbar-title">
              <span>插入变量</span>
              <small>点击后插入到光标位置</small>
            </div>
            <div class="variable-row">
              <button
                v-for="item in TEMPLATE_VARIABLES"
                :key="item.key"
                type="button"
                class="variable-token"
                @click="insertVariable(item.key)"
              >
                <span class="variable-token-key">{{ item.key }}</span>
                <span class="variable-token-meta">{{ item.label }} · {{ item.sample }}</span>
              </button>
            </div>
          </div>

          <label>
            <span>输出目录</span>
            <AppSelect
              v-model="selectedOutputDirMode"
              class="output-dir-select"
              :options="outputDirModeOptions"
            />
          </label>

          <label v-if="selectedTemplate.outputDirMode === 'fixed'">
            <span>固定目录</span>
            <div class="fixed-dir-row">
              <input
                :value="selectedTemplate.fixedOutputDir ?? ''"
                placeholder="请点击右侧按钮选择目录"
                readonly
              />
              <button
                type="button"
                class="secondary fixed-dir-pick"
                @click="chooseFixedOutputDir"
              >
                选择目录
              </button>
            </div>
            <small class="field-hint">固定目录必须通过系统目录选择器设置</small>
          </label>

          <div class="template-preview">
            <span>示例预览</span>
            <code>{{ templatePreview }}</code>
          </div>

          <div class="actions left">
            <button @click="persistOutputTemplates()">保存模板</button>
            <button class="secondary" @click="setDefaultTemplate">设为默认</button>
          </div>
        </div>
      </div>
    </section>
  </main>
</template>

<style scoped>
.output-section {
  order: 1;
}
.encode-section {
  order: 2;
}
.panel-heading {
  align-items: flex-start;
  gap: 12px;
}
.panel-heading-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  justify-content: flex-end;
}
.template-manager {
  display: grid;
  gap: 14px;
  grid-template-columns: minmax(156px, 200px) 1fr;
}
.template-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.template-list-toolbar {
  display: flex;
}
.template-list-row {
  position: relative;
}
.template-list-item,
.template-list-create {
  box-sizing: border-box;
  min-height: 66px;
  width: 100%;
}
.template-list-item {
  background: #f8fafb;
  border: 1px solid #e3e9ed;
  border-radius: 6px;
  color: #18202a;
  cursor: pointer;
  /* 左侧给 drag-handle 让位（22px），右侧给 hover 出现的复制/删除按钮预留空间（66px）。
     按钮 hover 才显形但宽度始终保留，避免 hover 时文字布局抖动。 */
  padding: 8px 66px 8px 22px;
  text-align: left;
}
.template-list-create {
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
.template-list-create:hover {
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
.template-list-row.active .template-list-item {
  background: #e8f4f8;
  border-color: #a8c8d2;
}
.template-list-row.over .template-list-item {
  border-color: #176b87;
  box-shadow: inset 3px 0 0 #176b87;
}
.template-list-row.dragging .template-list-item {
  opacity: 0.58;
}
.template-list-item strong,
.template-list-item small {
  display: block;
}
.template-list-item strong {
  font-size: 14px;
  line-height: 1.25;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.output-template-title {
  align-items: center;
  display: flex !important;
  gap: 7px;
  min-width: 0;
}
.output-template-title span {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.template-list-item small {
  background: #eef3f6;
  border-radius: 999px;
  color: #5f6f7b;
  display: inline-block;
  font-size: 11px;
  font-weight: 650;
  line-height: 1;
  margin-top: 7px;
  overflow: hidden;
  padding: 4px 7px;
  text-overflow: ellipsis;
  vertical-align: top;
  white-space: nowrap;
  max-width: 100%;
}
.template-list-row.active .template-list-item small {
  background: #d7e9ef;
  color: #176b87;
}
.drag-handle {
  color: #80919e;
  cursor: grab;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-style: normal;
  font-weight: 800;
  letter-spacing: 0;
  line-height: 1;
  min-height: 20px;
  position: absolute;
  left: 7px;
  top: 50%;
  transform: translateY(-50%);
  user-select: none;
  width: 12px;
  z-index: 2;
}
.drag-handle:active {
  cursor: grabbing;
}
.template-list-actions {
  display: flex;
  gap: 4px;
  position: absolute;
  right: 7px;
  top: 50%;
  transform: translateY(-50%);
  z-index: 3;
}
.mini-action {
  align-items: center;
  background: rgba(255, 255, 255, 0.92);
  border: 1px solid #d8e2e8;
  border-radius: 6px;
  color: #43515c;
  cursor: pointer;
  display: inline-flex;
  font-size: 11px;
  font-weight: 700;
  justify-content: center;
  min-height: 26px;
  opacity: 0;
  padding: 0;
  pointer-events: none;
  transition: opacity 0.12s ease, background 0.12s ease, color 0.12s ease, border-color 0.12s ease;
  width: 26px;
}
.template-list-row:hover .mini-action,
.template-list-row:focus-within .mini-action {
  opacity: 1;
  pointer-events: auto;
}
.mini-action:hover {
  background: #f4f8fa;
  border-color: #176b87;
  color: #176b87;
}
.mini-action.danger:hover:not(:disabled) {
  background: rgba(220, 38, 38, 0.08);
  border-color: #dc2626;
  color: #dc2626;
}
.mini-action:disabled {
  cursor: not-allowed;
  opacity: 0.45;
}
.template-list-row:hover .mini-action:disabled,
.template-list-row:focus-within .mini-action:disabled {
  opacity: 0.45;
}
.mini-icon {
  height: 14px;
  stroke-width: 2;
  width: 14px;
}
.template-editor {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.template-editor label {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.template-editor label > span,
.template-preview > span {
  color: #43515c;
  font-size: 12.5px;
  font-weight: 600;
}
.template-editor input,
.template-editor textarea {
  background: #fff;
  border: 1px solid #d6e0e6;
  border-radius: 6px;
  color: #18202a;
  font: inherit;
  min-height: 34px;
  padding: 6px 9px;
}
.template-editor textarea {
  font: 12.5px/1.5 ui-monospace, SFMono-Regular, Consolas, "Liberation Mono", monospace;
  min-height: 76px;
  resize: vertical;
}
.fixed-dir-row {
  display: grid;
  gap: 8px;
  grid-template-columns: minmax(0, 1fr) auto;
}
.fixed-dir-pick {
  min-height: 34px;
  padding: 0 14px;
  white-space: nowrap;
}
.field-hint {
  color: #697782;
  font-size: 12px;
  line-height: 1.4;
}
.preset-fields {
  display: grid;
  gap: 10px;
  grid-template-columns: repeat(3, minmax(0, 1fr));
}
.variable-toolbar {
  background: #f8fafb;
  border: 1px solid #dce6ec;
  border-radius: 7px;
  display: grid;
  gap: 8px;
  margin-top: -2px;
  padding: 9px 10px 10px;
}
.variable-toolbar-title {
  align-items: baseline;
  display: flex;
  gap: 8px;
}
.variable-toolbar-title span {
  color: #31424e;
  font-size: 12.5px;
  font-weight: 750;
}
.variable-toolbar-title small {
  color: #7a8894;
  font-size: 11.5px;
  font-weight: 500;
}
.variable-row {
  display: grid;
  gap: 7px;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
}
.variable-token {
  align-items: flex-start;
  background: #fff;
  border: 1px solid #d6e0e6;
  border-radius: 6px;
  color: #1f2d37;
  cursor: pointer;
  display: flex;
  flex-direction: column;
  gap: 3px;
  min-height: 48px;
  padding: 7px 10px;
  position: relative;
  text-align: left;
  transition: background 0.14s ease, border-color 0.14s ease, box-shadow 0.14s ease, transform 0.14s ease;
}
.variable-token::before {
  content: '+';
  color: #176b87;
  font-size: 15px;
  font-weight: 850;
  line-height: 1;
  position: absolute;
  transform: translate(-2px, 1px);
}
.variable-token-key {
  color: #18202a;
  font: 700 12.5px/1.25 "Cascadia Mono", Consolas, monospace;
  padding-left: 14px;
}
.variable-token-meta {
  color: #6a7884;
  font-size: 11px;
  font-weight: 600;
  line-height: 1.25;
  overflow: hidden;
  padding-left: 14px;
  text-overflow: ellipsis;
  white-space: nowrap;
  width: 100%;
}
.variable-token:hover,
.variable-token:focus-visible {
  background: #f2f8fa;
  border-color: #8bb4c2;
  box-shadow: 0 6px 14px rgba(23, 107, 135, 0.1);
  outline: none;
  transform: translateY(-1px);
}
.variable-token:active {
  transform: translateY(0);
}
.template-preview {
  background: #f8fafb;
  border: 1px solid #e3e9ed;
  border-radius: 6px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 9px 10px;
}
.template-preview code {
  color: #0f5268;
  white-space: normal;
  word-break: break-all;
}
.danger-lite {
  color: #b91c1c;
}
@media (max-width: 960px) {
  .template-manager,
  .preset-fields,
  .fixed-dir-row {
    grid-template-columns: 1fr;
  }
  .panel-heading-actions {
    justify-content: flex-start;
  }
}
</style>
