<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch, computed } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { cancelCompress, previewFfmpegCommand, startCompress } from '../api/compress'
import { loadConfig, saveConfig } from '../api/config'
import { inspectVideoMeta, clearFrameCache } from '../api/video'
import { pendingDrop, pushDiag } from '../stores/dropStore'
import { ffmpegStatus, initFfmpegStatus, refreshFfmpegStatus, shouldHideFfprobeOnlyFields } from '../stores/ffmpegStore'
import type {
  AppConfig,
  CompressJob,
  CompressStatus,
  LogoLayout,
  LogoLayoutEntry,
  RecentLogo,
  VideoMeta
} from '../types'

import CompressForm from '../components/CompressForm.vue'
import JobLogPanel from '../components/JobLogPanel.vue'
import VideoMetaCard from '../components/VideoMetaCard.vue'
import CommandPreviewCard from '../components/CommandPreviewCard.vue'
import LogoEditor from '../components/LogoEditor.vue'
import ColorMatrixWarningBanner from '../components/ColorMatrixWarningBanner.vue'
import type { SubtitleAnalysisResult } from '../api/compress'
import { checkColorMatrix } from '../utils/colorMatrix'

const loading = ref(false)
const running = ref(false)
// 主动取消标志：仅作徽章语义（取消中 / 已取消）。状态收尾交给 compress-log 监听器统一处理。
const cancelled = ref(false)
// 命令预览面板默认折叠；通过 actions 行内的开关按钮显示/隐藏，避免占用首屏纵向空间
const showCommandPreview = ref(false)
// ffmpeg 状态来自全局 store（首次 init 后缓存，避免每次切页面重复检测；调试 mock 也通过 store 透传过来）
const command = ref<string[]>([])
const logs = ref<string[]>([])
const percent = ref(0)
const statusLine = ref('')
const currentSeconds = ref(0)
const durationSeconds = ref(0)
const sizeKb = ref(0)
const speed = ref(0)
const fps = ref(0)
const bitrateKbps = ref(0)
const unlisteners: UnlistenFn[] = []
const lastAutoOutput = ref('')
// 记录上一次的 videoPath，用于在视频变化时根据用户对 outputPath 的修改推断"后缀模板"
const prevVideoPath = ref('')
const appConfig = ref<AppConfig | null>(null)
// LOGO 编辑器状态
const logoEditorOpen = ref(false)
const recentLogos = ref<RecentLogo[]>([])
// 按 (分辨率桶, LOGO 路径) 维度独立记忆的布局列表
const logoLayouts = ref<LogoLayoutEntry[]>([])
let logoConfigSaveTimer: ReturnType<typeof setTimeout> | null = null

// 壁钟耗时 / 平滑速度 / ETA
const startedAt = ref<number | null>(null)
const elapsedSeconds = ref(0)
const smoothSpeed = ref(0)
let elapsedTicker: ReturnType<typeof setInterval> | null = null

const remainingSeconds = computed(() => {
  const dur = durationSeconds.value
  const cur = currentSeconds.value
  const sp = smoothSpeed.value
  if (!dur || !sp || sp <= 0 || cur >= dur) return 0
  return Math.max(0, (dur - cur) / sp)
})

const etaSeconds = computed(() => {
  if (!remainingSeconds.value) return 0
  return elapsedSeconds.value + remainingSeconds.value
})

function startElapsedTicker() {
  stopElapsedTicker()
  elapsedTicker = setInterval(() => {
    if (startedAt.value !== null) {
      elapsedSeconds.value = (Date.now() - startedAt.value) / 1000
    }
  }, 500)
}

function stopElapsedTicker() {
  if (elapsedTicker) {
    clearInterval(elapsedTicker)
    elapsedTicker = null
  }
}

const videoMeta = ref<VideoMeta | null>(null)
const videoMetaLoading = ref(false)
const videoMetaError = ref('')
let videoMetaTimer: ReturnType<typeof setTimeout> | null = null
let videoMetaSeq = 0

// 展示用元数据：ffprobe 缺失模拟开启时，抹掉 ffprobe-only 字段，让 UI 真实演练降级表现
const displayVideoMeta = computed<VideoMeta | null>(() => {
  const m = videoMeta.value
  if (!m) return m
  if (!shouldHideFfprobeOnlyFields.value) return m
  return {
    ...m,
    frameRateMode: undefined,
    totalFrames: undefined
  }
})

const job = ref<CompressJob>(createJob())

// 字幕分析结果（来自 CompressForm 的 emit，避免重复调用后端 analyze_subtitle）
const subtitleAnalysis = ref<SubtitleAnalysisResult | null>(null)

// ASS YCbCr Matrix 与视频色域/色范围匹配检查
const colorMatrixCheck = computed(() => {
  const subtitlePath = job.value.subtitlePath ?? ''
  // 仅 ASS/SSA 才有此字段；其它字幕格式跳过
  const isAssLike = /\.(ass|ssa)$/i.test(subtitlePath)
  return checkColorMatrix(
    subtitleAnalysis.value?.assMatrix,
    videoMeta.value?.colorSpace,
    videoMeta.value?.colorRange,
    isAssLike,
  )
})

function onSubtitleAnalyzed(result: SubtitleAnalysisResult | null) {
  subtitleAnalysis.value = result
}

const logoButtonDisabled = computed(() => {
  return !job.value.videoPath || !videoMeta.value?.width || !videoMeta.value?.height
})

const logoButtonDisabledReason = computed(() => {
  if (!job.value.videoPath) return '请先选择视频文件'
  if (!videoMeta.value?.width || !videoMeta.value?.height) return '视频分辨率未解析完毕'
  return ''
})

function createJob(): CompressJob {
  return {
    id: crypto.randomUUID(),
    videoPath: '',
    subtitlePath: '',
    outputPath: '',
    crf: 18,
    maxBitrate: undefined,
    needLogo: true,
    needYadif: false,
    encoder: 'libx264',
    useAvs: false,
    logoLayout: null,
    logoOnTop: false
  }
}

function formatError(error: unknown): string {
  if (typeof error === 'string') return error
  const msg = (error as { message?: unknown })?.message
  if (typeof msg === 'string') return msg
  try {
    return JSON.stringify(error)
  } catch {
    return String(error)
  }
}

function buildDefaultOutput(videoPath: string): string {
  if (!videoPath) return ''
  const parts = splitVideoPath(videoPath)
  if (!parts) return ''
  return joinOutput(parts.dir, parts.sep, `${parts.stem} output`, '.mp4')
}

function splitVideoPath(p: string): { dir: string; sep: string; stem: string; ext: string } | null {
  if (!p) return null
  const sep = p.includes('\\') ? '\\' : '/'
  const idx = Math.max(p.lastIndexOf('\\'), p.lastIndexOf('/'))
  const dir = idx >= 0 ? p.slice(0, idx) : ''
  const file = idx >= 0 ? p.slice(idx + 1) : p
  const dotIdx = file.lastIndexOf('.')
  const stem = dotIdx > 0 ? file.slice(0, dotIdx) : file
  const ext = dotIdx > 0 ? file.slice(dotIdx) : ''
  return { dir, sep, stem, ext }
}

function joinOutput(dir: string, sep: string, stem: string, ext: string): string {
  const file = `${stem}${ext}`
  return dir ? `${dir}${sep}${file}` : file
}

// 用上次的视频 stem 解析当前 outputPath，拆出"用户后缀 + 扩展名"模板。
// 例：上次 video stem="a", outputPath="D:\v\a 中字.mp4" → 模板 { suffix: " 中字", ext: ".mp4" }
// 如果 outputPath 不以"上次 stem"开头（用户完全自定义命名），返回 null 表示不再联动。
function extractTemplate(
  outputPath: string,
  prevVideoPath: string
): { suffix: string; ext: string } | null {
  const prevParts = splitVideoPath(prevVideoPath)
  const outParts = splitVideoPath(outputPath)
  if (!prevParts || !outParts) return null
  if (!outParts.stem.startsWith(prevParts.stem)) return null
  const suffix = outParts.stem.slice(prevParts.stem.length)
  return { suffix, ext: outParts.ext || '.mp4' }
}

async function refreshFfmpeg() {
  loading.value = true
  try {
    const next = await refreshFfmpegStatus()
    pushDiag(`detectFfmpeg ok, available=${next?.available}`)
  } catch (error) {
    const msg = formatError(error)
    logs.value.push(msg)
    pushDiag(`detectFfmpeg failed: ${msg}`)
  } finally {
    loading.value = false
  }
}

async function previewCommand() {
  command.value = await previewFfmpegCommand(job.value)
  pushDiag(`preview command built, ${command.value.length} args`)
}

// 实时预览：参数变更 → debounce 300ms → 重新构建命令
let previewTimer: ReturnType<typeof setTimeout> | null = null
watch(
  () => job.value,
  () => {
    if (running.value) return  // 压制运行中不刷新预览，避免覆盖正在使用的命令
    if (previewTimer) clearTimeout(previewTimer)
    previewTimer = setTimeout(async () => {
      if (!job.value.videoPath) {
        command.value = []
        return
      }
      try {
        command.value = await previewFfmpegCommand(job.value)
      } catch {
        // 预览失败静默——可能是 ffmpeg 未配置，开始压制时会有明确报错
      }
    }, 300)
  },
  { deep: true }
)

async function runJob() {
  pushDiag('runJob clicked')
  logs.value = []
  percent.value = 0
  statusLine.value = ''
  currentSeconds.value = 0
  durationSeconds.value = 0
  sizeKb.value = 0
  speed.value = 0
  fps.value = 0
  bitrateKbps.value = 0
  elapsedSeconds.value = 0
  smoothSpeed.value = 0
  if (!job.value.videoPath) {
    const msg = '错误：视频路径为空，请先填写或拖入视频文件'
    logs.value.push(msg)
    pushDiag(msg)
    return
  }
  running.value = true
  cancelled.value = false
  // 不立即启动 ticker，等收到第一条进度事件再启动
  startedAt.value = null
  try {
    pushDiag('invoking preview_ffmpeg_command...')
    await previewCommand()
    pushDiag('invoking start_compress...')
    await startCompress(job.value)
    pushDiag('start_compress returned OK; waiting for compress-log events')
  } catch (error) {
    running.value = false
    stopElapsedTicker()
    const msg = formatError(error)
    logs.value.push(`runJob 异常：${msg}`)
    pushDiag(`runJob exception: ${msg}`)
  }
}

async function cancelJob() {
  try {
    await cancelCompress(job.value.id)
    cancelled.value = true
    logs.value.push('已发送取消请求')
    // 注意：不在此处置 running=false / stopElapsedTicker。
    // ffmpeg 收到 q 后要花时间写文件尾，期间仍在运行；
    // 状态收尾统一由 compress-log 中匹配 "Compression (completed|failed|exited)" 的监听器处理，
    // 避免出现"短暂回到待开始再跳到已完成"的闪烁。
  } catch (error) {
    logs.value.push(formatError(error))
    // 取消 RPC 失败（通常是 jobs 表已空，任务实际已结束）：直接复位本地状态。
    running.value = false
    if (startedAt.value !== null) {
      elapsedSeconds.value = (Date.now() - startedAt.value) / 1000
    }
    stopElapsedTicker()
  }
}

watch(
  () => job.value.videoPath,
  (newVal) => {
    const newParts = splitVideoPath(newVal)
    if (!newParts) {
      if (job.value.outputPath === lastAutoOutput.value) {
        job.value.outputPath = ''
      }
      lastAutoOutput.value = ''
      prevVideoPath.value = ''
      return
    }

    // 1) 没填过输出路径或仍是上次自动生成的值 → 直接用默认后缀 "output"
    if (!job.value.outputPath || job.value.outputPath === lastAutoOutput.value) {
      const next = joinOutput(newParts.dir, newParts.sep, `${newParts.stem} output`, '.mp4')
      job.value.outputPath = next
      lastAutoOutput.value = next
      prevVideoPath.value = newVal
      return
    }

    // 2) 用户改过 outputPath：尝试从"上次视频 stem + 当前 outputPath"提取后缀模板，
    //    套用到新视频的 dir + stem 上，让自定义后缀（如 "中字"）跟随新视频生效。
    const tpl = extractTemplate(job.value.outputPath, prevVideoPath.value)
    if (tpl) {
      const next = joinOutput(newParts.dir, newParts.sep, `${newParts.stem}${tpl.suffix}`, tpl.ext)
      job.value.outputPath = next
      lastAutoOutput.value = next
    }
    // 3) 完全自定义命名（outputPath 与上次 stem 无关）：不动，尊重用户意图
    prevVideoPath.value = newVal
  }
)

// 消费 App.vue 全局拖拽事件
watch(pendingDrop, (drop) => {
  if (!drop) return
  pushDiag(`HomeView consumed drop: video=${drop.videoPath ?? '-'} subtitle=${drop.subtitlePath ?? '-'}`)
  if (drop.videoPath) job.value.videoPath = drop.videoPath
  if (drop.subtitlePath) job.value.subtitlePath = drop.subtitlePath
})

// videoPath 改动 → 防抖 350ms 调用 inspect_video_meta 刷新视频信息卡片
watch(
  () => job.value.videoPath,
  (newVal) => {
    if (videoMetaTimer) clearTimeout(videoMetaTimer)
    const path = (newVal ?? '').trim()
    if (!path) {
      videoMeta.value = null
      videoMetaError.value = ''
      videoMetaLoading.value = false
      return
    }
    videoMetaTimer = setTimeout(async () => {
      const seq = ++videoMetaSeq
      videoMetaLoading.value = true
      videoMetaError.value = ''
      try {
        const meta = await inspectVideoMeta(path)
        if (seq !== videoMetaSeq) return // 已被更新的请求取代
        videoMeta.value = meta
      } catch (err) {
        if (seq !== videoMetaSeq) return
        videoMeta.value = null
        videoMetaError.value = formatError(err)
      } finally {
        if (seq === videoMetaSeq) videoMetaLoading.value = false
      }
    }, 350)
  },
  { immediate: false }
)

// 把 videoMeta 的"显示尺寸"（后端已应用 rotation）同步到 job，
// 让 command_builder 用同一份尺寸做 LOGO overlay 像素换算，避免横竖屏旋转视频压制时 LOGO 尺寸错位。
watch(
  () => [videoMeta.value?.width, videoMeta.value?.height] as const,
  ([w, h]) => {
    job.value.videoWidth = typeof w === 'number' && w > 0 ? w : undefined
    job.value.videoHeight = typeof h === 'number' && h > 0 ? h : undefined
  },
  { immediate: true }
)

// logoLayout 改动 → debounce 同步到 AppConfig.lastLogoLayout / logoLayouts / recentLogos
watch(
  () => job.value.logoLayout,
  (newVal) => {
    if (!appConfig.value) return
    if (logoConfigSaveTimer) clearTimeout(logoConfigSaveTimer)
    logoConfigSaveTimer = setTimeout(async () => {
      const cfg: AppConfig = {
        ...(appConfig.value as AppConfig),
        lastLogoLayout: newVal ?? null,
        recentLogos: recentLogos.value,
        logoLayouts: logoLayouts.value
      }
      try {
        await saveConfig(cfg)
        appConfig.value = cfg
        pushDiag('LOGO 布局已保存为默认配置')
      } catch (err) {
        pushDiag(`保存 LOGO 布局失败：${formatError(err)}`)
      }
    }, 400)
  },
  { deep: true }
)

// LOGO 编辑器：打开 / 保存 / 取消
function openLogoEditor() {
  if (logoButtonDisabled.value) return
  logoEditorOpen.value = true
}

function onLogoEditorSave(
  layout: LogoLayout,
  nextRecent: RecentLogo[],
  nextLogoLayouts: LogoLayoutEntry[]
) {
  job.value.logoLayout = layout
  recentLogos.value = nextRecent
  logoLayouts.value = nextLogoLayouts
  logoEditorOpen.value = false
  pushDiag(`LOGO 配置已保存：${layout.path}`)
  // 关闭时清理后端抽帧缓存（异步，失败可忽略）
  void clearFrameCache().catch(() => undefined)
}

function onLogoEditorCancel() {
  logoEditorOpen.value = false
  void clearFrameCache().catch(() => undefined)
}

// 即时同步最近 LOGO 列表（删除场景）；HomeView 的 watch 会 debounce 写回 AppConfig。
function onLogoEditorUpdateRecent(next: RecentLogo[]) {
  recentLogos.value = next
}

onMounted(async () => {
  pushDiag('HomeView mounted')
  // 先取配置，把默认 LOGO 布局/最近列表填入状态
  try {
    appConfig.value = await loadConfig()
    if (appConfig.value?.lastLogoLayout) {
      job.value.logoLayout = appConfig.value.lastLogoLayout
      pushDiag(`已载入上次 LOGO 布局：${appConfig.value.lastLogoLayout.path}`)
    }
    if (Array.isArray(appConfig.value?.recentLogos)) {
      recentLogos.value = appConfig.value.recentLogos
    }
    if (Array.isArray(appConfig.value?.logoLayouts)) {
      logoLayouts.value = appConfig.value.logoLayouts
    }
    if (typeof appConfig.value?.defaultUseAvs === 'boolean') {
      job.value.useAvs = appConfig.value.defaultUseAvs
    }
  } catch (err) {
    pushDiag(`loadConfig failed: ${formatError(err)}`)
  }
  await initFfmpegStatus()

  unlisteners.push(
    await listen<string>('compress-log', (event) => {
      logs.value.push(event.payload)
      // 匹配 "Compression completed" / "Compression failed" / "Compression exited"
      if (/Compression (completed|failed|exited)/.test(event.payload)) {
        running.value = false
        // 最后一次 tick 刷一下 elapsed，停 ticker
        if (startedAt.value !== null) {
          elapsedSeconds.value = (Date.now() - startedAt.value) / 1000
        }
        stopElapsedTicker()
      }
    })
  )
  unlisteners.push(
    await listen<CompressStatus>('compress-status', (event) => {
      if (event.payload.jobId !== job.value.id) return
      const p = event.payload

      // 第一次收到进度时启动计时器
      if (startedAt.value === null) {
        startedAt.value = Date.now()
        startElapsedTicker()
      }

      statusLine.value = p.statusLine ?? ''
      if (typeof p.percent === 'number') percent.value = p.percent
      if (typeof p.currentSeconds === 'number') currentSeconds.value = p.currentSeconds
      if (typeof p.durationSeconds === 'number') durationSeconds.value = p.durationSeconds
      if (typeof p.sizeKb === 'number') sizeKb.value = p.sizeKb
      if (typeof p.speed === 'number') {
        speed.value = p.speed
        if (p.speed > 0) {
          // 指数移动平均：起步用首个有效值；后续按 0.7/0.3 平滑
          smoothSpeed.value = smoothSpeed.value > 0
            ? smoothSpeed.value * 0.7 + p.speed * 0.3
            : p.speed
        }
      }
      if (typeof p.fps === 'number') fps.value = p.fps
      if (typeof p.bitrateKbps === 'number') bitrateKbps.value = p.bitrateKbps
    })
  )
  pushDiag('compress-log / compress-status listeners installed')

  // 如果在挂载前已经有拖入文件，立刻消费一次
  if (pendingDrop.value) {
    const drop = pendingDrop.value
    if (drop.videoPath) job.value.videoPath = drop.videoPath
    if (drop.subtitlePath) job.value.subtitlePath = drop.subtitlePath
  }
})

onUnmounted(() => {
  for (const unlisten of unlisteners) unlisten()
  stopElapsedTicker()
})
</script>

<template>
  <main class="workspace">
    <div v-if="!loading && ffmpegStatus && !ffmpegStatus.available" class="ffmpeg-missing">
      <strong>未检测到 ffmpeg</strong>
      <span>请前往左侧「设置」面板配置 ffmpeg 路径，或安装后将其加入系统 PATH。</span>
      <button class="secondary" @click="refreshFfmpeg">重新检测</button>
    </div>

    <VideoMetaCard
      :meta="displayVideoMeta"
      :loading="videoMetaLoading"
      :error="videoMetaError"
      :video-path="job.videoPath"
      :subtitle-path="job.subtitlePath"
      v-model:output-path="job.outputPath"
      @clear-video="job.videoPath = ''"
      @clear-subtitle="job.subtitlePath = ''"
      @pick-video="(p: string) => (job.videoPath = p)"
      @pick-subtitle="(p: string) => (job.subtitlePath = p)"
    />
    <ColorMatrixWarningBanner :check="colorMatrixCheck" />
    <CompressForm
      v-model="job"
      :logo-button-disabled="logoButtonDisabled"
      :logo-button-disabled-reason="logoButtonDisabledReason"
      @open-logo-editor="openLogoEditor"
      @subtitle-analyzed="onSubtitleAnalyzed"
    />

    <CommandPreviewCard v-if="command.length && showCommandPreview" :command="command" />

    <section class="actions">
      <button
        type="button"
        class="secondary command-toggle"
        :class="{ active: showCommandPreview }"
        :disabled="!command.length"
        :title="command.length ? '' : '等待视频与参数就绪后自动生成命令'"
        @click="showCommandPreview = !showCommandPreview"
      >
        {{ showCommandPreview ? '隐藏命令预览' : '显示命令预览' }}
      </button>
      <button v-if="running" class="danger" @click="cancelJob">取消压制</button>
      <button v-else :disabled="!ffmpegStatus?.available" @click="runJob">开始压制</button>
    </section>

    <JobLogPanel
      :lines="logs"
      :command="command"
      :percent="percent"
      :status-line="statusLine"
      :current-seconds="currentSeconds"
      :duration-seconds="durationSeconds"
      :size-kb="sizeKb"
      :speed="speed"
      :fps="fps"
      :bitrate-kbps="bitrateKbps"
      :elapsed-seconds="elapsedSeconds"
      :eta-seconds="etaSeconds"
      :remaining-seconds="remainingSeconds"
      :running="running"
      :cancelled="cancelled"
    />

    <LogoEditor
      v-if="logoEditorOpen"
      :video-path="job.videoPath"
      :video-width="videoMeta?.width"
      :video-height="videoMeta?.height"
      :video-duration="videoMeta?.durationSeconds"
      :initial-layout="job.logoLayout"
      :recent-logos="recentLogos"
      :logo-layouts="logoLayouts"
      @save="onLogoEditorSave"
      @cancel="onLogoEditorCancel"
      @update-recent="onLogoEditorUpdateRecent"
    />
  </main>
</template>
