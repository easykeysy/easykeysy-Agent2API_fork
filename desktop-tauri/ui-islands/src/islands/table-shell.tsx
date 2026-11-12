import * as React from 'react'
import {
  CardFooter,
  Input,
  Pager,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@ui'
import { t } from '../i18n'

/**
 * Agent2API · ** use **： 1 + client。
 *
 * 「」 this ：
 * · / to —— in ui/table-columns.js（ DOM ， use ，
 * that module header： can ， not down to ）；
 * · —— in file（React ， use ）。
 * has 1 ， as not by 。
 *
 * ── has （ use ）────────────────────────────────
 * N · current a–b ← 「current、 1 」
 * [50 ▾] ← 
 * [ 3 ] ← to 
 * [ up 1 ] [ down 1 ] x / y （ Pager）
 *
 * （ has 1 ）： 1 、， use can ——
 * when has when none 1 。
 *
 * ── ───────────────────────────────────────────
   * · **client**（ / Key / ）： 1 ，
 * file and （useClientPaging），「」 can 「」。
 * · **server**（request）：nonelimit，offset/limit by after ，
 * in （file）； not to 「」 this 1 
 * （ that will DOM）。
 *
 * ── is **** ───────────────────────────
 * `workbuddy-desktop-table-size:<>`：（、
 * request）， use 1 will 。
 */

/** ：， or 'all'（ not ， 1 ） */
export type PageSizeChoice = number | 'all'

/** not 「」 （server use ，file） */
export const SERVER_PAGE_SIZES: PageSizeChoice[] = [20, 50, 100, 200]

/** client default（ 1 「」） */
export const CLIENT_PAGE_SIZES: PageSizeChoice[] = [20, 50, 100, 200, 'all']

/** default（：server use need ） */
export const DEFAULT_PAGE_SIZE = 50

const SIZE_KEY_PREFIX = 'workbuddy-desktop-table-size:'

/** ：「」 is 1 */
export function pageSizeText(size: PageSizeChoice): string {
  return size === 'all' ? t('全部') : String(size)
}

/**
 * up 。
 *
 * `sizes` is this **current** ： already not in 
 * （e.g. after 「」）default —— otherwise will 「 down no has 
 * this 1 、 by 」 this no 。`fallback` is 「 no 」 when use 1 
 * （ to 「」： that is ， is ）。
 */
export function readPageSize(
  table: string,
  sizes: PageSizeChoice[] = CLIENT_PAGE_SIZES,
  fallback: PageSizeChoice = DEFAULT_PAGE_SIZE,
): PageSizeChoice {
  if (!sizes.includes(fallback)) fallback = sizes.includes(DEFAULT_PAGE_SIZE) ? DEFAULT_PAGE_SIZE : sizes[0]
  let raw: string | null = null
  try {
    raw = localStorage.getItem(SIZE_KEY_PREFIX + table)
  } catch {
    return fallback
  }
  if (raw === null || raw.trim() === '') return fallback
  if (raw === 'all') return sizes.includes('all') ? 'all' : fallback
  const value = Number(raw)
  return Number.isFinite(value) && sizes.includes(value) ? value : fallback
}

export function writePageSize(table: string, size: PageSizeChoice): void {
  try {
    localStorage.setItem(SIZE_KEY_PREFIX + table, String(size))
  } catch { /* etc not ： will */ }
}

/** useClientPaging return： use **1 **（ and use 1 ） */
export type ClientPaging = {
  size: PageSizeChoice
  /** current（1 ， already by total） */
  page: number
  pageCount: number
  /** 1 in （1 ）；empty as 0 */
  rangeStart: number
  /** after 1 ；empty as 0 */
  rangeEnd: number
  /** is not is 「」（ not ） */
  paged: boolean
  setSize: (size: PageSizeChoice) => void
  goto: (page: number) => void
  /** by current；「」return */
  slice: <T>(rows: readonly T[]) => T[]
}

/**
 * client。 1 use —— 、（）、
 * in this ， need need `slice(rows)` and return to TableFooter。
 *
 * `total` （、）current when **** after 1 ：otherwise use will 
 * in 1 empty up ， as 「 no has 」。
 */
export function useClientPaging(
  total: number,
  table: string,
  options: { sizes?: PageSizeChoice[]; defaultSize?: PageSizeChoice } = {},
): ClientPaging {
  const sizes = options.sizes ?? CLIENT_PAGE_SIZES
  const fallback = options.defaultSize ?? DEFAULT_PAGE_SIZE
  const [size, setSizeState] = React.useState<PageSizeChoice>(() => readPageSize(table, sizes, fallback))
  const [page, setPage] = React.useState(1)

  const pageCount = Math.max(1, size === 'all' ? 1 : Math.ceil(total / size))
  const current = Math.min(Math.max(1, page), pageCount)

  React.useEffect(() => {
    if (current !== page) setPage(current)
  }, [current, page])

  const setSize = React.useCallback((next: PageSizeChoice) => {
    setSizeState(next)
    writePageSize(table, next)
    // to 1 ： in 5 200 ， to will is ， no need 
    setPage(1)
  }, [table])

  const goto = React.useCallback((next: number) => {
    setPage(Math.min(Math.max(1, Math.round(next) || 1), pageCount))
  }, [pageCount])

  const start = size === 'all' ? 0 : (current - 1) * size
  const end = size === 'all' ? total : Math.min(total, start + size)

  const slice = React.useCallback(
    <T,>(rows: readonly T[]): T[] => (size === 'all' ? rows.slice() : rows.slice(start, end)),
    [size, start, end],
  )

  return {
    size,
    page: current,
    pageCount,
    rangeStart: total ? start + 1 : 0,
    rangeEnd: total ? end : 0,
    paged: size !== 'all',
    setSize,
    goto,
    slice,
  }
}

type TableFooterProps = {
  /** total（** after ** ： use to is ） */
  total: number
  page: number
  pageCount: number
  size: PageSizeChoice
  sizes?: PageSizeChoice[]
  onSizeChange: (size: PageSizeChoice) => void
  onPageChange: (page: number) => void
  /** （1 ）； not （「」） null */
  range?: { start: number; end: number } | null
  /** request in when （client not need need ） */
  disabled?: boolean
  /** ： use has ， not as */
  leading?: React.ReactNode
  className?: string
}

/**
 * 1 。 use （request is server， its is useClientPaging）。
 *
 * ： in 、 in （`.spacer` ）； to 1 not down when ，
 * to （`.panel-foot` flex-wrap， layout.css）。
 */
function TableFooter({
  total,
  page,
  pageCount,
  size,
  sizes = CLIENT_PAGE_SIZES,
  onSizeChange,
  onPageChange,
  range = null,
  disabled = false,
  leading,
  className,
}: TableFooterProps) {
  // is ****： in not ， / （ will 
  // in （1 → 12 → 123），request that will ）
  const [draft, setDraft] = React.useState(String(page))
  React.useEffect(() => setDraft(String(page)), [page])

  const commit = (): void => {
    const text = draft.trim()
    if (!text) { setDraft(String(page)); return }
    const next = Number(text)
    if (!Number.isFinite(next)) { setDraft(String(page)); return }
    const clamped = Math.min(Math.max(1, Math.round(next)), pageCount)
    setDraft(String(clamped))
    if (clamped !== page) onPageChange(clamped)
  }

  const single = pageCount <= 1
  const countText = total === 0
    ? t('共 0 条')
    : range
      ? t('共 {total} 条 · 当前第 {start}–{end} 条', { total, start: range.start, end: range.end })
      : t('共 {total} 条（全部显示）', { total })

  return (
    <CardFooter className={className ? `panel-foot table-foot-bar ${className}` : 'panel-foot table-foot-bar'}>
      {leading}
      <div className='spacer' />
      <div className='table-foot-ctl'>
        <span className='table-foot-count'>{countText}</span>
        <span className='table-foot-size'>
          {t('每页')}
          <Select
            value={String(size)}
            onValueChange={next => onSizeChange(next === 'all' ? 'all' : Number(next))}
          >
            <SelectTrigger className='h-[26px] w-[86px]' disabled={disabled} aria-label={t('每页条数')}>
              <SelectValue>{pageSizeText(size)}</SelectValue>
            </SelectTrigger>
            <SelectContent>
              {sizes.map(choice => (
                <SelectItem key={String(choice)} value={String(choice)}>{pageSizeText(choice)}</SelectItem>
              ))}
            </SelectContent>
          </Select>
          {t('条')}
        </span>
        <span className='table-foot-jump'>
          {t('跳至')}
          <Input
            className='h-[26px] w-[52px] px-1 text-center'
            value={draft}
            disabled={disabled || single}
            aria-label={t('跳转到指定页')}
            title={single ? t('当前只有一页') : t('跳转到第 1–{pageCount} 页', { pageCount })}
            inputMode='numeric'
            onChange={event => setDraft(event.target.value)}
            onKeyDown={event => {
              if (event.key === 'Enter') { event.preventDefault(); commit() }
              // ： （）
              else if (event.key === 'Escape') setDraft(String(page))
            }}
            onBlur={commit}
          />
          {t('页')}
        </span>
        <Pager page={page} pageCount={pageCount} onPageChange={onPageChange} disabled={disabled} />
      </div>
    </CardFooter>
  )
}

export { TableFooter }
