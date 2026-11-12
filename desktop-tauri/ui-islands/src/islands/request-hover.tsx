import * as React from 'react'
import { flushSync } from 'react-dom'
import { createRoot, type Root } from 'react-dom/client'
import { t } from '../i18n'
import { Tooltip, TooltipContent, TooltipProvider } from '@ui'

/**
 * request「 / 」 。
 *
 * in ， bind / hasProcessFacts / beforeListRedraw /
 * afterListRedraw 。 use Tooltip， not 、 not 。
 * new before after need need by and ， not to new when ，
 * use already 。
 */

/* ─── ───────────────────────────────────── */
/** / （ms）： and desktop-tooltip.tsx 1 ， 1 */
const SHOW_DELAY = 150
const HIDE_DELAY = 80
/** in ： this 1 is 「 in 」 ， not is */
const MAX_TERM_ROWS = 12
/** id： use aria-describedby （ and old 1 id） */
const PANEL_ID = 'req-hover-panel'

/** （ old `.rh-row` + `.rh-dim` ，， 1 ） */
const DIM_ROW = 'rh-row rh-dim min-w-0 break-words text-muted-foreground'
/** / use 「」（ old `.rh-row`， page-requests.css ） */
const FLOW_ROW = 'rh-row min-w-0 break-words'

/* ─── ───────────────────────────────────── */
/**
 * request：file use down this ， 1 by unknown （ after can can to / /
 * null）， 1 —— and old `Number(x) || 1` 1 。
 */
type RequestEntry = {
  attempts?: unknown
  provider?: unknown
  status?: unknown
  error?: unknown
  attemptDetails?: unknown
  sensitiveHits?: unknown
}

/** 1 （ after AttemptDetail； is after ） */
type AttemptDetail = {
  provider?: unknown
  account?: unknown
  status?: unknown
  error?: unknown
  notice?: unknown
  retries?: unknown
}

/** 1 1 */
type AttemptRetry = { reason?: unknown; status?: unknown; delayMs?: unknown }

/** in （ after SensitiveHit） */
type SensitiveHit = { word?: unknown; count?: unknown }

/**
 * ： `data-req-hover` —— has 'sensitive' is ， its （empty）
 * 1 by （ and old `=== 'sensitive' ? sensitive : chain` ）。
 */
type PanelKind = 'chain' | 'sensitive'

/** ；`null` = no has in */
type Remap = { action: 'move'; kind: string; key: string } | { action: 'close' }

/** bind parameter： + 「 → 」 （ bind ） */
type BindOptions = {
  host?: HTMLElement | null
  entryOf?: (tag: HTMLElement) => unknown
}

/**
 * window up by its 。
 *
 * use 「 + 」 not is declare global：wbApp / wbProviders is use ，
 * declare 1 will not 1 TS2717。file use declare global 
 * wbRequestHover（file）。
 */
type SharedWindow = {
  /** directory（providerLabel ：directory → id） */
  wbProviders?: { labelOf?: (id: string) => string }
  /** old from wbApp `esc` innerHTML，** not use **（React ， esc
   * 1 will `&` `&amp;`）。 as 。 */
  wbApp?: { esc?: (value: unknown) => string }
}

function shared(): SharedWindow {
  return window as unknown as SharedWindow
}

/* ─── and （ and after 1 1 for ）───── */
/** for / by for （ old is `if (!entry) return`）， its to null */
function asEntry(value: unknown): RequestEntry | null {
  return typeof value === 'object' && value !== null ? (value as RequestEntry) : null
}

/** 1 （ after is attemptDetails； old no has this → empty） */
const detailsOf = (entry: RequestEntry | null | undefined): AttemptDetail[] =>
  Array.isArray(entry?.attemptDetails) ? (entry.attemptDetails as AttemptDetail[]) : []

/** 1 （ old no has this → empty） */
const retriesOf = (item: AttemptDetail | null | undefined): AttemptRetry[] =>
  Array.isArray(item?.retries) ? (item.retries as AttemptRetry[]) : []

/** in （ after is sensitiveHits； old no has this → empty） */
const hitsOf = (entry: RequestEntry | null | undefined): SensitiveHit[] =>
  Array.isArray(entry?.sensitiveHits) ? (entry.sensitiveHits as SensitiveHit[]) : []

/**
 * provider id → ， not to id。
 *
 * ⚠️ old this is 「 after label → before providers directory → id」、
 * and request 「」（requests-panel.js targetCell），**
 * **（ `wbProviders.labelOf` + ， no providerLabel）。
 * as 、 not ； need not need 1 （： this item.providerLabel）。
 */
function providerLabel(id: unknown): string {
  const key = String(id ?? '').trim()
  if (!key) return ''
  return shared().wbProviders?.labelOf?.(key) || key
}

/**
 * this request is has ** **（ that 「」 not ）。
 *
 * Why not is `attempts > 1`：`attempts` ****（ after 
 * `TelemetrySnapshot::attempts`）， （11-128 、 when 5xx、
 * failure、401 new ） not —— 1 11-128 down 、 3 after success request `attempts` is 1，
 * old down this request in not 。 is 「 ** or ** ** or ** has 」。
 *
 * this is 1 1 ** to use **（requests-panel.js retryCell ）： need 
 * ， 1 will has 。 that not in this —— is 1 1 
 *（`sensitiveHits`） ，。
 */
function hasProcessFacts(entry: unknown): boolean {
  const row = asEntry(entry)
  if ((Number(row?.attempts) || 1) > 1) return true
  return detailsOf(row).some(item => retriesOf(item).length > 0 || Boolean(item?.notice))
}

/** → （ PanelKind ） */
function kindOfTag(tag: HTMLElement): PanelKind {
  return tag.dataset.reqHover === 'sensitive' ? 'sensitive' : 'chain'
}

/* ─── （， and old for ）── */
/**
 * path：`A → B → C`。** in ≥2 when **： has 1 when that 
 * is 「A → success」， no has ； this 1 in has when ，
 * this need is 「 attempts 」 （、 or no to ）。
 */
function ChainLine({ details }: { details: AttemptDetail[] }) {
  if (details.length < 2) return null
  const names = details.map(item => providerLabel(item.provider) || t('未知'))
  return (
    <div className='rh-chain mb-1.5 border-b border-border pb-1.5 break-words'>
      <span className='rh-chain-k font-semibold text-muted-foreground'>{t('切换路径：')}</span>
      <span className='rh-chain-v font-semibold text-foreground'>{names.join(' → ')}</span>
    </div>
  )
}

/**
 * 1 ：`↻ N ` + Reason。
 *
 * in that 1 down （ + ）， not is as and —— is 
 * ** 1 ** （ is when not is ）， and will 「path」 that 1 
 * ， 。：`Reason（HTTP ，none then ），Xseconds after `。
 */
function RetryRows({ item }: { item: AttemptDetail }) {
  const retries = retriesOf(item)
  if (!retries.length) return null
  return (
    <div className='rh-retry mt-0.5 mb-1 ml-2.5 border-l-2 border-border-strong pl-2'>
      <div className='rh-retry-head text-[11.5px] font-semibold text-warning'>
        {t('↻ 重试 {n} 次', { n: retries.length })}
      </div>
      {retries.map((retry, index) => {
        const reason = String(retry?.reason ?? '').trim()
        const status = Number(retry?.status)
        const hasStatus = retry?.status !== null && retry?.status !== undefined && Number.isFinite(status)
        const delayMs = Number(retry?.delayMs)
        const delay = Number.isFinite(delayMs) && delayMs > 0
          ? (delayMs >= 1000
              ? t('，{n}秒后重试', { n: Math.round(delayMs / 1000) })
              : t('，{n}毫秒后重试', { n: Math.round(delayMs) }))
          : ''
        return (
          <div key={index} className='rh-retry-row min-w-0 break-words'>
            <span className='rh-retry-why mr-1 text-subtle [overflow-wrap:anywhere]'>
              {reason || t('未知原因')}
            </span>
            {hasStatus ? (
              <span className='rh-retry-status mr-1 text-muted-foreground tabular-nums whitespace-nowrap'>
                {`HTTP ${String(status)}`}
              </span>
            ) : null}
            {delay ? <span className='rh-dim text-muted-foreground'>{delay}</span> : null}
          </div>
        )
      })}
    </div>
  )
}

/**
 * 1 ：` N · （）→ success(200) / failure(500)：error need `。
 *
 * and ：
 * · `error` empty → failure（），（ can can no has ：failure）
 * · `status` has 、noneerror → success（），
 * · none → （）。： this 1 in （`inFlight`， in after 
 * 1 —— 1 in ， this is ）， and 。 before 「 in …」，
 * after 「none」： for 1 in request「none」 will already failure。
 *
 * in after ： 1 can has ，「WorkBuddy / aibjchat001@gmail.com」 has 
 * can to that 1 （ this before in ）。
 */
function AttemptRow({ item, index, inFlight }: { item: AttemptDetail; index: number; inFlight: boolean }) {
  const name = providerLabel(item.provider)
  const account = String(item.account ?? '').trim()
  const status = Number(item.status)
  const hasStatus = item.status !== null && item.status !== undefined && Number.isFinite(status)
  const error = item.error ? String(item.error) : ''
  const notice = String(item.notice ?? '').trim()
  return (
    <>
      <div className={FLOW_ROW}>
        {/* from 1 （ after is ， not need need attempt_no） */}
        <span className='rh-no mr-1 text-muted-foreground'>{t('尝试 {n} ·', { n: index + 1 })}</span>
        <span className='rh-who mr-1 font-semibold text-foreground'>{name || t('未知')}</span>
        {account ? (
          <span className='rh-account mr-1 text-muted-foreground [overflow-wrap:anywhere]'>{account}</span>
        ) : null}
        <span className='rh-arrow mr-1 text-muted-foreground'>→</span>
        {error ? (
          <span className='rh-bad font-medium text-destructive [overflow-wrap:anywhere]'>
            {hasStatus
              ? t('失败（{status}）：{error}', { status: String(status), error })
              : t('失败：{error}', { error })}
          </span>
        ) : hasStatus ? (
          <span className='rh-ok font-medium text-success'>{t('成功（{status}）', { status: String(status) })}</span>
        ) : inFlight ? (
          <span className='rh-dim text-muted-foreground'>{t('进行中…')}</span>
        ) : (
          <span className='rh-dim text-muted-foreground'>{t('无结果记录')}</span>
        )}
      </div>
      {/* （ etc ）：failure、 1 */}
      {notice ? (
        <div className='rh-notice mt-0.5 mb-1 ml-2.5 border-l-2 border-border-strong pl-2 text-warning [overflow-wrap:anywhere]'>
          {`⚠️ ${notice}`}
        </div>
      ) : null}
      <RetryRows item={item} />
    </>
  )
}

/**
 * 。：
 * ① has ：path + 。
 * ② no has （ old / before failure）： to 1 「 N ， by X 」—— not to ，
 * already has （、）。** not in **： that will is 。
 * in ①： 1 in ， after 1 status / error as empty —— that is 「 this 1 in 」，
 * by AttemptRow inFlight 「 in …」。
 */
function ChainPanel({ entry }: { entry: RequestEntry }) {
  const attempts = Number(entry.attempts) || 1
  const details = detailsOf(entry)
  if (!details.length) {
    const finalProvider = providerLabel(entry.provider)
    const sentence = finalProvider
      ? t('共 {n} 次尝试，最终由 {provider} 承载', { n: attempts, provider: finalProvider })
      : t('共 {n} 次尝试', { n: attempts })
    return (
      <>
        <div className={DIM_ROW}>{sentence}</div>
        <div className={DIM_ROW}>
          {t('这条记录的尝试明细未采集（该字段上线前的旧数据，或请求在转发前就失败）')}
        </div>
      </>
    )
  }
  // when （： down is that ）。 is 「 attempts
  // 」， and after MAX_ATTEMPT_DETAILS none —— limit this 。
  // 「 in 」 can can is after 1 ： is ， before 1 not 。
  const running = (Number(entry.status) || 0) === 0 && !entry.error
  return (
    <>
      <ChainLine details={details} />
      {details.map((item, index) => (
        <AttemptRow
          key={index}
          item={item}
          index={index}
          inFlight={running && index === details.length - 1}
        />
      ))}
      {details.length < attempts ? (
        <div className={DIM_ROW}>
          {t('另有 {missing} 次尝试未记录明细（只保留最早的 {kept} 条）', {
            missing: attempts - details.length, kept: details.length,
          })}
        </div>
      ) : null}
    </>
  )
}

/**
 * ： + 「 × 」。 no has in （ has old ） when to 1 
 * —— by 「 in empty」， this in down not to ； is 「 has 
 * attempts 」 that 。
 */
function SensitivePanel({ entry }: { entry: RequestEntry }) {
  const hits = hitsOf(entry)
  if (!hits.length) {
    return <div className={DIM_ROW}>{t('这条记录命中了敏感词，但没有留下命中明细')}</div>
  }
  // after already by to ， this not （ has 1 ）
  return (
    <>
      <div className='rh-title mb-1 text-[11.5px] font-semibold tracking-[.02em] text-muted-foreground'>
        {t('命中的敏感词')}
      </div>
      {hits.slice(0, MAX_TERM_ROWS).map((hit, index) => (
        <div key={index} className='rh-row rh-hit flex items-baseline justify-between gap-2.5'>
          <span className='rh-hit-w break-words text-foreground'>{String(hit?.word ?? '')}</span>
          <span className='rh-hit-c text-muted-foreground tabular-nums whitespace-nowrap'>
            {`× ${String(Number(hit?.count) || 0)}`}
          </span>
        </div>
      ))}
      {hits.length > MAX_TERM_ROWS ? (
        <div className={DIM_ROW}>{t('另有 {n} 个词命中', { n: hits.length - MAX_TERM_ROWS })}</div>
      ) : null}
    </>
  )
}

/** and ， and 。 */
function HoverPanel({
  kind,
  entry,
}: {
  kind: PanelKind
  entry: RequestEntry
}) {
  return <TooltipProvider><Tooltip open disableHoverablePopup>
    <TooltipContent anchor={anchor} id={PANEL_ID} side='bottom' variant='plain' data-kind={kind}
      className={kind === 'sensitive'
        ? 'pointer-events-none w-max min-w-[220px] max-w-[min(900px,calc(100vw-16px))] text-[12px]'
        : 'pointer-events-none w-[min(520px,calc(100vw-16px))] max-w-[min(900px,calc(100vw-16px))] text-[12px]'}>
      {kind === 'sensitive' ? <SensitivePanel entry={entry} /> : <ChainPanel entry={entry} />}
    </TooltipContent>
  </Tooltip></TooltipProvider>
}

/* ─── ───────────────────────────────── */
/** current */
let anchor: HTMLElement | null = null
let showTimer = 0
let hideTimer = 0
/**
 * that （`showTimer` ）。 need need ： when up when `anchor` is 
 * null（ no ）， pointerout before `anchor` —— is 「 1 down 」 will ，
 * when pointerout not this when 。
 */
let pendingTag: HTMLElement | null = null

/** and 「 → 」 （bind when down ； after need use ） */
let listHost: HTMLElement | null = null
let entryOfFn: ((tag: HTMLElement) => unknown) | null = null

/** （ beforeListRedraw / afterListRedraw） */
let remap: Remap | null = null

/** div / React root / use / current（ when empty） */
let panelHost: HTMLElement | null = null
let panelRoot: Root | null = null
let panelKind: PanelKind = 'chain'

function cancelTimers(): void {
  window.clearTimeout(showTimer)
  window.clearTimeout(hideTimer)
  showTimer = 0
  hideTimer = 0
  pendingTag = null
}

/* ─── （ by need / ）────────────── */
/** （： click，「」 not ） */
function onDocumentClick(event: MouseEvent): void {
  if (!anchor) return
  const target = event.target
  if (!(target instanceof Node)) return
  // in or not 「」（ is button， not this ）
  if (target === anchor || anchor.contains(target)) return
  closePanel()
}

/**
 * / when **** not is new （ and desktop-tooltip.tsx ）： in up ， can 
 *（`.log-list`）， when in 、 is fixed ， will in up 、 in 。
 * after in up ， 1 will new 。
 */
function onScroll(): void {
  closePanel()
}

function onResize(): void {
  closePanel()
}

let globalsBound = false

function bindGlobals(): void {
  if (globalsBound) return
  globalsBound = true
  document.addEventListener('click', onDocumentClick, true)
  window.addEventListener('scroll', onScroll, true)
  window.addEventListener('resize', onResize)
}

function unbindGlobals(): void {
  if (!globalsBound) return
  globalsBound = false
  document.removeEventListener('click', onDocumentClick, true)
  window.removeEventListener('scroll', onScroll, true)
  window.removeEventListener('resize', onResize)
}

/* ─── （ by need 、）────────── */
/** and root（ when 1 ， not div、 not ） */
function unmountPanel(): void {
  unbindGlobals()
  if (panelRoot) {
    panelRoot.unmount()
    panelRoot = null
  }
  if (panelHost) {
    panelHost.remove()
    panelHost = null
  }
}

/** and update in 1 ；onlycreate。 */
function renderPanel(kind: PanelKind, entry: RequestEntry): void {
  panelKind = kind
  if (!panelHost || !panelRoot) {
    panelHost = document.createElement('div')
    document.body.append(panelHost)
    panelRoot = createRoot(panelHost)
  }
  const root = panelRoot
  flushSync(() => {
    root.render(<HoverPanel kind={kind} entry={entry} />)
  })
  bindGlobals()
}

/** （ by entryOf ； not to not ） */
function openPanel(el: HTMLElement): void {
  // must in ： will new ， 150ms down this 
  // to can can is old —— rect 0（ will in up ）， and not to 
  // pointerout、 not will 。 this can ： in up ， pointerover
  // will new 1 。
  if (!el.isConnected) return
  const entry = asEntry(entryOfFn?.(el))
  // not to （ in before new ）： not （ old returnempty HTML when ）
  if (!entry) return
  cancelTimers()
  if (anchor && anchor !== el) closePanel()
  anchor = el
  renderPanel(kindOfTag(el), entry)
  el.classList.add('active')
  el.setAttribute('aria-describedby', PANEL_ID)
}

function closePanel(): void {
  cancelTimers()
  if (anchor) {
    anchor.classList.remove('active')
    anchor.removeAttribute('aria-describedby')
    anchor = null
  }
  unmountPanel()
}

/* ─── when ───────────────────── */
/**
 * ** before ** use （ in `innerHTML` before ）： down 。
 *
 * has 「」 （ use in ， this new 1 ）， use 「 + 
 * 」（`data-req-hover` and `data-req-id`） down —— use not is use ：，
 * use 1 will 。 its 1 as ， by is 1 ：
 * · `hideTimer` = already 、 etc HIDE_DELAY —— after that when 
 * （`anchor === tag`） already not ， will 「 this no 」；
 * · not in in up = —— will 、 to body，
 * 「 not in up 、」 ；
 * · = this not is by ，none from 。
 */
function beforeListRedraw(): void {
  if (!anchor) return
  const kind = String(anchor.dataset.reqHover || '')
  const key = String(anchor.dataset.reqId || '')
  const movable = Boolean(kind && key) && anchor.matches(':hover') && !hideTimer
  remap = movable ? { action: 'move', kind, key } : { action: 'close' }
}

/**
 * ** after ** use （ in `innerHTML` after ）： by —— then 、、
 * （ use not to ）； not new （ current / empty / / not to ） or 
 * is then closePanel()。
 *
 * Why not 「」：default 1 seconds 1 ， will seconds 1 、 need 1 down 
 * 。 this is 「」 —— not （ not to new ） 1 。
 */
function afterListRedraw(): void {
  const pending = remap
  remap = null
  if (!pending || !anchor) return
  if (pending.action === 'close') {
    closePanel()
    return
  }
  const next = listHost ? findTag(listHost, pending.kind, pending.key) : null
  // not new （ current / empty / ） not 
  const entry = next ? asEntry(entryOfFn?.(next)) : null
  if (!next || !entry) {
    closePanel()
    return
  }
  if (next === anchor) return // 整表重绘必然换节点；这条兜住宿主改成局部更新后误伤
  anchor.classList.remove('active')
  anchor.removeAttribute('aria-describedby')
  anchor = next
  renderPanel(kindOfTag(next), entry)
  next.classList.add('active')
  next.setAttribute('aria-describedby', PANEL_ID)
}

/** in new DOM by 「 + 」； no has is this 1 not in */
function findTag(host: HTMLElement, kind: string, key: string): HTMLElement | null {
  // use not is ： after （id or when ）， not and 
  // —— 1 is 1 not to 。 1 ，50 for 1 seconds 1 
  // none。
  for (const el of host.querySelectorAll<HTMLElement>('[data-req-hover]')) {
    if (String(el.dataset.reqHover || '') === kind && String(el.dataset.reqId || '') === key) return el
  }
  return null
}

/* ─── （， is legacy DOM）── */
function tagFromEvent(event: Event): HTMLElement | null {
  const target = event.target
  if (!(target instanceof Element)) return null
  return target.closest<HTMLElement>('[data-req-hover]')
}

function onPointerOver(event: PointerEvent): void {
  const tag = tagFromEvent(event)
  if (!tag || !listHost?.contains(tag)) return
  cancelTimers()
  if (anchor === tag && panelRoot) return
  // already when （ use in ），otherwise etc 
  const delay = anchor ? 0 : SHOW_DELAY
  if (anchor) closePanel()
  pendingTag = tag
  showTimer = window.setTimeout(() => {
    showTimer = 0
    pendingTag = null
    openPanel(tag)
  }, delay)
}

function onPointerOut(event: PointerEvent): void {
  const tag = tagFromEvent(event)
  if (!tag) return
  // ① in （ no 、anchor as null）： that ——
  // 「 1 down 」 not this 。 when pointerout this ：
  // not ， when to will openPanel（ that ）
  if (pendingTag === tag) {
    window.clearTimeout(showTimer)
    showTimer = 0
    pendingTag = null
    return
  }
  // ② and is ： by HIDE_DELAY 
  if (anchor !== tag) return
  window.clearTimeout(showTimer)
  showTimer = 0
  hideTimer = window.setTimeout(() => {
    hideTimer = 0
    if (anchor === tag) closePanel()
  }, HIDE_DELAY)
}

function onFocusIn(event: FocusEvent): void {
  // can ： is button（ requests-panel.js retryCell），
  const tag = tagFromEvent(event)
  if (tag) openPanel(tag)
}

function onFocusOut(event: FocusEvent): void {
  const tag = tagFromEvent(event)
  if (tag && anchor === tag) closePanel()
}

function onKeyDown(event: KeyboardEvent): void {
  // Esc （ not ： need use Esc）。 and old —— in up ，
  // is 「 in 」（， when by Esc 1 in this ）。
  if (event.key !== 'Escape' || !anchor) return
  closePanel()
}

/**
 * ： 1 all， after valid。`entryOf(el)` by ： 1 
 * ，return that request。 not is `data-*`： 1 can 24 、error need 
 * 200 ， 50 is KB —— that will ， in `entries`
 * （ by is when ， and when and ， not in 「 in 、」 ）。
 */
function bind(options?: BindOptions): void {
  const host = options?.host
  if (!host) return
  // down and ： after by need use （ afterListRedraw）
  listHost = host
  entryOfFn = options?.entryOf ?? null
  host.addEventListener('pointerover', onPointerOver)
  host.addEventListener('pointerout', onPointerOut)
  host.addEventListener('focusin', onFocusIn)
  host.addEventListener('focusout', onFocusOut)
  host.addEventListener('keydown', onKeyDown)
}

/* ─── ───────────────────────────────────── */
/**
 * for （ ui/request-hover.js， and 1 ）。 need need ；
 * has —— need ， by up `data-req-hover` ， not not this by HTML。
 */
type RequestHoverApi = {
  /** to up （ 1 ； after not ） */
  bind(options?: BindOptions): void
  /** and */
  close(): void
  /** 「」 this not this —— in ， by （ hasProcessFacts） */
  hasProcessFacts(entry: unknown): boolean
  /** before ： down （ / ） */
  beforeListRedraw(): void
  /** after ： and ， not */
  afterListRedraw(): void
}

declare global {
  interface Window {
    /** request （ ui/request-hover.js， and 1 ） */
    wbRequestHover?: RequestHoverApi
  }
}

window.wbRequestHover = { bind, close: closePanel, hasProcessFacts, beforeListRedraw, afterListRedraw }
