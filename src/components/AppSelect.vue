<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch, nextTick } from 'vue'

interface SelectOption {
  value: string | number
  label: string
  description?: string
  title?: string
}

const props = defineProps<{
  modelValue: string | number
  options: SelectOption[]
  placeholder?: string
  disabled?: boolean
  title?: string
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', val: string | number): void
}>()

const triggerRef = ref<HTMLButtonElement | null>(null)
const panelRef = ref<HTMLDivElement | null>(null)
const open = ref(false)
const highlight = ref(0)
const panelStyle = ref<Record<string, string>>({})

const currentLabel = computed(() => {
  const found = props.options.find((o) => o.value === props.modelValue)
  return found?.label ?? ''
})

function updatePosition() {
  const el = triggerRef.value
  if (!el) return
  const r = el.getBoundingClientRect()
  panelStyle.value = {
    left: `${r.left}px`,
    top: `${r.bottom + 6}px`,
    width: `${r.width}px`
  }
}

function openPanel() {
  if (props.disabled) return
  updatePosition()
  const idx = props.options.findIndex((o) => o.value === props.modelValue)
  highlight.value = idx >= 0 ? idx : 0
  open.value = true
  nextTick(() => {
    const item = panelRef.value?.querySelector<HTMLElement>('.app-select-option.selected')
    item?.scrollIntoView({ block: 'nearest' })
  })
}

function closePanel() {
  open.value = false
}

function toggle() {
  if (open.value) closePanel()
  else openPanel()
}

function pick(opt: SelectOption) {
  emit('update:modelValue', opt.value)
  closePanel()
  triggerRef.value?.focus()
}

function onKeydown(e: KeyboardEvent) {
  if (props.disabled) return
  if (!open.value) {
    if (['ArrowDown', 'ArrowUp', 'Enter', ' '].includes(e.key)) {
      e.preventDefault()
      openPanel()
    }
    return
  }
  if (e.key === 'Escape') {
    e.preventDefault()
    closePanel()
  } else if (e.key === 'ArrowDown') {
    e.preventDefault()
    highlight.value = (highlight.value + 1) % props.options.length
  } else if (e.key === 'ArrowUp') {
    e.preventDefault()
    highlight.value = (highlight.value - 1 + props.options.length) % props.options.length
  } else if (e.key === 'Enter') {
    e.preventDefault()
    const opt = props.options[highlight.value]
    if (opt) pick(opt)
  } else if (e.key === 'Tab') {
    closePanel()
  }
}

function onDocPointer(e: MouseEvent) {
  if (!open.value) return
  const target = e.target as Node
  if (triggerRef.value?.contains(target)) return
  if (panelRef.value?.contains(target)) return
  closePanel()
}

function onWindowChange() {
  if (open.value) updatePosition()
}

onMounted(() => {
  document.addEventListener('mousedown', onDocPointer)
  window.addEventListener('resize', onWindowChange)
  window.addEventListener('scroll', onWindowChange, true)
})

onBeforeUnmount(() => {
  document.removeEventListener('mousedown', onDocPointer)
  window.removeEventListener('resize', onWindowChange)
  window.removeEventListener('scroll', onWindowChange, true)
})

watch(
  () => props.modelValue,
  () => {
    const idx = props.options.findIndex((o) => o.value === props.modelValue)
    if (idx >= 0) highlight.value = idx
  }
)
</script>

<template>
  <div class="app-select" :class="{ open, disabled }">
    <button
      ref="triggerRef"
      type="button"
      class="app-select-trigger"
      :disabled="disabled"
      :title="title"
      @click="toggle"
      @keydown="onKeydown"
    >
      <span class="app-select-value" :class="{ placeholder: !currentLabel }">
        {{ currentLabel || placeholder || '请选择…' }}
      </span>
      <svg
        class="app-select-chevron"
        :class="{ rotated: open }"
        viewBox="0 0 24 24"
        width="16"
        height="16"
        fill="none"
        stroke="currentColor"
        stroke-width="2.2"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <polyline points="6 9 12 15 18 9" />
      </svg>
    </button>
    <Teleport to="body">
      <Transition name="app-select-fade">
        <div
          v-if="open"
          ref="panelRef"
          class="app-select-panel"
          :style="panelStyle"
          role="listbox"
        >
          <button
            v-for="(opt, idx) in options"
            :key="opt.value"
            type="button"
            class="app-select-option"
            :class="{
              selected: opt.value === modelValue,
              highlighted: idx === highlight
            }"
            :title="opt.title"
            @click="pick(opt)"
            @mousemove="highlight = idx"
          >
            <span class="opt-copy">
              <span class="opt-label">{{ opt.label }}</span>
              <span v-if="opt.description" class="opt-description">{{ opt.description }}</span>
            </span>
            <svg
              v-if="opt.value === modelValue"
              class="opt-check"
              viewBox="0 0 24 24"
              width="14"
              height="14"
              fill="none"
              stroke="currentColor"
              stroke-width="2.6"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <polyline points="20 6 9 17 4 12" />
            </svg>
          </button>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<style scoped>
/* 用 :where 把根选择器优先级降到 0，外层 .quote-select / .bitrate-select 等可轻松覆盖 width */
:where(.app-select) {
  display: block;
  position: relative;
  width: 100%;
}

.app-select-trigger {
  align-items: center;
  background: #fff;
  border: 1px solid #d6dee5;
  border-radius: 8px;
  color: #18202a;
  cursor: pointer;
  display: flex;
  font-size: 13.5px;
  font-weight: 500;
  gap: 8px;
  justify-content: space-between;
  min-height: 38px;
  padding: 0 12px;
  transition: border-color 0.16s ease, box-shadow 0.16s ease, background-color 0.16s ease;
  width: 100%;
}

.app-select-trigger:hover {
  background: #f9fbfc;
  border-color: #a8c8d2;
}

.app-select.open .app-select-trigger,
.app-select-trigger:focus-visible {
  border-color: #176b87;
  box-shadow: 0 0 0 3px rgba(23, 107, 135, 0.15);
  outline: none;
}

.app-select.disabled .app-select-trigger {
  background: #f4f6f8;
  border-color: #e3e9ed;
  color: #9aa7b1;
  cursor: not-allowed;
}

.app-select-value {
  flex: 1;
  overflow: hidden;
  text-align: left;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.app-select-value.placeholder {
  color: #9aa7b1;
}

.app-select-chevron {
  color: #687582;
  flex-shrink: 0;
  transition: transform 0.18s ease, color 0.16s ease;
}

.app-select-trigger:hover .app-select-chevron,
.app-select.open .app-select-chevron {
  color: #176b87;
}

.app-select-chevron.rotated {
  transform: rotate(180deg);
}
</style>

<!-- 弹出面板通过 Teleport 到 body，scoped 不生效，必须放全局 -->
<style>
.app-select-panel {
  background: #ffffff;
  border: 1px solid #e3e9ed;
  border-radius: 10px;
  box-shadow: 0 12px 32px rgba(15, 23, 42, 0.14), 0 4px 12px rgba(15, 23, 42, 0.06);
  max-height: 280px;
  overflow: auto;
  padding: 6px;
  position: fixed;
  z-index: 9999;
}

.app-select-option {
  align-items: center;
  background: transparent;
  border: 0;
  border-radius: 6px;
  color: #18202a;
  cursor: pointer;
  display: flex;
  font-size: 13px;
  font-weight: 600;
  gap: 8px;
  justify-content: space-between;
  min-height: 36px;
  padding: 7px 10px;
  text-align: left;
  transition: background 0.12s ease, color 0.12s ease;
  width: 100%;
}

.app-select-option + .app-select-option {
  margin-top: 2px;
}

.app-select-option:hover,
.app-select-option.highlighted {
  background: #f2f7f9;
  color: #12384a;
}

.app-select-option.selected {
  background: #e4f2f6;
  color: #0f5268;
}

.app-select-option.selected:hover,
.app-select-option.selected.highlighted {
  background: #d9edf3;
  color: #0f5268;
}

.app-select-option .opt-copy {
  flex: 1;
  min-width: 0;
}

.app-select-option .opt-label,
.app-select-option .opt-description {
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.app-select-option .opt-description {
  color: #7a8894;
  font-size: 11px;
  font-weight: 500;
  line-height: 1.25;
  margin-top: 3px;
}

.app-select-option:hover .opt-description,
.app-select-option.highlighted .opt-description {
  color: #687782;
}

.app-select-option.selected .opt-description,
.app-select-option.selected:hover .opt-description,
.app-select-option.selected.highlighted .opt-description {
  color: #5f7580;
}

.app-select-option .opt-check {
  color: #176b87;
  flex-shrink: 0;
}

.app-select-fade-enter-active,
.app-select-fade-leave-active {
  transition: opacity 0.14s ease, transform 0.14s ease;
}

.app-select-fade-enter-from,
.app-select-fade-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}
</style>
