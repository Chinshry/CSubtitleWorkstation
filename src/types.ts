export type FfmpegStatus = {
  available: boolean
  source: 'system_path' | 'custom_path' | 'not_found'
  ffmpegPath?: string
  ffmpegVersion?: string
  ffprobePath?: string
  ffprobeVersion?: string
  message?: string
}

export type AvsStatus = {
  supportedPlatform: boolean
  ffmpegDemuxerAvailable: boolean
  avisynthInstalled: boolean
  avisynthVersion?: string
  avisynthInstallPath?: string
  avisynthDllPath?: string
  available: boolean
  message?: string
}

export type AppConfig = {
  ffmpegMode: 'system' | 'custom'
  ffmpegPath?: string
  defaultCrf: number
  defaultNeedLogo: boolean
  defaultNeedYadif: boolean
  defaultEncoder: string
  outputNameTemplate: string
  checkUpdateOnStartup: boolean
  defaultLogoDir?: string
  defaultUseAvs?: boolean
  recentLogos?: RecentLogo[]
  /** 未命中分辨率桶时的全局 fallback 布局；保留作向后兼容 */
  lastLogoLayout?: LogoLayout | null
  /** 按 (分辨率桶, LOGO 路径) 区分的布局记忆 */
  logoLayouts?: LogoLayoutEntry[]
}

export type RecentLogo = {
  path: string
  lastUsedAt: number
  /** 用户自定义的展示昵称；为空/不存在时回退到 path 的文件名 */
  displayName?: string
}

export type LogoLayout = {
  path: string
  xPct: number
  yPct: number
  wPct: number
  hPct: number
}

/** 按 (分辨率桶, LOGO 路径) 维度独立记忆的 LOGO 布局条目 */
export type LogoLayoutEntry = {
  /** 分辨率桶 key：720p-landscape / 720p-portrait / 1080p-landscape / 1080p-portrait / 4k-landscape / 4k-portrait */
  bucket: string
  path: string
  xPct: number
  yPct: number
  wPct: number
  hPct: number
  lastUsedAt: number
}

export type CompressJob = {
  id: string
  videoPath: string
  subtitlePath: string
  outputPath: string
  crf: number
  maxBitrate?: number
  needLogo: boolean
  needYadif: boolean
  encoder: 'libx264' | 'h264_nvenc' | 'h264_amf' | 'h264_videotoolbox'
  logoDir?: string
  useAvs?: boolean
  logoLayout?: LogoLayout | null
  /** LOGO 是否叠加在字幕之上。false=在字幕下（默认，非 AVS 现状）；true=在字幕上。
   *  AVS 模式当前固定为在字幕上，前端会禁用切换并提示。 */
  logoOnTop?: boolean
  /** 前端 inspectVideoMeta 解析出的"显示尺寸"（已应用 rotation），由后端用于 LOGO overlay 像素换算 */
  videoWidth?: number
  videoHeight?: number
}

export type CompressStatus = {
  jobId: string
  statusLine: string
  percent?: number
  currentSeconds?: number
  durationSeconds?: number
  sizeKb?: number
  bitrateKbps?: number
  speed?: number
  fps?: number
}

export type VideoMeta = {
  fileSizeBytes?: number
  createdAt?: string
  durationSeconds?: number
  durationText?: string
  startSeconds?: number
  overallBitrateKbps?: number
  format?: string
  width?: number
  height?: number
  sar?: string
  dar?: string
  videoCodec?: string
  videoProfile?: string
  pixelFormat?: string
  colorRange?: string
  colorSpace?: string
  fps?: number
  tbr?: number
  videoBitrateKbps?: number
  totalFrames?: number
  frameRateMode?: 'CFR' | 'VFR' | string
  audioCodec?: string
  audioProfile?: string
  audioSampleRate?: number
  audioChannels?: string
  audioBitrateKbps?: number
}
