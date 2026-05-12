<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import HomeView from './views/HomeView.vue'
import SettingsView from './views/SettingsView.vue'
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
        <strong>CC字幕压制工作站</strong>
        <span>CSubtitleWorkstation</span>
      </div>
      <nav>
        <button :class="{ active: active === 'home' }" @click="active = 'home'">压制</button>
        <button :class="{ active: active === 'settings' }" @click="active = 'settings'">设置</button>
      </nav>
    </aside>
    <KeepAlive>
      <HomeView v-if="active === 'home'" />
      <SettingsView v-else />
    </KeepAlive>
  </div>
</template>
