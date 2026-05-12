<script setup lang="ts">
import type { FfmpegStatus } from '../types'

defineProps<{
  status: FfmpegStatus | null
  loading: boolean
}>()
</script>

<template>
  <section class="panel ffmpeg-status">
    <div class="panel-heading">
      <div>
        <h2>ffmpeg 状态</h2>
        <p>应用只检测并调用本机 ffmpeg，不内置 ffmpeg。</p>
      </div>
      <span class="badge" :class="status?.available ? 'ok' : 'bad'">
        {{ loading ? '检测中' : status?.available ? '可用' : '不可用' }}
      </span>
    </div>

    <dl class="details">
      <div>
        <dt>来源</dt>
        <dd>{{ status?.source ?? '-' }}</dd>
      </div>
      <div>
        <dt>路径</dt>
        <dd>{{ status?.ffmpegPath ?? '-' }}</dd>
      </div>
      <div>
        <dt>版本</dt>
        <dd>{{ status?.ffmpegVersion ?? '-' }}</dd>
      </div>
      <div v-if="status?.message">
        <dt>提示</dt>
        <dd>{{ status.message }}</dd>
      </div>
    </dl>
  </section>
</template>
