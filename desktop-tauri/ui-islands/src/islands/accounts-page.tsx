

import * as React from 'react'
import { mountOnPage } from '../island-lazy'
import { flushSync } from 'react-dom'
import { createRoot } from 'react-dom/client'
import { t } from '../i18n'
import { Button, Checkbox, Input, ResponsiveSheet, SegmentedControl, Select, SelectContent, SelectItem, SelectTrigger, SelectValue, cn, Card, Label, Empty, Table, TableBody, TableRow, TableCell } from '@ui'
import { TableFooter, useClientPaging } from './table-shell'
import { shared, toast, type AccountRecord, type ColSettingsHandle } from './accounts-shared'
import { isDesktopAccount, isEnabled, providerOf, supportsUsage } from './accounts-domain'
import { ACCOUNT_COLUMNS, bindColumnGrips, columnWidths, tableMinWidth } from './accounts-columns'
import {
  allAccounts, bump, clearFilter, clearLimits, clearSelection, ensureProxyPoolOptions,
  filterActive, getStore, groupSortRules, groupView, hiddenAccounts, installAccountsApi, normalizeProviderFilter, openBatchDialog,
  providerSummaryList, queryAllUsage, refreshList, reorderQueue, rowContext, seats, segmentCounts,
  setAllGroupsCollapsed, setAllPicked, setProviderFilter, setSearchFilter, setSegmentFilter,
  snapshot, startConnectionsPolling, subscribe, subscribeProxyPool, syncGroupTotalPreference,
  tokenUsageOf, togglePick, usageEntryOf, GROUP_TOTAL_FILTERED_KEY,
} from './accounts-data'
import {
  AccountCell, ActionsCell, ConnectionsCell, ExpiryCell, GroupBandRow, LimitsCell, OrderCell,
  ProxyCell, StatusCell, TableHead, UsageCell,
} from './accounts-panels'
import { bindQueueDrag, type QueueDrop } from './accounts-queue-drag'
import { AccountsDialogs } from './accounts-dialogs'
import { HiddenAccountsNotice } from './accounts-hidden-notice'
import { activeGroupSortRules, manualOrderBlockedMessage } from './accounts-group-settings'
import { Badge, Spinner } from '@ui'
import { Alert } from '@ui'

/** set （ in after effect ， down ） */
let colHandle: ColSettingsHandle | null = null
/** set is already （ 1 ： is ， will ） */
let colSettingsRegistered = false
/**
 * when 。
 *
 * Why need need ： is `flushSync` （ as add-account-modal in 
 * **** can to 「」 that by ）， effect in that up ——
 * that 1 in `table-col-settings.tsx` before ，`window.wbColSettings` up ，
 * 1 will empty、 not （ old is ， no has this when ）。
 * not to 1 ， to 。
 */
let colSettingsRetry = 0
/** （ need use ； by <table ref> ） */
let tableEl: HTMLTableElement | null = null
/** already 「 in not in + can and for 」： has （ syncHead） */
let headSignature = ''

/** current can （config；set when ） */
function visibleColumns(): Array<(typeof ACCOUNT_COLUMNS)[number] & { align: 'left' | 'center' | 'right' }> {
  return colHandle ? colHandle.apply(ACCOUNT_COLUMNS) : ACCOUNT_COLUMNS
}

/** this then is （ when ： will then ） */
function sortActiveIn(account: AccountRecord): boolean {
  const provider = providerOf(account)
  return activeGroupSortRules(provider, groupSortRules(provider)).length > 0
}

/** 1 （store → ） */
function requestRender(): void {
  window.wbAccountsView?.render?.()
}

/**
 * （ / / for ）+ to 「 not is after 1 」 up 。
 *
 * for is required ： this need in **** after check（、、config
 * will ）， `syncStaticHead` is `appendChild` has —— none
 * 2 seconds（） 1 will not DOM 。
 */
function syncHead(): void {
  const columns = visibleColumns()
  const signature = `${tableEl ? 'T' : 'F'}|${columns.map(column => `${column.key}:${column.align}`).join('|')}`
  if (signature === headSignature) return
  headSignature = signature
  shared().wbColSettings?.syncStaticHead('accounts', tableEl)
  // need in 「 not is after 1 」 up ： for in （right: -4px）， in 
  // will 1 to 。「 1 in after 」 is use config ， by after 。
  const ths = [...(tableEl?.querySelectorAll('thead th') || [])]
  ths.forEach((th, index) => {
    const grip = th.querySelector('.col-grip') as HTMLElement | null
    if (grip) grip.style.display = index === ths.length - 1 ? 'none' : ''
  })
}

/* ─── ─────────────────────────────────── */

function AccountsPage() {
  const store = React.useSyncExternalStore(subscribe, getStore)
  const listRef = React.useRef<HTMLDivElement | null>(null)
  /** set after 1 use （ not store when ， down ） */
  const [, forceTick] = React.useReducer((tick: number) => tick + 1, 0)

  const loaded = Boolean(snapshot())
  const all = allAccounts()
  const hidden = hiddenAccounts()
  // groups = need （current down 1 not not empty）；allGroups = ，
  // and 「 / 」 use 
  const { groups, allGroups } = groupView(usageEntryOf, tokenUsageOf)
  // 「 not already / Token already 」 already 
  // （isLimitedNow ）， and need up and Token 
  // —— bump after this 
  const counts = segmentCounts(usageEntryOf, tokenUsageOf)
  /**
   * down ：`（N）` + 1 （ 0 —— need ）。
   * is ** **（ this down has ）， not ：
   * and not is 1 ——「 this 1 has 」 not is 「 can to 」，
   * is 「 has 、 has 」 not can to （， this is use down not use Reason）。
   */
  const providerOptions = [{ value: 'all', label: t('全部（{n}）', { n: all.length }) }]
    .concat(providerSummaryList().map(item => ({ value: item.id, label: t('{label}（{count}）', { label: t(item.label), count: item.count }) })))
  /** will （ → ）。 / / as ：
   * （ 1 enabled no has ） not this 「current to this 」—— otherwise「（8 ）」
   * will not 1 up ， for not up 。（ that ） 1 ：
   * is 「 1 has 」， not this then 。 */
  const shown = groups.flatMap(group => group.accounts)
  const seatMap = seats()
  const columns = visibleColumns()
  /**
   * client（ use ）：default to 「」 this 1 —— down by will 
   * 1 （ in up 1 ）， use can in 。
   *
   * when ** not can **（ groupLocked）： in down to 
   * 「 can 」（ not ）， ； not 
   * limit —— by can （ onDrop），current。
   */
  const paging = useClientPaging(shown.length, 'accounts', { defaultSize: 'all' })
  const pageRows = paging.paged ? paging.slice(shown) : shown
  /** only by enabled； its or ， not limit。 */
  const groupFilterLocked = store.filter.provider !== 'all'
    || store.filter.q.trim() !== ''
    || store.filter.limit !== 'all'
  const groupLocked = groupFilterLocked || paging.paged
  const lockReason = groupFilterLocked ? t('清除筛选') : t('把每页调回「全部」')

  // （「」 ） no （ 30 seconds，failure not ）。
  // Clash that 1 already not in this 1 —— 1 by 「 Clash Verge」
  // ， this （ accounts-panels ProxyCell）
  React.useEffect(() => { ensureProxyPoolOptions() })

  // is and 「」 use 1 （ proxy-pool-cache.ts）： that 1 
  // use / delete / Clash after will new ， this to ——
  // down not will in old up （ is 1 ， in 
  // down ， in after ）
  React.useEffect(() => subscribeProxyPool(() => { bump() }), [])

  // that 1 already not in need （、）「」—— otherwise down 
  // not to that 1 、「」， by （ store normalizeProviderFilter）
  React.useEffect(() => { normalizeProviderFilter() }, [store.version])

  // set「statisticscurrent 」 after ， need by new 。
  // store（ localStorage not storage ），
  // this ****（set / 1 ） 。
  React.useEffect(() => {
    const onStorage = (event: StorageEvent) => {
      if (event.key === null || event.key === GROUP_TOTAL_FILTERED_KEY) syncGroupTotalPreference()
    }
    window.addEventListener('storage', onStorage)
    return () => window.removeEventListener('storage', onStorage)
  }, [])

  // set must in after ： by need `.batch-actions`，
  // syncStaticHead need already in （ by id config）。 use effect is as 
  // and by in after before （ is flushSync ）。
  //
  // need 1 （forceTick， not is etc store —— that in 
  // effect ， this 1 no ）： before that 1 is 「」， use 
  // / must in to and up （ old in ， is for ）。
  React.useLayoutEffect(() => {
    if (!colSettingsRegistered) {
      // wbColSettings no up （ in glob in before ， is ）：
      // 1 ， this empty「」
      if (!shared().wbColSettings?.register) {
        window.clearTimeout(colSettingsRetry)
        colSettingsRetry = window.setTimeout(() => setRetryTick(tick => tick + 1), 50)
        return
      }
      colSettingsRegistered = true
      colHandle = shared().wbColSettings?.register({
        id: 'accounts',
        label: t('账号表'),
        columns: ACCOUNT_COLUMNS.map(column => ({
          key: column.key,
          label: column.label,
          align: column.align,
          legacyAlign: column.legacyAlign,
        })),
        // is （「 / 」 that ）： in before ，
        // in 「」（set is 「 this 」， and that 「 for 」
        // by not is 1 ，）
        mount: () => document.querySelector('#batch-bar .batch-actions'),
        onChange: requestRender,
      }) || null
      if (colHandle) forceTick()
      return
    }
    syncHead()
  })
  // ： when to 1 this ， up effect 1 
  const [, setRetryTick] = React.useState(0)
  React.useEffect(() => () => window.clearTimeout(colSettingsRetry), [])

  // （pointerdown / dblclick ） in up 1 can ：
  // after valid（ to ， not ）
  React.useEffect(() => {
    bindColumnGrips(listRef.current, requestRender)
  }, [])

  const visibleIds = shown.map(account => account.id)
  const allPicked = visibleIds.length > 0 && visibleIds.every(id => store.selected.has(id))
  const somePicked = visibleIds.some(id => store.selected.has(id))
  /** in in ： will used for，must， not can */
  const hiddenByFilter = [...store.selected].filter(id => !visibleIds.includes(id)).length
  const active = store.selected.size > 0
  // 「 / 」 is ****（ and by use for 1 ）：
  // can ， that will and for not up （ setAllGroupsCollapsed ）
  const allCollapsed = allGroups.length > 0 && allGroups.every(group => store.collapsed.has(group.provider))
  /** 、， by 1 store 。 */
  const [filterOpen, setFilterOpen] = React.useState(false)
  /** current ： to 「」 by up use ， use */
  const activeFilterCount =
    (store.filter.provider !== 'all' ? 1 : 0)
    + (String(store.filter.q || '').trim() ? 1 : 0)
    + (store.filter.enabled !== 'all' ? 1 : 0)
    + (store.filter.limit !== 'all' ? 1 : 0)
  /**
   * ： 「 to 」 1 **** to after 。
   *
   * from `members`（， ）， not is from DOM ： 
   * in DOM no has ， after need 「 1 」（ 1 will 400）。
   * down by to （ by in ）， this ；
   * / / when by （ ）。
   */
  /** need ：`all` use ****（ not ， 
   * in ， 1 will after by 「 not 」400 ）；`displayed` use 
   * down （ by ）「 in can before 」 ；`renderedRows`
   * to 1 —— by ** **（ that 1 ）， onDrop。 */
  const renderedRows = new Map<string, string[]>()
  for (const group of groups) {
    renderedRows.set(group.provider,
      pageRows.filter(account => providerOf(account) === group.provider).map(account => account.id))
  }
  const dragRef = React.useRef({ displayed: groups, all: allGroups, renderedRows })
  React.useEffect(() => { dragRef.current = { displayed: groups, all: allGroups, renderedRows } })
  /** （ 1 、 new ， down effect） */
  const dragHandlers = React.useRef({
    locked: false,
    lockReason: '',
    onDrop: (_drop: QueueDrop) => {},
  })
  const onDrop = React.useCallback((drop: QueueDrop) => {
    const { displayed, all: current } = dragRef.current
    if (drop.kind === 'group') {
      if (dragHandlers.current.locked) return
      const from = current.findIndex(group => group.provider === drop.provider)
      const visibleFrom = displayed.findIndex(group => group.provider === drop.provider)
      if (from < 0 || visibleFrom < 0) return
      const visible = displayed.filter(group => group.provider !== drop.provider)
      if (!visible.length || drop.index < 0 || drop.index > visible.length || drop.index === visibleFrom) return
      const moved = current[from]
      const rest = current.filter((_, position) => position !== from)
      // index by after can ； in after 1 can after 。
      const anchor = visible[drop.index] || visible[visible.length - 1]
      const at = rest.findIndex(group => group.provider === anchor.provider)
      if (at < 0) return
      const to = at + (drop.index === visible.length ? 1 : 0)
      if (to === from) return
      const next = [...rest.slice(0, to), moved, ...rest.slice(to)]
      // not ； then by 。
      const queue = next.flatMap(group => group.queueMembers.map(account => account.id))
      void reorderQueue(queue, t('{name} 移到第 {rank} 顺位', { name: moved.label, rank: to + 1 }))
      return
    }
    const orderRules = activeGroupSortRules(drop.provider, groupSortRules(drop.provider))
    if (orderRules.length > 0) {
      toast(manualOrderBlockedMessage(orderRules), 'err')
      return
    }
    const group = current.find(item => item.provider === drop.provider)
    if (!group) return
    // then when not ； after is 。
    const from = group.queueMembers.findIndex(account => account.id === drop.accountId)
    if (from < 0) return
    const moved = group.queueMembers[from]
    const rest = group.queueMembers.filter((_, position) => position !== from)
    // down ** ** that （ / / will 1 ），
    // need is members —— by ： to N before = members
    // that in down before ；N etc = to （ in ）。
    // from dragRef （ not is group.accounts can ）： 1 
    // when ，DOM has that 1 ，must by 「 」，otherwise 1 。
    // and （displayed → all） 1 ： can and is ， not 。
    const visible = dragRef.current.renderedRows.get(drop.provider) || []
    const anchor = drop.index < visible.length ? visible[drop.index] : null
    const at = anchor === null ? -1 : rest.findIndex(account => account.id === anchor)
    const to = anchor === null || at < 0 ? rest.length : at
    if (to === from) return
    const nextMembers = [...rest.slice(0, to), moved, ...rest.slice(to)]
    const next = current.map(item => item.provider === drop.provider
      ? { ...item, members: nextMembers, queueMembers: nextMembers }
      : item)
    const queue = next.flatMap(item => item.queueMembers.map(account => account.id))
    void reorderQueue(queue, t('{name} 移到 {group} 组内第 {rank} 位', {
      name: String(moved.name || moved.id), group: t(group.label), rank: to + 1,
    }))
  }, [])

  // （ / ）： in up ， after valid。
  // 1 （ and 1 ）—— ref new ： and 
  // will ， if ， 1 up 1 。
  React.useEffect(() => { dragHandlers.current = { locked: groupLocked, lockReason, onDrop } })
  React.useEffect(() => {
    bindQueueDrag(listRef.current, {
      isLocked: () => dragHandlers.current.locked,
      onLockedAttempt: () => toast(
        t('当前视图不能调整组顺位：{reason} 之后再拖动', { reason: dragHandlers.current.lockReason }),
        'err',
      ),
      isRowLocked: provider => activeGroupSortRules(provider, groupSortRules(provider)).length > 0,
      onRowLockedAttempt: provider => toast(
        manualOrderBlockedMessage(activeGroupSortRules(provider, groupSortRules(provider))),
        'err',
      ),
      onDrop: drop => dragHandlers.current.onDrop(drop),
    })
  }, [])

  /** ： by key （ up down ， its is 「 + up down 」 ） */
  function cell(key: string, account: AccountRecord, ctx: ReturnType<typeof rowContext>) {
    switch (key) {
      case 'pick':
        return (
          <Checkbox checked={ctx.picked} data-pick={account.id} title={t('勾选后可批量操作')}
            aria-label={t('勾选后可批量操作')}
            onCheckedChange={next => togglePick(account.id, next)} />
        )
      case 'order':
        return <OrderCell account={account} rank={ctx.rank} sortActive={sortActiveIn(account)} />
      case 'account':
        return <AccountCell account={account} namesHidden={store.namesHidden} />
      case 'proxy':
        return <ProxyCell account={account} />
      case 'connections':
        return <ConnectionsCell account={account} />
      case 'status':
        return <StatusCell account={account} />
      case 'limits':
        return <LimitsCell account={account} onClear={model => void clearLimits(account.id, model)} />
      case 'expiry':
        return <ExpiryCell account={account} />
      case 'usage':
        return <UsageCell account={account} />
      case 'actions':
        return <ActionsCell account={account} atFront={ctx.seat.position <= 1} />
      default:
        return null
    }
  }

  return (
    <>
      <Card render={<section />} className='panel account-table'>
        <div className='toolbar'>
          {/* 「」： in ≤767px （ by CSS ）。
               by up is current —— after ，
               this must can ，otherwise use will as 。 */}
          <Button size='default' variant='ghost' type='button' className='acct-filter-open hidden max-[767px]:inline-flex'
            onClick={() => setFilterOpen(true)}
            aria-label={t('打开筛选')}>
            {t('筛选')}
            {activeFilterCount > 0 ? <Badge variant='brand' shape='tag' className='acct-filter-badge'>{activeFilterCount}</Badge> : null}
          </Button>

          {/* ， not will 。 */}
          <ResponsiveSheet open={filterOpen} onOpenChange={setFilterOpen} title={t('筛选')}
            inlineClassName='contents' bodyClassName='acct-filter-body'
            actions={<Button variant='ghost' size='sm' disabled={activeFilterCount === 0} onClick={() => clearFilter()}>{t('清空')}</Button>}>
          {/* ： 「 1 」 in up （ 1 1 ），「 1 」 is 
               use —— 1 、 not its that 。 use down not is by ：
               is **** （ after 1 1 ）， by will 
               1 ；（`WorkBuddy （14）`）， is 「 has 、 has 」
               not use can to 。 and store as （） */}
          <div className='group' data-provider-group='1'>
            <span className='label'>{t('提供商')}</span>
            <Select value={store.filter.provider} onValueChange={value => setProviderFilter(String(value))}>

              <SelectTrigger className='max-w-none min-w-0 w-full min-[768px]:max-w-[200px] min-[768px]:min-w-[130px] min-[768px]:w-auto' aria-label={t('按提供商筛选账号')}>
                <SelectValue>
                  {providerOptions.find(item => item.value === store.filter.provider)?.label || t('全部')}
                </SelectValue>
              </SelectTrigger>
              <SelectContent>
                {providerOptions.map(item => (
                  <SelectItem key={item.value} value={item.value}>{item.label}</SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className='divider' data-provider-divider='1' />

          {/* ： after 「 1 」 is ， use in 
               by / / 。 and use 1 can 
              （accounts-domain matchesFilter）， in in */}
          <div className='group'>
            <span className='label'>{t('搜索')}</span>
            <Input className='acct-search' type='search' value={store.filter.q}
              placeholder={t('账号名 / 邮箱 / 备注')} aria-label={t('搜索账号')}
              onChange={event => setSearchFilter(event.currentTarget.value)} />
          </div>
          <div className='divider' />

          <div className='group'>
            <span className='label'>{t('状态')}</span>
            {/* （ / ） is **** （ and store as ）。
                 by （ filterCounts ）： is 1 ，
                （ and ， not ）； is ，「
                 after 」 。 in 「 use 」 when —— already use not 
                 not already */}
            <SegmentedControl options={[
              { value: 'all', label: t('全部'), count: counts.enabledAll ?? 0 },
              { value: 'enabled', label: t('启用'), count: counts.enabled ?? 0 },
              { value: 'disabled', label: t('禁用'), count: counts.disabled ?? 0 },
            ]} value={store.filter.enabled} onValueChange={value => setSegmentFilter('enabled', value)}
              aria-label={t('启用状态')} />
          </div>

          <div className='group'>
            <span className='label'>{t('限额')}</span>
            <SegmentedControl options={[
              { value: 'all', label: t('全部'), count: counts.limitAll ?? 0 },
              { value: 'normal', label: t('正常'), count: counts.normal ?? 0, disabled: store.filter.enabled === 'disabled' },
              { value: 'limited', label: t('已限流'), count: counts.limited ?? 0, disabled: store.filter.enabled === 'disabled' },
            ]} value={store.filter.limit} onValueChange={value => setSegmentFilter('limit', value)}
              aria-label={t('限额状态')} />
          </div>
          </ResponsiveSheet>

          <div className='actions'>
            <Button id='btn-refresh-accounts' variant='outline' disabled={store.listBusy}
              title={t('重新读取账号列表和插件状态，不刷新 Token 或查询余额')}
              onClick={() => void refreshList({ feedback: true })}>
              {store.listBusy ? t('刷新中…') : t('刷新')}
            </Button>
            {/* is 「」 ： 1 when ， not that 。
                「」 is —— use for is ， not by current（ store 
                setAllGroupsCollapsed ） */}
            <Button id='btn-fold-groups' variant='outline'
              disabled={!allGroups.length}
              title={allCollapsed ? t('展开全部分组') : t('折叠全部分组（只留组头）')}
              onClick={() => setAllGroupsCollapsed(allGroups.map(group => group.provider), !allCollapsed)}>
              {allCollapsed ? t('展开') : t('折叠')}
            </Button>
            {/* 「」 add-account.js（ by 1 id ）， this 
                 1 onClick is ****： 1 that no up ， by available
                （openModal / resetAddStep is etc ， use none use ） */}
            <Button id='btn-add-account-2' variant='default' title={t('登录 / 导入一个新账号')}
              onClick={() => {
                shared().wbAddAccountModal?.open?.()
                shared().wbAccountAddForms?.syncAddProvider?.()
              }}>{t('添加账号')}</Button>
            <Button id='btn-query-usage' variant='outline'
              disabled={store.usageBusy || !all.some(supportsUsage)}
              title={t('查询全部账号的余额（含已禁用账号 —— 禁用只表示不参与转发）')}
              onClick={() => void queryAllUsage()}>{store.usageBusy ? t('查询中…') : t('查询余额')}</Button>
          </div>
        </div>

        {store.listError ? (
          <Alert variant='destructive' className='acct-load-error' role='alert'>
            {loaded
              ? t('刷新账号列表失败，当前仍显示上一次成功读取的数据：{message}', { message: store.listError })
              : t('账号列表加载失败，请点击「刷新」重试：{message}', { message: store.listError })}
          </Alert>
        ) : null}

        {loaded && all.length > 0 && hidden.total > 0 ? (
          <HiddenAccountsNotice hidden={hidden} compact />
        ) : null}


        <div className={cn('batch-bar', active && 'active')} id='batch-bar'>
          <Label className='batch-select-all'>
            <Checkbox id='batch-select-all' checked={allPicked} indeterminate={!allPicked && somePicked}
              disabled={!visibleIds.length} aria-label={t('全选当前筛选结果')}
              onCheckedChange={next => setAllPicked(visibleIds, next)} />
            <span id='batch-select-label'>
              {visibleIds.length ? t('全选当前筛选结果（{n} 个）', { n: visibleIds.length }) : t('没有可全选的账号')}
            </span>
          </Label>
          <span className='batch-count'>
            {t('已选')} <b id='batch-count'>{store.selected.size}</b> {t('个 · 共')} <b id='accounts-count'>{all.length}</b> {t('个')}
          </span>
          {/* already to 「 has 」， this 1 「current 」 */}
          <span className='batch-summary' id='accounts-group-summary'>
            {groups.length
              ? t('{groups} 家 · 显示 {rows} 行', { groups: groups.length, rows: visibleIds.length })
              : ''}
          </span>
          {/* ****（ by can ， down ） */}
          {groupLocked ? (
            <span className='batch-hidden' title={lockReason}>
              {t('组顺位已锁定')}
            </span>
          ) : null}
          {filterActive() ? (
            <Button id='btn-clear-filter' variant='ghost' size='xs'
              title={t('清空搜索与三个筛选档位，回到全部账号')}
              onClick={() => clearFilter()}>{t('清除筛选')}</Button>
          ) : null}
          {hiddenByFilter ? (
            <span className='batch-hidden' id='batch-hidden-hint'>
              {t('另有 {n} 个已勾选账号被当前筛选隐藏，仍会参与操作', { n: hiddenByFilter })}
            </span>
          ) : null}
          <div className='batch-actions'>
            {/* 「set」 by by wbColSettings.register this before （，
                 by that ，） */}
            <Button id='btn-batch-open' variant='outline' disabled={!active}
              onClick={() => openBatchDialog([...store.selected], 'enable')}>{t('批量操作')}</Button>
            <Button id='btn-batch-clear' variant='outline' disabled={!active}
              onClick={() => clearSelection()}>{t('取消选择')}</Button>
          </div>
        </div>

        <div className='acct-scroll' id='account-list' ref={listRef}>
          {!loaded ? (
            <Empty className='empty'>
              {store.listError
                ? t('账号列表加载失败，请点击「刷新」重试')
                : <><Spinner />{t('正在加载…')}</>}
            </Empty>
          ) : !all.length ? (
            hidden.total > 0
              ? <HiddenAccountsNotice hidden={hidden} />
              : <Empty className='empty'>{t('暂无账号，请点击右上角「添加账号」')}</Empty>
          ) : !shown.length ? (
            <Empty className='empty'>
              {t('当前筛选条件下没有账号')}
              {filterActive() ? (
                <Button size='default' variant='ghost' type='button' className='link-btn' onClick={() => clearFilter()}>{t('清除筛选')}</Button>
              ) : null}
            </Empty>
          ) : (
            <Table container={false} className='acct-table' ref={setTableElement}
              style={{ minWidth: `${tableMinWidth(columns.map(column => column.key))}px` }}>
              {/* up that min-width is 「 to FLEX_MIN_WIDTH、」 that ，
                   by （ not by CSS ， accounts-columns tableMinWidth） */}
              <ColGroup />
              <TableHead namesHidden={store.namesHidden} allPicked={allPicked} somePicked={somePicked}
                disabled={!visibleIds.length} onToggleAll={next => setAllPicked(visibleIds, next)} />
              {groups.map(group => {
                const collapsed = store.collapsed.has(group.provider)
                const rows = pageRows.filter(account => providerOf(account) === group.provider)
                return (
                  <TableBody key={group.provider} className='acct-group' data-provider={group.provider}>
                    <GroupBandRow group={group} collapsed={collapsed} colSpan={columns.length} />
                    {collapsed ? null : rows.map(account => {
                      // **** （ not is current / current can ）：
                      // when will ， is 「 in this 1 」 this 
                      const rank = group.members.findIndex(member => member.id === account.id) + 1
                      const ctx = rowContext(account, seatMap, rank > 0 ? rank : 1)
                      return (
                        <React.Fragment key={account.id}>
                          <TableRow className={cn('acct-row', !isEnabled(account) && 'disabled', ctx.picked && 'selected')}
                            data-id={account.id} data-provider={group.provider}
                            data-desktop={isDesktopAccount(account) ? '1' : undefined}>
                            {columns.map(column => (
                              <TableCell key={column.key} className={`cell-${column.key} ta-${column.align}`}>
                                {cell(column.key, account, ctx)}
                              </TableCell>
                            ))}
                          </TableRow>
                        </React.Fragment>
                      )
                    })}
                  </TableBody>
                )
              })}
            </Table>
          )}
        </div>

        {/* is （ / / / ， use table-shell.tsx）。
             that （ is / / as ）
             by use need —— in not 。 */}
        <TableFooter
          total={shown.length}
          range={paging.paged ? { start: paging.rangeStart, end: paging.rangeEnd } : null}
          page={paging.page}
          pageCount={paging.pageCount}
          size={paging.size}
          onSizeChange={paging.setSize}
          onPageChange={paging.goto}
        />
      </Card>
      <AccountsDialogs />
    </>
  )
}

/** （ syncStaticHead use ）。 use ref ，
     —— will in when down up ， 1 empty */
function setTableElement(el: HTMLTableElement | null): void {
  tableEl = el
}

/** （colgroup）： ，`data-col` and 、set、 */
function ColGroup() {
  const widths = columnWidths()
  const col = (key: string) => (
    <col key={key} data-col={key} style={widths[key] ? { width: `${widths[key]}px` } : undefined} />
  )
  return (
    <colgroup>
      {col('pick')}
      {col('order')}
      {col('account')}
      {col('proxy')}
      {col('connections')}
      {col('status')}
      {col('limits')}
      {col('expiry')}
      {col('usage')}
      {col('actions')}
    </colgroup>
  )
}

/* ─── ： index.html has ─────── */

const PAGE_SELECTOR = '.page[data-page="accounts"]'

let root: ReturnType<typeof createRoot> | null = null

/**
 * React root in `.page[data-page="accounts"]` up （ not div： CSS use 
 * `.page[data-page="accounts"] .panel` this ， in 1 will ）。
 *
 * **must is **（flushSync， by file）： after update。
 */
function mount(): void {
  if (root) return
  const section = document.querySelector<HTMLElement>(PAGE_SELECTOR)
  if (!section) return
  section.replaceChildren()
  root = createRoot(section)
  flushSync(() => { root!.render(<AccountsPage />) })
}

// for and in when （app.js refresh() is ，
// report.js / add-account.js this its after —— not to will not ）
installAccountsApi()
startConnectionsPolling()

// to when （ island-lazy）： not in ， not 。
mountOnPage('accounts', PAGE_SELECTOR, mount)
