<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { convertFileSrc } from '@tauri-apps/api/core'
import { open as openDialog } from '@tauri-apps/plugin-dialog'
import type { LogoLayout, LogoLayoutEntry, RecentLogo } from '../types'
import { extractVideoFrame } from '../api/video'
import { globalDragActive, pendingDrop } from '../stores/dropStore'

const props = defineProps<{
  videoPath: string
  videoWidth?: number
  videoHeight?: number
  videoDuration?: number
  initialLayout?: LogoLayout | null
  recentLogos: RecentLogo[]
  logoLayouts: LogoLayoutEntry[]
}>()

const emit = defineEmits<{
  (e: 'save', layout: LogoLayout, recentLogos: RecentLogo[], logoLayouts: LogoLayoutEntry[]): void
  (e: 'cancel'): void
  /** 即时更新最近 LOGO 列表（删除场景） */
  (e: 'update-recent', recentLogos: RecentLogo[]): void
}>()

// 图片扩展名白名单：与 chooseLogo 对话框 filter 保持一致
const IMAGE_EXT_RE = /\.(png|jpe?g|webp|bmp)$/i
function isImagePath(p: string): boolean {
  return IMAGE_EXT_RE.test(p)
}

// === 视频帧预览状态 ===
const framePath = ref<string>('')
const frameLoading = ref(false)
const frameError = ref('')
const currentTime = ref(0)
const stageEl = ref<HTMLDivElement | null>(null)
const stageWrapEl = ref<HTMLDivElement | null>(null)
// stage 实际 px 尺寸：由 JS 根据 wrap 可用空间 + 视频宽高比算出
const stageBoxStyle = ref<Record<string, string>>({})
let stageResizeObserver: ResizeObserver | null = null
// 抽帧节流：拖动时间轴时只在停顿 250ms 后真正抽帧
let frameDebounce: ReturnType<typeof setTimeout> | null = null
let frameSeq = 0
const frameUrl = computed(() =>
  framePath.value ? `${convertFileSrc(framePath.value)}?t=${frameSeq}` : ''
)

const videoAspect = computed(() => {
  const w = props.videoWidth ?? 0
  const h = props.videoHeight ?? 0
  if (w > 0 && h > 0) return w / h
  return 16 / 9
})

const duration = computed(() => Math.max(0, props.videoDuration ?? 0))

// === 分辨率桶解析 ===
// 仅记忆 6 种常见分辨率：720p / 1080p / 4K × 横屏 / 竖屏。
// 非常见分辨率视频（如 720×400）保存时不写入 logoLayouts，但仍可临时编辑。
// 长边 / 短边 ± 8 像素容差用于兼容 1920×1088 这类 mod16 对齐尺寸。
function resolveBucket(w?: number, h?: number): string | null {
  if (!w || !h || w <= 0 || h <= 0) return null
  const tol = 8
  const long = Math.max(w, h)
  const short = Math.min(w, h)
  const isLandscape = w >= h
  const approx = (a: number, b: number) => Math.abs(a - b) <= tol
  if (approx(long, 1280) && approx(short, 720)) {
    return isLandscape ? '720p-landscape' : '720p-portrait'
  }
  if (approx(long, 1920) && approx(short, 1080)) {
    return isLandscape ? '1080p-landscape' : '1080p-portrait'
  }
  if (approx(long, 3840) && approx(short, 2160)) {
    return isLandscape ? '4k-landscape' : '4k-portrait'
  }
  return null
}

const currentBucket = computed(() => resolveBucket(props.videoWidth, props.videoHeight))

// 在 logoLayouts 中找 (bucket, path) 命中条目
function findLayoutEntry(bucket: string | null, path: string): LogoLayoutEntry | null {
  if (!bucket || !path) return null
  return props.logoLayouts.find((e) => e.bucket === bucket && e.path === path) ?? null
}

// 根据 stage 父容器实际可用空间 + 视频宽高比，反推 stage 的最大可显示像素尺寸。
// 这样竖屏视频在横向预览区里也不会撑爆高度，避免遮住底部时间轴/按钮。
function recalcStageBox() {
  const wrap = stageWrapEl.value
  if (!wrap) return
  const rect = wrap.getBoundingClientRect()
  // .le-stage-wrap 自身 padding 6*2 = 12
  const availW = Math.max(0, rect.width - 12)
  const availH = Math.max(0, rect.height - 12)
  if (availW <= 0 || availH <= 0) return
  const ratio = videoAspect.value
  let w = availW
  let h = w / ratio
  if (h > availH) {
    h = availH
    w = h * ratio
  }
  stageBoxStyle.value = {
    width: `${Math.floor(w)}px`,
    height: `${Math.floor(h)}px`
  }
}

watch(videoAspect, () => {
  // 视频比例切换时（不同视频）重新计算
  void nextTick(recalcStageBox)
})

// === LOGO 状态 ===
const logoPath = ref<string>('')
// 自然宽高比（LOGO 图片本身），用于缩放时锁定形变
const logoAspect = ref<number>(1)
const logoUrl = computed(() => (logoPath.value ? convertFileSrc(logoPath.value) : ''))
// LOGO 在视频画面内的位置和尺寸（按视频百分比存储）
const xPct = ref<number>(0.02)
const yPct = ref<number>(0.02)
const wPct = ref<number>(0.2)
const hPct = ref<number>(0.1)

// 标识自加载后是否有改动；用于关闭确认
const dirty = ref(false)

// 视频画面像素坐标（基于真实分辨率），用于状态条显示
const pxX = computed(() => Math.round(xPct.value * (props.videoWidth ?? 0)))
const pxY = computed(() => Math.round(yPct.value * (props.videoHeight ?? 0)))
const pxW = computed(() => Math.round(wPct.value * (props.videoWidth ?? 0)))
const pxH = computed(() => Math.round(hPct.value * (props.videoHeight ?? 0)))

const hasValidLogo = computed(() => !!logoPath.value)

// 当前 LOGO 的主显示名：优先 recentLogos 里的别名，回退到 path 文件名。
// 这样列表里命名后，「当前 LOGO」卡片也能立刻显示同名。
const currentDisplayName = computed(() => {
  const path = logoPath.value
  if (!path) return ''
  const hit = props.recentLogos.find((r) => r.path === path)
  const alias = (hit?.displayName ?? '').trim()
  return alias || pathBasename(path)
})

// === 初始化 ===
// 优先按 (currentBucket, candidate.path) 命中 logoLayouts entry；
// 找不到再使用当前任务传入的 initialLayout 或最近列表第一项。
function applyInitialLayout() {
  const bucket = currentBucket.value
  const init = props.initialLayout

  // 1) 先尝试 init.path 在当前桶的记忆
  if (init && init.path) {
    const hit = findLayoutEntry(bucket, init.path)
    if (hit) {
      applyEntry(hit)
      return
    }
    // 没命中桶 → 使用当前任务传入的 initialLayout。
    // 注意：init 的 hPct 是按旧视频宽高比算的，跨桶套用会拉伸 LOGO（例如横屏 1080 → 竖屏 720）。
    // 临时填入，等 aspect 加载完后按当前视频宽高比反算正确的 hPct。
    logoPath.value = init.path
    xPct.value = init.xPct
    yPct.value = init.yPct
    wPct.value = init.wPct
    hPct.value = init.hPct
    void preloadLogoAspect(init.path).then(fixHeightByAspect)
    return
  }

  // 2) 没存过布局：尝试用最近列表第一项 + 当前桶
  if (props.recentLogos.length > 0) {
    const recentPath = props.recentLogos[0].path
    const recentHit = findLayoutEntry(bucket, recentPath)
    if (recentHit) {
      applyEntry(recentHit)
      return
    }
    logoPath.value = recentPath
    void preloadLogoAspect(recentPath).then(fixHeightByAspect)
  }
}

function applyEntry(entry: LogoLayoutEntry) {
  logoPath.value = entry.path
  xPct.value = entry.xPct
  yPct.value = entry.yPct
  wPct.value = entry.wPct
  hPct.value = entry.hPct
  void preloadLogoAspect(entry.path).then(fixHeightByAspect)
}

// 按 logoAspect + 当前视频宽高比反算 hPct，保留 wPct/xPct/yPct 不动。
// 用于跨分辨率桶套用旧布局时防止 LOGO 拉伸（例如横屏 1080 切到竖屏 720）。
function fixHeightByAspect() {
  hPct.value = heightPctForWidthPct(wPct.value)
}

function heightPctForWidthPct(widthPct: number) {
  if (logoAspect.value > 0 && props.videoWidth && props.videoHeight) {
    const targetW = widthPct * props.videoWidth
    const targetH = targetW / logoAspect.value
    return clamp01(targetH / props.videoHeight)
  }
  return hPct.value
}

function widthPctForHeightPct(heightPct: number) {
  if (logoAspect.value > 0 && props.videoWidth && props.videoHeight) {
    const targetH = heightPct * props.videoHeight
    const targetW = targetH * logoAspect.value
    return clamp01(targetW / props.videoWidth)
  }
  return wPct.value
}

async function preloadLogoAspect(path: string) {
  const url = convertFileSrc(path)
  try {
    await new Promise<void>((resolve, reject) => {
      const img = new Image()
      img.onload = () => {
        if (img.naturalWidth > 0 && img.naturalHeight > 0) {
          logoAspect.value = img.naturalWidth / img.naturalHeight
        }
        resolve()
      }
      img.onerror = () => reject(new Error('LOGO 图片加载失败'))
      img.src = url
    })
  } catch {
    // 加载失败先静默，等用户重新选图
  }
}

// === 抽帧逻辑（debounce） ===
async function refreshFrame() {
  if (!props.videoPath) return
  const time = currentTime.value
  frameLoading.value = true
  frameError.value = ''
  try {
    const path = await extractVideoFrame(props.videoPath, time)
    framePath.value = path
    frameSeq += 1
  } catch (err) {
    frameError.value = formatError(err)
  } finally {
    frameLoading.value = false
  }
}

function scheduleFrameRefresh() {
  if (frameDebounce) clearTimeout(frameDebounce)
  frameDebounce = setTimeout(() => {
    void refreshFrame()
  }, 250)
}

watch(currentTime, () => {
  scheduleFrameRefresh()
})

function formatError(error: unknown): string {
  if (typeof error === 'string') return error
  const msg = (error as { message?: unknown })?.message
  if (typeof msg === 'string') return msg
  try {
    return JSON.stringify(error)
  } catch {
    return String(error)
  }
}

function formatTime(seconds: number, withHours = false): string {
  const s = Math.max(0, seconds)
  const h = Math.floor(s / 3600)
  const m = Math.floor((s - h * 3600) / 60)
  const r = s - h * 3600 - m * 60
  const mm = m.toString().padStart(2, '0')
  // 毫秒展示 3 位（如 00:15.234），padStart 长度 = 整数2 + 点1 + 小数3 = 6
  const ss = r.toFixed(3).padStart(6, '0')
  if (withHours) {
    return `${h.toString().padStart(2, '0')}:${mm}:${ss}`
  }
  return `${mm}:${ss}`
}

// 视频超过 1 小时时左右两侧时间统一展示 HH:MM:SS.s，避免出现 60:15 这种没换算到小时的写法
const showHours = computed(() => duration.value >= 3600)

// range input 自身的 background 渐变需要一个 0~1 之间的数值变量；
// 用 calc 把颜色分界点对齐到 thumb 中心（thumb 中心只能在 [r, width-r] 区间运动）。
const sliderProgressStyle = computed(() => {
  const d = duration.value
  const c = currentTime.value
  const p = d > 0 ? Math.min(1, Math.max(0, c / d)) : 0
  return { '--le-progress': p.toString() } as Record<string, string>
})

function pathBasename(p: string): string {
  if (!p) return ''
  const idx = Math.max(p.lastIndexOf('\\'), p.lastIndexOf('/'))
  return idx >= 0 ? p.slice(idx + 1) : p
}

// === 选择 LOGO ===
async function chooseLogo() {
  const selected = await openDialog({
    multiple: false,
    directory: false,
    title: '选择 LOGO 图片',
    filters: [
      { name: '图片', extensions: ['png', 'jpg', 'jpeg', 'webp', 'bmp'] }
    ]
  })
  if (typeof selected === 'string') {
    await applyLogoPath(selected)
  }
}

// === 删除最近 LOGO ===
// 立即 emit 通知父组件，父组件 watch recentLogos 会 debounce 同步到 AppConfig。
// 若删除的是当前选中 LOGO，则同步清空 logoPath（用户已表态不要这张）；
// 但保留 logoLayouts 中对应的布局记忆，下次手动选回时仍可复用。
function removeRecent(path: string, ev: Event) {
  ev.stopPropagation()
  const next = props.recentLogos.filter((r) => r.path !== path)
  emit('update-recent', next)
  if (logoPath.value === path) {
    logoPath.value = ''
    logoAspect.value = 1
    dirty.value = true
  }
}

// === 重命名最近 LOGO（仅修改昵称，不动原文件）===
// editingPath = null 表示无项处于编辑态；input 失焦或回车提交，Esc 取消。
const editingPath = ref<string | null>(null)
const editingName = ref('')
let editingInputEl: HTMLInputElement | null = null
// v-for 内的 ref 默认会聚成数组；这里只有一个 input 处于编辑态，用函数 ref 直接拿到单元素。
function setEditingInputRef(el: Element | { $el?: Element } | null) {
  editingInputEl = (el as HTMLInputElement | null) ?? null
}

function displayLabel(item: RecentLogo): string {
  const name = (item.displayName ?? '').trim()
  return name || pathBasename(item.path)
}

function startRename(item: RecentLogo, ev: Event) {
  ev.stopPropagation()
  editingPath.value = item.path
  // 默认填入当前展示名（若无别名则是文件名），方便用户在原基础上微调
  editingName.value = displayLabel(item)
  void nextTick(() => {
    editingInputEl?.focus()
    editingInputEl?.select()
  })
}

function commitRename(item: RecentLogo) {
  if (editingPath.value !== item.path) return
  const trimmed = editingName.value.trim()
  const original = pathBasename(item.path)
  // 空字符串或与文件名一致 → 视为清除昵称，避免冗余存储
  const nextName = trimmed && trimmed !== original ? trimmed : undefined
  // 只在确实变化时才 emit，避免无谓的持久化往返
  if ((item.displayName ?? undefined) !== nextName) {
    const next = props.recentLogos.map((r) =>
      r.path === item.path ? { ...r, displayName: nextName } : r
    )
    emit('update-recent', next)
  }
  editingPath.value = null
  editingName.value = ''
}

function cancelRename() {
  editingPath.value = null
  editingName.value = ''
}

// === 拖拽：复用全局 pendingDrop ===
// App.vue 已注册 tauri://drag-drop 监听并写入 pendingDrop.raw；
// 编辑器打开时若 raw 含图片扩展名则直接套用，避免与 HomeView 拖入视频/字幕逻辑冲突。
watch(pendingDrop, (drop) => {
  if (!drop) return
  const img = drop.raw.find((p) => isImagePath(p))
  if (!img) return
  void applyLogoPath(img)
})

async function applyLogoPath(path: string) {
  logoPath.value = path
  await preloadLogoAspect(path)
  // 优先：若 (当前桶, 新 LOGO) 有记忆，直接套用其完整布局
  const hit = findLayoutEntry(currentBucket.value, path)
  if (hit) {
    xPct.value = hit.xPct
    yPct.value = hit.yPct
    wPct.value = hit.wPct
    hPct.value = hit.hPct
    dirty.value = true
    return
  }
  // 未命中记忆：保持位置/宽度，按新图宽高比反算高度，避免变形
  if (logoAspect.value > 0 && props.videoWidth && props.videoHeight) {
    hPct.value = heightPctForWidthPct(wPct.value)
  }
  dirty.value = true
}

function clamp01(v: number) {
  if (!Number.isFinite(v)) return 0
  return Math.min(1, Math.max(0, v))
}

// === 拖动 / 缩放 ===
type DragMode = 'move' | 'tl' | 'tr' | 'bl' | 'br'
let dragMode: DragMode | null = null
let dragStart = {
  px: 0,
  py: 0,
  stageW: 0,
  stageH: 0,
  x: 0,
  y: 0,
  w: 0,
  h: 0
}

function onPointerDown(mode: DragMode, ev: PointerEvent) {
  if (!stageEl.value || !hasValidLogo.value) return
  ev.preventDefault()
  ev.stopPropagation()
  const rect = stageEl.value.getBoundingClientRect()
  dragMode = mode
  dragStart = {
    px: ev.clientX,
    py: ev.clientY,
    stageW: rect.width,
    stageH: rect.height,
    x: xPct.value,
    y: yPct.value,
    w: wPct.value,
    h: hPct.value
  }
  ;(ev.target as Element).setPointerCapture(ev.pointerId)
}

function onPointerMove(ev: PointerEvent) {
  if (!dragMode) return
  const dxPct = (ev.clientX - dragStart.px) / dragStart.stageW
  const dyPct = (ev.clientY - dragStart.py) / dragStart.stageH

  if (dragMode === 'move') {
    // 限制 LOGO 完整地留在画面内：x ∈ [0, 1-w], y ∈ [0, 1-h]
    const maxX = Math.max(0, 1 - wPct.value)
    const maxY = Math.max(0, 1 - hPct.value)
    xPct.value = Math.min(maxX, Math.max(0, dragStart.x + dxPct))
    yPct.value = Math.min(maxY, Math.max(0, dragStart.y + dyPct))
    dirty.value = true
    return
  }

  // 缩放：以对角点为锚保持位置；用宽度变化驱动，再按真实视频比例反算高度。
  let newW = dragStart.w
  let newX = dragStart.x
  let newY = dragStart.y

  switch (dragMode) {
    case 'br': {
      newW = dragStart.w + dxPct
      break
    }
    case 'tr': {
      newW = dragStart.w + dxPct
      break
    }
    case 'bl': {
      newW = dragStart.w - dxPct
      newX = dragStart.x + dxPct
      break
    }
    case 'tl': {
      newW = dragStart.w - dxPct
      newX = dragStart.x + dxPct
      break
    }
  }

  if (newW < 0.02) newW = 0.02
  if (newW > 1) newW = 1
  let newH = heightPctForWidthPct(newW)

  // 顶部手柄要让"下边沿"保持锚定
  if (dragMode === 'tl' || dragMode === 'tr') {
    const bottomY = dragStart.y + dragStart.h
    newY = bottomY - newH
  }

  // 最终约束：LOGO 不允许越出画面右边 / 下边 / 左边 / 上边。
  // 超出时把宽度截到能放得下、再按宽高比反算高度，避免拉伸。
  if (newX < 0) newX = 0
  if (newY < 0) newY = 0
  const maxW = 1 - newX
  if (newW > maxW) {
    newW = Math.max(0.02, maxW)
    newH = heightPctForWidthPct(newW)
    if (dragMode === 'tl' || dragMode === 'tr') {
      const bottomY = dragStart.y + dragStart.h
      newY = Math.max(0, bottomY - newH)
    }
  }
  const maxH = 1 - newY
  if (newH > maxH) {
    newH = Math.max(0.001, maxH)
    newW = Math.max(0.02, widthPctForHeightPct(newH))
    if (dragMode === 'tl' || dragMode === 'tr') {
      const bottomY = dragStart.y + dragStart.h
      newY = Math.max(0, bottomY - newH)
    }
  }

  xPct.value = newX
  yPct.value = newY
  wPct.value = newW
  hPct.value = newH
  dirty.value = true
}

function onPointerUp(ev: PointerEvent) {
  if (!dragMode) return
  dragMode = null
  try {
    ;(ev.target as Element).releasePointerCapture(ev.pointerId)
  } catch {
    /* ignore */
  }
}

// === 保存 / 取消 ===
function buildLayout(): LogoLayout {
  return {
    path: logoPath.value,
    xPct: xPct.value,
    yPct: yPct.value,
    wPct: wPct.value,
    hPct: hPct.value
  }
}

function buildRecent(): RecentLogo[] {
  const now = Date.now()
  const path = logoPath.value
  if (!path) return props.recentLogos
  const filtered = props.recentLogos.filter((r) => r.path !== path)
  return [{ path, lastUsedAt: now }, ...filtered].slice(0, 10)
}

// 构造下一份 logoLayouts：仅当当前视频命中常见分辨率桶时 upsert，
// 否则返回原列表（非常见分辨率不污染持久化记忆）。
function buildLogoLayouts(): LogoLayoutEntry[] {
  const bucket = currentBucket.value
  const path = logoPath.value
  if (!bucket || !path) return props.logoLayouts
  const now = Date.now()
  const filtered = props.logoLayouts.filter(
    (e) => !(e.bucket === bucket && e.path === path)
  )
  return [
    {
      bucket,
      path,
      xPct: xPct.value,
      yPct: yPct.value,
      wPct: wPct.value,
      hPct: hPct.value,
      lastUsedAt: now
    },
    ...filtered
  ]
}

function onSave() {
  if (!hasValidLogo.value) {
    frameError.value = '请先选择 LOGO 图片'
    return
  }
  emit('save', buildLayout(), buildRecent(), buildLogoLayouts())
}

function onCancel() {
  if (dirty.value) {
    const ok = window.confirm('当前 LOGO 配置尚未保存，确定要关闭吗？')
    if (!ok) return
  }
  emit('cancel')
}

function onKey(ev: KeyboardEvent) {
  if (ev.key === 'Escape') {
    onCancel()
  }
}

// === 生命周期 ===
onMounted(async () => {
  applyInitialLayout()
  document.addEventListener('keydown', onKey)
  // 先固定到第 1 秒抽一帧（前 0s 可能是黑场）
  currentTime.value = Math.min(1, duration.value || 1)
  await nextTick()
  // 计算 stage 尺寸并监听变化
  recalcStageBox()
  if (stageWrapEl.value && typeof ResizeObserver !== 'undefined') {
    stageResizeObserver = new ResizeObserver(() => recalcStageBox())
    stageResizeObserver.observe(stageWrapEl.value)
  }
  window.addEventListener('resize', recalcStageBox)
  void refreshFrame()
})

onBeforeUnmount(() => {
  document.removeEventListener('keydown', onKey)
  if (frameDebounce) clearTimeout(frameDebounce)
  if (stageResizeObserver) {
    stageResizeObserver.disconnect()
    stageResizeObserver = null
  }
  window.removeEventListener('resize', recalcStageBox)
})
</script>

<template>
  <div class="logo-editor-overlay app-modal-active" @pointermove="onPointerMove" @pointerup="onPointerUp">
    <div class="logo-editor" role="dialog" aria-modal="true">
      <header class="le-header">
        <!-- 关闭入口收敛到底部「取消」按钮 + ESC 键，避免三个重复出口 -->
        <h2>配置 LOGO 位置</h2>
      </header>

      <div class="le-body">
        <aside class="le-sidebar">
          <div
            class="le-dropzone"
            :class="{ active: globalDragActive }"
            @click="chooseLogo"
            role="button"
            tabindex="0"
            @keydown.enter.prevent="chooseLogo"
            @keydown.space.prevent="chooseLogo"
          >
            <svg class="le-dropzone-icon" viewBox="0 0 24 24" width="28" height="28" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <rect x="3" y="3" width="18" height="18" rx="2" />
              <circle cx="8.5" cy="8.5" r="1.5" />
              <path d="m21 15-5-5L5 21" />
            </svg>
            <div class="le-dropzone-title">{{ globalDragActive ? '松开以载入图片' : '选择 LOGO 图片' }}</div>
            <div class="le-dropzone-hint">点击选择，或拖入 PNG / JPG / WEBP / BMP</div>
          </div>

          <div class="le-current">
            <div class="le-section-title">当前 LOGO</div>
            <div v-if="hasValidLogo" class="le-current-card">
              <img :src="logoUrl" class="le-thumb" alt="LOGO" />
              <div class="le-current-name" v-tooltip="currentDisplayName">{{ currentDisplayName }}</div>
              <div class="le-current-path" v-tooltip="logoPath">
                <svg class="le-path-icon" viewBox="0 0 24 24" width="11" height="11" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                  <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
                </svg>
                <span>{{ logoPath }}</span>
              </div>
            </div>
            <div v-else class="le-empty">未选择 LOGO</div>
          </div>

          <div class="le-recent">
            <div class="le-section-title">最近使用</div>
            <ul v-if="recentLogos.length" class="le-recent-list">
              <li
                v-for="item in recentLogos"
                :key="item.path"
                :class="{ active: item.path === logoPath, editing: editingPath === item.path }"
                @click="editingPath !== item.path && applyLogoPath(item.path)"
                v-tooltip="item.path"
              >
                <img :src="convertFileSrc(item.path)" class="le-thumb-sm" alt="" />
                <input
                  v-if="editingPath === item.path"
                  :ref="setEditingInputRef"
                  v-model="editingName"
                  class="le-recent-input"
                  type="text"
                  maxlength="60"
                  :placeholder="pathBasename(item.path)"
                  @click.stop
                  @keydown.enter.prevent="commitRename(item)"
                  @keydown.esc.prevent="cancelRename"
                  @blur="commitRename(item)"
                />
                <span v-else class="le-recent-name">{{ displayLabel(item) }}</span>
                <button
                  v-if="editingPath !== item.path"
                  type="button"
                  class="le-recent-act le-recent-rename"
                  v-tooltip="'重命名'"
                  aria-label="重命名"
                  @click="startRename(item, $event)"
                >
                  <svg viewBox="0 0 24 24" width="13" height="13" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                    <path d="M12 20h9" />
                    <path d="M16.5 3.5a2.121 2.121 0 1 1 3 3L7 19l-4 1 1-4 12.5-12.5z" />
                  </svg>
                </button>
                <button
                  v-if="editingPath !== item.path"
                  type="button"
                  class="le-recent-act le-recent-del"
                  v-tooltip="`移除 ${displayLabel(item)}`"
                  aria-label="移除"
                  @click="removeRecent(item.path, $event)"
                >
                  <svg viewBox="0 0 24 24" width="13" height="13" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                    <path d="M18 6 6 18M6 6l12 12" />
                  </svg>
                </button>
              </li>
            </ul>
            <div v-else class="le-empty">暂无记录</div>
          </div>
        </aside>

        <section class="le-main">
          <div class="le-status-bar">
            <span>位置：<code>x={{ pxX }} y={{ pxY }}</code></span>
            <span>尺寸：<code>{{ pxW }} × {{ pxH }}</code></span>
            <span>百分比：<code>{{ (xPct * 100).toFixed(1) }}%, {{ (yPct * 100).toFixed(1) }}% / {{ (wPct * 100).toFixed(1) }}%, {{ (hPct * 100).toFixed(1) }}%</code></span>
            <span class="muted">视频：{{ videoWidth ?? '?' }}×{{ videoHeight ?? '?' }}</span>
          </div>

          <div class="le-stage-wrap" ref="stageWrapEl">
            <div
              ref="stageEl"
              class="le-stage"
              :style="stageBoxStyle"
            >
              <img v-if="frameUrl" :src="frameUrl" class="le-frame" alt="预览帧" draggable="false" />
              <div v-else class="le-frame-placeholder">
                <span v-if="frameLoading">抽帧中…</span>
                <span v-else-if="frameError">{{ frameError }}</span>
                <span v-else>等待视频帧…</span>
              </div>

              <div
                v-if="hasValidLogo"
                class="le-logo-box"
                :style="{
                  left: xPct * 100 + '%',
                  top: yPct * 100 + '%',
                  width: wPct * 100 + '%',
                  height: hPct * 100 + '%'
                }"
                @pointerdown.stop="(e) => onPointerDown('move', e)"
              >
                <img :src="logoUrl" class="le-logo-img" alt="LOGO" draggable="false" />
                <span class="le-handle tl" @pointerdown.stop="(e) => onPointerDown('tl', e)"></span>
                <span class="le-handle tr" @pointerdown.stop="(e) => onPointerDown('tr', e)"></span>
                <span class="le-handle bl" @pointerdown.stop="(e) => onPointerDown('bl', e)"></span>
                <span class="le-handle br" @pointerdown.stop="(e) => onPointerDown('br', e)"></span>
              </div>
            </div>
          </div>

          <div class="le-timeline">
            <span class="le-time le-time-current">{{ formatTime(currentTime, showHours) }}</span>
            <div
              class="le-slider-wrap"
              :class="{ disabled: !duration }"
              :style="sliderProgressStyle"
            >
              <input
                type="range"
                :min="0"
                :max="Math.max(1, duration)"
                :step="0.1"
                :disabled="!duration"
                v-model.number="currentTime"
                class="le-slider"
              />
            </div>
            <span class="le-time muted">{{ formatTime(duration, showHours) }}</span>
            <button
              class="le-refresh-btn"
              type="button"
              :class="{ loading: frameLoading }"
              :disabled="frameLoading"
              v-tooltip="frameLoading ? '抽帧中…' : '重新抽取当前帧'"
              :aria-label="frameLoading ? '抽帧中' : '重新抽取当前帧'"
              @click="refreshFrame"
            >
              <svg viewBox="0 0 24 24" width="13" height="13" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <path d="M21 12a9 9 0 1 1-3-6.7" />
                <path d="M21 4v5h-5" />
              </svg>
            </button>
          </div>
        </section>
      </div>

      <footer class="le-footer">
        <button class="secondary" type="button" @click="onCancel">取消</button>
        <button class="primary" type="button" :disabled="!hasValidLogo" @click="onSave">保存</button>
      </footer>
    </div>
  </div>
</template>

<script lang="ts">
export default { name: 'LogoEditor' }
</script>

<style scoped>
.logo-editor-overlay {
  align-items: center;
  background: rgba(15, 23, 32, 0.55);
  display: flex;
  inset: 0;
  justify-content: center;
  position: fixed;
  z-index: 200;
  backdrop-filter: blur(2px);
  /* 让 flex 居中的弹窗主体避开顶部 36px 标题栏区域：
     原 inset:0 + 96vh 弹窗导致顶部圆角侵入标题栏，
     按钮图标垂直跨越「mask 深色 / 弹窗白色」两种背景，对比度断层。
     overlay 仍覆盖标题栏区，但弹窗下移 36px，视觉层次干净。 */
  padding-top: 36px;
}

.logo-editor {
  background: #ffffff;
  border-radius: 12px;
  box-shadow: 0 24px 60px rgba(15, 23, 42, 0.32);
  display: flex;
  flex-direction: column;
  /* 强制撑满剩余空间：弹窗内 stage-wrap 是 flex:1，
     竖屏视频的预览框依赖 wrap 实际高度算最大宽度，
     不撑满高度会让竖屏视频缩成窄长条。
     注意：原 96vh 改为减去标题栏 36px 后的可用高度，
     避免与上方 padding-top:36px 叠加后底部被裁切。 */
  height: calc(96vh - 36px);
  max-height: calc(96vh - 36px);
  max-width: 1500px;
  overflow: hidden;
  width: 96vw;
}

.le-header {
  align-items: center;
  border-bottom: 1px solid #eef2f5;
  display: flex;
  padding: 10px 18px;
}
.le-header h2 {
  font-size: 16px;
  margin: 0;
}

.le-body {
  display: grid;
  flex: 1 1 auto;
  grid-template-columns: 260px 1fr;
  min-height: 0;
}

.le-sidebar {
  border-right: 1px solid #eef2f5;
  display: flex;
  flex-direction: column;
  gap: 12px;
  overflow-y: auto;
  padding: 12px;
}

.primary {
  background: #176b87;
  border: 1px solid #176b87;
  border-radius: 8px;
  color: #fff;
  cursor: pointer;
  font-weight: 600;
  min-height: 36px;
  padding: 0 14px;
}
.primary:hover {
  background: #14617a;
}
.primary:disabled {
  background: #c0d4dd;
  border-color: #c0d4dd;
  cursor: not-allowed;
}
.secondary {
  background: #fff;
  border: 1px solid #d6dee5;
  border-radius: 8px;
  color: #18202a;
  cursor: pointer;
  min-height: 36px;
  padding: 0 14px;
}
.secondary:hover {
  border-color: #176b87;
  color: #176b87;
}
.secondary:disabled {
  color: #9aa7b1;
  cursor: not-allowed;
}
.block {
  width: 100%;
}

.le-section-title {
  color: #5b6772;
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 0.4px;
  margin-bottom: 6px;
  text-transform: uppercase;
}

.le-current-card {
  align-items: center;
  background: #fff;
  border: 1px solid #e6edf2;
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 10px 8px 8px;
}
.le-thumb {
  background: #f7fafb url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='16' height='16'><rect width='8' height='8' fill='%23e7eef2'/><rect x='8' y='8' width='8' height='8' fill='%23e7eef2'/></svg>");
  background-size: 12px 12px;
  border: 1px solid #eef2f5;
  border-radius: 6px;
  height: 64px;
  max-width: 100%;
  object-fit: contain;
  padding: 4px;
}
/* 主标题：放大 + 加粗 + 深色，承担"这是哪个 LOGO"的信息焦点 */
.le-current-name {
  color: #18202a;
  font-size: 14px;
  font-weight: 700;
  letter-spacing: 0.2px;
  line-height: 1.3;
  margin-top: 4px;
  overflow: hidden;
  text-align: center;
  text-overflow: ellipsis;
  white-space: nowrap;
  width: 100%;
}
/* 完整路径：弱化为附属元信息——等宽字体、淡灰、虚线分隔，与主标题视觉强弱拉开 */
.le-current-path {
  align-items: flex-start;
  border-top: 1px dashed #eef2f5;
  color: #9aa7b1;
  display: flex;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 10.5px;
  gap: 4px;
  line-height: 1.5;
  margin-top: 2px;
  padding-top: 6px;
  width: 100%;
}
.le-current-path .le-path-icon {
  color: #c0cbd2;
  flex-shrink: 0;
  margin-top: 2px;
}
.le-current-path span {
  flex: 1;
  /* 让 Windows 反斜杠 / 长文件名可以在任意字符处自动换行展示完整 path */
  overflow-wrap: anywhere;
  word-break: break-all;
}

.le-empty {
  color: #9aa7b1;
  font-size: 12px;
  padding: 8px 0;
}

.le-recent-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
  list-style: none;
  margin: 0;
  padding: 0;
}
.le-recent-list li {
  align-items: center;
  border-radius: 6px;
  cursor: pointer;
  display: flex;
  gap: 8px;
  height: 44px;
  min-height: 44px;
  overflow: hidden;
  padding: 6px;
  transition: background 0.12s ease;
}
.le-recent-list li:hover {
  background: #eef7fa;
}
.le-recent-list li.active {
  background: #176b87;
  color: #fff;
}
.le-thumb-sm {
  background: #f0f3f6;
  border-radius: 3px;
  flex-shrink: 0;
  height: 28px;
  object-fit: contain;
  width: 28px;
}
.le-recent-name {
  flex: 1;
  font-size: 12px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  /* hover 时给按钮让位，避免按钮"叠"在文字尾部 */
  transition: padding-right 0.15s ease;
}
.le-recent-list li:hover .le-recent-name,
.le-recent-list li:focus-within .le-recent-name {
  padding-right: 52px;
}

/* dropzone：替代裸 button，作为选择/拖拽双入口 */
.le-dropzone {
  align-items: center;
  background: #f7fafb;
  border: 1.5px dashed #c7d3dc;
  border-radius: 10px;
  color: #5b6772;
  cursor: pointer;
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 14px 10px;
  text-align: center;
  transition: border-color 0.15s ease, background 0.15s ease, color 0.15s ease, transform 0.15s ease;
}
.le-dropzone:hover {
  background: #eef7fa;
  border-color: #176b87;
  color: #176b87;
}
.le-dropzone:focus-visible {
  border-color: #176b87;
  color: #176b87;
  outline: 2px solid rgba(23, 107, 135, 0.25);
  outline-offset: 2px;
}
.le-dropzone.active {
  background: #e3f3f8;
  border-color: #38bdf8;
  border-style: solid;
  color: #176b87;
  transform: scale(1.01);
}
.le-dropzone-icon {
  color: inherit;
}
.le-dropzone-title {
  font-size: 13px;
  font-weight: 600;
}
.le-dropzone-hint {
  color: #9aa7b1;
  font-size: 11px;
}
.le-dropzone.active .le-dropzone-hint {
  color: #176b87;
}

/* 最近列表按钮：绝对定位漂浮在右端，hover 才显形。
   平时完全不占文字宽度，避免文件名被两个 22px 按钮挤成省略号。 */
.le-recent-list li {
  padding-right: 6px;
  position: relative;
}
.le-recent-list li.editing {
  cursor: default;
}
.le-recent-act {
  align-items: center;
  background: rgba(255, 255, 255, 0.92);
  border: 1px solid #d8e2e8;
  border-radius: 3px;
  color: inherit;
  cursor: pointer;
  display: flex;
  height: 22px;
  justify-content: center;
  min-height: 22px;
  opacity: 0;
  padding: 0;
  pointer-events: none;
  position: absolute;
  top: 50%;
  transform: translateY(-50%);
  transition: opacity 0.12s ease, background 0.12s ease, color 0.12s ease;
  width: 22px;
}
/* 删除按钮贴最右，重命名按钮紧挨左边 */
.le-recent-act.le-recent-rename {
  right: 32px;
}
.le-recent-act.le-recent-del {
  right: 7px;
}
.le-recent-list li:hover .le-recent-act,
.le-recent-list li:focus-within .le-recent-act {
  opacity: 0.85;
  pointer-events: auto;
}
.le-recent-list li.active:hover .le-recent-act,
.le-recent-list li.active:focus-within .le-recent-act {
  background: rgba(255, 255, 255, 0.18);
}
.le-recent-act:hover {
  background: #f4f8fa;
  border-color: #176b87;
  color: #176b87;
  opacity: 1 !important;
}
.le-recent-list li.active .le-recent-act:hover {
  background: rgba(255, 255, 255, 0.22);
  border-color: rgba(255, 255, 255, 0.6);
  color: #fff;
}
.le-recent-list li.active .le-recent-rename:hover {
  background: #ffffff;
  border-color: #ffffff;
  color: #176b87;
}
.le-recent-del:hover {
  background: rgba(220, 38, 38, 0.14) !important;
  border-color: #dc2626 !important;
  color: #dc2626 !important;
}
.le-recent-list li.active .le-recent-del:hover {
  background: #ffffff !important;
  border-color: #ffffff !important;
  color: #dc2626 !important;
}

/* inline 重命名 input：占据原文件名位置，避免列表项高度跳动。
   用 input.le-recent-input 提升特异性，覆盖全局 input { min-height: 38px } */
input.le-recent-input {
  background: #fff;
  border: 1px solid #176b87;
  border-radius: 4px;
  color: #18202a;
  flex: 1;
  font-size: 12px;
  height: 26px;
  line-height: 24px;
  min-height: 26px;
  min-width: 0;
  outline: none;
  padding: 0 6px;
}
.le-recent-list li.active input.le-recent-input {
  background: #fff;
  color: #18202a;
}

.le-main {
  display: flex;
  flex-direction: column;
  /* 状态栏 ↔ 预览区 之间留出更明显的呼吸距离，避免视觉上贴在一起 */
  gap: 14px;
  min-height: 0;
  min-width: 0;
  padding: 10px 12px 12px;
}

.le-status-bar {
  align-items: center;
  background: #f7fafb;
  border: 1px solid #e6edf2;
  border-radius: 8px;
  color: #18202a;
  display: flex;
  flex-wrap: wrap;
  font-size: 11.5px;
  gap: 14px;
  /* 信息条整体压扁：上下 padding 收紧到 3px，行高同步降到 1.4 让单行更紧凑 */
  line-height: 1.4;
  padding: 3px 10px;
}
.le-status-bar code {
  background: rgba(23, 107, 135, 0.08);
  border-radius: 4px;
  color: #145a72;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  padding: 1px 6px;
}
/* 局部覆盖全局 .muted：去掉 margin-top:5px / font-size:13px 的干扰 */
.le-status-bar .muted {
  font-size: inherit;
  margin-top: 0;
}
.muted {
  color: #9aa7b1;
}

.le-stage-wrap {
  align-items: center;
  background: #0e1822;
  border-radius: 8px;
  display: flex;
  flex: 1 1 auto;
  justify-content: center;
  min-height: 0;
  overflow: hidden;
  padding: 6px;
}
.le-stage {
  background: #000;
  flex-shrink: 0;
  max-height: 100%;
  max-width: 100%;
  position: relative;
}
.le-frame {
  display: block;
  height: 100%;
  object-fit: contain;
  pointer-events: none;
  user-select: none;
  width: 100%;
}
.le-frame-placeholder {
  align-items: center;
  color: #9aa7b1;
  display: flex;
  height: 100%;
  justify-content: center;
  width: 100%;
}

.le-logo-box {
  border: 1px dashed rgba(56, 189, 248, 0.85);
  box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.35);
  cursor: move;
  position: absolute;
  touch-action: none;
}
.le-logo-img {
  height: 100%;
  pointer-events: none;
  user-select: none;
  width: 100%;
}
.le-handle {
  background: #38bdf8;
  border: 2px solid #fff;
  border-radius: 50%;
  height: 12px;
  position: absolute;
  width: 12px;
}
.le-handle.tl {
  cursor: nwse-resize;
  left: -7px;
  top: -7px;
}
.le-handle.tr {
  cursor: nesw-resize;
  right: -7px;
  top: -7px;
}
.le-handle.bl {
  bottom: -7px;
  cursor: nesw-resize;
  left: -7px;
}
.le-handle.br {
  bottom: -7px;
  cursor: nwse-resize;
  right: -7px;
}

.le-timeline {
  align-items: center;
  display: flex;
  gap: 10px;
}
.le-time {
  color: #5b6772;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 11.5px;
  font-variant-numeric: tabular-nums;
  /* 3 位毫秒后字符数 9（00:00.000）；mono 字体 11.5px 约 7px/字符 → 64px 起步 */
  min-width: 64px;
}
/* 当前时间需要"我在这里"的视觉锚——主色 + 加粗 */
.le-time.le-time-current {
  color: #176b87;
  font-weight: 600;
}
.le-time.muted {
  color: #9aa7b1;
  text-align: right;
}

/* 自定义 slider：
   - <input> 自身用 background 渐变画 track（含已播放/未播放配色）；
   - wrapper::after 画 thumb，position absolute + left calc(progress * 100%)，
     位置与渐变分界点天然对齐；
   - input 自带的 ::-webkit-slider-thumb 设为透明（占位以保证拖拽事件），
     视觉 thumb 完全由 ::after 决定。
   - thumb 默认就可见（旧版 hover 才显形会让用户误以为 slider 不可拖）。 */
.le-slider-wrap {
  align-items: center;
  display: flex;
  flex: 1;
  height: 14px;
  position: relative;
}
.le-slider-wrap::after {
  background: #ffffff;
  border: 2px solid #176b87;
  border-radius: 50%;
  box-shadow: 0 1px 3px rgba(15, 23, 42, 0.18);
  content: '';
  height: 12px;
  left: calc(var(--le-progress, 0) * 100%);
  opacity: 1;
  pointer-events: none;
  position: absolute;
  top: 50%;
  transform: translate(-50%, -50%);
  transition: transform 0.12s ease, box-shadow 0.12s ease, border-color 0.12s ease;
  width: 12px;
}
.le-slider-wrap:hover::after {
  border-color: #14617a;
  box-shadow: 0 2px 6px rgba(23, 107, 135, 0.35);
  transform: translate(-50%, -50%) scale(1.18);
}
.le-slider-wrap:active::after {
  transform: translate(-50%, -50%) scale(1.1);
}
.le-slider-wrap.disabled {
  opacity: 0.55;
}
.le-slider-wrap.disabled::after {
  opacity: 0;
}

.le-slider {
  -webkit-appearance: none;
  appearance: none;
  background: linear-gradient(
    to right,
    #176b87 0,
    #176b87 calc(var(--le-progress, 0) * 100%),
    #dde4ea calc(var(--le-progress, 0) * 100%),
    #dde4ea 100%
  );
  border: 0;
  border-radius: 999px;
  cursor: pointer;
  /* 3px 极细轨 + 12px thumb 形成"轨细钮显"对比，整体时间轴更扁 */
  height: 3px;
  margin: 0;
  outline: none;
  padding: 0;
  width: 100%;
}
.le-slider:disabled {
  cursor: not-allowed;
}
.le-slider::-webkit-slider-runnable-track {
  background: transparent;
  border-radius: 999px;
  height: 3px;
}
.le-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  background: transparent;
  border: 0;
  cursor: pointer;
  height: 14px;
  width: 14px;
}
.le-slider:disabled::-webkit-slider-thumb {
  cursor: not-allowed;
}

/* 刷新按钮改为 icon-only 圆形：与播放控制语义同行更协调，不抢"刷新"两字的视觉权重。
   loading 时 svg 持续旋转，比"抽帧中"文字更直观。 */
.le-refresh-btn {
  align-items: center;
  background: #fff;
  border: 1px solid #d6dee5;
  border-radius: 50%;
  color: #5b6772;
  cursor: pointer;
  display: flex;
  flex-shrink: 0;
  height: 26px;
  justify-content: center;
  /* 必须覆盖全局 button { min-height: 36px }，否则按钮被撑成竖椭圆 */
  min-height: 0;
  padding: 0;
  transition: border-color 0.12s ease, color 0.12s ease, background 0.12s ease, box-shadow 0.12s ease;
  width: 26px;
}
.le-refresh-btn:hover:not(:disabled) {
  background: #f4fafc;
  border-color: #176b87;
  box-shadow: 0 1px 4px rgba(23, 107, 135, 0.18);
  color: #176b87;
}
.le-refresh-btn:active:not(:disabled) {
  box-shadow: inset 0 1px 2px rgba(15, 23, 42, 0.1);
}
.le-refresh-btn:disabled {
  color: #9aa7b1;
  cursor: not-allowed;
  opacity: 0.7;
}
.le-refresh-btn.loading svg {
  animation: le-spin 0.9s linear infinite;
}
@keyframes le-spin {
  from { transform: rotate(0deg); }
  to   { transform: rotate(360deg); }
}

.le-footer {
  border-top: 1px solid #eef2f5;
  display: flex;
  gap: 10px;
  justify-content: flex-end;
  padding: 10px 18px;
}
</style>
