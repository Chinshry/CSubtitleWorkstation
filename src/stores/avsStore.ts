import { computed, ref } from 'vue'
import type { AvsStatus, LavFiltersStatus } from '../types'
import { detectAvs, detectLavFilters } from '../api/avs'

const realStatus = ref<AvsStatus | null>(null)
const realLavStatus = ref<LavFiltersStatus | null>(null)
export const avsChecking = ref(false)
export const lavChecking = ref(false)
export const lavStatusLoaded = computed(() => realLavStatus.value !== null)
let initPromise: Promise<void> | null = null
let lavInitPromise: Promise<void> | null = null

const MOCK_AVISYNTH_KEY = 'csubtitle-workstation:debug-mock-no-avisynth'
const MOCK_DEMUXER_KEY = 'csubtitle-workstation:debug-mock-no-avs-demuxer'
const MOCK_LAV_FILTERS_KEY = 'csubtitle-workstation:debug-mock-no-lav-filters'

function readBoolStorage(key: string): boolean {
  if (typeof localStorage === 'undefined') return false
  return localStorage.getItem(key) === '1'
}

const avisynthMissingMock = ref(readBoolStorage(MOCK_AVISYNTH_KEY))
const demuxerMissingMock = ref(readBoolStorage(MOCK_DEMUXER_KEY))
const lavFiltersMissingMock = ref(readBoolStorage(MOCK_LAV_FILTERS_KEY))

export const isAvisynthMissingMocked = computed(() => avisynthMissingMock.value)
export const isAvsDemuxerMissingMocked = computed(() => demuxerMissingMock.value)
export const isLavFiltersMissingMocked = computed(() => lavFiltersMissingMock.value)
export const isAvsMocked = computed(
  () => avisynthMissingMock.value || demuxerMissingMock.value || lavFiltersMissingMock.value
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

export function setLavFiltersMissingMock(value: boolean) {
  lavFiltersMissingMock.value = value
  persistBool(MOCK_LAV_FILTERS_KEY, value)
}

export function clearAllAvsMocks() {
  setAvisynthMissingMock(false)
  setAvsDemuxerMissingMock(false)
  setLavFiltersMissingMock(false)
}

export const avsStatus = computed<AvsStatus | null>(() => {
  const real = realStatus.value
  if (!real) return real

  const lav = realLavStatus.value
  let next: AvsStatus = lav
    ? {
        ...real,
        lavFiltersInstalled: lav.lavFiltersInstalled,
        lavFiltersVersion: lav.lavFiltersVersion,
        lavFiltersInstallPath: lav.lavFiltersInstallPath,
        lavFiltersX64Available: lav.lavFiltersX64Available,
        lavFiltersDirectshowRegistered: lav.lavFiltersDirectshowRegistered,
      }
    : real

  if (!avisynthMissingMock.value && !demuxerMissingMock.value && !lavFiltersMissingMock.value) {
    return next
  }

  next = { ...next }
  if (avisynthMissingMock.value) {
    next.avisynthInstalled = false
    next.avisynthVersion = undefined
    next.avisynthInstallPath = undefined
    next.avisynthDllPath = undefined
  }
  if (demuxerMissingMock.value) {
    next.ffmpegDemuxerAvailable = false
  }
  if (lavFiltersMissingMock.value) {
    next.lavFiltersInstalled = false
    next.lavFiltersVersion = undefined
    next.lavFiltersInstallPath = undefined
    next.lavFiltersX64Available = false
    next.lavFiltersDirectshowRegistered = false
  }

  next.available = next.ffmpegDemuxerAvailable && next.avisynthInstalled
  const parts: string[] = []
  if (avisynthMissingMock.value) parts.push('AviSynth+ 缺失')
  if (demuxerMissingMock.value) parts.push('ffmpeg avisynth demuxer 缺失')
  if (lavFiltersMissingMock.value) parts.push('LAV Filters 缺失')
  next.message = `[调试] 模拟 ${parts.join(' + ')}`
  return next
})

export async function initAvsStatus(): Promise<void> {
  if (realStatus.value) return
  if (initPromise) return initPromise
  initPromise = (async () => {
    avsChecking.value = true
    try {
      realStatus.value = await detectAvs()
    } catch {
      // UI can still retry.
    } finally {
      avsChecking.value = false
      initPromise = null
    }
  })()
  return initPromise
}

export async function initLavFiltersStatus(): Promise<void> {
  if (realLavStatus.value) return
  if (lavInitPromise) return lavInitPromise
  lavInitPromise = (async () => {
    lavChecking.value = true
    try {
      realLavStatus.value = await detectLavFilters()
    } catch {
      // UI can still retry.
    } finally {
      lavChecking.value = false
      lavInitPromise = null
    }
  })()
  return lavInitPromise
}

export async function refreshAvsStatus(): Promise<AvsStatus | null> {
  avsChecking.value = true
  try {
    const next = await detectAvs()
    realStatus.value = next
    return next
  } catch {
    return realStatus.value
  } finally {
    avsChecking.value = false
  }
}

export async function refreshLavFiltersStatus(): Promise<LavFiltersStatus | null> {
  lavChecking.value = true
  try {
    const next = await detectLavFilters()
    realLavStatus.value = next
    return next
  } catch {
    return realLavStatus.value
  } finally {
    lavChecking.value = false
  }
}
