<script setup lang="ts">
import { computed, defineAsyncComponent, nextTick, ref, watch } from 'vue'
import {
  activeTool,
  isMediaToolId,
  type ToolId
} from '../stores/toolStore'
import InfoHint from '../components/InfoHint.vue'

const loadTextConversionView = () => import('./TextConversionView.vue')
const loadProofreadView = () => import('./ProofreadView.vue')
const loadCcSubtitleView = () => import('./CcSubtitleView.vue')
const loadSubtitleFormatView = () => import('./SubtitleFormatView.vue')
const loadMediaRemuxView = () => import('./MediaRemuxView.vue')

const ToolLoadingView = {
  template: '<div class="tool-loading"><span class="tool-loading-spinner"></span><strong>正在准备工具</strong><p>首次打开需要加载工具页面，完成后会自动显示。</p></div>'
}

const asyncToolOptions = {
  loadingComponent: ToolLoadingView,
  delay: 0,
  timeout: 20000
}

const TextConversionView = defineAsyncComponent({
  ...asyncToolOptions,
  loader: loadTextConversionView
})
const ProofreadView = defineAsyncComponent({
  ...asyncToolOptions,
  loader: loadProofreadView
})
const CcSubtitleView = defineAsyncComponent({
  ...asyncToolOptions,
  loader: loadCcSubtitleView
})
const SubtitleFormatView = defineAsyncComponent({
  ...asyncToolOptions,
  loader: loadSubtitleFormatView
})
const MediaRemuxView = defineAsyncComponent({
  ...asyncToolOptions,
  loader: loadMediaRemuxView
})

const ccRuleHintItems = [
  '先读取样式参考 ASS，解析 [V4+ Styles]；必须手动选择听轴样式和花字样式。',
  'SRT 输入会转换为 ASS 输出；ASS / SSA 输入会处理已有 Dialogue 行。',
  '遇到 [方括号标签]：括号内文本去掉 []，使用花字样式。',
  '方括号标签后面的台词会另起一条，使用听轴样式。',
  '没有方括号标签的普通台词整条使用听轴样式。',
  '台词只处理 \\N 换行和多余空格；',
  '启用自定义词库时，会按词库规则替换名称或固定写法。'
]

type ToolItem = {
  id: ToolId
  name: string
  description: string
}

const textTools: ToolItem[] = [
  {
    id: 'cc-subtitle',
    name: 'CC 字幕整理',
    description: '整理 CC 字幕，把 [] 内文字拆成花字行，其余整理为听轴行；支持读取参考 ASS 样式并导出 ASS。'
  },
  {
    id: 'text-conversion',
    name: '繁简转换',
    description: '使用 zhconv 转换繁简文本，自定义词库会优先保护和替换指定词条，适合字幕和普通文本批量处理。'
  },
  {
    id: 'proofread',
    name: '字幕校对',
    description: '使用 jieba-rs 分词与词性标注，检查“的 / 地 / 得”疑似误用；自定义词库会提示专有名词、艺人名和固定译名的统一写法。'
  }
]

const formatTools: ToolItem[] = [
  {
    id: 'subtitle-format',
    name: '字幕格式转换',
    description: '使用 ffmpeg 在 ASS / SSA / SRT / VTT 之间转换；转到 SRT / VTT 时会丢弃原格式不支持的样式和特效。'
  },
  {
    id: 'media-remux',
    name: '视频转 MP4',
    description: '把常见视频容器重新封装为 MP4，默认只复制音视频流，不重新编码；TS / M2TS / MTS 会自动整理 AAC 音频封装头。'
  }
]

const mediaTools: ToolItem[] = [
  {
    id: 'media-concat-ts',
    name: 'TS 分片合并',
    description: '按文件顺序合并 TS / M2TS / MTS 分片，可输出 MP4 或 TS；MP4 会自动整理 AAC 音频封装头，不重新编码。'
  },
  {
    id: 'media-merge-av',
    name: '合并音视频',
    description: '保留视频画面，合并单独的音频来源输出 MP4；适合替换或补齐外部音轨。'
  },
  {
    id: 'media-cover',
    name: '添加封面',
    description: '给 MP4 写入 JPG / PNG 封面，原视频和音频会原样复制，不重新编码。'
  }
]

const toolGroups = [
  {
    id: 'text',
    name: '文字处理',
    description: '字幕文本和词库类的轻量处理。',
    tools: textTools
  },
  {
    id: 'format',
    name: '格式转换',
    description: '字幕格式和容器格式转换入口。',
    tools: formatTools
  },
  {
    id: 'media',
    name: '媒体处理',
    description: '不重新压制的媒体文件辅助操作。',
    tools: mediaTools
  }
]

const allTools = [...textTools, ...formatTools, ...mediaTools]
const selectedTool = ref<ToolId>(activeTool.value)
const renderedTool = ref<ToolId>(activeTool.value)
const preparingTool = ref(false)
let toolSwitchSeq = 0

const activeToolMeta = computed(() => allTools.find((tool) => tool.id === selectedTool.value) ?? textTools[0])

const activeToolComponent = computed(() => (
  isMediaToolId(renderedTool.value)
    ? MediaRemuxView
    : renderedTool.value === 'text-conversion'
      ? TextConversionView
      : renderedTool.value === 'cc-subtitle'
        ? CcSubtitleView
        : renderedTool.value === 'subtitle-format'
          ? SubtitleFormatView
        : ProofreadView
))

function loadToolView(tool: ToolId) {
  if (isMediaToolId(tool)) return loadMediaRemuxView()
  if (tool === 'text-conversion') return loadTextConversionView()
  if (tool === 'cc-subtitle') return loadCcSubtitleView()
  if (tool === 'subtitle-format') return loadSubtitleFormatView()
  return loadProofreadView()
}

function waitForPaint() {
  return new Promise<void>((resolve) => {
    requestAnimationFrame(() => {
      requestAnimationFrame(() => resolve())
    })
  })
}

async function switchTool(toolId: ToolId, syncActiveTool: boolean) {
  if (renderedTool.value === toolId && !preparingTool.value) return
  const seq = ++toolSwitchSeq
  selectedTool.value = toolId
  preparingTool.value = true
  await nextTick()
  await waitForPaint()
  try {
    await loadToolView(toolId)
  } catch {
    // Let the async component render its normal failure state if the loader fails.
  }
  if (seq !== toolSwitchSeq) return
  if (syncActiveTool && activeTool.value !== toolId) {
    activeTool.value = toolId
  }
  renderedTool.value = toolId
  await nextTick()
  if (seq === toolSwitchSeq) {
    preparingTool.value = false
  }
}

function selectTool(tool: ToolItem) {
  void switchTool(tool.id, true)
}

watch(activeTool, (toolId) => {
  if (toolId === selectedTool.value) return
  void switchTool(toolId, false)
})

</script>

<template>
  <main class="workspace tools-workspace">
    <aside class="tool-sidebar" aria-label="工具目录">
      <header class="tool-sidebar-header">
        <h2>工具</h2>
      </header>

      <div class="tool-groups" aria-label="工具分组">
        <section
          v-for="group in toolGroups"
          :key="group.id"
          class="tool-group"
          :aria-label="group.name"
        >
          <div class="tool-group-heading">
            <span class="tool-group-title">{{ group.name }}</span>
          </div>

          <div class="tool-list" role="tablist" :aria-label="`${group.name}工具`">
            <button
              v-for="tool in group.tools"
              :key="tool.id"
              type="button"
              role="tab"
              :aria-selected="selectedTool === tool.id"
              :class="{ active: selectedTool === tool.id }"
              @click="selectTool(tool)"
            >
              <strong>{{ tool.name }}</strong>
            </button>
            <span v-if="!group.tools.length" class="tool-empty">待添加</span>
          </div>
        </section>
      </div>
    </aside>

    <section
      class="tool-content"
      :class="{ 'is-fill-tool': ['cc-subtitle', 'text-conversion', 'proofread'].includes(selectedTool) }"
      :aria-label="activeToolMeta.name"
    >
      <header class="tool-content-header">
        <div>
          <h3>{{ activeToolMeta.name }}</h3>
          <p>
            <span>{{ activeToolMeta.description }}</span>
            <span v-if="selectedTool === 'cc-subtitle'" class="tool-header-rule">
              <span>整理规则</span>
              <InfoHint
                title="CC 字幕整理规则"
                command="读取样式 → 选择听轴/花字 → 导入待整理字幕"
                body="用于把 Web CC 字幕整理成适合 Aegisub 后续精修的 ASS 结构。"
                :items="ccRuleHintItems"
                placement="left"
              />
            </span>
          </p>
        </div>
      </header>

      <div v-if="preparingTool" class="tool-loading">
        <span class="tool-loading-spinner"></span>
        <strong>正在准备工具</strong>
        <p>正在加载当前工具页面，完成后会自动显示。</p>
      </div>

      <KeepAlive>
        <component v-if="!preparingTool" :is="activeToolComponent" :key="renderedTool" />
      </KeepAlive>
    </section>
  </main>
</template>

<style scoped>
.tools-workspace {
  align-items: stretch;
  align-content: stretch;
  display: grid;
  gap: 16px;
  grid-auto-rows: minmax(0, 1fr);
  grid-template-columns: minmax(160px, 180px) minmax(0, 1fr);
  grid-template-rows: minmax(0, 1fr);
  overflow: hidden;
  min-height: 0;
  padding: 0 0 0 22px;
}

.tool-content > :deep(.text-conversion-workspace),
.tool-content > :deep(.proofread-workspace),
.tool-content > :deep(.cc-subtitle-workspace) {
  height: 100%;
  min-height: 0;
}

.tool-content > :deep(.subtitle-format-workspace),
.tool-content > :deep(.media-remux-workspace) {
  min-height: 0;
}

.tool-content > :deep(.log-panel) {
  margin-bottom: 22px;
}

.tool-sidebar {
  border-right: 1px solid #d8e2e8;
  display: flex;
  flex-direction: column;
  gap: 18px;
  min-height: 0;
  padding: 2px 16px 0 0;
}

.tool-sidebar-header h2 {
  color: #102030;
  font-size: 18px;
  margin: 0;
}

.tool-sidebar-header p {
  color: #667582;
  font-size: 13px;
  line-height: 1.5;
  margin: 6px 0 0;
}

.tool-groups {
  display: flex;
  flex-direction: column;
  gap: 20px;
  min-height: 0;
  overflow: auto;
}

.tool-group {
  display: grid;
  gap: 9px;
}

.tool-group-heading {
  align-items: center;
  display: flex;
  gap: 10px;
}

.tool-group-title {
  color: #536474;
  flex: 0 0 auto;
  font-size: 13px;
  font-weight: 800;
  line-height: 1;
}

.tool-group-heading::after {
  background: #d8e2e8;
  content: '';
  flex: 1 1 auto;
  height: 1px;
}

.tool-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.tool-list button {
  align-items: center;
  background: #eef3f6;
  border: 1px solid #d6e0e7;
  border-radius: 8px;
  color: #344552;
  display: flex;
  min-height: 48px;
  padding: 0 12px;
  text-align: left;
  width: 100%;
}

.tool-list button.active {
  background: #e8f3f7;
  border-color: #176b87;
  box-shadow: inset 4px 0 0 #176b87;
  color: #102030;
}

.tool-list button:hover {
  background: #eaf1f5;
  border-color: #c5d5de;
}

.tool-list strong {
  font-size: 15px;
  font-weight: 800;
  line-height: 1;
}

.tool-empty {
  align-items: center;
  background: #f4f6f8;
  border: 1px dashed #cbd6dd;
  border-radius: 6px;
  color: #8794a0;
  display: inline-flex;
  font-size: 13px;
  min-height: 40px;
  min-width: 92px;
  padding: 0 12px;
}

.tool-content {
  align-content: start;
  display: grid;
  gap: 12px;
  grid-template-rows: auto max-content;
  height: 100%;
  min-height: 0;
  min-width: 0;
  overflow-x: hidden;
  overflow-y: auto;
  padding: 0 22px 22px 0;
}

.tool-content.is-fill-tool {
  grid-template-rows: auto minmax(0, 1fr);
}

.tool-loading {
  align-items: center;
  align-self: stretch;
  background: #f7fafc;
  border: 1px solid #d8e2e8;
  border-radius: 8px;
  color: #667582;
  display: flex;
  flex-direction: column;
  gap: 8px;
  justify-content: center;
  min-height: 280px;
  text-align: center;
}

.tool-loading strong {
  color: #102030;
  font-size: 16px;
}

.tool-loading p {
  font-size: 13px;
  line-height: 1.5;
  margin: 0;
}

.tool-loading-spinner {
  border: 3px solid #d8e8ee;
  border-top-color: #176b87;
  border-radius: 999px;
  display: inline-block;
  height: 30px;
  width: 30px;
  animation: tool-spin 0.85s linear infinite;
}

@keyframes tool-spin {
  to {
    transform: rotate(360deg);
  }
}

.tool-content-header {
  border-bottom: 1px solid #d8e2e8;
  display: block;
  padding: 0 0 12px;
}

.tool-content-header h3 {
  color: #102030;
  font-size: 24px;
  line-height: 1.2;
  margin: 0;
}

.tool-content-header p {
  align-items: center;
  color: #667582;
  display: flex;
  flex-wrap: wrap;
  font-size: 13px;
  gap: 8px;
  line-height: 1.5;
  margin: 6px 0 0;
}

.tool-header-rule {
  align-items: center;
  display: inline-flex;
  gap: 6px;
  white-space: nowrap;
}

.tool-header-rule :deep(.rich-hint-card) {
  max-width: min(560px, calc(100vw - 72px));
  min-width: min(460px, calc(100vw - 72px));
  padding: 14px 16px;
}

.tool-header-rule :deep(.tip-left .rich-hint-card) {
  left: auto;
  right: 0;
  transform: translateY(8px);
}

.tool-header-rule :deep(.rich-hint-list) {
  gap: 7px;
}

.tool-header-rule :deep(.rich-hint-list span) {
  font-size: 12.5px;
  line-height: 1.55;
}

@media (max-width: 920px) {
  .tools-workspace {
    grid-template-columns: minmax(0, 1fr);
    grid-template-rows: auto max-content;
    padding-left: 18px;
  }

  .tool-sidebar {
    border-bottom: 1px solid #d8e2e8;
    border-right: 0;
    padding: 0 0 12px;
  }

  .tool-groups {
    overflow: visible;
  }

  .tool-list {
    flex-direction: row;
    overflow-x: auto;
    padding-bottom: 2px;
  }

  .tool-list button {
    min-width: 220px;
  }

  .tool-content-header p {
    text-align: left;
  }
}
</style>
