<script setup lang="ts">
import { computed } from 'vue'
import type { CompressJob } from '../types'
import type { EncoderOption } from '../composables/useEncoderOptions'
import AppSelect from './AppSelect.vue'

type BitrateMode = 'none' | 'auto' | 'custom'

export type EncodeSettingsModel = {
  encoder: CompressJob['encoder']
  crf: number
  maxBitrate?: number
}

defineProps<{
  encoderOptions: EncoderOption[]
}>()

const settings = defineModel<EncodeSettingsModel>({ required: true })

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

const qualityModel = computed({
  get() {
    return settings.value.crf
  },
  set(value: number) {
    settings.value = { ...settings.value, crf: value }
  },
})

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
        质量值
        <span
          class="hint tip-right"
          :data-tip="`对应命令：x264/x265 使用 -crf ${settings.crf}，NVENC 使用 -cq ${settings.crf}，AMF 使用 -qp_i/-qp_p/-qp_b ${settings.crf}。\nVideoToolbox 不使用该质量值，建议通过最大码率控制。\n\n数值越小画质越好、文件越大。\nlibx264 / libx265 推荐 18-28：18 视觉无损，23 默认，28 偏低质量。\nNVENC / AMF 推荐 18-28：通常 19-23 比较均衡。`"
        ></span>
      </span>
      <input v-model.number="qualityModel" type="number" min="0" max="51" />
    </label>

    <label class="bitrate-cell">
      <span>
        最大码率
        <span
          class="hint tip-right"
          data-tip="对应命令：-maxrate {值}k -bufsize {值×2}k

限制视频码率峰值，防止画面剧烈变化时码率失控。
不限制：完全跟随质量值。
自动：取原视频码率 + 1000 Kbps。
自定义：按填写的 Kbps 直接生效。"
        ></span>
      </span>
      <div class="bitrate-control">
        <AppSelect
          v-model="bitrateMode"
          class="bitrate-select"
          :options="[
            { value: 'none', label: '不限制' },
            { value: 'auto', label: '自动（视频原码率 + 1000 Kbps）' },
            { value: 'custom', label: '自定义' }
          ]"
        />
        <span v-if="bitrateMode === 'custom'" class="bitrate-input-wrap">
          <input
            v-model.number="customBitrate"
            type="number"
            min="1"
            class="bitrate-input"
            placeholder="例如 3000"
          />
          <span>Kbps</span>
        </span>
      </div>
    </label>

    <label class="encoder-cell">
      <span>
        编码器
        <span
          class="hint tip-right"
          :data-tip="`对应命令：-c:v ${settings.encoder}\n\nlibx264：H.264 CPU 软编，兼容性最好、画质稳定，支持 AVS。\nlibx265：H.265/HEVC CPU 软编，体积更小，速度较慢。\nh264_nvenc：NVIDIA 显卡硬编，速度快，不支持 AVS。\nh264_amf：AMD 显卡硬编，速度快，不支持 AVS。\nh264_videotoolbox：macOS 硬编，不支持 AVS。`"
        ></span>
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
}
.encoder-select {
  flex: 1;
  min-width: 0;
}
.encoder-control :slotted(*) {
  flex: 0 0 auto;
}
</style>
