<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { loadConfig, saveConfig } from '../api/config'
import type { AppConfig, OutputNameTemplate, VideoEncodePreset } from '../types'
import { DEFAULT_ENCODE_PRESETS, normalizeEncodePresets } from '../utils/encodePresets'
import {
  DEFAULT_OUTPUT_TEMPLATE,
  TEMPLATE_VARIABLES,
  normalizeOutputTemplates,
  renderOutputName,
} from '../utils/outputTemplates'

const appConfig = ref<AppConfig | null>(null)
const outputTemplates = ref<OutputNameTemplate[]>([])
const selectedTemplateId = ref('default')
const templateMessage = ref('')
const encodePresets = ref<VideoEncodePreset[]>([])
const selectedEncodePresetId = ref('balanced-x264')
const encodePresetMessage = ref('')
const patternInputRef = ref<HTMLInputElement | null>(null)
const patternCursor = ref<number | null>(null)

const selectedTemplate = computed(() => {
  return outputTemplates.value.find((item) => item.id === selectedTemplateId.value) ?? outputTemplates.value[0]
})

const selectedEncodePreset = computed(() => {
  return encodePresets.value.find((item) => item.id === selectedEncodePresetId.value) ?? encodePresets.value[0]
})

const templatePreview = computed(() => {
  const tpl = selectedTemplate.value
  if (!tpl) return ''
  return renderOutputName(tpl.pattern, {
    id: 'preview',
    videoPath: 'E:\\Videos\\250313 MCD CHACHACHA ZB1 CUT 1080P.mp4',
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
    || encodePresets.value.find((item) => item.isDefault)?.id
    || encodePresets.value[0]?.id
    || 'balanced-x264'
}

async function persistOutputTemplates(message = '命名模板已保存') {
  if (!appConfig.value) return
  const defaultId = outputTemplates.value.find((item) => item.isDefault)?.id ?? selectedTemplateId.value
  const next: AppConfig = {
    ...appConfig.value,
    outputTemplates: outputTemplates.value,
    defaultOutputTemplateId: defaultId,
    outputNameTemplate: outputTemplates.value.find((item) => item.id === defaultId)?.pattern
      ?? DEFAULT_OUTPUT_TEMPLATE.pattern,
  }
  await saveConfig(next)
  appConfig.value = next
  templateMessage.value = message
}

function newTemplate() {
  const tpl: OutputNameTemplate = {
    id: crypto.randomUUID(),
    name: `模板 ${outputTemplates.value.length + 1}`,
    pattern: DEFAULT_OUTPUT_TEMPLATE.pattern,
    outputDirMode: 'sameAsVideo',
  }
  outputTemplates.value = [...outputTemplates.value, tpl]
  selectedTemplateId.value = tpl.id
  void persistOutputTemplates('已新建命名模板')
}

function duplicateTemplate() {
  const source = selectedTemplate.value
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

function deleteTemplate() {
  const tpl = selectedTemplate.value
  if (!tpl || tpl.id === 'default') return
  if (!confirm(`删除模板「${tpl.name}」？此操作不会影响已经生成的输出路径。`)) return
  outputTemplates.value = outputTemplates.value.filter((item) => item.id !== tpl.id)
  selectedTemplateId.value = outputTemplates.value.find((item) => item.isDefault)?.id
    ?? outputTemplates.value[0]?.id
    ?? 'default'
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

function setOutputDirMode(value: string) {
  if (value === 'sameAsVideo' || value === 'fixed' || value === 'manual') {
    updateSelectedTemplate({ outputDirMode: value })
  }
}

function rememberPatternCursor(event: Event) {
  const input = event.target as HTMLInputElement
  patternCursor.value = input.selectionStart ?? input.value.length
}

async function persistEncodePresets(message = '压制预设已保存') {
  if (!appConfig.value) return
  const defaultId = encodePresets.value.find((item) => item.isDefault)?.id ?? selectedEncodePresetId.value
  const next: AppConfig = {
    ...appConfig.value,
    encodePresets: encodePresets.value,
    defaultEncodePresetId: defaultId,
  }
  await saveConfig(next)
  appConfig.value = next
  encodePresetMessage.value = message
}

function newEncodePreset() {
  const tpl: VideoEncodePreset = {
    id: crypto.randomUUID(),
    name: `压制预设 ${encodePresets.value.length + 1}`,
    encoder: 'libx264',
    crf: 18,
    customVideoArgs: '-preset slow -profile:v high -pix_fmt yuv420p',
  }
  encodePresets.value = [...encodePresets.value, tpl]
  selectedEncodePresetId.value = tpl.id
  void persistEncodePresets('已新建压制预设')
}

function duplicateEncodePreset() {
  const source = selectedEncodePreset.value
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

function deleteEncodePreset() {
  const tpl = selectedEncodePreset.value
  if (!tpl || encodePresets.value.length <= 1) return
  if (!confirm(`删除压制预设「${tpl.name}」？`)) return
  encodePresets.value = encodePresets.value.filter((item) => item.id !== tpl.id)
  selectedEncodePresetId.value = encodePresets.value.find((item) => item.isDefault)?.id
    ?? encodePresets.value[0]?.id
    ?? 'balanced-x264'
  void persistEncodePresets('已删除压制预设')
}

function setDefaultEncodePreset() {
  const id = selectedEncodePresetId.value
  encodePresets.value = encodePresets.value.map((item) => ({
    ...item,
    isDefault: item.id === id,
  }))
  void persistEncodePresets('已设为默认压制预设')
}

function resetEncodePresets() {
  encodePresets.value = DEFAULT_ENCODE_PRESETS.map((item) => ({ ...item }))
  selectedEncodePresetId.value = encodePresets.value.find((item) => item.isDefault)?.id ?? encodePresets.value[0].id
  void persistEncodePresets('已恢复内置压制预设')
}

function updateSelectedEncodePreset(patch: Partial<VideoEncodePreset>) {
  const id = selectedEncodePresetId.value
  encodePresets.value = encodePresets.value.map((item) =>
    item.id === id ? { ...item, ...patch } : item
  )
}

function setPresetEncoder(value: string) {
  if (
    value === 'libx264'
    || value === 'libx265'
    || value === 'h264_nvenc'
    || value === 'h264_amf'
    || value === 'h264_videotoolbox'
  ) {
    updateSelectedEncodePreset({ encoder: value })
  }
}

function parsePresetBitrate(value: string): number | undefined {
  const trimmed = value.trim()
  if (!trimmed) return undefined
  const n = Number(trimmed)
  if (!Number.isFinite(n) || n < 0) return undefined
  return Math.round(n)
}

onMounted(loadPresetConfig)
</script>

<template>
  <main class="workspace">
    <section class="panel template-panel encode-section">
      <div class="panel-heading">
        <div>
          <h2>压制预设</h2>
          <p>管理常用编码器、CRF、最大码率和高级 ffmpeg 视频参数；压制页可直接选择并应用。</p>
        </div>
        <button class="secondary" @click="newEncodePreset">新建预设</button>
      </div>

      <div class="template-manager" v-if="selectedEncodePreset">
        <aside class="template-list">
          <button
            v-for="preset in encodePresets"
            :key="preset.id"
            type="button"
            class="template-list-item"
            :class="{ active: preset.id === selectedEncodePresetId }"
            @click="selectedEncodePresetId = preset.id"
          >
            <strong>{{ preset.name }}</strong>
            <span>{{ preset.isDefault ? '默认预设' : `${preset.encoder} / CRF ${preset.crf}` }}</span>
          </button>
        </aside>

        <div class="template-editor">
          <label>
            <span>预设名称</span>
            <input
              :value="selectedEncodePreset.name"
              @input="updateSelectedEncodePreset({ name: ($event.target as HTMLInputElement).value })"
            />
          </label>

          <div class="preset-fields">
            <label>
              <span>编码器</span>
              <select
                :value="selectedEncodePreset.encoder"
                @change="setPresetEncoder(($event.target as HTMLSelectElement).value)"
              >
                <option value="libx264">libx264</option>
                <option value="libx265">libx265</option>
                <option value="h264_nvenc">h264_nvenc</option>
                <option value="h264_amf">h264_amf</option>
                <option value="h264_videotoolbox">h264_videotoolbox</option>
              </select>
            </label>
            <label>
              <span>CRF / CQ</span>
              <input
                type="number"
                min="0"
                max="51"
                :value="selectedEncodePreset.crf"
                @input="updateSelectedEncodePreset({ crf: Number(($event.target as HTMLInputElement).value) })"
              />
            </label>
            <label>
              <span>最大码率 Kbps</span>
              <input
                type="number"
                min="0"
                :value="selectedEncodePreset.maxBitrate ?? ''"
                placeholder="留空为不限制，0 为自动"
                @input="updateSelectedEncodePreset({ maxBitrate: parsePresetBitrate(($event.target as HTMLInputElement).value) })"
              />
            </label>
          </div>

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
            <button class="secondary" @click="setDefaultEncodePreset">设为默认</button>
            <button class="secondary" @click="duplicateEncodePreset">复制</button>
            <button class="secondary" @click="resetEncodePresets">恢复内置</button>
            <button class="secondary danger-lite" :disabled="encodePresets.length <= 1" @click="deleteEncodePreset">删除</button>
          </div>
          <p v-if="encodePresetMessage" class="notice">{{ encodePresetMessage }}</p>
        </div>
      </div>
    </section>

    <section class="panel template-panel output-section">
      <div class="panel-heading">
        <div>
          <h2>输出命名模板</h2>
          <p>建立常用命名规则，在压制页选择模板后可一键套用到输出路径。</p>
        </div>
        <button class="secondary" @click="newTemplate">新建模板</button>
      </div>

      <div class="template-manager" v-if="selectedTemplate">
        <aside class="template-list">
          <button
            v-for="tpl in outputTemplates"
            :key="tpl.id"
            type="button"
            class="template-list-item"
            :class="{ active: tpl.id === selectedTemplateId }"
            @click="selectedTemplateId = tpl.id"
          >
            <strong>{{ tpl.name }}</strong>
            <span>{{ tpl.isDefault ? '默认模板' : tpl.pattern }}</span>
          </button>
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

          <div class="variable-row">
            <button
              v-for="item in TEMPLATE_VARIABLES"
              :key="item.key"
              type="button"
              class="secondary variable-token"
              :title="`${item.label}，示例：${item.sample}`"
              @click="insertVariable(item.key)"
            >
              {{ item.key }}
            </button>
          </div>

          <label>
            <span>输出目录</span>
            <select
              :value="selectedTemplate.outputDirMode"
              @change="setOutputDirMode(($event.target as HTMLSelectElement).value)"
            >
              <option value="sameAsVideo">跟随视频目录</option>
              <option value="manual">沿用当前输出框目录</option>
              <option value="fixed">固定目录</option>
            </select>
          </label>

          <label v-if="selectedTemplate.outputDirMode === 'fixed'">
            <span>固定目录</span>
            <input
              :value="selectedTemplate.fixedOutputDir ?? ''"
              placeholder="E:\Output"
              @input="updateSelectedTemplate({ fixedOutputDir: ($event.target as HTMLInputElement).value })"
            />
          </label>

          <div class="template-preview">
            <span>示例预览</span>
            <code>{{ templatePreview }}</code>
          </div>

          <div class="actions left">
            <button @click="persistOutputTemplates()">保存模板</button>
            <button class="secondary" @click="setDefaultTemplate">设为默认</button>
            <button class="secondary" @click="duplicateTemplate">复制</button>
            <button class="secondary danger-lite" :disabled="selectedTemplate.id === 'default'" @click="deleteTemplate">删除</button>
          </div>
          <p v-if="templateMessage" class="notice">{{ templateMessage }}</p>
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
.template-manager {
  display: grid;
  gap: 14px;
  grid-template-columns: minmax(180px, 240px) 1fr;
}
.template-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.template-list-item {
  background: #f8fafb;
  border: 1px solid #e3e9ed;
  border-radius: 6px;
  color: #18202a;
  cursor: pointer;
  padding: 9px 10px;
  text-align: left;
}
.template-list-item.active {
  background: #e8f4f8;
  border-color: #a8c8d2;
}
.template-list-item strong,
.template-list-item span {
  display: block;
}
.template-list-item span {
  color: #687682;
  font-size: 12px;
  margin-top: 3px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
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
.template-editor select,
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
.preset-fields {
  display: grid;
  gap: 10px;
  grid-template-columns: repeat(3, minmax(0, 1fr));
}
.variable-row {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.variable-token {
  min-height: 28px;
  padding: 0 8px;
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
@media (max-width: 760px) {
  .template-manager,
  .preset-fields {
    grid-template-columns: 1fr;
  }
}
</style>
