<script setup lang="ts">
import { computed, ref } from 'vue'
import {
  addRuleDictionaryEntry,
  applyCapturePlaceholders,
  buildRegexFromPattern,
  buildEditableRuleDictionaryEntries,
  parseRuleDictionary,
  removeRuleDictionaryEntry,
  setRuleDictionaryEntry
} from '../utils/ruleDictionary'

type PreviewMatch = {
  ruleLabel: string
  original: string
  suggestion: string
  before: string
  after: string
}

const props = withDefaults(defineProps<{
  open: boolean
  modelValue: string
  title: string
  description: string
  targetLabel: string
  patternLabel: string
  targetPlaceholder: string
  patternPlaceholder: string
  rawPlaceholder: string
  ariaLabel: string
  validatePattern?: boolean
  supportsCapture?: boolean
}>(), {
  validatePattern: true,
  supportsCapture: true
})

const emit = defineEmits<{
  'update:open': [value: boolean]
  'update:modelValue': [value: string]
}>()

const mode = ref<'table' | 'raw'>('table')
const testText = ref('')

const rules = computed(() => parseRuleDictionary(props.modelValue))
const entries = computed(() => buildEditableRuleDictionaryEntries(props.modelValue, {
  validatePattern: props.validatePattern
}))
const invalidCount = computed(() => entries.value.filter((entry) => !entry.valid || !entry.patternValid).length)
const previewMatches = computed<PreviewMatch[]>(() => {
  if (!testText.value.trim()) return []

  const matches: PreviewMatch[] = []
  for (const rule of rules.value) {
    const regex = props.validatePattern
      ? buildRegexFromPattern(rule.pattern)
      : buildRegexFromPattern(escapeRegExp(rule.pattern))
    if (!regex) continue

    for (const match of testText.value.matchAll(regex)) {
      const original = match[0]
      if (!original) continue

      const suggestion = props.supportsCapture
        ? applyCapturePlaceholders(rule.target, match)
        : rule.target
      if (original === suggestion) continue

      const index = match.index ?? 0
      matches.push({
        ruleLabel: rule.target,
        original,
        suggestion,
        before: testText.value.slice(Math.max(0, index - 12), index),
        after: testText.value.slice(index + original.length, index + original.length + 12)
      })
      if (matches.length >= 8) return matches
    }
  }

  return matches
})

function escapeRegExp(value: string) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}

function setOpen(value: boolean) {
  emit('update:open', value)
}

function updateEntry(lineIndex: number, field: 'target' | 'pattern', value: string) {
  emit('update:modelValue', setRuleDictionaryEntry(props.modelValue, lineIndex, field, value))
}

function addEntry() {
  emit('update:modelValue', addRuleDictionaryEntry(props.modelValue))
}

function removeEntry(lineIndex: number) {
  emit('update:modelValue', removeRuleDictionaryEntry(props.modelValue, lineIndex))
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="open"
      class="rule-dictionary-modal app-modal-active"
      role="presentation"
      @click.self="setOpen(false)"
    >
      <section class="rule-dictionary-dialog" role="dialog" aria-modal="true" :aria-label="ariaLabel">
        <div class="rule-dictionary-dialog-head">
          <div>
            <h2>{{ title }}</h2>
            <p>{{ description }}</p>
          </div>
          <button type="button" class="field-tool" @click="setOpen(false)">关闭</button>
        </div>

        <div class="rule-dictionary-toolbar">
          <div class="rule-dictionary-tabs" role="tablist" aria-label="词库编辑方式">
            <button type="button" :class="{ active: mode === 'table' }" @click="mode = 'table'">
              词条编辑
            </button>
            <button type="button" :class="{ active: mode === 'raw' }" @click="mode = 'raw'">
              原始文本
            </button>
          </div>
          <button type="button" class="field-tool primary" @click="addEntry">新增</button>
        </div>

        <div v-if="mode === 'table'" class="rule-entry-editor">
          <div class="rule-entry-head">
            <span>{{ targetLabel }}</span>
            <span>{{ patternLabel }}</span>
            <span></span>
          </div>
          <div v-if="entries.length" class="rule-entry-list">
            <div
              v-for="entry in entries"
              :key="entry.lineIndex"
              class="rule-entry-row"
              :class="{ invalid: !entry.valid || !entry.patternValid }"
            >
              <label>
                <span>{{ targetLabel }}</span>
                <input
                  :value="entry.target"
                  :placeholder="targetPlaceholder"
                  @input="updateEntry(entry.lineIndex, 'target', ($event.target as HTMLInputElement).value)"
                />
              </label>
              <label>
                <span>{{ patternLabel }}</span>
                <input
                  :value="entry.pattern"
                  :placeholder="patternPlaceholder"
                  @input="updateEntry(entry.lineIndex, 'pattern', ($event.target as HTMLInputElement).value)"
                />
              </label>
              <button type="button" class="field-tool" @click="removeEntry(entry.lineIndex)">删除</button>
              <small v-if="!entry.valid">这一行格式无法识别，可切到原始文本检查。</small>
              <small v-else-if="!entry.patternValid">匹配规则看起来不是有效正则。</small>
            </div>
          </div>
          <button v-else type="button" class="rule-entry-empty" @click="addEntry">新增第一条规则</button>
        </div>

        <div v-if="mode === 'table'" class="rule-preview">
          <label>
            <span>试匹配</span>
            <input
              v-model="testText"
              placeholder="输入一小段字幕文本，检查上面的规则会不会命中"
            />
          </label>
          <div v-if="testText" class="rule-preview-result">
            <div v-if="previewMatches.length" class="rule-preview-list">
              <div v-for="(match, index) in previewMatches" :key="`${match.ruleLabel}-${index}`" class="rule-preview-item">
                <strong>{{ match.ruleLabel }}</strong>
                <span class="rule-preview-context">
                  <span>{{ match.before }}</span>
                  <mark class="original">{{ match.original }}</mark>
                  <span>{{ match.after }}</span>
                </span>
                <span class="rule-preview-replace">{{ match.original }} → {{ match.suggestion }}</span>
              </div>
            </div>
            <span v-else class="rule-preview-empty">当前没有匹配到任何词条。</span>
          </div>
        </div>

        <textarea
          v-else
          :value="modelValue"
          spellcheck="false"
          :placeholder="rawPlaceholder"
          @input="emit('update:modelValue', ($event.target as HTMLTextAreaElement).value)"
        ></textarea>

        <div class="rule-dictionary-dialog-foot">
          <span>
            {{ rules.length }} 条可用规则<span v-if="invalidCount">，{{ invalidCount }} 条需要检查</span>。修改后会自动记忆并重新处理<span v-if="supportsCapture">；目标文本支持 %1 捕获组</span>。
          </span>
          <button type="button" class="field-tool primary" @click="setOpen(false)">完成</button>
        </div>
      </section>
    </div>
  </Teleport>
</template>

<style>
.rule-dictionary-modal {
  align-items: center;
  background: rgba(15, 23, 32, 0.38);
  display: flex;
  inset: 0;
  justify-content: center;
  padding: 28px;
  position: fixed;
  z-index: 120;
}

.rule-dictionary-dialog {
  background: #fff;
  border: 1px solid #d8e2e8;
  border-radius: 8px;
  box-shadow: 0 22px 64px rgba(15, 23, 32, 0.22);
  color: #24313c;
  display: grid;
  gap: 12px;
  max-height: min(780px, calc(100vh - 56px));
  max-width: min(900px, 100%);
  overflow: auto;
  padding: 16px;
  width: 900px;
}

.rule-dictionary-dialog-head,
.rule-dictionary-dialog-foot,
.rule-dictionary-toolbar {
  align-items: center;
  display: flex;
  gap: 12px;
  justify-content: space-between;
}

.rule-dictionary-dialog-head h2 {
  color: #102030;
  font-size: 17px;
  margin: 0;
}

.rule-dictionary-dialog-head p,
.rule-dictionary-dialog-foot span {
  color: #667582;
  font-size: 13px;
  line-height: 1.55;
  margin: 5px 0 0;
}

.rule-dictionary-tabs {
  background: #eef3f6;
  border: 1px solid #d8e2e8;
  border-radius: 8px;
  display: flex;
  padding: 3px;
}

.rule-dictionary-tabs button {
  background: transparent;
  color: #4d5c68;
  font-size: 13px;
  min-height: 30px;
  padding: 0 12px;
}

.rule-dictionary-tabs button.active {
  background: #fff;
  box-shadow: 0 1px 4px rgba(15, 23, 32, 0.12);
  color: #102030;
}

.rule-entry-editor {
  background: #f7f9fb;
  border: 1px solid #d8e2e8;
  border-radius: 8px;
  display: grid;
  gap: 8px;
  max-height: min(360px, 44vh);
  overflow: auto;
  padding: 10px;
}

.rule-entry-head,
.rule-entry-row {
  display: grid;
  gap: 8px;
  grid-template-columns: minmax(150px, 0.9fr) minmax(220px, 1.4fr) 68px;
}

.rule-entry-head {
  color: #667582;
  font-size: 12px;
  font-weight: 700;
  padding: 0 8px;
}

.rule-entry-list {
  display: grid;
  gap: 8px;
}

.rule-entry-row {
  align-items: center;
  background: #fff;
  border: 1px solid #e2e8ee;
  border-radius: 6px;
  padding: 8px;
}

.rule-entry-row.invalid {
  border-color: #ef9a92;
  box-shadow: inset 3px 0 0 #d04437;
}

.rule-entry-row label {
  display: grid;
  gap: 4px;
  min-width: 0;
}

.rule-entry-row label span {
  color: #667582;
  display: none;
  font-size: 12px;
}

.rule-entry-row input,
.rule-preview input {
  background: #fbfcfd;
  border: 1px solid #d8e2e8;
  border-radius: 6px;
  color: #18202a;
  font: 13px/1.5 "Microsoft YaHei", "Segoe UI", sans-serif;
  min-height: 34px;
  min-width: 0;
  outline: none;
  padding: 0 10px;
  width: 100%;
}

.rule-entry-row input:focus,
.rule-preview input:focus,
.rule-dictionary-dialog textarea:focus {
  border-color: #176b87;
  box-shadow: 0 0 0 3px rgba(23, 107, 135, 0.12);
}

.rule-entry-row small {
  color: #b42318;
  font-size: 12px;
  grid-column: 1 / -1;
}

.rule-entry-empty {
  align-self: center;
  background: #fff;
  border: 1px dashed #b9c8d2;
  color: #176b87;
  justify-self: center;
  min-height: 40px;
  padding: 0 16px;
}

.rule-preview {
  background: #fbfcfd;
  border: 1px solid #d8e2e8;
  border-radius: 8px;
  display: grid;
  gap: 10px;
  padding: 10px;
}

.rule-preview label {
  display: grid;
  gap: 6px;
}

.rule-preview label span {
  color: #667582;
  font-size: 12px;
  font-weight: 700;
}

.rule-preview-result {
  color: #4d5c68;
  font-size: 12px;
}

.rule-preview-list {
  display: grid;
  gap: 6px;
}

.rule-preview-item {
  align-items: center;
  background: #fff;
  border: 1px solid #e2e8ee;
  border-radius: 6px;
  display: grid;
  gap: 6px;
  grid-template-columns: minmax(100px, 0.7fr) minmax(180px, 1.4fr) minmax(120px, 0.9fr);
  padding: 8px;
}

.rule-preview-item strong {
  color: #102030;
}

.rule-preview-context {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.rule-preview-context mark {
  background: rgba(23, 107, 135, 0.14);
  border-radius: 4px;
  color: #102030;
  padding: 1px 3px;
}

.rule-preview-replace,
.rule-preview-empty {
  color: #667582;
}

.rule-dictionary-dialog textarea {
  background: #fbfcfd;
  border: 1px solid #d8e2e8;
  border-radius: 8px;
  color: #18202a;
  font: 14px/1.7 "Microsoft YaHei", "Segoe UI", sans-serif;
  min-height: 320px;
  outline: none;
  padding: 12px;
  resize: none;
  width: 100%;
}

@media (max-width: 760px) {
  .rule-dictionary-modal {
    align-items: stretch;
    padding: 12px;
  }

  .rule-dictionary-dialog {
    max-height: calc(100vh - 24px);
  }

  .rule-dictionary-toolbar,
  .rule-dictionary-dialog-head,
  .rule-dictionary-dialog-foot {
    align-items: stretch;
    flex-direction: column;
  }

  .rule-entry-head {
    display: none;
  }

  .rule-entry-row {
    grid-template-columns: 1fr;
  }

  .rule-entry-row label span {
    display: inline;
  }
}
</style>
