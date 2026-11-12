import * as React from 'react'
import { createRoot } from 'react-dom/client'
import {
  Button,
  Dialog,
  DialogBody,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  RadioGroup,
  RadioGroupItem,
  Label } from '@ui'
import { t } from '../i18n'

/**
 * request 「」（ that 「」 by ）。
 *
 * is ui/request-clear-modal.js —— that use innerHTML .modal-mask /
 * .clear-mode that 。 for and ** 1 **：
 * `window.wbRequestClearModal.open()`（noneparameter，）， use 
 * （requests-panel.js 「」 by ） 1 not use 。 in ： 
 * Dialog 1 （Esc / 、、 by ），
 * RadioGroup + RadioGroupItem， by Tailwind ， not ui/css 
 * .modal-* and .clear-mode 1 。
 *
 * OmniProxy ClearLogsModal：
 * · ——「delete」（mode=all，default）/「onlyempty」
 * （mode=raw）； （ not statistics、 not ） in ，
 * use in that 1 to after ；
 * · when by current clear-preview （ will / / use ）；
 * · 「」 is （ not ，）： after VACUUM，
 * before 3 seconds vacuumRunning to ；
 * · 「」 wbConfirm ，DELETE mode and 。
 *
 * ── （ ， already ， stats_api.rs module header）─────
 *   GET    /api/stats/requests/clear-preview   {all, raw, upstreamRaw, downstreamRaw, dbBytes, vacuumRunning}
 * POST /api/stats/requests/compact （ 409）
 *   DELETE /api/stats/requests?mode=raw|all    {deleted, raw, upstreamRaw, downstreamRaw}
 *
 * ── parameterWhy wbRequestsPanel need ───────────────────────
 * 「 N 」 and 「 that 」must is 1 ， use 
 * `wbRequestsPanel.clearParams()` —— and GET 1 （filterParams）。
 * after GET / DELETE / clear-preview by use 1 
 *（stats_api::filter_from_params）， before not 1 。
 *
 * by need create、（ and conc-dialog.tsx 1 ）： not index.html
 * empty。
 */

/* ─── and ─────────────────────────────── */

type Mode = 'all' | 'raw'

/**
 * 。 and after for （ stats_api.rs 
 * clear_stats_requests）：「 by not 」 is delete ——
 * is ，none by ；empty then 1 。
 * and old MODES 1 （ old esc() after innerHTML，
 * this to React ， not ）。
 * as t() （ in ）： not is ，。
 */
const MODES: Record<Mode, { title: string; desc: string }> = {
  all: {
    title: t('全部删除'),
    desc: t('删除请求记录，并同步清理对应的上游调试报文与下游正文。有筛选时只删除命中的记录，按天聚合报表不受影响；无筛选时清空全部并重置报表。此操作无法恢复。'),
  },
  raw: {
    title: t('仅清空报文原文'),
    desc: t('保留请求记录、用量统计与报表，同时清理上游调试报文和下游请求 / 响应正文。清理后「预览对话」与「原始报文」不再保留原文。有筛选时只清理命中请求的报文；无筛选时清空两侧全部报文。此操作无法恢复。'),
  },
}

/** ：default in 「delete」（ not when use ；raw is ，default in ） */
const MODE_ORDER: readonly Mode[] = ['all', 'raw']

/** ：3 seconds 1 （ OmniProxy ） */
const POLL_MS = 3_000

/** clear-preview response； by unknown ， 1 to toCounts（ after can can is ） */
type ClearPreviewResponse = {
  all?: unknown
  raw?: unknown
  downstreamRaw?: unknown
  upstreamRaw?: unknown
  dbBytes?: unknown
  vacuumRunning?: unknown
}

type ClearResponse = {
  deleted?: unknown
  downstreamRaw?: unknown
  upstreamRaw?: unknown
}

/** 1 after */
type Counts = {
  all: number
  raw: number
  downstreamRaw: number | null
  upstreamRaw: number | null
  dbBytes: number
  vacuumRunning: boolean
}

/** or when ， not 。 */
type PreviewState =
  | { status: 'loading' }
  | { status: 'failed' }
  | ({ status: 'ready' } & Omit<Counts, 'vacuumRunning'>)

/** after ： use to this */
type ClearBridge = {
  /** GET /api/stats/requests/clear-preview */
  getStatsClearPreview(query: string): Promise<ClearPreviewResponse | null | undefined>
  /** POST /api/stats/requests/compact（ 409 / in error） */
  compactStatsDb(): Promise<unknown>
  /** DELETE /api/stats/requests?mode=raw|all */
  clearStatsRequests(query: string): Promise<ClearResponse | null | undefined>
  getStorage(): Promise<unknown>
  getDebug(): Promise<unknown>
}

/**
 * window up by its / its 。
 *
 * use 「 + 」 not is declare global Window up ：
 * workbuddyDesktop / wbApp / wbRequestsPanel is use ， if 
 * declare 1 ， and will not 1 TS2717 —— and when 
 * 。file wbRequestClearModal（file）。
 */
type SharedWindow = {
  workbuddyDesktop?: ClearBridge
  wbApp?: { toast?: (message: string, kind?: 'err' | 'ok') => void }
  wbConfirm?: {
    ask?: (options: {
      title?: string
      /** ， <strong> etc ； by use */
      html?: string
      okText?: string
      /** danger = not can （） */
      okClass?: string
    }) => Promise<boolean>
  }
  wbSettingsPanel?: {
    renderStorage?: (data: unknown) => void
    renderDebug?: (data: unknown) => void
  }
  wbRequestsPanel?: {
    /** current parameter（， and GET ） */
    clearParams?: () => unknown
    /** after new （ + 1 ） */
    notifyCleared?: () => unknown
    /** current */
    load?: (options?: { silent?: boolean }) => unknown
  }
}

function shared(): SharedWindow {
  return window as unknown as SharedWindow
}

/* ─── and ────────────────────────── */

/**
 * request in （， not is state）： closeModal() must can 
 * **** to —— Esc / in ， if in state ，
 * update is 、 1 will 。 in when not ：DELETE already ， will 
 * 「 to no 」unknown。
 */
let clearing = false

/** / toast 1 ：toast seconds after no ， 1 */
function toast(message: string, kind?: 'err' | 'ok') {
  shared().wbApp?.toast?.(message, kind)
}

/** can ：B → KB（1 ）→ MB（1 ）→ GB（2 ）。 not */
function formatBytes(bytes: unknown): string {
  const value = Math.max(0, Number(bytes) || 0)
  if (value < 1024) return `${Math.round(value)} B`
  const kb = value / 1024
  if (kb < 1024) return `${kb.toFixed(1)} KB`
  const mb = kb / 1024
  if (mb < 1024) return `${mb.toFixed(1)} MB`
  return `${(mb / 1024).toFixed(2)} GB`
}

/**
 * current parameter（， and GET ）。
 * / return not is when by 「none」 —— that down is 
 * ， after has all=1 ， not will 。
 */
function currentQuery(): string {
  const query = shared().wbRequestsPanel?.clearParams?.()
  return typeof query === 'string' ? query : ''
}

function countOrNull(value: unknown): number | null {
  if (value === null || value === undefined || value === '') return null
  const count = Number(value)
  return Number.isSafeInteger(count) && count >= 0 ? count : null
}

function toCounts(preview: ClearPreviewResponse | null | undefined): Counts {
  const all = countOrNull(preview?.all)
  const raw = countOrNull(preview?.raw)
  if (all === null || raw === null) throw new Error(t('清理预览数据不完整'))
  return {
    all,
    raw,
    downstreamRaw: countOrNull(preview?.downstreamRaw),
    upstreamRaw: countOrNull(preview?.upstreamRaw),
    dbBytes: Number(preview?.dbBytes) || 0,
    vacuumRunning: preview?.vacuumRunning === true,
  }
}

async function refreshStorageCounts() {
  const api = shared().workbuddyDesktop
  const panel = shared().wbSettingsPanel
  if (!api || !panel) return
  await Promise.all([
    api.getStorage().then(data => panel.renderStorage?.(data)),
    api.getDebug().then(data => panel.renderDebug?.(data)),
  ]).catch(error => console.warn('刷新报文存储统计失败:', errorMessage(error)))
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}

/** is HTML ， 1 （ and app.js esc ） */
function escapeHtml(value: string): string {
  return value.replace(/[&<>"']/g, ch => ({
    '&': '&amp;',
    '<': '&lt;',
    '>': '&gt;',
    '"': '&quot;',
    "'": '&#39;',
  }[ch] ?? ch))
}

/* ─── ───────────────────────────────── */

type RequestClearModalProps = {
  /** （ by ； in when will ） */
  onClose: () => void
}

function RequestClearModal({ onClose }: RequestClearModalProps) {
  const [mode, setMode] = React.useState<Mode>('all')
  const [preview, setPreview] = React.useState<PreviewState>({ status: 'loading' })
  const [hint, setHint] = React.useState('')
  const [vacuumRunning, setVacuumRunning] = React.useState(false)
  /** in ****（ by 「 in …」 and use ）； use is clearing */
  const [clearingView, setClearingView] = React.useState(false)
  const queryRef = React.useRef(currentQuery())
  const canClear = preview.status === 'ready'
    && preview.downstreamRaw !== null && preview.upstreamRaw !== null
    && (mode !== 'raw' || preview.raw > 0)

  /** ：await use ， response 1 */
  const aliveRef = React.useRef(true)
  /** when ；null = no in */
  const timerRef = React.useRef<number | null>(null)

  /**
   * in need ： clearing to closeModal() （），
   * state by and use 。 1 ，。
   */
  const setInFlight = React.useCallback((next: boolean) => {
    clearing = next
    setClearingView(next)
  }, [])

  const stopPolling = React.useCallback(() => {
    if (timerRef.current !== null) {
      window.clearInterval(timerRef.current)
      timerRef.current = null
    }
  }, [])

  /** 1 and by */
  const applyPreview = React.useCallback((next: Counts) => {
    const { vacuumRunning: running, ...counts } = next
    setPreview({ status: 'ready', ...counts })
    setVacuumRunning(running)
  }, [])

  /**
   * 1 ： 1 ；vacuumRunning false 。
   * failure、 down 1 （ and OmniProxy 1 ）—— in after ，
   * 1 not to not this 「 in 」 failure。
   */
  const pollOnce = React.useCallback(async () => {
    const api = shared().workbuddyDesktop
    if (!api) return
    let next: Counts
    try {
      next = toCounts(await api.getStatsClearPreview(queryRef.current))
    } catch {
      return
    }
    if (!aliveRef.current) return
    applyPreview(next)
    if (next.vacuumRunning) return
    stopPolling()
    setHint('')
    toast(t('✅ 压缩完成'))
    // not ， use ： up that responseupdate，
    // 1 （vacuum can can new request）
    void shared().wbRequestsPanel?.load?.({ silent: true })
    void refreshStorageCounts()
  }, [applyPreview, stopPolling])

  /** ： already in up （ not ），otherwise by 「 in …」 */
  const startPolling = React.useCallback(() => {
    if (timerRef.current !== null) return
    setVacuumRunning(true)
    timerRef.current = window.setInterval(() => {
      void pollOnce()
    }, POLL_MS)
  }, [pollOnce])

  /**
   * when 1 。success；failure 「failure」、
   * 1 error（toast not in this —— ， use no ）。
   */
  async function refreshPreview() {
    setPreview({ status: 'loading' })
    try {
      const api = shared().workbuddyDesktop
      if (!api) throw new Error(t('后端桥不可用'))
      const raw = await api.getStatsClearPreview(queryRef.current)
      if (!aliveRef.current) return // 等待期间已关窗：丢弃这次结果
      const next = toCounts(raw)
      applyPreview(next)
      // when in （ / up no ）： up 
      if (next.vacuumRunning) startPolling()
    } catch (error) {
      if (!aliveRef.current) return
      setPreview({ status: 'failed' })
      setHint(t('预览读取失败：{error}', { error: errorMessage(error) }))
    }
  }

  // （ old open() ）
  React.useEffect(() => {
    aliveRef.current = true
    void refreshPreview()
    return () => {
      aliveRef.current = false
      // ： when will 1 after （ old is when stopVacuumPolling）
      stopPolling()
      // in need 。path by setInFlight(false) ，
      // 「 in when 」（openModal unmount up 1 ）—— that down 
      // will in true， after closeModal and handleExecute ， not 
      // not 。 when none， new from 。
      clearing = false
    }
    // in when 1 ；stopPolling / applyPreview / startPolling is empty use 
  }, [])

  /**
   * ：POST compact 「 is 」， clear-preview 
   * vacuumRunning 。409（ already has 1 in ） not failure —— up ，
   * use to and 「success」 1 ， is 1 。
   */
  async function handleCompact() {
    const api = shared().workbuddyDesktop
    if (!api) return
    setVacuumRunning(true) // 先切「压缩中…」并禁用（旧实现同序）
    try {
      await api.compactStatsDb()
      setHint(t('压缩已在后台开始，完成后会自动提示'))
      startPolling()
    } catch (error) {
      const message = errorMessage(error)
      // 409 = 1 in ： after 「 in in …」，
      // this （ not ）， 1 down 
      if (/409|正在进行/.test(message)) {
        toast(t('压缩已在进行中'))
        setHint(t('压缩已在进行中，接上它的进度等待完成'))
        startPolling()
        return
      }
      setVacuumRunning(false)
      setHint(t('压缩失败：{message}', { message }))
      toast(t('压缩失败：{message}', { message }), 'err')
    }
  }

  /**
   * after 。DELETE parameter（****）：
   * · `mode` （ after is all，，request 1 ）；
   * · = in （`mode=all&<parameter>`）；
   * · not = empty， mode must `all=1`（ after ，
   * stats_api::clear_stats_requests）—— parameter is not can ，
   * this ， not after that 400 。
   */
  async function handleExecute() {
    if (clearing || !canClear || preview.status !== 'ready') return
    const api = shared().workbuddyDesktop
    const ask = shared().wbConfirm?.ask
    if (!api || !ask) return

    const label = MODES[mode].title
    const query = queryRef.current
    const hasFilters = query.length > 0
    const scope = hasFilters ? t('当前筛选范围') : t('全部请求数据')
    const counts = {
      count: preview.all,
      upstream: preview.upstreamRaw ?? 0,
      downstream: preview.downstreamRaw ?? 0,
    }
    const summary = mode === 'raw'
      ? t('保留 {count} 条请求记录；将清理上游调试报文 {upstream} 条、下游正文 {downstream} 条。用量统计与报表不变。', counts)
      : t('将删除 {count} 条请求记录，并清理上游调试报文 {upstream} 条、下游正文 {downstream} 条。', counts)
    const confirmed = await ask({
      title: t('清理请求日志'),
      html: t('确定对<strong>{scope}</strong>执行「<strong>{label}</strong>」？<br>{summary}<br>此操作无法恢复。', {
        scope: escapeHtml(scope), label: escapeHtml(label), summary: escapeHtml(summary),
      }),
      okText: mode === 'raw' ? t('清空两侧原文') : t('删除'),
      okClass: 'danger',
    })
    if (!confirmed || clearing || !aliveRef.current) return

    setInFlight(true)
    try {
      const requestQuery = hasFilters ? `mode=${mode}&${query}` : `mode=${mode}&all=1`
      const result = await api.clearStatsRequests(requestQuery)
      const deleted = countOrNull(result?.deleted)
      const upstream = countOrNull(result?.upstreamRaw)
      const downstream = countOrNull(result?.downstreamRaw)
      if (deleted === null || upstream === null || downstream === null) {
        toast(t('清理请求已返回，但结果数据不完整，请刷新后确认。'), 'err')
      } else {
        toast(mode === 'raw'
          ? t('已清理上游调试报文 {upstream} 条、下游正文 {downstream} 条；请求记录与报表已保留。', { upstream, downstream })
          : t('已删除 {count} 条请求记录，并清理上游调试报文 {upstream} 条、下游正文 {downstream} 条。', { count: deleted, upstream, downstream }))
      }
      setInFlight(false) // 先解除守卫，closeModal() 才放行（与旧实现同序）
      onClose()
      // new ： can can （ after down not this old ），
      // by notifyCleared 1 and （ + 1 ）
      void shared().wbRequestsPanel?.notifyCleared?.()
      void refreshStorageCounts()
    } catch (error) {
      const message = errorMessage(error)
      setHint(t('清理失败：{message}', { message }))
      toast(t('清理失败：{message}', { message }), 'err')
      // failuremust，otherwise not （ by and 1 and available）
      setInFlight(false)
    }
  }

  const previewText = preview.status === 'ready'
    ? preview.downstreamRaw === null || preview.upstreamRaw === null
      ? t('当前网关未提供上游 / 下游清理统计，请更新网关后再清理。')
      : mode === 'raw'
        ? t('保留 {count} 条请求记录 · 将清理上游调试报文 {upstream} 条、下游正文 {downstream} 条 · 数据库占用 {db}', {
            count: preview.all, upstream: preview.upstreamRaw, downstream: preview.downstreamRaw,
            db: formatBytes(preview.dbBytes),
          })
        : t('将删除 {count} 条请求记录 · 同步清理上游调试报文 {upstream} 条、下游正文 {downstream} 条 · 数据库占用 {db}', {
            count: preview.all, upstream: preview.upstreamRaw, downstream: preview.downstreamRaw,
            db: formatBytes(preview.dbBytes),
          })
    : preview.status === 'failed'
      ? t('预览读取失败')
      : t('正在统计…')

  return (
    <Dialog
      open
      onOpenChange={(next, eventDetails) => {
        // request（Esc / / up ✕） to this 。
        if (next) return
        // in when ：must eventDetails.cancel()，「 not update open prop」
        // is not —— Base UI store and prop ，ignore will 
        // open false。 will 「 to no 」unknown。
        if (clearing) {
          eventDetails.cancel()
          return
        }
        onClose()
      }}
    >
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t('清理请求日志')}</DialogTitle>
        </DialogHeader>
        <DialogBody>
          <RadioGroup
            aria-label={t('清理方式')}
            value={mode}
            onValueChange={next => {
              if (!clearing) setMode(next === 'raw' ? 'raw' : 'all')
            }}
          >
            {MODE_ORDER.map(key => {
              const checked = mode === key
              return (
                <Label
                  key={key}
                  // in by value ， to data-checked 
                  // （ and data-checked:， group-data-checked/card:）。
                  // in when ** not this **（ not is data-checked="false"）：
                  // Tailwind data-checked: is 「 in in 」， false
                  // will in to in ； this and Base UI 1 
                  //（ use data-checked="" / data-unchecked="" ）。
                  data-checked={checked ? '' : undefined}
                  className='group/card grid cursor-pointer grid-cols-[auto_minmax(0,1fr)] items-center gap-x-2.5 gap-y-0.5 rounded-md border border-border px-3 py-2.5 transition-colors duration-150 ease-out hover:border-border-strong data-checked:border-primary data-checked:bg-primary-soft'
                >
                  {/* 、 to in ； and in up down */}
                  <RadioGroupItem value={key} className='row-span-2 self-center' />
                  <span className='text-[13px] font-semibold text-foreground group-data-checked/card:text-primary-fg'>
                    {MODES[key].title}
                  </span>
                  <span className='text-[11.5px] leading-[1.55] text-subtle'>{MODES[key].desc}</span>
                </Label>
              )
            })}
          </RadioGroup>
          <p className='text-[11.5px] text-muted-foreground'>{previewText}</p>
          <p className='text-[11.5px] text-muted-foreground'>
            {t('数据库占用包含主文件与 WAL 文件；清理后可点击「压缩数据库」回收空闲空间。')}
          </p>
        </DialogBody>
        <DialogFooter>
          <span className='min-w-0 text-[11.5px] text-muted-foreground'>{hint}</span>
          <div className='mr-auto' />
          <Button
            variant='outline'
            size='sm'
            onClick={() => {
              void handleCompact()
            }}
            disabled={vacuumRunning}
            title={
              vacuumRunning
                ? t('压缩正在后台执行，完成后自动恢复')
                : t('回收已删除数据占用的磁盘空间（checkpoint + VACUUM，后台执行）')
            }
          >
            {vacuumRunning ? t('压缩中…') : t('压缩数据库')}
          </Button>
          <Button variant='outline' onClick={onClose} disabled={clearingView}>
            {t('取消')}
          </Button>
          <Button
            variant='destructive'
            onClick={() => {
              void handleExecute()
            }}
            disabled={clearingView || !canClear}
          >
            {clearingView ? t('清理中…') : t('执行清理')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

/* ─── ： and old window.wbRequestClearModal 1 ─── */

let root: ReturnType<typeof createRoot> | null = null
let host: HTMLElement | null = null

function unmountModal() {
  if (root) {
    root.unmount()
    root = null
  }
  if (host) {
    host.remove()
    host = null
  }
}

/** ： in when not response（ clearing ） */
function closeModal() {
  if (clearing) return
  // 1 and （ effect ）， this not 
  unmountModal()
}

/** ： by need ；default「delete」， by when */
function openModal() {
  unmountModal() // 重复打开先拆掉上一份（旧实现同样是 close() 打头）
  host = document.createElement('div')
  document.body.append(host)
  root = createRoot(host)
  root.render(<RequestClearModal onClose={closeModal} />)
}

declare global {
  interface Window {
    /** request「」（ ui/request-clear-modal.js， and 1 ） */
    wbRequestClearModal?: { open(): void }
  }
}

window.wbRequestClearModal = { open: openModal }
