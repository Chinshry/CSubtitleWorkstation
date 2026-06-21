<script setup lang="ts">
import { computed, defineAsyncComponent, ref, watch } from 'vue'
import {
  activeTool,
  isMediaToolId,
  type ToolId
} from '../stores/toolStore'
import InfoHint from '../components/InfoHint.vue'
import { isChineseLocale, useI18n } from '../i18n'

const loadTextConversionView = () => import('./TextConversionView.vue')
const loadProofreadView = () => import('./ProofreadView.vue')
const loadCcSubtitleView = () => import('./CcSubtitleView.vue')
const loadSubtitleFormatView = () => import('./SubtitleFormatView.vue')
const loadMediaRemuxView = () => import('./MediaRemuxView.vue')

const asyncToolOptions = {
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

const { t } = useI18n()

type ToolItem = {
  id: ToolId
  name: string
  description: string
}

const ccRuleHintItems = computed(() => [
  t('tools.ccRule.items.readStyle'),
  t('tools.ccRule.items.convertInput'),
  t('tools.ccRule.items.bracketText'),
  t('tools.ccRule.items.dialogAfterBracket'),
  t('tools.ccRule.items.plainDialog'),
  t('tools.ccRule.items.cleanText'),
  t('tools.ccRule.items.dictionary')
])

const textTools = computed<ToolItem[]>(() => {
  const tools: ToolItem[] = [{
    id: 'cc-subtitle',
    name: t('tools.items.ccSubtitle.name'),
    description: t('tools.items.ccSubtitle.description')
  }]
  if (isChineseLocale.value) {
    tools.push(
      {
        id: 'text-conversion',
        name: t('tools.items.textConversion.name'),
        description: t('tools.items.textConversion.description')
      },
      {
        id: 'proofread',
        name: t('tools.items.proofread.name'),
        description: t('tools.items.proofread.description')
      }
    )
  }
  return tools
})

const formatTools = computed<ToolItem[]>(() => [
  {
    id: 'subtitle-format',
    name: t('tools.items.subtitleFormat.name'),
    description: t('tools.items.subtitleFormat.description')
  },
  {
    id: 'media-remux',
    name: t('tools.items.mediaRemux.name'),
    description: t('tools.items.mediaRemux.description')
  }
])

const mediaTools = computed<ToolItem[]>(() => [
  {
    id: 'media-concat-ts',
    name: t('tools.items.mediaConcatTs.name'),
    description: t('tools.items.mediaConcatTs.description')
  },
  {
    id: 'media-merge-av',
    name: t('tools.items.mediaMergeAv.name'),
    description: t('tools.items.mediaMergeAv.description')
  },
  {
    id: 'media-cover',
    name: t('tools.items.mediaCover.name'),
    description: t('tools.items.mediaCover.description')
  }
])

const toolGroups = computed(() => [
  {
    id: 'text',
    name: t('tools.groups.text.name'),
    description: t('tools.groups.text.description'),
    tools: textTools.value
  },
  {
    id: 'format',
    name: t('tools.groups.format.name'),
    description: t('tools.groups.format.description'),
    tools: formatTools.value
  },
  {
    id: 'media',
    name: t('tools.groups.media.name'),
    description: t('tools.groups.media.description'),
    tools: mediaTools.value
  }
])

const allTools = computed(() => [...textTools.value, ...formatTools.value, ...mediaTools.value])
const selectedTool = ref<ToolId>(activeTool.value)

const activeToolMeta = computed(() => allTools.value.find((tool) => tool.id === selectedTool.value) ?? textTools.value[0])

const activeToolComponent = computed(() => (
  isMediaToolId(selectedTool.value)
    ? MediaRemuxView
    : selectedTool.value === 'text-conversion'
      ? TextConversionView
      : selectedTool.value === 'cc-subtitle'
        ? CcSubtitleView
        : selectedTool.value === 'subtitle-format'
          ? SubtitleFormatView
        : ProofreadView
))

function switchTool(toolId: ToolId, syncActiveTool: boolean) {
  if (selectedTool.value === toolId) {
    if (syncActiveTool && activeTool.value !== toolId) {
      activeTool.value = toolId
    }
    return
  }
  selectedTool.value = toolId
  if (syncActiveTool && activeTool.value !== toolId) {
    activeTool.value = toolId
  }
}

function selectTool(tool: ToolItem) {
  switchTool(tool.id, true)
}

watch(activeTool, (toolId) => {
  if (toolId === selectedTool.value) return
  if (!allTools.value.some((tool) => tool.id === toolId)) {
    switchTool('cc-subtitle', true)
    return
  }
  switchTool(toolId, false)
})

watch(allTools, (tools) => {
  if (tools.some((tool) => tool.id === selectedTool.value)) return
  switchTool('cc-subtitle', true)
})

</script>

<template>
  <main class="workspace tools-workspace">
    <aside class="tool-sidebar" :aria-label="t('tools.sidebarLabel')">
      <header class="tool-sidebar-header">
        <h2>{{ t('nav.tools') }}</h2>
      </header>

      <div class="tool-groups" :aria-label="t('tools.groupListLabel')">
        <section
          v-for="group in toolGroups"
          :key="group.id"
          class="tool-group"
          :aria-label="group.name"
        >
          <div class="tool-group-heading">
            <span class="tool-group-title">{{ group.name }}</span>
          </div>

          <div class="tool-list" role="tablist" :aria-label="t('tools.groupToolsLabel', { group: group.name })">
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
            <span v-if="!group.tools.length" class="tool-empty">{{ t('tools.empty') }}</span>
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
              <span>{{ t('tools.ccRule.label') }}</span>
              <InfoHint
                :title="t('tools.ccRule.title')"
                :command="t('tools.ccRule.command')"
                :body="t('tools.ccRule.body')"
                :items="ccRuleHintItems"
                placement="left"
              />
            </span>
          </p>
        </div>
      </header>

      <KeepAlive>
        <component :is="activeToolComponent" :key="selectedTool" />
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
  max-width: min(760px, calc(100vw - 48px));
  min-width: min(420px, calc(100vw - 48px));
  padding: 14px 16px;
  white-space: normal;
  width: min(760px, calc(100vw - 48px));
  overflow-wrap: anywhere;
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

.tool-header-rule :deep(.rich-hint-card code),
.tool-header-rule :deep(.rich-hint-body),
.tool-header-rule :deep(.rich-hint-list span) {
  white-space: normal;
  overflow-wrap: anywhere;
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
