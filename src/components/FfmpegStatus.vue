<script setup lang="ts">
import type { FfmpegStatus } from '../types'
import { useI18n } from '../i18n'

defineProps<{
  status: FfmpegStatus | null
  loading: boolean
}>()

const { t } = useI18n()
</script>

<template>
  <section class="panel ffmpeg-status">
    <div class="panel-heading">
      <div>
        <h2>{{ t('ffmpegStatus.title') }}</h2>
        <p>{{ t('ffmpegStatus.description') }}</p>
      </div>
      <span class="badge" :class="status?.available ? 'ok' : 'bad'">
        {{ loading ? t('ffmpegStatus.checking') : status?.available ? t('ffmpegStatus.available') : t('ffmpegStatus.unavailable') }}
      </span>
    </div>

    <dl class="details">
      <div>
        <dt>{{ t('ffmpegStatus.source') }}</dt>
        <dd>{{ status?.source ?? '-' }}</dd>
      </div>
      <div>
        <dt>{{ t('ffmpegStatus.path') }}</dt>
        <dd>{{ status?.ffmpegPath ?? '-' }}</dd>
      </div>
      <div>
        <dt>{{ t('ffmpegStatus.version') }}</dt>
        <dd>{{ status?.ffmpegVersion ?? '-' }}</dd>
      </div>
      <div v-if="status?.message">
        <dt>{{ t('ffmpegStatus.message') }}</dt>
        <dd>{{ status.message }}</dd>
      </div>
    </dl>
  </section>
</template>
