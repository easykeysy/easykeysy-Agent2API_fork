/**
 * **（）+ **（ ui/accounts-columns.js and 
 * ui/accounts-table.js COLUMNS ）。
 *
 * ── and set（wbColSettings） ────────────────────────────
 * this is **** not ，：
 * · file**** —— `<col>` inline width， localStorage；
 * · wbColSettings ** and **（ for ）—— has `th[data-col]` /
 * `col[data-col]`， and from DOM 。
 * use 1 ：`ACCOUNT_COLUMNS` key（= CSS after `.cell-<key>`）。
 *
 * is table-layout: fixed， by `<colgroup>` `<col>` —— 
 * that 1 style.width， its not 。default has DEFAULTS this 1 （colgroup 
 * inline width），page-accounts-table.css `.cell-*` is 1 ，
 * need （ ： use 「」 after to 1 ）。
 * is （FLEX_COLUMNS）：** not **， remaining。
 */

import { t } from '../i18n'
import type { Align } from './accounts-shared'

/** （ / / / default for / old default for ） */
export type AccountColumn = {
  key: string
  label: string
  /** */
  title?: string
  align: Align
  /** ** up 1 default for **， table-col-settings normalize use old that 1 
   * is 「 use 」 is 「 old default」—— ， default for for use not */
  legacyAlign?: Align
}

/**
 * ： / / / / / / / valid / / 。
 *
 * ── （2026-10）「」 and 「」 ──────────
 * by ****， not in ： 1 is in （`#N`），
 * after by —— is 「」 that 1 （ + ↑↓） down ，
 * 「」 up ， and 1 （`⠿ #2`， OrderCell）。
 * 「」 when ： and already in up ， 1 is 。
 * set old config this key by table-col-settings 1 （ not not ）。
 *
 * `key` when is CSS after （`cell-<key>`），default in page-accounts-table.css by 
 * this —— has this 1 。
 *
 * label to 「」 not is empty： in no has （ that 1 is 「」
 * ），setmust has —— by label ，empty will key。
 *
 * default for ：（ when has 1 ）， its in 
 * （ is / / / this etc or ， in after 1 for 
 * to 1 in ， for ）。legacyAlign in 「 up 1 default and new not 」 up 。
 */
export const ACCOUNT_COLUMNS: AccountColumn[] = [
  { key: 'pick', label: t('选择'), align: 'center', legacyAlign: 'left' },
  {
    key: 'order', label: t('顺序'),
    title: t('组内先后：按住左边的把手上下拖动即可调整；整组的先后拖分组带的把手'),
    align: 'center', legacyAlign: 'left',
  },
  { key: 'account', label: t('账号'), align: 'center', legacyAlign: 'left' },
  {
    key: 'proxy', label: t('代理'),
    title: t('该账号出网走的代理（已保存的代理 / 自定义 / 直连）；点击可修改'),
    align: 'center',
  },
  {
    key: 'connections', label: t('连接数'),
    title: t('此刻正在使用这个账号的请求数（含还在下发内容的流式请求）；为 0 时不显示'),
    align: 'center',
  },
  { key: 'status', label: t('状态'), align: 'center', legacyAlign: 'left' },
  {
    key: 'limits', label: t('限流'),
    title: t('该账号当前限流中的模型；点徽章看明细'),
    align: 'center', legacyAlign: 'left',
  },
  { key: 'expiry', label: t('有效期'), align: 'center', legacyAlign: 'left' },
  // 「」（ by in ）： 1 「 / 」
  // will as this can 。「」 this down not （ / ）
  { key: 'usage', label: t('余额'), align: 'center', legacyAlign: 'left' },
  { key: 'actions', label: t('操作'), align: 'right' },
]

/* ─── ─────────────────────────────────── */

const STORE_KEY = 'agent2api-accounts-col-widths'

/**
 * default（px）： and page-accounts-table.css `.cell-*` 1 1 for 。
 *
 * this **must** when CSS and that file —— is 1 。
 * ： 1241px（ 47 / 84 / 186 / 56 / 80 /
 * 148 / valid 132 / 308 / 200）。
 * valid 132 is （only CodeArts）「Refresh Token · 317 after 」 ；
 * 1 ， empty by in ， not will empty。
 * 84 is 「 （20）+ （ #12， 30）+ 」 ：
 * 132px （ + ）， 48px；
 * 186 is 「 :」 + 41px ；
 * 308 or and ； use after as to ， not ；
 * 200 is by and （「 already to / / set / ⋯」），
 * by or by when 1 。
 *
 * ** not in this ** —— is 1 ， by FLEX_COLUMNS。
 */
const DEFAULTS: Record<string, number> = {
  pick: 47,
  order: 84,
  proxy: 186,
  connections: 56,
  status: 80,
  limits: 148,
  expiry: 132,
  usage: 308,
  actions: 200,
}

/**
 * key：** not **， by table-layout: fixed remaining to 。
 *
 * has in this 1 ： is 1 not can （ / uid / ），
 * this ； its is （ / / / /
 * by ）， 1 will 1 in not down ， to for 。
 *
 * this **** to 300px（DEFAULTS.account）， is 11 is 、 1 
 * not ： 1493px this ， has min(80vw, ) −
 * ， to 、 「 / ⋯」 1 （ by 
 * 104px）。 page-accounts-table.css that from 「.cell-account not 
 * ：table-layout: fixed down will down 」—— and that 1 300px is 
 * 。
 *
 * use after old is （ use as ）， 。
 */
const FLEX_COLUMNS = new Set(['account'])

/**
 * down ： to need this 1 when ，** to to **，
 * not is （page-accounts-table.css then ： not ， can ）。
 *
 * 300 is 「 default」（ before that 1 ）。 when is 
 * ** to **：「（1013）+ 300 = 1313px」 when ，
 * then —— not 、 「 / ⋯」 not 。
 */
const FLEX_MIN_WIDTH = 300

/** down ： this not */
const MIN_WIDTH = 56

/** has （ default + ）： by */
const KNOWN_COLUMNS = new Set([...Object.keys(DEFAULTS), ...FLEX_COLUMNS])

/** use （ has and default not will has ）， when from localStorage */
const overrides: Record<string, number> = (() => {
  try {
    const raw = JSON.parse(localStorage.getItem(STORE_KEY) || '{}') as Record<string, unknown>
    const clean: Record<string, number> = {}
    for (const [key, value] of Object.entries(raw || {})) {
      const width = Number(value)
      // by 「 in and 」， not is by 「 has no has default」—— no has default， can has 
      // use （ use will in down when ）
      if (KNOWN_COLUMNS.has(key) && Number.isFinite(width) && width >= MIN_WIDTH) clean[key] = Math.round(width)
    }
    return clean
  } catch {
    return {}
  }
})()

function persist(): void {
  try {
    localStorage.setItem(STORE_KEY, JSON.stringify(overrides))
  } catch { /* etc not ： will */ }
}

/**
 * when ： default + use 。
 *
 * in ** ** when will in this —— no has key，`<ColGroup>` not to that 1 
 * inline width， is 。 this 1 is can not can ， before 
 * ： this map， 1 ，。
 */
export function columnWidths(): Record<string, number> {
  return { ...DEFAULTS, ...overrides }
}

/**
 * down ：`∑ can + down `（ already already from DOM ， not ）。
 *
 * use 1 not can ：
 * · table-layout: fixed down remaining，「 and 」 when will 0
 * （ not ）—— this ui/table-columns.js ；
 * · when is 「 to 」 that 1 ： not 、，
 * 。 need 1 in down will not will ， is 。
 *
 * this must** from **， not can 。 is CSS 1 
 * `min-width: 1155px`（= 1193 − 38）， when 300px ——
 * need need 1493px，1155 not to ： no ， no can Note to 
 * 「 that 1400px already down 」（1193 + 300 + 11 = 1504 > 1400，
 * that down ）。
 */
export function tableMinWidth(visibleKeys: readonly string[]): number {
  let total = 0
  for (const key of visibleKeys) {
    const override = overrides[key]
    // not MIN_WIDTH： and already 
    if (override) total += override
    else if (FLEX_COLUMNS.has(key)) total += FLEX_MIN_WIDTH
    // default****， not need Math.max(MIN_WIDTH, …)：MIN_WIDTH is 「 can to 
    // 」 down ， not is —— 47px ， 47 is for 
    // （16 + 15 + 16， page-accounts-table.css .cell-pick）， to 56 will in 
    // has when 
    else if (DEFAULTS[key]) total += DEFAULTS[key]
    else total += MIN_WIDTH
  }
  return Math.round(total)
}

/**
 * index for 「 key + `<col>`」。
 *
 * set can 、 can ，** not can ** by ： is use ， 
 * is `data-col`（ th and colgroup col up ）， to key after by key 
 * that `<col>` —— 1 ， use after not will to up 。
 */
function columnAt(table: Element | null, index: number): { key: string; col: Element } | null {
  const header = table?.querySelector(`thead th:nth-child(${index + 1})`)
  const key = header?.getAttribute('data-col') || header?.className.match(/cell-([a-z]+)/)?.[1]
  if (!key) return null
  const col = table?.querySelector(`colgroup col[data-col="${CSS.escape(key)}"]`)
    || table?.querySelectorAll('colgroup col')[index]
  return col ? { key, col } : null
}

/** 1 `<col>`（ in when use is ） */
function applyWidth(col: Element, key: string, px: number): void {
  const width = Math.max(MIN_WIDTH, Math.round(px))
  ;(col as HTMLElement).style.width = width + 'px'
  // default not ； no has 「default」 can —— need down 
  // （ by from 「remaining」， ）
  if (FLEX_COLUMNS.has(key) || width !== DEFAULTS[key]) overrides[key] = width
  else delete overrides[key]
}

/** after （colgroup by widths() 1 ， DOM and for ） */
let repaint = (): void => {}

/**
 * ：pointerdown 、dblclick 。 in up 1 can ，
 * after valid（ to ， not ）。
 *
 * and table-columns.js 1 （ + setPointerCapture + ）：
 * if （）， not ， 1 。
 */
export function bindColumnGrips(host: HTMLElement | null, onChange: () => void): void {
  repaint = onChange
  if (!host) return
  let dragging: { col: Element; key: string; startX: number; startWidth: number; grip: Element; pointerId: number } | null = null

  host.addEventListener('pointerdown', event => {
    if (event.button !== 0) return
    const grip = (event.target as Element | null)?.closest?.('.col-grip')
    if (!grip) return
    event.preventDefault()
    const th = grip.closest('th')
    const table = th?.closest('table') || null
    const index = th?.parentElement ? [...th.parentElement.children].indexOf(th) : -1
    const column = columnAt(table, index)
    if (!column) return
    const startX = event.clientX
    const startWidth = column.col.getBoundingClientRect().width
    grip.classList.add('active')
    document.body.classList.add('col-resizing')
    try { (grip as Element & { setPointerCapture(id: number): void }).setPointerCapture(event.pointerId) } catch { /* */ }
    dragging = { ...column, startX, startWidth, grip, pointerId: event.pointerId }
    window.addEventListener('pointermove', move)
    window.addEventListener('pointerup', up)
    window.addEventListener('pointercancel', up)
    window.addEventListener('blur', up)

    function move(moveEvent: PointerEvent): void {
      if (!dragging) return
      if (moveEvent.buttons === 0) { up(); return }
      applyWidth(dragging.col, dragging.key, dragging.startWidth + moveEvent.clientX - dragging.startX)
    }

    function up(): void {
      if (!dragging) return
      const { grip: activeGrip, pointerId } = dragging
      dragging = null
      activeGrip.classList.remove('active')
      document.body.classList.remove('col-resizing')
      try { (activeGrip as Element & { releasePointerCapture(id: number): void }).releasePointerCapture(pointerId) } catch { /* already */ }
      window.removeEventListener('pointermove', move)
      window.removeEventListener('pointerup', up)
      window.removeEventListener('pointercancel', up)
      window.removeEventListener('blur', up)
      persist()
      repaint()
    }
  })

  host.addEventListener('dblclick', event => {
    const grip = (event.target as Element | null)?.closest?.('.col-grip')
    if (!grip) return
    const th = grip.closest('th')
    const table = th?.closest('table') || null
    const index = th?.parentElement ? [...th.parentElement.children].indexOf(th) : -1
    const column = columnAt(table, index)
    if (!column) return
    delete overrides[column.key]
    persist()
    repaint()
  })
}
