<script setup lang="ts">
const props = withDefaults(defineProps<{
  label: string
  placeholder?: string
  disabled?: boolean
  readonly?: boolean
  buttonLabel?: string
}>(), {
  placeholder: '',
  disabled: false,
  readonly: true,
  buttonLabel: '选择'
})

const model = defineModel<string>({ default: '' })

const emit = defineEmits<{
  (e: 'pick'): void
}>()
</script>

<template>
  <label class="path-picker-field">
    <span>{{ props.label }}</span>
    <div class="path-picker-control">
      <input
        v-model="model"
        :readonly="props.readonly"
        :placeholder="props.placeholder"
      />
      <button
        type="button"
        class="secondary"
        :disabled="props.disabled"
        @click="emit('pick')"
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
</style>
