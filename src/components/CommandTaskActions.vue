<script setup lang="ts">
import { useI18n } from '../i18n'

const props = withDefaults(defineProps<{
  command: string[]
  running: boolean
  canRun: boolean
  startLabel: string
  cancelLabel: string
  runningLabel?: string
  cancelable?: boolean
  previewDisabledTip?: string
  runDisabledTip?: string
}>(), {
  cancelable: true
})

const previewOpen = defineModel<boolean>('previewOpen', { required: true })
const { t } = useI18n()

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
      v-tooltip="props.command.length ? '' : (props.previewDisabledTip ?? t('common.commandPreviewDisabledTip'))"
      @click="previewOpen = !previewOpen"
    >
      {{ previewOpen ? t('common.hideCommandPreview') : t('common.showCommandPreview') }}
    </button>
    <button v-if="props.running && props.cancelable" type="button" class="danger" @click="emit('cancel')">
      {{ props.cancelLabel }}
    </button>
    <button v-else-if="props.running" type="button" disabled>
      {{ props.runningLabel ?? t('common.processing') }}
    </button>
    <button
      v-else
      type="button"
      :disabled="!props.canRun"
      v-tooltip="props.canRun ? '' : (props.runDisabledTip ?? t('common.runDisabledTip'))"
      @click="emit('run')"
    >
      {{ props.startLabel }}
    </button>
  </section>
</template>
