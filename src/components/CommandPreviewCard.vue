<script setup lang="ts">
import { computed, ref } from 'vue'
import { isWindows } from '../stores/platformStore'
import AppSelect from './AppSelect.vue'

const props = defineProps<{
  command: string[]
  stale?: boolean
}>()

const copyHint = ref('')

type Quoting = 'auto' | 'posix' | 'windows' | 'raw'

// 默认按当前平台选 shell 方言；用户可手动切到另一种或"原始"
const quoting = ref<Quoting>('auto')

const effectiveQuoting = computed<'posix' | 'windows' | 'raw'>(() => {
  if (quoting.value === 'auto') return isWindows.value ? 'windows' : 'posix'
  return quoting.value
})

// POSIX (bash/zsh)：用单引号包裹，内部 ' → '\''
function quotePosix(arg: string): string {
  if (arg === '') return "''"
  if (/^[A-Za-z0-9_\-.,/:=@%+]+$/.test(arg)) return arg
  return `'${arg.replace(/'/g, `'\\''`)}'`
}

const STAGED_SUBTITLE_TOKEN = '__CSUBTITLE_PREVIEW_SUBTITLE__'

function parseFilterQuotedPath(filter: string, marker: string): string | null {
  const start = filter.indexOf(marker)
  if (start < 0) return null

  let i = start + marker.length
  let out = ''
  while (i < filter.length) {
    if (filter.startsWith("'\\''", i)) {
      out += "'"
      i += 4
      continue
    }

    const ch = filter[i]
    if (ch === "'") return out
    if (ch === '\\' && i + 1 < filter.length) {
      out += filter[i + 1]
      i += 2
      continue
    }
    out += ch
    i += 1
  }

  return null
}

function replaceFilterQuotedPath(filter: string, marker: string, nextPath: string): string {
  const start = filter.indexOf(marker)
  if (start < 0) return filter

  let i = start + marker.length
  while (i < filter.length) {
    if (filter.startsWith("'\\''", i)) {
      i += 4
      continue
    }
    if (filter[i] === "'") {
      return `${filter.slice(0, start + marker.length)}${nextPath}${filter.slice(i)}`
    }
    if (filter[i] === '\\' && i + 1 < filter.length) {
      i += 2
      continue
    }
    i += 1
  }

  return filter
}

function needsZshSubtitleStage(path: string): boolean {
  return /[^\x20-\x7e]/.test(path) || path.includes("'")
}

function subtitleTempName(path: string): string {
  const match = path.match(/\.([A-Za-z0-9]{1,8})$/)
  const ext = match?.[1]?.toLowerCase()
  if (ext && ['ass', 'srt', 'vtt', 'sub'].includes(ext)) return `subtitle.${ext}`
  return 'subtitle.ass'
}

function quoteZshDoubleKeepingToken(arg: string): string {
  const escaped = arg
    .replace(/\\/g, '\\\\')
    .replace(/"/g, '\\"')
    .replace(/`/g, '\\`')
    .replace(/\$/g, '\\$')
    .replace(STAGED_SUBTITLE_TOKEN, '$tmpdir')
  return `"${escaped}"`
}

function renderPosixCommand(args: string[]): string {
  const vfIndex = args.findIndex((arg) => arg === '-vf')
  const vfArg = vfIndex >= 0 ? args[vfIndex + 1] : undefined
  const subtitlePath = vfArg ? parseFilterQuotedPath(vfArg, "subtitles='") : null
  if (!vfArg || !subtitlePath || !needsZshSubtitleStage(subtitlePath)) {
    return args.map(quotePosix).join(' ')
  }

  const tempName = subtitleTempName(subtitlePath)
  const stagedVf = replaceFilterQuotedPath(
    vfArg,
    "subtitles='",
    `${STAGED_SUBTITLE_TOKEN}/${tempName}`
  )
  const commandArgs = args.map((arg, index) => {
    if (index === vfIndex + 1) return quoteZshDoubleKeepingToken(stagedVf)
    return quotePosix(arg)
  })

  return [
    '(',
    '  tmpdir=$(mktemp -d)',
    `  trap ${quotePosix('rm -rf "$tmpdir"')} EXIT`,
    `  cp ${quotePosix(subtitlePath)} "$tmpdir/${tempName}"`,
    `  ${commandArgs.join(' ')}`,
    ')'
  ].join('\n')
}

// Windows (cmd / PowerShell 公共子集)：用双引号包裹，内部 " → ""
// 反斜杠在双引号内部不需要转义（cmd 不要求；PowerShell 字面字符串里也直接传）
function quoteWindows(arg: string): string {
  if (arg === '') return '""'
  if (/^[A-Za-z0-9_\-.,:\\/=@%+]+$/.test(arg)) return arg
  return `"${arg.replace(/"/g, '""')}"`
}

const commandText = computed(() => {
  const args = props.command
  if (!args.length) return ''
  switch (effectiveQuoting.value) {
    case 'raw':
      return args.join(' ')
    case 'windows':
      return args.map(quoteWindows).join(' ')
    case 'posix':
    default:
      return renderPosixCommand(args)
  }
})

const viewHintText = computed(() => {
  switch (effectiveQuoting.value) {
    case 'raw':
      return '原始数组拼接（含空格/特殊字符路径不能直接粘到终端）'
    case 'windows':
      return '已按 Windows 规则加引号，可粘到 cmd / PowerShell / Windows Terminal'
    case 'posix':
      return '已按 POSIX 规则加引号，可粘到 bash / zsh / Linux & macOS 终端'
  }
  return ''
})

async function copyCommand() {
  if (!commandText.value.trim()) {
    copyHint.value = '无命令'
    setTimeout(() => (copyHint.value = ''), 1500)
    return
  }
  try {
    await navigator.clipboard.writeText(commandText.value)
    copyHint.value = '已复制'
  } catch {
    copyHint.value = '复制失败'
  }
  setTimeout(() => (copyHint.value = ''), 1500)
}
</script>

<template>
  <section class="panel command-preview">
    <div class="command-head">
      <div>
        <h2>命令预览</h2>
        <p>{{ viewHintText }}</p>
      </div>
      <div class="command-tools">
        <AppSelect
          v-model="quoting"
          class="quote-select"
          title="选择终端方言"
          :options="[
            { value: 'auto', label: '自动（按本机平台）' },
            { value: 'windows', label: 'Windows · cmd/PowerShell' },
            { value: 'posix', label: 'POSIX · bash/zsh' },
            { value: 'raw', label: '原始（数组拼接，不转义）' }
          ]"
        />
        <button
          class="copy-btn"
          :class="{ active: copyHint }"
          :disabled="!command.length"
          @click="copyCommand"
          data-tip="复制完整命令"
        >
          <span v-if="copyHint">{{ copyHint }}</span>
          <span v-else>复制</span>
        </button>
      </div>
    </div>
    <pre v-if="command.length" class="command" :class="{ stale }">{{ commandText }}</pre>
    <p v-else class="muted">填写视频路径后将自动生成命令。</p>
  </section>
</template>

<style scoped>
.command-tools {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}
.quote-select {
  width: auto;
  min-width: 200px;
}
</style>
