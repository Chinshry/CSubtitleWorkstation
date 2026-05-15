import { reactive } from 'vue'

export type ToastType = 'success' | 'error' | 'info' | 'warning'

export interface ToastItem {
  id: number
  message: string
  type: ToastType
}

interface ToastState {
  items: ToastItem[]
}

const state = reactive<ToastState>({ items: [] })
const timers = new Map<number, ReturnType<typeof setTimeout>>()
let nextId = 1

const DEFAULT_DURATION = 5000

function dismiss(id: number) {
  const timer = timers.get(id)
  if (timer) {
    clearTimeout(timer)
    timers.delete(id)
  }
  const idx = state.items.findIndex((item) => item.id === id)
  if (idx >= 0) state.items.splice(idx, 1)
}

function show(message: string, type: ToastType = 'info', duration = DEFAULT_DURATION) {
  const id = nextId++
  state.items.push({ id, message, type })
  if (duration > 0) {
    const timer = setTimeout(() => dismiss(id), duration)
    timers.set(id, timer)
  }
  return id
}

export function useToast() {
  return {
    items: state.items,
    show,
    success: (message: string, duration?: number) => show(message, 'success', duration),
    error: (message: string, duration?: number) => show(message, 'error', duration),
    info: (message: string, duration?: number) => show(message, 'info', duration),
    warning: (message: string, duration?: number) => show(message, 'warning', duration),
    dismiss,
  }
}
