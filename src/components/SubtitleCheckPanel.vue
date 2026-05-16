<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { SubtitleAnalysisResult } from '../api/compress'
import type { ColorMatrixCheck, CheckLevel } from '../utils/colorMatrix'

type CheckItem = {
  id: string
  level: CheckLevel
  label: string
  title: string
  metaLayout?: 'default' | 'inline' | 'font-grid'
  detail?: string
  suggestion?: string
  meta?: Array<{ label: string; value: string }>
  tagValues?: string[]
}

const props = defineProps<{
  matrixCheck: ColorMatrixCheck | null
  analysis: SubtitleAnalysisResult | null
}>()

const expanded = ref(true)
const openItems = ref<Record<string, boolean>>({})

const items = computed<CheckItem[]>(() => {
  const next: CheckItem[] = []
  const matrix = props.matrixCheck
  if (matrix?.shouldWarn) {
    next.push({
      id: 'matrix',
      level: matrix.level,
      label: levelLabel(matrix.level),
      title: matrix.title,
      metaLayout: 'inline',
      detail: matrix.detail,
      suggestion: matrix.suggestion,
      meta: [
        { label: 'ASS 声明', value: matrix.assRaw || '未声明' },
        { label: '视频色域', value: matrix.videoStandard || '未知' },
        { label: '视频量化范围', value: matrix.videoRangeKind || '未知' },
      ],
    })
  }

  const missingImgPaths = props.analysis?.missingImgPaths ?? []
  if (missingImgPaths.length > 0) {
    next.push({
      id: 'missing-img-paths',
      level: 'error',
      label: levelLabel('error'),
      title: '字幕引用的图片路径不存在',
      detail: 'ASS/SSA 中的 \\img / \\1img-\\4img 图片填充标签引用了本机不存在的文件，AVS/VSFilterMod 渲染时会缺图或失败。',
      suggestion: '请把图片文件放回原路径，或修改字幕中的 img 路径后重新检测。',
      meta: missingImgPaths.slice(0, 12).map((item) => ({
        label: `第 ${item.line} 行 ${item.tag}`,
        value: item.resolvedPath || item.path,
      })),
    })
  }

  const missingFonts = props.analysis?.missingFonts ?? []
  if (missingFonts.length > 0) {
    next.push({
      id: 'missing-fonts',
      level: 'warn',
      label: levelLabel('warn'),
      title: '字幕使用的字体未检测到安装',
      metaLayout: 'font-grid',
      detail: '缺失字体会触发系统或渲染器字体替换，可能导致字形、字重、排版宽度和特效位置变化。',
      suggestion: '请安装字幕包附带字体，或把 ASS 样式 Fontname 改为本机已安装字体。',
      meta: missingFonts.slice(0, 12).map((item) => ({
        label: item.line ? `第 ${item.line} 行` : item.source,
        value: item.font,
      })),
    })
  }

  const missingStyles = props.analysis?.missingStyles ?? []
  if (missingStyles.length > 0) {
    next.push({
      id: 'missing-styles',
      level: 'error',
      label: levelLabel('error'),
      title: '字幕行引用了不存在的样式',
      detail: 'Events 段中的 Dialogue/Comment 行引用了 Styles 段未定义的样式名，渲染时会回退默认样式或出现异常效果。',
      suggestion: '请在 [V4+ Styles] 中补齐对应 Style，或把事件行的 Style 字段改为已有样式。',
      meta: missingStyles.slice(0, 12).map((item) => ({
        label: `第 ${item.line} 行`,
        value: item.style,
      })),
    })
  }

  const tags = props.analysis?.detectedTags ?? []
  if (tags.length > 0) {
    const hasImg = tags.some((tag) => /img/i.test(tag))
    const hasModTag = tags.some((tag) => !/img/i.test(tag))
    next.push({
      id: 'effects',
      level: 'info',
      label: levelLabel('info'),
      title: effectTitle(),
      detail: effectDetail(hasImg, hasModTag),
      suggestion: hasImg ? '建议启用 AVS 压制模式，或确认非 AVS 输出是否符合预期。' : undefined,
      tagValues: tags,
    })
  }

  return next.sort((a, b) => levelRank(b.level) - levelRank(a.level))
})

const issueCount = computed(() => items.value.length)
const highestLevel = computed<CheckLevel>(() => {
  if (items.value.some((item) => item.level === 'error')) return 'error'
  if (items.value.some((item) => item.level === 'warn')) return 'warn'
  if (items.value.some((item) => item.level === 'info')) return 'info'
  return 'ok'
})
const panelClass = computed(() => `level-${highestLevel.value}`)

watch(
  () => items.value.map((item) => `${item.id}:${item.title}`).join('|'),
  () => {
    expanded.value = items.value.length > 0
    const defaults: Record<string, boolean> = {}
    for (const item of items.value) {
      defaults[item.id] = item.level === 'error'
    }
    openItems.value = defaults
  },
  { immediate: true },
)

function levelRank(level: CheckLevel) {
  if (level === 'error') return 3
  if (level === 'warn') return 2
  if (level === 'info') return 1
  return 0
}

function levelLabel(level: CheckLevel) {
  if (level === 'error') return '错误'
  if (level === 'warn') return '警告'
  if (level === 'info') return '信息'
  return '正常'
}

function effectTitle() {
  return '检测到 VSFilterMod 标签，建议启用 AVS 压制模式'
}

function effectDetail(hasImg: boolean, hasModTag: boolean) {
  if (hasImg) {
    return '这些标签通常依赖 AVS/VSFilterMod 渲染；请确认素材资源完整，并开启 AVS 压制以尽量还原字幕效果。'
  }
  if (hasModTag) {
    return '这些标签通常依赖 AVS/VSFilterMod 渲染；建议开启 AVS 压制以尽量还原字幕效果。'
  }
  return '字幕中包含建议使用 AVS 压制的标签，请在压制前确认 AVS 模式已开启。'
}

function toggleItem(id: string) {
  openItems.value = {
    ...openItems.value,
    [id]: !openItems.value[id],
  }
}
</script>

<template>
  <section v-if="issueCount" class="subtitle-check" :class="panelClass">
    <div class="subtitle-check-head">
      <div>
        <h3>字幕检查</h3>
        <p>发现 {{ issueCount }} 个需要确认的项目</p>
      </div>
      <button type="button" class="subtitle-check-toggle" @click="expanded = !expanded">
        {{ expanded ? '收起' : '展开' }}
      </button>
    </div>

    <div v-if="expanded" class="subtitle-check-list">
      <article v-for="item in items" :key="item.id" class="check-item" :class="`item-${item.level}`">
        <div class="check-item-main">
          <span class="check-level">{{ item.label }}</span>
          <strong>{{ item.title }}</strong>
          <button
            v-if="item.detail || item.suggestion || item.meta?.length || item.tagValues?.length"
            type="button"
            class="check-detail-toggle"
            @click="toggleItem(item.id)"
          >
            {{ openItems[item.id] ? '收起详情' : '查看详情' }}
          </button>
        </div>
        <div v-if="openItems[item.id]" class="check-detail">
          <p v-if="item.detail">{{ item.detail }}</p>
          <p v-if="item.suggestion" class="check-suggestion">{{ item.suggestion }}</p>
          <dl v-if="item.meta?.length" class="check-meta" :class="`layout-${item.metaLayout ?? 'default'}`">
            <div v-for="entry in item.meta" :key="entry.label">
              <dt>{{ entry.label }}</dt>
              <dd>{{ entry.value }}</dd>
            </div>
          </dl>
          <div v-if="item.tagValues?.length" class="check-tag-list" aria-label="命中标签">
            <span v-for="tag in item.tagValues" :key="tag" class="check-tag">{{ tag }}</span>
          </div>
        </div>
      </article>
    </div>
  </section>
</template>

<style scoped>
.subtitle-check {
  background: #f8fafb;
  border: 1px solid #dce5ea;
  border-radius: 8px;
  box-shadow: inset 3px 0 0 #8aa4b4;
  margin: 0 0 12px;
  overflow: hidden;
}
.subtitle-check.level-error {
  background: #fff5f5;
  border-color: #f2b8b8;
  box-shadow: inset 3px 0 0 #d94a4a;
}
.subtitle-check.level-warn {
  background: #fff8e8;
  border-color: #efce83;
  box-shadow: inset 3px 0 0 #d99a18;
}
.subtitle-check-head {
  align-items: center;
  display: flex;
  gap: 12px;
  justify-content: space-between;
  padding: 12px 14px;
}
.subtitle-check-head h3 {
  color: #18202a;
  font-size: 14px;
  margin: 0;
}
.subtitle-check-head p {
  color: #687682;
  font-size: 12.5px;
  margin: 2px 0 0;
}
.subtitle-check-toggle,
.check-detail-toggle {
  background: rgba(255, 255, 255, 0.66);
  border: 1px solid rgba(24, 32, 42, 0.16);
  border-radius: 6px;
  color: #43515c;
  cursor: pointer;
  flex-shrink: 0;
  font-size: 12.5px;
  font-weight: 600;
  padding: 6px 10px;
}
.subtitle-check-list {
  border-top: 1px solid rgba(24, 32, 42, 0.08);
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 10px 14px 14px;
}
.check-item {
  background: rgba(255, 255, 255, 0.72);
  border: 1px solid rgba(24, 32, 42, 0.1);
  border-radius: 6px;
  padding: 9px 10px;
}
.check-item-main {
  align-items: center;
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
.check-item-main strong {
  color: #18202a;
  flex: 1;
  font-size: 13.5px;
  line-height: 1.45;
  min-width: 220px;
}
.check-level {
  border-radius: 999px;
  flex-shrink: 0;
  font-size: 12px;
  font-weight: 650;
  line-height: 1;
  padding: 5px 8px;
}
.item-error .check-level {
  background: #f8d7d7;
  color: #7f2424;
}
.item-warn .check-level {
  background: #f4dfaa;
  color: #735107;
}
.item-info .check-level {
  background: #dbeafe;
  color: #1e4f86;
}
.check-detail {
  color: #43515c;
  font-size: 12.5px;
  line-height: 1.65;
  margin-top: 8px;
}
.check-detail p {
  margin: 0 0 5px;
}
.check-suggestion {
  color: #18202a;
  font-weight: 650;
}
.check-meta {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin: 8px 0 0;
}
.check-meta div {
  align-items: baseline;
  background: rgba(248, 250, 251, 0.9);
  border: 1px solid rgba(24, 32, 42, 0.08);
  border-radius: 6px;
  display: flex;
  gap: 8px;
  min-height: 34px;
  padding: 6px 9px;
}
.check-meta dt {
  color: #687682;
  flex-shrink: 0;
  font-weight: 600;
}
.check-meta dd {
  color: #18202a;
  font-weight: 650;
  margin: 0;
}
.check-meta.layout-inline {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
}
.check-meta.layout-inline div {
  align-items: center;
  min-width: 0;
}
.check-meta.layout-inline dt,
.check-meta.layout-inline dd {
  min-width: 0;
}
.check-meta.layout-inline dd {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.check-meta.layout-font-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
}
.check-meta.layout-font-grid div {
  align-items: center;
  min-width: 0;
}
.check-meta.layout-font-grid dd {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.check-tag-list {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 8px;
}
.check-tag {
  background: #edf6f9;
  border: 1px solid #c6e0e8;
  border-radius: 999px;
  color: #0f5268;
  font-family: "Cascadia Code", Consolas, monospace;
  font-size: 12px;
  font-weight: 650;
  line-height: 1;
  padding: 6px 9px;
}
@media (max-width: 1080px) {
  .check-meta.layout-inline,
  .check-meta.layout-font-grid {
    grid-template-columns: 1fr;
  }
}
</style>
