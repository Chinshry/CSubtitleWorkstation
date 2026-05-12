<script setup lang="ts">
import { computed, onMounted, watch } from 'vue'
import type { CompressJob } from '../types'
import { isWindows } from '../stores/platformStore'
import { avsStatus, initAvsStatus } from '../stores/avsStore'
import AppSelect from './AppSelect.vue'

const job = defineModel<CompressJob>({ required: true })

// 副标题：仅 Windows 提示可启用 AVS，其它平台说明走 filter 模式
const avsHint = computed(() =>
  isWindows.value
    ? '可启用 AVS 兼容模式；不勾选则走 ffmpeg filter 模式。'
    : '当前平台不支持 AVS，使用 ffmpeg filter 模式。'
)

// AVS 开关启用条件：仅 Windows + 检测通过；缺失依赖时禁用并给出提示
const avsToggleDisabled = computed(() => !isWindows.value || !avsStatus.value?.available)
const avsToggleTip = computed(() => {
  if (!isWindows.value) return 'AVS 压制仅 Windows 支持'
  const s = avsStatus.value
  if (!s) return '正在检测 AVS 环境…'
  if (s.available) return '已检测到 AviSynth+ 与 ffmpeg avisynth demuxer，可启用 AVS 压制'
  return s.message ?? 'AVS 环境不可用'
})

// 平台不支持或 mock 切换导致已勾选但不可用时，强制关掉
function syncAvsAvailability() {
  if (avsToggleDisabled.value && job.value.useAvs) {
    job.value.useAvs = false
  }
}

// AVS 状态变化时（含调试 mock 切换）自动同步
watch(avsToggleDisabled, syncAvsAvailability)

onMounted(async () => {
  if (isWindows.value) {
    await initAvsStatus()
  }
  syncAvsAvailability()
})

// 最大码率三选一：none(留空) / auto(=0) / custom(>0)
type BitrateMode = 'none' | 'auto' | 'custom'

const bitrateMode = computed<BitrateMode>({
  get(): BitrateMode {
    const v = job.value.maxBitrate
    if (v === undefined || v === null || (typeof v === 'number' && v < 0)) return 'none'
    if (v === 0) return 'auto'
    return 'custom'
  },
  set(mode: BitrateMode) {
    if (mode === 'none') job.value.maxBitrate = undefined
    else if (mode === 'auto') job.value.maxBitrate = 0
    else {
      const cur = job.value.maxBitrate
      if (!cur || cur <= 0) job.value.maxBitrate = 3000
    }
  }
})

const customBitrate = computed<number | undefined>({
  get() {
    const v = job.value.maxBitrate
    return typeof v === 'number' && v > 0 ? v : undefined
  },
  set(v) {
    if (typeof v === 'number' && v > 0) job.value.maxBitrate = v
  }
})
</script>

<template>
  <section class="panel">
    <div class="panel-heading">
      <div>
        <h2>压制参数</h2>
        <p>{{ avsHint }}</p>
      </div>
    </div>

    <div class="form-grid">
      <div class="param-row wide">
        <label class="crf-cell">
          <span>
            CRF
            <span
              class="hint"
              :data-tip="`对应命令：-crf ${job.crf}\n\n常量码率因子，数值越小画质越好、文件越大。\nlibx264 / libx265 推荐 18 – 28：18 视觉无损，23 默认，28 偏低质量。\n硬件编码器（nvenc / amf / videotoolbox）的 CRF 含义略有不同，仅作近似画质参考。`"
            ></span>
          </span>
          <input v-model.number="job.crf" type="number" min="0" max="51" />
        </label>
        <label class="bitrate-cell">
          <span>
            最大码率（Kbps）
            <span
              class="hint"
              data-tip="对应命令：-maxrate {值}k -bufsize {值×2}k

限制视频码率峰值，防止画面剧烈变化时码率失控。
不限制：完全跟随 CRF。
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
            <input
              v-if="bitrateMode === 'custom'"
              v-model.number="customBitrate"
              type="number"
              min="1"
              class="bitrate-input"
              placeholder="Kbps"
            />
          </div>
        </label>
        <label class="wide encoder-cell">
          <span>
            编码器（GraphicsType）
            <span
              class="hint"
              :data-tip="`对应命令：-c:v ${job.encoder}\n\nlibx264：CPU 软编，兼容性最好、画质稳定，支持 AVS。\nh264_nvenc：NVIDIA 显卡硬编，速度快，不支持 AVS。\nh264_amf：AMD 显卡硬编，速度快，不支持 AVS。\nh264_videotoolbox：macOS 硬编，不支持 AVS。`"
            ></span>
          </span>
          <AppSelect
            v-model="job.encoder"
            :options="[
              { value: 'libx264', label: 'CPU libx264（CPU 软编，兼容性最好，支持 AVS）' },
              { value: 'h264_nvenc', label: 'NVIDIA h264_nvenc（N 卡硬编，压制更快，不支持 AVS）' },
              { value: 'h264_amf', label: 'AMD h264_amf（A 卡硬编，速度快，不支持 AVS）' },
              { value: 'h264_videotoolbox', label: 'macOS h264_videotoolbox（Apple Silicon/Intel 硬编，不支持 AVS）' }
            ]"
          />
        </label>
      </div>

      <div class="switch-row-wrap wide">
        <label class="switch-row">
          <input v-model="job.needLogo" type="checkbox" />
          <span class="switch"></span>
          <span>解析并压制 ASS logo</span>
        </label>
        <label class="switch-row">
          <input v-model="job.needYadif" type="checkbox" />
          <span class="switch"></span>
          <span>使用反交错压制</span>
          <span class="hint" data-tip="对应命令：-vf yadif

把交错信号合成连续画面，消除横向锯齿/梳状伪影。
TV 录制、转录、DV、磁带数字化等素材容易出现隔行，需要开启。
网络发布的视频通常已经是逐行扫描，不需要开启。"></span>
        </label>
        <label class="switch-row" :class="{ 'switch-row-disabled': avsToggleDisabled }" :title="avsToggleTip">
          <input
            v-model="job.useAvs"
            type="checkbox"
            :disabled="avsToggleDisabled"
          />
          <span class="switch"></span>
          <span>AVS 兼容模式</span>
          <span
            class="hint"
            data-tip="启用 AviSynth+ 脚本作为 ffmpeg 输入，字幕由 VSFilterMod 的 TextSubMod 渲染。
仅 Windows 支持；需要本机安装 AviSynth+ 且 ffmpeg 启用了 --enable-avisynth（如 Gyan.dev full 版）。
启用后 LOGO overlay 与 yadif 仍然有效，但 ffmpeg subtitles 滤镜会被跳过。"
          ></span>
        </label>
      </div>

      <label v-if="job.needLogo" class="wide">
        <span>logo 检测目录</span>
        <input v-model="job.logoDir" placeholder="例如：E:\Project\CBash\VIDEO_COMPRESSION\@@压制工作站\res\logo" />
      </label>
    </div>
  </section>
</template>
