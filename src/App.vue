<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import HomeView from './views/HomeView.vue'
import SettingsView from './views/SettingsView.vue'
import brandLogo from './assets/brand-logo.png'
import {
  globalDragActive,
  pendingDrop,
  pushDiag
} from './stores/dropStore'

const active = ref<'home' | 'settings'>('home')
const unlisteners: UnlistenFn[] = []

function classifyPaths(paths: string[]) {
  const out: { videoPath?: string; subtitlePath?: string } = {}
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
    }
  }
  return out
}

onMounted(async () => {
  pushDiag('App mounted, registering Tauri drag-drop listeners...')

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
        pendingDrop.value = {
          ...classified,
          raw: paths,
          receivedAt: Date.now()
        }
        // 自动切回压制页，避免用户在设置页拖入后看不到效果
        if (active.value !== 'home') active.value = 'home'
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
  <div class="app-shell" :class="{ 'drag-active': globalDragActive }">
    <aside class="sidebar">
      <div class="brand">
        <img :src="brandLogo" alt="CC字幕压制工作站" class="brand-logo" />
        <div class="brand-text">
          <strong>CC字幕压制工作站</strong>
          <span class="brand-sub">Subtitle WorkStation</span>
        </div>
      </div>
      <nav>
        <button :class="{ active: active === 'home' }" @click="active = 'home'">
          <span class="nav-icon" aria-hidden="true">
            <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <rect x="3" y="5" width="18" height="14" rx="2" />
              <path d="M7 11h4M7 15h7" />
            </svg>
          </span>
          <span>压制</span>
        </button>
        <button :class="{ active: active === 'settings' }" @click="active = 'settings'">
          <span class="nav-icon" aria-hidden="true">
            <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="3" />
              <path d="M19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1-1.5 1.7 1.7 0 0 0-1.8.3l-.1.1A2 2 0 1 1 4.4 17l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.5-1 1.7 1.7 0 0 0-.3-1.8l-.1-.1A2 2 0 1 1 7 4.4l.1.1a1.7 1.7 0 0 0 1.8.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8V9a1.7 1.7 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1z" />
            </svg>
          </span>
          <span>设置</span>
        </button>
      </nav>
    </aside>
    <KeepAlive>
      <HomeView v-if="active === 'home'" />
      <SettingsView v-else />
    </KeepAlive>
  </div>
</template>
