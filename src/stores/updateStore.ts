import { computed, ref } from 'vue'
import { checkAppUpdate } from '../api/updater'
import type { AppUpdateInfo } from '../types'

export type UpdateState = 'idle' | 'success' | 'error' | 'progress'

export const updateInfo = ref<AppUpdateInfo | null>(null)
export const updateMessage = ref('')
export const updateState = ref<UpdateState>('idle')
export const updateChecking = ref(false)

export const availableUpdateVersion = computed(() => updateInfo.value?.latestVersion ?? '')
export const hasAvailableUpdate = computed(() => !!updateInfo.value?.available)
export const updateReleaseUrl = computed(() => {
  const version = updateInfo.value?.latestVersion
  return version
    ? `https://github.com/Chinshry/CSubtitleWorkstation/releases/tag/v${version}`
    : 'https://github.com/Chinshry/CSubtitleWorkstation/releases/latest'
})

export async function refreshAppUpdate(options: { silent?: boolean } = {}) {
  if (updateChecking.value) return updateInfo.value
  updateChecking.value = true
  updateState.value = 'progress'
  if (!options.silent) {
    updateMessage.value = '正在连接更新服务器...'
  }

  try {
    const info = await checkAppUpdate()
    updateInfo.value = info
    updateState.value = info.available ? 'success' : 'idle'
    if (info.available) {
      updateMessage.value = `发现新版本：${info.latestVersion}`
    } else if (!options.silent) {
      updateMessage.value = '当前已是最新版本'
    }
    return info
  } catch (err) {
    updateState.value = 'error'
    if (!options.silent) {
      updateMessage.value = formatUpdateError(err, '检查')
    }
    return null
  } finally {
    updateChecking.value = false
  }
}

function formatUpdateError(err: unknown, action: '检查') {
  const raw = err instanceof Error ? err.message : String(err)
  const message = raw.replace(/^检查更新失败[:：]?\s*/u, '')
  if (/failed to fetch|networkerror|load failed/i.test(message)) {
    return `更新${action}失败：无法访问更新清单，请确认 GitHub Pages 已启用，且 docs/updates/latest.json 已提交并推送到远程仓库。`
  }
  return `更新${action}失败：${message}`
}
