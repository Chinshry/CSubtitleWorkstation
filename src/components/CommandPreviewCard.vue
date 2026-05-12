<script setup lang="ts">
import { computed, ref } from 'vue'
import { isWindows } from '../stores/platformStore'

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
      return args.map(quotePosix).join(' ')
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
        <select v-model="quoting" class="quote-select" title="选择终端方言">
          <option value="auto">自动（按本机平台）</option>
          <option value="windows">Windows · cmd/PowerShell</option>
          <option value="posix">POSIX · bash/zsh</option>
          <option value="raw">原始（数组拼接，不转义）</option>
        </select>
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
