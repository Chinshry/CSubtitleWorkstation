<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { SubtitleAnalysisResult } from '../api/compress'
import type { ColorMatrixCheck, CheckLevel } from '../utils/colorMatrix'
import { useI18n } from '../i18n'

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
  analyzing?: boolean
}>()

const { t } = useI18n()
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
        { label: t('subtitleCheck.matrix.assRaw'), value: matrix.assRaw || t('subtitleCheck.matrix.undeclared') },
        { label: t('subtitleCheck.matrix.videoStandard'), value: matrix.videoStandard || t('subtitleCheck.matrix.unknown') },
        { label: t('subtitleCheck.matrix.videoRange'), value: matrix.videoRangeKind || t('subtitleCheck.matrix.unknown') },
      ],
    })
  }

  const missingImgPaths = props.analysis?.missingImgPaths ?? []
  if (missingImgPaths.length > 0) {
    next.push({
      id: 'missing-img-paths',
      level: 'error',
      label: levelLabel('error'),
      title: t('subtitleCheck.missingImage.title'),
      detail: t('subtitleCheck.missingImage.detail'),
      suggestion: t('subtitleCheck.missingImage.suggestion'),
      meta: missingImgPaths.slice(0, 12).map((item) => ({
        label: t('subtitleCheck.lineTag', { line: item.line, tag: item.tag }),
        value: item.resolvedPath || item.path,
      })),
    })
  }

  const missingFonts = props.analysis?.missingFonts ?? []
  if (missingFonts.length > 0) {
    next.push({
      id: 'missing-fonts',
      level: 'error',
      label: levelLabel('error'),
      title: t('subtitleCheck.missingFont.title'),
      metaLayout: 'font-grid',
      detail: t('subtitleCheck.missingFont.detail'),
      suggestion: t('subtitleCheck.missingFont.suggestion'),
      meta: missingFonts.slice(0, 12).map((item) => ({
        label: item.line ? t('subtitleCheck.line', { line: item.line }) : item.source,
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
      title: t('subtitleCheck.missingStyle.title'),
      detail: t('subtitleCheck.missingStyle.detail'),
      suggestion: t('subtitleCheck.missingStyle.suggestion'),
      meta: missingStyles.slice(0, 12).map((item) => ({
        label: t('subtitleCheck.line', { line: item.line }),
        value: item.style,
      })),
    })
  }

  const tags = props.analysis?.detectedTags ?? []
  const bannerHits = props.analysis?.bannerHits ?? []
  if (tags.length > 0) {
    const hasImg = tags.some((tag) => /img/i.test(tag))
    const hasModTag = tags.some((tag) => !/img/i.test(tag))
    const hasBanner = bannerHits.length > 0
    next.push({
      id: 'effects',
      level: 'info',
      label: levelLabel('info'),
      title: effectTitle(),
      detail: effectDetail(hasImg, hasModTag, hasBanner),
      suggestion: hasImg ? t('subtitleCheck.effectSuggestion') : undefined,
      // 命中行号详情（仅 banner 需要，VSFilterMod override 标签无行号信息）
      meta: hasBanner
        ? bannerHits.slice(0, 12).map((hit) => ({
            label: t('subtitleCheck.line', { line: hit.line }),
            value: hit.raw,
          }))
        : undefined,
      tagValues: tags,
    })
  }

  return next.sort((a, b) => levelRank(b.level) - levelRank(a.level))
})

const issueCount = computed(() => items.value.length)
const headerTitle = computed(() => props.analyzing ? t('subtitleCheck.titleChecking') : t('subtitleCheck.title'))
const headerSummary = computed(() => {
  if (props.analyzing) return t('subtitleCheck.analyzingSummary')
  return t('subtitleCheck.summary', { count: issueCount.value })
})
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
  if (level === 'error') return t('subtitleCheck.level.error')
  if (level === 'warn') return t('subtitleCheck.level.warn')
  if (level === 'info') return t('subtitleCheck.level.info')
  return t('subtitleCheck.level.ok')
}

function effectTitle() {
  return t('subtitleCheck.effects.title')
}

function effectDetail(hasImg: boolean, hasModTag: boolean, hasBanner: boolean) {
  if (hasBanner) {
    return t('subtitleCheck.effects.banner')
  }
  if (hasImg) {
    return t('subtitleCheck.effects.image')
  }
  if (hasModTag) {
    return t('subtitleCheck.effects.modTag')
  }
  return t('subtitleCheck.effects.fallback')
}

function toggleItem(id: string) {
  openItems.value = {
    ...openItems.value,
    [id]: !openItems.value[id],
  }
}
</script>

<template>
  <section v-if="props.analyzing || issueCount" class="subtitle-check" :class="panelClass">
    <div class="subtitle-check-head">
      <div>
        <h3>{{ headerTitle }}</h3>
        <p>{{ headerSummary }}</p>
      </div>
      <button v-if="issueCount" type="button" class="subtitle-check-toggle" @click="expanded = !expanded">
        {{ expanded ? t('subtitleCheck.collapse') : t('subtitleCheck.expand') }}
      </button>
    </div>

    <div v-if="expanded && issueCount" class="subtitle-check-list">
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
            {{ openItems[item.id] ? t('subtitleCheck.collapseDetail') : t('subtitleCheck.viewDetail') }}
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
          <div v-if="item.tagValues?.length" class="check-tag-list" :aria-label="t('subtitleCheck.hitTags')">
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
