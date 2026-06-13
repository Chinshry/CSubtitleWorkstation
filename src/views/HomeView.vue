<script setup lang="ts">
import { onActivated, onMounted, onUnmounted, ref, watch, computed } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { cancelCompress, inspectAvsStagingPlan, previewFfmpegCommand, startCompress } from '../api/compress'
import { loadConfig, saveConfig } from '../api/config'
import { inspectVideoMeta, clearFrameCache } from '../api/video'
import { pendingDrop, pushDiag } from '../stores/dropStore'
import { currentVideoPath } from '../stores/currentJobStore'
import { configRevision } from '../stores/configStore'
import { ffmpegChecking, ffmpegStatus, initFfmpegStatus, refreshFfmpegStatus, shouldHideFfprobeOnlyFields } from '../stores/ffmpegStore'
import type {
  AppConfig,
  CompressJob,
  CompressStatus,
  LogoLayout,
  LogoLayoutEntry,
  RecentLogo,
  VideoEncodePreset,
  VideoMeta
} from '../types'

import CompressForm from '../components/CompressForm.vue'
import JobLogPanel from '../components/JobLogPanel.vue'
import VideoMetaCard from '../components/VideoMetaCard.vue'
import CommandPreviewCard from '../components/CommandPreviewCard.vue'
import LogoEditor from '../components/LogoEditor.vue'
import SubtitleCheckPanel from '../components/SubtitleCheckPanel.vue'
import type { AvsStagingPlan, SubtitleAnalysisResult } from '../api/compress'
import { checkColorMatrix } from '../utils/colorMatrix'
import { buildOutputPath, getDefaultOutputTemplate, normalizeOutputTemplates } from '../utils/outputTemplates'
import { applyEncodePresetToJob, getDefaultEncodePreset, normalizeEncodePresets } from '../utils/encodePresets'

const loading = ref(false)
const running = ref(false)
// 主动取消标志：仅作徽章语义（取消中 / 已取消）。状态收尾交给 compress-log 监听器统一处理。
const cancelled = ref(false)
// 命令预览面板默认折叠；通过 actions 行内的开关按钮显示/隐藏，避免占用首屏纵向空间
const showCommandPreview = ref(false)
// ffmpeg 状态来自全局 store（首次 init 后缓存，避免每次切页面重复检测；调试 mock 也通过 store 透传过来）
const command = ref<string[]>([])
const logs = ref<string[]>([])
const avsStagingPlan = ref<AvsStagingPlan | null>(null)
let avsStagingResolve: ((value: boolean) => void) | null = null
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
const appConfig = ref<AppConfig | null>(null)
const selectedOutputTemplateId = ref('default')
const selectedEncodePresetId = ref('balanced-x264')
// LOGO 编辑器状态
const logoEditorOpen = ref(false)
const recentLogos = ref<RecentLogo[]>([])
// 按 (分辨率桶, LOGO 路径) 维度独立记忆的布局列表
const logoLayouts = ref<LogoLayoutEntry[]>([])
let logoConfigSaveTimer: ReturnType<typeof setTimeout> | null = null
let homeReady = false

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
const isInitialDropOnly = computed(() => !job.value.videoPath?.trim() && !job.value.subtitlePath?.trim())

watch(
  () => job.value.videoPath,
  (path) => {
    currentVideoPath.value = path ?? ''
  },
  { immediate: true }
)

// 字幕分析结果（来自 CompressForm 的 emit，避免重复调用后端 analyze_subtitle）
const subtitleAnalysis = ref<SubtitleAnalysisResult | null>(null)
const subtitleAnalyzing = ref(false)
const outputTemplates = computed(() => normalizeOutputTemplates(appConfig.value))
const encodePresets = computed(() => normalizeEncodePresets(appConfig.value))
const selectedOutputTemplate = computed(() => {
  return outputTemplates.value.find((item) => item.id === selectedOutputTemplateId.value)
    ?? getDefaultOutputTemplate(appConfig.value)
})
const selectedEncodePreset = computed<VideoEncodePreset>(() => {
  return encodePresets.value.find((item) => item.id === selectedEncodePresetId.value)
    ?? getDefaultEncodePreset(appConfig.value)
})

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

function onSubtitleAnalyzing(value: boolean) {
  subtitleAnalyzing.value = value
}

function applyOutputTemplate() {
  if (!job.value.videoPath) return
  job.value.outputPath = buildOutputPath(selectedOutputTemplate.value, job.value, videoMeta.value)
  lastAutoOutput.value = job.value.outputPath
}

function applyOutputTemplateIfAuto() {
  if (!job.value.videoPath) return
  if (job.value.outputPath && job.value.outputPath !== lastAutoOutput.value) return
  applyOutputTemplate()
}

function applyConfigDefaults(config: AppConfig) {
  job.value.crf = config.defaultCrf
  job.value.needLogo = config.defaultNeedLogo
  job.value.needYadif = config.defaultNeedYadif
  if (isSupportedEncoder(config.defaultEncoder)) {
    job.value.encoder = config.defaultEncoder
  }
}

function isSupportedEncoder(value: string): value is CompressJob['encoder'] {
  return ['libx264', 'libx265', 'h264_nvenc', 'h264_amf', 'h264_videotoolbox'].includes(value)
}

function applyEncodePreset(presetId?: string) {
  const preset = presetId
    ? encodePresets.value.find((item) => item.id === presetId) ?? selectedEncodePreset.value
    : selectedEncodePreset.value
  applyEncodePresetToJob(job.value, preset)
  if (presetId) {
    selectedEncodePresetId.value = presetId
  }
  if (appConfig.value && selectedEncodePresetId.value) {
    const next = {
      ...appConfig.value,
      defaultEncodePresetId: selectedEncodePresetId.value,
    }
    appConfig.value = next
    void saveConfig(next)
  }
}

function selectAvailableTemplate(config: AppConfig) {
  const templates = normalizeOutputTemplates(config)
  if (templates.some((item) => item.id === selectedOutputTemplateId.value)) return
  selectedOutputTemplateId.value = config.defaultOutputTemplateId
    || getDefaultOutputTemplate(config).id
}

async function refreshHomeConfig() {
  try {
    const next = await loadConfig()
    appConfig.value = next
    selectAvailableTemplate(next)
    applyOutputTemplateIfAuto()
  } catch (err) {
    pushDiag(`refresh config failed: ${formatError(err)}`)
  }
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
    customVideoArgs: '',
    useAvs: false,
    logoLayout: null,
    logoOnTop: false,
    quickProcess: {
      enabled: false,
      transform: 'none',
      rotation: 'none',
      mirror: 'none',
      scale: 'none',
      customScale: '',
      frameRate: undefined,
      videoBitrateKbps: undefined
    }
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
      } catch (error) {
        pushDiag(`preview command failed: ${formatError(error)}`)
        command.value = []
      }
    }, 300)
  },
  { deep: true }
)

async function runJob() {
  pushDiag('runJob clicked')
  logs.value = []
  percent.value = 0
  statusLine.value = 'Preparing compression job...'
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
  logs.value.push('正在检查视频与 AVS 预处理需求...')
  const canContinue = await confirmAvsStagingIfNeeded()
  if (!canContinue) {
    logs.value.push('已取消压制：VP9 AVS 兼容模式需要临时复制源视频。')
    pushDiag('runJob cancelled by AVS staging confirmation')
    return
  }
  if (job.value.useAvs) {
    logs.value.push('正在准备 AVS 临时文件；如果源视频是 VP9，大文件复制期间 ffmpeg 进度会暂时保持 0%。')
    statusLine.value = 'Preparing AVS temporary files...'
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

async function confirmAvsStagingIfNeeded() {
  const plan = await inspectAvsStagingPlan(job.value)
  if (!plan?.required) return true

  avsStagingPlan.value = plan
  return new Promise<boolean>((resolve) => {
    avsStagingResolve = resolve
  })
}

function resolveAvsStagingConfirm(value: boolean) {
  avsStagingPlan.value = null
  const resolve = avsStagingResolve
  avsStagingResolve = null
  resolve?.(value)
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
      return
    }

    // 没填过输出路径或仍是上次自动生成的值，才按当前模板重新生成。
    // 用户手动改过输出路径后，切换视频不再从旧路径推断后缀。
    if (!job.value.outputPath || job.value.outputPath === lastAutoOutput.value) {
      const tpl = selectedOutputTemplate.value
      const next = buildOutputPath(tpl, job.value, videoMeta.value)
        || joinOutput(newParts.dir, newParts.sep, `${newParts.stem} 中字`, '.mp4')
      job.value.outputPath = next
      lastAutoOutput.value = next
      return
    }
  }
)

watch(configRevision, () => {
  if (!homeReady) return
  void refreshHomeConfig()
})

// 消费 App.vue 全局拖拽事件
watch(pendingDrop, (drop) => {
  if (!drop) return
  if (drop.target !== 'home') return
  pushDiag(`HomeView consumed drop: video=${drop.videoPath ?? '-'} subtitle=${drop.subtitlePath ?? '-'}`)
  if (drop.videoPath) job.value.videoPath = drop.videoPath
  if (drop.subtitlePath) job.value.subtitlePath = drop.subtitlePath
  pendingDrop.value = null
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

// logoLayout 改动 → debounce 同步到 AppConfig.logoLayouts / recentLogos
watch(
  () => job.value.logoLayout,
  (newVal) => {
    if (!appConfig.value) return
    if (logoConfigSaveTimer) clearTimeout(logoConfigSaveTimer)
    logoConfigSaveTimer = setTimeout(async () => {
      const cfg: AppConfig = {
        ...(appConfig.value as AppConfig),
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
    recentLogos.value = appConfig.value.recentLogos
    logoLayouts.value = appConfig.value.logoLayouts
    job.value.useAvs = appConfig.value.defaultUseAvs
    selectedOutputTemplateId.value = appConfig.value.defaultOutputTemplateId
      || getDefaultOutputTemplate(appConfig.value).id
    selectedEncodePresetId.value = appConfig.value.defaultEncodePresetId
      || encodePresets.value[0]?.id
      || getDefaultEncodePreset(appConfig.value).id
    applyConfigDefaults(appConfig.value)
  } catch (err) {
    pushDiag(`loadConfig failed: ${formatError(err)}`)
  }
  void initFfmpegStatus()

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
    if (drop.target === 'home') {
      if (drop.videoPath) job.value.videoPath = drop.videoPath
      if (drop.subtitlePath) job.value.subtitlePath = drop.subtitlePath
      pendingDrop.value = null
    }
  }
  homeReady = true
})

onActivated(() => {
  if (!homeReady) return
  void refreshHomeConfig()
})

onUnmounted(() => {
  for (const unlisten of unlisteners) unlisten()
  stopElapsedTicker()
})
</script>

<template>
  <main class="workspace" :class="{ 'initial-drop-workspace': isInitialDropOnly }">
    <div v-if="!isInitialDropOnly && ffmpegChecking" class="ffmpeg-missing ffmpeg-checking">
      <strong>正在检测 ffmpeg 环境</strong>
      <span>正在检测 ffmpeg / ffprobe / subtitles/libass，请稍候。</span>
    </div>
    <div v-else-if="!isInitialDropOnly && !loading && ffmpegStatus && !ffmpegStatus.available" class="ffmpeg-missing">
      <strong>{{ ffmpegStatus.ffmpegPath ? 'ffmpeg 功能不完整' : '未检测到 ffmpeg' }}</strong>
      <span>{{ ffmpegStatus.message ?? '请前往左侧「设置」面板配置 ffmpeg 路径，或安装后将其加入系统 PATH。' }}</span>
      <button class="secondary" @click="refreshFfmpeg">重新检测</button>
    </div>

    <VideoMetaCard
      :class="{ 'initial-drop-card': isInitialDropOnly }"
      :meta="displayVideoMeta"
      :loading="videoMetaLoading"
      :error="videoMetaError"
      :video-path="job.videoPath"
      :subtitle-path="job.subtitlePath"
      :output-templates="outputTemplates"
      :selected-output-template-id="selectedOutputTemplateId"
      v-model:output-path="job.outputPath"
      @update:selected-output-template-id="selectedOutputTemplateId = $event"
      @apply-output-template="applyOutputTemplate"
      @clear-video="job.videoPath = ''"
      @clear-subtitle="job.subtitlePath = ''"
      @pick-video="(p: string) => (job.videoPath = p)"
      @pick-subtitle="(p: string) => (job.subtitlePath = p)"
    />
    <SubtitleCheckPanel
      v-if="!isInitialDropOnly"
      :matrix-check="colorMatrixCheck"
      :analysis="subtitleAnalysis"
      :analyzing="subtitleAnalyzing"
    />
    <CompressForm
      v-if="!isInitialDropOnly"
      v-model="job"
      :encode-presets="encodePresets"
      :selected-encode-preset-id="selectedEncodePresetId"
      :logo-button-disabled="logoButtonDisabled"
      :logo-button-disabled-reason="logoButtonDisabledReason"
      :video-codec="videoMeta?.videoCodec"
      @update:selected-encode-preset-id="selectedEncodePresetId = $event"
      @apply-encode-preset="applyEncodePreset"
      @open-logo-editor="openLogoEditor"
      @subtitle-analyzing="onSubtitleAnalyzing"
      @subtitle-analyzed="onSubtitleAnalyzed"
    />

    <CommandPreviewCard v-if="!isInitialDropOnly && command.length && showCommandPreview" :command="command" />

    <section v-if="!isInitialDropOnly" class="actions">
      <button
        type="button"
        class="secondary command-toggle"
        :class="{ active: showCommandPreview }"
        :disabled="!command.length"
        v-tooltip="command.length ? '' : '等待视频与参数就绪后自动生成命令'"
        @click="showCommandPreview = !showCommandPreview"
      >
        {{ showCommandPreview ? '隐藏命令预览' : '显示命令预览' }}
      </button>
      <button v-if="running" class="danger" @click="cancelJob">取消压制</button>
      <button v-else :disabled="!ffmpegStatus?.available" @click="runJob">开始压制</button>
    </section>

    <JobLogPanel
      v-if="!isInitialDropOnly"
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

    <div v-if="avsStagingPlan" class="avs-staging-overlay app-modal-active" role="presentation">
      <section class="avs-staging-dialog" role="dialog" aria-modal="true" aria-labelledby="avs-staging-title">
        <div class="avs-staging-icon" aria-hidden="true">!</div>
        <div class="avs-staging-content">
          <p class="avs-staging-kicker">AVS 兼容模式</p>
          <h2 id="avs-staging-title">需要临时复制 VP9 源视频</h2>
          <p class="avs-staging-summary">
            检测到 VP9 视频。本次会先把源视频复制到 ASCII 临时路径，再通过本机 64 位 DirectShow 解码链读取视频；通常需要 64 位 LAV Filters。
          </p>

          <dl class="avs-staging-facts">
            <div>
              <dt>临时占用</dt>
              <dd>{{ avsStagingPlan.sourceSizeLabel }}</dd>
            </div>
            <div class="avs-staging-path">
              <span>临时路径</span>
              <code>{{ avsStagingPlan.tempPath }}</code>
            </div>
          </dl>

          <div class="avs-staging-actions">
            <button class="secondary" type="button" @click="resolveAvsStagingConfirm(false)">取消</button>
            <button type="button" @click="resolveAvsStagingConfirm(true)">继续压制</button>
          </div>
        </div>
      </section>
    </div>
  </main>
</template>
