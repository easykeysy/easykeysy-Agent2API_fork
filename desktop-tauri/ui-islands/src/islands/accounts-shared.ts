import { t } from '../i18n'
import { type ProxyUsage } from '../../../ui-shared/proxy-usage'
import type { BalanceQueryPublic } from './balance-query'
import type { LimiterGroup, LimiterSnapshot } from './accounts-limiter-types'
import type { AccountSorting, AccountSortRule as GroupSortRule } from '../../../ui-shared/account-sorting'
export type { AccountSorting, AccountSortRule as GroupSortRule } from '../../../ui-shared/account-sorting'
export type { LimiterGroup } from './accounts-limiter-types'

/**
 * file use ** and **（：`.ts` not import.meta.glob ）。
 *
 * by 「 .tsx + / .ts」 file， need 1 
 * window up has （workbuddyDesktop / wbApp / wbColSettings …）。 if file
 * 1 ， and will not 1 TS2717 —— and when 
 * （ table-col-settings.tsx ）。 this ** 1 **： + in this 。
 *
 * this use to that ， and ** not Window up **： for 
 * （wbAccountsView / wbAccountsModel / wbUsageActions / wbAccountPanel） in 
 * accounts-data.ts declare 1 （ has 1 file， not will ）。
 */

/* ─── after （ to ， its by unknown ）───── */

/** config（ and after workbuddy-proxy.mjs 1 ） */
export type ProxyConfig = {
  /** old source； new in config.source down ， */
  source?: string
  label?: string
  /** failure Reason： when ， up */
  error?: string
  /**
   * （`source === 'rotate'`， for OpenCode Zen ） ：
   * and its in available （ after `proxy_rotate::summary`）。
   */
  rotate?: { all?: boolean; total?: number; usable?: number } | null
  config?: {
    source?: string
    listenerUid?: string
    /** `source === 'pool'` when id（ PoolItem） */
    proxyId?: string
    /** `source === 'rotate'`：true = enabled */
    all?: boolean
    /** `source === 'rotate' && all !== true`： need id */
    proxyIds?: string[]
    protocol?: string
    host?: string
    port?: number
    username?: string
    password?: string
  } | null
}

/** by （`rateLimits[model]`） */
export type RateLimitInfo = {
  status?: number | string
  code?: string | number
  resetAt?: number
  /**
   * ** when **（ms）：`store_admin::mark_rate_limited` will 。
   * and `resetAt` this is 、 to —— that 
   * 「」 is 。 old can can no has this ， that when not 
   * （`progress: null`）， its old 。
   */
  at?: number
  message?: string
}

/**
 * limit then （ after `core::limiter::LimiterRule` ）。 1 
 * （`limiters`）， then can ：（ / Token）+ + +
 * enabled，Token then 。
 *
 * after ** as **： up no has `limiters` （ old ） when by old 
 * `lowBalance` / provider ****（ accounts-domain `limitersOf`），
 * before 、 and after to is 1 valid then 。
 */
export type LimiterRule = {
  /** = （）；Token = to （≥） */
  type: 'balance' | 'token'
  /** = （ / ）； use = not ， need enabled */
  action: 'skip' | 'disable'
  /** ： and 1 ；Token （「」） */
  threshold: number
  /**
   * only「」 Token then ：（seconds，30 minutes ~ 24 when ，
   * for when ）； then not this 。
   */
  period?: number
  /**
   * only Token then ：（ N minutes/ when ） or （
   * when 0 —— has 0 ）。 then fixed。
   * （ old no has this ）= fixed。
   */
  reset?: 'fixed' | 'daily'
  /** disabled in then not and ； after 1 after to */
  enabled?: boolean
}

export type BatchLimiterMode = 'append' | 'replace' | 'enable' | 'disable' | 'clear'

export type BatchAccountsPayload = {
  action: string
  ids: string[]
  proxy?: unknown
  maxConcurrent?: number
  usageQuery?: { enabled: boolean; interval: number }
  limiterGroups?: LimiterGroup[]
  limiters?: LimiterRule[]
  limiterMode?: BatchLimiterMode
}

export type BatchAccountsResult = {
  ok?: Array<{ id?: string; name?: string; changes?: string[] }>
  removed?: Array<{ id?: string }>
  failed?: Array<{ id?: string; error?: string }>
  skipped?: Array<{ id?: string; name?: string; reason?: string }>
  list?: AccountsSnapshot
}

export type AccountUpdateResult = {
  account?: AccountRecord
  changes?: string[]
  list?: AccountsSnapshot
}

/** and after MAX_CONCURRENT_LIMIT 1 。 */
export const ACCOUNT_CONCURRENCY_MAX = 999

/**
 * 1 Token then in **current** （usage `tokenUsage` ，
 * after `core::limiter` ）。`kind` is ： =
 * seconds， = 0。`windowStart` is （ms）： and by 
 * for not up = already ， by 0 （ accounts-domain 
 * `tokenReadingForRule`）。
 */
export type TokenReading = {
  kind: number
  windowStart: number
  used: number
}

/**
 * 。 after 、 and provider not （identifier / expiry 
 * by can ）， 1 —— can to （uid / userId / account）
 * can by 。
 */
export type AccountRecord = {
  id: string
  provider?: string
  name?: string
  nickname?: string
  /** use set（ after in update_account to name when ， apply_patch） */
  nameCustom?: boolean
  email?: string
  priority?: number
  enabled?: boolean
  /** （ after to ； as available）：false = / not 。
   * by excluding（ and after `resolve_batch_targets` 1 ），
   * and `enabled` is —— use not and ， can 。 */
  available?: boolean
  /** when （ from clientfile） */
  desktop?: boolean
  /**
   * after ****： this has no has available（`has_token || desktop ||
   * apiKey || jwt || noAuth`， after `state::has_credentials`）。
   *
   * in use 1 ： to 「 and no has 」 1 「not configured」——
   * that in will ， not use not Whyrequest not 。
   * （ old after ） by 「 has 」， can 1 not need 。
   */
  hasCredentials?: boolean
  edition?: string
  editionLabel?: string
  chatSupported?: boolean
  canClaim?: boolean
  /** ZCode： this 1 current up （`coding-plan` / `start-plan`， accounts-domain ZCODE_PLAN_*） */
  zcodePlan?: string
  hasAccessToken?: boolean
  accessTokenExpiresAt?: number | null
  hasRefreshToken?: boolean
  refreshTokenLongLived?: boolean
  refreshExpiresAt?: number | null
  /** new when （ms）， not when expiresAt。 */
  refreshTokenExpiresAt?: number | null
  /** already to or ，none； new after 。 */
  needsRelogin?: boolean
  /** and `needsRelogin` for ： after to Reason（ as empty no has ）。 */
  needsReloginReason?: string
  /**
   * ： this is not is ****（ after for this ，
   * `store_view::public_account` and `chatglm_accounts::guest_of_record`）。
   *
   * is and to ： no has （ up 1 401），
   * not and ， not in to —— is 1 。
   */
  isGuest?: boolean
  hasBalanceToken?: boolean
  /**
   * ： has no has empty `apiKey`（set and 
   * has none「」 by ）。** not **， after to this 。
   */
  hasApiKey?: boolean
  /** ： is 「 this up none need 」（ and `hasApiKey` ， after `state::no_auth`） */
  noAuth?: boolean
  /**
   * ： **baseUrl **（ this ；
   * to baseUrl）。「」 and after `is_opencode_zen_account`
   * need —— can can OpenCode Go ， that 1 will 。
   */
  baseUrl?: string
  /**
   * ：**config**（ after in this when ；
   * `providers::custom::balance` and `balance-query.tsx`）。
   *
   * ： id（）、 （`userId` /
   * `username`）、 ** in **（`hasAccessToken` / `hasPassword` /
   * `hasToken` —— not ）。 no has this = this 1 not ，「set」
   * no has 。
   */
  balanceQuery?: BalanceQueryPublic
  maxConcurrent?: number
  allowRouteSkip?: boolean
  checkinAt?: number
  /**
   * set（ after as for ；** = 、1 minutes**，
   * `{enabled:false}` is ； accounts-domain `usageQueryOf`）。
   * `interval` is seconds（30 ~ 86400）。
   */
  usageQuery?: { enabled?: boolean; interval?: number }
  /**
   * not when （** old ，limit up after is **： by after in 
   * `limiters` when ， valid then `limiters`）。`threshold` and 
   * 1 （`available` / workbuddy `totalLeft`）。
   */
  lowBalance?: { mode?: 'off' | 'skip' | 'disable'; threshold?: number }
  /**
   * limit then （ after as ，config = by old lowBalance /
   * provider valid then ； `LimiterRule`）。
   */
  limiterGroups?: LimiterGroup[]
  limiters?: LimiterRule[]
  addedAt?: number
  updatedAt?: number
  tokenTail?: string
  tokenExpiresAt?: number
  expiresAt?: number
  source?: string
  proxy?: ProxyConfig | null
  rateLimits?: Record<string, RateLimitInfo>
  upstreamCooldownUntil?: number | null
  upstreamBlockedReason?: string
  [key: string]: unknown
}

export type HiddenAccountReason = 'not_installed' | 'pack_disabled' | 'provider_disabled' | 'not_loaded'

export type HiddenAccountProvider = {
  id: string
  label: string
  count: number
  reason: HiddenAccountReason
}

export type HiddenAccountsSummary = {
  total: number
  byProvider: Record<string, number>
  providers: HiddenAccountProvider[]
}

/** `wbApp.getState()?.accounts` （providers need + ） */
export type AccountsSnapshot = {
  accounts?: AccountRecord[]
  providers?: Array<{ id?: string; label?: string; count?: number }>
  sorting?: AccountSorting
  /** ， and ， not 。 */
  hiddenAccounts?: {
    total?: number
    byProvider?: Record<string, number>
    providers?: HiddenAccountProvider[]
  }
}

/** ：undefined / null in / string failure / for */
export type UsageEntry = null | string | Record<string, unknown> | undefined

/** for （ and table-col-settings Align 1 ） */
export type Align = 'left' | 'center' | 'right'

/* ─── new ───────────────────────────────
 *
 * this already to in 1 （ `ui/checkin/state.js` 
 * 1 after ）。 no has ： to in in iframe ，， 
 * not to 。 */

/* ─── ─────────────────────────────────── */

type ConfirmOptions = {
  title?: string
  /** ， <strong> etc ； by use */
  html?: string
  okText?: string
  /** danger = not can （） */
  okClass?: string
  bodyClass?: string
}

export type AccountExportSelection = { id: string; provider?: never } | { provider: string; id?: never }

export type AccountExportResult = {
  canceled?: boolean
  total?: number
  count?: number
  failed?: Array<{ id?: string; name?: string; error?: string }>
  file?: string
  saved?: boolean
  downloadStarted?: boolean
}

export type AccountsBridge = {
  platform?: string
  exportAccountFiles(selection: AccountExportSelection): Promise<AccountExportResult | null | undefined>
  getAccountConnections(): Promise<{ counts?: Record<string, unknown> } | null | undefined>
  updateAccount(id: string, patch: Record<string, unknown>): Promise<AccountUpdateResult | null | undefined>
  /** （）： new up （ from to id ） */
  reorderAccounts(queue: string[]): Promise<unknown>
  getAccountSorting(): Promise<AccountSorting>
  saveAccountSorting(payload: { provider: string; rules: GroupSortRule[] | null; onlyIfUnset?: boolean }): Promise<AccountSorting>
  moveAccount(id: string, direction: 'up' | 'down'): Promise<unknown>
  clearRateLimits(id: string, model?: string | null): Promise<unknown>
  /**
   * by （ / / / / ） and can 。
   * `setRoutingConfig` ****、return after （ and /api/timeouts
   * 1 ）；`getRoutingQueue` model = not 。
   */
  /** will （/api/session，app.js ）； by models / lastRequestModel */
  getState?(): Promise<{
    models?: Array<{ id?: string; providerLabel?: string }>
    lastRequestModel?: string | null
  } | null | undefined>
  getRoutingConfig(): Promise<Record<string, unknown> | null | undefined>
  setRoutingConfig(patch: Record<string, unknown>): Promise<Record<string, unknown> | null | undefined>
  getRoutingQueue(model?: string): Promise<Record<string, unknown> | null | undefined>
  /**
   * （ after `POST /api/accounts/batch`）。 must in 
   * bridge.rs and web_shim.rs when 。
   * `skipped` is can not use ， and failure `failed` 。
   */
  batchAccounts(payload: BatchAccountsPayload): Promise<BatchAccountsResult | null | undefined>
  getAllBalances(id?: string): Promise<{ results?: Array<Record<string, unknown>> } | null | undefined>
  /**
   * usage（ after `usage_query::snapshot`）： Token limit 
   * —— `tokenAt` is this when （ and `at` ， before 
   * by need not need use ），`tokenUsage` by to enabled in Token 1 
   * `{period, windowStart, used}`（ no then not ）。
   */
  getBalancesSnapshot(): Promise<LimiterSnapshot & {
    at?: number
    results?: Array<Record<string, unknown>>
    tokenAt?: number
    tokenUsage?: Record<string, unknown>
    sorting?: AccountSorting
  } | null | undefined>
  getProxies(): Promise<{ clash?: import('../../../ui-shared/clash-proxy').ClashProxySnapshot } | null | undefined>
  getProxyPool(): Promise<{ items?: PoolItem[] } | null | undefined>
  /** 1 （ in use when 「」）：
   * will that 「 up 」， and that by 1 */
  testProxyPoolItem(id: string): Promise<{
    success?: boolean
    ip?: string
    durationMs?: unknown
    error?: string
  } | null | undefined>
  testProxy(payload: { proxy: unknown }): Promise<{
    success?: boolean
    status?: unknown
    ip?: string
    durationMs?: unknown
    error?: string
  } | null | undefined>
  switchAccount(id: string): Promise<{ changed?: boolean } | null | undefined>
  refreshAccountToken(id: string): Promise<unknown>
  removeAccount(id: string): Promise<unknown>
  /** request and ； not 。 */
  zcodeCaptchaStats?(): Promise<{
    ready?: number
    target?: number
    required?: boolean
    needsTokens?: boolean
    demandActive?: boolean
    lastSyncAt?: number
    configError?: string
    captchaAccountId?: string
    error?: string
    startPlanAccounts?: number
    rejected?: number
    minted?: number
    consumed?: number
    ttlMs?: number
  } | null | undefined>
}

/**
 * 「」（） 1 —— use to ，
 * ui-islands/src/islands/proxies-page.tsx PoolItem。
 * proxy can by id use （`{source:'pool', proxyId}`）。
 */
export type PoolItem = ProxyUsage & {
  id: string
  name?: string
  source?: string
  enabled?: boolean
  protocol?: string
  host?: string
  port?: number | null
  resolved?: { protocol?: string; host?: string; port?: number | null; label?: string } | null
  resolveError?: string | null
}

/** 1 when ， and down for 。 */
export function poolItemAddress(item: PoolItem): string {
  const proxy = item.resolved || item
  if (proxy.protocol && proxy.host && proxy.port) {
    const host = proxy.host.includes(':') && !proxy.host.startsWith('[') ? `[${proxy.host}]` : proxy.host
    return `${String(proxy.protocol).toLowerCase()}://${host}:${proxy.port}`
  }
  return item.resolved?.label || ''
}

export function poolItemLabel(item: PoolItem): string {
  const name = item.name || item.id
  const parts = [item.source === 'clash' ? 'Clash Verge' : '', item.resolveError ? t('解析失败') : poolItemAddress(item), item.enabled === false ? t('已禁用') : ''].filter(Boolean)
  return parts.length ? t('{name}（{parts}）', { name, parts: parts.join(' · ') }) : name
}

/**
 * down 「」 this 1 before ：`pool:<proxyId>`。
 *
 * Why need need before not is use proxyId： 1 down Clash uid
 * （ verge.yaml， not ，`px_...` up not is not can can ），
 * and `__proxy_*__` 。 before ， can 。
 * （ / updateset ） is 1 ——
 * 1 before 、 1 no ， is 「unsupported」，。
 */
export const POOL_VALUE_PREFIX = 'pool:'

/**
 * window up by / 。** use to **，
 * and use 「 + 」， not declare global（file）。
 */
export type SharedWindow = {
  workbuddyDesktop?: AccountsBridge
  wbApp?: {
    esc?: (value: unknown) => string
    toast?: (message: string, kind?: 'err' | 'ok') => void
    formatTime?: (value: unknown) => string
    showPage?: (page: string) => void
    refresh?: () => Promise<boolean>
    getRefreshStatus?: () => { pending: boolean; error: string }
    subscribeRefresh?: (listener: () => void) => () => void
    runAccountAction?: (action: string, id: string) => void
    getState?: () => {
      accounts?: AccountsSnapshot
      /** parameter（`/api/session` `accountFeatures`， as provider id） */
      accountFeatures?: Record<string, unknown>
    } | null | undefined
    /** ： will 「 already 」， use after up */
    renderTopbarStatus?: () => void
    readonly currentPage?: string
  }
  wbConfirm?: { ask?: (options: ConfirmOptions) => Promise<boolean> }
  wbProviders?: {
    labelOf?: (id: string) => string
    all?: () => Array<{ id?: string; label?: string }>
    refreshCustom?: () => Promise<unknown>
    customList?: () => Array<{
      id?: string; name?: string; protocol?: string; baseUrl?: string
      /** client（`'opencode'` = by OpenCode CLI request）。
       * set「 this is not is OpenCode Zen」 ， accounts-domain
       * `isOpencodeZenProvider`。 */
      clientEmulation?: string
    }>
    customRequest?: (method: string, path: string, body?: unknown) => Promise<unknown>
    PROTOCOL_OPTIONS?: Array<{ value: string; label: string }>
  }
  wbFilterMemory?: {
    load<T extends Record<string, string>>(key: string, defaults: T): T
    save(key: string, patch: Record<string, string>): void
  }
  wbIcons?: { icon?: (name: string, size?: number) => string }
  wbColSettings?: {
    register(spec: ColSettingsSpec): ColSettingsHandle
    apply<C extends { key: string }>(id: string, columns: C[]): Array<C & { align: Align }>
    configOf(id: string): Array<{ key: string; visible: boolean; align: Align }> | null
    syncStaticHead(id: string, table: Element | null | undefined, viewHidden?: ReadonlySet<string> | null): void
    close(): void
  }
  /** directory（ 1 / 1 / 1 ），set 「」 1 use */
  wbCustomProvidersUi?: {
    find?: (id: string) => Promise<{
      id: string; name?: string; protocol?: string; baseUrl?: string
      /** client（'' / 'opencode'）：set 「」 */
      clientEmulation?: string
    } | null | undefined>
    update?: (patch: {
      id: string; name: string; protocol: string; baseUrl: string; clientEmulation?: string
    }) => Promise<unknown>
    remove?: (id: string) => Promise<boolean>
  }
  /**
   * ZCode ****（ui/zcode-claim.js）：`preview` can 1 
   * （ not need ，failure when throw），`claim` 1 （ need ，return）。
   * in （zcode-plans-modal.tsx），file 。
   */
  wbZcodeClaim?: {
    preview?: (accountId: string) => Promise<unknown>
    claim?: (accountId: string, planId: string, signal?: AbortSignal, onPhase?: (phase: string) => void) => Promise<unknown>
  }
  /** ZCode （zcode-plans-modal.tsx）： can + down already has ， */
  wbZcodePlans?: { open?: (options: { id: string; name?: string }) => void }
  wbZcodeCaptchaPool?: {
    status?(): { state: 'idle' | 'verifying' | 'paused' | 'error' | 'disabled' | 'needs_verification'; error: string }
    refresh?(): ReturnType<NonNullable<AccountsBridge['zcodeCaptchaStats']>>
    verify?(accountId: string, signal?: AbortSignal, onPhase?: (phase: string) => void): Promise<boolean>
  }
  /** CodeArts「」（ui/codearts-welfare.js： → → → ） */
  wbCodeArtsWelfare?: { start?: (account: AccountRecord | undefined) => Promise<unknown> }
  /** 「」（ 1 ， open） */
  wbAddAccountModal?: { open?: () => void; close?: () => void }
  /** （ after by providers need ） */
  wbAccountAddForms?: { syncAddProvider?: () => void }
}

/** set （ table-col-settings.tsx TableSpec use to that ） */
export type ColSettingsSpec = {
  id: string
  label?: string
  columns: Array<{ key: string; label: string; align: Align; legacyAlign?: Align }>
  mount?: () => Element | null
  onChange?: () => void
}

export type ColSettingsHandle = {
  apply<C extends { key: string }>(columns: C[]): Array<C & { align: Align }>
  config(): Array<{ key: string; visible: boolean; align: Align }>
}

export function shared(): SharedWindow {
  return window as unknown as SharedWindow
}

/* ─── （file use ： / / when ）───── */

export function toast(message: string, kind?: 'err' | 'ok'): void {
  shared().wbApp?.toast?.(message, kind)
}

/**
 * HTML ：file 1 wbApp（app.js ）， has no when 
 * 1 —— wbConfirm.ask is HTML ， is 。
 */
export function esc(value: unknown): string {
  const fn = shared().wbApp?.esc
  if (fn) return fn(value)
  return String(value ?? '').replace(/[&<>"']/g, char => (
    { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[char] || char
  ))
}

/** when → when （0 and returnempty， use that ） */
export function formatTime(value: unknown): string {
  const fn = shared().wbApp?.formatTime
  if (fn) return fn(value)
  const time = Number(value)
  if (!Number.isFinite(time) || time <= 0) return ''
  return new Date(time).toLocaleString('zh-CN')
}

export function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}

/**
 * → ：****， not to to 「—」。
 *
 * ── Why need has limit ──────────────────────────────────────
 * up has to is `*Precise` this ****（WorkBuddy 
 * `CycleCapacityRemainPrecise` `3547.430000335`）， after by 「 not 、
 * to 」， is that will and 。 to 
 * already up up to ， is 。 in this 1 
 * （ / / total / / limit）， in 1 。
 *
 * not （`1500.00` → `1500`， by `Number` ）； not 
 * **** —— up to 「 not 」 this ， not can 1 has 「—」。
 */
export function numberText(value: unknown): string {
  if (value === null || value === undefined || value === '') return '—'
  const number = Number(value)
  if (!Number.isFinite(number)) return String(value)
  return String(Number(number.toFixed(2)))
}
