import { invoke } from '@tauri-apps/api/core'

export type ProofreadIssue = {
  id: string
  line: number
  start: number
  end: number
  original: string
  suggestion: string
  message: string
  reason: string
  context: string
  confidence: 'low' | 'medium' | 'high' | string
}

export type ProofreadTermRule = {
  canonical: string
  pattern: string
}

export type SavedProofreadFile = {
  outputPath: string
}

export function proofreadText(text: string, termRules: ProofreadTermRule[] = []) {
  return invoke<ProofreadIssue[]>('proofread_text', { text, termRules })
}

export function readProofreadFile(path: string) {
  return invoke<string>('read_proofread_file', { path })
}

export function saveProofreadFile(path: string, text: string, suffix: string, overwrite = false) {
  return invoke<SavedProofreadFile>('save_proofread_file', { path, text, suffix, overwrite })
}

export function saveProofreadToPath(path: string, text: string) {
  return invoke<SavedProofreadFile>('save_proofread_to_path', { path, text })
}
