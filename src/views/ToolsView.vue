<script setup lang="ts">
import { defineAsyncComponent, ref } from 'vue'

type ToolId = 'text-conversion'

const activeTool = ref<ToolId>('text-conversion')
const TextConversionView = defineAsyncComponent(() => import('./TextConversionView.vue'))

const tools: Array<{ id: ToolId; name: string; description: string }> = [
  {
    id: 'text-conversion',
    name: '繁简转换',
    description: '繁体与简体互转'
  }
]
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
          <span>{{ tool.description }}</span>
        </button>
      </div>
    </section>

    <TextConversionView v-if="activeTool === 'text-conversion'" />
  </main>
</template>

<style scoped>
.tools-workspace {
  gap: 12px;
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

.tools-bar p {
  color: #667582;
  font-size: 13px;
  margin: 4px 0 0;
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
  display: grid;
  gap: 2px;
  min-height: 48px;
  min-width: 138px;
  padding: 8px 12px;
  text-align: left;
}

.tool-tabs button.active {
  background: #176b87;
  border-color: #176b87;
  color: #fff;
}

.tool-tabs strong {
  font-size: 13px;
}

.tool-tabs span {
  font-size: 11px;
  opacity: 0.82;
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
