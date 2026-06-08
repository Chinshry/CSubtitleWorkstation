<script setup lang="ts">
import { computed, defineAsyncComponent } from 'vue'
import { activeTool, type ToolId } from '../stores/toolStore'

const TextConversionView = defineAsyncComponent(() => import('./TextConversionView.vue'))
const ProofreadView = defineAsyncComponent(() => import('./ProofreadView.vue'))

const tools: Array<{ id: ToolId; name: string }> = [
  {
    id: 'proofread',
    name: '字幕校对'
  },
  {
    id: 'text-conversion',
    name: '繁简转换'
  }
]

const activeToolComponent = computed(() => (
  activeTool.value === 'text-conversion' ? TextConversionView : ProofreadView
))
</script>

<template>
  <main class="workspace tools-workspace">
    <section class="tools-bar" aria-label="工具栏">
      <div>
        <h2>工具</h2>
      </div>
      <div class="tool-tabs" role="tablist" aria-label="工具列表">
        <button
          v-for="tool in tools"
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
.tools-workspace > :deep(.proofread-workspace) {
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

@media (max-width: 920px) {
  .tools-bar {
    align-items: stretch;
    flex-direction: column;
  }

  .tool-tabs {
    justify-content: flex-start;
  }
}
</style>
