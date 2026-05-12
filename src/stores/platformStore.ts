import { computed, ref } from 'vue'

export type Platform = 'windows' | 'macos' | 'linux' | 'other'

function detectNative(): Platform {
  const ua = navigator.userAgent || ''
  if (/Windows/i.test(ua)) return 'windows'
  if (/Mac OS X|Macintosh/i.test(ua)) return 'macos'
  if (/Linux/i.test(ua)) return 'linux'
  return 'other'
}

export const nativePlatform: Platform = detectNative()

// 调试覆盖：localStorage 持久化，方便重启后保持
const STORAGE_KEY = 'csubtitle-workstation:debug-platform-override'
const VALID_PLATFORMS: Platform[] = ['windows', 'macos', 'linux', 'other']
const storedRaw = (typeof localStorage !== 'undefined' ? localStorage.getItem(STORAGE_KEY) : null) as Platform | null
const platformOverride = ref<Platform | null>(
  storedRaw && VALID_PLATFORMS.includes(storedRaw) ? storedRaw : null
)

export const platform = computed<Platform>(() => platformOverride.value ?? nativePlatform)
export const isWindows = computed(() => platform.value === 'windows')
export const isMacOS = computed(() => platform.value === 'macos')
export const isLinux = computed(() => platform.value === 'linux')
export const isPlatformOverridden = computed(() => platformOverride.value !== null)
export const currentOverride = computed<Platform | null>(() => platformOverride.value)

// 平台中文标签
const PLATFORM_LABELS: Record<Platform, string> = {
  windows: 'Windows',
  macos: 'macOS',
  linux: 'Linux',
  other: '其它平台'
}

export function platformLabel(value: Platform): string {
  return PLATFORM_LABELS[value]
}

export const nativePlatformLabel = platformLabel(nativePlatform)

export function setPlatformOverride(value: Platform | null) {
  platformOverride.value = value
  if (typeof localStorage === 'undefined') return
  if (value === null) {
    localStorage.removeItem(STORAGE_KEY)
  } else {
    localStorage.setItem(STORAGE_KEY, value)
  }
}
