import * as React from 'react'
import { mountOnPage } from '../island-lazy'
import { createRoot } from 'react-dom/client'
import { Badge, Empty, SegmentedControl, type SegmentedControlOption, Card, CardHeader, CardContent, CardFooter } from '@ui'
import {
  RANGES, DEFAULT_RANGE, RANGE_KEY, RANGE_LABEL, RANGE_OPTION_LABEL,
  CUSTOM_RANGE,
  readRange, readCustomRange, saveCustomRange,
  modelColorMap,
  overviewCells, rankRows, rankTotalText, donutView,
  heatmapView, heatLegendItems, heatThresholdsOf, cacheRateCells,
  type CustomRange, type HeatmapView,
  type RankRowView, type DonutView,
  type StatsDay, type StatsGroup, type StatsHour, type StatsSummary,
} from './report-charts'
import { t } from '../i18n'
import { MetricCard, MetricGrid, Meter } from '@ui'
import { CacheTrendChart, UsageDonut } from './report/report-recharts'
import { SourceStatsPanel } from './report/source-stats'
import { ApiKeyUsageDialog } from './report/api-key-detail'
import { ReportDailyTrend } from './report/daily-trend-panel'
import { DateRangeFields } from './date-range-fields'
import { updateCustomRange } from '../../../ui-shared/date-range'



/* ─── ─────────────────────────────────── */

/** /api/scheduled-tasks 1 （ reportAutoRefresh this ） */
type IntervalTask = { id?: string; enabled?: boolean; interval?: number; unit?: string }

/** use to （ bridge.rs statistics that 1 ） */
type StatsBridge = {
  /** GET /api/stats/summary?range=<>[&startDate=&endDate=]（dates in when to ） */
  getStatsSummary(range: string, dates?: CustomRange): Promise<StatsSummary | null | undefined>
  /** new config（scheduledTasks.reportAutoRefresh） */
  getScheduledTasks(): Promise<{ tasks?: IntervalTask[] } | null | undefined>
}

/** need that （wbApp 1 ） */
type AccountLike = { id?: string; provider?: string; edition?: string; editionLabel?: string }

/**
 * window up by its / its 。
 *
 * use 「 + 」 not is declare global Window up ：workbuddyDesktop /
 * wbApp / wbUnits is use ， if declare 1 ， and will not 1 
 * TS2717 —— and when 。file declare wbReport（file）。
 */
type SharedWindow = {
  workbuddyDesktop?: StatsBridge
  wbApp?: {
    /** in 「 use 」 when */
    readonly currentPage?: string
    /** （）： by id */
    getState?: () => { accounts?: { accounts?: AccountLike[] } } | null
  }
  /** directory（labelOf： after label → directory → id） */
  wbProviders?: { labelOf?: (provider: string) => string }
  /** （editionSuffix： / ， and 1 ） */
  wbAccountsModel?: { editionSuffix?: (account: AccountLike) => string }
}

function shared(): SharedWindow {
  return window as unknown as SharedWindow
}

/** errorresponse can can ，， not for 。 */
function errorMessage(error: unknown, depth = 0): string {
  if (typeof error === 'string') return error.trim()
  if (error instanceof Error) return error.message.trim()
  if (!error || typeof error !== 'object' || depth > 3) return ''
  const record = error as Record<string, unknown>
  for (const key of ['message', 'error', 'msg', 'reason']) {
    const text = errorMessage(record[key], depth + 1)
    if (text) return text
  }
  return ''
}

/* ─── ─────────────────────────────────── */

/**
 * new （ms） ，1 seconds = after default
 * （`DEFAULT_REPORT_AUTO_REFRESH_SECONDS`）。
 */
const DEFAULT_AUTO_REFRESH_MS = 1_000

/** to ：SegmentedControl to new need new ， can this */
const RANGE_OPTIONS: readonly SegmentedControlOption<string>[] = RANGES.map(value => ({
  value, label: RANGE_OPTION_LABEL[value],
}))

/**
 * up that （ index.html data-tip， 1 not can 、
 * not can ： 1 is ）。 is `.tip-q` + data-tip has —— and when 
 * 1 ， by desktop-tooltip.tsx MutationObserver React 。
 */
const TIP_ACCOUNTS = t('所选范围内每个账号各消耗了多少 Token，按 Token 用量从多到少排。账号是比提供商更细的一维：同一家可以挂多个账号，转发时按优先级在候选链上依次尝试，所以这里反映的是「具体哪个登录态在出力」。百分比按本区间的总 Token 数算（这一块只看用量，请求次数与成功率在概览卡片里）。「未知账号」是指那批请求没有账号身份：走默认登录态转发（未配置账号列表）时会这样；这一维上线前写出的旧聚合行也没有账号明细，但它们会在升级导入旧数据时按明细自动补算回来 —— 只有明细已被手动清理掉的那几天补不回来，仍留在「未知账号」里，那是确实无从恢复的部分。')
const TIP_PROVIDERS = t('所选范围内每个提供商各消耗了多少 Token，按 Token 用量从多到少排。同一个模型名可能有多家都能提供，网关按设置页「转发路由」的优先级依次尝试，所以占比也反映了实际落到谁身上。百分比按本区间的总 Token 数算（这一块只看用量，请求次数与成功率在概览卡片里）。「未知」是指那批请求没记下承载它的提供商 —— 主要是记录提供商这个功能上线之前的旧明细，其次是请求在选定上游之前就失败了（请求体非法、模型不存在、没有可用账号）。这一维与账号那一维不同：旧明细里根本没有「哪家承载」的信息（早期版本只有单一上游，但那是配置事实而非逐条记录），所以无法回算，旧数据会一直留在「未知」里；此后新产生的请求都会记上具体的提供商。')
const TIP_MODELS = t('所选范围内每个模型各消耗了多少 Token，按用量从多到少排，扇区角度即占比。中心是这一维的合计（等于概览里的总 Token），右侧每行给出该模型的 Token 用量与百分比。扇区内的百分比只在 ≥3% 时标出，更小的会挤成一团 —— 完整读数在右侧列表与悬停气泡里。颜色按用量排名分配：最大的那段是红色，依次往后；同一份报表里换个维度看，配色口径不变。')
const TIP_PROVIDERS_PIE = t('所选范围内每个提供商各消耗了多少 Token，按用量从多到少排，扇区角度即占比。同一个模型名可能有多家都能提供，网关按设置页「转发路由」的优先级依次尝试，所以占比也反映了实际落到谁身上。中心是这一维的合计（等于概览里的总 Token），右侧每行给出该提供商的 Token 用量与百分比。「未知」是指那批请求没记下承载它的提供商 —— 主要是记录提供商这个功能上线之前的旧明细，其次是请求在选定上游之前就失败了。')
const TIP_CACHE_RATES = t('命中率 = 缓存读取 Token / 输入 Token（promptTokens）。窗口越短越贴近「此刻」：上游把缓存读取与输入分开上报时，该比值合法地可能超过 100%，此处不夹取值、如实展示。')
/**
 * when 。 in this ：
 * 「 N 」 is in before N （ is N-1 before to ）， not is 「 before N 」；
 * 「」 is 1 ， and 「」 use 1 。
 */
const TIP_RANGE = t('时间范围。「今天」是本地自然日今天 0 点到现在；「昨天」是昨天一整天（0 点到 24 点）；「近 7 天 / 近 30 天」含今天在内往前数，即今天与之前的 6 / 29 天，不是「不含今天的 7 / 30 天」。')

/* ─── （ 、 and ）────── */

/**
 * new config： by 「 when 」（applyAutoRefresh）， when 1 。
 * not is state：applyAutoRefresh from React use ， and before must can to 。
 */
let autoRefreshMs = DEFAULT_AUTO_REFRESH_MS
/** when false： when not （「」） */
let autoEnabled = true
/**
 * is already from after to config。① 1 ， not request；② 「 when 」 
 * ， 1 to failure use 。
 */
let autoSynced = false
/**
 * is has 1 in in 。 when is setInterval（ not etc up 1 ）， can to 
 * 1 seconds —— 1 response will and after in 1 。load by new （ not will 
 * ），** will 1 **， after ； this 1 up in request when ，
 * by down 1 up （ and logs-panel / requests-panel polling 1 ）。
 */
let polling = false
/** request： and when new 1 （ loadPanel ） */
let seq = 0
/** 1 success to summary：`lastSummary()` （ need 1 ， as 
 * from React use ） */
let summaryValue: StatsSummary | null = null

/** after ： */
type PanelHandle = {
  load(options?: LoadOptions): Promise<StatsSummary | null>
  render(data?: StatsSummary | null): void
  applyAutoConfig(ms: number, enabled: boolean): void
}
let handle: PanelHandle | null = null
/** before to render： after （path use not to ，） */
let pendingRender: { data: StatsSummary | null } | null = null

type LoadOptions = { silent?: boolean }

/* ─── ─────────────────────────────────── */

/** 1 empty / error： and .empty */
function placeholder(text: string, className = 'empty') {
  return <Empty className={className} style={{ padding: '14px 0' }}>{text}</Empty>
}

/** up ： by desktop-tooltip.tsx （ TIP_* ）， this empty */
function TipQ({ text }: { text: string }) {
  return <span className='tip-q' data-tip={text} />
}

/**
 * ： and 「」** 1 **（`pbadge p-<provider>`， by 
 * provider id ， edition 「 / 」 in 1 ，
 * wbAccountsModel.editionSuffix）。
 *
 * is **current**（wbApp ）： no has provider ， after this 1 need 
 * and no has ；「 this 」 use is 。 already delete（ 
 * ） or after 「unknown」 not to ， not to —— empty is 。
 */
function ProviderBadge({ accountId }: { accountId: string }) {
  const account = (shared().wbApp?.getState?.()?.accounts?.accounts || [])
    .find(item => item?.id === accountId)
  const provider = typeof account?.provider === 'string' ? account.provider : ''
  if (!provider) return null
  const label = shared().wbProviders?.labelOf?.(provider) || provider
  const suffix = (account && shared().wbAccountsModel?.editionSuffix?.(account)) || ''
  const text = suffix ? `${label} ${suffix}` : label
  return <Badge variant='outline' shape='tag' className={`pbadge p-${provider}`} title={t('提供商：{text}', { text })}>{text}</Badge>
}

/* ─── 1 ：statistics ────────────────────── */

function OverviewCells({ summary, range }: { summary: StatsSummary; range: string }) {
  const trend = Array.isArray(summary.dailyTrend) ? summary.dailyTrend : []
  return (
    <>
      {overviewCells(summary.overview, range, trend.length).map(cell => (
        <MetricCard className='field' key={cell.key}>
          <div className='label'>{cell.label}</div>
          <div className={cell.mono ? 'value mono' : 'value'}>
            {cell.value}
            {cell.sub ? <div className='sub'>{cell.sub}</div> : null}
          </div>
        </MetricCard>
      ))}
    </>
  )
}

/* ─── ：Top / Top ────── */

/**
 * 1 。`rows === null` no has this 1 （， and 1 ——
 * 1 empty will as ）；empty this 1 this when no has （ to empty）。
 * is Token total， and 1 。
 */
function RankPanel({ panelId, listId, labelId, title, tip, rows, totalText, withBadge, emptyText }: {
  panelId: string; listId: string; labelId: string
  title: string; tip: string
  rows: RankRowView[]
  /** ：Token total（ by use by **** and ， and old 1 ） */
  totalText: string
  /** before 1 （providers no has this 1 ） */
  withBadge: boolean
  emptyText: string
}) {
  return (
    <Card render={<section />} className='panel' id={panelId}>
      <CardHeader className='panel-head'>
        <h2>{title}</h2>
        <TipQ text={tip} />
        <span className='panel-sub' id={labelId}>{totalText}</span>
      </CardHeader>
      <CardContent className='panel-body'>
        <div className='rank-shares' id={listId}>
          {rows.length ? rows.map(row => (
            <div className='rank-row' data-tip={row.tip} key={row.key}>
              <span className='name' title={row.full}>
                {withBadge ? <ProviderBadge accountId={row.id} /> : null}
                {row.name}
              </span>
              <Meter value={Number.parseFloat(row.barWidth) || 0} className='track' trackClassName='h-2.5' aria-label={row.full} />
              <span className='num' title={`${row.tokensText} tokens`}>{row.tokensText}</span>
              <span className='pct'>{row.percentText}</span>
            </div>
          )) : placeholder(emptyText)}
        </div>
      </CardContent>
    </Card>
  )
}

/* ─── ：usage（ / ）──── */

function DonutPanel({ panelId, listId, title, tip, ariaLabel, view, emptyText }: {
  panelId: string; listId: string; title: string; tip: string
  ariaLabel: string
  view: DonutView | null
  emptyText: string
}) {
  return (
    <Card render={<section />} className='panel' id={panelId}>
      <CardHeader className='panel-head'>
        <h2>{title}</h2>
        <TipQ text={tip} />
      </CardHeader>
      <CardContent className='panel-body'>
        <div id={listId}>
          {view ? <UsageDonut view={view} ariaLabel={ariaLabel} /> : placeholder(emptyText)}
        </div>
      </CardContent>
    </Card>
  )
}

/* ─── ： ──────────────────────── */

function HeatmapChart({ view }: { view: HeatmapView }) {
  return (
    <svg className='report-svg' width={view.width} height={view.height}
      viewBox={`0 0 ${view.width} ${view.height}`} role='img' aria-label={t('近 365 天活跃热力图')}>
      {view.months.map(item => (
        <text className='hm-axis' x={item.x} y={10} key={item.key}>{item.text}</text>
      ))}
      {view.weekdays.map(item => (
        <text className='hm-axis' x={item.x} y={item.y} textAnchor='end'
          dominantBaseline='middle' key={item.key}>{item.text}</text>
      ))}
      {view.cells.map(cell => (
        // tabindex="-1"： not Tab （otherwise use need by Tab can to down 1 ），
        // when data-tip —— desktop-tooltip.tsx in ** not ** [tabindex] when 0
        <rect className={`hm-cell l${cell.level}`} x={cell.x} y={cell.y} width={cell.size}
          height={cell.size} rx={2} tabIndex={-1} data-tip={cell.tip} key={cell.key} />
      ))}
    </svg>
  )
}

/* ─── ──────────────────────────── */

/** no when clientWidth is 0， use old widthOf 1 ，
 * etc ResizeObserver to will */
const FALLBACK_WIDTH = 680

/**
 * available。****： will ， if response will 
 * （ old watch(id) ）。 when clientWidth is 0， and use up 1 ，
 * new when 0 → will 1 。
 */
function useContainerWidth(): [React.RefObject<HTMLDivElement | null>, number] {
  const ref = React.useRef<HTMLDivElement | null>(null)
  const [width, setWidth] = React.useState(0)
  React.useLayoutEffect(() => {
    const box = ref.current
    if (!box) return
    let last = box.clientWidth
    if (last) setWidth(last)
    if (typeof ResizeObserver !== 'function') return
    const observer = new ResizeObserver(() => {
      const next = box.clientWidth
      if (!next || next === last) return
      last = next
      setWidth(next)
    })
    observer.observe(box)
    return () => observer.disconnect()
  }, [])
  return [ref, width || FALLBACK_WIDTH]
}

/* ─── new （ by 「 when 」config）──── */

/**
 * when 1 config。
 *
 * Why need not is etc 「 when 」： use can can （ up ），
 * from when —— that not will ， 1 is 。 and logs-panel /
 * requests-panel 1 。
 *
 * failure** not ** as already ， is when （loadPanel ） will 1 —— 
 * failure Reason is 「 after no 」（）， 1 failure use ， use will as 
 * 「 in when no 」。
 */
async function syncAutoRefresh(): Promise<void> {
  if (autoSynced) return
  try {
    const list = await shared().workbuddyDesktop?.getScheduledTasks()
    const task = (list?.tasks || []).find(item => item.id === 'reportAutoRefresh')
    applyAutoRefresh(task || null)
    // requestsuccess（ this 1 not in —— that is after old ， not will has ）
    autoSynced = Array.isArray(list?.tasks)
  } catch (error) {
    // not to use （ applyAutoRefresh）， down 
    console.warn('读取报表自动刷新间隔失败，按默认 1 秒:', error instanceof Error ? error.message : error)
    applyAutoRefresh(null)
  }
}

/**
 * use 「 when 」 new config（used for when ）。
 * `task` = `/api/scheduled-tasks` 1 （`{enabled, interval, unit}`）。
 * null / not when default —— not this as 1 not to config new 。
 */
function applyAutoRefresh(task: IntervalTask | null): void {
  // config already by to ， up already ： not （ is ， that if failure will 
  // use in when ）
  autoSynced = true
  const interval = Number(task?.interval)
  const valid = !!task && typeof task === 'object'
    && Number.isFinite(interval) && interval > 0
    && (task.unit === 'seconds' || task.unit === 'minutes')
  if (!valid) {
    autoEnabled = true
    autoRefreshMs = DEFAULT_AUTO_REFRESH_MS
  } else {
    autoEnabled = task.enabled !== false
    autoRefreshMs = task.unit === 'minutes' ? interval * 60_000 : interval * 1000
  }
  handle?.applyAutoConfig(autoRefreshMs, autoEnabled)
}

/* ─── ───────────────────────────────── */

/**
 * 1 in up 。`error` and `summary` ：failure when summary empty、
 * error（ old renderFailure ）—— up 1 will as is can 。
 */
type PageData = { summary: StatsSummary | null; error: string | null }

const EMPTY_DATA: PageData = { summary: null, error: null }

function ReportPage() {
  const [data, setData] = React.useState<PageData>(EMPTY_DATA)
  const [range, setRange] = React.useState<string>(readRange)
  /** （）： has in 「」 when request */
  const [custom, setCustom] = React.useState<CustomRange>(readCustomRange)
  const [inspectedKey, setInspectedKey] = React.useState<StatsGroup | null>(null)
  /** new config： before already in ， this is new that */
  const [auto, setAuto] = React.useState(() => ({ ms: autoRefreshMs, enabled: autoEnabled }))
  /** （ in / ⇄ M/k）： 1 no ， is 「」，
   * 1 down is when —— not new 1 */
  const [, setUnitsTick] = React.useState(0)

  /** current：（loadPanel）must to new ，state update need etc down 1 ，
   * when when ref and state， 1 ref.current */
  const rangeRef = React.useRef(range)
  /** （ rangeRef ）： 1 1 ，must to for that */
  const customRef = React.useRef(custom)
  /**
   * up 1 ** already use ** that （JSON ）。
   *
   * new is when （「 when 」）， 1 no 
   * （ no has new request、 no has ）—— that when `setData` 1 new for ，React
   * will 1 ：365 、、、 1 ，
   * up not will has 。 this is use that 1 。
   *
   * use JSON not is ： is 1 ， etc 
   * 「 has 」 1 to this （ after 1 ）。 this （ KB）
   * in ms down ， is 。errorempty —— from 「failure」
   * when is 1 ， not this this 。
   */
  const signatureRef = React.useRef('')

  const applyData = React.useCallback((next: PageData, signature: string) => {
    signatureRef.current = signature
    setData(next)
  }, [])

  /* ─── ─────────────────────────────── */

  /**
   * 1 。 not ， is 「 after 1 request」： use when ， use will after 
   * （ in old up ， no ）； this ， new that ，
   * old response 1 。
   */
  const loadPanel = React.useCallback(async (options: LoadOptions = {}): Promise<StatsSummary | null> => {
    // 1 config（ when can can up 「 after no 」failure， 1 return）
    void syncAutoRefresh()
    const token = ++seq
    const requested = rangeRef.current
    // in this request up （ after in range=custom when ；
    // its not ，「 and 」 request）
    const dates = requested === CUSTOM_RANGE ? customRef.current : undefined
    try {
      const api = shared().workbuddyDesktop
      if (!api) throw new Error('后端桥不可用')
      const payload = await api.getStatsSummary(requested, dates)
      if (token !== seq) return summaryValue
      if (!payload || typeof payload !== 'object') throw new Error('后端未返回报表数据')
      summaryValue = payload
      // and up 1 1 not state（ signatureRef ）： to 「 no has 」
      // is ， that 1 not this 
      const signature = JSON.stringify(payload)
      if (signature !== signatureRef.current) {
        applyData({ summary: payload, error: null }, signature)
      }
      return summaryValue
    } catch (error) {
      if (token !== seq) return summaryValue
      const message = errorMessage(error)
      summaryValue = null
      // silent：（ / use ），error in up 
      if (!options.silent) console.warn('读取报表数据失败:', message)
      applyData({ summary: null, error: message || t('未知错误') }, '')
      return null
    }
  }, [applyData])

  /** 1 （ render）： not parameter = （ old can ，React 
   * no no has need ）； null lastSummary ， not （ and old renderAll
   * in summary as empty when 1 ） */
  const renderData = React.useCallback((next?: StatsSummary | null): void => {
    if (next === undefined) return
    summaryValue = next || null
    if (next) applyData({ summary: next, error: null }, JSON.stringify(next))
  }, [applyData])

  /** ： as ，localStorage ； 1 */
  const changeRange = React.useCallback((next: string): void => {
    const value = RANGES.includes(next) ? next : DEFAULT_RANGE
    if (value === rangeRef.current) return
    rangeRef.current = value
    setRange(value)
    try {
      localStorage.setItem(RANGE_KEY, value)
    } catch { /* unavailable when down ， will */ }
    void loadPanel()
  }, [loadPanel])

  /**
   * 1 （ onChange）。
   *
   * ：
   * - `YYYY-MM-DD`：empty when to empty， will request up 
   * parameter（ after 400）—— in old up ；
   * - （start to end） when 1 ** to 1 **， not is or 
   * 400： use when this is 1 ；
   * - has current is 「」 when （ in up not in ）。
   */
  const changeCustomDate = React.useCallback((patch: Partial<CustomRange>): void => {
    const normalized = updateCustomRange(customRef.current, patch)
    if (normalized === customRef.current) return
    customRef.current = normalized
    setCustom(normalized)
    saveCustomRange(normalized)
    if (rangeRef.current === CUSTOM_RANGE) void loadPanel()
  }, [loadPanel])

  /* ─── / ───────────────────────── */

  React.useEffect(() => {
    handle = {
      load: loadPanel,
      render: renderData,
      applyAutoConfig: (ms, enabled) => setAuto({ ms, enabled }),
    }
    // before render （path use not to ： use ）
    if (pendingRender) {
      const queued = pendingRender
      pendingRender = null
      renderData(queued.data)
    }
    // ：app.js showPage in before （ in app.js after ）， that use 
    // not to window.wbReport； use up if in ， this must 1 。 not in not this 1 。
    if (shared().wbApp?.currentPage === 'overview') void loadPanel({ silent: true })
    return () => { handle = null }
  }, [loadPanel, renderData])

  /**
   * when ：config 1 （ old startAuto stopAuto）， when 。 before 
   * 1 not can ： already 、 as 、 can when has 。 when not when （ not is 
   * 1 not ），otherwise「 when in 」 will 「 no 」。
   */
  React.useEffect(() => {
    if (!auto.enabled || auto.ms <= 0) return
    const timer = window.setInterval(() => {
      // in can when ， after nonerequest
      if (document.hidden || shared().wbApp?.currentPage !== 'overview') return
      // up 1 no this 1 （ polling ）
      if (polling) return
      polling = true
      void loadPanel({ silent: true }).finally(() => { polling = false })
    }, auto.ms)
    return () => { window.clearInterval(timer) }
  }, [auto, loadPanel])

  /** （ setUnitsTick ） */
  React.useEffect(() => {
    const onChange = () => setUnitsTick(tick => tick + 1)
    window.addEventListener('wb-units-changed', onChange)
    return () => window.removeEventListener('wb-units-changed', onChange)
  }, [])

  /* ─── ─────────────────────────────── */

  const heatWidth = useContainerWidth()

  const { summary, error } = data
  // use after （ and this 1 ）， after no 
  const rangeKey = summary && RANGES.includes(summary.range ?? '') ? (summary.range as string) : range

  const heatmapDays: StatsDay[] = summary && Array.isArray(summary.heatmap) ? summary.heatmap : []
  const cacheTrendHours: StatsHour[] = summary && Array.isArray(summary.cacheTrend24h) ? summary.cacheTrend24h : []
  // no has this 1 （undefined）；empty is 「 this 1 no 」（ to empty）
  const providerList: StatsGroup[] | null = summary && Array.isArray(summary.providers) ? summary.providers : null
  const accountList: StatsGroup[] | null = summary && Array.isArray(summary.accounts) ? summary.accounts : null
  const modelList: StatsGroup[] | null = summary && Array.isArray(summary.models) ? summary.models : null

  /** ：error / in / */
  const errorBlock = error ? placeholder(t('读取报表失败：{error}', { error }), 'empty report-error') : null
  const loadingBlock = placeholder(t('正在加载…'))
  const loading = errorBlock ?? loadingBlock

  const modelColors = React.useMemo(() => modelColorMap(modelList || []), [modelList])
  const heatView = summary ? heatmapView(heatmapDays, heatWidth[1]) : null

  return (
    <>
      {/* ── statistics ──
            when in this 1 ： is this 1 statistics， and 
            1 。 in 「」 when （） */}
      <Card render={<section />} className='panel'>
        <CardHeader className='panel-head'>
          <h2>{t('统计概览')}</h2>
          {/* 「 N 」 is ， has （ = before N-1 ）。
               not is ： already has 7 ， after will */}
          <TipQ text={TIP_RANGE} />
          <span className='panel-sub' id='report-range-label'>{RANGE_LABEL[rangeKey] || ''}</span>
          <div className='head-actions'>
            <SegmentedControl options={RANGE_OPTIONS} value={range}
              onValueChange={changeRange} aria-label={t('报表时间范围')} />
            {range === CUSTOM_RANGE ? <DateRangeFields dates={custom} onChange={changeCustomDate} /> : null}
          </div>
        </CardHeader>
        <CardContent className='panel-body'>
          <MetricGrid className='field-grid' id='report-overview'>
            {summary ? <OverviewCells summary={summary} range={rangeKey} /> : loading}
          </MetricGrid>
        </CardContent>
      </Card>

      {/* ── Top / Top （ and ，）── */}
      <div className='report-rank-grid'>
        {accountList ? (
          <RankPanel panelId='report-accounts-panel' listId='report-accounts' labelId='report-accounts-label'
            title={t('Top 账号')} tip={TIP_ACCOUNTS} rows={rankRows(accountList) || []}
            totalText={rankTotalText(accountList)} withBadge emptyText={t('所选范围内还没有账号用量')} />
        ) : null}
        {providerList ? (
          <RankPanel panelId='report-providers-panel' listId='report-providers' labelId='report-providers-label'
            title={t('Top 提供商')} tip={TIP_PROVIDERS} rows={rankRows(providerList) || []}
            totalText={rankTotalText(providerList)} withBadge={false} emptyText={t('所选范围内还没有请求记录')} />
        ) : null}
      </div>

      {/* ── usage： / and （）── */}
      <div className='report-donut-grid'>
        {modelList ? (
          <DonutPanel panelId='report-models-panel' listId='report-models-donut' title={t('模型用量')}
            tip={TIP_MODELS} ariaLabel={t('模型用量占比')}
            view={donutView(modelList, t('未知模型'))} emptyText={t('所选范围内还没有模型用量')} />
        ) : null}
        {providerList ? (
          <DonutPanel panelId='report-providers-pie-panel' listId='report-providers-donut' title={t('提供商用')}
            tip={TIP_PROVIDERS_PIE} ariaLabel={t('提供商用占比')}
            view={donutView(providerList, t('未知'))} emptyText={t('所选范围内还没有提供商用量')} />
        ) : null}
      </div>

      {/* ── （ 365 ， and when ）── */}
      <Card render={<section />} className='panel'>
        <CardHeader className='panel-head'>
          <h2>{t('活跃热力图')}</h2>
          <span className='panel-sub'>{t('近 365 天，按天 Token 用量着色（档位按本窗口峰值自动定）')}</span>
        </CardHeader>
        <CardContent className='panel-body'>
          <div className='heat-wrap' id='report-heatmap' ref={heatWidth[0]}>
            {summary
              ? (heatView ? <HeatmapChart view={heatView} /> : placeholder(t('暂无热力图数据')))
              : loading}
          </div>
        </CardContent>
        <CardFooter className='panel-foot'>
          <span className='heat-scale-label'>{t('Token')}</span>
          {/* and use 1 ：， need new ，
              otherwise up will in up 1 up */}
          <span className='heat-legend' id='report-heat-legend'>
            {summary ? heatLegendItems(heatThresholdsOf(heatmapDays)).map(item => (
              <span className='heat-legend-item' key={item.level}>
                <span className={`l${item.level}`} />{item.text}
              </span>
            )) : null}
          </span>
          <div className='spacer' />
          <span>{t('方格为本地自然日，无请求显示为空槽')}</span>
        </CardFooter>
      </Card>

      {/* ── in ── */}
      <Card render={<section />} className='panel'>
        <CardHeader className='panel-head'>
          <h2>{t('缓存命中率')}</h2>
          <TipQ text={TIP_CACHE_RATES} />
        </CardHeader>
        <CardContent className='panel-body'>
          <MetricGrid className='field-grid' id='report-cache-rates'>
            {summary ? cacheRateCells(summary.cacheRates).map(cell => (
              <MetricCard className='field rate' key={cell.key}>
                <div className='label'>{cell.label}</div>
                <div className='value big'>
                  {cell.value}
                  <div className='sub'>{cell.sub}</div>
                </div>
              </MetricCard>
            )) : loading}
          </MetricGrid>
        </CardContent>
      </Card>

      {/* ── 24 when in （ 24 ）── */}
      <Card render={<section />} className='panel'>
        <CardHeader className='panel-head'>
          <h2>{t('近 24 小时缓存命中率趋势')}</h2>
          <span className='panel-sub'>{t('按本地整点，无请求的整点不标注')}</span>
        </CardHeader>
        <CardContent className='panel-body'>
          <div className='chart-wrap' id='report-cache-trend'>
            {summary ? <CacheTrendChart series={cacheTrendHours} colors={modelColors} /> : loading}
          </div>
        </CardContent>
        {/* ： 1 （= in ，=total Token）， is 「」
             1 ，must */}
        <CardFooter className='panel-foot'>
          <span className='chart-legend'>
            <span className='swatch rate' />{t('缓存命中率')}<span className='axis-hint'>{t('左轴')}</span>
          </span>
          <span className='chart-legend'>
            <span className='swatch tokens' />{t('总 Token')}<span className='axis-hint'>{t('右轴')}</span>
          </span>
          <div className='spacer' />
          <span>{t('无请求的整点按 0% 计，但不写读数')}</span>
        </CardFooter>
      </Card>

      <ReportDailyTrend summary={summary} loading={loading} />

      {summary?.clients ? <SourceStatsPanel kind='clients' rows={summary.clients} summary={summary} /> : null}
      {summary?.apiKeys ? <SourceStatsPanel kind='apiKeys' rows={summary.apiKeys} summary={summary} onInspect={setInspectedKey} /> : null}
      {inspectedKey ? <ApiKeyUsageDialog target={inspectedKey} range={range}
        dates={range === CUSTOM_RANGE ? custom : undefined} onClose={() => setInspectedKey(null)} /> : null}
    </>
  )
}

/* ─── for （ use file）──────────────────── */

/** app.js to / upgrade-panel after 1 */
async function load(options: LoadOptions = {}): Promise<StatsSummary | null> {
  // before use not ： when will 1 （ and old etc ）
  if (!handle) return summaryValue
  return handle.load(options)
}

/** 1 （ old can ，；current no has use ） */
function render(data?: StatsSummary | null): void {
  if (data === undefined) return
  if (!handle) {
    pendingRender = { data: data ?? null }
    return
  }
  handle.render(data)
}

/** 1 success to summary（ old ，） */
function lastSummary(): StatsSummary | null {
  return summaryValue
}

declare global {
  interface Window {
    /** （ ui/report.js， and 1 ） */
    wbReport?: {
      /** to / after new （app.js:117、upgrade-panel.js:73） */
      load(options?: LoadOptions): Promise<StatsSummary | null>
      /** 1 （ old can ） */
      render(data?: StatsSummary | null): void
      /** 1 success to summary */
      lastSummary(): StatsSummary | null
      /** 「 when 」 after （tasks-panel.tsx:323） */
      applyAutoRefresh(task: IntervalTask | null): void
    }
  }
}

window.wbReport = { load, render, lastSummary, applyAutoRefresh }

/* ─── ： index.html has ─────── */

const PAGE_SELECTOR = '.page[data-page="overview"]'

let mounted = false

/**
 * React root in up （ not div， by file）。 
 * （.panel / .panel-head / .field-grid …）： down by new ， will and React 。
 */
function mount(): void {
  if (mounted) return
  const section = document.querySelector<HTMLElement>(PAGE_SELECTOR)
  if (!section) return
  mounted = true
  section.replaceChildren()
  createRoot(section).render(<ReportPage />)
}

// to when （ island-lazy）： not in ， not 。
mountOnPage('overview', PAGE_SELECTOR, mount)

// new config not in when ： 1 to when loadPanel will 1 
// （ loadPanel syncAutoRefresh）， no not this this request。
