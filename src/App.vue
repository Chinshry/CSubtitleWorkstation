<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import HomeView from './views/HomeView.vue'
import PresetsView from './views/PresetsView.vue'
import ToolsView from './views/ToolsView.vue'
import SettingsView from './views/SettingsView.vue'
import TitleBar from './components/TitleBar.vue'
import AppToast from './components/AppToast.vue'
import brandLogo from './assets/brand-logo.png'
import { loadConfig } from './api/config'
import {
  globalDragActive,
  pendingDrop,
  pushDiag
} from './stores/dropStore'
import { activeTool, isMediaToolId, type ToolId } from './stores/toolStore'
import { hasAvailableUpdate, refreshAppUpdate } from './stores/updateStore'
import { initFfmpegStatus } from './stores/ffmpegStore'
import { initEncoderOptions } from './composables/useEncoderOptions'
import { useToast } from './composables/useToast'
import { isWindows } from './stores/platformStore'
import { initAvsStatus, initLavFiltersStatus } from './stores/avsStore'

type ViewId = 'home' | 'presets' | 'tools' | 'settings'

const active = ref<ViewId>('home')
const sidebarCollapsed = ref(true)
const unlisteners: UnlistenFn[] = []
const toast = useToast()

function activateView(view: ViewId) {
  active.value = view
}

function classifyPaths(paths: string[]) {
  const out: { videoPath?: string; subtitlePath?: string; textPath?: string } = {}
  for (const p of paths) {
    const lower = p.toLowerCase()
    if (
      /\.(mp4|mkv|mov|ts|m4v|flv|avi|webm|wmv|mpg|mpeg|3gp|3g2|rm|rmvb|vob|mts|m2ts|ogv|ogg|divx|asf|f4v|hevc|h265)$/.test(
        lower
      ) &&
      !out.videoPath
    ) {
      out.videoPath = p
    } else if (/\.(ass|ssa|srt|vtt|sub)$/.test(lower) && !out.subtitlePath) {
      out.subtitlePath = p
      if (!out.textPath) out.textPath = p
    } else if (/\.txt$/.test(lower) && !out.textPath) {
      out.textPath = p
    }
  }
  return out
}

function isLikelyMediaToolPath(path: string) {
  const lower = path.toLowerCase()
  if (/\.(mp4|mkv|mov|ts|m4v|flv|avi|webm|wmv|mpg|mpeg|3gp|3g2|rm|rmvb|vob|mts|m2ts|ogv|ogg|divx|asf|f4v|hevc|h265)$/.test(lower)) {
    return true
  }
  if (/\.(jpe?g|png)$/.test(lower)) {
    return true
  }
  if (/\.(m4a|aac|mp3|wav|flac|ac3|eac3|opus|ogg)$/.test(lower)) {
    return true
  }
  return !/\.[a-z0-9]{1,8}$/i.test(path)
}

function isSubtitlePath(path: string) {
  return /\.(ass|ssa|srt|vtt|sub)$/i.test(path)
}

function isTextPath(path: string) {
  return /\.txt$/i.test(path)
}

function isTsSegmentPath(path: string) {
  return /\.(ts|m2ts|mts)$/i.test(path) || !/\.[a-z0-9]{1,8}$/i.test(path)
}

function isCoverImagePath(path: string) {
  return /\.(jpe?g|png)$/i.test(path)
}

function isAudioPath(path: string) {
  return /\.(m4a|aac|mp3|wav|flac|ac3|eac3|opus|ogg)$/i.test(path)
}

function supportsActiveToolDrop(
  tool: ToolId,
  classified: { videoPath?: string; subtitlePath?: string; textPath?: string },
  paths: string[]
) {
  switch (tool) {
    case 'proofread':
    case 'text-conversion':
      return Boolean(classified.textPath || classified.subtitlePath) && paths.every((path) => isTextPath(path) || isSubtitlePath(path))
    case 'cc-subtitle':
    case 'subtitle-format':
      return Boolean(classified.subtitlePath) && paths.every(isSubtitlePath)
    case 'media-remux':
      return paths.some(isLikelyMediaToolPath) && Boolean(classified.videoPath)
    case 'media-concat-ts':
      return paths.length === 1 || paths.every(isTsSegmentPath)
    case 'media-cover':
      return Boolean(classified.videoPath) || paths.some(isCoverImagePath)
    case 'media-merge-av':
      return Boolean(classified.videoPath) || paths.some(isAudioPath)
    default:
      return false
  }
}

function resolveDropRoute(
  classified: { videoPath?: string; subtitlePath?: string; textPath?: string },
  paths: string[]
): { supported: true; target: 'home' | 'tools'; tool?: ToolId } | { supported: false; message: string } {
  if (active.value === 'tools') {
    if (supportsActiveToolDrop(activeTool.value, classified, paths)) {
      return { supported: true, target: 'tools', tool: activeTool.value }
    }
    return { supported: false, message: '当前工具不支持拖入此类文件' }
  }

  if (active.value === 'home') {
    if (classified.videoPath || classified.subtitlePath) {
      return { supported: true, target: 'home' }
    }
    return { supported: false, message: '压制页只支持拖入视频或字幕文件' }
  }

  if (
    classified.subtitlePath &&
    !classified.videoPath &&
    paths.every(isSubtitlePath)
  ) {
    return { supported: true, target: 'tools', tool: 'subtitle-format' }
  }
  if (classified.textPath && !classified.videoPath && !classified.subtitlePath) {
    return { supported: true, target: 'tools', tool: 'text-conversion' }
  }
  if (classified.videoPath || classified.subtitlePath) {
    return { supported: true, target: 'home' }
  }
  return { supported: false, message: '不支持拖入此类文件' }
}

function runStartupWarmup() {
  const run = async (label: string, task: () => Promise<void>) => {
    try {
      await task()
    } catch (err) {
      pushDiag(`Startup ${label} check skipped: ${String(err)}`)
    }
  }

  void (async () => {
    await run('ffmpeg', () => initFfmpegStatus({ silent: true }))
    await run('encoder', initEncoderOptions)
    if (!isWindows.value) return
    await run('AVS', initAvsStatus)
    await run('LAV Filters', initLavFiltersStatus)
  })()
}

function runStartupUpdateCheck() {
  loadConfig()
    .then((config) => {
      if (config.checkUpdateOnStartup) {
        return refreshAppUpdate({ silent: true })
      }
    })
    .catch((err) => {
      pushDiag(`Startup update check skipped: ${String(err)}`)
    })
}

onMounted(async () => {
  pushDiag('App mounted, registering Tauri drag-drop listeners...')

  runStartupWarmup()
  runStartupUpdateCheck()

  // 直接监听 Tauri 核心拖拽事件，不依赖 webview 封装
  try {
    unlisteners.push(
      await listen<{ paths: string[] }>('tauri://drag-enter', () => {
        globalDragActive.value = true
        pushDiag('drag-enter')
      })
    )
    unlisteners.push(
      await listen('tauri://drag-leave', () => {
        globalDragActive.value = false
        pushDiag('drag-leave')
      })
    )
    unlisteners.push(
      await listen<{ paths: string[] }>('tauri://drag-drop', (event) => {
        globalDragActive.value = false
        const paths = event.payload?.paths ?? []
        pushDiag(`drag-drop received ${paths.length} path(s): ${paths.join(' | ')}`)
        if (!paths.length) {
          pushDiag('WARN: drop event has no paths. 请确认 tauri.conf.json 的 dragDropEnabled=true 已生效（须重启 tauri dev）')
          return
        }
        const classified = classifyPaths(paths)
        const route = resolveDropRoute(classified, paths)
        if (!route.supported) {
          toast.warning(route.message)
          pushDiag(`drop rejected: ${route.message}`)
          return
        }
        if (route.tool) activeTool.value = route.tool
        pendingDrop.value = {
          ...route,
          ...classified,
          raw: paths,
          receivedAt: Date.now()
        }
        // 自动切到对应工具页，避免用户拖入后看不到处理结果。
        if (route.target === 'tools') {
          activateView('tools')
        } else if (route.target === 'home' && active.value !== 'home') {
          activateView('home')
        }
      })
    )
    pushDiag('drag-drop listeners installed.')
  } catch (err) {
    pushDiag(`Failed to register drag-drop listeners: ${String(err)}`)
  }
})

onUnmounted(() => {
  for (const u of unlisteners) u()
})
</script>

<template>
  <div class="app-shell" :class="{ 'drag-active': globalDragActive }" :style="{ '--sidebar-width': sidebarCollapsed ? '80px' : '240px' }">
    <TitleBar />
    <aside class="sidebar" :class="{ collapsed: sidebarCollapsed }">
      <div class="brand">
        <img :src="brandLogo" alt="CC字幕压制工作站" class="brand-logo" />
        <div class="brand-text">
          <strong>CC字幕压制工作站</strong>
          <span class="brand-sub">Subtitle WorkStation</span>
        </div>
      </div>
      <nav>
        <button :class="{ active: active === 'home' }" @click="activateView('home')" v-tooltip="'压制'">
          <span class="nav-icon" aria-hidden="true">
            <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <rect x="3" y="5" width="18" height="14" rx="2" />
              <path d="M7 11h4M7 15h7" />
            </svg>
          </span>
          <span>压制</span>
        </button>
        <button :class="{ active: active === 'presets' }" @click="activateView('presets')" v-tooltip="'预设'">
          <span class="nav-icon" aria-hidden="true">
            <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M4 7h16" />
              <path d="M7 12h10" />
              <path d="M10 17h4" />
              <circle cx="8" cy="7" r="2" />
              <circle cx="16" cy="12" r="2" />
              <circle cx="12" cy="17" r="2" />
            </svg>
          </span>
          <span>预设</span>
        </button>
        <button :class="{ active: active === 'tools' }" @click="activateView('tools')" v-tooltip="'工具'">
          <span class="nav-icon" aria-hidden="true">
            <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <rect x="3" y="8" width="18" height="12" rx="2" />
              <path d="M8 8V6a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
              <path d="M3 13h18" />
              <path d="M12 13v2" />
            </svg>
          </span>
          <span>工具</span>
        </button>
        <button :class="{ active: active === 'settings' }" @click="activateView('settings')" v-tooltip="'设置'">
          <span class="nav-icon" aria-hidden="true">
            <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="3" />
              <path d="M19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1-1.5 1.7 1.7 0 0 0-1.8.3l-.1.1A2 2 0 1 1 4.4 17l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.5-1 1.7 1.7 0 0 0-.3-1.8l-.1-.1A2 2 0 1 1 7 4.4l.1.1a1.7 1.7 0 0 0 1.8.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8V9a1.7 1.7 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1z" />
            </svg>
          </span>
          <span>设置</span>
          <span v-if="hasAvailableUpdate" class="nav-update-dot" aria-hidden="true"></span>
        </button>
      </nav>
      <button class="sidebar-toggle" @click="sidebarCollapsed = !sidebarCollapsed" v-tooltip="sidebarCollapsed ? '展开' : '折叠'">
        <svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="15 18 9 12 15 6"></polyline>
        </svg>
      </button>
    </aside>
    <KeepAlive>
      <HomeView v-if="active === 'home'" />
      <PresetsView v-else-if="active === 'presets'" />
      <ToolsView v-else-if="active === 'tools'" />
      <SettingsView v-else />
    </KeepAlive>
    <AppToast />
  </div>
</template>
