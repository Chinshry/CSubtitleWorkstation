<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref } from 'vue'

const props = withDefaults(defineProps<{
  title: string
  command?: string
  body?: string
  items?: string[]
  placement?: 'left' | 'right' | 'center'
}>(), {
  placement: 'center'
})

const hintRef = ref<HTMLElement | null>(null)
const isVisible = ref(false)
const tooltipPosition = ref({ left: 0, top: 0 })
const tooltipPlacement = ref<'top' | 'bottom'>('bottom')

const tooltipClasses = computed(() => [
  `rich-hint-card-${props.placement}`,
  `rich-hint-card-${tooltipPlacement.value}`,
  { 'is-visible': isVisible.value }
])

function getTooltipCard() {
  return document.querySelector<HTMLElement>('.rich-hint-card.is-visible')
}

function updatePosition() {
  const anchor = hintRef.value
  const card = getTooltipCard()
  if (!anchor || !card) return

  const anchorRect = anchor.getBoundingClientRect()
  const cardRect = card.getBoundingClientRect()
  const pad = 12
  const gap = 8
  const viewportWidth = window.innerWidth
  const viewportHeight = window.innerHeight

  let left = anchorRect.left + anchorRect.width / 2 - cardRect.width / 2
  if (props.placement === 'left') {
    left = anchorRect.right - cardRect.width
  } else if (props.placement === 'right') {
    left = anchorRect.left
  }
  left = Math.max(pad, Math.min(left, viewportWidth - cardRect.width - pad))

  const belowTop = anchorRect.bottom + gap
  const aboveTop = anchorRect.top - cardRect.height - gap
  const hasRoomBelow = belowTop + cardRect.height <= viewportHeight - pad
  const hasMoreRoomBelow = viewportHeight - anchorRect.bottom >= anchorRect.top
  const placeBelow = hasRoomBelow || hasMoreRoomBelow
  tooltipPlacement.value = placeBelow ? 'bottom' : 'top'

  const top = placeBelow
    ? Math.min(belowTop, viewportHeight - cardRect.height - pad)
    : Math.max(pad, aboveTop)

  tooltipPosition.value = {
    left: Math.round(left),
    top: Math.round(top)
  }
}

async function showHint() {
  isVisible.value = true
  await nextTick()
  updatePosition()
}

function hideHint() {
  isVisible.value = false
}

function onGlobalChange() {
  if (isVisible.value) updatePosition()
}

window.addEventListener('resize', onGlobalChange)
window.addEventListener('scroll', onGlobalChange, true)

onBeforeUnmount(() => {
  window.removeEventListener('resize', onGlobalChange)
  window.removeEventListener('scroll', onGlobalChange, true)
})
</script>

<template>
  <span
    ref="hintRef"
    class="hint rich-hint"
    :class="{
      'tip-left': placement === 'left',
      'tip-right': placement === 'right',
    }"
    tabindex="0"
    :aria-label="title"
    @mouseenter="showHint"
    @mouseleave="hideHint"
    @focus="showHint"
    @blur="hideHint"
  ></span>
  <Teleport to="body">
    <span
      v-if="isVisible"
      class="rich-hint-card"
      :class="tooltipClasses"
      :style="{ left: `${tooltipPosition.left}px`, top: `${tooltipPosition.top}px` }"
      role="tooltip"
    >
      <strong>{{ title }}</strong>
      <code v-if="command">{{ command }}</code>
      <span v-if="body" class="rich-hint-body">{{ body }}</span>
      <span v-if="items?.length" class="rich-hint-list">
        <span v-for="item in items" :key="item">{{ item }}</span>
      </span>
    </span>
  </Teleport>
</template>
