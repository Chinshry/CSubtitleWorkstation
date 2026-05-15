/**
 * ASS YCbCr Matrix 与视频 color_space / color_range 一致性判定
 *
 * 背景：ASS 文件可在 [Script Info] 中声明 `YCbCr Matrix: TV.709` 等，标识字幕渲染
 * 时把 RGB 颜色转换到 YUV 时使用的色彩矩阵与范围。若该声明与视频实际矩阵不一致，
 * libass / VSFilterMod 会按 ASS 声明的矩阵执行 RGB→YUV，导致烧入字幕颜色相对视频
 * 整体偏色（Aegisub 取色对、压制出来不对）。
 *
 * 本模块为纯函数，无副作用，可单测。
 */

/** 标准矩阵类别，统一归一化用于比对 */
export type MatrixStandard = '601' | '709' | '2020' | 'FCC' | '240M' | 'unknown'

/** YUV 量化范围 */
export type RangeKind = 'limited' | 'full' | 'unknown'

/** 判定级别 */
export type CheckLevel = 'ok' | 'warn' | 'error' | 'info'

export interface ColorMatrixCheck {
  level: CheckLevel
  /** 是否需要弹 banner 给用户。ok / 平凡 info 情况下为 false */
  shouldWarn: boolean
  /** 一句话标题，已格式化，可直接渲染 */
  title: string
  /** 详细说明（可选） */
  detail?: string
  /** 修复建议文案；通常是"建议把 ASS 头部改为 YCbCr Matrix: XXX" */
  suggestion?: string
  /** 解析后的 ASS 信息（透传给 UI 详情面板用） */
  assRaw?: string
  assRange?: 'TV' | 'PC'
  assStandard?: MatrixStandard
  /** 视频侧归一化结果（透传给 UI） */
  videoStandard?: MatrixStandard
  videoRangeKind?: RangeKind
}

interface ParsedAssMatrix {
  range: 'TV' | 'PC'
  standard: MatrixStandard
  none: false
}

/** ASS YCbCr Matrix 字段解析结果（None / 缺失 / 解析失败 用不同形态表达） */
type AssMatrixParseResult =
  | ParsedAssMatrix
  | { none: true }
  | { unrecognized: true; raw: string }
  | null

/** 解析 "TV.709" / "pc.601" / "None" 等 ASS 头字段；解析失败返回 unrecognized */
export function parseAssMatrix(raw: string | undefined | null): AssMatrixParseResult {
  if (!raw) return null
  const trimmed = raw.trim()
  if (!trimmed) return null
  if (trimmed.toLowerCase() === 'none') return { none: true }

  // 标准格式 "{TV|PC}.{标识}"，大小写不敏感
  const m = /^(TV|PC)\.([0-9A-Za-z]+)$/i.exec(trimmed)
  if (!m) return { unrecognized: true, raw: trimmed }
  const range = m[1].toUpperCase() as 'TV' | 'PC'
  const std = normalizeStandard(m[2])
  if (std === 'unknown') return { unrecognized: true, raw: trimmed }
  return { range, standard: std, none: false }
}

/** 把 "709" / "601" / "2020" / "FCC" / "240M" 等归一化 */
function normalizeStandard(token: string): MatrixStandard {
  const t = token.trim().toUpperCase()
  if (t === '709') return '709'
  if (t === '601') return '601'
  if (t === '2020') return '2020'
  if (t === 'FCC') return 'FCC'
  if (t === '240M') return '240M'
  return 'unknown'
}

/** 把 ffprobe color_space 归一化到矩阵标准 */
export function normalizeVideoColorSpace(cs: string | undefined): MatrixStandard {
  if (!cs) return 'unknown'
  const v = cs.trim().toLowerCase()
  switch (v) {
    case 'bt709':
      return '709'
    case 'smpte170m':
    case 'bt470bg':
    case 'bt601':
      return '601'
    case 'bt2020nc':
    case 'bt2020c':
    case 'bt2020':
      return '2020'
    case 'fcc':
      return 'FCC'
    case 'smpte240m':
      return '240M'
    default:
      return 'unknown'
  }
}

/** ffprobe color_range 归一化到 limited/full */
export function normalizeVideoRange(range: string | undefined): RangeKind {
  if (!range) return 'unknown'
  const v = range.trim().toLowerCase()
  if (v === 'tv' || v === 'mpeg' || v === 'limited') return 'limited'
  if (v === 'pc' || v === 'jpeg' || v === 'full') return 'full'
  return 'unknown'
}

/** TV/PC → limited/full */
function assRangeToKind(r: 'TV' | 'PC'): RangeKind {
  return r === 'TV' ? 'limited' : 'full'
}

/** MatrixStandard → 人类可读 */
function describeStandard(s: MatrixStandard): string {
  switch (s) {
    case '709':
      return 'BT.709'
    case '601':
      return 'BT.601'
    case '2020':
      return 'BT.2020'
    case 'FCC':
      return 'FCC'
    case '240M':
      return 'SMPTE 240M'
    default:
      return '未知'
  }
}

/** 根据视频侧矩阵反推建议的 ASS Matrix 字符串 */
function suggestAssMatrix(videoStd: MatrixStandard, rangeKind: RangeKind): string | undefined {
  if (videoStd === 'unknown') return undefined
  const stdPart = videoStd
  const rangePart = rangeKind === 'full' ? 'PC' : 'TV' // 未知时默认 TV（更常见）
  return `${rangePart}.${stdPart}`
}

/**
 * 主判定函数。
 *
 * @param assMatrix ASS 头部 YCbCr Matrix 原始字符串（来自后端 analyze_subtitle）
 * @param videoColorSpace ffprobe `color_space`
 * @param videoColorRange ffprobe `color_range`
 * @param isAssLikeSubtitle 字幕是否是 ASS/SSA（SRT/VTT 没有此字段，跳过检测）
 */
export function checkColorMatrix(
  assMatrix: string | undefined,
  videoColorSpace: string | undefined,
  videoColorRange: string | undefined,
  isAssLikeSubtitle: boolean,
): ColorMatrixCheck {
  const videoStandard = normalizeVideoColorSpace(videoColorSpace)
  const videoRangeKind = normalizeVideoRange(videoColorRange)
  const base = {
    videoStandard,
    videoRangeKind,
    assRaw: assMatrix,
  }

  // 非 ASS/SSA 字幕：完全没有此字段概念，跳过
  if (!isAssLikeSubtitle) {
    return { ...base, level: 'info', shouldWarn: false, title: '' }
  }

  const parsed = parseAssMatrix(assMatrix)

  // 1) ASS 未声明 Matrix
  if (parsed === null) {
    // 视频明显异常的情况（HDR / FullRange）下，libass 启发式不一定对，提醒
    if (videoStandard === '2020') {
      return {
        ...base,
        level: 'warn',
        shouldWarn: true,
        title: 'ASS 未声明 YCbCr Matrix，但视频是 BT.2020（HDR/4K）',
        detail: 'libass 默认按分辨率启发式选择矩阵（PlayResY≥720→BT.709），在 BT.2020 视频上烧入字幕颜色会偏。',
        suggestion: '建议在 ASS [Script Info] 段加入：YCbCr Matrix: TV.2020',
      }
    }
    if (videoRangeKind === 'full') {
      return {
        ...base,
        level: 'warn',
        shouldWarn: true,
        title: 'ASS 未声明 YCbCr Matrix，但视频是 full range',
        detail: 'libass 默认按 limited range 渲染字幕，烧到 full range 视频上黑色会发灰或白色过曝。',
        suggestion: `建议在 ASS [Script Info] 段加入：YCbCr Matrix: ${
          suggestAssMatrix(videoStandard === 'unknown' ? '709' : videoStandard, 'full') ?? 'PC.709'
        }`,
      }
    }
    return { ...base, level: 'info', shouldWarn: false, title: '' }
  }

  // 2) ASS 显式声明 None（罕见，表示直接按 RGB 处理不转换）
  if ('none' in parsed && parsed.none) {
    return {
      ...base,
      level: 'info',
      shouldWarn: false,
      title: 'ASS YCbCr Matrix: None',
      detail: '字幕作者显式跳过 RGB→YUV 转换。除非你清楚意图，否则一般不需要这样设置。',
    }
  }

  // 3) ASS 值无法识别（拼写错误等）
  if ('unrecognized' in parsed) {
    return {
      ...base,
      level: 'warn',
      shouldWarn: true,
      title: `ASS YCbCr Matrix 值无法识别：${parsed.raw}`,
      detail: '标准取值为 TV.601 / TV.709 / TV.2020 / PC.601 / PC.709 / PC.2020 / None',
      suggestion: videoStandard !== 'unknown' ? `建议改为 ${suggestAssMatrix(videoStandard, videoRangeKind)}` : undefined,
    }
  }

  // 4) ASS 解析成功，开始与视频比对
  const assStandard = parsed.standard
  const assRange = parsed.range
  const assRangeKind = assRangeToKind(assRange)

  const matrixMismatch = videoStandard !== 'unknown' && assStandard !== videoStandard
  const rangeMismatch =
    videoRangeKind !== 'unknown' && assRangeKind !== videoRangeKind

  if (matrixMismatch) {
    const suggested = suggestAssMatrix(videoStandard, videoRangeKind === 'unknown' ? assRangeKind : videoRangeKind)
    return {
      ...base,
      level: 'error',
      shouldWarn: true,
      assRange,
      assStandard,
      title: `ASS 矩阵(${describeStandard(assStandard)}) 与视频(${describeStandard(videoStandard)}) 不匹配`,
      detail:
        '烧入字幕颜色会整体偏色（红/蓝偏移），这是 libass / VSFilterMod 按 ASS 声明的矩阵执行 RGB→YUV 导致的。',
      suggestion: suggested ? `建议把 ASS 头部改为：YCbCr Matrix: ${suggested}` : undefined,
    }
  }

  if (rangeMismatch) {
    const suggested = suggestAssMatrix(assStandard, videoRangeKind)
    return {
      ...base,
      level: 'warn',
      shouldWarn: true,
      assRange,
      assStandard,
      title: `ASS 量化范围(${assRange}) 与视频(${videoRangeKind === 'full' ? 'PC/full' : 'TV/limited'}) 不匹配`,
      detail: '矩阵一致但量化范围不同；黑/白电平会偏，常见表现为字幕黑色发灰或白色过曝。',
      suggestion: suggested ? `建议把 ASS 头部改为：YCbCr Matrix: ${suggested}` : undefined,
    }
  }

  // 5) 视频侧字段缺失，无法严格比对
  if (videoStandard === 'unknown') {
    return {
      ...base,
      level: 'info',
      shouldWarn: false,
      assRange,
      assStandard,
      title: '视频未声明 color_space，无法严格比对',
    }
  }

  return {
    ...base,
    level: 'ok',
    shouldWarn: false,
    assRange,
    assStandard,
    title: `色彩矩阵一致（${assRange}.${assStandard}）`,
  }
}
