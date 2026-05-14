<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'

// 沉浸式自定义标题栏：
// - 顶部 36px 隐形拖拽条（横跨整个窗口顶部）
// - 右上角悬浮 Windows 风格的 — □ × 三按钮
// - 关闭按钮 hover 红底白字，符合 Windows 系统直觉
// - 最大化状态下切换为"还原"双框图标
const appWindow = getCurrentWindow()
const isMaximized = ref(false)
let unlistenResize: (() => void) | null = null

async function refreshMaximized() {
  try {
    isMaximized.value = await appWindow.isMaximized()
  } catch {
    /* ignore */
  }
}

async function onMinimize() {
  await appWindow.minimize()
}

async function onToggleMaximize() {
  await appWindow.toggleMaximize()
  // toggleMaximize 内部异步，状态略滞后；onResized 也会回调，这里再 refresh 一次确保即时
  await refreshMaximized()
}

async function onClose() {
  await appWindow.close()
}

onMounted(async () => {
  await refreshMaximized()
  unlistenResize = await appWindow.onResized(() => {
    refreshMaximized()
  })
})

onUnmounted(() => {
  if (unlistenResize) unlistenResize()
})
</script>

<template>
  <div class="title-bar">
    <!-- 横跨整个窗口顶部的隐形拖拽区 -->
    <div class="title-bar-drag" data-tauri-drag-region></div>

    <!-- 悬浮在右上角的窗口控件 -->
    <div class="title-bar-controls">
      <button
        type="button"
        class="title-btn"
        :title="'最小化'"
        aria-label="最小化"
        @click="onMinimize"
      >
        <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
          <line x1="1" y1="5" x2="9" y2="5" stroke="currentColor" stroke-width="1" />
        </svg>
      </button>

      <button
        type="button"
        class="title-btn"
        :title="isMaximized ? '向下还原' : '最大化'"
        :aria-label="isMaximized ? '向下还原' : '最大化'"
        @click="onToggleMaximize"
      >
        <!-- 最大化：单方框 -->
        <svg
          v-if="!isMaximized"
          width="10"
          height="10"
          viewBox="0 0 10 10"
          aria-hidden="true"
        >
          <rect
            x="1"
            y="1"
            width="8"
            height="8"
            stroke="currentColor"
            stroke-width="1"
            fill="none"
          />
        </svg>
        <!-- 已最大化：双方框（向下还原）。两矩形纯描边互相错位，hover 时颜色随 currentColor 一致 -->
        <svg
          v-else
          width="10"
          height="10"
          viewBox="0 0 10 10"
          aria-hidden="true"
        >
          <rect
            x="3"
            y="1"
            width="6"
            height="6"
            stroke="currentColor"
            stroke-width="1"
            fill="none"
          />
          <rect
            x="1"
            y="3"
            width="6"
            height="6"
            stroke="currentColor"
            stroke-width="1"
            fill="none"
          />
        </svg>
      </button>

      <button
        type="button"
        class="title-btn close-btn"
        :title="'关闭'"
        aria-label="关闭"
        @click="onClose"
      >
        <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
          <line x1="1" y1="1" x2="9" y2="9" stroke="currentColor" stroke-width="1" />
          <line x1="9" y1="1" x2="1" y2="9" stroke="currentColor" stroke-width="1" />
        </svg>
      </button>
    </div>
  </div>
</template>

<style scoped>
.title-bar {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  height: 36px;
  z-index: 1000;
  /* 容器本身不接收事件，避免遮挡下层 UI；事件由子元素的拖拽区/按钮分别接管 */
  pointer-events: none;
  user-select: none;
}

.title-bar-drag {
  position: absolute;
  top: 0;
  left: 0;
  /* 物理避开右侧 138px (3 × 46px) 控件区，杜绝事件位置冲突 */
  right: 138px;
  height: 36px;
  pointer-events: auto;
  background: transparent;
}

.title-bar-controls {
  position: absolute;
  top: 0;
  right: 0;
  display: flex;
  height: 36px;
  width: 138px;
  pointer-events: auto;
  z-index: 1;
}

.title-btn {
  align-items: center;
  background: transparent;
  border: 0;
  border-radius: 0;
  color: #43515c;
  cursor: pointer;
  display: inline-flex;
  height: 36px;
  justify-content: center;
  min-height: 0;
  padding: 0;
  transition: background 0.12s ease, color 0.12s ease;
  width: 46px;
  /* 覆盖全局 button 样式（来自 styles.css 的 background:#176b87） */
}

.title-btn:hover {
  background: rgba(0, 0, 0, 0.06);
  color: #18202a;
}

.title-btn:active {
  background: rgba(0, 0, 0, 0.1);
}

.title-btn.close-btn:hover {
  background: #e81123;
  color: #ffffff;
}

.title-btn.close-btn:active {
  background: #c50f1f;
  color: #ffffff;
}

/* 当页面中存在全屏 modal（标志 class：.app-modal-active）时，
   按钮区背后是半透明深色遮罩 rgba(15,23,32,0.55) + blur，
   原本 #43515c 深色图标在该背景下几乎与遮罩同色 → 对比度坍塌。
   用 :has() 自动探测 modal 存在，切到浅色图标 + 半透明白底 hover：
   - 关闭按钮的红色 hover 本身高对比，无需特殊处理（继承通用规则被红色覆盖即可）。
   - 选择 :has() 而非 JS 同步 body class：单一组件内闭环，无跨组件状态。 */
body:has(.app-modal-active) .title-btn {
  color: #d6dde5;
}
body:has(.app-modal-active) .title-btn:hover {
  background: rgba(255, 255, 255, 0.18);
  color: #ffffff;
}
body:has(.app-modal-active) .title-btn:active {
  background: rgba(255, 255, 255, 0.28);
}
body:has(.app-modal-active) .title-btn.close-btn:hover {
  background: #e81123;
  color: #ffffff;
}
body:has(.app-modal-active) .title-btn.close-btn:active {
  background: #c50f1f;
  color: #ffffff;
}

/* 当标题栏覆盖在 sidebar 深色区域时，按钮在浅色背景中已对比明显；
   sidebar 顶部 24px 留白本身就在 36px 拖拽区之内，不会与品牌区冲突 */
</style>
