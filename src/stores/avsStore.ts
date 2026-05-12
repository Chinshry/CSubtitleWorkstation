import { computed, ref } from 'vue'
import type { AvsStatus } from '../types'
import { detectAvs } from '../api/avs'

const realStatus = ref<AvsStatus | null>(null)
let initPromise: Promise<void> | null = null

// === 调试 mock 层 ===
// 与 ffmpegStore 同思路：localStorage 持久化两个独立开关。
const MOCK_AVISYNTH_KEY = 'csubtitle-workstation:debug-mock-no-avisynth'
const MOCK_DEMUXER_KEY = 'csubtitle-workstation:debug-mock-no-avs-demuxer'

function readBoolStorage(key: string): boolean {
  if (typeof localStorage === 'undefined') return false
  return localStorage.getItem(key) === '1'
}

const avisynthMissingMock = ref(readBoolStorage(MOCK_AVISYNTH_KEY))
const demuxerMissingMock = ref(readBoolStorage(MOCK_DEMUXER_KEY))

export const isAvisynthMissingMocked = computed(() => avisynthMissingMock.value)
export const isAvsDemuxerMissingMocked = computed(() => demuxerMissingMock.value)
export const isAvsMocked = computed(
  () => avisynthMissingMock.value || demuxerMissingMock.value
)

function persistBool(key: string, value: boolean) {
  if (typeof localStorage === 'undefined') return
  if (value) localStorage.setItem(key, '1')
  else localStorage.removeItem(key)
}

export function setAvisynthMissingMock(value: boolean) {
  avisynthMissingMock.value = value
  persistBool(MOCK_AVISYNTH_KEY, value)
}

export function setAvsDemuxerMissingMock(value: boolean) {
  demuxerMissingMock.value = value
  persistBool(MOCK_DEMUXER_KEY, value)
}

export function clearAllAvsMocks() {
  setAvisynthMissingMock(false)
  setAvsDemuxerMissingMock(false)
}

// 暴露给 UI 的最终状态：mock 在真实状态基础上叠加修改
export const avsStatus = computed<AvsStatus | null>(() => {
  const real = realStatus.value
  if (!real) return real

  if (!avisynthMissingMock.value && !demuxerMissingMock.value) {
    return real
  }

  const next: AvsStatus = { ...real }
  if (avisynthMissingMock.value) {
    next.avisynthInstalled = false
    next.avisynthVersion = undefined
    next.avisynthInstallPath = undefined
    next.avisynthDllPath = undefined
  }
  if (demuxerMissingMock.value) {
    next.ffmpegDemuxerAvailable = false
  }
  next.available = next.ffmpegDemuxerAvailable && next.avisynthInstalled
  const parts: string[] = []
  if (avisynthMissingMock.value) parts.push('AviSynth+ 缺失')
  if (demuxerMissingMock.value) parts.push('ffmpeg avisynth demuxer 缺失')
  next.message = `[调试] 模拟 ${parts.join(' + ')}`
  return next
})

// 首次调用时检测；后续调用复用结果。ffmpeg 状态切换后请手动 refresh。
export async function initAvsStatus(): Promise<void> {
  if (realStatus.value) return
  if (initPromise) return initPromise
  initPromise = (async () => {
    try {
      realStatus.value = await detectAvs()
    } catch {
      // 静默；UI 仍可重试
    } finally {
      initPromise = null
    }
  })()
  return initPromise
}

export async function refreshAvsStatus(): Promise<AvsStatus | null> {
  try {
    const next = await detectAvs()
    realStatus.value = next
    return next
  } catch {
    return realStatus.value
  }
}
