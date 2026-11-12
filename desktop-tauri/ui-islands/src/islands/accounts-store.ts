

import {
  shared, type AccountRecord, type AccountsSnapshot, type AccountSorting,
  type HiddenAccountProvider, type HiddenAccountsSummary,
} from './accounts-shared'
import {
  accountGroups, filterCounts, matchesFilter, positionMap, providerSummaries,
  defaultGroupSortRules,
  type AccountFilter, type AccountGroup, type GroupSortRule, type TokenUsageLookup, type UsageLookup,
} from './accounts-domain'

/** ：set（ ）/ （ + ）/ set（）， null */
export type AccountsDialog =
  | { kind: 'settings'; id: string }
  | { kind: 'batch'; ids: string[]; action: string }
  | { kind: 'group'; provider: string }
  | null

export type AccountsStore = {
  /** +1；React 「 this 」（ for when ） */
  version: number
  listBusy: boolean
  listError: string
  filter: AccountFilter
  /** （provider id）。 after 「」， that by */
  collapsed: ReadonlySet<string>
  selected: ReadonlySet<string>
  connections: ReadonlyMap<string, number>
  dialog: AccountsDialog
  namesHidden: boolean
  /** in （ by and use ） */
  usageBusy: boolean
  /** in （ up that 「」 by ） */
  usageInflight: ReadonlySet<string>
  /**
   * （available / available） is statistics**current / after can **。
   * default（= by statistics， accounts-panels groupBalanceOf）； in 
   * set → → 。 and namesHidden 「 this 」 。
   */
  groupTotalFiltered: boolean
  /** then by ；onlyused for old set。 */
  groupSort: Readonly<Record<string, GroupSortRule[]>>
  sorting: AccountSorting | null
  sortBusy: boolean
}

/* ─── ───────────────────────── */

const FILTERS_KEY = 'workbuddy-desktop-accounts-filters'
/** ： and 1 「 this 」， */
const COLLAPSED_KEY = 'workbuddy-desktop-accounts-collapsed'
/** （「」 that ）： after 。
 * this is 「 this 」 ， and 1 ， localStorage */
const NAMES_HIDDEN_KEY = 'workbuddy-desktop-accounts-names-hidden'
/**
 * ：`'1'` = statisticscurrent after can ， / its = statistics。
 *
 * this is and set** use **：file `readGroupTotalFiltered` /
 * `setGroupTotalFiltered` ， not 1 —— ， when 1 
 * is 「 not 」。
 */
export const GROUP_TOTAL_FILTERED_KEY = 'workbuddy-desktop-accounts-group-total-filtered'

/** old set ， not new then 。 */
export const GROUP_SORT_KEY = 'workbuddy-desktop-accounts-group-sort'

/** then （， not 1 ） */
function loadGroupSort(): Record<string, GroupSortRule[]> {
  try {
    const raw = JSON.parse(localStorage.getItem(GROUP_SORT_KEY) || '{}') as unknown
    if (!raw || typeof raw !== 'object' || Array.isArray(raw)) return {}
    const out: Record<string, GroupSortRule[]> = {}
    for (const [provider, value] of Object.entries(raw as Record<string, unknown>)) {
      if (!Array.isArray(value)) continue
      const rules: GroupSortRule[] = []
      for (const item of value) {
        if (!item || typeof item !== 'object') continue
        const entry = item as Record<string, unknown>
        if (entry.key === 'credit') {
          rules.push({ key: 'credit', on: entry.on !== false, dir: entry.dir === 'late' ? 'late' : 'soon' })
        } else if (entry.key === 'added') {
          rules.push({ key: 'added', on: entry.on !== false, dir: entry.dir === 'old' ? 'old' : 'new' })
        }
      }
      // then no not ： that is 「 to default」 not is 「 1 then not and 」
      if (rules.length) out[provider] = rules
    }
    return out
  } catch {
    return {}
  }
}

/**
 * 。
 *
 * provider / enabled / limit ：
 * · `q`（）** not ** —— down to 1 ， 
 * not in ， is as 「」 1 ；
 * · `provider` ****（ and 1 「 this 」）： that 
 * already no by `normalizeProviderFilter` 「」， not when ——
 * will 「 up is 」nonenone， can 。
 */
function loadFilter(): AccountFilter {
  const defaults: AccountFilter = { provider: 'all', enabled: 'all', limit: 'all', q: '' }
  const stored = shared().wbFilterMemory?.load(FILTERS_KEY, defaults) || defaults
  return {
    provider: String(stored.provider || 'all'),
    enabled: String(stored.enabled || 'all'),
    limit: String(stored.limit || 'all'),
    q: '',
  }
}

/** ： is provider id （，none） */
function loadCollapsed(): Set<string> {
  try {
    const raw = JSON.parse(localStorage.getItem(COLLAPSED_KEY) || '[]') as unknown
    return new Set(Array.isArray(raw) ? raw.map(item => String(item)).filter(Boolean) : [])
  } catch {
    return new Set()
  }
}

/** current（set need 1 ， and loadNamesHidden ） */
export function readNamesHidden(): boolean {
  return loadNamesHidden()
}

/** current（set） */
export function readGroupTotalFiltered(): boolean {
  return loadGroupTotalFiltered()
}

function loadNamesHidden(): boolean {
  try {
    return localStorage.getItem(NAMES_HIDDEN_KEY) === '1'
  } catch {
    return false
  }
}

/** ： has '1' 「 by 」， no / by （default） */
function loadGroupTotalFiltered(): boolean {
  try {
    return localStorage.getItem(GROUP_TOTAL_FILTERED_KEY) === '1'
  } catch {
    return false
  }
}

/* ─── store ─────────────────────────────────── */

let store: AccountsStore = {
  version: 0,
  listBusy: false,
  listError: '',
  filter: loadFilter(),
  collapsed: loadCollapsed(),
  selected: new Set<string>(),
  connections: new Map<string, number>(),
  dialog: null,
  namesHidden: loadNamesHidden(),
  usageBusy: false,
  usageInflight: new Set<string>(),
  groupTotalFiltered: loadGroupTotalFiltered(),
  groupSort: loadGroupSort(),
  sorting: null,
  sortBusy: false,
}

const listeners = new Set<() => void>()

export function subscribe(listener: () => void): () => void {
  listeners.add(listener)
  return () => { listeners.delete(listener) }
}

export function getStore(): AccountsStore {
  return store
}

export function patch(partial: Partial<AccountsStore>): void {
  store = { ...store, ...partial, version: store.version + 1 }
  listeners.forEach(listener => listener())
}

/** Map / Set after （ for use no ） */
export function bump(): void {
  patch({})
}

/* ─── from after （ 1 ）─────────── */

export function snapshot(): AccountsSnapshot | null | undefined {
  return shared().wbApp?.getState?.()?.accounts
}

export function allAccounts(): AccountRecord[] {
  const list = snapshot()?.accounts
  return Array.isArray(list) ? list : []
}

/** need ； old after has when use in 「」Reason。 */
export function hiddenAccounts(): HiddenAccountsSummary {
  const hidden = snapshot()?.hiddenAccounts
  const raw = hidden?.byProvider
  const byProvider: Record<string, number> = {}
  if (raw && typeof raw === 'object' && !Array.isArray(raw)) {
    for (const [provider, count] of Object.entries(raw)) {
      const value = Number(count)
      if (Number.isFinite(value) && value > 0) byProvider[provider] = value
    }
  }
  const details = new Map<string, HiddenAccountProvider>()
  for (const item of Array.isArray(hidden?.providers) ? hidden.providers : []) {
    if (!item || typeof item.id !== 'string' || !item.id) continue
    const reason = item.reason === 'not_installed' || item.reason === 'pack_disabled'
      || item.reason === 'provider_disabled' ? item.reason : 'not_loaded'
    details.set(item.id, {
      id: item.id,
      label: typeof item.label === 'string' && item.label ? item.label : item.id,
      count: Number(item.count) || 0,
      reason,
    })
  }
  const providers = Object.entries(byProvider).map(([id, count]): HiddenAccountProvider => {
    const detail = details.get(id)
    return {
      id,
      count,
      label: detail?.label || shared().wbProviders?.labelOf?.(id) || id,
      reason: detail?.reason || 'not_loaded',
    }
  }).sort((a, b) => b.count - a.count)
  const total = Number(hidden?.total)
  return { total: Number.isFinite(total) && total > 0 ? total : 0, byProvider, providers }
}

export function findAccount(id: string): AccountRecord | null {
  return allAccounts().find(account => account.id === id) || null
}

/** current can （ + ） and use accounts-domain 1 。
 * `usageOf`（） and `tokenUsageOf`（Token ） by use 
 * ： 「 not already / Token already 」 already ，
 * in accounts-data（file not to ， store⇄data ） */
export function visibleList(usageOf?: UsageLookup, tokenUsageOf?: TokenUsageLookup): AccountRecord[] {
  return groupView(usageOf, tokenUsageOf).groups.flatMap(group => group.accounts)
}

export type GroupView = {
  /** need （ by down then ） */
  groups: AccountGroup[]
  /** ****（ not then ）： need by （ after need 「 
   * 1 」， 1 400）， by —— accounts-page onDrop */
  allGroups: AccountGroup[]
}

/**
 * （ ）： = after ， can 。
 * and `visibleList` —— can by is can 。
 *
 * ── 1 1 「 not 」 then ： no has can ──────────────────
 * （ / / ） or this 1 not —— 1 has 
 * empty is 。 by ****：「enabled」 when use 1 not 
 * ，「」 when in （ that is 「」 this has ）。
 *
 * ⚠️ has then ：「 1 enabled no has 」 1 not 。 and 「 = 」
 * —— use 「」， that ** use ** （ is 
 * ） 1 ， up is 。 up 
 * 「N · enabled 0 · use N」， 1 is not is use by ， not this by 「」
 * 。 this then already delete。
 */
export function groupView(
  usageOf?: UsageLookup,
  tokenUsageOf?: TokenUsageLookup,
): GroupView {
  const allGroups = accountGroups(
    allAccounts(),
    account => matchesFilter(account, store.filter, usageOf, tokenUsageOf),
    usageOf,
    tokenUsageOf,
    groupSortRules,
    store.sorting || snapshot()?.sorting,
  )
  const groups = allGroups.filter(group => group.accounts.length > 0)
  return { groups, allGroups }
}

/** already ；not configured by current can default then 。 */
export function groupSortRules(provider: string): GroupSortRule[] {
  const effective = store.sorting?.rules[provider] || snapshot()?.sorting?.rules[provider]
  const saved = effective || store.groupSort[provider]
  if (saved) {
    const rules = saved.map(rule => ({ ...rule }))
    return [...rules, ...defaultGroupSortRules(provider).filter(rule => !rules.some(current => current.key === rule.key)).map(rule => ({ ...rule, on: false }))]
  }
  return defaultGroupSortRules(provider)
}

/** this 1 has no has **** then （set「default」 is can ） */
export function hasCustomGroupSort(provider: string): boolean {
  return Object.prototype.hasOwnProperty.call(store.sorting?.configured || store.groupSort, provider)
}

export function providerSummaryList(): Array<{ id: string; label: string; count: number }> {
  return providerSummaries(snapshot())
}

export function segmentCounts(usageOf?: UsageLookup, tokenUsageOf?: TokenUsageLookup): Record<string, number> {
  return filterCounts(allAccounts(), store.filter, usageOf, tokenUsageOf)
}

/** valid ， not 。 */
export function seats(): Map<string, { position: number; total: number }> {
  return positionMap(allAccounts(), store.sorting || snapshot()?.sorting)
}

/* ─── ─────────────────────────────────── */

function persistFilter(): void {
  // provider / enabled / limit：（q） is 1 ， not （ loadFilter）
  shared().wbFilterMemory?.save(FILTERS_KEY, {
    provider: store.filter.provider,
    enabled: store.filter.enabled,
    limit: store.filter.limit,
  })
}

/** ： / / / （ not ） */
export function setSearchFilter(value: string): void {
  if (store.filter.q === value) return
  patch({ filter: { ...store.filter, q: value } })
}

/** down ：'all' or provider id。 and 1 ， + */
export function setProviderFilter(value: string): void {
  const provider = value || 'all'
  if (store.filter.provider === provider) return
  patch({ filter: { ...store.filter, provider } })
  persistFilter()
}

/** ： + 。 and enabled —— 「 use 」 when / has 
 * not in ， is as 「」（ by options.disabled ） */
export function setSegmentFilter(key: 'enabled' | 'limit', value: string): void {
  const filter: AccountFilter = { ...store.filter, [key]: value }
  if (filter.enabled === 'disabled' && filter.limit !== 'all') filter.limit = 'all'
  patch({ filter })
  persistFilter()
}

/** has no has / in （ and 「」 use this 1 ） */
export function filterActive(): boolean {
  return store.filter.provider !== 'all'
    || store.filter.enabled !== 'all'
    || store.filter.limit !== 'all'
    || store.filter.q.trim() !== ''
}

/** 1 ： + （ / / ）。
 * down ， not —— when */
export function clearFilter(): void {
  patch({ filter: { provider: 'all', enabled: 'all', limit: 'all', q: '' } })
  persistFilter()
}

/**
 * to already not in need （、）「」。
 *
 * not will ： down not to that 1 ，「」， that 
 * 。 in need 、 is 1 no has ** not ** —— use can can need 。
 */
export function normalizeProviderFilter(): void {
  const provider = store.filter.provider
  if (provider === 'all') return
  if (providerSummaryList().some(item => item.id === provider)) return
  patch({ filter: { ...store.filter, provider: 'all' } })
  persistFilter()
}

/* ─── （ when use ； and 1 ）─────── */

/**
 * 。
 *
 * **must that 1 **（ will new ）， not can `store.collapsed`：
 * in `patch()` before use ， that 1 `store.collapsed` is ** before ** old ——
 * old ， after 1 （ up 、 after is up 1 ；
 * use that to is 「 after down is ， this not 」）。
 * parameter not is store， is as this not can can 。
 */
function persistCollapsed(collapsed: ReadonlySet<string>): void {
  try {
    localStorage.setItem(COLLAPSED_KEY, JSON.stringify([...collapsed]))
  } catch { /* etc not ： will */ }
}

export function groupCollapsed(provider: string): boolean {
  return store.collapsed.has(provider)
}

export function toggleGroupCollapsed(provider: string): void {
  const next = new Set(store.collapsed)
  if (next.has(provider)) next.delete(provider)
  else next.add(provider)
  persistCollapsed(next)
  patch({ collapsed: next })
}

/**
 * / 。
 *
 * `providers` by ****（ not ）： by up is 「」， is 。
 * current can ， is down 「 that is 」 this not ——
 * use when to is ， will 「 no / no 」。
 * （ 、⋯ ） is 「」 。
 */
export function setAllGroupsCollapsed(providers: string[], collapsed: boolean): void {
  const next = new Set(store.collapsed)
  for (const provider of providers) {
    if (collapsed) next.add(provider)
    else next.delete(provider)
  }
  persistCollapsed(next)
  patch({ collapsed: next })
}

/* ─── （used forcurrent， not ）─── */

export function isPicked(id: string): boolean {
  return store.selected.has(id)
}

export function togglePick(id: string, picked: boolean): void {
  const next = new Set(store.selected)
  if (picked) next.add(id)
  else next.delete(id)
  patch({ selected: next })
}

export function setAllPicked(ids: string[], picked: boolean): void {
  const next = new Set(store.selected)
  for (const id of ids) {
    if (picked) next.add(id)
    else next.delete(id)
  }
  patch({ selected: next })
}

export function clearSelection(): void {
  patch({ selected: new Set<string>() })
}

/* ─── ─────────────────────────────── */

export function toggleNamesHidden(): void {
  const next = !store.namesHidden
  try { localStorage.setItem(NAMES_HIDDEN_KEY, next ? '1' : '0') } catch { /* not down */ }
  patch({ namesHidden: next })
}

/** （set use ； that toggle） */
export function setNamesHidden(next: boolean): void {
  try { localStorage.setItem(NAMES_HIDDEN_KEY, next ? '1' : '0') } catch { /* not down */ }
  patch({ namesHidden: next })
}

/* ─── ───────────────────────────── */

/**
 * from and to （ in ** / ** when use up ）。
 *
 * 1 set and use this 1 store，set 
 * `setGroupTotalFiltered`， not need need 。 this path to 「 1 / 1 
 * 」 after `storage` —— not ， that need new 
 * 。 no when not patch， 1 。
 */
export function syncGroupTotalPreference(): void {
  const next = loadGroupTotalFiltered()
  if (next === store.groupTotalFiltered) return
  patch({ groupTotalFiltered: next })
}

export function setGroupTotalFiltered(next: boolean): void {
  try { localStorage.setItem(GROUP_TOTAL_FILTERED_KEY, next ? '1' : '0') } catch { /* not down */ }
  patch({ groupTotalFiltered: next })
}

/**
 * ： by 、4~12 —— not will 1 ， and when 
 * ； 12 then not to 。
 */
export function maskName(name: string): string {
  return '*'.repeat(Math.min(Math.max(name.length, 4), 12))
}
