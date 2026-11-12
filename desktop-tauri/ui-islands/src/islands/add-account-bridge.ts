/**
 * Agent2API · 「 / 」 （window / / / ）。
 *
 * from add-account.js + add-provider-forms.js and 1 ： this old 
 * app.js （$ / toast / esc / refresh） and window up ， to up 
 * after this in in this 1 ， import 。
 *
 * Why not use declare global Window up ：workbuddyDesktop / wbApp /
 * wbProviders is use ， declare 1 will not 1 
 * TS2717（ and ）。 this **** that 
 * （ add-account-modal.tsx ）。
 */

import { isSuspendedProvider } from '../../../ui-shared/provider-availability'
import { t } from '../i18n'

/* ─── window up （ + ）────────────── */

/** use to （ its not ） */
export type AccountRecord = {
  id?: string
  name?: string
  uid?: string
  userId?: string
  provider?: string
}

/** use current when need in provider； old if already down ， 501 path。 */
export type RuntimeProviderRecord = {
  id?: string
  label?: string
  count?: number
  category?: 'agent' | 'web'
  source?: 'builtin' | 'plugin'
  meta?: Record<string, unknown>
  login_methods?: unknown[]
  loginMethods?: unknown[]
  capabilities?: Record<string, unknown>
}

export type LoginOptions = {
  /** ； = old （ no has ，e.g.） */
  methodIndex?: number
  inputs?: Record<string, string>
  proxy?: unknown
}

/** ； userCode， not device_code or 。 */
export type LoginProgress = {
  active?: boolean
  provider?: string
  state?: string
  manualCallback?: boolean
  callbackInput?: 'url' | 'code'
  instructions?: string
  hint?: string
  userCode?: string
}

/** ok/payload、 session 1 ；/emptyresponse not success。 */
export type LoginResult = {
  ok: boolean
  canceled: boolean
  silent?: boolean
  account?: AccountRecord
  session?: Record<string, unknown>
  proxySaved?: boolean
  warning?: string
  instructions?: string
}

/** directory：（wbProviders.customList） and directory（wbPresetProviders） use */
export type CustomProviderRecord = {
  id: string
  name?: string
  protocol?: string
  baseUrl?: string
  presetKey?: string
  accountCount?: number
  /** client（ ）：'opencode' = by CLI request */
  clientEmulation?: string
  /**
   * （ ， `custom_providers::BALANCE_SCHEMES`）：
   * has = this 1 can ， and set will 「set」 。
   * ****： is 「 this 1 is 」 ， up not to use 。
   */
  balanceScheme?: string
  quirks?: {
    urlSuffix?: string
    headers?: Record<string, string>
    anthropicToolType?: string
  }
}

export type PresetRecord = {
  key: string
  name: string
  protocol?: string
  baseUrl?: string
  hint?: string
  /** Base URL （ no has when to 1 「 」） */
  placeholder?: string
  quirks?: CustomProviderRecord['quirks']
  /** client：create（OpenCode Zen use ） */
  clientEmulation?: string
  /** this default（）：noAuth = 「 this up none need 」 */
  account?: { noAuth?: boolean }
  /** ：create when `balanceScheme`（ `CustomProviderRecord`） */
  balance?: { scheme?: string }
}

export type WebLoginView = {
  label: string
  hint: string
  disabled: boolean
  waiting: boolean
}

/** web-login.js （ use to ） */
export type WebLoginController = {
  provider: string
  syncTexts(): void
  applyState(): void
  start(): void
  cancel(): void
  dispose(): void
}

/** autoclaw-oauth.js */
export type OauthController = {
  provider: string
  start(vendor: 'zai' | 'google'): void
  syncTexts(): void
  cancel(): void
}

export type SharedWindow = {
  workbuddyDesktop?: {
    /** （'macos' / 'windows' / 'linux' / 'web'） */
    readonly platform?: string
    startLogin(edition: string, mode: string, provider: string, socialRestore?: boolean, options?: LoginOptions): Promise<unknown>
    submitLoginCallback?: (state: string, callbackUrl: string) => Promise<unknown>
    onLoginState?: (callback: (progress: LoginProgress) => void) => (() => void) | void
    onLoginProgress?: (callback: (progress: LoginProgress) => void) => (() => void) | void
    cancelLogin?(): Promise<unknown>
  }
  wbApp?: {
    toast?: (message: string, kind?: 'err' | 'ok' | 'warn') => void
    refresh?: () => Promise<unknown> | unknown
    getState?: () => {
      accounts?: { accounts?: AccountRecord[]; providers?: RuntimeProviderRecord[] }
    } | null | undefined
    /** （app.js ）：empty「」 use to */
    showPage?: (page: string) => void
  }
  /** （web-login.js）： create / refresh / cancelIfActive */
  wbWebLogin?: {
    create(config: {
      provider: string
      buttonId: string
      cancelId: string
      hintId: string
      busyText?: string
      texts?: () => { button?: string; hint?: string }
      onStateChange?: (state: WebLoginView) => void
      start: () => unknown
      onSuccess: (result?: unknown) => unknown
    }): WebLoginController | null | undefined
    refresh?: () => Promise<unknown>
    isActive?: () => boolean
    activeProvider?: () => string
    cancelIfActive?: (provider: string) => Promise<boolean>
  }
  /** （sms-login.js）： create， in */
  wbSmsLogin?: {
    create(config: { provider: string; onSuccess: (data?: unknown) => unknown }): unknown
  }
  /** AutoClaw OAuth （autoclaw-oauth.js）：create / cancel（） */
  wbAutoclawOauth?: {
    create(config: {
      provider: string
      mode: () => string
      hint: () => string
      cancelId?: string
      onSuccess: (result?: unknown) => unknown
    }): OauthController | null | undefined
    cancel?: () => void
  }
  /** directory（custom-provider-ui.js， that 1 ） */
  wbCustomProvidersUi?: {
    remove?: (providerId: string) => Promise<boolean>
  }
  /** （file in ， use ） */
  wbAddAccountModal?: { open?: () => void; close?: () => void }
  /** （；account-panel / need ） */
  wbAccountAddForms?: {
    syncAddProvider?: () => void
    openNewCustomForm?: () => void
  }
  wbProviders?: {
    all?: () => RuntimeProviderRecord[]
    labelOf?: (id: string) => string
    load?: () => Promise<unknown>
    customList?: () => CustomProviderRecord[]
    refreshCustom?: () => Promise<CustomProviderRecord[] | unknown>
    customRequest?: (method: string, path: string, body?: unknown) => Promise<unknown>
    PROTOCOL_OPTIONS?: Array<{ value: string; label: string }>
  }
  wbPresetProviders?: {
    list?: PresetRecord[]
    presetOf?: (key: string) => PresetRecord | null
    iconOf?: (key: string) => string
  }
  /** IPC：POST /api/accounts in no has ， can */
  __TAURI_INTERNALS__?: { invoke?: (cmd: string, args: unknown) => Promise<unknown> }
}

export function shared(): SharedWindow {
  return window as unknown as SharedWindow
}

/* ─── use ─────────────────────────────── */

export function toast(message: string, kind?: 'err' | 'ok' | 'warn'): void {
  shared().wbApp?.toast?.(message, kind)
}

/** error， not response（ its in can can ）。 */
function errorText(error: unknown, depth = 0): string {
  if (typeof error === 'string') return error.trim()
  if (error instanceof Error) return error.message.trim()
  if (!error || typeof error !== 'object' || depth > 3) return ''
  const record = error as Record<string, unknown>
  for (const key of ['message', 'error', 'msg', 'reason']) {
    const text = errorText(record[key], depth + 1)
    if (text) return text
  }
  return ''
}

/** Tauri rejection can can is ；error message/error etc 。 */
export function describeError(error: unknown): string {
  const text = errorText(error)
  return text ? t(text) : t('未知错误')
}

/** HTTP ；onlyreturn when ， can 。 */
export function isMethodsUnavailable(error: unknown, depth = 0): boolean {
  if (error && typeof error === 'object' && depth < 4) {
    const record = error as Record<string, unknown>
    for (const key of ['status', 'statusCode', 'status_code', 'httpStatus']) {
      const value = record[key]
      if ((typeof value === 'number' || typeof value === 'string') && Number(value) === 501) return true
    }
    if (record.code === 'not_implemented' || record.code === 'auth_methods_unavailable') return true
    if (record.error && record.error !== error && isMethodsUnavailable(record.error, depth + 1)) return true
  }
  const text = errorText(error)
  return text === '该提供商尚未申报动态登录方式，请使用既有登录入口'
    || /\b501\b|auth\.methods[^\n]*(?:not implemented|no method|未实现|不支持)/i.test(text)
}

export function runtimeProviderOf(provider: string): RuntimeProviderRecord | undefined {
  if (isSuspendedProvider(provider)) return undefined
  const live = shared().wbApp?.getState?.()?.accounts?.providers
  const list = Array.isArray(live) ? live : shared().wbProviders?.all?.() || []
  return list.find(item => item.id === provider)
}

/** this down （ ， not ） */
export function accountCountOf(providerId: string): number {
  const accounts = shared().wbApp?.getState?.()?.accounts?.accounts || []
  return accounts.filter(account => (account?.provider || 'workbuddy') === providerId).length
}

/* ─── and ─────────────────────────────── */

/** 1 API ， data and success/data ， not failuresuccess。 */
export async function accountRequest(method: string, path: string, body?: unknown): Promise<unknown> {
  const internals = shared().__TAURI_INTERNALS__
  if (!internals || typeof internals.invoke !== 'function') {
    throw new Error(t('桌面运行时不可用（Tauri 未初始化）'))
  }
  const result = await internals.invoke('api_request', {
    request: { method, path, body: body === undefined ? null : body },
  })
  if (result && typeof result === 'object') {
    const record = result as Record<string, unknown>
    if (record.success === false) throw result
    if (record.success === true && Object.hasOwn(record, 'data')) return record.data
  }
  return result
}

/** 1 ：POST /api/accounts， 。 */
export function postAccount(payload: Record<string, unknown>): Promise<unknown> {
  return accountRequest('POST', '/api/accounts', payload)
}

function resultRecord(value: unknown): Record<string, unknown> | undefined {
  return value && typeof value === 'object' && !Array.isArray(value)
    ? value as Record<string, unknown> : undefined
}

/** ， not response in 。 */
function publicAccount(value: unknown): AccountRecord | undefined {
  const record = resultRecord(value)
  if (!record) return undefined
  const account: AccountRecord = {}
  for (const key of ['id', 'name', 'uid', 'userId', 'provider'] as const) {
    if (typeof record[key] === 'string') account[key] = record[key]
  }
  return account.id || account.uid || account.userId ? account : undefined
}

export function normalizeLoginResult(value: unknown): LoginResult {
  let record = resultRecord(value)
  if (record?.success === false) throw value
  if (record?.success === true && Object.hasOwn(record, 'data')) record = resultRecord(record.data)
  const payload = resultRecord(record?.payload)
  const session = resultRecord(record?.session) || resultRecord(payload?.session)
  const sources = [record, payload, session].filter((item): item is Record<string, unknown> => Boolean(item))
  if (sources.some(item => item.canceled === true || item.state === 'canceled' || item.status === 'canceled')) {
    return { ok: false, canceled: true, silent: sources.some(item => item.silent === true) }
  }
  for (const item of sources) {
    if (item.error || item.ok === false || item.success === false || item.state === 'failed'
      || item.status === 'failed' || item.type === 'error') {
      throw new Error(describeError(item.error || item))
    }
    if (item.pending === true || resultRecord(item.pending) || item.accepted === true
      || item.done === false || item.state === 'pending' || item.status === 'pending') {
      throw new Error(t('登录尚未完成，请继续通过共享登录桥等待结果'))
    }
  }
  const account = sources.map(item => publicAccount(item.account)).find(Boolean)
  const summary = sources.find(item =>
    (typeof item.accountUid === 'string' && Boolean(item.accountUid))
    || (typeof item.accountId === 'string' && Boolean(item.accountId))
    || (typeof item.nickname === 'string' && Boolean(item.nickname)))
  if (!account && !summary && sources.some(item => (item.flow || item.authUrl) && item.done !== true)) {
    throw new Error(t('登录尚未完成，请继续通过共享登录桥等待结果'))
  }
  if (!account && !summary && record?.ok !== true) {
    throw new Error(session && !Object.keys(session).length
      ? t('登录桥返回了空会话，无法确认账号已添加')
      : t('登录桥未返回已完成的账号或会话，请重试'))
  }
  const text = (key: string): string | undefined => {
    const value = sources.map(item => item[key]).find(item => typeof item === 'string' && item.trim())
    return typeof value === 'string' ? value : undefined
  }
  const proxySaved = sources.map(item => item.proxySaved).find(item => typeof item === 'boolean')
  return {
    ok: true, canceled: false, account, warning: text('warning'), instructions: text('instructions'),
    ...(typeof proxySaved === 'boolean' ? { proxySaved } : {}),
    // will ， not response。
    session: summary ? Object.fromEntries(['accountUid', 'accountId', 'nickname', 'edition', 'provider']
      .filter(key => typeof summary[key] === 'string').map(key => [key, summary[key]])) : undefined,
  }
}

/**
 * provider-first ； has window bridge edition/mode/provider old 。
 * new options can in 5 parameter， not can 4 socialRestore。
 * in to start_login options parameter； etc and by has 。
 */
export async function startLogin(
  provider: string, edition = 'cn', mode = 'embedded', socialRestore?: boolean, options?: LoginOptions,
): Promise<LoginResult> {
  // has methodIndex is ； proxy is old 
  // （ ， add-provider-blocks WebLoginSection）。
  // not ：must 1 id， old old not 。
  const dynamic = options !== undefined && options.methodIndex !== undefined
  if (isSuspendedProvider(provider) || (dynamic && !runtimeProviderOf(provider))) {
    throw new Error(t('该提供商已不在运行时目录中，请返回列表重新选择'))
  }
  try {
    const bridge = shared().workbuddyDesktop
    let result: unknown
    if (dynamic && bridge?.platform !== 'web') {
      const internals = shared().__TAURI_INTERNALS__
      if (!internals?.invoke) throw new Error(t('桌面运行时不可用（Tauri 未初始化）'))
      result = await internals.invoke('start_login', {
        provider, edition, mode, socialRestore: socialRestore === true, options,
      })
    } else {
      if (!bridge?.startLogin) throw new Error(t('共享登录桥不可用，请重新打开面板'))
      result = await bridge.startLogin(edition, mode, provider, socialRestore, options)
    }
    const normalized = normalizeLoginResult(result)
    if (dynamic && !normalized.canceled) {
      const accountId = normalized.account?.id || normalized.session?.accountUid || normalized.session?.accountId
      const resultProvider = normalized.account?.provider || normalized.session?.provider
      if (!accountId || resultProvider && resultProvider !== provider) {
        throw new Error(t('登录桥返回了空会话，无法确认账号已添加'))
      }
    }
    return normalized
  } catch (failure) {
    const record = resultRecord(failure)
    if (record?.canceled === true) return { ok: false, canceled: true, silent: record.silent === true }
    // web-login.js by error.message error； invoke rejection can can is String。
    throw new Error(describeError(failure))
  }
}

/** success after ： name 1 in ， by */
export function addedLabelOf(account?: AccountRecord | null): string {
  return account?.name || account?.uid || account?.userId || ''
}

/**
 * 。 （ etc ） 
 * （add-account-modal.tsx closeModal）， this is 1 ——
 * / when window use 。
 */
export function closeAddModals(): void {
  shared().wbAddAccountModal?.close?.()
}

/** success after 1 ：、 new 、 */
export async function afterAdd(name: string, label: string): Promise<void> {
  closeAddModals()
  await shared().wbApp?.refresh?.()
  toast(t('✅ {label}账号已添加{name}', { label, name: name ? `：${name}` : '' }))
}

/* ─── （ 「 in 」）──────────────
 *
 * old in DOM after not ： by has ，
 * （ ） when is up ， has successempty。
 * will ， this is that ——
 * （defaultValue + onChange ） in new when from this 。
 *
 * （web-login / sms-login / autoclaw-oauth） by id `.value`，
 * must is input / textarea， and id and old 1 。 */

const draft = new Map<string, string>()

export function draftProps(id: string): {
  defaultValue: string
  onChange: (event: { currentTarget: { value: string } }) => void
} {
  return {
    defaultValue: draft.get(id) ?? '',
    onChange: event => {
      const value = event.currentTarget.value
      if (value) draft.set(id, value)
      else draft.delete(id)
    },
  }
}

/** （）： and DOM 1 ， not can */
export function setDraftValue(id: string, value: string): void {
  if (value) draft.set(id, value)
  else draft.delete(id)
  const node = document.getElementById(id)
  if (node instanceof HTMLInputElement || node instanceof HTMLTextAreaElement) node.value = value
}

/** current： DOM（ use can can ）， not to */
export function readField(id: string): string {
  const node = document.getElementById(id)
  if (node instanceof HTMLInputElement || node instanceof HTMLTextAreaElement) return node.value.trim()
  return (draft.get(id) ?? '').trim()
}

/** empty if （success after use ；failure when ） */
export function clearFields(ids: string[]): void {
  for (const id of ids) setDraftValue(id, '')
}

/* ─── ─────────────────────────
 *
 * old this in DOM and segState up ， ——
 * is up （resetAddStep and ， not ）。
 * to up after use this for ： as ， when 。 */

export type Edition = 'cn' | 'intl'
export type LoginMode = 'embedded' | 'external'

export const loginPrefs: { edition: Edition; loginMode: LoginMode; socialRestore: boolean } = {
  edition: 'cn',
  loginMode: 'embedded',
  socialRestore: false,
}
