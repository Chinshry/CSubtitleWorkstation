<script setup lang="ts">
const props = withDefaults(defineProps<{
  command: string[]
  running: boolean
  canRun: boolean
  startLabel: string
  cancelLabel: string
  previewDisabledTip?: string
}>(), {
  previewDisabledTip: '选择输入和输出后自动生成命令'
})

const previewOpen = defineModel<boolean>('previewOpen', { required: true })

const emit = defineEmits<{
  (e: 'run'): void
  (e: 'cancel'): void
}>()
</script>

<template>
  <section class="actions command-task-actions">
    <button
      type="button"
      class="secondary command-toggle"
      :class="{ active: previewOpen }"
      :disabled="!props.command.length"
      v-tooltip="props.command.length ? '' : props.previewDisabledTip"
      @click="previewOpen = !previewOpen"
    >
      {{ previewOpen ? '隐藏命令预览' : '显示命令预览' }}
    </button>
    <button v-if="props.running" type="button" class="danger" @click="emit('cancel')">
      {{ props.cancelLabel }}
    </button>
    <button v-else type="button" :disabled="!props.canRun" @click="emit('run')">
      {{ props.startLabel }}
    </button>
  </section>
</template>
