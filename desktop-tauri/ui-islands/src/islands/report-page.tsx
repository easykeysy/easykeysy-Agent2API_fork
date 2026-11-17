import * as React from 'react'
import { mountOnPage } from '../island-lazy'
import { createRoot } from 'react-dom/client'
import { Badge, Empty, SegmentedControl, type SegmentedControlOption, PageHeader, PageTitle, PageDescription } from '@ui'
import {
  RANGES, DEFAULT_RANGE, RANGE_KEY, RANGE_LABEL, RANGE_OPTION_LABEL,
  CUSTOM_RANGE,
  readRange, readCustomRange, saveCustomRange,
  modelColorMap,
  type CustomRange, type StatsGroup, type StatsSummary,
} from './report-charts'
import { t } from '../i18n'
import { ReportOverview } from './report/overview-panel'
import { RankPanel } from './report/rank-panel'
import { ActivityPanel } from './report/activity-panel'
import { CacheWindows, CacheTrendPanel } from './report/cache-panels'
import { ReportUsageDonuts } from './report/usage-donuts'
import { SourceStatsPanel } from './report/source-stats'
import { ApiKeyUsageDialog } from './report/api-key-detail'
import { ReportDailyTrend } from './report/daily-trend-panel'
import { DateRangeFields } from './date-range-fields'
import { updateCustomRange } from '../../../ui-shared/date-range'
import { PageAutoRefresh, pageRefreshStore, usePageAutoRefresh } from './page-auto-refresh'



/* ─── 类型 ─────────────────────────────────── */

/** /api/scheduled-tasks 里的一条（本页只关心 reportAutoRefresh 这条的形状） */
type IntervalTask = { id?: string; enabled?: boolean; interval?: number; unit?: string }

/** 本页用到的壳侧接口（见 bridge.rs 的统计那一段） */
type StatsBridge = {
  /** GET /api/stats/summary?range=<档位>[&startDate=&endDate=]（dates 只在自定义区间时给） */
  getStatsSummary(range: string, dates?: CustomRange): Promise<StatsSummary | null | undefined>
}

/** 账号行徽章要读的那点账号信息（wbApp 状态里的一条） */
type AccountLike = { id?: string; provider?: string; edition?: string; editionLabel?: string }

/**
 * window 上由其它脚本 / 其它岛挂载的共享桥。
 *
 * 刻意用「局部窄类型 + 转型」而不是 declare global 往 Window 上加属性：workbuddyDesktop /
 * wbApp / wbUnits 是多个岛共用的桥，若每个岛各 declare 一份，接口合并会因同名属性类型不一致
 * 直接报 TS2717 —— 并行迁移时必然互相撞车。本文件只 declare 自己独占的 wbReport（见文件末尾）。
 */
type SharedWindow = {
  workbuddyDesktop?: StatsBridge
  wbApp?: {
    /** 本页只在「用户正看着报表页」时轮询 */
    readonly currentPage?: string
    /** 主状态（账号表）：排行卡里的提供商徽章按账号 id 现查归属 */
    getState?: () => { accounts?: { accounts?: AccountLike[] } } | null
  }
  /** 提供商展示名目录（labelOf：后端 label → 目录 → 原样回显 id） */
  wbProviders?: { labelOf?: (provider: string) => string }
  /** 账号展示模型（editionSuffix：国际版 / 国内版的判定，与账号页同一处口径） */
  wbAccountsModel?: { editionSuffix?: (account: AccountLike) => string }
}

function shared(): SharedWindow {
  return window as unknown as SharedWindow
}

/** 错误响应可能包含凭据，只取文字字段，不序列化整个对象。 */
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

/* ─── 常量 ─────────────────────────────────── */

/** 选项提到模块级：SegmentedControl 每拿到新数组都要重新量滑块位置，常量能省掉这轮测量 */
const RANGE_OPTIONS: readonly SegmentedControlOption<string>[] = RANGES.map(value => ({
  value, label: RANGE_OPTION_LABEL[value],
}))

/**
 * 各面板头上那枚小问号的说明（原文逐字照抄 index.html 的 data-tip，一条都不能少、
 * 不能改：每一条都是踩过的边界）。仍是 `.tip-q` + data-tip 的既有形态 —— 与定时任务页
 * 同一处理，由 desktop-tooltip.tsx 的 MutationObserver 增强 React 插入的节点。
 */
const TIP_ACCOUNTS = t('How many tokens each account used in the selected range, sorted by token usage from high to low. Accounts are a finer dimension than providers: one provider can have several accounts, and requests try each account along the candidate chain by priority, so this reflects which specific login session is doing the work. Percentages are based on the total tokens for the range (this block only looks at usage; request counts and success rates are in the overview cards). "Unknown account" means those requests had no account identity: this happens when forwarding through the default login session (no account list configured); old aggregate rows written before this dimension existed also lack account details, but they are recomputed automatically from detail records when old data is imported during upgrade — only days whose details have already been cleared manually cannot be recovered and remain in "Unknown account"; those are genuinely unrecoverable.')
const TIP_PROVIDERS = t('How many tokens each provider used in the selected range, sorted by token usage from high to low. The same model name may be offered by several providers; the Gateway tries them in priority order as configured in the Forwarding route setting, so the share also reflects where requests actually landed. Percentages are based on the total tokens for the range (this block only looks at usage; request counts and success rates are in the overview cards). "Unknown" means those requests did not record the provider that served them—mainly detail rows from before provider recording existed, and requests that failed before an upstream was chosen (invalid request body, model not found, no available account). This dimension differs from accounts: old detail rows have no "which provider served it" information at all (early versions had a single upstream, but that was a configuration fact, not a per-request record), so they cannot be recomputed and old data stays in "Unknown" forever; requests created from now on all record their specific provider.')
/**
 * 时间范围控件旁的小问号。两个容易误读的口径都在这里说清：
 * 「近 N 天」是含今天在内往前数 N 天（所以是 N-1 天前到今天），不是「前 N 天」；
 * 「昨天」是本地自然日的昨天一整天，与「今天」共用同一套日界线。
 */
const TIP_RANGE = t('Time range. "Today" is the local calendar day from 00:00 until now; "Yesterday" is the whole of yesterday (00:00–24:00); "Last 7 days" / "Last 30 days" count backwards from today inclusive, i.e. today plus the previous 6 / 29 days — not 7 / 30 days excluding today.')

/* ─── 模块级状态（跨渲染的守卫、缓存与入口登记）────── */

/**
 * 是否有一次轮询触发的拉取还在途中。定时器是 setInterval（不等上一次完成），而间隔可以调到
 * 1 秒 —— 一次慢响应就会与后来的几拍叠在一起。load 本身按序号只认最新结果（不会显示乱序
 * 数据），但**每次都会打一次接口**，白白压着后端；所以轮询这一拍撞上在途请求时直接跳过，
 * 由下一拍补上（与 logs-panel / requests-panel 的 polling 同一处理）。
 */
let polling = false
/** 请求序号：并发时只认最新一次的结果（见 loadPanel 的说明） */
let seq = 0
/** 最近一次成功拿到的 summary：`lastSummary()` 读它（组件状态之外还要留一份，因为契约方法
 *  从 React 之外被调用） */
let summaryValue: StatsSummary | null = null

/** 组件挂载后登记的入口：契约方法都经它转发 */
type PanelHandle = {
  load(options?: LoadOptions): Promise<StatsSummary | null>
  render(data?: StatsSummary | null): void
}
let handle: PanelHandle | null = null
/** 挂载前收到的 render：挂载后立刻补发（正常路径用不到，兜底） */
let pendingRender: { data: StatsSummary | null } | null = null

type LoadOptions = { silent?: boolean }

/* ─── 小件 ─────────────────────────────────── */

/** 统一的空态 / 错误态文案：与组件层 .empty 同档留白 */
function placeholder(text: string, className = 'empty') {
  return <Empty className={className} style={{ padding: '14px 0' }}>{text}</Empty>
}

/** 面板头上的小问号：内容由 desktop-tooltip.tsx 增强（见 TIP_* 的说明），这里只留空壳 */
function TipQ({ text }: { text: string }) {
  return <span className='tip-q' data-tip={text} />
}

/**
 * 账号行的提供商徽章：与账号页「提供商」列**同一枚**（`pbadge p-<provider>`，配色按
 * provider id 生成，带 edition 的家把「国际版 / 国内版」拼在同一枚里，判定走
 * wbAccountsModel.editionSuffix）。
 *
 * 查的是**当前账号表**（wbApp 的状态）：报表聚合里没有 provider 维度，后端补这一维要动记账
 * 链路且历史数据没有；而「这条账号属于哪家」现查现用就是准确的。账号已删除（聚合里的历史名字
 * 快照）或后端的「未知账号」行查不到归属，不给徽章 —— 空壳徽章是噪音。
 */
function ProviderBadge({ accountId }: { accountId: string }) {
  const account = (shared().wbApp?.getState?.()?.accounts?.accounts || [])
    .find(item => item?.id === accountId)
  const provider = typeof account?.provider === 'string' ? account.provider : ''
  if (!provider) return null
  const label = shared().wbProviders?.labelOf?.(provider) || provider
  const suffix = (account && shared().wbAccountsModel?.editionSuffix?.(account)) || ''
  const text = suffix ? `${label} ${suffix}` : label
  return <Badge variant='outline' shape='tag' className={`pbadge p-${provider}`} title={t('Provider: {text}', { text })}>{text}</Badge>
}

function applyAutoRefresh(task: IntervalTask | null): void {
  pageRefreshStore.apply('reportAutoRefresh', task)
}

/* ─── 面板本体 ───────────────────────────────── */

/**
 * 一次读取在界面上的全部读数。`error` 与 `summary` 互斥：读取失败时 summary 置空、各板块换成
 * 错误态（旧实现 renderFailure 的口径）—— 留着上一轮的条反而会让人以为它还是可信的。
 */
type PageData = { summary: StatsSummary | null; error: string | null }

const EMPTY_DATA: PageData = { summary: null, error: null }

function ReportPage() {
  const [data, setData] = React.useState<PageData>(EMPTY_DATA)
  const [range, setRange] = React.useState<string>(readRange)
  /** 自定义区间（起止本地自然日）：只有档位选中「自定义」时随请求发出 */
  const [custom, setCustom] = React.useState<CustomRange>(readCustomRange)
  const [inspectedKey, setInspectedKey] = React.useState<StatsGroup | null>(null)
  const auto = usePageAutoRefresh('reportAutoRefresh')
  /** 单位口径（中文亿/万 ⇄ 英文 M/k）变了就整页重算：数值一个都没变，变的只是「怎么写成字」，
   *  所以拨一下开关是瞬时的 —— 不必重新拉一次报表 */
  const [, setUnitsTick] = React.useState(0)

  /** 当前档位：异步回调（loadPanel）必须立刻读到最新值，state 的更新要等下一次渲染，
   *  所以写入时同时落 ref 与 state，读一律走 ref.current */
  const rangeRef = React.useRef(range)
  /** 自定义区间同理（见 rangeRef 的说明）：日期一次改一个端，异步回调必须读到配好对的那份 */
  const customRef = React.useRef(custom)
  /**
   * 上一次**已应用**的那份数据的签名（JSON 文本）。
   *
   * 自动刷新是定时轮询，而两次轮询之间数据常常一个字都没变
   * （没有新请求、没有跨整点）—— 那种时候 `setData` 进来一个内容相同的新对象，React
   * 照样会把整页重渲染一遍：365 格热力图、两条折线、环形图、四张排行的元素全部重建一次，
   * 而屏幕上不会有任何变化。这张签名就是用来挡掉那一轮的。
   *
   * 比较用 JSON 文本而不是逐字段：报表是一棵形状固定的数据树，写逐字段比较等于把
   * 「报表有哪些字段」复制一份到这里（后端加字段就漏一个）。序列化这份数据（几十 KB）
   * 的成本在毫秒以下，而它省掉的是整页渲染。错误态写空串 —— 从「读取失败」恢复成
   * 正常数据时必然是一次真正的内容变化，不该被这张签名挡住。
   */
  const signatureRef = React.useRef('')
  const loadingRef = React.useRef(0)

  const applyData = React.useCallback((next: PageData, signature: string) => {
    signatureRef.current = signature
    setData(next)
  }, [])

  /* ─── 加载 ─────────────────────────────── */

  /**
   * 拉一次报表。不做互斥锁，而是「最后一次请求胜出」：用户连点几档范围时，用锁会把后面几次
   * 点击直接吞掉（界面停在旧数据上，看着像没反应）；这里让它们照常发出，只认最新那次的结果，
   * 先回来的旧响应一律作废。
   */
  const loadPanel = React.useCallback(async (options: LoadOptions = {}): Promise<StatsSummary | null> => {
    const token = ++seq
    loadingRef.current += 1
    const requested = rangeRef.current
    // 自定义区间的两个日期跟在这条请求上（后端只在 range=custom 时读它们；
    // 其余档位不带，避免出现「档位与日期各说各话」的请求）
    const dates = requested === CUSTOM_RANGE ? customRef.current : undefined
    try {
      const api = shared().workbuddyDesktop
      if (!api) throw new Error('Backend bridge unavailable')
      const payload = await api.getStatsSummary(requested, dates)
      if (token !== seq) return summaryValue
      if (!payload || typeof payload !== 'object') throw new Error('The backend did not return report data')
      summaryValue = payload
      // 内容与上一轮完全一致就不落 state（见 signatureRef 的说明）：轮询到「没有变化」
      // 是常态，那一轮不该付整页重渲染的代价
      const signature = JSON.stringify(payload)
      if (signature !== signatureRef.current) {
        applyData({ summary: payload, error: null }, signature)
      }
      return summaryValue
    } catch (error) {
      if (token !== seq) return summaryValue
      const message = errorMessage(error)
      summaryValue = null
      // silent：只压掉控制台噪音（首屏自持加载 / 轮询用），错误态照常显示在页面上
      if (!options.silent) console.warn('Failed to read the report data:', message)
      applyData({ summary: null, error: message || t('Unknown error') }, '')
      return null
    } finally {
      loadingRef.current -= 1
    }
  }, [applyData])

  /** 外部直接塞一份数据（契约里的 render）：不带参数 = 只重绘（旧实现的能力，React 里状态
   *  没变就没有重绘的必要）；传 null 只清 lastSummary 的读数，视图不动（与旧实现 renderAll
   *  在 summary 为空时整块早退一致） */
  const renderData = React.useCallback((next?: StatsSummary | null): void => {
    if (next === undefined) return
    summaryValue = next || null
    if (next) applyData({ summary: next, error: null }, JSON.stringify(next))
  }, [applyData])

  /** 换档位：内存为准，localStorage 只负责跨次启动恢复；换完立刻重拉一次 */
  const changeRange = React.useCallback((next: string): void => {
    const value = RANGES.includes(next) ? next : DEFAULT_RANGE
    if (value === rangeRef.current) return
    rangeRef.current = value
    setRange(value)
    try {
      localStorage.setItem(RANGE_KEY, value)
    } catch { /* 存储不可用时只影响下次启动，本次会话照常 */ }
    void loadPanel()
  }, [loadPanel])

  /**
   * 改自定义区间的某一端（日期输入框的 onChange）。
   *
   * 三条约定：
   *   - 只采纳合法的 `YYYY-MM-DD`：浏览器清空日期输入时给空串，放它过去会让请求带上
   *     非法参数（后端 400）——界面停在旧日期上更好解释；
   *   - 两端交叉（start 被改到晚于 end）时把另一端**拉齐到同一天**，而不是报错或
   *     静默 400：用户手输日期时这是最常见的一次失误；
   *   - 只有当前档位就是「自定义」时才重拉（在别的档位上改日期不影响正在看的数据）。
   */
  const changeCustomDate = React.useCallback((patch: Partial<CustomRange>): void => {
    const normalized = updateCustomRange(customRef.current, patch)
    if (normalized === customRef.current) return
    customRef.current = normalized
    setCustom(normalized)
    saveCustomRange(normalized)
    if (rangeRef.current === CUSTOM_RANGE) void loadPanel()
  }, [loadPanel])

  /* ─── 挂载 / 卸载 ───────────────────────── */

  React.useEffect(() => {
    handle = {
      load: loadPanel,
      render: renderData,
    }
    // 挂载前就来的 render 先补发（正常路径用不到：岛的挂载早于任何用户操作）
    if (pendingRender) {
      const queued = pendingRender
      pendingRender = null
      renderData(queued.data)
    }
    // 首屏自持加载：app.js 的 showPage 在本岛加载前就执行过（脚本排在 app.js 之后），那次调用
    // 还拿不到 window.wbReport；用户上次若停留在报表页，这里必须补一次。不在本页就不打这一枪。
    if (shared().wbApp?.currentPage === 'overview') void loadPanel({ silent: true })
    return () => { handle = null }
  }, [loadPanel, renderData])

  /**
   * 轮询定时器：配置一变就重排（旧实现的 startAuto 每次先 stopAuto），卸载时清表。三个前置
   * 条件缺一不可：任务已开启、间隔为正、页面可见时才有意义。任务被关掉时不排定时器（而不是
   * 排一个永不触发的），否则「关掉了但定时器还在跑」会让「间隔改了却像没生效」变得难排查。
   */
  React.useEffect(() => {
    if (!auto.enabled || auto.ms <= 0) return
    const timer = window.setInterval(() => {
      // 只在报表页可见时轮询，避免后台无谓请求
      if (document.hidden || shared().wbApp?.currentPage !== 'overview') return
      // 上一轮还没回来就跳过这一拍（见 polling 的说明）
      if (polling || loadingRef.current > 0) return
      polling = true
      void loadPanel({ silent: true }).finally(() => { polling = false })
    }, auto.ms)
    return () => { window.clearInterval(timer) }
  }, [auto.enabled, auto.ms, loadPanel])

  /** 单位口径变了就原地重绘（见 setUnitsTick 的说明） */
  React.useEffect(() => {
    const onChange = () => setUnitsTick(tick => tick + 1)
    window.addEventListener('wb-units-changed', onChange)
    return () => window.removeEventListener('wb-units-changed', onChange)
  }, [])

  /* ─── 渲染 ─────────────────────────────── */

  const { summary, error } = data
  // 概览小格的说明文字用后端回显的档位（与这份数据同一轮），后端没回显才退回本地档位
  const rangeKey = summary && RANGES.includes(summary.range ?? '') ? (summary.range as string) : range

  // 契约里没有这一维（undefined）就整块隐藏；空数组是「这一维没数据」（给空态）
  const providerList: StatsGroup[] | null = summary && Array.isArray(summary.providers) ? summary.providers : null
  const accountList: StatsGroup[] | null = summary && Array.isArray(summary.accounts) ? summary.accounts : null
  const modelList: StatsGroup[] | null = summary && Array.isArray(summary.models) ? summary.models : null

  /** 各板块的公共三态：错误态 / 正在加载 / 内容 */
  const errorBlock = error ? placeholder(t('Failed to load report: {error}', { error }), 'empty report-error') : null
  const loadingBlock = placeholder(t('Loading…'))
  const loading = errorBlock ?? loadingBlock

  const modelColors = React.useMemo(() => modelColorMap(modelList || (summary?.cacheTrend24h || []).flatMap(hour =>
    (hour.models || []).map(row => ({ model: row.model, label: row.label, tokens: row.tokens })))), [modelList, summary?.cacheTrend24h])

  return (
    <>
      <PageAutoRefresh id='reportAutoRefresh' settings={auto} onRefresh={() => loadPanel()} />
      <div className='report-page'>
        <PageHeader className='report-page-heading'>
          <div><PageTitle>{t('Usage report')}</PageTitle><PageDescription>{t('View usage, request sources, and cache performance')}</PageDescription></div>
          <div className='report-period-control'>
            <div className='report-date-caption' id='report-range-label'>{summary ? `${summary.startDate || '—'} — ${summary.endDate || '—'}` : RANGE_LABEL[rangeKey]} <TipQ text={TIP_RANGE} /></div>
            <SegmentedControl options={RANGE_OPTIONS} value={range} onValueChange={changeRange} aria-label={t('Report time range')} />
            {range === CUSTOM_RANGE ? <DateRangeFields dates={custom} onChange={changeCustomDate} /> : null}
          </div>
        </PageHeader>
        <ReportOverview summary={summary} loading={loading} />

      {/* ── Top 提供商 / Top 账号（并排两张卡，各自独立判断显隐）── */}
      <div className='report-rank-grid'>
        {accountList ? (
          <RankPanel panelId='report-accounts-panel' listId='report-accounts' labelId='report-accounts-label'
            title={t('Top accounts')} tip={TIP_ACCOUNTS} rows={accountList}
            badge={id => <ProviderBadge accountId={id} />} emptyText={t('所选范围内还没有账号用量')} />
        ) : null}
        {providerList ? (
          <RankPanel panelId='report-providers-panel' listId='report-providers' labelId='report-providers-label'
            title={t('Top providers')} tip={TIP_PROVIDERS} rows={providerList}
            emptyText={t('No request records in the selected range')} />
        ) : null}
      </div>

      <ReportUsageDonuts models={modelList} providers={providerList} />

      <ActivityPanel summary={summary} loading={loading} />
      <CacheWindows summary={summary} loading={loading} />
      <ReportDailyTrend summary={summary} loading={loading} />
      <CacheTrendPanel summary={summary} loading={loading} colors={modelColors} />

      {summary?.clients ? <SourceStatsPanel kind='clients' rows={summary.clients} summary={summary} /> : null}
      {summary?.apiKeys ? <SourceStatsPanel kind='apiKeys' rows={summary.apiKeys} summary={summary} onInspect={setInspectedKey} /> : null}
      </div>
      {inspectedKey ? <ApiKeyUsageDialog target={inspectedKey} range={range}
        dates={range === CUSTOM_RANGE ? custom : undefined} onClose={() => setInspectedKey(null)} /> : null}
    </>
  )
}

/* ─── 对外契约（调用点见文件头）──────────────────── */

/** app.js 切到本页 / upgrade-panel 升级完成后调它拉一次 */
async function load(options: LoadOptions = {}): Promise<StatsSummary | null> {
  // 挂载前来的调用不必单独补发：组件挂载时本来就会自拉一次（与旧实现的模块级首屏加载等价）
  if (!handle) return summaryValue
  return handle.load(options)
}

/** 直接塞一份数据（旧实现的能力，保留；当前没有调用点） */
function render(data?: StatsSummary | null): void {
  if (data === undefined) return
  if (!handle) {
    pendingRender = { data: data ?? null }
    return
  }
  handle.render(data)
}

/** 最近一次成功拿到的 summary（旧实现的读数出口，保留） */
function lastSummary(): StatsSummary | null {
  return summaryValue
}

declare global {
  interface Window {
    /** 报表页（替换 ui/report.js，接口与原实现一致） */
    wbReport?: {
      /** 切到本页 / 升级完成后刷新（app.js:117、upgrade-panel.js:73） */
      load(options?: LoadOptions): Promise<StatsSummary | null>
      /** 直接塞一份数据（保留旧实现的能力） */
      render(data?: StatsSummary | null): void
      /** 最近一次成功拿到的 summary */
      lastSummary(): StatsSummary | null
      /** 外部推送沿用同一份页面刷新配置。 */
      applyAutoRefresh(task: IntervalTask | null): void
    }
  }
}

window.wbReport = { load, render, lastSummary, applyAutoRefresh }

/* ─── 挂载：接管 index.html 里既有的页面区块 ─────── */

const PAGE_SELECTOR = '.page[data-page="overview"]'

let mounted = false

/**
 * 把 React root 直接建在页面区块上（不套宿主 div，理由见文件头）。先清掉骨架里的静态子节点
 * （.panel / .panel-head / .field-grid …）：下面按同样的类名重新渲染，留着会与 React 打架。
 */
function mount(): void {
  if (mounted) return
  const section = document.querySelector<HTMLElement>(PAGE_SELECTOR)
  if (!section) return
  mounted = true
  section.replaceChildren()
  createRoot(section).render(<ReportPage />)
}

// 首次切到本页时才挂载（见 island-lazy）：隐藏页不在首屏渲染，也不拉数据。
mountOnPage('overview', PAGE_SELECTOR, mount)
