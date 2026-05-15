<script setup lang="ts">
import { useToast } from '../composables/useToast'

const { items, dismiss } = useToast()
</script>

<template>
  <Teleport to="body">
    <div class="toast-stack" role="region" aria-live="polite" aria-label="通知">
      <transition-group name="toast-fade" tag="div" class="toast-stack-inner">
        <div
          v-for="item in items"
          :key="item.id"
          class="toast-item"
          :class="`toast-${item.type}`"
          role="status"
          @click="dismiss(item.id)"
        >
          <span class="toast-icon" aria-hidden="true">
            <svg v-if="item.type === 'success'" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="4 12 10 18 20 6" />
            </svg>
            <svg v-else-if="item.type === 'error'" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="9" />
              <path d="M8 8l8 8M16 8l-8 8" />
            </svg>
            <svg v-else-if="item.type === 'warning'" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round">
              <path d="M12 3 2 21h20L12 3z" />
              <path d="M12 10v5" />
              <circle cx="12" cy="18" r="0.6" fill="currentColor" stroke="none" />
            </svg>
            <svg v-else viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="9" />
              <path d="M12 8v0M12 11v6" />
            </svg>
          </span>
          <span class="toast-message">{{ item.message }}</span>
        </div>
      </transition-group>
    </div>
  </Teleport>
</template>

<style scoped>
.toast-stack {
  position: fixed;
  top: 64px;
  right: 40px;
  z-index: 9999;
  pointer-events: none;
}
.toast-stack-inner {
  display: flex;
  flex-direction: column;
  gap: 8px;
  align-items: flex-end;
}
.toast-item {
  align-items: center;
  background: #ffffff;
  border: 1px solid #d6e0e6;
  border-radius: 8px;
  box-shadow: 0 6px 18px rgba(15, 32, 45, 0.12), 0 1px 3px rgba(15, 32, 45, 0.08);
  color: #18202a;
  cursor: pointer;
  display: inline-flex;
  font-size: 13px;
  font-weight: 500;
  gap: 8px;
  max-width: 380px;
  min-width: 200px;
  padding: 10px 14px;
  pointer-events: auto;
  user-select: none;
}
.toast-icon {
  align-items: center;
  display: inline-flex;
  flex: 0 0 auto;
  justify-content: center;
}
.toast-message {
  flex: 1 1 auto;
  line-height: 1.4;
  word-break: break-word;
}
.toast-success {
  background: #f0faf4;
  border-color: #a8d8b9;
  color: #1a6b3a;
}
.toast-error {
  background: #fef2f2;
  border-color: #f3b3b3;
  color: #b91c1c;
}
.toast-warning {
  background: #fff8eb;
  border-color: #f0c97a;
  color: #92520a;
}
.toast-info {
  background: #eff6fb;
  border-color: #a8c8d2;
  color: #0f5268;
}
.toast-fade-enter-active,
.toast-fade-leave-active {
  transition: opacity 0.22s ease, transform 0.22s ease;
}
.toast-fade-enter-from {
  opacity: 0;
  transform: translateX(12px);
}
.toast-fade-leave-to {
  opacity: 0;
  transform: translateX(12px);
}
.toast-fade-move {
  transition: transform 0.22s ease;
}
</style>
