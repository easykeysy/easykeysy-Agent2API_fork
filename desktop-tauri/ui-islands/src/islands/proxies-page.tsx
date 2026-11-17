/** 网络代理页：手动 HTTP / SOCKS5 地址与 Clash Verge 同步端点。 */

import * as React from 'react'
import { mountOnPage } from '../island-lazy'
import { createRoot } from 'react-dom/client'
import {
  Badge, Button, Card, CardHeader, Checkbox, InputGroup, InputGroupAddon,
  InputGroupInput, ResponsiveSheet, SegmentedControl, Select, SelectContent,
  SelectItem, SelectTrigger, SelectValue, Spinner, StatusDot, Switch, Table, TableBody, TableCell,
  TableHead, TableHeader, TableRow, cn,
} from '@ui'
import { poolItemAddress } from './accounts-shared'
import {
  batchRemove, batchTest, clearSelection, durationText, formatTime, getSnapshot,
  isPicked, items, loadPanel, removeItem, setAllPicked, subscribe, syncClash, testItem,
  toggleItem, togglePick, wb, type ProxyPoolItem,
} from './proxies-state'
import { ProxyModal, ProxySettingsModal } from './proxies-dialogs'
import { COL_CLASS, closeColumnPanel, repaintColumnWidths, setupColumns, visibleColumns } from './proxies-columns'
import { TableFooter, useClientPaging } from './table-shell'
import { t } from '../i18n'
import { proxyServiceLabels } from '../../../ui-shared/proxy-usage'
import { shouldShowProxyRetirement } from '../../../ui-shared/proxy-retirement'
import { ProxyRetirementDialog } from '../../../ui-shared/proxy-retirement-dialog'

let noticeRevision = 0
const noticeListeners = new Set<() => void>()
function enteredPage(): void {
  if (!shouldShowProxyRetirement()) return
  noticeRevision += 1
  noticeListeners.forEach(listener => listener())
}

function ProxiesPage() {
  const snapshot = React.useSyncExternalStore(subscribe, getSnapshot)
  const notice = React.useSyncExternalStore(listener => {
    noticeListeners.add(listener)
    return () => { noticeListeners.delete(listener) }
  }, () => noticeRevision)
  const [noticeOpen, setNoticeOpen] = React.useState(false)
  const [modal, setModal] = React.useState<{ item: ProxyPoolItem | null } | null>(null)
  const [settingsOpen, setSettingsOpen] = React.useState(false)
  const [filterOpen, setFilterOpen] = React.useState(false)
  const [keyword, setKeyword] = React.useState('')
  const [protocol, setProtocol] = React.useState('all')
  const [status, setStatus] = React.useState('all')
  const [colRetry, setColRetry] = React.useState(0)
  const [, forceTick] = React.useReducer((tick: number) => tick + 1, 0)
  const rows = items()
  const ready = snapshot.data !== null
  const selectedCount = snapshot.selected.size
  const progress = snapshot.batchProgress
  const locked = snapshot.syncing || snapshot.batchBusy || snapshot.testing !== null || snapshot.pending.size > 0
  const columns = visibleColumns()
  const query = keyword.trim().toLowerCase()
  const shown = rows.filter(item => (protocol === 'all' || (item.resolved?.protocol || item.protocol) === protocol)
    && (status === 'all' || stateOf(item) === status)
    && (!query || [item.name, poolItemAddress(item), item.host, item.protocol, item.port].join(' ').toLowerCase().includes(query)))
  const shownIds = shown.map(item => item.id)
  const allPicked = shownIds.length > 0 && shownIds.every(id => snapshot.selected.has(id))
  const somePicked = shownIds.some(id => snapshot.selected.has(id)) && !allPicked
  const paging = useClientPaging(shown.length, 'proxies')
  const pageRows = paging.paged ? paging.slice(shown) : shown

  React.useEffect(() => { if (notice > 0) setNoticeOpen(true) }, [notice])
  React.useEffect(() => { paging.goto(1) }, [query, protocol, status])
  React.useLayoutEffect(() => {
    if (!setupColumns(() => forceTick())) {
      const timer = window.setTimeout(() => setColRetry(tick => tick + 1), 50)
      return () => window.clearTimeout(timer)
    }
    forceTick()
    return undefined
  }, [colRetry])
  React.useLayoutEffect(() => { repaintColumnWidths() })
  React.useEffect(() => () => { closeColumnPanel() }, [])

  function clearFilters(): void { setKeyword(''); setProtocol('all'); setStatus('all') }

  function cell(column: string, item: ProxyPoolItem): React.ReactNode {
    switch (column) {
      case 'pick':
        return <Checkbox checked={isPicked(item.id)} disabled={locked}
          aria-label={t('Select "{name}"', { name: item.name || t('Untitled') })}
          onCheckedChange={next => togglePick(item.id, next)} />
      case 'name':
        /* 名称一行、说明一行：说明不套彩色标签，只用一档灰字 ——
           标签盒子一多，整张表就从「列表」变成「仪表盘」 */
        return <div className='flex min-w-0 flex-col gap-0.5'>
          <span className='truncate text-[12.5px] font-medium' title={item.name || t('Untitled')}>{item.name || t('Untitled')}</span>
          <span className='flex min-w-0 items-center gap-1.5 text-[10.5px] text-muted-foreground'>
            <span className='shrink-0 font-mono'>{(item.resolved?.protocol || item.protocol || 'http').toUpperCase()}</span>
            <span className='opacity-45' aria-hidden='true'>·</span>
            {item.source === 'clash'
              ? <span className='truncate' title={t('Read-only: edit it in Clash Verge')}>Clash Verge</span>
              : <span className='truncate'>{t('Manual entry')}</span>}
            {item.nonDirect ? <>
              <span className='opacity-45' aria-hidden='true'>·</span>
              <span className='shrink-0' title={t('Connect through the system proxy first, then through this proxy')}>{t('Chained')}</span>
            </> : null}
            {item.enabled === false ? <>
              <span className='opacity-45' aria-hidden='true'>·</span>
              <span className='shrink-0'>{t('Disabled')}</span>
            </> : null}
          </span>
        </div>
      case 'addr': {
        const address = poolItemAddress(item)
        /* 协议前缀压淡、主机端口保持正常色：一列地址扫起来先认端口。
           另：原写法用的 text-text-2 从未定义过（等于没上色），换成真令牌 text-subtle。
           只有真拆出前缀时才上灰色，'http://' 这种只有前缀的残缺值原样显示 */
        const scheme = /^[a-z0-9]+:\/\//i.exec(address)?.[0] ?? ''
        const rest = scheme && address.length > scheme.length ? address.slice(scheme.length) : address
        return <div className='flex min-w-0 items-center gap-1'>
          <span className='min-w-0 flex-1 truncate font-mono text-xs leading-[1.5] text-subtle'
            title={item.resolveError || address}>
            {rest !== address ? <span className='text-muted-foreground'>{scheme}</span> : null}
            {rest || t('Incomplete address')}
          </span>
          {address ? <Button variant='ghost' size='icon-xs' className='copy-btn shrink-0' data-copy={address}
            title={t('Copy address')} aria-label={t('Copy address')}><CopyIcon /></Button> : null}
        </div>
      }
      case 'usage': {
        const services = proxyServiceLabels(item, t)
        const usedBy = item.usedBy || []
        const accountLine = usedBy.length ? t('{n} accounts using', { n: usedBy.length }) : ''
        return <div className='flex min-w-0 flex-col gap-0.5'>
          {accountLine ? <span className='truncate text-[11.5px] text-subtle'
            title={usedBy.map(entry => entry.name || entry.id).join(t(', '))}>{accountLine}</span> : null}
          {services.length ? <span className='truncate text-[10.5px] text-muted-foreground'
            title={services.join(t(', '))}>{services.join(' · ')}</span> : null}
          {!accountLine && !services.length ? <span className='text-[11.5px] text-muted-foreground'>{t('Not in use')}</span> : null}
        </div>
      }
      case 'state':
        return <Switch checked={item.enabled !== false} disabled={locked || item.source === 'clash' && item.sourceEnabled === false}
          title={item.source === 'clash'
            ? item.sourceEnabled === false ? t('This listener is disabled in Clash Verge, so enabling it here has no effect — turn it on in Clash Verge')
              : t('Once disabled this machine stops using it (accounts referencing it fall back to direct); the port stays as it is on the Clash Verge side')
            : undefined}
          aria-label={t('Enable "{name}"', { name: item.name || t('Untitled') })}
          onCheckedChange={next => void toggleItem(item, next)} />
      case 'test': {
        if (snapshot.testing === item.id) return <span className='flex items-center gap-1.5 text-[11.5px] text-muted-foreground'><Spinner />{t('Testing…')}</span>
        const last = item.lastTest
        if (!last?.at) return <span className='flex items-center gap-1.5 text-[11.5px] text-muted-foreground'>
          <StatusDot variant='muted' />{t('Not checked')}</span>
        /* 颜色只落在状态点上、文字是常规色：结果一眼能扫，整列却不吵。
           时间在前、出口 IP 或失败原因在后 —— 时间永远不会被长文案挤掉 */
        return <div className='flex min-w-0 flex-col gap-0.5'>
          <span className='flex items-baseline gap-1.5'>
            <StatusDot variant={last.success ? 'success' : 'destructive'} className='translate-y-[-1px]' />
            <span className='text-[12px]'>{last.success ? t('usable') : t('Failed')}</span>
            <span className='font-mono text-[10.5px] text-muted-foreground'>{durationText(last.durationMs)}</span>
          </span>
          <span className='flex min-w-0 items-baseline gap-1 text-[10.5px]'>
            <span className='shrink-0 font-mono text-muted-foreground'>{formatTime(last.at)}</span>
            <span className='opacity-45' aria-hidden='true'>·</span>
            <span className={cn('truncate font-mono', last.success ? 'text-muted-foreground' : 'text-destructive')}
              title={last.success ? last.ip || undefined : last.error || t('Connection failed')}>
              {last.success ? last.ip || '' : last.error || t('Connection failed')}
            </span>
          </span>
        </div>
      }
      case 'act':
        /* 「测试」压在最右边：整列右对齐，于是它永远贴着表格右缘 ——
           次要操作（只读提示 / 编辑 / 删除）留在它左边，安静但不消失 */
        return <div className='row-actions'>
          {item.source === 'clash'
            ? <span className='self-center text-[10.5px] text-muted-foreground'
              title={t('Edit the name and port in Clash Verge; the enable switch can be turned off on this page')}>{t('Read-only')}</span>
            : <>
              <Button size='icon-xs' variant='ghost' disabled={locked} title={t('Edit')} aria-label={t('Edit')}
                onClick={() => setModal({ item })}><EditIcon /></Button>
              <Button size='icon-xs' variant='ghost' disabled={locked} title={t('Delete')} aria-label={t('Delete')}
                className='text-muted-foreground hover:text-destructive'
                onClick={() => void removeItem(item)}><TrashIcon /></Button>
            </>}
          <Button size='sm' variant='outline' disabled={locked} onClick={() => void testItem(item)}>{t('Test')}</Button>
        </div>
      default: return null
    }
  }

  return <>
    <Card render={<section />} className='panel'>
      <CardHeader className='panel-head'>
        <h2>{t('Proxy list')}</h2><Badge variant='secondary' shape='tag'>{rows.length}</Badge>
        <span className='panel-sub'>{ready ? t('{n} enabled', { n: rows.filter(item => item.enabled !== false).length }) : t('Loading…')}</span>
        <div className='head-actions'>
          {/* 主次分明：只有「新增代理」是实心主操作，其余按重要性由浅到深排开
              （同步是第二条常用动作，保留描边；测试设置与刷新是轻动作，走 ghost） */}
          <Button disabled={locked} onClick={() => setModal({ item: null })}>{t('+ New proxy')}</Button>
          <Button variant='ghost' className='hidden max-[767px]:inline-flex' onClick={() => setFilterOpen(true)}>{t('Filter')}</Button>
          <Button variant='outline' disabled={locked} onClick={() => void syncClash()}>{snapshot.syncing ? t('Syncing…') : t('Sync Clash Verge')}</Button>
          <Button variant='ghost' disabled={locked} onClick={() => setSettingsOpen(true)}>{t('Test settings')}</Button>
          <Button variant='ghost' disabled={snapshot.loading || locked} onClick={() => void loadPanel()}>{snapshot.loading ? t('Refreshing…') : t('Refresh')}</Button>
        </div>
      </CardHeader>
      {selectedCount ? <div className='batch-bar active'>
        <span className='batch-count'>{t('Selected')} <b>{selectedCount}</b> {t('total')}</span>
        <div className='batch-actions'>
          <Button variant='outline' disabled={locked} onClick={() => void batchTest()}>{progress ? t('Testing {done}/{total}', { done: progress.done, total: progress.total }) : t('Test selected')}</Button>
          <Button variant='destructive' disabled={locked} onClick={() => void batchRemove()}>{t('Delete selected')}</Button>
          <Button variant='outline' disabled={locked} onClick={clearSelection}>{t('Deselect')}</Button>
        </div>
      </div> : <ResponsiveSheet open={filterOpen} onOpenChange={setFilterOpen} title={t('Filter')}
        inlineClassName='flex flex-wrap items-center gap-2.5 border-b border-border px-4 py-3'
        bodyClassName='proxy-filter-body'
        actions={<Button variant='ghost' size='sm' onClick={clearFilters}>{t('Clear')}</Button>}>
        <InputGroup className='w-[clamp(150px,22vw,240px)]'>
          <InputGroupInput type='search' value={keyword} aria-label={t('Search proxies')} placeholder={t('Search name / address / port')}
            onChange={event => setKeyword(event.currentTarget.value)} />
          <InputGroupAddon align='inline-start' aria-hidden='true'><SearchIcon /></InputGroupAddon>
        </InputGroup>
        <Select value={protocol} onValueChange={value => setProtocol(value || 'all')}>
          <SelectTrigger className='w-[120px]' aria-label={t('Filter by protocol')}><SelectValue>{protocol === 'all' ? t('All protocols') : protocol.toUpperCase()}</SelectValue></SelectTrigger>
          <SelectContent><SelectItem value='all'>{t('All protocols')}</SelectItem><SelectItem value='http'>HTTP</SelectItem><SelectItem value='socks5'>SOCKS5</SelectItem></SelectContent>
        </Select>
        <SegmentedControl className='ml-auto' options={statusOptions(rows)} value={status} onValueChange={setStatus} aria-label={t('Filter by status')} />
      </ResponsiveSheet>}
      <div className='models-table-wrap'>
        <Table container={false} className='models-table proxies-table'>
          <colgroup>{columns.map(column => <col key={column.key} data-col={column.key} className={COL_CLASS[column.key] || undefined} />)}</colgroup>
          <TableHeader><TableRow>{columns.map(column => <TableHead key={column.key} data-col={column.key} className={`ta-${column.align}`}>
            {column.key === 'pick' ? <Checkbox checked={allPicked} indeterminate={somePicked} disabled={!shownIds.length || locked}
              aria-label={t('Select all filtered endpoints')} onCheckedChange={next => setAllPicked(shownIds, next)} /> : column.label}
          </TableHead>)}</TableRow></TableHeader>
          <TableBody>{pageRows.length ? pageRows.map(item => <TableRow key={item.id} data-id={item.id}
            className={cn(item.enabled === false && 'off', isPicked(item.id) && 'bg-primary-soft')}>
            {columns.map(column => <TableCell key={column.key} className={`ta-${column.align}`}>{cell(column.key, item)}</TableCell>)}
          </TableRow>) : <TableRow><TableCell colSpan={columns.length} className='empty'>
            {!ready ? snapshot.error || t('Loading…') : !rows.length ? <div className='flex flex-col items-center gap-3 py-8'>
              <b className='text-sm text-foreground'>{t('No saved proxies yet')}</b>
              <p className='text-xs text-subtle'>{t('Add a manual HTTP / SOCKS5 address or sync Clash Verge endpoints for use by accounts and service routes.')}</p>
              <Button onClick={() => setModal({ item: null })}>{t('Add your first proxy')}</Button>
            </div> : <div className='flex flex-col items-center gap-3 py-5'>
              <span>{t('No endpoints match the current filters')}</span><Button variant='outline' size='sm' onClick={clearFilters}>{t('Clear filters')}</Button>
            </div>}
          </TableCell></TableRow>}</TableBody>
        </Table>
      </div>
      <TableFooter leading={snapshot.error ? <span className='text-destructive'>{t('Load failed: {reason}', { reason: snapshot.error })}</span> : undefined}
        total={shown.length} page={paging.page} pageCount={paging.pageCount} size={paging.size}
        onSizeChange={paging.setSize} onPageChange={paging.goto}
        range={paging.paged ? { start: paging.rangeStart, end: paging.rangeEnd } : null} />
    </Card>
    {modal ? <ProxyModal item={modal.item} onClose={() => setModal(null)} /> : null}
    {settingsOpen ? <ProxySettingsModal onClose={() => setSettingsOpen(false)} /> : null}
    <ProxyRetirementDialog open={noticeOpen} onClose={() => setNoticeOpen(false)} t={t} />
  </>
}

function stateOf(item: ProxyPoolItem): string {
  if (item.enabled === false) return 'off'
  if (!item.lastTest?.at) return 'none'
  return item.lastTest.success ? 'ok' : 'fail'
}

function statusOptions(rows: ProxyPoolItem[]): Array<{ value: string; label: string; count: number }> {
  return [
    { value: 'all', label: t('All'), count: rows.length },
    { value: 'ok', label: t('usable'), count: rows.filter(item => stateOf(item) === 'ok').length },
    { value: 'fail', label: t('Failed'), count: rows.filter(item => stateOf(item) === 'fail').length },
    { value: 'none', label: t('Not checked'), count: rows.filter(item => stateOf(item) === 'none').length },
    { value: 'off', label: t('Disabled'), count: rows.filter(item => stateOf(item) === 'off').length },
  ]
}

function SearchIcon() {
  return <svg width='14' height='14' viewBox='0 0 24 24' fill='none' stroke='currentColor' strokeWidth='2' strokeLinecap='round' aria-hidden='true'><circle cx='11' cy='11' r='8' /><path d='m21 21-4.3-4.3' /></svg>
}

/* 行内图标：线条、14px、只吃 currentColor —— 与上方搜索图标同一口径。
   原来是文字符号（行内 ⧉）与纯文字按钮，换成图形后三颗按钮的宽度与
   对齐都稳定了，也不再依赖系统字体里有没有那个符号。 */
function CopyIcon() {
  return <svg viewBox='0 0 24 24' className='size-3.5' fill='none' stroke='currentColor' strokeWidth='1.5' strokeLinecap='round' strokeLinejoin='round' aria-hidden='true'>
    <path d='M9 9V6.5A1.5 1.5 0 0 1 10.5 5h7A1.5 1.5 0 0 1 19 6.5v7a1.5 1.5 0 0 1-1.5 1.5H15' />
    <path d='M5.5 9h7A1.5 1.5 0 0 1 14 10.5v7A1.5 1.5 0 0 1 12.5 19h-7A1.5 1.5 0 0 1 4 17.5v-7A1.5 1.5 0 0 1 5.5 9Z' />
  </svg>
}

function EditIcon() {
  return <svg viewBox='0 0 24 24' className='size-3.5' fill='none' stroke='currentColor' strokeWidth='1.5' strokeLinecap='round' strokeLinejoin='round' aria-hidden='true'>
    <path d='M4.5 19.5h3l9-9a2.1 2.1 0 0 0-3-3l-9 9v3Z' />
    <path d='m14.5 6.5 3 3' />
  </svg>
}

function TrashIcon() {
  return <svg viewBox='0 0 24 24' className='size-3.5' fill='none' stroke='currentColor' strokeWidth='1.5' strokeLinecap='round' strokeLinejoin='round' aria-hidden='true'>
    <path d='M5 7h14' />
    <path d='M9 7V5.5A1.5 1.5 0 0 1 10.5 4h3A1.5 1.5 0 0 1 15 5.5V7' />
    <path d='M7 7l.8 11a1.5 1.5 0 0 0 1.5 1.4h5.4A1.5 1.5 0 0 0 16.2 18L17 7' />
  </svg>
}

declare global {
  interface Window {
    wbProxiesPanel?: {
      load: () => Promise<void>
      activate: () => void
      visibleColumns: () => Array<{ key: string }>
    }
  }
}

window.wbProxiesPanel = {
  load: () => loadPanel(),
  activate: enteredPage,
  visibleColumns: () => visibleColumns().map(column => ({ key: column.key })),
}
const PAGE_SELECTOR = '.page[data-page="proxies"]'
function mount(): void {
  const section = document.querySelector<HTMLElement>(PAGE_SELECTOR)
  if (!section || section.dataset.mounted === '1') return
  section.dataset.mounted = '1'
  section.replaceChildren()
  createRoot(section).render(<ProxiesPage />)
  if (wb().wbApp?.currentPage === 'proxies') { enteredPage(); void loadPanel() }
}
mountOnPage('proxies', PAGE_SELECTOR, mount)
