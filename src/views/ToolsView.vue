<script setup lang="ts">
import { computed, defineAsyncComponent } from 'vue'
import { activeTool, type ToolId } from '../stores/toolStore'

const TextConversionView = defineAsyncComponent(() => import('./TextConversionView.vue'))
const ProofreadView = defineAsyncComponent(() => import('./ProofreadView.vue'))
const CcSubtitleView = defineAsyncComponent(() => import('./CcSubtitleView.vue'))

type ToolItem = {
  id: ToolId
  name: string
}

const textTools: ToolItem[] = [
  {
    id: 'proofread',
    name: '字幕校对'
  },
  {
    id: 'text-conversion',
    name: '繁简转换'
  },
  {
    id: 'cc-subtitle',
    name: 'CC 字幕整理'
  }
]

const mediaTools: ToolItem[] = []

const activeToolComponent = computed(() => (
  activeTool.value === 'text-conversion'
    ? TextConversionView
    : activeTool.value === 'cc-subtitle'
      ? CcSubtitleView
      : ProofreadView
))
</script>

<template>
  <main class="workspace tools-workspace">
    <section class="tools-bar" aria-label="工具栏">
      <div>
        <h2>工具</h2>
      </div>
      <div class="tool-groups" aria-label="工具分组">
        <section class="tool-group" aria-label="文字处理">
          <span class="tool-group-title">文字处理</span>
          <div class="tool-tabs" role="tablist" aria-label="文字处理工具">
            <button
              v-for="tool in textTools"
              :key="tool.id"
              type="button"
              role="tab"
              :aria-selected="activeTool === tool.id"
              :class="{ active: activeTool === tool.id }"
              @click="activeTool = tool.id"
            >
              <strong>{{ tool.name }}</strong>
            </button>
          </div>
        </section>

        <section class="tool-group" aria-label="媒体处理">
          <span class="tool-group-title">媒体处理</span>
          <div class="tool-tabs" role="tablist" aria-label="媒体处理工具">
            <button
              v-for="tool in mediaTools"
              :key="tool.id"
              type="button"
              role="tab"
              :aria-selected="activeTool === tool.id"
              :class="{ active: activeTool === tool.id }"
              @click="activeTool = tool.id"
            >
              <strong>{{ tool.name }}</strong>
            </button>
            <span v-if="!mediaTools.length" class="tool-empty">待添加</span>
          </div>
        </section>
      </div>
    </section>

    <KeepAlive>
      <component :is="activeToolComponent" />
    </KeepAlive>
  </main>
</template>

<style scoped>
.tools-workspace {
  gap: 12px;
  grid-template-rows: auto minmax(0, 1fr);
}

.tools-workspace > :deep(.text-conversion-workspace),
.tools-workspace > :deep(.proofread-workspace),
.tools-workspace > :deep(.cc-subtitle-workspace) {
  min-height: 0;
}

.tools-bar {
  align-items: center;
  border-bottom: 1px solid #d8e2e8;
  display: flex;
  gap: 16px;
  justify-content: space-between;
  padding: 2px 0 12px;
}

.tools-bar h2 {
  color: #102030;
  font-size: 18px;
  margin: 0;
}

.tool-groups {
  align-items: flex-end;
  display: flex;
  flex-wrap: wrap;
  gap: 14px 18px;
  justify-content: flex-end;
}

.tool-group {
  align-items: flex-end;
  display: grid;
  gap: 6px;
}

.tool-group-title {
  color: #667582;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.04em;
}

.tool-tabs {
  align-items: center;
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  justify-content: flex-end;
}

.tool-tabs button {
  align-items: flex-start;
  background: #eef3f6;
  border: 1px solid #d6e0e7;
  color: #344552;
  min-height: 40px;
  min-width: 138px;
  padding: 8px 12px;
  text-align: center;
}

.tool-tabs button.active {
  background: #176b87;
  border-color: #176b87;
  color: #fff;
}

.tool-tabs strong {
  font-size: 13px;
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

@media (max-width: 920px) {
  .tools-bar {
    align-items: stretch;
    flex-direction: column;
  }

  .tool-groups,
  .tool-tabs {
    justify-content: flex-start;
  }

  .tool-group {
    align-items: flex-start;
  }
}
</style>
