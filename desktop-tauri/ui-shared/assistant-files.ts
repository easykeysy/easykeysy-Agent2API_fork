/**
 * AI （JSON / .info）：、 use and to 。
 *
 * ── Why 1 ──────────────────────────────────────
 * 、、 will and need use 1 「、
 * 、file」。 in will 「 by new 1 、
 * by will 」 this —— to `attachment:1` to will 。
 *
 * ── 「 1 use 」 ──────────────────
 * use in 1 can file， after use use 1 。
 * 1 not file when ， old can （ will ），
 * 1 up that as —— and `attachment:N` 1 。
 *
 * ── to not is ──────────────────────────────────────
 * file in （ in ）， 1 request「 + +
 * before 1200 」。 by `read_attachment` by need ，otherwise 1 
 * KB JSON will in 1 up down 。
 */

export type AssistantTextFile = {
  name: string
  /** （ use ；） */
  size: number
  text: string
}

/** 1 file（ and ） */
export const MAX_TEXT_FILES = 3
/** file limit： JSON has KB，512 KB already */
export const MAX_TEXT_FILE_BYTES = 512 * 1024
/** 1 request file */
export const MAX_FILE_PREVIEW_CHARS = 1200

export function isAssistantTextFile(value: unknown): value is AssistantTextFile {
  if (!value || typeof value !== 'object') return false
  const file = value as Record<string, unknown>
  return typeof file.name === 'string'
    && typeof file.text === 'string'
    && (typeof file.size === 'number' || file.size === undefined)
}

/** will file（ not will not ） */
export function assistantFiles(value: unknown): AssistantTextFile[] {
  if (!Array.isArray(value)) return []
  return value.filter(isAssistantTextFile).map(file => ({
    name: file.name,
    size: Number.isFinite(file.size) ? Number(file.size) : file.text.length,
    text: file.text,
  }))
}

/** 1 use file（ from 1 is ） */
export function assistantFileContext(messages: Array<{ role?: unknown; files?: unknown }>): AssistantTextFile[] {
  for (let index = messages.length - 1; index >= 0; index -= 1) {
    const message = messages[index]
    if (message?.role !== 'user') continue
    const files = assistantFiles(message.files)
    if (files.length) return files
  }
  return []
}

/**
 * to use ：`attachment:1`、`1`、`1.json` or file。
 * ； by when has 1 in ，file。
 */
export function assistantFileOf(
  files: AssistantTextFile[],
  ref: unknown,
): { file: AssistantTextFile; index: number } | null {
  const text = typeof ref === 'string' ? ref.trim() : typeof ref === 'number' ? String(ref) : ''
  if (!text) return null
  const numbered = /^attachment:(\d+)$/i.exec(text) || /^(\d+)$/.exec(text)
  if (numbered) {
    const index = Number(numbered[1]) - 1
    if (index < 0 || index >= files.length) return null
    return { file: files[index], index }
  }
  const exact = files.findIndex(file => file.name === text)
  if (exact >= 0) return { file: files[exact], index: exact }
  const lower = text.toLowerCase()
  const matches = files.flatMap((file, index) => file.name.toLowerCase() === lower ? [index] : [])
  if (matches.length === 1) return { file: files[matches[0]], index: matches[0] }
  return null
}

export function formatFileSize(bytes: number): string {
  const value = Math.max(0, Math.round(Number(bytes) || 0))
  if (value < 1024) return `${value} B`
  if (value < 1024 * 1024) {
    const kb = value / 1024
    return `${kb >= 10 ? Math.round(kb) : kb.toFixed(1).replace(/\.0$/, '')} KB`
  }
  const mb = value / 1024 / 1024
  return `${mb >= 10 ? Math.round(mb) : mb.toFixed(1).replace(/\.0$/, '')} MB`
}

/** （ to 1 「 this is 」 need ）： not is for / when returnempty */
export function jsonTopLevelKeys(text: string): string[] {
  try {
    const parsed = JSON.parse(text)
    if (Array.isArray(parsed)) return [`数组（${parsed.length} 项）`]
    if (parsed && typeof parsed === 'object') {
      return Object.keys(parsed as Record<string, unknown>).slice(0, 20)
    }
    return []
  } catch {
    return []
  }
}

/** file in 1 （ not up ： use need file to ） */
function fileBlock(file: AssistantTextFile, index: number): string {
  const ordinal = index + 1
  const keys = jsonTopLevelKeys(file.text)
  const shape = keys.length ? `顶层字段：${keys.join(', ')}` : '不是合法 JSON（可先用 read_attachment 读取，再修正结构）'
  const truncated = file.text.length > MAX_FILE_PREVIEW_CHARS
  const preview = truncated
    ? `${file.text.slice(0, MAX_FILE_PREVIEW_CHARS)}\n…（预览已截断，剩余 ${file.text.length - MAX_FILE_PREVIEW_CHARS} 字符）`
    : file.text
  const hint = truncated
    ? `完整内容请用 read_attachment 工具读取附件 ${ordinal}；导入时可直接用 attachment:${ordinal} 引用。`
    : `完整内容已在上方；导入时可引用 attachment:${ordinal}，也可以用 read_attachment 重新读取。`
  return [
    `【附件 ${ordinal}：${file.name}（${formatFileSize(file.size)}，${file.text.length} 字符）】`,
    shape,
    '',
    '--- 内容开始 ---',
    preview,
    '--- 内容结束 ---',
    hint,
  ].join('\n')
}

/**
 * 1 use + → to up 。
 *
 * （ up for ）， is in 
 * after ； when by use this 1 text part。
 */
export function userContentWithFiles(content: string, files: AssistantTextFile[]): string {
  const body = content
  if (!files.length) return body
  const blocks = files.map((file, index) => fileBlock(file, index)).join('\n\n')
  return `${body.trim() ? `${body}\n\n` : ''}【用户上传的文件】\n\n${blocks}`
}
