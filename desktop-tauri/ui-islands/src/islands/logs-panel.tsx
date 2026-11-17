import * as React from 'react'
import { mountOnPage } from '../island-lazy'
import { createRoot } from 'react-dom/client'
import {
  Badge, Button, InputGroup, InputGroupAddon, InputGroupInput,
  Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
  SegmentedControl, Switch,
  Card, CardHeader, CardContent, Label } from '@ui'
import { SERVER_PAGE_SIZES, TableFooter, readPageSize, writePageSize, type PageSizeChoice } from './table-shell'
import { t } from '../i18n'
import {
  errorMessage, LEVEL_OPTIONS, logCategory, logGroups, logViewNote, persistFilters,
  queryFor, RANGE_OPTIONS, readFilters, shared, toast, VIEW_OPTIONS,
  type Filters, type IntervalTask, type LoadOptions, type LogGroup, type LogPage, type LogStats,
} from './logs-model'
import { LogDetail, LogsList } from './logs-list'
import { DateRangeFields } from './date-range-fields'
import { PageAutoRefresh, pageRefreshStore, usePageAutoRefresh } from './page-auto-refresh'

type PanelHandle = {
  load(options?: LoadOptions): Promise<void>
  render(data?: LogPage | null): void
  showCategory(category: string): Promise<void>
}
let handle: PanelHandle | null = null
let pendingCategory: string | null = null
let lastStatsValue: LogStats = null

function applyStats(stats: LogStats): void {
  lastStatsValue = stats
  shared().wbApp?.updateLogsBadge?.(stats)
}

function applyAutoRefresh(task: IntervalTask | null): void {
  pageRefreshStore.apply('logsAutoRefresh', task)
}

function LogsPanel() {
  const [data, setData] = React.useState<LogPage>({ entries: [], groups: [], total: 0, matched: 0, grouped: 0 })
  const [loaded, setLoaded] = React.useState(false)
  const [filters, setFilters] = React.useState<Filters>(readFilters)
  const [page, setPage] = React.useState(1)
  const [size, setSize] = React.useState(() => Number(readPageSize('logs', SERVER_PAGE_SIZES)))
  const auto = usePageAutoRefresh('logsAutoRefresh')
  const [paused, setPaused] = React.useState(false)
  const [reading, setReading] = React.useState(false)
  const [pending, setPending] = React.useState(0)
  const [detail, setDetail] = React.useState<LogGroup | null>(null)
  const [loading, setLoading] = React.useState(false)
  const [error, setError] = React.useState('')
  const [busy, setBusy] = React.useState<'clear' | 'export' | null>(null)
  const listRef = React.useRef<HTMLDivElement | null>(null)
  const current = React.useRef({ filters, page, size, data, paused, detail })
  current.current = { filters, page, size, data, paused, detail }
  const snapshotRef = React.useRef<number | undefined>(undefined)
  const seqRef = React.useRef(0)
  const pollingRef = React.useRef(false)
  const busyRef = React.useRef(false)
  const loadingRef = React.useRef(0)
  const restoredRef = React.useRef(false)
  const touchedRef = React.useRef(false)
  const keywordTimerRef = React.useRef<number | undefined>(undefined)
  const scrollRef = React.useRef<number | null>(null)

  const loadPanel = React.useCallback(async (options: LoadOptions = {}): Promise<void> => {
    if (!restoredRef.current && shared().wbFilterMemory) {
      restoredRef.current = true
      if (!touchedRef.current) {
        const saved = readFilters()
        current.current.filters = saved
        setFilters(saved)
      }
    }
    const state = current.current
    if (options.resetPage || options.latest) {
      state.page = 1
      setPage(1)
      snapshotRef.current = undefined
    }
    if (options.latest) {
      state.paused = false
      state.detail = null
      setDetail(null)
      setPaused(false)
      setReading(false)
      if (listRef.current) listRef.current.scrollTop = 0
    }
    const seq = ++seqRef.current
    loadingRef.current += 1
    const selectedPage = state.page
    const selectedSize = state.size
    const selectedFilters = state.filters
    if (!options.silent) setLoading(true)
    try {
      const api = shared().workbuddyDesktop
      if (!api) throw new Error(t('Backend bridge unavailable'))
      const locked = !options.latest && !options.resetPage && (selectedPage > 1 || state.paused || state.detail !== null || (listRef.current?.scrollTop || 0) > 20)
      const [result, stats] = await Promise.all([
        api.getLogs(queryFor(selectedFilters, {
          aggregate: selectedFilters.aggregate, offset: (selectedPage - 1) * selectedSize,
          limit: selectedSize, untilId: locked ? snapshotRef.current : undefined,
        })), api.getLogStats(),
      ])
      if (seq !== seqRef.current) return
      if (!result) throw new Error(t('Failed to load logs'))
      const totalGroups = Number(result.grouped ?? result.matched ?? result.entries?.length) || 0
      const maxPage = Math.max(1, Math.ceil(totalGroups / selectedSize))
      if (selectedPage > maxPage) {
        current.current.page = maxPage
        setPage(maxPage)
        await loadPanel(options)
        return
      }
      const nowReading = current.current.paused || current.current.detail !== null || (listRef.current?.scrollTop || 0) > 20
      if (options.silent && !options.manual && !locked && nowReading && snapshotRef.current !== undefined) {
        const changed = await api.getLogs(queryFor(current.current.filters, { sinceId: snapshotRef.current, limit: 1 }))
        if (seq === seqRef.current) setPending(Number(changed?.matched) || 0)
        return
      }
      scrollRef.current = options.resetPage || options.latest || !options.silent ? 0 : (listRef.current?.scrollTop || 0)
      current.current.data = result
      setData(result)
      setLoaded(true)
      setError('')
      if (!locked || snapshotRef.current === undefined) {
        snapshotRef.current = Number(result.lastId ?? stats?.lastId) || 0
        setPending(0)
      }
      applyStats(stats)
      if (selectedFilters.category && result.categories && !Object.hasOwn(result.categories, selectedFilters.category)) {
        const next = { ...current.current.filters, category: '' }
        current.current.filters = next
        setFilters(next)
        persistFilters(next)
        await loadPanel({ resetPage: true })
      }
    } catch (failure) {
      if (seq === seqRef.current) setError(errorMessage(failure))
    } finally {
      loadingRef.current -= 1
      if (seq === seqRef.current) setLoading(false)
    }
  }, [])

  const poll = React.useCallback(async () => {
    if (pollingRef.current || loadingRef.current > 0 || busyRef.current || keywordTimerRef.current !== undefined) return
    pollingRef.current = true
    try {
      const state = current.current
      const locked = state.paused || state.detail !== null || state.page > 1 || (listRef.current?.scrollTop || 0) > 20
      if (locked && snapshotRef.current !== undefined) {
        const api = shared().workbuddyDesktop
        if (!api) return
        const sinceId = snapshotRef.current
        const filtersBefore = state.filters
        const [result, stats] = await Promise.all([
          api.getLogs(queryFor(filtersBefore, { sinceId, limit: 1 })), api.getLogStats(),
        ])
        if (filtersBefore !== current.current.filters || sinceId !== snapshotRef.current) return
        if (stats?.lastId !== undefined && stats.lastId < sinceId) {
          snapshotRef.current = undefined
          await loadPanel({ resetPage: true })
        } else {
          setPending(Number(result?.matched) || 0)
          applyStats(stats)
        }
      } else await loadPanel({ silent: true })
    } catch (failure) { setError(errorMessage(failure)) }
    finally { pollingRef.current = false }
  }, [loadPanel])

  const changeFilters = React.useCallback((patch: Partial<Filters>, debounce = false) => {
    touchedRef.current = true
    const next = { ...current.current.filters, ...patch }
    current.current.filters = next
    setFilters(next)
    if (keywordTimerRef.current !== undefined) window.clearTimeout(keywordTimerRef.current)
    ++seqRef.current
    setLoading(true)
    const apply = () => {
      keywordTimerRef.current = undefined
      persistFilters(next)
      setPending(0)
      void loadPanel({ resetPage: true })
    }
    if (debounce) keywordTimerRef.current = window.setTimeout(apply, 300)
    else apply()
  }, [loadPanel])

  React.useEffect(() => {
    handle = {
      load: loadPanel,
      render: result => {
        if (!result) return
        current.current.data = result
        setData(result)
        setLoaded(true)
      },
      showCategory: async category => {
        changeFilters({ category, view: 'important' })
        shared().wbApp?.showPage?.('logs', { persist: true })
      },
    }
    if (pendingCategory !== null) {
      const category = pendingCategory
      pendingCategory = null
      void handle.showCategory(category)
    } else void loadPanel({ silent: true })
    return () => {
      handle = null
      ++seqRef.current
      if (keywordTimerRef.current !== undefined) window.clearTimeout(keywordTimerRef.current)
    }
  }, [changeFilters, loadPanel])

  React.useEffect(() => {
    if (!auto.enabled) return
    const timer = window.setInterval(() => {
      if (!document.hidden && shared().wbApp?.currentPage === 'logs') void poll()
    }, auto.ms)
    return () => window.clearInterval(timer)
  }, [auto.enabled, auto.ms, poll])

  React.useLayoutEffect(() => {
    if (scrollRef.current === null) return
    if (listRef.current) listRef.current.scrollTop = scrollRef.current
    scrollRef.current = null
  })

  const groups = logGroups(data)
  const totalGroups = Number(data.grouped ?? data.matched ?? groups.length) || 0
  const matched = Number(data.matched) || 0
  const pageCount = Math.max(1, Math.ceil(totalGroups / size))
  const badgeText = t('{n} entries', { n: matched })
  React.useEffect(() => { shared().wbApp?.renderTopbarStatus?.() }, [badgeText])

  function gotoPage(target: number): void {
    current.current.page = Math.max(1, Math.min(target, pageCount))
    setPage(current.current.page)
    setReading(false)
    if (listRef.current) listRef.current.scrollTop = 0
    void loadPanel()
  }

  function changeSize(choice: PageSizeChoice): void {
    const next = Number(choice)
    const first = (current.current.page - 1) * current.current.size
    current.current.size = next
    current.current.page = Math.floor(first / next) + 1
    setSize(next)
    setPage(current.current.page)
    writePageSize('logs', next)
    void loadPanel()
  }

  async function runAction(action: 'clear' | 'export', run: () => Promise<void>): Promise<void> {
    if (busyRef.current) return
    busyRef.current = true
    setBusy(action)
    try { await run() } catch (failure) { toast(t('Operation failed: {error}', { error: errorMessage(failure) }), 'err') }
    finally { busyRef.current = false; setBusy(null) }
  }

  async function clearLogs(): Promise<void> {
    if (busyRef.current || loading || error || !matched) return
    const query = queryFor(current.current.filters, { untilId: snapshotRef.current })
    const confirmed = await shared().wbConfirm?.ask?.({
      title: t('Clear runtime logs'),
      html: t('Clear the <strong>{count}</strong> filtered log entries? This cannot be undone.', { count: matched }),
      okText: t('Clear'), okClass: 'danger',
    })
    if (!confirmed) return
    await runAction('clear', async () => {
      const api = shared().workbuddyDesktop
      if (!api) throw new Error(t('Backend bridge unavailable'))
      await api.clearLogs(query)
      await loadPanel({ resetPage: true })
      toast(t('Runtime logs cleared'), 'ok')
    })
  }

  async function exportLogs(): Promise<void> {
    await runAction('export', async () => {
      const api = shared().workbuddyDesktop
      if (!api) throw new Error(t('Backend bridge unavailable'))
      const result = await api.exportLogs(queryFor(current.current.filters, { untilId: snapshotRef.current }))
      if (!result || result.canceled) return
      if (result.count === 0) { toast(t('No logs to export'), 'err'); return }
      toast(t('✅ Exported {count} log entries to {file}', { count: String(result.count), file: String(result.file) }), 'ok')
    })
  }

  const held = paused || reading || page > 1 || detail !== null
  const levelLabel = LEVEL_OPTIONS.find(option => option.value === filters.level)?.label || t('All levels')
  const counts = data.byLevel || {}
  const empty = !loaded && !error ? t('Loading logs…') : Number(data.total) ? t('No logs match the filters') : t('No logs')

  return (
    <Card render={<section />} className='panel' id='logs-panel-events'>
      <PageAutoRefresh id='logsAutoRefresh' settings={auto} onRefresh={() => loadPanel({ silent: true, manual: true })} disabled={busy !== null || loading} />
      <CardHeader className='panel-head'>
        <h2>{t('System events')}</h2><Badge id='logs-badge' variant='outline' data-tone=''>{loaded ? badgeText : '—'}</Badge>
        <div className='head-actions'>
          <Button variant='ghost' aria-pressed={paused} disabled={busy !== null} onClick={() => {
            const next = !current.current.paused
            current.current.paused = next
            setPaused(next)
            if (!next && !reading && page === 1 && !detail) void loadPanel({ latest: true })
          }}>{paused ? t('End reading') : t('Reading mode')}</Button>
          <Button id='btn-logs-export' variant='outline' disabled={busy !== null || loading || Boolean(error)} onClick={() => void exportLogs()}>
            {busy === 'export' ? t('Exporting…') : t('Export filtered logs')}</Button>
          <Button id='btn-logs-clear' variant='destructive' disabled={busy !== null || loading || Boolean(error) || !matched} onClick={() => void clearLogs()}>
            {busy === 'clear' ? t('Clearing…') : t('Clear filtered logs')}</Button>
        </div>
      </CardHeader>
      <CardContent className='panel-body'>
        <div className='event-log-toolbar'>
          <SegmentedControl options={VIEW_OPTIONS} value={filters.view} onValueChange={view => changeFilters({ view })} aria-label={t('Log view')} />
          <span className='event-log-status'>{held ? t('Reading mode: list stays in place') : auto.enabled ? t('Auto-refresh') : t('Auto-refresh disabled')}</span>
          <Label className='event-log-aggregate'><Switch checked={filters.aggregate} onCheckedChange={aggregate => changeFilters({ aggregate })} aria-label={t('Collapse repeated logs')} />{t('折叠重复日志')}</Label>
        </div>
        <div className='log-filters'>
          <SegmentedControl options={RANGE_OPTIONS} value={filters.range} onValueChange={range => changeFilters({ range })} aria-label={t('Event log time range')} className='shrink-0' />
          {filters.range === 'custom' ? <DateRangeFields dates={filters.dates}
            onChange={dates => changeFilters({ dates })} onClear={() => changeFilters({ range: 'all' })} /> : null}
          {filters.view !== 'debug' ? <Select value={filters.level} onValueChange={value => changeFilters({ level: String(value ?? '') })}>
            <SelectTrigger className='min-w-[120px]' aria-label={t('Filter by minimum level')}><SelectValue>{levelLabel}</SelectValue></SelectTrigger>
            <SelectContent>{LEVEL_OPTIONS.map(option => <SelectItem key={option.value} value={option.value}>{option.label}</SelectItem>)}</SelectContent>
          </Select> : null}
          <Select value={filters.category} onValueChange={value => changeFilters({ category: String(value ?? '') })}>
            <SelectTrigger className='min-w-[120px]' aria-label={t('Filter by category')}><SelectValue>{filters.category ? logCategory(filters.category, t) : t('All categories')}</SelectValue></SelectTrigger>
            <SelectContent><SelectItem value=''>{t('All categories')}</SelectItem>{Object.keys(data.categories || {}).map(category =>
              <SelectItem key={category} value={category}>{logCategory(category, t)}</SelectItem>)}</SelectContent>
          </Select>
          <InputGroup className='w-auto flex-auto'><InputGroupInput type='search' placeholder={t('Search plugins, messages, providers or accounts…')}
            aria-label={t('Search plugins, messages, providers or accounts')} autoComplete='off' value={filters.keyword}
            onChange={event => changeFilters({ keyword: event.currentTarget.value }, true)} /><InputGroupAddon aria-hidden='true'>⌕</InputGroupAddon></InputGroup>
        </div>
        <div className='event-log-overview'>
          <span>{t('{groups} groups · {entries} raw records', { groups: totalGroups, entries: matched })}</span>
          <span className='event-log-errors'>{t('Errors {n}', { n: counts.error || 0 })}</span>
          <span className='event-log-warnings'>{t('Warnings {n}', { n: counts.warn || 0 })}</span>
          {filters.view !== 'important' ? <span>{t('Debug {n}', { n: counts.debug || 0 })}</span> : null}
        </div>
        <div className='event-log-note'>{logViewNote(filters.view, t)}</div>
        {filters.view === 'debug' && !data.debugEnabled ? <div className='event-log-note'>{t('Debug collection is off. Only retained records are shown. Enable debug mode in Settings to collect new records.')}</div> : null}
        {error ? <div className='event-log-error' role='status'>{error}<Button variant='ghost' size='xs' onClick={() => void loadPanel()}>{t('Retry')}</Button></div> : null}
        {pending > 0 || held ? <div className='event-log-new' role='status'>
          <span>{pending ? t('{n} new log entries', { n: pending }) : t('Reading mode: list stays in place')}</span>
          <Button variant='outline' size='sm' disabled={loading} onClick={() => void loadPanel({ latest: true })}>{t('View latest')}</Button>
        </div> : null}
        <LogsList groups={groups} empty={empty} loading={loading} listRef={listRef}
          onScroll={() => setReading((listRef.current?.scrollTop || 0) > 20)} onDetail={group => { current.current.detail = group; setDetail(group) }} />
      </CardContent>
      <TableFooter total={totalGroups} page={page} pageCount={pageCount} size={size} sizes={SERVER_PAGE_SIZES}
        disabled={loading || busy !== null} range={{ start: groups.length ? (page - 1) * size + 1 : 0, end: (page - 1) * size + groups.length }}
        leading={<span className='event-log-footer-note'>{t('Pagination counts collapsed groups')}</span>} onSizeChange={changeSize} onPageChange={gotoPage} />
      <LogDetail group={detail} onClose={() => { current.current.detail = null; setDetail(null) }} />
    </Card>
  )
}

async function load(options: LoadOptions = {}): Promise<void> { await handle?.load(options) }
function render(data?: LogPage | null): void { handle?.render(data) }
function lastStats(): LogStats { return lastStatsValue }
async function showCategory(category: string): Promise<void> {
  if (!handle) { pendingCategory = category; return }
  await handle.showCategory(category)
}

function mount(): void {
  const section = document.querySelector<HTMLElement>('.page[data-page="logs"]')
  if (!section) return
  section.replaceChildren()
  createRoot(section).render(<LogsPanel />)
}
mountOnPage('logs', '.page[data-page="logs"]', mount)

declare global {
  interface Window {
    wbLogsPanel?: {
      load(options?: LoadOptions): Promise<void>
      render(data?: LogPage | null): void
      lastStats(): LogStats
      applyAutoRefresh(task: IntervalTask | null): void
      showCategory(category: string): Promise<void>
    }
  }
}
window.wbLogsPanel = { load, render, lastStats, applyAutoRefresh, showCategory }
