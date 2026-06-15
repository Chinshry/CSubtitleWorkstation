import { loadConfig, saveConfig } from '../api/config'
import type { AvsStatus, EncoderInfo, EnvironmentCache, FfmpegStatus, LavFiltersStatus } from '../types'

function normalizeCache(cache: EnvironmentCache | undefined): EnvironmentCache {
  return cache ?? {}
}

async function readEnvironmentCache(): Promise<EnvironmentCache> {
  const config = await loadConfig()
  return normalizeCache(config.environmentCache)
}

async function updateEnvironmentCache(mutator: (cache: EnvironmentCache) => EnvironmentCache) {
  const config = await loadConfig()
  const nextCache = {
    ...mutator(normalizeCache(config.environmentCache)),
    updatedAt: Date.now()
  }
  await saveConfig({
    ...config,
    environmentCache: nextCache
  })
}

export async function readCachedFfmpegStatus() {
  return (await readEnvironmentCache()).ffmpegStatus ?? null
}

export async function writeCachedFfmpegStatus(value: FfmpegStatus | null) {
  await updateEnvironmentCache((cache) => ({
    ...cache,
    ffmpegStatus: value ?? undefined
  }))
}

export async function readCachedEncoderOptions() {
  return (await readEnvironmentCache()).encoderOptions ?? null
}

export async function writeCachedEncoderOptions(value: EncoderInfo[] | null) {
  await updateEnvironmentCache((cache) => ({
    ...cache,
    encoderOptions: value ?? undefined
  }))
}

export async function readCachedAvsStatus() {
  return (await readEnvironmentCache()).avsStatus ?? null
}

export async function writeCachedAvsStatus(value: AvsStatus | null) {
  await updateEnvironmentCache((cache) => ({
    ...cache,
    avsStatus: value ?? undefined
  }))
}

export async function readCachedLavFiltersStatus() {
  return (await readEnvironmentCache()).lavFiltersStatus ?? null
}

export async function writeCachedLavFiltersStatus(value: LavFiltersStatus | null) {
  await updateEnvironmentCache((cache) => ({
    ...cache,
    lavFiltersStatus: value ?? undefined
  }))
}

export async function clearCachedFfmpegDependentEnvironment() {
  await updateEnvironmentCache((cache) => ({
    ...cache,
    encoderOptions: undefined,
    avsStatus: undefined,
    lavFiltersStatus: undefined
  }))
}
