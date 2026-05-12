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
