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
import { activeTool, type ToolId } from './stores/toolStore'
import { hasAvailableUpdate, refreshAppUpdate } from './stores/updateStore'

const active = ref<'home' | 'presets' | 'tools' | 'settings'>('home')
const sidebarCollapsed = ref(true)
const unlisteners: UnlistenFn[] = []

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

function resolveDropRoute(
  classified: { videoPath?: string; subtitlePath?: string; textPath?: string },
  paths: string[]
) {
  const route: { target: 'home' | 'tools'; tool?: ToolId } = { target: 'home' }
  if (active.value === 'tools' && activeTool.value === 'media-remux' && paths.some(isLikelyMediaToolPath)) {
    route.target = 'tools'
    route.tool = 'media-remux'
    return route
  }
  if (
    active.value !== 'tools' &&
    classified.subtitlePath &&
    !classified.videoPath &&
    paths.every((path) => /\.(ass|ssa|srt|vtt|sub)$/i.test(path))
  ) {
    route.target = 'tools'
    route.tool = 'subtitle-format'
    return route
  }
  if (active.value === 'tools' && classified.textPath) {
    route.target = 'tools'
    route.tool = activeTool.value
    return route
  }
  if (classified.textPath && !classified.videoPath && !classified.subtitlePath) {
    route.target = 'tools'
    route.tool = activeTool.value
  }
  return route
}

onMounted(async () => {
  pushDiag('App mounted, registering Tauri drag-drop listeners...')

  loadConfig()
    .then((config) => {
      if (config.checkUpdateOnStartup) {
        return refreshAppUpdate({ silent: true })
      }
    })
    .catch((err) => {
      pushDiag(`Startup update check skipped: ${String(err)}`)
    })

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
        if (route.tool) activeTool.value = route.tool
        pendingDrop.value = {
          ...route,
          ...classified,
          raw: paths,
          receivedAt: Date.now()
        }
        // 自动切到对应工具页，避免用户拖入后看不到处理结果。
        if (route.target === 'tools') {
          active.value = 'tools'
        } else if (route.target === 'home' && active.value !== 'home') {
          active.value = 'home'
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
        <button :class="{ active: active === 'home' }" @click="active = 'home'" v-tooltip="'压制'">
          <span class="nav-icon" aria-hidden="true">
            <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <rect x="3" y="5" width="18" height="14" rx="2" />
              <path d="M7 11h4M7 15h7" />
            </svg>
          </span>
          <span>压制</span>
        </button>
        <button :class="{ active: active === 'presets' }" @click="active = 'presets'" v-tooltip="'预设'">
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
        <button :class="{ active: active === 'tools' }" @click="active = 'tools'" v-tooltip="'工具'">
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
        <button :class="{ active: active === 'settings' }" @click="active = 'settings'" v-tooltip="'设置'">
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
