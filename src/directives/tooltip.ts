import type { Directive, DirectiveBinding } from 'vue'

// 全局单例：所有 v-tooltip 共用同一个浮层节点 + 同一个箭头节点
// 单例的好处：DOM 节点数量恒定 O(1)，hover 切换无频繁创建销毁
let tipEl: HTMLDivElement | null = null
let arrowEl: HTMLDivElement | null = null
let activeTarget: HTMLElement | null = null
let showTimer: number | null = null
let hideTimer: number | null = null

const SHOW_DELAY = 200   // hover 多久才显示，防擦肩而过的闪烁
const HIDE_DELAY = 80    // 离开后短暂保留，便于跨过缝隙到下一个元素
const GAP = 8            // 浮层与触发元素的间距（含三角箭头空间）
const VIEWPORT_PAD = 6   // 浮层距视口边缘的最小留白

function ensureTipEl(): HTMLDivElement {
  if (tipEl) return tipEl
  tipEl = document.createElement('div')
  tipEl.className = 'app-tooltip'
  tipEl.setAttribute('role', 'tooltip')
  arrowEl = document.createElement('div')
  arrowEl.className = 'app-tooltip-arrow'
  tipEl.appendChild(arrowEl)
  const text = document.createElement('span')
  text.className = 'app-tooltip-text'
  tipEl.appendChild(text)
  document.body.appendChild(tipEl)
  return tipEl
}

function setText(content: string) {
  const tip = ensureTipEl()
  const textNode = tip.querySelector('.app-tooltip-text') as HTMLSpanElement
  textNode.textContent = content
}

type Placement = 'top' | 'bottom'

function place(target: HTMLElement) {
  const tip = ensureTipEl()
  // 必须先显示 + 重置位置，才能拿到 tip 的尺寸
  tip.style.display = 'block'
  tip.style.left = '0px'
  tip.style.top = '0px'
  tip.classList.remove('app-tooltip-bottom')

  const rect = target.getBoundingClientRect()
  const tipRect = tip.getBoundingClientRect()
  const vw = window.innerWidth
  const vh = window.innerHeight

  // 默认放下方，下方放不下再放上方（与原生 title 直觉一致：贴近鼠标）
  const spaceBelow = vh - rect.bottom
  const spaceAbove = rect.top
  const placement: Placement =
    spaceBelow >= tipRect.height + GAP + VIEWPORT_PAD || spaceBelow >= spaceAbove
      ? 'bottom'
      : 'top'

  // 水平居中对齐到触发元素，再做视口边缘 clamp
  let left = rect.left + rect.width / 2 - tipRect.width / 2
  left = Math.max(VIEWPORT_PAD, Math.min(left, vw - tipRect.width - VIEWPORT_PAD))

  let top: number
  if (placement === 'bottom') {
    top = rect.bottom + GAP
    tip.classList.add('app-tooltip-bottom')
  } else {
    top = rect.top - tipRect.height - GAP
  }

  tip.style.left = `${Math.round(left)}px`
  tip.style.top = `${Math.round(top)}px`

  // 箭头：跟着触发元素中点走，相对浮层 left 偏移
  if (arrowEl) {
    const targetCenter = rect.left + rect.width / 2
    const arrowLeft = Math.max(8, Math.min(targetCenter - left, tipRect.width - 8))
    arrowEl.style.left = `${Math.round(arrowLeft)}px`
  }
}

function showFor(target: HTMLElement) {
  const content = target.dataset.tooltipContent
  if (!content) return
  setText(content)
  place(target)
  // 触发淡入动画
  requestAnimationFrame(() => {
    ensureTipEl().classList.add('is-visible')
  })
  activeTarget = target
}

function hide() {
  if (!tipEl) return
  tipEl.classList.remove('is-visible')
  tipEl.style.display = 'none'
  activeTarget = null
}

function clearTimers() {
  if (showTimer !== null) {
    window.clearTimeout(showTimer)
    showTimer = null
  }
  if (hideTimer !== null) {
    window.clearTimeout(hideTimer)
    hideTimer = null
  }
}

function onEnter(e: Event) {
  const el = e.currentTarget as HTMLElement
  clearTimers()
  showTimer = window.setTimeout(() => {
    showFor(el)
    showTimer = null
  }, SHOW_DELAY)
}

function onLeave() {
  clearTimers()
  hideTimer = window.setTimeout(() => {
    hide()
    hideTimer = null
  }, HIDE_DELAY)
}

// 全局滚动 / window resize / click 立即收起，避免浮层"飘在原位"
function onGlobalDismiss() {
  if (!activeTarget) return
  clearTimers()
  hide()
}

let globalListenersAttached = false
function attachGlobalListeners() {
  if (globalListenersAttached) return
  globalListenersAttached = true
  // 捕获阶段：抓住所有滚动容器的滚动
  window.addEventListener('scroll', onGlobalDismiss, true)
  window.addEventListener('resize', onGlobalDismiss)
  window.addEventListener('mousedown', onGlobalDismiss, true)
}

function bind(el: HTMLElement, value: unknown) {
  const text = typeof value === 'string' ? value : value == null ? '' : String(value)
  if (text.trim().length === 0) {
    // 空字符串：移除监听并清理 dataset，避免触发空白浮窗
    unbind(el)
    return
  }
  el.dataset.tooltipContent = text
  if (!el.dataset.tooltipBound) {
    el.dataset.tooltipBound = '1'
    el.addEventListener('mouseenter', onEnter)
    el.addEventListener('mouseleave', onLeave)
    el.addEventListener('focus', onEnter)
    el.addEventListener('blur', onLeave)
  }
}

function unbind(el: HTMLElement) {
  if (el.dataset.tooltipBound) {
    el.removeEventListener('mouseenter', onEnter)
    el.removeEventListener('mouseleave', onLeave)
    el.removeEventListener('focus', onEnter)
    el.removeEventListener('blur', onLeave)
    delete el.dataset.tooltipBound
  }
  delete el.dataset.tooltipContent
  if (activeTarget === el) {
    clearTimers()
    hide()
  }
}

export const tooltip: Directive<HTMLElement, unknown> = {
  mounted(el, binding: DirectiveBinding<unknown>) {
    attachGlobalListeners()
    bind(el, binding.value)
  },
  updated(el, binding: DirectiveBinding<unknown>) {
    // 值变化时同步 dataset；触发元素本身不需要重绑
    if (binding.value === binding.oldValue) return
    bind(el, binding.value)
    // 如果当前正在显示，热更新文本与位置
    if (activeTarget === el) {
      const text = el.dataset.tooltipContent ?? ''
      if (text.length === 0) {
        hide()
      } else {
        setText(text)
        place(el)
      }
    }
  },
  beforeUnmount(el) {
    unbind(el)
  }
}

export default tooltip
