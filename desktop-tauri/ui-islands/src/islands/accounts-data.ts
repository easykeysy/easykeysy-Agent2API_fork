import {
  errorMessage, POOL_VALUE_PREFIX, shared, toast,
  type AccountRecord,
  type UsageEntry,
} from './accounts-shared'
import {
  isDesktopAccount, isEnabled, isLimitedNow, isRateLimited, supportsUsage,
} from './accounts-domain'
import * as domain from './accounts-domain'
import { t } from '../i18n'
import {
  allAccounts, bump, findAccount, getStore, isPicked, patch,
} from './accounts-store'
import { invalidateProxyPoolCache, proxyPoolSnapshot, refreshProxyPool } from './proxy-pool-cache'
import { installListRefresh, refreshList } from './accounts-refresh'
import { syncAccountSorting } from './accounts-sort'
import { confirmBatchResult, refreshSavedAccounts, updateAccountConfirmed } from './accounts-update'
import {
  applyBalances, pruneUsageCache, queryAllUsage, queryUsageFor, refreshUsageAfterCheckin,
  syncBalancesSnapshot, tokenUsageOf, usageEntryOf,
} from './accounts-usage'

export * from './accounts-usage'

export { refreshList } from './accounts-refresh'

// ：（accounts-page / panels / dialogs）file this 1 path
export * from './accounts-store'
export { saveGroupSortRules as setGroupSortRules } from './accounts-sort'

/* ─── （「」 ， down ）─── */

export {
  acceptProxyPool, ensureProxyPoolOptions, invalidateProxyPoolCache, poolError,
  proxyPoolOptions, proxyPoolSnapshot, refreshProxyPool, subscribeProxyPool,
} from './proxy-pool-cache'

/* ─── ─────────────────────────────────── */

export function openSettingsDialog(id: string): void {
  if (!findAccount(id)) { toast(t('账号不存在，请刷新后重试'), 'err'); return }
  patch({ dialog: { kind: 'settings', id } })
}

export function closeDialog(): void {
  patch({ dialog: null })
}

export function openBatchDialog(ids: string[], action = 'enable'): void {
  const selected = (Array.isArray(ids) ? ids : []).filter(id => findAccount(id))
  if (!selected.length) { toast(t('请先勾选要操作的账号'), 'err'); return }
  patch({ dialog: { kind: 'batch', ids: selected, action } })
}

/**
 * 1 **set**（ up that by ）。
 *
 * not 「 this 1 in not in 」 in ： is from ，
 * not not to ；set has （ will default / new ） in 
 * after has ， 1 no 。
 */
export function openGroupDialog(provider: string): void {
  if (!provider) return
  patch({ dialog: { kind: 'group', provider } })
}

/* ─── delete after ───────────────────── */

/** already delete （app.js refresh after use ） */
export function refreshCaches(validIds: Set<string>): void {
  let touched = pruneUsageCache(validIds)
  const connections = new Map(getStore().connections)
  for (const id of [...connections.keys()]) if (!validIds.has(id)) { connections.delete(id); touched = true }
  const inflight = new Set(getStore().usageInflight)
  for (const id of [...inflight]) if (!validIds.has(id)) { inflight.delete(id); touched = true }
  const selected = new Set(getStore().selected)
  for (const id of [...selected]) if (!validIds.has(id)) { selected.delete(id); touched = true }
  if (touched) patch({ connections, usageInflight: inflight, selected })
}

/* ─── （ when ，2 seconds 1 ）────────────────────
 * and OmniProxy up 「」 1 ：** in use this request**
 * （ 1 request in when ， after core::upstream::connections）。
 * 1 2 seconds not is app.js that 20 seconds 1 ： in 「 in 」。
 * after that is （ not 、 not ）， this 。 */

/** 2 seconds：「 in 」， not DevTools */
export const CONNECTIONS_POLL_MS = 2000

export function connectionsOf(id: string): number {
  return getStore().connections.get(id) || 0
}

/**
 * 1 。failure：2 seconds 1 ， after not can when toast will ；
 * and up 1 empty（ use to is 「 in 」）。
 *
 * ── no not store（）──────────────────────────
 * this 2 seconds 1 ，「 has request in 」 is when ：
 * `connections` is 1 empty。 not ， store `patch` will 1 
 * new （`useSyncExternalStore` by use 「 no 」）—— is 2 seconds
 * 1 ，43 × ， up 1 no 。 this 1 ，
 * return： that 2 seconds 1 「」 in empty when 。
 *
 * and must 1 （ is 「 count > 0 」）—— Map 
 * is 1 ，「key + 」 etc etc 「 this and up 1 」。
 */
export async function syncConnections(): Promise<boolean> {
  try {
    const data = await shared().workbuddyDesktop?.getAccountConnections?.()
    const counts = data?.counts && typeof data.counts === 'object' ? data.counts : {}
    const connections = new Map<string, number>()
    for (const [id, value] of Object.entries(counts)) {
      const count = Number(value) || 0
      if (count > 0) connections.set(id, count)
    }
    if (sameConnections(connections, getStore().connections)) return true
    patch({ connections })
    return true
  } catch {
    // ： down 1 （ and app.js refresh / syncLogsBadge 1 ）
    return false
  }
}

/** is （key and ； syncConnections ） */
function sameConnections(
  next: ReadonlyMap<string, number>,
  prev: ReadonlyMap<string, number>,
): boolean {
  if (next.size !== prev.size) return false
  for (const [id, count] of next) {
    if (prev.get(id) !== count) return false
  }
  return true
}

/**
 * when （ 1 ）。 in ** can ** when request： to 、 or 
 * when not after —— and requests-page / logs-panel 1 。
 */
let connectionsTimer: ReturnType<typeof setInterval> | null = null
export function startConnectionsPolling(): void {
  if (connectionsTimer) return
  connectionsTimer = setInterval(() => {
    if (document.hidden || shared().wbApp?.currentPage !== 'accounts') return
    void syncConnections()
  }, CONNECTIONS_POLL_MS)
}

/* ─── （enabled / / / ）─────
 * 「」 that 1 down ： not by ， by 
 * （`reorderQueue`）。 after `PATCH priority` / `move` in （ client in use ），
 * is not 。 */

/**
 * enabled / use （ and ⋯ 1 1 ）。
 *
 * Why not use app.js runAccountAction： that switch 
 * switch / refresh / remove， action will not 。
 * use updateAccount PATCH —— after apply_patch ， this enabled。
 */
export async function setAccountEnabled(id: string, enabled: boolean): Promise<void> {
  try {
    await updateAccountConfirmed(id, { enabled })
    toast(enabled ? t('✅ 已启用') : t('✅ 已禁用'))
    try { await refreshSavedAccounts() }
    catch (error) { toast(t('账号已保存，但列表刷新失败：{error}', { error: errorMessage(error) }), 'err') }
  } catch (error) {
    toast(t('操作失败：{message}', { message: errorMessage(error) }), 'err')
    // failure when ： up not can 1 「 already 」 
    void shared().wbApp?.refresh?.()
  }
}

/**
 * ****（ 1 ）： new to after 。
 *
 * use to is 「 → 」 after id （ from to ）， after by 
 * priority 0、1、2… —— 1 request 1 ， not in PATCH
 * in （ in failure will down ， that will 「 1 」 this not ）。
 *
 * failuremust new ： when is ** use **， is old ，
 * not ， down 1 new will to 「」。
 */
export async function reorderQueue(ids: string[], note?: string): Promise<boolean> {
  if (!ids.length) return false
  try {
    await shared().workbuddyDesktop?.reorderAccounts?.(ids)
    toast(note ? t('✅ 已保存：{note}', { note }) : t('✅ 顺序已保存'))
    await shared().wbApp?.refresh?.()
    return true
  } catch (error) {
    toast(t('顺序未保存：{message}', { message: errorMessage(error) }), 'err')
    void shared().wbApp?.refresh?.()
    return false
  }
}

/**
 * enabled / use （ ⋯ ）： has ， 1 1 。
 * and 1 （`PATCH enabled`）， is ids 1 。
 */
export async function setGroupEnabled(ids: string[], enabled: boolean): Promise<void> {
  if (!ids.length) return
  try {
    const bridge = shared().workbuddyDesktop
    if (!bridge?.batchAccounts) throw new Error(t('批量操作接口不可用，请重新打开页面'))
    const payload = { action: enabled ? 'enable' : 'disable', ids }
    const data = confirmBatchResult(await bridge.batchAccounts(payload), payload)
    const count = data.ok?.length || 0
    const changed = data.ok?.filter(item => item.changes?.length).length || 0
    const failed = data.failed?.length || 0
    if (failed || data.skipped?.length) {
      toast(t('{verb}完成：成功 {ok} 个，未变更 {unchanged} 个，跳过 {skip} 个，失败 {fail} 个', {
        verb: enabled ? t('启用') : t('禁用'), ok: changed, unchanged: count - changed,
        skip: data.skipped?.length || 0, fail: failed,
      }), failed ? 'err' : 'ok')
    } else {
      toast(enabled ? t('✅ 已启用 {n} 个账号', { n: count }) : t('✅ 已禁用 {n} 个账号', { n: count }))
    }
    try { await refreshSavedAccounts() }
    catch (error) { toast(t('批量操作已保存，但列表刷新失败：{error}', { error: errorMessage(error) }), 'err') }
  } catch (error) {
    toast(t('操作失败：{message}', { message: errorMessage(error) }), 'err')
    void shared().wbApp?.refresh?.()
  }
}

/** ：（model） or 。 after return new ， to new for 。 */
export async function clearLimits(id: string, model?: string): Promise<void> {
  try {
    await shared().workbuddyDesktop?.clearRateLimits?.(id, model || null)
    toast(model ? t('✅ 已清除 {model} 的限流标记', { model }) : t('✅ 已清除该账号全部限流标记'))
    await shared().wbApp?.refresh?.()
  } catch (error) {
    toast(t('清除失败：{message}', { message: errorMessage(error) }), 'err')
  }
}

/**
 * in （`value` is down ）。 down has ：
 * '' → （proxy = null）
 * `pool:<proxyId>` → use 「」 
 * `__proxy_*__` → 「current 」 and 「set」，
 * not is 1 （ in before ）
 *
 * 1 after ， this ** not ** `{source:'clash'}` —— that 
 * ， use in this down 1 or will 。
 *
 * ── use Why in this 1 ─────────────────────────────
 * already use （`disabled`）， down ， down 
 * （ new 、 already ） can can to 1 already use 。 after 
 * `normalize_account_proxy` has ** not ** down （ not 
 * ）， this not will 「success → failure → 」， use 
 * to 1 「 already use 」。 in this ， and up is 1 。
 */
export async function applyProxyPick(id: string, value: string, fallback: string): Promise<void> {
  if (value === PROXY_CUSTOM_EDIT) {
    // ：（ down by use in —— is ，）
    openSettingsDialog(id)
    return
  }
  // currentconfig ： not is 1 
  if (value === PROXY_CUSTOM_CURRENT) return
  if (value === fallback) return
  if (value.startsWith(POOL_VALUE_PREFIX)) {
    const poolId = value.slice(POOL_VALUE_PREFIX.length)
    const item = (proxyPoolSnapshot() || []).find(entry => entry.id === poolId)
    if (item && item.enabled === false) {
      toast(t('「{name}」已在「网络代理」页禁用，请先启用它或换一条', {
        name: item.name || poolId,
      }), 'err')
      return
    }
  }
  try {
    const proxy = value === ''
      ? null
      : { source: 'pool', proxyId: value.slice(POOL_VALUE_PREFIX.length) }
    await updateAccountConfirmed(id, { proxy })
    toast(`✅ ${t('代理已更新')}`)
  } catch (error) {
    toast(t('保存失败：{message}', { message: errorMessage(error) }), 'err')
  } finally {
    // ：success this after ，failure down 
    try { await refreshSavedAccounts() }
    catch (error) { toast(t('刷新账号列表失败：{message}', { message: errorMessage(error) }), 'err') }
  }
}

/** down 「 not is uid」 ：currentconfig 、set */
export const PROXY_CUSTOM_CURRENT = '__proxy_custom__'
export const PROXY_CUSTOM_EDIT = '__proxy_custom_edit__'

/**
 * CodeArts 「」 already to 「 to in 」 （checkin-page.tsx
 * legacy `ui/codearts-welfare.js` `wbCodeArtsWelfare.start`），
 * not has this by —— this not in 。
 *
 * ZCode 「」 1 ： that by already （ and in 
 * to in 「」， that 1 is 1 `wbZcodePlans` ），
 * `openZcodePlans` 1 and 。
 */

/* ─── for （window）────────────────────── */

/** 1 up down （ by 、、 and ） */
export type RowContext = {
  /** （1 ）：⋯ 「 already in 1 」、 use 。
   * down ** not **（ up is ）， accounts-domain module header */
  seat: { position: number; total: number }
  /** （1 ）： `#N` is */
  rank: number
  picked: boolean
  usageEntry: UsageEntry
  connections: number
}

export function rowContext(
  account: AccountRecord,
  seatMap: Map<string, { position: number; total: number }>,
  rank = 1,
): RowContext {
  return {
    seat: seatMap.get(account.id) || { position: 1, total: 1 },
    rank,
    picked: isPicked(account.id),
    usageEntry: usageEntryOf(account),
    connections: connectionsOf(account.id),
  }
}

export type AccountsViewApi = {
  render(): void
  refreshList: typeof refreshList
  refreshCaches(validIds: Set<string>): void
  syncConnections(): Promise<boolean>
  /**
   * 「」 （ when ）。
   *
   * down is ， when by update； this 1 is 
   * —— can can in use （ 1 、）， in before 
   * can can 1 no new 。failure not ， by `poolError()` 「failure」。
   */
  refreshProxyPool(): Promise<void>
  applyBalances(
    balances: { results?: Array<Record<string, unknown>>; at?: unknown } | null | undefined,
  ): number
  syncBalancesSnapshot(): Promise<boolean>
  queryUsageFor(id?: string | null): Promise<unknown>
  queryAllUsage(): Promise<void>
  refreshUsageAfterCheckin(id?: string | null): Promise<void>
  supportsUsage: typeof supportsUsage
  isDesktopAccount: typeof isDesktopAccount
  isEnabled: typeof isEnabled
  isRateLimited: typeof isRateLimited
}

export type AccountsPanelApi = {
  open(id: string): void
  close(): void
  openBatch(ids: string[], action?: string): void
  closeBatch(): void
  syncAddProvider(): void
  invalidate(): void
}

/**
 * `window.wbAccountsModel` ： and ui/accounts-model.js ** 1 **，
 * ** HTML ** （statusTag / accountTags / moreMenuHtml /
 * limitPanelHtml / checkinPanelHtml）—— 1 React ，none
 * use （）。accountTags etc in （return， by Badge）。
 * after `isLimited`（「 already 」 from isRateLimited ） is 1 ：
 * in file， not to —— for 「 when 」， in 。
 */
type AccountsModelApi = {
  DEFAULT_PROVIDER_ID: string
  RACCOON_PROVIDER_ID: string
  providerOf: typeof domain.providerOf
  providerFeatures: typeof domain.providerFeatures
  providerSummaries: typeof domain.providerSummaries
  identifierOf: typeof domain.identifierOf
  tokenExpiryOf: typeof domain.tokenExpiryOf
  supportsUsage: typeof domain.supportsUsage
  isDesktopAccount: typeof domain.isDesktopAccount
  supportsChat: typeof domain.supportsChat
  byPriorityOrder: typeof domain.byPriorityOrder
  typeLabel: typeof domain.typeLabel
  isEnabled: typeof domain.isEnabled
  isRateLimited: typeof domain.isRateLimited
  /** 「 already 」 （isLimitedNow： or limit）， and Token by file */
  isLimited: (account: AccountRecord | null | undefined) => boolean
  accountEdition: typeof domain.accountEdition
  matchEnabled: typeof domain.matchEnabled
  matchLimit: typeof domain.matchLimit
  visibleAccounts: typeof domain.visibleAccounts
  filterCounts: typeof domain.filterCounts
  positionMap: typeof domain.positionMap
  activeLimits: typeof domain.activeLimits
  editionSuffix: typeof domain.editionSuffix
  formatResetText: typeof domain.formatResetText
}

declare global {
  interface Window {
    /** （ ui/accounts-view.js；app.js / providers.js / tasks-panel use ） */
    wbAccountsView?: AccountsViewApi
    /** （ ui/accounts-model.js；app.js / report.js / models-fetch-modal use ） */
    wbAccountsModel?: AccountsModelApi
    /** set / （ ui/account-panel.js；app.js:623 / :645 use ） */
    wbAccountPanel?: AccountsPanelApi
  }
}

/** for （ up ； not is `export *`， up ） */
const ACCOUNTS_MODEL_API: AccountsModelApi = {
  DEFAULT_PROVIDER_ID: domain.DEFAULT_PROVIDER_ID,
  RACCOON_PROVIDER_ID: domain.RACCOON_PROVIDER_ID,
  providerOf: domain.providerOf,
  providerFeatures: domain.providerFeatures,
  providerSummaries: domain.providerSummaries,
  identifierOf: domain.identifierOf,
  tokenExpiryOf: domain.tokenExpiryOf,
  supportsUsage: domain.supportsUsage,
  isDesktopAccount: domain.isDesktopAccount,
  supportsChat: domain.supportsChat,
  byPriorityOrder: domain.byPriorityOrder,
  typeLabel: domain.typeLabel,
  isEnabled: domain.isEnabled,
  isRateLimited: domain.isRateLimited,
  isLimited: account => isLimitedNow(
    account,
    account ? usageEntryOf(account) : undefined,
    tokenUsageOf(account),
  ),
  accountEdition: domain.accountEdition,
  matchEnabled: domain.matchEnabled,
  matchLimit: domain.matchLimit,
  visibleAccounts: domain.visibleAccounts,
  filterCounts: domain.filterCounts,
  positionMap: domain.positionMap,
  activeLimits: domain.activeLimits,
  editionSuffix: domain.editionSuffix,
  formatResetText: domain.formatResetText,
}

/**
 * for 。**must in when **：app.js refresh() is （ await
 * after to ）， report.js / add-account.js this its after —— 
 * not to will not 。
 */
export function installAccountsApi(): void {
  installListRefresh()
  window.wbAccountsView = {
    render: () => { syncAccountSorting(); bump() },
    refreshList,
    refreshCaches,
    syncConnections,
    refreshProxyPool,
    applyBalances,
    syncBalancesSnapshot,
    queryUsageFor,
    queryAllUsage,
    refreshUsageAfterCheckin,
    supportsUsage,
    isDesktopAccount,
    isEnabled,
    isRateLimited,
  }
  window.wbAccountsModel = ACCOUNTS_MODEL_API
  window.wbAccountPanel = {
    open: openSettingsDialog,
    close: closeDialog,
    openBatch: openBatchDialog,
    closeBatch: closeDialog,
    /** 「」 when available： by providers need and to WorkBuddy */
    syncAddProvider: () => shared().wbAccountAddForms?.syncAddProvider?.(),
    
    // ， not ：refresh() will this ， has 
    // use to 。 to when showPage will refreshProxyPool() 。
    invalidate: () => { invalidateProxyPoolCache() },
  }
}
