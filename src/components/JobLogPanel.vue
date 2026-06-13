<script setup lang="ts">
import { computed, ref } from 'vue'

const props = defineProps<{
  lines: string[]
  command: string[]
  percent: number
  statusLine?: string
  currentSeconds?: number
  durationSeconds?: number
  sizeKb?: number
  speed?: number
  fps?: number
  bitrateKbps?: number
  elapsedSeconds?: number
  etaSeconds?: number
  remainingSeconds?: number
  running?: boolean
  cancelled?: boolean
  title?: string
  idleTitle?: string
  idleTip?: string
}>()

// 根据日志判断最终状态
const finalStatus = computed(() => {
  // 取消优先级最高：运行中表示"取消中..."，已停止表示"已取消"
  if (props.running) return props.cancelled ? 'cancelling' : 'running'
  if (props.cancelled) return 'cancelled'
  if (!props.lines.length) return 'idle'
  const lastLine = props.lines[props.lines.length - 1]
  if (lastLine.includes('❌')) return 'failed'
  if (/^(Compression|Media tool) completed/.test(lastLine)) return 'completed'
  return 'idle'
})

const statusLabel = computed(() => {
  switch (finalStatus.value) {
    case 'running': return '运行中'
    case 'cancelling': return '取消中…'
    case 'cancelled': return '已取消'
    case 'completed': return '已完成'
    case 'failed': return '已失败'
    default: return '待开始'
  }
})

const statusClass = computed(() => {
  switch (finalStatus.value) {
    case 'running': return 'status-running'
    case 'cancelling': return 'status-cancelling'
    case 'cancelled': return 'status-cancelled'
    case 'completed': return 'status-completed'
    case 'failed': return 'status-failed'
    default: return 'status-idle'
  }
})

// 真正的“待开始空态”：未运行、未取消，且无任何历史日志
const isIdleEmpty = computed(
  () => finalStatus.value === 'idle' && !props.lines.length
)

const copyHint = ref('')

async function copyAll() {
  const text = props.lines.join('\n')
  if (!text.trim()) {
    copyHint.value = '无内容'
    setTimeout(() => (copyHint.value = ''), 1500)
    return
  }
  try {
    await navigator.clipboard.writeText(text)
    copyHint.value = '已复制'
  } catch {
    copyHint.value = '复制失败'
  }
  setTimeout(() => (copyHint.value = ''), 1500)
}

function formatTime(sec?: number): string {
  const s = typeof sec === 'number' && sec > 0 ? sec : 0
  const h = Math.floor(s / 3600)
  const m = Math.floor((s % 3600) / 60)
  const ss = Math.floor(s % 60)
  return `${String(h).padStart(2, '0')}:${String(m).padStart(2, '0')}:${String(ss).padStart(2, '0')}`
}

function formatTimeOrDash(sec?: number): string {
  if (!sec || sec <= 0) return '--:--:--'
  return formatTime(sec)
}

function formatSize(kb?: number): string {
  if (!kb || kb <= 0) return '-'
  if (kb < 1024) return `${kb.toFixed(0)} KB`
  const mb = kb / 1024
  if (mb < 1024) return `${mb.toFixed(1)} MB`
  return `${(mb / 1024).toFixed(2)} GB`
}

// 大小维度：当前大小 / 预估最终大小（按 current/duration 等比例推算）
const estimatedSizeKb = computed(() => {
  if (!props.sizeKb || !props.currentSeconds || !props.durationSeconds) return 0
  if (props.currentSeconds <= 0) return 0
  return Math.round((props.sizeKb / props.currentSeconds) * props.durationSeconds)
})
</script>

<template>
  <section class="panel log-panel" :class="{ 'is-idle-empty': isIdleEmpty }">
    <div class="panel-heading">
      <div class="heading-title">
        <h2>{{ title ?? '压制进度' }}</h2>
        <span :class="['status-badge', statusClass]">{{ statusLabel }}</span>
      </div>
      <button v-if="!isIdleEmpty" class="copy-btn" :class="{ active: copyHint }" @click="copyAll" data-tip="复制全部日志">
        <span v-if="copyHint">{{ copyHint }}</span>
        <span v-else>复制</span>
      </button>
    </div>

    <div v-if="isIdleEmpty" class="idle-hero">
      <div class="idle-hero-icon" aria-hidden="true">
        <svg viewBox="0 0 24 24" width="28" height="28" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
          <polygon points="6 4 20 12 6 20 6 4"></polygon>
        </svg>
      </div>
      <p class="idle-hero-title">{{ idleTitle ?? '尚未开始压制' }}</p>
      <p class="idle-hero-tip">{{ idleTip ?? '配置好参数后点击上方「开始压制」按钮' }}</p>
    </div>

    <template v-else>
      <div class="progress-row">
        <div class="progress-track">
          <div class="progress-fill" :style="{ width: `${Math.min(100, Math.max(0, percent))}%` }"></div>
        </div>
        <span class="progress-percent">{{ percent.toFixed(1) }}%</span>
      </div>

      <div class="eta-pills" v-if="(elapsedSeconds ?? 0) > 0">
        <span class="eta-pill" v-tooltip="'壁钟耗时：从开始压制到现在的真实流逝时间'">
          <em>已用</em>
          <span>{{ formatTime(elapsedSeconds) }}</span>
        </span>
        <span class="eta-pill highlight" v-tooltip="'剩余 = (视频总时长 - 已压制) / 当前速度'">
          <em>剩余</em>
          <span>{{ formatTimeOrDash(remainingSeconds) }}</span>
        </span>
        <span class="eta-pill estimate" v-tooltip="'预计总耗时 = 已用 + 剩余（按当前平滑速度估算）'">
          <em>预计</em>
          <span>{{ formatTimeOrDash(etaSeconds) }}</span>
        </span>
      </div>

      <div class="progress-meta" v-if="(elapsedSeconds ?? 0) > 0">
        <div class="meta-item">
          <span class="meta-label">时长</span>
          <span class="meta-value">{{ formatTime(currentSeconds) }} / {{ formatTime(durationSeconds) }}</span>
        </div>
        <div class="meta-item">
          <span class="meta-label">大小</span>
          <span class="meta-value">
            {{ formatSize(sizeKb) }}
            <small v-if="estimatedSizeKb">/ 预估 {{ formatSize(estimatedSizeKb) }}</small>
          </span>
        </div>
        <div class="meta-item" v-if="speed">
          <span class="meta-label">速度</span>
          <span class="meta-value">{{ speed.toFixed(2) }}x</span>
        </div>
        <div class="meta-item" v-if="fps">
          <span class="meta-label">FPS</span>
          <span class="meta-value">{{ fps.toFixed(1) }}</span>
        </div>
        <div class="meta-item" v-if="bitrateKbps">
          <span class="meta-label">码率</span>
          <span class="meta-value">{{ bitrateKbps.toFixed(0) }} kbps</span>
        </div>
      </div>

      <div v-if="statusLine" class="status-line" v-tooltip="statusLine">{{ statusLine }}</div>

      <div class="log-lines">
        <p v-for="(line, index) in lines" :key="index">{{ line }}</p>
        <p v-if="!lines.length" class="muted">暂无日志</p>
      </div>
    </template>
  </section>
</template>
