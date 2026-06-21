import { computed, ref } from 'vue'
import { checkAppUpdate } from '../api/updater'
import type { AppUpdateInfo } from '../types'
import { t } from '../i18n'

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
    updateMessage.value = t('update.message.connecting')
  }

  try {
    const info = await checkAppUpdate()
    updateInfo.value = info
    updateState.value = info.available ? 'success' : 'idle'
    if (info.available) {
      updateMessage.value = t('update.message.available', { version: info.latestVersion })
    } else if (!options.silent) {
      updateMessage.value = t('update.message.latest')
    }
    return info
  } catch (err) {
    updateState.value = 'error'
    if (!options.silent) {
      updateMessage.value = formatUpdateError(err)
    }
    return null
  } finally {
    updateChecking.value = false
  }
}

function formatUpdateError(err: unknown) {
  const raw = err instanceof Error ? err.message : String(err)
  const message = raw.trim()
  if (/failed to fetch|networkerror|load failed/i.test(message)) {
    return t('update.error.manifestUnavailable')
  }
  return t('update.error.generic', { message })
}
