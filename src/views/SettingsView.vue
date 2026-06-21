<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { loadConfig, saveConfig } from '../api/config'
import { setFfmpegPath, resetFfmpegToSystem } from '../api/ffmpeg'
import { getCurrentAppVersion } from '../api/updater'
import {
  ffmpegStatus,
  isFfmpegMissingMocked,
  isFfmpegMocked,
  isFfprobeMissingMocked,
  isSubtitleFilterMissingMocked,
  clearAllFfmpegMocks,
  refreshFfmpegStatus,
  setFfmpegMissingMock,
  setFfprobeMissingMock,
  setSubtitleFilterMissingMock,
  setFfmpegStatus,
  ffmpegChecking
} from '../stores/ffmpegStore'
import {
  isWindows,
  isMacOS,
  isLinux,
  isPlatformOverridden,
  nativePlatform,
  nativePlatformLabel,
  platform,
  setPlatformOverride,
  type Platform
} from '../stores/platformStore'
import { avsChecking, avsStatus, lavChecking, refreshAvsStatus, refreshLavFiltersStatus, isAvisynthMissingMocked, isAvsDemuxerMissingMocked, isLavFiltersMissingMocked, setAvisynthMissingMock, setAvsDemuxerMissingMock, setLavFiltersMissingMock, clearAllAvsMocks, isAvsMocked } from '../stores/avsStore'
import {
  refreshAppUpdate,
  updateInfo,
  updateMessage,
  updateReleaseUrl,
  updateState
} from '../stores/updateStore'
import AppSelect from '../components/AppSelect.vue'
import type { AppConfig } from '../types'
import authorAvatarUrl from '../assets/avatar-chinshry.png'
import { currentLanguagePreference, currentLanguageTag, initLocale, isLanguagePreference, setLanguagePreference, t } from '../i18n'

const status = ffmpegStatus
const appVersion = ref('')
const appConfig = ref<AppConfig | null>(null)
const guideOpen = ref(false)
const avsGuideOpen = ref(false)
const lavGuideOpen = ref(false)
const debugPanelOpen = ref(false)
const avsPanelChecking = ref(false)
const lavPanelChecking = ref(false)

const languageOptions = computed(() => [
  { value: 'system', label: t('settings.language.system') },
  { value: 'zh-CN', label: t('settings.language.zhCn') },
  { value: 'en-US', label: t('settings.language.enUs') }
])

// 作者头像：用 vite 打包的本地静态资源，离线 / 网络受限场景始终可用
// 仍保留 onError fallback，对极端情况（资源构建丢失）兜底
const avatarFailed = ref(false)
function onAvatarError() {
  avatarFailed.value = true
}

// 仅在开发构建里显示调试面板
const isDev = import.meta.env.DEV

const sourceText = computed(() => {
  switch (status.value?.source) {
    case 'system_path': return t('settings.ffmpeg.sourceSystemPath')
    case 'custom_path': return t('settings.ffmpeg.sourceCustomPath')
    case 'not_found': return t('settings.ffmpeg.sourceNotFound')
    default: return '—'
  }
})

// 调试覆盖单选项
function displayPlatformLabel(value: Platform) {
  if (value === 'other') return t('settings.debug.otherPlatform')
  return value === 'macos' ? 'macOS' : value.charAt(0).toUpperCase() + value.slice(1)
}

const nativePlatformDisplay = computed(() => displayPlatformLabel(nativePlatform))

const overrideOptions = computed<Array<{ value: Platform | null; label: string }>>(() => [
  { value: null, label: t('settings.debug.followSystemPlatform', { platform: nativePlatformDisplay.value }) },
  { value: 'windows', label: 'Windows' },
  { value: 'macos', label: 'macOS' },
  { value: 'linux', label: 'Linux' }
])

const overrideModel = computed<Platform | 'native'>({
  get: () => (isPlatformOverridden.value ? platform.value : 'native'),
  set: (value) => {
    setPlatformOverride(value === 'native' ? null : value)
  }
})

// ffmpeg 调试 mock：两个独立开关，可同时勾选
const ffmpegMissingModel = computed<boolean>({
  get: () => isFfmpegMissingMocked.value,
  set: (value) => setFfmpegMissingMock(value)
})

const ffprobeMissingModel = computed<boolean>({
  get: () => isFfprobeMissingMocked.value,
  set: (value) => setFfprobeMissingMock(value)
})

const subtitleFilterMissingModel = computed<boolean>({
  get: () => isSubtitleFilterMissingMocked.value,
  set: (value) => setSubtitleFilterMissingMock(value)
})

const mockSummary = computed(() => {
  const parts: string[] = []
  if (isFfmpegMissingMocked.value) parts.push(t('settings.debug.mockFfmpegMissingShort'))
  if (isFfprobeMissingMocked.value) parts.push(t('settings.debug.mockFfprobeMissingShort'))
  if (isSubtitleFilterMissingMocked.value) parts.push(t('settings.debug.mockSubtitleFilterMissingShort'))
  return parts.join(' + ')
})

// AVS 调试 mock
const avisynthMissingModel = computed<boolean>({
  get: () => isAvisynthMissingMocked.value,
  set: (value) => setAvisynthMissingMock(value)
})
const avsDemuxerMissingModel = computed<boolean>({
  get: () => isAvsDemuxerMissingMocked.value,
  set: (value) => setAvsDemuxerMissingMock(value)
})
const lavFiltersMissingModel = computed<boolean>({
  get: () => isLavFiltersMissingMocked.value,
  set: (value) => setLavFiltersMissingMock(value)
})
const avsMockSummary = computed(() => {
  const parts: string[] = []
  if (isAvisynthMissingMocked.value) parts.push(t('settings.debug.mockAvisynthMissingShort'))
  if (isAvsDemuxerMissingMocked.value) parts.push(t('settings.debug.mockAvsDemuxerMissingShort'))
  if (isLavFiltersMissingMocked.value) parts.push(t('settings.debug.mockLavFiltersMissingShort'))
  return parts.join(' + ')
})

const lavReady = computed(() => {
  const status = avsStatus.value
  return Boolean(
    status?.lavFiltersInstalled &&
    status.lavFiltersX64Available &&
    status.lavFiltersDirectshowRegistered
  )
})

async function refresh() {
  await refreshFfmpegStatus()
}

async function chooseFfmpeg() {
  const selected = await open({
    multiple: false,
    directory: false,
    title: t('settings.ffmpeg.dialogChoose')
  })
  if (typeof selected === 'string') {
    setFfmpegStatus(await setFfmpegPath(selected))
  }
}

async function useSystemPath() {
  setFfmpegStatus(await resetFfmpegToSystem())
}

async function checkUpdate() {
  await refreshAppUpdate()
}

async function refreshAvsPanel() {
  avsPanelChecking.value = true
  try {
    return await refreshAvsStatus()
  } finally {
    avsPanelChecking.value = false
  }
}

async function refreshLavPanel() {
  lavPanelChecking.value = true
  try {
    return await refreshLavFiltersStatus()
  } finally {
    lavPanelChecking.value = false
  }
}

const updateNotesTitle = computed(() => t('settings.update.notesTitle'))

const formattedUpdatePubDate = computed(() => formatUpdatePubDate(updateInfo.value?.pubDate))

const updateNotesMeta = computed(() => {
  const parts: string[] = []
  if (formattedUpdatePubDate.value) {
    parts.push(t('settings.update.publishedAt', { date: formattedUpdatePubDate.value }))
  }
  return parts.filter(Boolean).join(' · ')
})

const updateNoteLines = computed(() => parseUpdateNotes(updateInfo.value?.notes))

type UpdateNoteLine = {
  text: string
  kind: 'section' | 'item' | 'paragraph'
}

function parseUpdateNotes(notes?: string): UpdateNoteLine[] {
  return (notes ?? '')
    .replace(/\r\n/g, '\n')
    .split('\n')
    .map((line) => line.trim())
    .filter(Boolean)
    .map(parseUpdateNoteLine)
}

function parseUpdateNoteLine(line: string): UpdateNoteLine {
  const heading = line.match(/^#{1,6}\s+(.+)$/)
  if (heading?.[1]) {
    return { text: heading[1].trim(), kind: 'section' }
  }

  const item = line.match(/^[-*]\s+(.+)$/)
  if (item?.[1]) {
    return { text: item[1].trim(), kind: 'item' }
  }

  return { text: line, kind: 'paragraph' }
}

function formatUpdatePubDate(value?: string) {
  if (!value) return ''
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return ''

  return new Intl.DateTimeFormat(currentLanguageTag.value, {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit'
  }).format(date)
}

async function setStartupUpdateCheck(value: boolean) {
  if (!appConfig.value) return
  const next = {
    ...appConfig.value,
    checkUpdateOnStartup: value
  }
  appConfig.value = next
  await saveConfig(next)
}

async function setLanguage(value: string | number) {
  if (!appConfig.value) return
  const rawLanguage = String(value)
  if (!isLanguagePreference(rawLanguage)) return
  const language = rawLanguage
  const next = {
    ...appConfig.value,
    language
  }
  appConfig.value = next
  setLanguagePreference(language)
  await saveConfig(next)
}

// 异步按钮 loading 状态管理：
// - 延迟 180ms 才显示 spinner：耗时极短的操作（如 mock 检测）完全跳过 spinner，避免一闪
// - 一旦显示，至少停留 400ms：消除"显示瞬间又消失"造成的布局闪烁
const busy = reactive<Record<string, boolean>>({})
const isBusy = (key: string) => !!busy[key]

const SPINNER_SHOW_DELAY = 180
const SPINNER_MIN_DURATION = 400

async function withBusy<T>(key: string, fn: () => Promise<T>): Promise<T | undefined> {
  if (busy[key]) return
  let shownAt: number | null = null
  const timer = window.setTimeout(() => {
    busy[key] = true
    shownAt = Date.now()
  }, SPINNER_SHOW_DELAY)
  try {
    return await fn()
  } finally {
    if (shownAt !== null) {
      const elapsed = Date.now() - shownAt
      if (elapsed < SPINNER_MIN_DURATION) {
        await new Promise<void>((r) => window.setTimeout(r, SPINNER_MIN_DURATION - elapsed))
      }
      busy[key] = false
    } else {
      window.clearTimeout(timer)
    }
  }
}

onMounted(async () => {
  const [version, config] = await Promise.all([
    getCurrentAppVersion(),
    loadConfig()
  ])
  appVersion.value = version
  appConfig.value = config
  initLocale(config.language)
})
</script>

<template>
  <main class="workspace">
    <section class="panel">
      <div class="panel-heading">
        <div>
          <h2>{{ t('settings.language.title') }}</h2>
          <p>{{ t('settings.language.description') }}</p>
        </div>
      </div>
      <dl class="details">
        <div>
          <dt>{{ t('settings.language.label') }}</dt>
          <dd>
            <AppSelect
              v-if="appConfig"
              class="language-select"
              :model-value="currentLanguagePreference"
              :options="languageOptions"
              @update:model-value="setLanguage"
            />
          </dd>
        </div>
      </dl>
    </section>

    <!-- 调试覆盖横幅：覆盖生效时即可见（生产版也保留提示，避免误以为 bug） -->
    <div v-if="isPlatformOverridden" class="debug-banner">
      <strong>⚠ {{ t('settings.debug.platformOverrideTitle') }}</strong>
      <span>
        {{ t('settings.debug.platformOverridePrefix') }}
        <code>{{ displayPlatformLabel(platform) }}</code>
        {{ t('settings.debug.platformOverrideMiddle') }}
        <code>{{ nativePlatformDisplay }}</code>{{ t('ruleDictionary.endPunctuation') }}
      </span>
      <button class="secondary" @click="setPlatformOverride(null)">{{ t('settings.debug.restoreSystem') }}</button>
    </div>

    <div v-if="isFfmpegMocked" class="debug-banner">
      <strong>⚠ {{ t('settings.debug.ffmpegMockTitle') }}</strong>
      <span>
        {{ t('settings.debug.mocking') }}<code>{{ mockSummary }}</code>{{ t('ruleDictionary.endPunctuation') }}
      </span>
      <button class="secondary" @click="clearAllFfmpegMocks">{{ t('settings.debug.restoreRealDetection') }}</button>
    </div>

    <section class="panel panel-check-scope">
      <div class="panel-heading">
        <div>
          <h2>{{ t('settings.ffmpeg.title') }}</h2>
        </div>
      </div>
      <dl class="details">
        <div>
          <dt>{{ t('settings.ffmpeg.status') }}</dt>
          <dd>
            <span class="status-pill" :class="status?.available ? 'ok' : 'bad'" v-tooltip="status?.ffmpegPath ?? t('settings.ffmpeg.ffmpegMissing')">
              <span class="status-icon">{{ status?.available ? '✓' : '✕' }}</span>
              <span>ffmpeg</span>
            </span>
            <span class="status-pill" :class="status?.ffprobePath ? 'ok' : 'bad'" style="margin-left:8px;" v-tooltip="status?.ffprobePath ?? t('settings.ffmpeg.ffprobeMissingTip')">
              <span class="status-icon">{{ status?.ffprobePath ? '✓' : '✕' }}</span>
              <span>ffprobe</span>
            </span>
            <span class="status-pill" :class="status?.subtitleFilterAvailable ? 'ok' : 'bad'" style="margin-left:8px;" v-tooltip="status?.subtitleFilterAvailable ? t('settings.ffmpeg.subtitleFilterAvailable') : t('settings.ffmpeg.subtitleFilterMissing')">
              <span class="status-icon">{{ status?.subtitleFilterAvailable ? '✓' : '✕' }}</span>
              <span>subtitles/libass</span>
            </span>
          </dd>
        </div>
        <div><dt>{{ t('settings.ffmpeg.source') }}</dt><dd>{{ sourceText }}</dd></div>
        <div><dt>ffmpeg</dt><dd>{{ status?.ffmpegPath ?? '—' }}</dd></div>
        <div><dt>ffprobe</dt><dd>{{ status?.ffprobePath ?? t('settings.ffmpeg.ffprobeNotFoundSameDir') }}</dd></div>
        <div><dt>{{ t('settings.ffmpeg.version') }}</dt><dd>{{ status?.ffmpegVersion ?? '—' }}</dd></div>
      </dl>
      <p v-if="status?.message" class="notice" style="color:#a35000;">
        ⚠ {{ status.message }}
      </p>
      <div class="actions left">
        <button
          :class="{ 'is-busy': isBusy('chooseFfmpeg') }"
          @click="withBusy('chooseFfmpeg', chooseFfmpeg)"
        >{{ t('settings.ffmpeg.choose') }}</button>
        <button
          class="secondary"
          :class="{ 'is-busy': isBusy('useSystemPath') }"
          @click="withBusy('useSystemPath', useSystemPath)"
        >{{ t('settings.ffmpeg.useSystemPath') }}</button>
        <button
          class="secondary"
          :class="{ 'is-busy': isBusy('ffmpegRefresh') }"
          @click="withBusy('ffmpegRefresh', refresh)"
        >{{ t('settings.ffmpeg.refresh') }}</button>
        <button class="secondary" @click="guideOpen = !guideOpen">
          {{ guideOpen ? t('settings.ffmpeg.hideGuide') : t('settings.ffmpeg.showGuide') }}
        </button>
      </div>

      <div
        v-if="guideOpen && isWindows"
        class="install-guide"
        v-html="t('settings.ffmpeg.guides.windows')"
      ></div>
      <div
        v-if="guideOpen && isMacOS"
        class="install-guide"
        v-html="t('settings.ffmpeg.guides.macos')"
      ></div>
      <div
        v-if="guideOpen && isLinux"
        class="install-guide"
        v-html="t('settings.ffmpeg.guides.linux')"
      ></div>
      <div v-if="ffmpegChecking" class="panel-check-overlay" role="status" aria-live="polite">
        <div class="panel-check-dialog">
          <span class="panel-check-spinner" aria-hidden="true"></span>
          <div>
            <strong>{{ t('settings.ffmpeg.checkingTitle') }}</strong>
            <span>{{ t('settings.ffmpeg.checkingDescription') }}</span>
          </div>
        </div>
      </div>
    </section>

    <!-- AVS 设置：仅 Windows 展示，仿 ffmpeg 设置面板风格 -->
    <section v-if="isWindows" class="panel panel-check-scope">
      <div v-if="isAvsMocked" class="debug-banner">
        <strong>⚠ {{ t('settings.debug.avsMockTitle') }}</strong>
        <span>{{ t('settings.debug.mocking') }}<code>{{ avsMockSummary }}</code>{{ t('ruleDictionary.endPunctuation') }}</span>
        <button class="secondary" @click="clearAllAvsMocks">{{ t('settings.debug.restoreRealDetection') }}</button>
      </div>

      <div class="panel-heading">
        <div>
          <h2>{{ t('settings.avs.title') }}</h2>
          <p>{{ t('settings.avs.description') }}</p>
        </div>
      </div>
      <dl class="details">
        <div>
          <dt>{{ t('settings.avs.status') }}</dt>
          <dd>
            <span
              class="status-pill"
              :class="avsStatus?.ffmpegDemuxerAvailable ? 'ok' : 'bad'"
              v-tooltip="t('settings.avs.demuxerTip')"
            >
              <span class="status-icon">{{ avsStatus?.ffmpegDemuxerAvailable ? '✓' : '✕' }}</span>
              <span>ffmpeg avisynth demuxer</span>
            </span>
            <span
              class="status-pill"
              :class="avsStatus?.avisynthInstalled ? 'ok' : 'bad'"
              style="margin-left:8px;"
              v-tooltip="t('settings.avs.avisynthTip')"
            >
              <span class="status-icon">{{ avsStatus?.avisynthInstalled ? '✓' : '✕' }}</span>
              <span>AviSynth+</span>
            </span>
          </dd>
        </div>
        <div><dt>{{ t('settings.avs.avisynthVersion') }}</dt><dd>{{ avsStatus?.avisynthVersion ?? '—' }}</dd></div>
        <div><dt>{{ t('settings.avs.installPath') }}</dt><dd>{{ avsStatus?.avisynthInstallPath ?? '—' }}</dd></div>
        <div><dt>AviSynth.dll</dt><dd>{{ avsStatus?.avisynthDllPath ?? t('settings.avs.dllNotFound') }}</dd></div>
      </dl>
      <p v-if="avsStatus && !avsStatus.available" class="notice" style="color:#a35000;">
        ⚠ {{ avsStatus.message ?? t('settings.avs.unavailable') }}
      </p>
      <div class="actions left">
        <button
          class="secondary"
          :class="{ 'is-busy': isBusy('avsRefresh') }"
          @click="withBusy('avsRefresh', refreshAvsPanel)"
        >{{ t('settings.ffmpeg.refresh') }}</button>
        <button class="secondary" @click="avsGuideOpen = !avsGuideOpen">
          {{ avsGuideOpen ? t('settings.ffmpeg.hideGuide') : t('settings.avs.showGuide') }}
        </button>
      </div>

      <div
        v-if="avsGuideOpen"
        class="install-guide"
        v-html="t('settings.avs.guide')"
      ></div>
      <div v-if="avsPanelChecking || avsChecking" class="panel-check-overlay" role="status" aria-live="polite">
        <div class="panel-check-dialog">
          <span class="panel-check-spinner" aria-hidden="true"></span>
          <div>
            <strong>{{ t('settings.avs.checkingTitle') }}</strong>
            <span>{{ t('settings.avs.checkingDescription') }}</span>
          </div>
        </div>
      </div>
    </section>

    <section v-if="isWindows" class="panel panel-check-scope">
      <div class="panel-heading">
        <div>
          <h2>{{ t('settings.lav.title') }}</h2>
          <p>{{ t('settings.lav.description') }}</p>
        </div>
      </div>
      <dl class="details">
        <div>
          <dt>{{ t('settings.ffmpeg.status') }}</dt>
          <dd>
            <span class="status-pill" :class="lavReady ? 'ok' : 'bad'">
              <span class="status-icon">{{ lavReady ? '✓' : '✕' }}</span>
              <span>{{ lavReady ? t('settings.lav.ready') : t('settings.lav.required') }}</span>
            </span>
          </dd>
        </div>
        <div><dt>{{ t('settings.ffmpeg.version') }}</dt><dd>{{ avsStatus?.lavFiltersVersion ?? '—' }}</dd></div>
        <div><dt>{{ t('settings.avs.installPath') }}</dt><dd>{{ avsStatus?.lavFiltersInstallPath ?? '—' }}</dd></div>
        <div><dt>{{ t('settings.lav.x64Components') }}</dt><dd>{{ avsStatus?.lavFiltersX64Available ? t('settings.lav.detected') : t('settings.lav.notDetected') }}</dd></div>
        <div><dt>{{ t('settings.lav.directshow') }}</dt><dd>{{ avsStatus?.lavFiltersDirectshowRegistered ? t('settings.lav.directshowRegistered') : t('settings.lav.directshowMissing') }}</dd></div>
      </dl>
      <div class="actions left">
        <button
          class="secondary"
          :class="{ 'is-busy': isBusy('lavRefresh') }"
          @click="withBusy('lavRefresh', refreshLavPanel)"
        >{{ t('settings.ffmpeg.refresh') }}</button>
        <button class="secondary" @click="lavGuideOpen = !lavGuideOpen">
          {{ lavGuideOpen ? t('settings.ffmpeg.hideGuide') : t('settings.lav.showGuide') }}
        </button>
      </div>
      <div
        v-if="lavGuideOpen"
        class="install-guide"
        v-html="t('settings.lav.guide')"
      ></div>
      <div v-if="lavPanelChecking || lavChecking" class="panel-check-overlay" role="status" aria-live="polite">
        <div class="panel-check-dialog">
          <span class="panel-check-spinner" aria-hidden="true"></span>
          <div>
            <strong>{{ t('settings.lav.checkingTitle') }}</strong>
            <span>{{ t('settings.lav.checkingDescription') }}</span>
          </div>
        </div>
      </div>
    </section>

    <section class="panel">
      <div class="panel-heading">
        <div>
          <div class="update-title-row">
            <h2>{{ t('settings.update.title') }}</h2>
            <span class="current-version">{{ t('settings.update.currentVersion', { version: appVersion }) }}</span>
          </div>
        </div>
      </div>
      <div class="actions left">
        <button
          :class="{ 'is-busy': isBusy('checkUpdate') }"
          @click="withBusy('checkUpdate', checkUpdate)"
        >{{ t('settings.update.check') }}</button>
        <label v-if="appConfig" class="switch-row update-startup-toggle">
          <input
            type="checkbox"
            :checked="appConfig.checkUpdateOnStartup"
            @change="setStartupUpdateCheck(($event.target as HTMLInputElement).checked)"
          />
          <span class="switch" aria-hidden="true"></span>
          <span>{{ t('settings.update.checkOnStartup') }}</span>
        </label>
      </div>
      <div v-if="updateMessage" class="update-result" :class="`update-result-${updateState}`">
        <div class="update-result-content">
          <div class="update-result-header">
            <div class="update-result-heading">
              <div class="update-result-title">
                <span class="update-result-icon">
                  {{ updateState === 'error' ? '!' : updateState === 'success' ? '✓' : 'i' }}
                </span>
                <span class="update-result-message">{{ updateMessage }}</span>
              </div>
            </div>
              <a
                v-if="updateState === 'success' && updateInfo?.available"
                class="button-link update-download-link"
                :href="updateReleaseUrl"
                target="_blank"
                rel="noopener noreferrer"
            >{{ t('settings.update.download') }}</a>
          </div>
          <div v-if="updateInfo?.notes" class="update-notes">
            <div class="update-notes-head">
              <span class="update-notes-title">{{ updateNotesTitle }}</span>
              <span v-if="updateNotesMeta" class="update-notes-meta">
                {{ updateNotesMeta }}
              </span>
            </div>
            <div class="update-notes-body">
              <span
                v-for="(line, index) in updateNoteLines"
                :key="`${index}-${line.text}`"
                class="update-note-line"
                :class="`update-note-line-${line.kind}`"
              >{{ line.text }}</span>
            </div>
          </div>
        </div>
      </div>
    </section>

    <section class="panel">
      <div class="panel-heading">
        <div>
          <h2>{{ t('settings.about.title') }}</h2>
        </div>
      </div>
      <dl class="details">
        <div>
          <dt>{{ t('settings.about.author') }}</dt>
          <dd>
            <a
              class="author-link"
              href="https://github.com/Chinshry"
              target="_blank"
              rel="noopener noreferrer"
              v-tooltip="t('settings.about.authorTip')"
            >
              <img
                v-if="!avatarFailed"
                :src="authorAvatarUrl"
                class="author-avatar"
                :alt="t('settings.about.authorAvatarAlt')"
                width="24"
                height="24"
                @error="onAvatarError"
              />
              <span v-else class="author-avatar author-avatar-fallback" aria-hidden="true">
                <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
                  <circle cx="12" cy="7" r="4" />
                </svg>
              </span>
              <span class="author-name">Chinshry</span>
            </a>
          </dd>
        </div>
        <div>
          <dt>{{ t('settings.about.repository') }}</dt>
          <dd>
            <a
              class="repo-link"
              href="https://github.com/Chinshry/CSubtitleWorkstation"
              target="_blank"
              rel="noopener noreferrer"
              v-tooltip="t('settings.about.repositoryTip')"
            >
              <svg class="repo-link-icon" viewBox="0 0 16 16" width="14" height="14" aria-hidden="true">
                <path fill="currentColor" fill-rule="evenodd" d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"/>
              </svg>
              {{ t('settings.about.githubRepository') }}
            </a>
          </dd>
        </div>
        <div>
          <dt>{{ t('settings.about.license') }}</dt>
          <dd>
            <a
              class="license-chip"
              href="https://github.com/Chinshry/CSubtitleWorkstation/blob/master/LICENSE"
              target="_blank"
              rel="noopener noreferrer"
              v-tooltip="t('settings.about.licenseTip')"
            >GNU GPL v3.0</a>
          </dd>
        </div>
      </dl>
    </section>

    <!-- 调试面板：仅开发构建可见 -->
    <section v-if="isDev" class="panel debug-panel">
      <div class="panel-heading">
        <div>
          <h2>{{ t('settings.debug.title') }}</h2>
          <p>
            {{ t('settings.debug.description') }}
            {{ t('settings.debug.nativePlatform') }}<code>{{ nativePlatformDisplay }}</code>{{ t('ruleDictionary.endPunctuation') }}
          </p>
        </div>
        <button class="secondary" @click="debugPanelOpen = !debugPanelOpen">
          {{ debugPanelOpen ? t('nav.collapse') : t('nav.expand') }}
        </button>
      </div>
      <div v-if="debugPanelOpen" class="debug-body">
        <div class="debug-group">
          <h4>{{ t('settings.debug.platformOverride') }}</h4>
          <div class="debug-options">
            <label v-for="opt in overrideOptions" :key="String(opt.value)" class="debug-radio">
              <input
                type="radio"
                name="platform-override"
                :value="opt.value === null ? 'native' : opt.value"
                v-model="overrideModel"
              />
              <span>{{ opt.label }}</span>
            </label>
          </div>
        </div>
        <div class="debug-group">
          <h4>{{ t('settings.debug.ffmpegMocks') }}</h4>
          <p class="muted">{{ t('settings.debug.ffmpegMocksDescription') }}</p>
          <div class="debug-options">
            <label class="debug-check">
              <input type="checkbox" v-model="ffmpegMissingModel" />
              <span>
                {{ t('settings.debug.mockFfmpegMissing') }}
                <em class="muted">{{ t('settings.debug.mockFfmpegMissingHint') }}</em>
              </span>
            </label>
            <label class="debug-check">
              <input type="checkbox" v-model="ffprobeMissingModel" />
              <span>
                {{ t('settings.debug.mockFfprobeMissing') }}
                <em class="muted">{{ t('settings.debug.mockFfprobeMissingHint') }}</em>
              </span>
            </label>
            <label class="debug-check">
              <input type="checkbox" v-model="subtitleFilterMissingModel" />
              <span>
                {{ t('settings.debug.mockSubtitleFilterMissing') }}
                <em class="muted">{{ t('settings.debug.mockSubtitleFilterMissingHint') }}</em>
              </span>
            </label>
          </div>
        </div>
        <div class="debug-group">
          <h4>{{ t('settings.debug.avsMocks') }}</h4>
          <p class="muted">{{ t('settings.debug.avsMocksDescription') }}</p>
          <div class="debug-options">
            <label class="debug-check">
              <input type="checkbox" v-model="avisynthMissingModel" />
              <span>
                {{ t('settings.debug.mockAvisynthMissing') }}
                <em class="muted">{{ t('settings.debug.mockAvisynthMissingHint') }}</em>
              </span>
            </label>
            <label class="debug-check">
              <input type="checkbox" v-model="avsDemuxerMissingModel" />
              <span>
                {{ t('settings.debug.mockAvsDemuxerMissing') }}
                <em class="muted">{{ t('settings.debug.mockAvsDemuxerMissingHint') }}</em>
              </span>
            </label>
          </div>
        </div>
        <div class="debug-group">
          <h4>{{ t('settings.debug.lavMocks') }}</h4>
          <p class="muted">{{ t('settings.debug.lavMocksDescription') }}</p>
          <div class="debug-options">
            <label class="debug-check">
              <input type="checkbox" v-model="lavFiltersMissingModel" />
              <span>
                {{ t('settings.debug.mockLavFiltersMissing') }}
                <em class="muted">{{ t('settings.debug.mockLavFiltersMissingHint') }}</em>
              </span>
            </label>
          </div>
        </div>
      </div>
    </section>
  </main>
</template>

<style scoped>
.debug-banner {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  align-items: center;
  padding: 8px 14px;
  margin-bottom: 12px;
  background: #fff8d6;
  border: 1px solid #e0c870;
  border-radius: 6px;
  color: #6a5300;
  font-size: 13px;
}
.debug-banner code {
  background: rgba(0, 0, 0, 0.06);
  padding: 1px 6px;
  border-radius: 3px;
}
.debug-banner button {
  margin-left: auto;
}

.language-select {
  max-width: 260px;
}

.debug-panel {
  border: 1px dashed #b89aff;
  background: #f7f2ff;
}
.debug-body {
  display: flex;
  flex-direction: column;
  gap: 14px;
  margin-top: 8px;
}
.debug-group h4 {
  margin: 0 0 6px 0;
  font-size: 13px;
  color: #5a3da6;
}
.debug-group .muted {
  margin: 0 0 6px 0;
  font-size: 12px;
  color: #7a7a7a;
}
.debug-options {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.debug-radio {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  font-size: 14px;
}
.debug-check {
  display: inline-flex;
  align-items: flex-start;
  gap: 8px;
  cursor: pointer;
  font-size: 14px;
}
.debug-check input[type="checkbox"] {
  margin-top: 3px;
}
.debug-radio em,
.debug-check em {
  font-style: normal;
  font-size: 12px;
  color: #7a7a7a;
}
</style>
