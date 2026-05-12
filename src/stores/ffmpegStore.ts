import { computed, ref } from 'vue'
import type { FfmpegStatus } from '../types'
import { detectFfmpeg } from '../api/ffmpeg'

// 真实检测结果（仅 store 内部 + 设置面板的 setFfmpegStatus 写入）
const realStatus = ref<FfmpegStatus | null>(null)

// === 调试 mock 层 ===
// 仅用于 UI 测试：两个独立开关，模拟 ffmpeg / ffprobe 缺失场景。
// ffmpeg 缺失语义上隐含 ffprobe 也缺失，UI 上由 shouldHideFfprobeOnlyFields 兜底处理。
const MOCK_FFMPEG_KEY = 'csubtitle-workstation:debug-mock-no-ffmpeg'
const MOCK_FFPROBE_KEY = 'csubtitle-workstation:debug-mock-no-ffprobe'

function readBoolStorage(key: string): boolean {
  if (typeof localStorage === 'undefined') return false
  return localStorage.getItem(key) === '1'
}

const ffmpegMissingMock = ref(readBoolStorage(MOCK_FFMPEG_KEY))
const ffprobeMissingMock = ref(readBoolStorage(MOCK_FFPROBE_KEY))

export const isFfmpegMissingMocked = computed(() => ffmpegMissingMock.value)
export const isFfprobeMissingMocked = computed(() => ffprobeMissingMock.value)
export const isFfmpegMocked = computed(
  () => ffmpegMissingMock.value || ffprobeMissingMock.value
)

// ffmpeg 缺失会自动让 ffprobe 也按缺失处理
export const shouldHideFfprobeOnlyFields = computed(
  () => ffmpegMissingMock.value || ffprobeMissingMock.value
)

function persistBool(key: string, value: boolean) {
  if (typeof localStorage === 'undefined') return
  if (value) localStorage.setItem(key, '1')
  else localStorage.removeItem(key)
}

export function setFfmpegMissingMock(value: boolean) {
  ffmpegMissingMock.value = value
  persistBool(MOCK_FFMPEG_KEY, value)
}

export function setFfprobeMissingMock(value: boolean) {
  ffprobeMissingMock.value = value
  persistBool(MOCK_FFPROBE_KEY, value)
}

export function clearAllFfmpegMocks() {
  setFfmpegMissingMock(false)
  setFfprobeMissingMock(false)
}

// 暴露给 UI 的最终状态：mock 优先于真实
export const ffmpegStatus = computed<FfmpegStatus | null>(() => {
  const real = realStatus.value

  // ffmpeg 缺失：整套 ✕
  if (ffmpegMissingMock.value) {
    return {
      available: false,
      source: 'not_found',
      message: '[调试] 模拟 ffmpeg 未找到'
    }
  }

  // 仅 ffprobe 缺失：在真实状态基础上抹掉 ffprobe 字段
  if (ffprobeMissingMock.value) {
    const base: FfmpegStatus = real ?? {
      available: true,
      source: 'system_path',
      ffmpegPath: '/usr/local/bin/ffmpeg',
      ffmpegVersion: 'ffmpeg version (debug mock)'
    }
    return {
      ...base,
      ffprobePath: undefined,
      ffprobeVersion: undefined,
      message: '[调试] 模拟 ffprobe 缺失（仅影响视频信息精度）'
    }
  }

  return real
})

let initPromise: Promise<void> | null = null

// 首次调用时检测；后续调用复用已检测的结果或正在进行中的 Promise。
export async function initFfmpegStatus(): Promise<void> {
  if (realStatus.value) return
  if (initPromise) return initPromise
  initPromise = (async () => {
    try {
      realStatus.value = await detectFfmpeg()
    } catch {
      // 错误吞掉，UI 上仍可点"重新检测"
    } finally {
      initPromise = null
    }
  })()
  return initPromise
}

// 用户主动刷新 / 改路径后调用，强制重新跑一次 detect。
export async function refreshFfmpegStatus(): Promise<FfmpegStatus | null> {
  try {
    const next = await detectFfmpeg()
    realStatus.value = next
    return next
  } catch {
    return realStatus.value
  }
}

// setFfmpegPath / resetFfmpegToSystem 已经返回新的 FfmpegStatus，直接把结果灌进 store。
export function setFfmpegStatus(next: FfmpegStatus | null) {
  realStatus.value = next
}
