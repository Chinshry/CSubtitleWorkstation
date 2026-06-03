import { createApp } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import App from './App.vue'
import { tooltip } from './directives/tooltip'
import './styles.css'

createApp(App).directive('tooltip', tooltip).mount('#app')

async function showMainWindowWhenReady() {
  if (!('__TAURI_INTERNALS__' in window)) return

  await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()))

  try {
    await getCurrentWindow().show()
  } catch {
    /* keep browser/dev fallback silent */
  }
}

void showMainWindowWhenReady()
