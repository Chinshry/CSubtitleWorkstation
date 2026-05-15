<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { ColorMatrixCheck } from '../utils/colorMatrix'

const props = defineProps<{
  check: ColorMatrixCheck | null
}>()

const expanded = ref(false)

watch(
  () => props.check?.title,
  () => {
    expanded.value = false
  },
)

const visible = computed(() => {
  if (!props.check) return false
  return props.check.shouldWarn
})

const levelClass = computed(() => `level-${props.check?.level ?? 'info'}`)
const levelText = computed(() => (props.check?.level === 'error' ? '矩阵提醒' : '色彩提醒'))

function toggleDetail() {
  expanded.value = !expanded.value
}
</script>

<template>
  <transition name="banner">
    <section v-if="visible && check" class="cm-banner" :class="levelClass" role="alert">
      <div class="cm-header">
        <span class="cm-badge">{{ levelText }}</span>
        <strong class="cm-title">{{ check.title }}</strong>
        <button
          v-if="check.detail || check.suggestion"
          type="button"
          class="cm-toggle"
          @click="toggleDetail"
        >
          {{ expanded ? '收起详情' : '查看详情' }}
        </button>
      </div>

      <div v-if="expanded" class="cm-detail">
        <p v-if="check.detail" class="cm-copy">{{ check.detail }}</p>
        <p v-if="check.suggestion" class="cm-suggestion">{{ check.suggestion }}</p>

        <dl class="cm-grid" aria-label="色彩矩阵检测详情">
          <div class="cm-meta">
            <dt>ASS 声明</dt>
            <dd>{{ check.assRaw || '未声明' }}</dd>
          </div>
          <div class="cm-meta">
            <dt>视频色域</dt>
            <dd>{{ check.videoStandard || '未知' }}</dd>
          </div>
          <div class="cm-meta">
            <dt>视频量化范围</dt>
            <dd>{{ check.videoRangeKind || '未知' }}</dd>
          </div>
        </dl>
      </div>
    </section>
  </transition>
</template>

<style scoped>
.cm-banner {
  border-radius: 8px;
  margin: 0 0 12px;
  overflow: hidden;
  padding: 0;
}
.cm-banner.level-error {
  background: #fff5f5;
  border: 1px solid #f2b8b8;
  box-shadow: inset 3px 0 0 #d94a4a;
  color: #6b1f1f;
}
.cm-banner.level-warn {
  background: #fff8e8;
  border: 1px solid #efce83;
  box-shadow: inset 3px 0 0 #d99a18;
  color: #6a5104;
}
.cm-header {
  align-items: center;
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  min-height: 44px;
  padding: 10px 14px;
}
.cm-badge {
  border-radius: 999px;
  flex-shrink: 0;
  font-size: 12px;
  font-weight: 650;
  line-height: 1;
  padding: 5px 8px;
}
.level-error .cm-badge {
  background: #f8d7d7;
  color: #7f2424;
}
.level-warn .cm-badge {
  background: #f4dfaa;
  color: #735107;
}
.cm-title {
  flex: 1;
  font-size: 14px;
  font-weight: 650;
  line-height: 1.45;
  min-width: 220px;
}
.cm-toggle {
  background: rgba(255, 255, 255, 0.55);
  border: 1px solid currentColor;
  border-radius: 6px;
  color: inherit;
  cursor: pointer;
  flex-shrink: 0;
  font-size: 12.5px;
  font-weight: 600;
  line-height: 1;
  opacity: 0.8;
  padding: 7px 10px;
  transition: background 0.15s ease, opacity 0.15s ease;
}
.cm-toggle:hover {
  background: rgba(255, 255, 255, 0.85);
  opacity: 1;
}
.cm-detail {
  border-top: 1px solid rgba(107, 31, 31, 0.12);
  font-size: 12.5px;
  line-height: 1.65;
  padding: 0 14px 14px;
}
.cm-copy {
  margin: 10px 0 4px;
}
.cm-suggestion {
  font-weight: 650;
  margin: 4px 0 10px;
}
.cm-grid {
  display: grid;
  gap: 8px;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  margin: 0;
}
.cm-meta {
  align-items: baseline;
  background: rgba(255, 255, 255, 0.58);
  border: 1px solid rgba(107, 31, 31, 0.12);
  border-radius: 6px;
  display: flex;
  gap: 8px;
  min-height: 36px;
  padding: 7px 10px;
}
.cm-meta dt {
  color: inherit;
  flex-shrink: 0;
  font-weight: 600;
  opacity: 0.72;
}
.cm-meta dd {
  font-weight: 650;
  margin: 0;
}

.banner-enter-active,
.banner-leave-active {
  transition: opacity 0.18s ease, transform 0.18s ease;
}
.banner-enter-from,
.banner-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}
</style>
