export type RuleDictionaryEntry = {
  target: string
  pattern: string
}

export type EditableRuleDictionaryEntry = RuleDictionaryEntry & {
  lineIndex: number
  valid: boolean
  patternValid: boolean
}

export function parseRuleDictionary(text: string): RuleDictionaryEntry[] {
  return text
    .replace(/\r\n/g, '\n')
    .split('\n')
    .map((line) => line.trim())
    .filter((line) => line && !line.startsWith('#'))
    .map(parseRuleDictionaryLine)
    .filter((rule): rule is RuleDictionaryEntry => Boolean(rule))
}

export function parseRuleDictionaryLine(line: string): RuleDictionaryEntry | null {
  const tableMatch = line.match(/^\["(.*?)"\]\s*=\s*"(.*?)"\s*,?$/)
  if (tableMatch) {
    return { target: tableMatch[1].trim(), pattern: tableMatch[2].trim() }
  }

  const separators = ['->', '=>', '=', '\t']
  for (const separator of separators) {
    const index = line.indexOf(separator)
    if (index <= 0) continue
    const target = stripOptionalQuotes(line.slice(0, index).trim())
    const pattern = stripOptionalQuotes(line.slice(index + separator.length).trim().replace(/,$/, ''))
    if (target && pattern) return { target, pattern }
  }
  return null
}

export function serializeRuleDictionaryEntry(target: string, pattern: string) {
  return `[${JSON.stringify(target.trim())}] = ${JSON.stringify(pattern.trim())}`
}

export function buildEditableRuleDictionaryEntries(
  text: string,
  options: { validatePattern?: boolean } = {}
): EditableRuleDictionaryEntry[] {
  return getRuleDictionaryLines(text)
    .map((line, lineIndex) => ({ line, lineIndex }))
    .filter(({ line }) => line.trim() && !line.trim().startsWith('#'))
    .map(({ line, lineIndex }) => {
      const rule = parseRuleDictionaryLine(line.trim())
      return {
        lineIndex,
        target: rule?.target ?? '',
        pattern: rule?.pattern ?? line.trim(),
        valid: Boolean(rule),
        patternValid: !options.validatePattern || (rule ? isValidRegexPattern(rule.pattern) : false)
      }
    })
}

export function getRuleDictionaryLines(text: string) {
  return text.replace(/\r\n/g, '\n').split('\n')
}

export function setRuleDictionaryEntry(
  text: string,
  lineIndex: number,
  field: 'target' | 'pattern',
  value: string
) {
  const lines = getRuleDictionaryLines(text)
  const current = parseRuleDictionaryLine(lines[lineIndex]?.trim() ?? '')
  const target = field === 'target' ? value : current?.target ?? ''
  const pattern = field === 'pattern' ? value : current?.pattern ?? ''
  lines[lineIndex] = target.trim() || pattern.trim()
    ? serializeRuleDictionaryEntry(target, pattern)
    : ''
  return lines.join('\n')
}

export function addRuleDictionaryEntry(text: string) {
  const lines = getRuleDictionaryLines(text)
  if (lines.length === 1 && !lines[0]) {
    lines[0] = serializeRuleDictionaryEntry('', '')
  } else {
    lines.push(serializeRuleDictionaryEntry('', ''))
  }
  return lines.join('\n')
}

export function removeRuleDictionaryEntry(text: string, lineIndex: number) {
  const lines = getRuleDictionaryLines(text)
  lines.splice(lineIndex, 1)
  return lines.join('\n')
}

export function isValidRegexPattern(pattern: string) {
  if (!pattern.trim()) return false
  try {
    const normalized = pattern.replace(/^\(\?i\)/, '')
    const flags = pattern.startsWith('(?i)') ? 'i' : ''
    new RegExp(normalized, flags)
    return true
  } catch {
    return false
  }
}

export function buildRegexFromPattern(pattern: string) {
  try {
    const flags = pattern.startsWith('(?i)') ? 'giu' : 'gu'
    const normalized = pattern.replace(/^\(\?i\)/, '')
    return new RegExp(normalized, flags)
  } catch {
    return null
  }
}

export function applyCapturePlaceholders(target: string, captures: RegExpMatchArray) {
  let next = target
  for (let index = 1; index < captures.length; index++) {
    next = next.replace(new RegExp(`%${index}`, 'g'), captures[index] ?? '')
  }
  return next
}

function stripOptionalQuotes(value: string) {
  const bracketMatch = value.match(/^\["(.+)"\]$/)
  if (bracketMatch) return bracketMatch[1]
  if (
    (value.startsWith('"') && value.endsWith('"')) ||
    (value.startsWith("'") && value.endsWith("'"))
  ) {
    return value.slice(1, -1)
  }
  return value
}
