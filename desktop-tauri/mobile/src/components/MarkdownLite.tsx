import type { ReactNode } from 'react'

/**
 * Markdown （ ）。
 *
 * ── Why need need ──────────────────────────────────────────────
 * Markdown：`****`、`` `ID` ``、`- `。
 * in ， use to is and —— its is / 
 * this this use etc ， 。
 *
 * ── Why not up 1 Markdown ────────────────────────────────
 * up need need ：、、。 markdown-it / react-markdown
 * will to KB， need HTML （ is not can ，
 * `dangerouslySetInnerHTML` etc 1 XSS ）。 this use React ，
 * ， no has 。
 *
 * （```） by —— up this ，
 * as 1 to + not 。
 */
export function MarkdownLite({ text }: { text: string }) {
  const lines = String(text || '').split('\n')
  const blocks: ReactNode[] = []
  let list: string[] = []

  const flushList = (key: string) => {
    if (!list.length) return
    blocks.push(
      <ul key={key} style={{ margin: '4px 0', paddingLeft: 18 }}>
        {list.map((item, index) => (
          <li key={index} style={{ margin: '2px 0' }}>
            {inline(item)}
          </li>
        ))}
      </ul>,
    )
    list = []
  }

  lines.forEach((line, index) => {
    const trimmed = line.trim()
    // ：（ by ，file）
    if (trimmed.startsWith('```')) return
    const bullet = /^[-*+]\s+(.*)$/.exec(trimmed)
    const numbered = /^\d+[.)]\s+(.*)$/.exec(trimmed)
    if (bullet) {
      list.push(bullet[1])
      return
    }
    if (numbered) {
      list.push(numbered[1])
      return
    }
    flushList(`list-${index}`)
    if (!trimmed) {
      blocks.push(<div key={`gap-${index}`} style={{ height: 6 }} />)
      return
    }
    // （## / ###）：
    const heading = /^#{1,6}\s+(.*)$/.exec(trimmed)
    blocks.push(
      <div key={`line-${index}`} style={{ margin: '1px 0', fontWeight: heading ? 600 : undefined }}>
        {inline(heading ? heading[1] : trimmed)}
      </div>,
    )
  })
  flushList('list-tail')

  return <>{blocks}</>
}

/** ：**** and ``， its */
function inline(text: string): ReactNode[] {
  const parts = text.split(/(\*\*[^*]+\*\*|`[^`]+`)/g)
  return parts.filter(Boolean).map((part, index) => {
    if (part.startsWith('**') && part.endsWith('**') && part.length > 4) {
      return <strong key={index}>{part.slice(2, -2)}</strong>
    }
    if (part.startsWith('`') && part.endsWith('`') && part.length > 2) {
      return (
        <code
          key={index}
          style={{
            padding: '1px 5px',
            borderRadius: 5,
            background: 'var(--m-surface-2)',
            fontFamily: 'var(--m-mono)',
            fontSize: '0.94em',
            wordBreak: 'break-all',
          }}
        >
          {part.slice(1, -1)}
        </code>
      )
    }
    return <span key={index}>{part}</span>
  })
}
