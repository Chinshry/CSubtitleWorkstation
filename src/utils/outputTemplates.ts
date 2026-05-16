import type { AppConfig, CompressJob, OutputNameTemplate, VideoMeta } from '../types'

export const DEFAULT_OUTPUT_TEMPLATE: OutputNameTemplate = {
  id: 'default',
  name: '默认',
  pattern: '{video_name} output.mp4',
  outputDirMode: 'sameAsVideo',
  isDefault: true,
}

export const TEMPLATE_VARIABLES = [
  { key: '{date:YYYYMMDD}', label: '日期格式1', sample: '20260101' },
  { key: '{date:YYMMDD}', label: '日期格式2', sample: '260101' },
  { key: '{video_name}', label: '视频文件名', sample: 'input' },
  { key: '{resolution}', label: '分辨率', sample: '1080' },
  { key: '{encoder}', label: '编码器', sample: 'libx264' },
  { key: '{crf}', label: 'CRF', sample: '18' },
]

export function normalizeOutputTemplates(config: AppConfig | null): OutputNameTemplate[] {
  const templates = Array.isArray(config?.outputTemplates) ? config!.outputTemplates : []
  const normalized = templates.length
    ? templates
    : [{
        ...DEFAULT_OUTPUT_TEMPLATE,
        pattern: config?.outputNameTemplate || DEFAULT_OUTPUT_TEMPLATE.pattern,
      }]
  if (!normalized.some((item) => item.id === 'default')) {
    normalized.unshift(DEFAULT_OUTPUT_TEMPLATE)
  }
  return normalized.map((item, index) => ({
    ...item,
    name: item.name || `模板 ${index + 1}`,
    pattern: item.pattern || DEFAULT_OUTPUT_TEMPLATE.pattern,
    outputDirMode: item.outputDirMode || 'sameAsVideo',
    isDefault: item.isDefault || item.id === (config?.defaultOutputTemplateId ?? 'default'),
  }))
}

export function getDefaultOutputTemplate(config: AppConfig | null): OutputNameTemplate {
  const templates = normalizeOutputTemplates(config)
  const id = config?.defaultOutputTemplateId
  return templates.find((item) => item.id === id) ?? templates.find((item) => item.isDefault) ?? templates[0]
}

export function renderOutputName(
  pattern: string,
  job: CompressJob,
  meta: VideoMeta | null,
): string {
  const video = splitPath(job.videoPath)
  const resolution = meta?.height ? String(meta.height) : (job.videoHeight ? String(job.videoHeight) : '')
  const date = new Date()
  const values: Record<string, string> = {
    video_name: video.stem || 'output',
    resolution,
    encoder: job.encoder,
    crf: String(job.crf),
    date: formatDateTime(date, 'YYYYMMDD'),
  }
  let out = pattern || DEFAULT_OUTPUT_TEMPLATE.pattern
  for (const [key, value] of Object.entries(values)) {
    out = out.split(`{${key}}`).join(sanitizeFilenamePart(value))
  }
  out = out.replace(/\{date:([^}]+)\}/g, (_, fmt: string) => sanitizeFilenamePart(formatDateTime(date, fmt)))
  if (!/\.[A-Za-z0-9]{2,5}$/.test(out)) {
    out += '.mp4'
  }
  return out
}

export function buildOutputPath(
  template: OutputNameTemplate,
  job: CompressJob,
  meta: VideoMeta | null,
): string {
  const video = splitPath(job.videoPath)
  const file = renderOutputName(template.pattern, job, meta)
  let dir = video.dir
  if (template.outputDirMode === 'fixed' && template.fixedOutputDir) {
    dir = template.fixedOutputDir
  } else if (template.outputDirMode === 'manual') {
    const current = splitPath(job.outputPath)
    dir = current.dir || video.dir
  }
  return dir ? `${dir}${video.sep}${file}` : file
}

export function splitPath(path: string): { dir: string; sep: string; stem: string; ext: string } {
  const sep = path.includes('\\') ? '\\' : '/'
  const idx = Math.max(path.lastIndexOf('\\'), path.lastIndexOf('/'))
  const dir = idx >= 0 ? path.slice(0, idx) : ''
  const file = idx >= 0 ? path.slice(idx + 1) : path
  const dotIdx = file.lastIndexOf('.')
  return {
    dir,
    sep,
    stem: dotIdx > 0 ? file.slice(0, dotIdx) : file,
    ext: dotIdx > 0 ? file.slice(dotIdx) : '',
  }
}

function sanitizeFilenamePart(value: string): string {
  return value.replace(/[<>:"/\\|?*]/g, '_').trim()
}

function formatDateTime(date: Date, format: string): string {
  const YYYY = String(date.getFullYear())
  const YY = YYYY.slice(-2)
  const MM = String(date.getMonth() + 1).padStart(2, '0')
  const DD = String(date.getDate()).padStart(2, '0')
  const HH = String(date.getHours()).padStart(2, '0')
  const mm = String(date.getMinutes()).padStart(2, '0')
  const ss = String(date.getSeconds()).padStart(2, '0')
  return format
    .replace(/YYYY/g, YYYY)
    .replace(/YY/g, YY)
    .replace(/MM/g, MM)
    .replace(/DD/g, DD)
    .replace(/HH/g, HH)
    .replace(/mm/g, mm)
    .replace(/ss/g, ss)
}
