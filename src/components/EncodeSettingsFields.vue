<script setup lang="ts">
import { computed } from 'vue'
import type { CompressJob } from '../types'
import type { EncoderOption } from '../composables/useEncoderOptions'
import AppSelect from './AppSelect.vue'
import InfoHint from './InfoHint.vue'
import { useI18n } from '../i18n'

type BitrateMode = 'none' | 'auto' | 'custom'

export type EncodeSettingsModel = {
  encoder: CompressJob['encoder']
  crf: number | null
  maxBitrate?: number
}

defineProps<{
  encoderOptions: EncoderOption[]
}>()

const settings = defineModel<EncodeSettingsModel>({ required: true })
const { t } = useI18n()

const encoderModel = computed({
  get() {
    return settings.value.encoder
  },
  set(value: string | number) {
    if (
      value === 'libx264'
      || value === 'libx265'
      || value === 'h264_nvenc'
      || value === 'h264_amf'
      || value === 'h264_videotoolbox'
    ) {
      settings.value = { ...settings.value, encoder: value }
    }
  },
})

const qualityModel = computed<number | ''>({
  get() {
    return settings.value.crf ?? ''
  },
  set(value: number | string) {
    if (value === '' || value === null || value === undefined) {
      settings.value = { ...settings.value, crf: null }
      return
    }
    const parsed = typeof value === 'number' ? value : Number(value)
    settings.value = {
      ...settings.value,
      crf: Number.isFinite(parsed) ? Math.min(51, Math.max(0, Math.round(parsed))) : null
    }
  },
})

function onQualityInput(event: Event) {
  const raw = (event.target as HTMLInputElement).value
  qualityModel.value = raw === '' ? '' : Number(raw)
}

const bitrateMode = computed<BitrateMode>({
  get(): BitrateMode {
    const value = settings.value.maxBitrate
    if (value === undefined || value === null || (typeof value === 'number' && value < 0)) return 'none'
    if (value === 0) return 'auto'
    return 'custom'
  },
  set(mode: BitrateMode) {
    if (mode === 'none') settings.value = { ...settings.value, maxBitrate: undefined }
    else if (mode === 'auto') settings.value = { ...settings.value, maxBitrate: 0 }
    else {
      const current = settings.value.maxBitrate
      if (!current || current <= 0) settings.value = { ...settings.value, maxBitrate: 3000 }
    }
  },
})

const customBitrate = computed<number | undefined>({
  get() {
    const value = settings.value.maxBitrate
    return typeof value === 'number' && value > 0 ? value : undefined
  },
  set(value) {
    if (typeof value === 'number' && value > 0) {
      settings.value = { ...settings.value, maxBitrate: Math.round(value) }
    }
  },
})
</script>

<template>
  <div class="param-row encode-settings-fields">
    <label class="crf-cell">
      <span>
        {{ t('encodeSettings.quality') }}
        <InfoHint
          placement="right"
          :title="t('encodeSettings.qualityTitle')"
          :command="settings.crf === null ? t('encodeSettings.qualityEmptyCommand') : t('encodeSettings.qualityCommand', { crf: settings.crf })"
          :body="t('encodeSettings.qualityBody')"
          :items="[t('encodeSettings.qualityItems.x264'), t('encodeSettings.qualityItems.hardware'), t('encodeSettings.qualityItems.videotoolbox')]"
        />
      </span>
      <input
        :value="qualityModel"
        type="number"
        min="0"
        max="51"
        :placeholder="t('encodeSettings.emptyPlaceholder')"
        @input="onQualityInput"
      />
    </label>

    <label class="bitrate-cell">
      <span>
        {{ t('encodeSettings.maxBitrate') }}
        <InfoHint
          placement="right"
          :title="t('encodeSettings.maxBitrateTitle')"
          :command="t('encodeSettings.maxBitrateCommand')"
          :body="t('encodeSettings.maxBitrateBody')"
          :items="[t('encodeSettings.maxBitrateItems.none'), t('encodeSettings.maxBitrateItems.auto'), t('encodeSettings.maxBitrateItems.custom')]"
        />
      </span>
      <div class="bitrate-control">
        <AppSelect
          v-model="bitrateMode"
          class="bitrate-select"
          :options="[
            { value: 'none', label: t('encodeSettings.bitrateOptions.none') },
            { value: 'auto', label: t('encodeSettings.bitrateOptions.auto') },
            { value: 'custom', label: t('encodeSettings.bitrateOptions.custom') }
          ]"
        />
        <span v-if="bitrateMode === 'custom'" class="bitrate-input-wrap">
          <input
            v-model.number="customBitrate"
            type="number"
            min="1"
            class="bitrate-input"
            :placeholder="t('encodeSettings.bitratePlaceholder')"
          />
          <span>Kbps</span>
        </span>
      </div>
    </label>

    <label class="encoder-cell">
      <span>
        {{ t('encodeSettings.encoder') }}
        <InfoHint
          placement="right"
          :title="t('encodeSettings.encoderTitle')"
          :command="`-c:v ${settings.encoder}`"
          :body="t('encodeSettings.encoderBody')"
          :items="[t('encodeSettings.encoderItems.x264'), t('encodeSettings.encoderItems.x265'), t('encodeSettings.encoderItems.hardware'), t('encodeSettings.encoderItems.videotoolbox')]"
        />
      </span>
      <div class="encoder-control">
        <AppSelect
          v-model="encoderModel"
          class="encoder-select"
          :options="encoderOptions"
        />
        <slot name="encoder-trailing" />
      </div>
    </label>
  </div>
</template>

<style scoped>
.encoder-control {
  align-items: center;
  display: flex;
  gap: 10px;
  min-width: 0;
  width: 100%;
}
.encoder-select {
  flex: 1;
  min-width: 0;
}
.encoder-control :slotted(*) {
  flex: 0 0 auto;
}
</style>
