import * as React from 'react'
import { createRoot } from 'react-dom/client'
import { flushSync } from 'react-dom'
import { Button, Popover, PopoverContent, SegmentedControl, Switch, cn } from '@ui'
import { t } from '../i18n'

/**
 * Agent2API · set（ / · · for ）—— React 。
 *
 * ui/table-col-settings.js。 that has ，：
 *
 * ── 1 ：config can （ old ）──────────────────────────
 * `register` / `apply` / `configOf` / `syncStaticHead` / `close` is for ，
 * （、、 Key、request、 to ） in 。 is 1 
 * 「 key → { , for } has config」+ localStorage （ before old ），
 * and —— this not is 「」， is use ****： 1 
 * （`apply` to 、`configOf` return、`syncStaticHead` for ），
 * / / for will and 1 。 this TS， not as 。
 *
 * ── ：（）────────────────────────────────
 * to `Popover`（Base UI，`modal={false}` ）：（floating-ui）、
 * 、Esc、 is ， this not 、 not scroll / resize
 * 、 not pointerdown and keydown。 old ： by need div + createRoot、
 * when `unmount()` + （ and conc-dialog / request-clear-modal 1 ）—— is 
 * ， as 1 「 when not in 」 React root not 。
 *
 * ** use that by **：PopoverContent `anchor` Base UI 
 * `Positioner.anchor` ， is `button.getBoundingClientRect()`，
 * not to by 、 not 1 （ up will by hover and title）。
 * 「 and by that 1 down 」 1 ： no has Trigger， by click will Base UI
 * 「」， this 1 。
 * ： `Switch`、 for `SegmentedControl`、 by `Button`。
 *
 * ── by ────────────────────────────────────────────────
 * Button in display:contents in ， by register 。
 * must 1 in in ，otherwise React none to by 。
 *
 * ── and by that 1 down （ this is ）──────
 * Base UI 「」 is in document ****（useDismiss click ），
 * by click in —— is 「 when by 」 is ：
 * ① ： not in 、 not is already → as → （ React root）；
 * ② ： by togglePanel → no has → ** 1 **。
 * is by by down not 。 is ① that 1 click ：onOpenChange 
 * eventDetails ， down ， by to 1 for 、
 * not （ outsidePressEvent / makeButton）。 use for not is when ：
 * 1 click will this ， not will 。
 *
 * ── Why not up ──────────────────────────────────
 * and old 1 ： in config 1 need （ and for config），
 * HTML5 will in ** that ** up ， 1 ； in JS ，
 * not will 。 this old pointerdown / pointermove / pointerup，
 * old ： 50%、 1 in 1 、 and 。
 *
 * ── and old placePanel ────────────────────────────
 * old is 「 down 、 down not down to up 、 8px」， is ； in use 
 * `side='bottom'` + `align='end'` + `sideOffset=6` + `collisionPadding=8`（ after in ）
 * 1 to ， to and to floating-ui flip / shift。 and by ；
 * 1 is 「 / 」—— for this is 
 * ****（`data-anchor-hidden`） not is ， and old as use 。
 */

/* ─── and ─────────────────────────────── */

/** config before ： and old 1 ， use already config not this */
const STORE_PREFIX = 'agent2api-col-config:'

/** for ； class after （ta-left / ta-center / ta-right） */
type Align = 'left' | 'center' | 'right'

/**
 * and 。 to SegmentedControl options
 *（`{ value, label }` is need ）， has this 1 。
 */
const ALIGNS: readonly { value: Align; label: string }[] = [
  { value: 'left', label: t('左') },
  { value: 'center', label: t('中') },
  { value: 'right', label: t('右') },
]

/** for this （ this default， normalize） */
const isAlign = (value: unknown): value is Align => ALIGNS.some(align => align.value === value)

/** for class after ： when after ， */
const ALIGN_CLASSES = ALIGNS.map(align => `ta-${align.value}`)

/** in body up （cursor: grabbing + in ， in ui/css/table-col-settings.css， old use ） */
const DRAGGING_CLASS = 'colset-dragging'

/** by 「」 （`.colset-btn.open` in ui/css ， old use ） */
const BUTTON_OPEN_CLASS = 'open'

/**
 * 1 「 as 」 that click（ for ）。
 * when by ，Base UI will in and ， by 
 * in —— this 「 that 1 down is 」， not 。
 * use for not is when ： 1 click will this ， not will 。
 */
let outsidePressEvent: Event | null = null

/** and use 1 ： / / / for 。
 * minmax(78px,1fr)： not is for ；
 * for minmax(106px,auto)： down （ and old 116px ） */
const ROW_GRID = 'grid grid-cols-[18px_minmax(78px,1fr)_44px_minmax(106px,auto)] items-center gap-2'

/** 1 config： key → { , for }。** is use ** */
type ColConfigItem = { key: string; visible: boolean; align: Align }

/** in register when （， register ） */
type ColumnDecl = {
  key: string
  label?: string
  /** default is （ not ）；current no use to ， as 1 */
  visible?: boolean
  align?: Align
  /** up 1 default for ： to normalize old that 1 is 「 use 」 is 「 old default」 */
  legacyAlign?: Align
}

/** register （ to 1 ；mount is 、 or return ） */
type TableSpec = {
  id: string
  label?: string
  columns: ColumnDecl[]
  mount?: string | (() => Element | null)
  /** after ：（ + ） */
  onChange?: (config: ColConfigItem[]) => void
  /** by ：default before ；'last' to （ need in after ） */
  buttonPlacement?: 'first' | 'last'
  buttonClass?: string
}

/** = + currentconfig（config is that 1 ，） */
type TableEntry = TableSpec & { config: ColConfigItem[] }

/** v2：`{ v, defaults, items }`；v1 is （ normalize） */
type SavedV2 = { v?: number; defaults?: Record<string, string>; items?: unknown[] }

/**
 * window up by its / its 。
 *
 * use 「 + 」 not is declare global Window up ：wbIcons /
 * wbTableColumns is use ， declare 1 will not 1 
 * TS2717 —— and when 。file wbColSettings。
 */
type SharedWindow = {
  /** （icons.js ；return 1 SVG ） */
  wbIcons?: { icon?: (name: string, size?: number) => string }
  /** ： after need new for 1 and */
  wbTableColumns?: { repaint?: (id: string) => void }
}

function shared(): SharedWindow {
  return window as unknown as SharedWindow
}

/* ─── config and 1 （ old ）────── */

/** ：id → */
const registry = new Map<string, TableEntry>()

function defaultsOf(spec: TableSpec): ColConfigItem[] {
  return spec.columns.map(column => ({
    key: column.key,
    visible: column.visible !== false,
    align: column.align || 'left',
  }))
}

/**
 * config → currentavailable config。
 *
 * 1 （ is as 「 after old config can use 」）：
 * 1. already not in key （ that 1 ）；
 * 2. spec add key by default to in spec ** for **
 * （ in spec in in before 、 and use config in that 1 after ）；
 * 3. align ， this default。
 * as —— is use 。
 *
 * add to not is to ： has when is （ in 
 * 「」 and 「」）， to will already config use to default and not 。
 * in 「 no has this key」 when ， use not 。
 *
 * for need can 「 is use 」 and 「 is when default」：**default will **，
 * 1 as use not to new default。 is 「 ≠ when that default」 as 
 * use ， is need need ** when ** default， has 1 ：
 * · v2 `{ v, defaults, items }`：defaults is up when default for （）；
 * · v1 （ before ， no has ）： use up `legacyAlign`（= up 1 
 * default for ）， for that default has need 。
 * in 「 etc old default」 when new default， this 1 （ use and old default
 * ） not will use ； and not this 。
 */
function normalize(spec: TableSpec, saved: unknown): ColConfigItem[] {
  // v1 is ，v2 is { v, defaults, items }。「 + optional」
  const legacy = Array.isArray(saved)
  const savedObject = !legacy && saved && typeof saved === 'object' ? (saved as SavedV2) : null
  const items: unknown[] = legacy ? (saved as unknown[]) : Array.isArray(savedObject?.items) ? savedObject.items : []
  const snapshot = savedObject?.defaults && typeof savedObject.defaults === 'object' ? savedObject.defaults : null

  const byKey = new Map(spec.columns.map(column => [column.key, column]))
  const defaults = new Map(defaultsOf(spec).map(item => [item.key, item]))
  const out: ColConfigItem[] = [];
  const seen = new Set<string>()
  for (const raw of items) {
    const item = (raw && typeof raw === 'object' ? raw : {}) as { key?: unknown; visible?: unknown; align?: unknown }
    const key = String(item.key || '')
    if (!byKey.has(key) || seen.has(key)) continue
    seen.add(key)
    const column = byKey.get(key) as ColumnDecl
    const fallback = column.align || 'left'
    // 「 when that default」：v2 ，v1 up old default（currentdefault）
    const before = snapshot ? (snapshot[key] ?? fallback) : (column.legacyAlign ?? fallback)
    const stored = isAlign(item.align) ? item.align : null
    out.push({
      key,
      visible: item.visible !== false,
      // etc old default → use no this 1 → new default
      align: stored && stored !== before ? stored : fallback,
    })
  }
  for (const column of spec.columns) {
    if (seen.has(column.key)) continue
    insertBySpecOrder(out, spec, { ...(defaults.get(column.key) as ColConfigItem) })
  }
  return out
}

/**
 * 1 add to `out` 、 in `spec.columns` in before 
 * （ before not in `out` before ， not to then to before ）。
 *
 * add by `spec.columns` use ， for and spec 1 ——
 * as new in 「 before that already in 」 after ， before new already 。
 */
function insertBySpecOrder(out: ColConfigItem[], spec: TableSpec, item: ColConfigItem): void {
  const at = spec.columns.findIndex(column => column.key === item.key)
  for (let index = at - 1; index >= 0; index -= 1) {
    const previous = out.findIndex(entry => entry.key === spec.columns[index].key)
    if (previous >= 0) {
      out.splice(previous + 1, 0, item)
      return
    }
  }
  out.unshift(item)
}

function load(spec: TableSpec): ColConfigItem[] {
  try {
    const raw = localStorage.getItem(STORE_PREFIX + spec.id)
    return normalize(spec, raw ? JSON.parse(raw) : [])
  } catch {
    // default：set not not this not 
    return defaultsOf(spec)
  }
}

/**
 * 。 `{ v, defaults, items }`：
 * · `items` is config（key / visible / align）， and v1 ；
 * · `defaults` is ** when default for **， as down when can 「 this 1 is 
 * use 」 is 「 when default」—— normalize。 not this ， will default when 
 * no for 「 no 」。
 * v / defaults not （ will failure， is default
 * —— that is before as ， not will ）。
 */
function persist(spec: TableEntry): void {
  try {
    const defaults: Record<string, string> = {}
    for (const column of spec.columns) defaults[column.key] = column.align || 'left'
    localStorage.setItem(
      STORE_PREFIX + spec.id,
      JSON.stringify({ v: 2, defaults, items: spec.config }),
    )
  } catch {
    /* etc not ： will */
  }
}

/* ─── for ──────────────────────────────── */

/** register return： by config（`apply` is 1 ， register ） */
type ColSettingsHandle = {
  apply<C extends { key: string }>(columns: C[]): (C & { align: Align })[]
  config(): ColConfigItem[]
}

const configOf = (id: string): ColConfigItem[] | null => registry.get(id)?.config || null

const labelOf = (spec: TableSpec, key: string): string =>
  spec.columns.find(column => column.key === key)?.label || key

/**
 * 1 「 by 」 use config and ， and up for 。
 * 1 need key（ its ， can 
 * 1 ）。 id return —— and old 1 （set no when ）。
 */
function apply<C extends { key: string }>(id: string, columns: C[]): (C & { align: Align })[] {
  const entry = registry.get(id)
  if (!entry) return columns as (C & { align: Align })[]
  const byKey = new Map(columns.map(column => [column.key, column]))
  return entry.config
    .filter(item => item.visible && byKey.has(item.key))
    .map(item => ({ ...(byKey.get(item.key) as C), align: item.align }))
}

/**
 * ： → this → 1 。
 * `silent` old （ onChange，），currentnone use ， is as not 。
 */
function commit(spec: TableEntry, { silent = false }: { silent?: boolean } = {}): void {
  persist(spec)
  if (!silent) spec.onChange?.(spec.config)
  if (panel && panel.id === spec.id) panel.repaint()
}

/* ─── （： by need 、）────── */

/** is that by （PopoverContent anchor to Base UI） */
type Anchor = HTMLButtonElement

/** current。 has this 1 （ 1 is not can can ） */
type PanelHandle = {
  id: string
  spec: TableEntry
  /** React ； no when is empty（ old renderPanel） */
  repaint: () => void
  /** React root + ； （ will 、 .open） in unmount */
  destroy: () => void
}

let panel: PanelHandle | null = null

function closePanel(): void {
  if (!panel) return
  const current = panel
  // use ：unmount will ， that if to close not will 
  panel = null
  current.destroy()
}

function openPanelFor(spec: TableEntry, anchor: Anchor): void {
  closePanel()
  const host = document.createElement('div')
  document.body.append(host)
  const root = createRoot(host)
  const repaintRef: { current: (() => void) | null } = { current: null }
  const handle: PanelHandle = {
    id: spec.id,
    spec,
    repaint: () => repaintRef.current?.(),
    destroy: () => {
      root.unmount()
      host.remove()
    },
  }
  panel = handle
  root.render(<ColSettingsPanel spec={spec} anchor={anchor} repaintRef={repaintRef} onClose={closePanel} />)
}

/** 1 by 1 = ； 1 by = （ old ） */
function togglePanel(spec: TableEntry, anchor: Anchor): void {
  if (panel && panel.id === spec.id) closePanel()
  else openPanelFor(spec, anchor)
}

/* ─── ───────────────────────────────── */

/** 1 will ：teardown 、 body 、（ when need can ） */
type DragSession = { teardown: () => void }

/** ： from config to to */
function moveItem(config: ColConfigItem[], from: number, to: number): ColConfigItem[] {
  const next = [...config]
  const [item] = next.splice(from, 1)
  next.splice(to, 0, item)
  return next
}

/**
 * down that 1 key（ when use 「 1 」）。
 * in when —— can can has data-col-key。
 */
function rowKeyAt(x: number, y: number, rows: HTMLElement | null, config: ColConfigItem[]): string {
  const node = document.elementFromPoint(x, y)?.closest?.('[data-col-key]')
  if (!node || !rows || !rows.contains(node)) return ''
  const key = node.getAttribute('data-col-key') || ''
  return config.some(item => item.key === key) ? key : ''
}

/** （ SVG， and icons.js ： not ） */
const GRIP_ICON = (
  <svg viewBox='0 0 24 24' width={13} height={13} fill='currentColor' aria-hidden='true' className='block'>
    <circle cx='9' cy='6' r='1.6' />
    <circle cx='15' cy='6' r='1.6' />
    <circle cx='9' cy='12' r='1.6' />
    <circle cx='15' cy='12' r='1.6' />
    <circle cx='9' cy='18' r='1.6' />
    <circle cx='15' cy='18' r='1.6' />
  </svg>
)

type PanelProps = {
  spec: TableEntry
  /** ： that by ， is */
  anchor: Anchor
  /** React ：commit up config（ old renderPanel） */
  repaintRef: { current: (() => void) | null }
  /** （ by ） */
  onClose: () => void
}

/**
 * set。
 *
 * config** not in state **： that is can （`spec.config`）， 
 * `apply()` 1 。， use 1 ，
 * when `spec.config`。
 */
function ColSettingsPanel({ spec, anchor, repaintRef, onClose }: PanelProps) {
  const button = anchor
  const rowsRef = React.useRef<HTMLDivElement | null>(null)
  const [draggingKey, setDraggingKey] = React.useState<string | null>(null)
  const dragSessionRef = React.useRef<DragSession | null>(null)
  const [, setVersion] = React.useState(0)
  /** ：config use no （ or ） when can up */
  const repaint = React.useCallback(() => setVersion(version => version + 1), [])

  // to can commit（ old renderPanel etc ）
  React.useLayoutEffect(() => {
    repaintRef.current = repaint
    return () => {
      repaintRef.current = null
    }
  }, [repaint, repaintRef])

  // by 「」： to up —— in ， and aria in ，。
  // is 「 root」 1 to （ no has ）， this not will before （ in ui/css ， old use ）
  React.useEffect(() => {
    button.classList.add(BUTTON_OPEN_CLASS)
    button.setAttribute('aria-expanded', 'true')
    return () => {
      button.classList.remove(BUTTON_OPEN_CLASS)
      button.setAttribute('aria-expanded', 'false')
    }
  }, [button])

  // / ： this is old 1 no to 1 ——
  // Positioner will 「 + 」， after will in up 、 and by 
  React.useEffect(() => {
    const check = () => {
      const rect = button.getBoundingClientRect()
      if (!button.isConnected || !rect.width || rect.bottom <= 0 || rect.top >= window.innerHeight) onClose()
    }
    window.addEventListener('scroll', check, true)
    window.addEventListener('resize', check)
    return () => {
      window.removeEventListener('scroll', check, true)
      window.removeEventListener('resize', check)
    }
  }, [button, onClose])

  // when will （ old in this up will ： and body need etc down 1 pointerup ）。
  // already （、 not ）， 1 commit —— not and 
  // localStorage will not 1 （ by ， down by ）
  React.useEffect(() => () => {
    const session = dragSessionRef.current
    if (!session) return
    session.teardown()
    commit(spec)
  }, [spec])

  /** */
  function setVisible(key: string, next: boolean): void {
    const item = spec.config.find(entry => entry.key === key)
    if (!item) return
    // after 1 not ： after empty， use 「 is 」 can from 
    // 。 not 1 。 old is （ ）， this is 
    // not —— not ， up etc 。
    if (!next && spec.config.filter(entry => entry.visible).length <= 1) return
    item.visible = next
    commit(spec)
  }

  /** for */
  function setAlign(key: string, next: Align): void {
    const item = spec.config.find(entry => entry.key === key)
    if (!item || item.align === next) return
    item.align = next
    commit(spec)
  }

  /** default：、、 for 1 to （ old ） */
  function resetAll(): void {
    spec.config = defaultsOf(spec)
    commit(spec)
  }

  /**
   * 。 use not is HTML5 （ by file）。
   * in 1 1 config and ，** not 、 not ** ——
   * 1 will ；（ or ） commit。
   */
  function startDrag(key: string, event: React.PointerEvent<HTMLElement>): void {
    event.preventDefault()
    dragSessionRef.current?.teardown() // 理论上不会有上一段没收的会话，兜一手
    setDraggingKey(key)
    document.body.classList.add(DRAGGING_CLASS)
    const rows = rowsRef.current

    function teardown(): void {
      window.removeEventListener('pointermove', onMove)
      window.removeEventListener('pointerup', onUp)
      document.body.classList.remove(DRAGGING_CLASS)
      dragSessionRef.current = null
      setDraggingKey(null)
    }

    function onMove(moveEvent: PointerEvent): void {
      const overKey = rowKeyAt(moveEvent.clientX, moveEvent.clientY, rows, spec.config)
      if (!overKey || overKey === key) return
      const from = spec.config.findIndex(item => item.key === key)
      const to = spec.config.findIndex(item => item.key === overKey)
      if (from < 0 || to < 0) return
      spec.config = moveItem(spec.config, from, to)
      repaint()
    }

    function onUp(): void {
      teardown()
      commit(spec)
    }

    dragSessionRef.current = { teardown }
    window.addEventListener('pointermove', onMove)
    window.addEventListener('pointerup', onUp)
  }

  return (
    <Popover
      open
      // （Base UI default）： not this has 、 not this —— is 「」 
      modal={false}
      // ： by （ 1 ， makeButton ）、、Esc
      // to this 。eventDetails in 「」 when need —— that click down ，
      // by 「 this 1 down is 」， not （file）
      onOpenChange={(next, eventDetails) => {
        if (next) return
        if (eventDetails?.reason === 'outside-press') outsidePressEvent = eventDetails.event ?? null
        onClose()
      }}
    >
      <PopoverContent
        // down ：align='end' = for by 、side='bottom' = in by down ，
        // sideOffset is old 6px ； 8px use collisionPadding （= old EDGE）。
        // to by floating-ui flip ，file。
        // use that by ： no has Trigger， need use anchor 。
        // （z-[35]） and （ / / / / ） in 
        anchor={button}
        side='bottom'
        align='end'
        sideOffset={6}
        collisionPadding={8}
        className='w-[336px]'
        aria-label={t('{name}的列设置', { name: spec.label || spec.id })}
      >
        {/* ： and 1 ， is / / / for */}
        <div
          className={cn(
            ROW_GRID,
            'border-b border-hairline px-3 pt-[9px] pb-[7px]',
            'text-[10.5px] font-semibold tracking-[.05em] text-muted-foreground',
          )}
        >
          <span />
          <span className='text-center'>{t('列')}</span>
          <span className='text-center'>{t('显示')}</span>
          <span className='text-center'>{t('对齐')}</span>
        </div>

        {/* ： when not ， by （） max-h */}
        <div ref={rowsRef} className='min-h-0 flex-1 overflow-y-auto px-3 py-1'>
          {spec.config.map(item => {
            const label = labelOf(spec, item.key)
            const dragging = draggingKey === item.key
            return (
              <div
                key={item.key}
                data-col-key={item.key}
                className={cn(
                  ROW_GRID,
                  'rounded-sm py-[5px] hover:bg-nav-hover',
                  // that 1 ： down 。 not and 
                  //（ is + repaint， startDrag）
                  dragging && 'opacity-50',
                )}
              >
                <span
                  className='flex cursor-grab items-center justify-center text-muted-foreground opacity-50 hover:opacity-100'
                  title={t('按住拖动调整列顺序')}
                  onPointerDown={event => startDrag(item.key, event)}
                >
                  {GRIP_ICON}
                </span>
                <span className='min-w-0 truncate text-xs text-foreground' title={label}>
                  {label}
                </span>
                <Switch
                  className='justify-self-center'
                  checked={item.visible}
                  onCheckedChange={next => setVisible(item.key, next)}
                  aria-label={t('显示「{label}」列', { label })}
                  title={item.visible ? t('这一列正在显示') : t('这一列已隐藏')}
                />
                <SegmentedControl
                  className='w-full justify-center'
                  options={ALIGNS}
                  value={item.align}
                  onValueChange={next => setAlign(item.key, next)}
                  aria-label={t('「{label}」列的对齐', { label })}
                />
              </div>
            )
          })}
        </div>

        <div className='flex items-center gap-2 border-t border-hairline px-3 pt-2 pb-[9px]'>
          <span className='flex-1 text-[11px] text-muted-foreground'>{t('拖动 ⋮⋮ 调整顺序')}</span>
          <Button variant='outline' size='sm' onClick={resetAll}>
            {t('恢复默认')}
          </Button>
        </div>
      </PopoverContent>
    </Popover>
  )
}

/* ─── （ / Key）───── */
//
// this <colgroup> and <thead> in index.html （ not is ），
// and need ** has **， not can by ：
// · <col> up table-columns.js inline ， will ；
// · <th> （.col-grip）， after need etc down 。
// appendChild for already in is 「」，。
// before continue that 1 ： ** from DOM ** not is display:none ——
// <col> display in as not 1 ， is can 「 this 1 not in 」。

/** 1 for to / up （， after ） */
function applyAlign(el: Element | undefined, align: Align): void {
  if (!el) return
  el.classList.remove(...ALIGN_CLASSES)
  el.classList.add(`ta-${align}`)
}

/** ：id → { table, row, group, ths, cols } */
type StaticHead = {
  table: Element
  row: Element
  group: Element | null
  ths: Map<string, Element>
  cols: Map<string, Element>
}

/**
 * Whymust： 1 is <th> / <col> from DOM ****（ syncStaticHead），
 * after `querySelectorAll` not to —— ， use new up that 1 will ** not **
 * （ in ）。 when this ， after 1 from 。
 * when （ use ， or thead not in ） —— after DOM 
 * when can ， this is can ： will 。
 */
const staticHeads = new Map<string, StaticHead>()

function staticHeadOf(id: string, table: Element | null | undefined): StaticHead | null {
  const cached = staticHeads.get(id)
  if (cached && cached.table === table && cached.row.isConnected) return cached
  const row = table?.querySelector('thead tr')
  if (!row) return null
  const group = table?.querySelector('colgroup') ?? null
  const entry: StaticHead = {
    table: table as Element,
    row,
    group,
    ths: new Map([...row.querySelectorAll('th[data-col]')].map(el => [el.getAttribute('data-col') || '', el])),
    cols: new Map([...(group?.querySelectorAll('col[data-col]') || [])].map(el => [el.getAttribute('data-col') || '', el])),
  }
  staticHeads.set(id, entry)
  return entry
}

/**
 * 1 （<colgroup> + <thead> in HTML ） up config：
 * by config、 、 for new 1 。
 *
 * `viewHidden` is **** （optional）： in when need 
 * 「」「」 —— that is ， no has 。 and use in set 
 * config is ， not is config： when （ in staticHeads
 * ，）。 by by 1 can ， and not will 1 。
 */
function syncStaticHead(
  id: string,
  table: Element | null | undefined,
  viewHidden?: ReadonlySet<string> | null,
): void {
  const spec = registry.get(id)
  if (!spec || !table) return
  const head = staticHeadOf(id, table)
  if (!head) return
  const { row, group, ths, cols } = head

  for (const item of spec.config) {
    const th = ths.get(item.key)
    if (!th) continue
    const col = cols.get(item.key)
    if (!item.visible || viewHidden?.has(item.key)) {
      th.remove()
      col?.remove()
      continue
    }
    applyAlign(th, item.align)
    // config：appendChild to ， is 
    row.appendChild(th)
    if (col && group) group.appendChild(col)
  }
  // 1 ， that 1 need new for 1 ： is by 「 1 in after 」 
  //（ after 1 not to —— in will to ）， after is 1 by set。
  shared().wbTableColumns?.repaint?.(id)
}

/* ─── by and ────────────────────────────── */

/**
 * 「set」 by ： old use by and legacy （ by file）。
 * icons.js（ wbIcons）， and old 1 。
 *
 * when is （PopoverContent anchor ）， not 。
 */
function makeButton(spec: TableEntry): HTMLElement {
  const host = document.createElement('span')
  host.style.display = 'contents'
  flushSync(() => createRoot(host).render(<Button variant='outline' size='sm'
    className={`colset-btn${spec.buttonClass ? ` ${spec.buttonClass}` : ''}`}
    id={`btn-colset-${spec.id}`}
    title={t('调整「{name}」的列：显示 / 隐藏、顺序、对齐', { name: spec.label || spec.id })}
    aria-haspopup='dialog' aria-expanded={false}
    onClick={event => {
      if (outsidePressEvent === event.nativeEvent) { outsidePressEvent = null; return }
      togglePanel(spec, event.currentTarget)
    }}>
    <span dangerouslySetInnerHTML={{ __html: shared().wbIcons?.icon?.('settings', 14) || '' }} />
    <span>{t('列设置')}</span>
  </Button>))
  return host
}

function mountOf(spec: TableSpec): Element | null {
  const target = typeof spec.mount === 'function' ? spec.mount() : spec.mount ? document.querySelector(spec.mount) : null
  return target || null
}

/**
 * 1 ：config、 「set」 by 。
 *
 * `mount` this （、 or return ）：、 、
 * or that by 。 by default in ** before **： is 「 this 」 ，
 * and that 「 for 」 by not is 1 ， in before not will by 。
 * `buttonPlacement: 'last'` to —— need in by after 。
 *
 * key and for can has 1 ：`spec.columns` 。must by this 
 * （ / Key DOM by to <td> ， is ）。
 */
function register(spec: TableSpec): ColSettingsHandle {
  const entry: TableEntry = { ...spec, config: load(spec) }
  registry.set(spec.id, entry)
  const host = mountOf(spec)
  const button = makeButton(entry)
  if (host) {
    if (spec.buttonPlacement === 'last') host.appendChild(button)
    else host.insertBefore(button, host.firstChild)
  }
  return {
    apply: columns => apply(spec.id, columns),
    config: () => entry.config,
  }
}

/* ─── ： for ─────────────────────────── */

window.wbColSettings = { register, apply, configOf, syncStaticHead, close: closePanel }

declare global {
  interface Window {
    /** set（ ui/table-col-settings.js， and 1 ） */
    wbColSettings?: {
      /** 1 and by ；return apply / config */
      register(spec: TableSpec): ColSettingsHandle
      /** 「 by 」 use config / ， and up for */
      apply<C extends { key: string }>(id: string, columns: C[]): (C & { align: Align })[]
      /** has config（ and for ） */
      configOf(id: string): ColConfigItem[] | null
      /** （ / Key）；viewHidden is */
      syncStaticHead(id: string, table: Element | null | undefined, viewHidden?: ReadonlySet<string> | null): void
      /** （ when by app.js ） */
      close(): void
    }
  }
}
