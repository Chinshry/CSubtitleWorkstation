<script setup lang="ts">
import { computed, nextTick, ref } from 'vue'

const props = withDefaults(defineProps<{
  label: string
  placeholder?: string
  disabled?: boolean
  readonly?: boolean
  buttonLabel?: string
  compact?: boolean
  compactAction?: 'pick' | 'clear' | 'edit'
}>(), {
  placeholder: '',
  disabled: false,
  readonly: true,
  buttonLabel: '选择',
  compact: false,
  compactAction: 'pick'
})

const model = defineModel<string>({ default: '' })

const emit = defineEmits<{
  (e: 'pick'): void
  (e: 'clear'): void
}>()

const inputRef = ref<HTMLInputElement | null>(null)
const editing = ref(false)
const editOriginal = ref('')
const effectiveReadonly = computed(() => (
  props.compact && props.compactAction === 'edit'
    ? !editing.value
    : props.readonly
))
const compactButtonText = computed(() => {
  if (!props.compact) return props.buttonLabel
  if (props.compactAction === 'clear') return model.value ? '✕' : '+'
  if (props.compactAction === 'edit') return '✎'
  return model.value ? '✎' : '+'
})

function beginEdit() {
  if (props.disabled) return
  editOriginal.value = model.value
  editing.value = true
  nextTick(() => {
    inputRef.value?.focus()
    inputRef.value?.select()
  })
}

function commitEdit() {
  if (!editing.value) return
  editing.value = false
}

function cancelEdit() {
  if (!editing.value) return
  model.value = editOriginal.value
  editing.value = false
}

function onAction() {
  if (!props.compact) {
    emit('pick')
    return
  }
  if (props.compactAction === 'clear') {
    if (model.value) {
      model.value = ''
      emit('clear')
    } else {
      emit('pick')
    }
    return
  }
  if (props.compactAction === 'edit') {
    beginEdit()
    return
  }
  emit('pick')
}
</script>

<template>
  <div v-if="props.compact" class="path-picker-field compact">
    <span>{{ props.label }}</span>
    <div class="path-row compact-path-row">
      <input
        v-if="props.compactAction === 'edit' && editing"
        ref="inputRef"
        v-model="model"
        class="path-input"
        :placeholder="props.placeholder"
        @blur="commitEdit"
        @keyup.enter="commitEdit"
        @keyup.esc="cancelEdit"
      />
      <span
        v-else
        class="path-text"
        :class="{ readonly: props.compactAction !== 'edit' }"
        v-tooltip="model || props.placeholder || '未设置'"
      >
        {{ model || '—' }}
      </span>
      <button
        type="button"
        class="path-action"
        :class="`compact-${props.compactAction}`"
        :disabled="props.disabled"
        :aria-label="props.buttonLabel"
        @click="onAction"
      >
        {{ compactButtonText }}
      </button>
    </div>
  </div>
  <label v-else class="path-picker-field">
    <span>{{ props.label }}</span>
    <div class="path-picker-control">
      <input
        ref="inputRef"
        v-model="model"
        :readonly="effectiveReadonly"
        :placeholder="props.placeholder"
        @blur="commitEdit"
        @keyup.enter="commitEdit"
        @keyup.esc="cancelEdit"
      />
      <button
        type="button"
        class="secondary"
        :disabled="props.disabled"
        :aria-label="props.buttonLabel"
        @click="onAction"
      >
        {{ props.buttonLabel }}
      </button>
    </div>
  </label>
</template>

<style scoped>
.path-picker-field {
  display: grid;
  gap: 6px;
}

.path-picker-field > span {
  color: #4d5b66;
  font-size: 13px;
}

.path-picker-control {
  display: grid;
  gap: 8px;
  grid-template-columns: minmax(0, 1fr) auto;
}

.path-picker-control input {
  min-width: 0;
  width: 100%;
}

.path-picker-control button {
  flex: 0 0 auto;
}

.compact-path-row {
  align-items: center;
  background: #f5f8fa;
  border: 1px solid #dce5ec;
  border-radius: 6px;
  display: flex;
  gap: 8px;
  margin: 0;
  min-height: 32px;
  padding: 4px 4px 4px 10px;
}

.compact-path-row .path-text {
  color: #18202a;
  flex: 1;
  font-size: 13px;
  font-variant-numeric: tabular-nums;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.compact-path-row .path-text.readonly {
  color: #43515c;
  -webkit-user-select: text;
  user-select: text;
}

.compact-path-row .path-input {
  background: #fff;
  border: 1px solid #cfd8df;
  border-radius: 4px;
  color: #18202a;
  flex: 1;
  font-size: 13px;
  min-height: 28px;
  padding: 0 8px;
}

.compact-path-row .path-action {
  align-items: center;
  background: transparent;
  border: 0;
  border-radius: 4px;
  color: #43515c;
  cursor: pointer;
  display: inline-flex;
  flex-shrink: 0;
  font-size: 13px;
  height: 26px;
  justify-content: center;
  min-height: 26px;
  padding: 0;
  position: relative;
  width: 26px;
}

.compact-path-row .path-action:hover,
.compact-path-row .path-action.active {
  background: #e2e8ec;
  color: #18202a;
}
</style>
