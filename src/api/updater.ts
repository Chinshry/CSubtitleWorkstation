import { invoke } from '@tauri-apps/api/core'
import type { AppUpdateInfo } from '../types'

const UPDATE_MANIFEST_URL = 'https://chinshry.github.io/CSubtitleWorkstation/updates/latest.json'

type UpdateManifest = {
  version?: string
  notes?: string
  pub_date?: string
  platforms?: Record<string, { url?: string }>
}

export function getCurrentAppVersion() {
  return invoke<string>('get_current_app_version')
}

export async function checkAppUpdate(): Promise<AppUpdateInfo> {
  const currentVersion = await getCurrentAppVersion()
  const response = await fetch(UPDATE_MANIFEST_URL, { cache: 'no-store' })

  if (!response.ok) {
    throw new Error(`更新服务器返回 ${response.status}`)
  }

  const manifest = (await response.json()) as UpdateManifest
  if (!manifest.version) {
    throw new Error('更新清单缺少 version 字段')
  }

  const platform = manifest.platforms?.['windows-x86_64'] ?? Object.values(manifest.platforms ?? {})[0]
  return {
    available: compareVersions(manifest.version, currentVersion) > 0,
    currentVersion,
    latestVersion: manifest.version,
    notes: manifest.notes,
    pubDate: manifest.pub_date,
    downloadUrl: platform?.url
  }
}

function compareVersions(left: string, right: string) {
  const a = normalizeVersion(left)
  const b = normalizeVersion(right)
  const length = Math.max(a.length, b.length)

  for (let i = 0; i < length; i += 1) {
    const diff = (a[i] ?? 0) - (b[i] ?? 0)
    if (diff !== 0) return diff
  }
  return 0
}

function normalizeVersion(version: string) {
  return version
    .replace(/^v/i, '')
    .split(/[.-]/)
    .map((part) => Number.parseInt(part, 10))
    .map((part) => (Number.isFinite(part) ? part : 0))
}
