/**
 * Agent2API · set ** and **（ store / / / for ）。
 *
 * from settings-page.tsx ：「 + + 1 」 already 
 * file， this 1 —— no has JSX。 to （ import ，
 * settings-model.ts）。
 *
 * not is ：file is .ts， not will src/index.tsx `islands/*.tsx` glob ；set has 
 * settings-page.tsx 1 （ file）。
 *
 * ── + useSyncExternalStore（ update-panel / models-panel-state）──
 * for （load / renderXxx / showCategory） from React use ， and must and use 1 
 * ； useState not to this 1 。 is 1 ， 1 publish()
 * （ new for ）。
 *
 * ── 「 after 」 is （LoadStatus）────────────────
 * loading / ready / unavailable —— is 「 in …」 not is 「unavailable」， this in old 
 * is （ HTML 「 in …」， to 「unavailable」）。available has 1 ：
 * 「unavailable」 is ** to not **， and 「 not to 」 not is 1 。
 *
 * ── and old not （ in ）────────────
 * · by use （ busy is 'retention' / 'retry' this ）， old is 
 * DOM disabled —— etc ， has 1 ；
 * · in （ ）， 「」， not old 
 * that from DOM in 。
 *
 * ── （ and old for ， 1 not can ）────────────────
 * ：getAppSettings / saveAppSettings（ and ）、exportAccounts / importAccounts；
 * HTTP ：retention / retry / timeouts / debug / sanitize / prompt / storage / captcha。
 * window.workbuddyDesktop，file not URL。
 */

import * as React from 'react'
import {
  CATEGORIES,
  CAPTCHA_POOL_FIELDS,
  IMAGE_COMPRESS_ENABLED_KEY,
  IMAGE_COMPRESS_FIELDS,
  NO_RETRY_CODES_KEY,
  PROMPT_MODES,
  QUEUE_FIELDS,
  RETENTION_FIELDS,
  RETRY_CODE_MAX,
  RETRY_CODE_MIN,
  RETRY_FIELDS,
  RETRY_MAX_CODES,
  ROUTE_STICKY_OPTIONS,
  ROUTE_STRATEGIES,
  SETTINGS_CAT_KEY,
  TIMEOUT_FIELDS,
  errorMessage,
  formatBytes,
  normalizeApp,
  normalizeNumbers,
  parseInteger,
  shared,
  toast,
  type AppSettings,
  type CaptchaSetting,
  type ClineHeadersData,
  type GatewayBlocks,
  type ModelsDevState,
  type NumberField,
  type PromptPatch,
} from './settings-model'
import {
  INITIAL_SANITIZE, sanitizeStateFrom, type SanitizePatch, type SanitizeState,
} from './settings-sanitize-model'
import { t } from '../i18n'
import { retentionSavedMessage, retentionShrinkHead, type RetentionField } from './settings-retention-model'

/* ─── ─────────────────────────────── */

/** ：「 in …」、 to 、 not to （ for and old 1 for ） */
export type LoadStatus = 'loading' | 'ready' | 'unavailable'

/**
 * ： this 1 use （ old is disabled，
 * this 1 ）。null = empty。
 */
export type BusyScope =
  | 'app'
  | 'lan'
  | 'retention'
  | 'retry'
  | 'codes'
  | 'timeouts'
  | 'queue'
  | 'imageCompress'
  | 'captchaPool'
  | 'routing'
  | 'debug'
  | 'sanitize'
  | 'developerRoleCompat'
  | 'clineHeaders'
  | 'zcodeVersion'
  | 'cors'
  | 'prompt'
  | 'captcha'
  | 'modelsDev'
  | 'modelsDevRefresh'
  | 'export'
  | 'import'
  | 'dbExport'
  | 'dbImport'
  | null

/**
 * and 。unavailable when ** can **（ old ： patch，
 * success to ready）， and 。
 *
 * 1 set；`lanIp` / `port` / `adminRegistered` is 
 * and use ****，、failure old （ is 
 * 「 up 」 ，failure not this set）。
 */
export type AppState = {
  status: LoadStatus
  closeToTray: boolean
  autostart: boolean
  /** ： 0.0.0.0（「 use 」） */
  lanAccess: boolean
  /** when is when */
  lanPanel: boolean
  /** ：（ WebView2 ）， by need */
  lightweightMode: boolean
  /** in IP（ not to as null， as ） */
  lanIp: string | null
  /** （0 = no to ）， use */
  port: number
  /** is already （null = no ； will 1 new ） */
  adminRegistered: boolean | null
}

/** ：values as null when （loading / unavailable） use */
export type NumericState = { status: LoadStatus; values: Record<string, number> | null }

/**
 * by （ + will ）： is ， not to when after default
 * （order / auto， by by ）。
 */
export type RoutingState = {
  status: LoadStatus
  strategy: string
  sticky: string
  /** failure：failure 1 will （ after default） */
  failoverBackoff: boolean
  /** ： has when 429 to （ after default） */
  sink: boolean
  /** ：failure to after （ after default） */
  breakerEnabled: boolean
}

export type DebugState = {
  status: LoadStatus
  on: boolean
  /** already / limit； after no to available when as null（ that 1 not ） */
  count: number | null
  limit: number | null
}

/** */
export type DeveloperRoleCompatState = { status: LoadStatus; on: boolean }

/**
 * will ： 1 + parameter。`values` as null（loading / unavailable）
 * when use ；`enabled` in to before by （ and developerRoleCompat
 * ）。
 */
export type ImageCompressState = {
  status: LoadStatus
  enabled: boolean
  values: Record<string, number> | null
}

/** Cline ：（default / / ） after ， and */
export type ClineHeadersState = {
  status: LoadStatus
  defaults: Record<string, string>
  overrides: Record<string, string>
  effective: Record<string, string>
}

/** （/v1/*） */
export type CorsState = { status: LoadStatus; on: boolean }

/**
 * ZCode client（`/api/zcode/version`）。
 *
 * `version` is **set **，empty = use default。
 * `envOverride` empty when `ZCODE_APP_VERSION` ——
 * need 「 this not 」，otherwise use will for 1 no 
 * 。
 */
export type ZcodeVersionState = {
  status: LoadStatus
  version: string
  envOverride: string
}

/**
 * models.dev ： + 「 this is when 、 has 」。
 *
 * `source` and after （`builtin` / `cache` / `network`）， in 
 * ；unknown（ after will ， not will empty）。
 */
export type ModelsDevPanelState = {
  status: LoadStatus
  enabled: boolean
  source: string
  updatedAt: number
  count: number
}

export type PromptState = {
  status: LoadStatus
  mode: string
  file: string
  /** ****（`passthrough` down is empty）： this 1 */
  text: string
  /** after to ：'file' | 'builtin' | 'inline' | ''（ in ） */
  source: string
  lines: number
  fileError: string
  degradeActive: boolean
  degradeUntilText: string
  /** ** by ** （ already config ， after already by id ） */
  providers: ProviderPromptState[]
  /** **** ： （ = default） */
  gateway: Record<string, boolean>
  /** ** **： and （ = ） */
  gatewayText: Record<string, GatewayBlocks>
  /** can config （ down use ）：， 、 and */
  options: ProviderPromptOption[]
}

/** 1 （ and that ， 1 ） */
export type ProviderPromptState = {
  id: string
  mode: string
  file: string
  /** ****（ up ；`passthrough` down is empty） */
  text: string
  source: string
  lines: number
  fileError: string
  /** this 1 **** is up （ true； has `gatewayNote` has ） */
  gateway: boolean
  /** is / file / ； has when ，client。 */
  configured: boolean
}

/**
 * can config 。
 *
 * `gatewayNote` empty = this has ； or already has when its 。
 * `gatewayChars` is that （ 「 N 」；0 / not ）。
 * `gatewayBlocks` is that ****（，Environment ）：
 * ， is 「」 —— when is null，
 * when not （ use 1 empty ）。
 */
export type ProviderPromptOption = {
  id: string
  label: string
  gatewayNote: string
  gatewayChars: number
  gatewayBlocks: GatewayBlocks | null
}

export type StorageState = {
  status: LoadStatus
  /** can not can （false = 「unavailable」）； and status 「 not to 」 not */
  available: boolean
  file: string
  bytes: number
  /** null after available。 */
  accounts: number | null
  logs: number | null
  requests: number | null
  dailyDays: number | null
  downstreamRaw: number | null
  upstreamRaw: number | null
}

/** failure（ before 3 ，more has ） */
export type IoFailure = { failed: number; detail: string; more: boolean } | null

export type SettingsSnapshot = {
  /** current（ and by ） */
  category: string
  /** showCategory ： （ old is scrollTop） */
  scrollReset: number
  busy: BusyScope
  app: AppState
  unitsChinese: boolean
  retention: NumericState
  retry: NumericState
  /** 「error」；retry.status as unavailable when is null */
  retryCodes: number[] | null
  timeouts: NumericState
  queue: NumericState
  /** will ： + parameter（ / / ） */
  imageCompress: ImageCompressState
  captchaPool: NumericState
  /** by ： + will （「 by 」） */
  routing: RoutingState
  debug: DebugState
  sanitize: SanitizeState
  developerRoleCompat: DeveloperRoleCompatState
  clineHeaders: ClineHeadersState
  zcodeVersion: ZcodeVersionState
  cors: CorsState
  prompt: PromptState
  storage: StorageState
  /**
   * （ALTCHA）： 1 ，`localPanel` is 1 
   * （ true / false）， 「 in use this 1 」。
   *
   * `available = false` not to after （unavailable）， ——
   * and retention / cors 1 。
   */
  captcha: { available: boolean; desktop: boolean; web: boolean; localPanel: boolean }
  /**
   * （set「」）。
   *
   * `registered = false` this no —— when not need need ，
   * available， is no has 「」 can 。`username` used for。
   */
  account: {
    status: 'loading' | 'ready' | 'unavailable'
    registered: boolean
    username: string
    /** （empty = ；busy when by ，error empty） */
    password: { busy: boolean; error: string } | null
  }
  /**
   * models.dev （ can and ）。
   *
   * and `captcha` not ， this ****（ and when 、）：
   * this in 「 that is when 、 has 」， to 1 
   * etc can 1 not has no has by 。
   */
  modelsDev: ModelsDevPanelState
  ioFailure: IoFailure
  /** （empty）； / to resolveRetentionConfirm */
  retentionConfirm: { head: string } | null
  /**
   * / （「」）：`dbIncludeLogs` is current；
   * `dbBackupConfirm` empty = （ is when need parameter）。
   */
  dbIncludeLogs: boolean
  dbExportConfirm: boolean
  dbBackupConfirm: { includeLogs: boolean } | null
  /**
   * return （`DatabaseImportResult.warning`；null = no has ）。
   *
   * not is toast： this after can not can （ not when 
   * is empty ），must**** in etc use to —— toast 3.5 seconds（app.js），
   * no has 。 and ioFailure ： 1 ，
   * down 1 when 。
   */
  dbWarning: string | null
  /**
   * / （empty = that ** not can ** ）。
   *
   * is 、 in not can ， no has by ： in not to ，
   * / failure after to 1 「」。 by after to （ `db::backup` 
   * `RestoreProgress`）， before 500ms 1 。
   */
  dbRestore: DbRestoreProgress | null
  /** （empty）； / to resolveLanConfirm */
  lanConfirm: LanConfirm
  /** （empty = to ） */
  lanRegister: LanRegister
  /** ：only */
  panelLogin: boolean
}

/**
 * 。`mode` after （enable need ，
 * can can ）；`panel` is 「」 （mode as panel when ）。
 */
export type LanConfirm = { mode: 'enable' | 'disable' | 'panel'; panel: boolean } | null

/** and （） */
export type DbRestoreProgress = {
  /** in （ when not to ） */
  active: boolean
  /** success */
  done: boolean
  /** failure */
  failed: boolean
  /** current can （ after to ， before ） */
  phase: string
  /** （ not ） */
  percent: number
  /** failureReason（`failed` when has ） */
  error: string
  /** （`done` when has ） */
  tables: number
  /** before file（`done` when has ） */
  safetyBackup: string
  /** not （`done` when can can has ） */
  warning: string
  reencryptedSecrets: number
  localKeysPreserved: boolean
  startedAt: number
  finishedAt: number
}

/** ：busy when by ，error empty when in */
export type LanRegister = { busy: boolean; error: string } | null

/** is （ use ， no has 「」 ） */
export function readPanelLogin(): boolean {
  return shared().workbuddyDesktop?.platform === 'web'
}

/** ： is 「 in …」， and 1 */
const INITIAL: SettingsSnapshot = {
  // 1 ：restoreCategory / load will localStorage up 
  category: CATEGORIES[0].id,
  scrollReset: 0,
  busy: null,
  app: {
    status: 'loading',
    closeToTray: false,
    autostart: false,
    lanAccess: false,
    lanPanel: false,
    lightweightMode: false,
    lanIp: null,
    port: 0,
    adminRegistered: null,
  },
  unitsChinese: true,
  retention: { status: 'loading', values: null },
  retry: { status: 'loading', values: null },
  retryCodes: null,
  timeouts: { status: 'loading', values: null },
  queue: { status: 'loading', values: null },
  imageCompress: { status: 'loading', enabled: true, values: null },
  captchaPool: { status: 'loading', values: null },
  routing: { status: 'loading', strategy: 'order', sticky: 'auto', failoverBackoff: true, sink: true, breakerEnabled: false },
  debug: { status: 'loading', on: false, count: null, limit: null },
  sanitize: INITIAL_SANITIZE,
  developerRoleCompat: { status: 'loading', on: false },
  clineHeaders: { status: 'loading', defaults: {}, overrides: {}, effective: {} },
  zcodeVersion: { status: 'loading', version: '', envOverride: '' },
  cors: { status: 'loading', on: false },
  prompt: {
    status: 'loading',
    mode: 'passthrough',
    file: '',
    text: '',
    source: '',
    lines: 0,
    fileError: '',
    degradeActive: false,
    degradeUntilText: '',
    providers: [],
    gateway: {},
    gatewayText: {},
    options: [],
  },
  storage: {
    status: 'loading',
    available: true,
    file: '',
    bytes: 0,
    accounts: null,
    logs: null,
    requests: null,
    dailyDays: null,
    downstreamRaw: null,
    upstreamRaw: null,
  },
  captcha: { available: true, desktop: false, web: false, localPanel: false },
  account: { status: 'loading', registered: false, username: '', password: null },
  modelsDev: { status: 'loading', enabled: true, source: '', updatedAt: 0, count: 0 },
  ioFailure: null,
  retentionConfirm: null,
  // ：default and （）；default
  dbIncludeLogs: true,
  dbExportConfirm: false,
  dbBackupConfirm: null,
  dbWarning: null,
  dbRestore: null,
  lanConfirm: null,
  lanRegister: null,
  panelLogin: false,
}

/* ─── store ───────────────────────────── */

let snapshot: SettingsSnapshot = { ...INITIAL, panelLogin: readPanelLogin() }

const subscribers = new Set<() => void>()

export function subscribe(listener: () => void): () => void {
  subscribers.add(listener)
  return () => { subscribers.delete(listener) }
}

export function getSnapshot(): SettingsSnapshot {
  return snapshot
}

/** （useSyncExternalStore use ，publish must new for ） */
export function useSettings(): SettingsSnapshot {
  return React.useSyncExternalStore(subscribe, getSnapshot)
}

function publish(patch: Partial<SettingsSnapshot>): void {
  snapshot = { ...snapshot, ...patch }
  for (const listener of subscribers) listener()
}

/**
 * `app` （** app 1 this **）。
 *
 * not can `publish({ app: { ...snapshot.app, ... } })`： for in 
 * ** that 1 **。 use need await， old 、 etc await
 * —— this when and its （`loadSettings` status/lanAccess、
 * lanIp/port） already will old 。
 *
 * （ up after ）：`loadLanExtras` `...snapshot.app` and 
 * `await api.localIp()` in 1 ，、「」 after when ，
 * set in 「 in …」、 —— set its 
 * （ after 0.0.0.0）， is old 。 then in 
 * ** that 1 **， after not will 。
 */
function publishApp(patch: Partial<AppState>): void {
  publish({ app: { ...snapshot.app, ...patch } })
}

/**
 * 1 （ not ）。
 *
 * use has 1 ： in 「 in 」 when need —— old is 
 * DOM checked ，React this need 1 can 。
 */
function repaint(): void {
  publish({})
}

/**
 * 「 1 1 」 （ old panelBusy）。
 *
 * not is ： need **** to （ 1 down 、disabled 
 * blur change ）。 busy use ， not 。
 */
let busyScope: BusyScope = null

function beginBusy(scope: Exclude<BusyScope, null>): void {
  busyScope = scope
  publish({ busy: scope })
}

function endBusy(): void {
  busyScope = null
  publish({ busy: null })
}

/* ─── ：current / ─────────── */

/**
 * 。 by （ not by HTML ），； can can 
 * localStorage、 can can use ， not in to 1 ， when has 1 。
 * to （ scrollReset）。
 *
 * ** not ** localStorage：update-panel 「update」 will to 「update」，
 * that not this use default（ is selectCategory）。
 */
export function showCategory(category?: string | null): void {
  const target = CATEGORIES.some(item => item.id === category)
    ? String(category)
    : CATEGORIES[0].id
  publish({ category: target, scrollReset: snapshot.scrollReset + 1 })
}

/** ： and */
export function selectCategory(category: string): void {
  showCategory(category)
  try { localStorage.setItem(SETTINGS_CAT_KEY, category) } catch { /* unavailable down */ }
}

/** by localStorage up in （ by showCategory ） */
export function restoreCategory(): void {
  let saved: string | null = null
  try { saved = localStorage.getItem(SETTINGS_CAT_KEY) } catch { saved = null }
  showCategory(saved)
}

/**
 * is before （ and this 、default is by units.js ），
 * this current、 and in when 。 1 down ：
 * `wb-units-changed`， to after use that 。
 */
export function renderUnits(): void {
  publish({ unitsChinese: shared().wbUnits?.isChinese?.() !== false })
}

export function applyUnits(on: boolean): void {
  shared().wbUnits?.setChinese?.(on)
  renderUnits()
  toast(on ? t('✅ 已改用本地量级词（万 / 亿式）') : t('✅ 已改用 k / M / B 缩写'))
}

/* ─── and ───────────────────────────── */

/**
 * set。 undefined not （React is ， no has 「 by old 」 this ）；
 * null / for by 「return」，**current** —— old that when no DOM 
 * checked， new failure not this use 。
 */
export function renderSettings(data?: unknown): void {
  if (data === undefined) return
  if (!data || typeof data !== 'object') {
    publishApp({ status: 'unavailable' })
    return
  }
  const record = data as Record<string, unknown>
  // return 1 by 「 true」， when by （ and after default 1 ）。
  // lanIp / port / adminRegistered is ， not in set response —— current。
  publishApp({
    status: 'ready',
    closeToTray: record.closeToTray === true,
    autostart: record.autostart === true,
    lanAccess: record.lanAccess === true,
    lanPanel: record.lanPanel === true,
    lightweightMode: record.lightweightMode === true,
  })
}

async function loadSettings(): Promise<void> {
  try {
    renderSettings(await shared().workbuddyDesktop?.getAppSettings())
  } catch (error) {
    console.warn('读取应用设置失败:', errorMessage(error))
    renderSettings(null)
  }
}

/**
 * ： IP、、。
 *
 * 、failure old —— and ，
 * 1 is empty not this set。web （shim no has this ）：
 * not this 1 。
 */
async function loadLanExtras(): Promise<void> {
  const api = shared().workbuddyDesktop
  if (!api?.localIp) return
  try {
    publishApp({ lanIp: (await api.localIp()) || null })
  } catch { /* not to not ， */ }
  try {
    const port = Number((await api.getBackendStatus?.())?.port) || 0
    if (port) publishApp({ port })
  } catch { /* 0， */ }
  try {
    const registered = (await api.panelAdminStatus?.())?.registered === true
    publishApp({ adminRegistered: registered })
  } catch { /* 「」； will 1 new */ }
}

/**
 * / 。patch is ****（ need ）， its 
 * current —— old is DOM that checkbox checked， etc 。
 * in when not ： checked ，「 this 1 down 」。
 */
export async function saveToggle(kind: 'tray' | 'autostart' | 'lightweight', next: boolean): Promise<void> {
  if (busyScope) { repaint(); return }
  const previous = snapshot.app
  const patch: AppSettings = {
    closeToTray: kind === 'tray' ? next : previous.closeToTray,
    autostart: kind === 'autostart' ? next : previous.autostart,
    // 「」（ has and ），
    // patch up ， 
    lanAccess: previous.lanAccess,
    lanPanel: previous.lanPanel,
    lightweightMode: kind === 'lightweight' ? next : previous.lightweightMode,
  }
  beginBusy('app')
  publishApp(patch)
  const label = kind === 'autostart'
    ? t('开机自动启动')
    : kind === 'lightweight'
      ? t('轻量模式')
      : t('关闭窗口时最小化到托盘')
  try {
    const api = shared().workbuddyDesktop
    if (!api) throw new Error(t('主进程桥不可用'))
    const saved = await api.saveAppSettings(patch)
    // return set as ， and not 1 （current）
    publishApp({ status: 'ready', ...normalizeApp(saved, patch) })
    toast(t('✅ 已更新「{label}」', { label }))
  } catch (error) {
    // to before （ old ： already by ，unknown this ）
    publishApp(previous)
    toast(t('保存失败：{message}', { message: errorMessage(error) }), 'err')
  } finally {
    endBusy()
  }
}

/* ─── （only；） ── */

/**
 * 「」。 and ：
 * （ + can can need ）， up ，
 * and need ** use ** —— this this use 。
 * before not （），success after new 。
 */
export function toggleLan(next: boolean): void {
  if (busyScope) { repaint(); return }
  publish({ lanConfirm: { mode: next ? 'enable' : 'disable', panel: snapshot.app.lanPanel } })
}

/** 「 when 」（only in when can ； need use ） */
export function toggleLanPanel(next: boolean): void {
  if (busyScope) { repaint(); return }
  publish({ lanConfirm: { mode: 'panel', panel: next } })
}

/** （ and resolveRetentionConfirm ）： true / and false */
export function resolveLanConfirm(accepted: boolean): void {
  const current = snapshot.lanConfirm
  publish({ lanConfirm: null })
  if (!current || !accepted) return
  if (current.mode === 'disable') { void applyLan(false, false); return }
  if (current.mode === 'panel') { void applyLan(true, current.panel); return }
  void proceedEnable()
}

/**
 * ： after （ 1 ， not use when old ——
 * from to can can ）。 already use ； no ，
 * success after none（ is use in to that 「 will 」）。
 */
async function proceedEnable(): Promise<void> {
  const api = shared().workbuddyDesktop
  if (!api?.panelAdminStatus) { toast(t('当前环境不支持局域网访问设置'), 'err'); return }
  beginBusy('lan')
  let registered: boolean
  try {
    registered = (await api.panelAdminStatus())?.registered === true
  } catch (error) {
    endBusy()
    toast(t('查询管理员状态失败：{message}', { message: errorMessage(error) }), 'err')
    return
  }
  endBusy()
  publishApp({ adminRegistered: registered })
  if (registered) {
    await applyLan(true, snapshot.app.lanPanel)
    return
  }
  publish({ lanRegister: { busy: false, error: '' } })
}

/**
 * ： → → 。 and after 1 
 * （ 64 、 8 ）；「 already in 」 by 
 * `existed: true`， this success —— 「 in has no has 」。
 */
export async function submitLanRegister(username: string, password: string): Promise<void> {
  if (busyScope) return
  const api = shared().workbuddyDesktop
  if (!api?.panelRegister) return
  const name = username.trim()
  if (!name || name.length > 64) {
    publish({ lanRegister: { busy: false, error: t('请填写管理员账号（64 字符以内）') } })
    return
  }
  if (password.length < 8) {
    publish({ lanRegister: { busy: false, error: t('密码至少 8 位') } })
    return
  }
  publish({ lanRegister: { busy: true, error: '' } })
  try {
    await api.panelRegister(name, password)
    // use in ： in （ already ，none，
    // down will ）， not can 1 already use use 
    if (snapshot.lanRegister === null) return
    publish({ lanRegister: null })
    publishApp({ adminRegistered: true })
    await applyLan(true, snapshot.app.lanPanel)
  } catch (error) {
    publish({ lanRegister: { busy: false, error: errorMessage(error) } })
  }
}

/** ： in ，（ if already ，none） */
export function cancelLanRegister(): void {
  publish({ lanRegister: null })
}

/**
 * use ：set and （「 Key」 in ，return）。
 * successpath up use up 、， this toast 1 down —— failurepath
 * （failure、 ）must ， loadSettings 。
 */
async function applyLan(enabled: boolean, panel: boolean): Promise<void> {
  if (busyScope) return
  const api = shared().workbuddyDesktop
  if (!api?.changeLanAccess) { toast(t('当前环境不支持局域网访问设置'), 'err'); return }
  beginBusy('lan')
  publishApp({ lanAccess: enabled, lanPanel: panel })
  try {
    const result = await api.changeLanAccess(enabled, panel) as { createdKey?: unknown } | null | undefined
    if (result?.createdKey) toast(t('✅ 已自动创建网关 Key「默认」（可在「网关 Key」页查看）'))
    toast(t('✅ 设置已保存，应用正在重启…'))
  } catch (error) {
    toast(t('保存失败：{message}', { message: errorMessage(error) }), 'err')
    await loadSettings() // 回滚到磁盘上的真实值
  } finally {
    endBusy()
  }
}

/* ─── / ──────────────────────── */

/**
 * 。 and old guard() ： down that by use and （ by 
 * busy === 'export' ）， 1 by can will down （ old is this ）。
 */
export async function exportAccounts(): Promise<void> {
  if (busyScope) return
  publish({ ioFailure: null })
  beginBusy('export')
  try {
    const result = await shared().workbuddyDesktop?.exportAccounts()
    if (result?.canceled) { toast(t('已取消导出')); return }
    const count = Number(result?.count) || 0
    const providers = Number(result?.customProviders) || 0
    if (!count && !providers) { toast(t('没有可导出的账号'), 'err'); return }
    // v2 file： as 0 has when 
    const providerNote = providers ? t('、{n} 个自定义提供商', { n: providers }) : ''
    const fileNote = result?.file ? t(' 到 {file}', { file: result.file }) : ''
    toast(t('✅ 已导出 {count} 个账号{note}{file}', { count, note: providerNote, file: fileNote }))
  } catch (error) {
    toast(t('操作失败：{message}', { message: errorMessage(error) }), 'err')
  } finally {
    endBusy()
  }
}

/** ： and in after ， this （failure， by ） */
export async function importAccounts(): Promise<void> {
  if (busyScope) return
  publish({ ioFailure: null })
  beginBusy('import')
  try {
    const result = await shared().workbuddyDesktop?.importAccounts()
    if (result?.canceled) { toast(t('已取消导入')); return }

    const added = Number(result?.added) || 0
    const updated = Number(result?.updated) || 0
    const skipped = Number(result?.skipped) || 0
    const failed = Number(result?.failed) || 0
    const errors = Array.isArray(result?.errors) ? result.errors : []
    const custom = result?.customProviders ?? {}
    const customAdded = Number(custom.added) || 0
    const customUpdated = Number(custom.updated) || 0

    const extras: string[] = []
    if (skipped) extras.push(t('跳过 {n} 个', { n: skipped }))
    if (failed) extras.push(t('失败 {n} 个', { n: failed }))
    const suffix = extras.length ? t('，{items}', { items: extras.join(t('、')) }) : ''
    const providerNote = (customAdded || customUpdated)
      ? t('，自定义提供商新增 {added} 个、更新 {updated} 个', { added: customAdded, updated: customUpdated })
      : ''
    const summary = t('新增 {added} 个、更新 {updated} 个{suffix}{note}', {
      added, updated, suffix, note: providerNote,
    })

    if (failed) {
      toast(t('导入完成：{summary}', { summary }), 'err')
      // failure before 3 ， and 1 ；
      // （customProvider ） no has ， when 
      const detail = errors.slice(0, 3)
        .map(item => {
          const label = item?.customProvider
            ? t('自定义提供商 {id}', { id: item?.id || t('(无 id)') })
            : (item?.id ?? t('未知账号'))
          const message = item?.message ?? t('未知原因')
          return t('{label}（{message}）', { label, message })
        })
        .join(t('；'))
      publish({ ioFailure: { failed, detail, more: errors.length > 3 } })
    } else {
      toast(t('✅ 导入完成：{summary}', { summary }))
    }

    // （add/update） after ：、 etc ；
    // has when need （、 will ）
    if (added || updated || customAdded || customUpdated) await shared().wbApp?.refresh?.()
  } catch (error) {
    toast(t('操作失败：{message}', { message: errorMessage(error) }), 'err')
  } finally {
    endBusy()
  }
}

/* ─── / （， db::backup module header） ──────────────── */

/** 「 and 」（default up ； after ） */
export function toggleDbIncludeLogs(on: boolean): void {
  publish({ dbIncludeLogs: on })
}

/**
 * ： by after to path（file not in ）， by 
 * down （ web_shim.rs），return。 no has 「empty not 」。
 */
export function confirmDbExport(): void {
  if (busyScope || snapshot.dbRestore?.active) return
  publish({ dbExportConfirm: true })
}

export function cancelDbExport(): void {
  publish({ dbExportConfirm: false })
}

export async function exportDatabase(password: string): Promise<void> {
  if (busyScope || snapshot.dbRestore?.active) return
  publish({ dbExportConfirm: false })
  beginBusy('dbExport')
  try {
    const api = shared().workbuddyDesktop
    if (!api?.exportDatabase) {
      toast(t('当前环境不支持整库备份'), 'err')
      return
    }
    const result = await api.exportDatabase(snapshot.dbIncludeLogs, password)
    if (result?.canceled) { toast(t('已取消导出')); return }
    const size = Number(result?.bytes) || 0
    const tables = Array.isArray(result?.tables) ? result.tables.length : 0
    toast(t('✅ 已加密备份 {n} 张表（{size}），包含恢复密钥：{file}', {
      n: tables,
      size: formatBytes(size),
      file: result?.file || '',
    }))
  } catch (error) {
    toast(t('操作失败：{message}', { message: errorMessage(error) }), 'err')
  } finally {
    endBusy()
  }
}

/** 「」：（ is ， after ） */
export function confirmDbRestore(): void {
  if (busyScope || snapshot.dbRestore?.active) return
  publish({ dbBackupConfirm: { includeLogs: snapshot.dbIncludeLogs } })
}

/** （ and resolveRetentionConfirm ）： after */
export function resolveDbBackupConfirm(accepted: boolean, password = ''): void {
  publish({ dbBackupConfirm: null })
  if (accepted) void proceedDbRestore(password)
}

/**
 * 。
 *
 * ── Why in 1 「 not can 」 ──────────────
 * is ****： 。 1 not can —— in will down 
 * 1 not is old not is new （ to 1 、 to 1 ）。：
 * · ** no has by **，Esc / / empty not （`onOpenChange` ignore）；
 * · by after to （`GET /api/backup/progress`， `db::backup` ），
 * before 500ms 1 —— this use in new ，set can up 
 * 1 （ by ， not is ）。
 *
 * ── in this ──────────────────────────────────
 * success to 「 N 」+ file；failure to Reason。 not 
 * when `dbWarning`（）， as use after need need to 。
 */
async function proceedDbRestore(password: string): Promise<void> {
  if (busyScope) return
  const api = shared().workbuddyDesktop
  if (!api?.importDatabase) {
    toast(t('当前环境不支持整库恢复'), 'err')
    return
  }
  beginBusy('dbImport')
  // 1 ： up 1 down （ and ioFailure 1 ）
  //
  // ── Why this not up ──────────────────────────────
  // `importDatabase()` is 「file → → after 」 1 use ，
  // in that can can （ use in filedirectory）。if this when ，
  // use will to 1 「 in file…」 —— file no 。
  // by ** after **： 1 to `active: true` 。
  publish({ dbWarning: null })
  // ： after `startedAt` 1 。 use 「 up 1 down 
  // already 」 and 「 this 1 」（file for can can need ， this 
  // after to is up 1 Done）。
  const attemptStartedAt = Date.now()
  const stop = watchRestoreProgress(attemptStartedAt)
  try {
    const result = await api.importDatabase(password)
    // （ file / file {canceled:true}）：
    // not is failure not is 「 0 」， can 
    if (result?.canceled) {
      stop()
      publish({ dbRestore: null })
      toast(t('已取消恢复'))
      return
    }
    const tables = Array.isArray(result?.tables) ? result.tables.length : 0
    // not （ will none）： after to ， this 
    // in **** —— that 1 ， use after will for empty
    // ， not 「Why no 」
    const warning = typeof result?.warning === 'string' ? result.warning.trim() : ''
    if (warning) publish({ dbWarning: warning })
    stop()
    // in 「」， etc use 「」—— need 
    publish({
      dbRestore: {
        active: false,
        done: true,
        failed: false,
        phase: t('恢复完成'),
        percent: 100,
        error: '',
        tables,
        safetyBackup: typeof result?.safetyBackup === 'string' ? result.safetyBackup : '',
        warning,
        reencryptedSecrets: Number(result?.reencryptedSecrets) || 0,
        localKeysPreserved: result?.localKeysPreserved === true,
        startedAt: snapshot.dbRestore?.startedAt ?? Date.now(),
        finishedAt: Date.now(),
      },
    })
    // （/set、、、 etc ）。
    //
    // ── this failure** not can **failure ────────────────────────
    // after 1 is empty（`access::clear_access_tokens`），
    // `load()` can can to 401 —— that is 「 need new 」， not is 「
    // no success」。 this in try ， 1 1 success failure。
    // in ： up publish as ， new failure。
    try {
      await load()
      await shared().wbApp?.refresh?.()
    } catch {
      /* new failure not ； use down 1 will */
    }
  } catch (error) {
    const message = errorMessage(error)
    if (snapshot.dbRestore?.done) {
      stop()
      return
    }
    // ── failure ────────────────────────────────────────
    // is 1 use ： HTTP 、 fetch can can after 
    // （ 61 seconds up 60 seconds）。 that when after old in ， ——
    // 1 「failure」 not is ， will use 1 ， 1 
    // 。 after 1 after ：** in not **， after to 
    // in that （ will etc to after down Done / Failed）。
    if (await restoreStillRunning(api)) {
      publish({
        dbRestore: {
          ...EMPTY_RESTORE,
          ...snapshot.dbRestore,
          active: true,
          done: false,
          failed: false,
          phase: t('恢复仍在进行中…'),
          startedAt: snapshot.dbRestore?.startedAt || Date.now(),
        },
      })
      toast(t('请求已断开，但恢复仍在后台进行 —— 请等待进度弹窗给出结果'))
      return
    }
    stop()
    publish({
      dbRestore: {
        active: false,
        done: false,
        failed: true,
        phase: t('恢复失败'),
        percent: 100,
        error: message,
        tables: 0,
        safetyBackup: '',
        warning: '',
        reencryptedSecrets: 0,
        localKeysPreserved: false,
        startedAt: snapshot.dbRestore?.startedAt ?? Date.now(),
        finishedAt: Date.now(),
      },
    })
  } finally {
    endBusy()
  }
}

/** after is in （ not to by 「 no has 」，failure up ） */
async function restoreStillRunning(api: ReturnType<typeof shared>['workbuddyDesktop']): Promise<boolean> {
  try {
    const data = await api?.getBackupProgress?.()
    return data?.active === true
  } catch {
    return false
  }
}

/** （`dbRestore` as null 「 no has in 」） */
const EMPTY_RESTORE: DbRestoreProgress = {
  active: false,
  done: false,
  failed: false,
  phase: '',
  percent: 0,
  error: '',
  tables: 0,
  safetyBackup: '',
  warning: '',
  reencryptedSecrets: 0,
  localKeysPreserved: false,
  startedAt: 0,
  finishedAt: 0,
}

/** ：500ms。 when seconds， is */
const RESTORE_POLL_MS = 500

/**
 * total when limit：40 minutes。
 *
 * request（`commands.rs` `BACKUP_TRANSFER_TIMEOUT_MS`，30 minutes）
 * 10 minutes —— when after in ， need etc to 。
 */
const RESTORE_POLL_MAX_MS = 40 * 60 * 1000

/**
 * ，return「」。
 *
 * ── Why need failure ────────────────────────────────────
 * after 1 is `config::reload()` and empty， will ——
 * this in has 1 request can can to 401 / 。 that not is failure， is 「 no 
 * up 」，**ignore**error， `importDatabase` return as 。
 */
function watchRestoreProgress(attemptStartedAt: number): () => void {
  let stopped = false
  let querying = false
  // limit： already to 30 minutes， this to 1 。 to etc not to 
  // not （ when will ）—— 1 no has interval
  // 1 in after 。
  const deadline = Date.now() + RESTORE_POLL_MAX_MS
  let timer = 0
  const tick = async () => {
    if (stopped) return
    if (Date.now() > deadline) {
      stopped = true
      window.clearInterval(timer)
      return
    }
    if (querying) return
    querying = true
    try {
      const data = await shared().workbuddyDesktop?.getBackupProgress?.()
      if (stopped || !data) return
      const startedAt = Number(data.startedAt) || 0
      // up 1 down （`startedAt` ）： not is this ，
      // 1 none —— otherwise use in filefile， already 「」。
      if (startedAt && startedAt < attemptStartedAt) return
      if (!startedAt) return
      const current = snapshot.dbRestore
      // ── after already ： by ──────────────────────────
      // down `importDatabase()` return will 1 ，path to 
      // 1 （ 1 ）。**request** when has this can —— when after 
      // after in ， this 1 can etc to ，otherwise will in 
      // 「 in in …」。
      if (data.active !== true) {
        const body = (data.result ?? {}) as { tables?: unknown; warning?: unknown; safetyBackup?: unknown; reencryptedSecrets?: unknown; localKeysPreserved?: unknown }
        const tables = Array.isArray(body.tables) ? body.tables.length : 0
        const warning = typeof body.warning === 'string' ? body.warning.trim() : ''
        const failed = data.failed === true
        if (!failed && data.done !== true) return
        if (failed && !(typeof data.error === 'string' && data.error.trim())) {
          // no by 、 not in ： is ， etc down 1 
          return
        }
        publish({
          dbRestore: {
            ...EMPTY_RESTORE,
            ...current,
            active: false,
            done: !failed,
            failed,
            phase: failed ? t('恢复失败') : t('恢复完成'),
            percent: 100,
            error: failed ? String(data.error ?? '') : '',
            tables,
            safetyBackup: typeof body.safetyBackup === 'string' ? body.safetyBackup : '',
            warning,
            reencryptedSecrets: Number(body.reencryptedSecrets) || 0,
            localKeysPreserved: body.localKeysPreserved === true,
            startedAt: startedAt || current?.startedAt || attemptStartedAt,
            finishedAt: Number(data.finishedAt) || Date.now(),
          },
        })
        if (warning) publish({ dbWarning: warning })
        // already ：。 after success that need —— ，
        // config、、 etc 1 old 。
        //
        // in 「 already not etc this request」 when （`busyScope` already ）：successpath
        // up will `load()`， this 1 is 1 。 
        // then —— already ， has this can up this new 。
        stopped = true
        window.clearInterval(timer)
        if (!failed && !busyScope) {
          try {
            await load()
            await shared().wbApp?.refresh?.()
          } catch {
            /* new failure not （ by `proceedDbRestore` success） */
          }
        }
        return
      }
      // 1 to after in = use file、 → this when 
      publish({
        dbRestore: {
          ...EMPTY_RESTORE,
          ...current,
          active: true,
          done: false,
          failed: false,
          phase: typeof data.phase === 'string' && data.phase ? data.phase : (current?.phase ?? t('正在恢复…')),
          // not ： after to is ， this 1 ，
          percent: Math.max(current?.percent ?? 0, Number(data.percent) || 0),
          startedAt: startedAt || current?.startedAt || Date.now(),
        },
      })
    } catch {
      /* ： */
    } finally {
      querying = false
    }
  }
  void tick()
  timer = window.setInterval(() => void tick(), RESTORE_POLL_MS)
  return () => {
    stopped = true
    window.clearInterval(timer)
  }
}

async function reconnectDbRestore(): Promise<void> {
  if (busyScope || snapshot.dbRestore) return
  try {
    const progress = await shared().workbuddyDesktop?.getBackupProgress?.()
    if (progress?.active) watchRestoreProgress(Number(progress.startedAt) || Date.now())
  } catch {
    /* unavailable not its set。 */
  }
}

/** （ in `done` / `failed` after can ） */
export function dismissDbRestore(): void {
  if (snapshot.dbRestore?.active) return
  publish({ dbRestore: null })
}

/* ─── ──────────────────────────── */

export function renderRetention(data?: unknown): void {
  if (data === undefined) return
  const values = normalizeNumbers(RETENTION_FIELDS, data, snapshot.retention.values)
  publish({ retention: { status: values === null ? 'unavailable' : 'ready', values } })
}

async function loadRetention(): Promise<void> {
  try {
    renderRetention(await shared().workbuddyDesktop?.getRetention())
  } catch (error) {
    console.warn('读取数据保留设置失败:', errorMessage(error))
    renderRetention(null)
  }
}

/** Promise resolver；empty */
let retentionConfirmResolver: ((accepted: boolean) => void) | null = null

/** and to etc （ use none use ：resolver empty） */
export function resolveRetentionConfirm(accepted: boolean): void {
  const resolve = retentionConfirmResolver
  retentionConfirmResolver = null
  publish({ retentionConfirm: null })
  resolve?.(accepted)
}

/** ，return Promise<boolean>： as true， / / Esc as false */
function askRetentionShrink(head: string): Promise<boolean> {
  return new Promise(resolve => {
    // up when has 1 （saveRetentionField already ）， this 1 ：
    // 1 has ， old by 「」， not is Promise 
    if (retentionConfirmResolver) resolveRetentionConfirm(false)
    retentionConfirmResolver = resolve
    publish({ retentionConfirm: { head } })
  })
}

/**
 * ： → （ when ） → 。 1 failure not ，
 * ， to （ old is revert）。
 */
export async function saveRetentionField(field: RetentionField, raw: string): Promise<void> {
  if (busyScope) return
  // when not new ： 1 1 ，otherwise will 1 
  // （resolver can 1 ）， that will in no 「」 down 。
  // already ， to this Tab path， 1 down 。
  if (snapshot.retentionConfirm) return

  const parsed = parseInteger(raw, field.min, field.max, field.kind === 'count' ? t('条数') : t('天数'))
  if (!parsed.ok) { toast(parsed.message, 'err'); return }

  const known = snapshot.retention.values?.[field.key]
  const previous = typeof known === 'number' && Number.isInteger(known) ? known : null
  // and after 1 not request： （ 007） will change
  if (previous !== null && parsed.value === previous) return

  // not to old when none from is —— has will ， this can 1 ：
  // 1 ， use 
  const shrinking = previous === null || parsed.value < previous
  if (shrinking && !await askRetentionShrink(retentionShrinkHead(field, previous, parsed.value))) return

  await commitRetention(field, parsed.value, shrinking)
}

/** ， its set。 */
async function commitRetention(field: RetentionField, value: number, shrinking: boolean): Promise<void> {
  if (busyScope) return
  beginBusy('retention')
  // ， use （ by busy === 'retention' use ）。 by ：Chromium 
  // 「 in disabled」 will 1 blur， blur can can change —— that 
  // will ； not new ， use will to 「 old 、 1 down 」。
  // failure down catch will after ， this not will in up 。
  const optimistic = snapshot.retention.values
  if (optimistic) publish({ retention: { status: 'ready', values: { ...optimistic, [field.key]: value } } })
  try {
    const saved = await shared().workbuddyDesktop?.saveRetention({ [field.key]: value })
    renderRetention(saved)
    const values = snapshot.retention.values
    if (RETENTION_FIELDS.every(item => Number.isInteger(values?.[item.key]))) {
      toast(retentionSavedMessage(field, values?.[field.key] ?? value, shrinking))
      return
    }
    // response not when new ， already set。
    await loadRetention()
    toast(retentionSavedMessage(field, snapshot.retention.values?.[field.key] ?? value, shrinking))
  } catch (error) {
    // 400 message（ 、） 1 to 
    toast(t('保存失败：{message}', { message: errorMessage(error) }), 'err')
    await loadRetention() // 回滚到后端的真实值
  } finally {
    endBusy()
  }
}

/* ─── set use （request / request when use ）── */

/**
 * 1 「 after 1 + up if 」 ：、、
 * failure、 after unavailable when use 。****（ is 「 + update
 * + return」 that ）， 1 。 not and —— has and 
 * use ， not 。
 *
 * `syncExtras(data | null)`：； 1 （ 「error
 * 」） by ， in to / unavailable when 1 。
 */
type NumericPanel = {
  scope: 'retry' | 'timeouts' | 'queue' | 'imageCompress' | 'captchaPool'
  fields: NumberField[]
  consoleLabel: string
  /** / ： not in when to undefined（ use by 「 not to 」） */
  get: () => Promise<unknown> | undefined
  save: (patch: Record<string, number>) => Promise<unknown> | undefined
  /** / （null = unavailable） */
  read: () => NumericState
  write: (state: NumericState) => void
  syncExtras?: (data: unknown) => void
}

/** by response： ， use up 1 ； 1 no has then unavailable */
function renderNumericPanel(panel: NumericPanel, data: unknown): void {
  if (data === undefined) return
  const values = normalizeNumbers(panel.fields, data, panel.read().values)
  panel.write({ status: values === null ? 'unavailable' : 'ready', values })
  panel.syncExtras?.(values === null ? null : data)
}

async function loadNumericPanel(panel: NumericPanel): Promise<void> {
  try {
    renderNumericPanel(panel, await panel.get())
  } catch (error) {
    console.warn(`${panel.consoleLabel}失败:`, errorMessage(error))
    renderNumericPanel(panel, null)
  }
}

/** ： → ，failure（ and ， 1 ） */
async function saveNumericField(panel: NumericPanel, field: NumberField, raw: string): Promise<void> {
  if (busyScope) return

  const parsed = parseInteger(raw, field.min, field.max, field.label)
  if (!parsed.ok) { toast(parsed.message, 'err'); return }

  const known = panel.read().values?.[field.key]
  // and after 1 not request： （ 05） will change
  if (Number.isInteger(known) && parsed.value === known) return

  beginBusy(panel.scope)
  // （ by commitRetention）
  const current = panel.read().values
  if (current) panel.write({ status: 'ready', values: { ...current, [field.key]: parsed.value } })
  try {
    const saved = await panel.save({ [field.key]: parsed.value })
    // PUT return after ， down use response new can ， not 1 GET
    renderNumericPanel(panel, saved)
    const applied = panel.read().values?.[field.key]
    const shown = Number.isInteger(applied) ? applied : parsed.value
    toast(t('✅ 已保存：{label} {value}', { label: field.label, value: String(shown) }))
  } catch (error) {
    // 400 message（ 、） 1 to 
    toast(t('保存失败：{message}', { message: errorMessage(error) }), 'err')
    await loadNumericPanel(panel) // 回滚到后端的真实值
  } finally {
    endBusy()
  }
}

/* ─── etc ─────────────────────────────── */

const queuePanel: NumericPanel = {
  scope: 'queue',
  fields: QUEUE_FIELDS,
  consoleLabel: '读取排队等待设置',
  get: () => shared().workbuddyDesktop?.getQueue(),
  save: patch => shared().workbuddyDesktop?.saveQueue(patch),
  read: () => snapshot.queue,
  write: state => publish({ queue: state }),
}

export function renderQueue(data?: unknown): void {
  renderNumericPanel(queuePanel, data)
}

export async function loadQueue(): Promise<void> {
  await loadNumericPanel(queuePanel)
}

export async function saveQueueField(field: NumberField, raw: string): Promise<void> {
  await saveNumericField(queuePanel, field, raw)
}

/* ─── will （ + parameter）────────────── */

/** from response（ for / not is true ， and after 「 true」 1 ） */
function imageCompressEnabledFlag(data: unknown): boolean {
  if (!data || typeof data !== 'object') return false
  return (data as Record<string, unknown>)[IMAGE_COMPRESS_ENABLED_KEY] === true
}

/**
 * ： use / / ； 1 
 * 1 path（ and developerRoleCompat ）。 use 1 
 * `/api/image-compress` ，response when and parameter。
 */
const imageCompressPanel: NumericPanel = {
  scope: 'imageCompress',
  fields: IMAGE_COMPRESS_FIELDS,
  consoleLabel: '读取会话图片压缩设置',
  get: () => shared().workbuddyDesktop?.getImageCompress(),
  save: patch => shared().workbuddyDesktop?.saveImageCompress(patch),
  read: () => ({ status: snapshot.imageCompress.status, values: snapshot.imageCompress.values }),
  write: state => publish({
    imageCompress: { ...snapshot.imageCompress, status: state.status, values: state.values },
  }),
  // 1 response； not ， this 
  syncExtras: data => {
    publish({ imageCompress: { ...snapshot.imageCompress, enabled: imageCompressEnabledFlag(data) } })
  },
}

export function renderImageCompress(data?: unknown): void {
  renderNumericPanel(imageCompressPanel, data)
}

export async function loadImageCompress(): Promise<void> {
  await loadNumericPanel(imageCompressPanel)
}

export async function saveImageCompressField(field: NumberField, raw: string): Promise<void> {
  await saveNumericField(imageCompressPanel, field, raw)
}

/** path（ 1 ；failure to after ） */
export async function saveImageCompressEnabled(next: boolean): Promise<void> {
  if (busyScope) { repaint(); return }
  beginBusy('imageCompress')
  publish({ imageCompress: { ...snapshot.imageCompress, enabled: next } })
  try {
    const saved = await shared().workbuddyDesktop?.saveImageCompress({ [IMAGE_COMPRESS_ENABLED_KEY]: next })
    renderImageCompress(saved)
    toast(next ? t('✅ 会话图片压缩已开启') : t('已关闭会话图片压缩'))
  } catch (error) {
    toast(t('保存失败：{message}', { message: errorMessage(error) }), 'err')
    await loadImageCompress() // 回滚到后端的真实值
  } finally {
    endBusy()
  }
}

/* ─── ─────────────────────────── */

const captchaPoolPanel: NumericPanel = {
  scope: 'captchaPool',
  fields: CAPTCHA_POOL_FIELDS,
  consoleLabel: '读取验证码令牌池设置',
  get: () => shared().workbuddyDesktop?.getCaptchaPool(),
  save: patch => shared().workbuddyDesktop?.saveCaptchaPool(patch),
  read: () => snapshot.captchaPool,
  write: state => publish({ captchaPool: state }),
}

export function renderCaptchaPool(data?: unknown): void {
  renderNumericPanel(captchaPoolPanel, data)
}

export async function loadCaptchaPool(): Promise<void> {
  await loadNumericPanel(captchaPoolPanel)
}

export async function saveCaptchaPoolField(field: NumberField, raw: string): Promise<void> {
  await saveNumericField(captchaPoolPanel, field, raw)
}

/* ─── by （ + will ）───────────── */

/** after to by config （ and /api/routing for ） */
const ROUTING_STRATEGY_KEY = 'routingStrategy'
const ROUTING_STICKY_KEY = 'routingSticky'
const ROUTING_FAILOVER_BACKOFF_KEY = 'routingFailoverBackoff'
const ROUTING_SINK_KEY = 'routingSink'
const ROUTING_BREAKER_ENABLED_KEY = 'routingBreakerEnabled'

/** by config （ settings-model ， and use ） */
const ROUTING_VALID: Record<string, ReadonlySet<string>> = {
  [ROUTING_STRATEGY_KEY]: new Set(ROUTE_STRATEGIES.map(item => item.value)),
  [ROUTING_STICKY_KEY]: new Set(ROUTE_STICKY_OPTIONS.map(item => item.value)),
}

/** by response → ：， not to / when by this default */
function routingValuesFrom(data: unknown): {
  strategy: string
  sticky: string
  failoverBackoff: boolean
  sink: boolean
  breakerEnabled: boolean
} {
  const record = (data && typeof data === 'object' ? data : {}) as Record<string, unknown>
  const pick = (key: string, fallback: string): string => {
    const raw = record[key]
    return typeof raw === 'string' && ROUTING_VALID[key].has(raw) ? raw : fallback
  }
  // ： true / false（ after return 1 is ）， when after default
  const pickBool = (key: string, fallback: boolean): boolean => {
    const raw = record[key]
    return typeof raw === 'boolean' ? raw : fallback
  }
  return {
    strategy: pick(ROUTING_STRATEGY_KEY, 'order'),
    sticky: pick(ROUTING_STICKY_KEY, 'auto'),
    failoverBackoff: pickBool(ROUTING_FAILOVER_BACKOFF_KEY, true),
    sink: pickBool(ROUTING_SINK_KEY, true),
    breakerEnabled: pickBool(ROUTING_BREAKER_ENABLED_KEY, false),
  }
}

export async function loadRouting(): Promise<void> {
  try {
    const data = await shared().workbuddyDesktop?.getRoutingConfig()
    if (data === undefined) { publish({ routing: { ...snapshot.routing, status: 'unavailable' } }); return }
    publish({ routing: { status: 'ready', ...routingValuesFrom(data) } })
  } catch (error) {
    console.warn('读取路由设置失败:', errorMessage(error))
    publish({ routing: { ...snapshot.routing, status: 'unavailable' } })
  }
}

/**
 * 1 ：PUT return after （ and when / 1 ）， by response new 。
 * after ；unknown in up not （SegmentedControl ）。
 */
export async function saveRoutingField(patch: {
  routingStrategy?: string
  routingSticky?: string
  routingFailoverBackoff?: boolean
  routingSink?: boolean
  routingBreakerEnabled?: boolean
}): Promise<void> {
  if (busyScope) return
  const bridge = shared().workbuddyDesktop
  if (!bridge?.setRoutingConfig) {
    toast(t('桌面运行时不可用，暂时无法保存'), 'err')
    return
  }
  beginBusy('routing')
  try {
    const saved = await bridge.setRoutingConfig(patch)
    if (saved !== undefined) {
      publish({ routing: { status: 'ready', ...routingValuesFrom(saved) } })
    } else {
      // not response： 1 （loadRouting will to ready / unavailable）
      await loadRouting()
    }
    toast(t('✅ 路由设置已保存'))
  } catch (error) {
    toast(t('保存失败：{message}', { message: errorMessage(error) }), 'err')
    await loadRouting() // 回滚到后端的真实值
  } finally {
    endBusy()
  }
}

/* ─── request ─────────────────────────────── */

const retryPanel: NumericPanel = {
  scope: 'retry',
  fields: RETRY_FIELDS,
  consoleLabel: '读取请求重试设置',
  get: () => shared().workbuddyDesktop?.getRetry(),
  save: patch => shared().workbuddyDesktop?.saveRetry(patch),
  read: () => snapshot.retry,
  write: state => publish({ retry: state }),
  syncExtras: data => {
    // 「error」 ： 100–599 （ after already ， this not 
    // —— is after to ）。（ old after ） when use up 1 ， not 「empty」；
    // data as null（unavailable） when and 。
    if (data === null) { publish({ retryCodes: null }); return }
    const raw = (data as Record<string, unknown>)[NO_RETRY_CODES_KEY]
    if (!Array.isArray(raw)) return
    const list = raw as unknown[]
    publish({
      retryCodes: list.filter((code): code is number =>
        typeof code === 'number' && Number.isInteger(code)
        && code >= RETRY_CODE_MIN && code <= RETRY_CODE_MAX),
    })
  },
}

export function renderRetry(data?: unknown): void {
  renderNumericPanel(retryPanel, data)
}

export async function loadRetry(): Promise<void> {
  await loadNumericPanel(retryPanel)
}

export async function saveRetryField(field: NumberField, raw: string): Promise<void> {
  await saveNumericField(retryPanel, field, raw)
}

/** after 1 ：update → PUT → use response */
async function saveRetryCodes(codes: number[]): Promise<void> {
  if (busyScope) return
  publish({ retryCodes: codes })
  beginBusy('codes')
  try {
    const saved = await shared().workbuddyDesktop?.saveRetry({ [NO_RETRY_CODES_KEY]: codes })
    // PUT return after （ ）， to 1 ，
    // after （ after ）
    renderNumericPanel(retryPanel, saved)
    toast(t('✅ 已保存：指定错误码直接换号'))
  } catch (error) {
    toast(t('保存失败：{message}', { message: errorMessage(error) }), 'err')
    await loadNumericPanel(retryPanel) // 回滚到后端的真实值
  } finally {
    endBusy()
  }
}

/** and 1 ：、100–599、、（ and after 400 ） */
export async function addRetryCode(raw: string): Promise<void> {
  const codes = snapshot.retryCodes
  if (codes === null) return
  const text = String(raw ?? '').trim()
  if (!text) return
  if (!/^\d+$/.test(text) || Number(text) < RETRY_CODE_MIN || Number(text) > RETRY_CODE_MAX) {
    toast(t('状态码必须是 {min}–{max} 的整数（收到: {text}）', {
      min: RETRY_CODE_MIN, max: RETRY_CODE_MAX, text,
    }), 'err')
    return
  }
  const code = Number(text)
  if (codes.includes(code)) { toast(t('状态码 {code} 已在名单里', { code })); return }
  if (codes.length >= RETRY_MAX_CODES) {
    toast(t('名单最多 {max} 个状态码', { max: RETRY_MAX_CODES }), 'err')
    return
  }
  await saveRetryCodes([...codes, code])
}

/** delete 1 （ up ✕） */
export async function removeRetryCode(code: number): Promise<void> {
  const codes = snapshot.retryCodes
  if (!codes || !codes.includes(code)) return
  await saveRetryCodes(codes.filter(item => item !== code))
}

/** as empty when after 1 （ and GitHub Topics 1 ） */
export async function dropLastRetryCode(): Promise<void> {
  const codes = snapshot.retryCodes
  if (!codes?.length) return
  await saveRetryCodes(codes.slice(0, -1))
}

/* ─── request when ─────────────────────────────── */

const timeoutsPanel: NumericPanel = {
  scope: 'timeouts',
  fields: TIMEOUT_FIELDS,
  consoleLabel: '读取请求超时设置',
  get: () => shared().workbuddyDesktop?.getTimeouts(),
  save: patch => shared().workbuddyDesktop?.saveTimeouts(patch),
  read: () => snapshot.timeouts,
  write: state => publish({ timeouts: state }),
}

export async function loadTimeouts(): Promise<void> {
  await loadNumericPanel(timeoutsPanel)
}

export async function saveTimeoutField(field: NumberField, raw: string): Promise<void> {
  await saveNumericField(timeoutsPanel, field, raw)
}

/* ─── / ── */

export function renderDebug(data?: unknown): void {
  if (data === undefined) return
  if (!data || typeof data !== 'object') {
    publish({ debug: { status: 'unavailable', on: false, count: null, limit: null } })
    return
  }
  const record = data as Record<string, unknown>
  const count = Number(record.count)
  const limit = Number(record.limit)
  publish({
    debug: {
      status: 'ready',
      on: record.debugMode === true,
      count: Number.isInteger(count) ? count : null,
      limit: Number.isInteger(limit) ? limit : null,
    },
  })
}

async function loadDebug(): Promise<void> {
  try {
    renderDebug(await shared().workbuddyDesktop?.getDebug())
  } catch (error) {
    console.warn('读取调试模式设置失败:', errorMessage(error))
    renderDebug(null)
  }
}

export async function saveDebug(next: boolean): Promise<void> {
  // in ： checked ， not etc 「 this 1 down 」
  if (busyScope) { repaint(); return }
  beginBusy('debug')
  publish({ debug: { ...snapshot.debug, on: next } })
  try {
    const saved = await shared().workbuddyDesktop?.saveDebug(next)
    renderDebug(saved)
    toast(next ? t('✅ 调试模式已开启') : t('✅ 调试模式已关闭'))
  } catch (error) {
    toast(t('保存失败：{message}', { message: errorMessage(error) }), 'err')
    await loadDebug() // 回滚到后端的真实值
  } finally {
    endBusy()
  }
}

export function renderSanitize(data?: unknown): void {
  if (data === undefined) return
  publish({ sanitize: sanitizeStateFrom(data, snapshot.sanitize) })
}

async function loadSanitize(): Promise<void> {
  try {
    renderSanitize(await shared().workbuddyDesktop?.getSanitize())
  } catch (error) {
    console.warn('读取出站指纹脱敏设置失败:', errorMessage(error))
    renderSanitize(null)
  }
}

export async function saveSanitize(next: boolean): Promise<void> {
  await saveSanitizeConfig({ sanitizeBlacklistFingerprints: next }, next
    ? t('✅ 出站指纹脱敏已开启') : t('已关闭出站指纹脱敏'))
}

export async function saveSanitizeConfig(patch: SanitizePatch, message?: string): Promise<boolean> {
  if (busyScope) { repaint(); return false }
  beginBusy('sanitize')
  try {
    const api = shared().workbuddyDesktop
    if (!api?.saveSanitize) throw new Error(t('主进程桥不可用'))
    const request = !snapshot.sanitize.editable && typeof patch.sanitizeBlacklistFingerprints === 'boolean'
      ? patch.sanitizeBlacklistFingerprints : patch
    const data = await api.saveSanitize(request)
    const state = sanitizeStateFrom(data, snapshot.sanitize)
    if (state.status !== 'ready') throw new Error(t('未能读取保存后的指纹脱敏设置'))
    publish({ sanitize: state })
    toast(
      state.saved
        ? message ?? t('✅ 指纹脱敏设置已保存')
        : t('指纹脱敏设置仅本次运行生效，写入数据库失败，请检查配置目录'),
      state.saved ? 'ok' : 'err',
    )
    return true
  } catch (error) {
    toast(t('保存失败：{message}', { message: errorMessage(error) }), 'err')
    return false
  } finally {
    endBusy()
  }
}

/* ─── （developer 1 as system）── */

export function renderDeveloperRoleCompat(data?: unknown): void {
  if (data === undefined) return
  if (!data || typeof data !== 'object') {
    publish({ developerRoleCompat: { status: 'unavailable', on: false } })
    return
  }
  const record = data as Record<string, unknown>
  publish({ developerRoleCompat: { status: 'ready', on: record.developerRoleCompat === true } })
}

async function loadDeveloperRoleCompat(): Promise<void> {
  try {
    renderDeveloperRoleCompat(await shared().workbuddyDesktop?.getDeveloperRoleCompat())
  } catch (error) {
    console.warn('读取开发者角色兼容设置失败:', errorMessage(error))
    renderDeveloperRoleCompat(null)
  }
}

export async function saveDeveloperRoleCompat(next: boolean): Promise<void> {
  if (busyScope) { repaint(); return }
  beginBusy('developerRoleCompat')
  publish({ developerRoleCompat: { status: 'ready', on: next } })
  try {
    const saved = await shared().workbuddyDesktop?.saveDeveloperRoleCompat(next)
    renderDeveloperRoleCompat(saved)
    toast(next ? t('✅ 开发者角色兼容已开启') : t('已关闭开发者角色兼容'))
  } catch (error) {
    toast(t('保存失败：{message}', { message: errorMessage(error) }), 'err')
    await loadDeveloperRoleCompat() // 回滚到后端的真实值
  } finally {
    endBusy()
  }
}

/* ─── Cline （ ）── */

/** 1 （ after will to ， this is to 「 not for response」） */
function asStringTable(value: unknown): Record<string, string> {
  if (!value || typeof value !== 'object') return {}
  const out: Record<string, string> = {}
  for (const [key, text] of Object.entries(value as Record<string, unknown>)) {
    if (typeof text === 'string') out[key] = text
  }
  return out
}

export function renderClineHeaders(data?: unknown): void {
  if (data === undefined) return
  if (!data || typeof data !== 'object') {
    publish({ clineHeaders: { status: 'unavailable', defaults: {}, overrides: {}, effective: {} } })
    return
  }
  const record = data as ClineHeadersData
  publish({
    clineHeaders: {
      status: 'ready',
      defaults: asStringTable(record.defaults),
      overrides: asStringTable(record.overrides),
      effective: asStringTable(record.effective),
    },
  })
}

async function loadClineHeaders(): Promise<void> {
  try {
    renderClineHeaders(await shared().workbuddyDesktop?.getClineHeaders())
  } catch (error) {
    console.warn('读取 Cline 伪装头失败:', errorMessage(error))
    renderClineHeaders(null)
  }
}

export async function refreshClineHeaders(): Promise<void> {
  await loadClineHeaders()
  toast(t('Cline 伪装头已刷新'))
}

/**
 * （ and after PUT ）： after ——
 * default「default」 not 、empty = this not 、 = default。
 * empty = default（「default」 by is 1 empty）。
 */
export async function saveClineHeaders(overrides: Record<string, string>): Promise<void> {
  if (busyScope) { repaint(); return }
  beginBusy('clineHeaders')
  try {
    const saved = await shared().workbuddyDesktop?.saveClineHeaders(overrides)
    renderClineHeaders(saved)
    toast(t('✅ Cline 伪装头已保存'))
  } catch (error) {
    toast(t('保存失败：{message}', { message: errorMessage(error) }), 'err')
    await loadClineHeaders() // 回滚到后端的真实值
  } finally {
    endBusy()
  }
}

/* ─── ZCode client（ up by down not down ）── */

export function renderZcodeVersion(data?: unknown): void {
  if (data === undefined) return
  if (!data || typeof data !== 'object') {
    publish({ zcodeVersion: { status: 'unavailable', version: '', envOverride: '' } })
    return
  }
  const source = data as { version?: unknown; envOverride?: unknown }
  publish({
    zcodeVersion: {
      status: 'ready',
      version: typeof source.version === 'string' ? source.version : '',
      // null / 1 empty： need 「empty = 」
      envOverride: typeof source.envOverride === 'string' ? source.envOverride : '',
    },
  })
}

async function loadZcodeVersion(): Promise<void> {
  try {
    renderZcodeVersion(await shared().workbuddyDesktop?.getZcodeVersion())
  } catch (error) {
    console.warn('读取 ZCode 客户端版本号失败:', errorMessage(error))
    renderZcodeVersion(null)
  }
}

export async function refreshZcodeVersion(): Promise<void> {
  await loadZcodeVersion()
  toast(t('ZCode 版本号已刷新'))
}

/**
 * （empty = empty、default）。
 *
 * after and ，** not 「 this in 」** —— up for 
 * is 。 is 「 is not to 」， after after ，
 * use to to is 。
 */
export async function saveZcodeVersion(version: string): Promise<void> {
  if (busyScope) { repaint(); return }
  beginBusy('zcodeVersion')
  try {
    const saved = await shared().workbuddyDesktop?.saveZcodeVersion(version)
    renderZcodeVersion(saved)
    toast(version.trim() ? t('✅ ZCode 版本号已保存') : t('已清空，回落插件内置默认值'))
  } catch (error) {
    toast(t('保存失败：{message}', { message: errorMessage(error) }), 'err')
    await loadZcodeVersion() // 回滚到后端的真实值
  } finally {
    endBusy()
  }
}

/* ─── （/v1/* CORS，default）── */

export function renderCors(data?: unknown): void {
  if (data === undefined) return
  if (!data || typeof data !== 'object') {
    publish({ cors: { status: 'unavailable', on: false } })
    return
  }
  const record = data as Record<string, unknown>
  publish({ cors: { status: 'ready', on: record.corsEnabled === true } })
}

async function loadCors(): Promise<void> {
  try {
    renderCors(await shared().workbuddyDesktop?.getCors())
  } catch (error) {
    console.warn('读取网关跨域访问设置失败:', errorMessage(error))
    renderCors(null)
  }
}

export async function saveCors(next: boolean): Promise<void> {
  if (busyScope) { repaint(); return }
  beginBusy('cors')
  publish({ cors: { status: 'ready', on: next } })
  try {
    const saved = await shared().workbuddyDesktop?.saveCors(next)
    renderCors(saved)
    toast(next ? t('网关跨域访问已开启') : t('已关闭网关跨域访问'))
  } catch (error) {
    toast(t('保存失败：{message}', { message: errorMessage(error) }), 'err')
    await loadCors() // 回滚到后端的真实值
  } finally {
    endBusy()
  }
}

/* ─── （ / ALTCHA ，）── */

/** `/api/captcha` response（ and after use 1 ） */
function captchaState(state: CaptchaSetting | null | undefined): SettingsSnapshot['captcha'] {
  if (!state || typeof state !== 'object') {
    return { available: false, desktop: false, web: false, localPanel: false }
  }
  return {
    available: true,
    desktop: state.captchaDesktop === true,
    web: state.captchaWeb === true,
    localPanel: state.localPanel === true,
  }
}

async function loadCaptcha(): Promise<void> {
  try {
    const state = await shared().workbuddyDesktop?.getCaptchaSetting()
    publish({ captcha: captchaState(state) })
  } catch (error) {
    // failure use （ retention ）
    publish({ captcha: captchaState(null) })
    console.warn('读取机器人校验设置失败:', errorMessage(error))
  }
}

/** 1 （`side` ， 1 not ） */
export async function saveCaptcha(side: 'desktop' | 'web', next: boolean): Promise<void> {
  if (busyScope) { repaint(); return }
  beginBusy('captcha')
  // update： that 1 
  publish({ captcha: { ...snapshot.captcha, [side]: next } })
  const label = side === 'desktop' ? t('桌面端') : t('网页端')
  try {
    const state = await shared().workbuddyDesktop?.saveCaptchaSetting(
      side === 'desktop' ? { captchaDesktop: next } : { captchaWeb: next },
    )
    publish({ captcha: captchaState(state) })
    toast(next ? t('✅ 已开启「{label}」的机器人校验', { label }) : t('⚠️ 已关闭「{label}」的机器人校验', { label }))
  } catch (error) {
    toast(t('保存失败：{message}', { message: errorMessage(error) }), 'err')
    await loadCaptcha()
  } finally {
    endBusy()
  }
}

/* ─── （set「」）── */

async function loadAccount(): Promise<void> {
  try {
    const state = await shared().workbuddyDesktop?.getPanelAccount()
    publish({
      account: {
        status: 'ready',
        registered: state?.registered === true,
        username: typeof state?.username === 'string' ? state.username : '',
        password: snapshot.account.password,
      },
    })
  } catch (error) {
    publish({ account: { ...snapshot.account, status: 'unavailable' } })
    console.warn('读取面板账户失败:', errorMessage(error))
  }
}

/** （ by has ， this and ） */
export function openPasswordDialog(): void {
  if (!snapshot.account.registered) return
  publish({ account: { ...snapshot.account, password: { busy: false, error: '' } } })
}

/** （ in not —— request in up ， not to ） */
export function closePasswordDialog(): void {
  if (snapshot.account.password?.busy) return
  publish({ account: { ...snapshot.account, password: null } })
}

/**
 * 。 and after 1 （ new 8 、 not can and old ），
 * before 1 is as 1 ； in `panel_change_password`。
 */
export async function submitPasswordChange(oldPassword: string, newPassword: string): Promise<void> {
  const current = snapshot.account.password
  if (!current || current.busy) return
  if (!oldPassword) {
    publish({ account: { ...snapshot.account, password: { busy: false, error: t('请输入当前密码') } } })
    return
  }
  if (newPassword.length < 8) {
    publish({ account: { ...snapshot.account, password: { busy: false, error: t('新密码至少 8 位') } } })
    return
  }
  if (newPassword === oldPassword) {
    publish({ account: { ...snapshot.account, password: { busy: false, error: t('新密码不能与当前密码相同') } } })
    return
  }
  publish({ account: { ...snapshot.account, password: { busy: true, error: '' } } })
  try {
    await shared().workbuddyDesktop?.changePanelPassword(oldPassword, newPassword)
    // its will already after ， this valid —— 、 can 
    publish({ account: { ...snapshot.account, password: null } })
    toast(t('✅ 密码已修改，其他设备的登录已失效'))
  } catch (error) {
    publish({
      account: {
        ...snapshot.account,
        password: { busy: false, error: errorMessage(error) },
      },
    })
  }
}

/* ─── models.dev （ can and ）── */

/** after response（ use use ：set and new after ） */
function modelsDevState(state: ModelsDevState | null | undefined): ModelsDevPanelState {
  if (!state || typeof state !== 'object') {
    return { status: 'unavailable', enabled: true, source: '', updatedAt: 0, count: 0 }
  }
  return {
    status: 'ready',
    // when by 「」： this default（ after ）， will use as 
    // no ， 1 already 
    enabled: state.modelsDevEnabled !== false,
    source: typeof state.source === 'string' ? state.source : '',
    updatedAt: Number(state.updatedAt) || 0,
    count: Number(state.count) || 0,
  }
}

async function loadModelsDev(): Promise<void> {
  try {
    const state = await shared().workbuddyDesktop?.getModelsDev?.()
    publish({ modelsDev: modelsDevState(state) })
  } catch (error) {
    publish({ modelsDev: { ...snapshot.modelsDev, status: 'unavailable' } })
    console.warn('读取 models.dev 元数据设置失败:', errorMessage(error))
  }
}

export async function saveModelsDev(next: boolean): Promise<void> {
  if (busyScope) { repaint(); return }
  beginBusy('modelsDev')
  publish({ modelsDev: { ...snapshot.modelsDev, enabled: next } })
  try {
    const state = await shared().workbuddyDesktop?.saveModelsDev?.(next)
    publish({ modelsDev: modelsDevState(state) })
    toast(next ? t('✅ models.dev 模型元数据已开启') : t('已关闭 models.dev 联网刷新'))
  } catch (error) {
    toast(t('保存失败：{message}', { message: errorMessage(error) }), 'err')
    await loadModelsDev() // 回滚到后端的真实值
  } finally {
    endBusy()
  }
}

/**
 * 「 new 」： 1 models.dev（ not is set）。
 *
 * and new by not —— that by is **config**， this 1 is 
 * ****， by directory new that （success / no / failure）。
 */
export async function refreshModelsDev(): Promise<void> {
  if (busyScope) { repaint(); return }
  beginBusy('modelsDevRefresh')
  try {
    const payload = await shared().workbuddyDesktop?.refreshModelsDev?.()
    publish({ modelsDev: modelsDevState(payload?.status) })
    const result = payload?.result
    const status = String(result?.status || '')
    if (status === 'refreshed') {
      toast(t('models.dev 元数据已更新（{count} 条）', { count: Number(result?.count) || 0 }))
    } else if (status === 'failed') {
      toast(t('刷新失败：{message}', { message: String(result?.message || '') }), 'err')
    } else {
      // skipped： to / set —— Reason by after to ，
      toast(String(result?.message || t('本次无需刷新')))
    }
  } catch (error) {
    toast(t('刷新失败：{message}', { message: errorMessage(error) }), 'err')
    await loadModelsDev()
  } finally {
    endBusy()
  }
}

/* ─── （ + file + ） ───── */

export function renderPrompt(data?: unknown): void {
  if (data === undefined) return
  if (!data || typeof data !== 'object') {
    publish({ prompt: { ...snapshot.prompt, status: 'unavailable' } })
    return
  }
  const record = data as Record<string, unknown>
  const gateway = gatewayPrompts(record.promptGateway)
  publish({
    prompt: {
      status: 'ready',
      mode: String(record.promptMode || 'passthrough'),
      file: String(record.promptFile || ''),
      text: String(record.promptText || ''),
      source: String(record.promptSource || ''),
      lines: Number(record.promptLines) || 0,
      fileError: String(record.promptFileError || ''),
      degradeActive: record.degradeActive === true,
      degradeUntilText: String(record.degradeUntilText || ''),
      providers: providerPrompts(record.promptProviders, gateway),
      gateway,
      gatewayText: gatewayTexts(record.promptGatewayText),
      options: providerOptions(record.promptProviderOptions),
    },
  })
}

/**
 * 1 ：
 * `{"<id>": {mode, promptFile, promptText, promptSource, promptLines, promptFileError}}`。
 *
 * 1 to 「 in default」 not is undefined： for this ，
 * 1 1 ， that is use 「 this to no has 」 。
 *
 * `gateway` from ** 1 **（`promptGateway`）， = default —— in 
 * after is （ `KEY_PROMPT_GATEWAY`）， this ：
 * 「client system 」 and 「」 not 。
 */
function providerPrompts(raw: unknown, gateway: Record<string, boolean>): ProviderPromptState[] {
  if (!raw || typeof raw !== 'object') return []
  return Object.entries(raw as Record<string, unknown>)
    .filter(([id, value]) => !!id && !!value && typeof value === 'object')
    .map(([id, value]) => {
      const entry = value as Record<string, unknown>
      return {
        id,
        mode: String(entry.promptMode || 'passthrough'),
        file: String(entry.promptFile || ''),
        text: String(entry.promptText || ''),
        source: String(entry.promptSource || ''),
        lines: Number(entry.promptLines) || 0,
        fileError: String(entry.promptFileError || ''),
        gateway: gateway[id] ?? true,
        configured: true,
      }
    })
    // after already by id （BTreeMap）， this ： before 1 will 1 then 
    .sort((a, b) => a.id.localeCompare(b.id))
}

/** can config （ after to is ， not —— ） */
function providerOptions(raw: unknown): ProviderPromptOption[] {
  if (!Array.isArray(raw)) return []
  return raw
    .filter(item => !!item && typeof item === 'object' && String((item as Record<string, unknown>).id || ''))
    .map(item => {
      const entry = item as Record<string, unknown>
      return {
        id: String(entry.id),
        label: String(entry.label || entry.id),
        gatewayNote: String(entry.gatewayNote || ''),
        gatewayChars: Number(entry.gatewayChars) || 0,
        gatewayBlocks: gatewayBlocks(entry.gatewayBlocks),
      }
    })
}

/**
 * 1 ：`{"<id>": true|false}`； not 。
 *
 * ** = default**： ， by `[id] ?? true` 。
 */
function gatewayPrompts(raw: unknown): Record<string, boolean> {
  if (!raw || typeof raw !== 'object') return {}
  const flags: Record<string, boolean> = {}
  for (const [id, value] of Object.entries(raw as Record<string, unknown>)) {
    const key = String(id || '').trim()
    if (!key || typeof value !== 'boolean') continue
    flags[key] = value
  }
  return flags
}

/**
 * **** 1 ：`{"<id>": {identity?, stable?, dynamic?}}`。
 *
 * ** to **： by use （ after is this ——
 * 「 no this 1 」 is 「 this use 」）。 in this 1 ，
 * and after `GatewayBlocks::FIELDS` for 。
 */
function gatewayTexts(raw: unknown): Record<string, GatewayBlocks> {
  if (!raw || typeof raw !== 'object') return {}
  const table: Record<string, GatewayBlocks> = {}
  for (const [id, value] of Object.entries(raw as Record<string, unknown>)) {
    const key = String(id || '').trim()
    if (!key || !value || typeof value !== 'object') continue
    const entry = value as Record<string, unknown>
    const blocks: GatewayBlocks = { identity: '', stable: '', dynamic: '' }
    let any = false
    for (const field of ['identity', 'stable', 'dynamic'] as const) {
      const text = String(entry[field] || '')
      blocks[field] = text
      if (text.trim()) any = true
    }
    if (any) table[key] = blocks
  }
  return table
}

/** （config / ） 1 ； no to to null */
function gatewayBlocks(raw: unknown): GatewayBlocks | null {
  if (!raw || typeof raw !== 'object') return null
  const entry = raw as Record<string, unknown>
  const blocks: GatewayBlocks = {
    identity: String(entry.identity || ''),
    stable: String(entry.stable || ''),
    dynamic: String(entry.dynamic || ''),
  }
  if (!blocks.identity.trim() && !blocks.stable.trim() && !blocks.dynamic.trim()) return null
  return blocks
}

async function loadPrompt(): Promise<void> {
  try {
    renderPrompt(await shared().workbuddyDesktop?.getPrompt())
  } catch (error) {
    console.warn('读取系统提示词设置失败:', errorMessage(error))
    renderPrompt(null)
  }
}

/**
 * 1 （ / file / ）。 and old 1 ** has **： that 1 。
 * old DOM up （ that is 1 can to 「current」 ）；React this 
 * in ， 1 ， not in 「」，
 * after （ ）。
 *
 * return = this is success：（ that ） not ——
 * failure when in ， use not new 1 。
 */
async function savePromptField(label: string, patch: PromptPatch): Promise<boolean> {
  if (busyScope) {
    await loadPrompt() // 有别的操作在跑：把界面拉回后端真实值，别让用户以为改了
    return false
  }
  beginBusy('prompt')
  try {
    const saved = await shared().workbuddyDesktop?.savePrompt(patch)
    renderPrompt(saved)
    toast(t('✅ 已保存：{label}', { label }))
    return true
  } catch (error) {
    toast(t('保存失败：{message}', { message: errorMessage(error) }), 'err')
    await loadPrompt() // 回滚到后端的真实值
    return false
  } finally {
    endBusy()
  }
}

export async function savePromptMode(mode: string): Promise<void> {
  const option = PROMPT_MODES.find(item => item.value === mode)
  await savePromptField(t('模式改为「{mode}」', { mode: option?.toastLabel ?? mode }), { promptMode: mode })
}

export async function savePromptFile(raw: string): Promise<void> {
  await savePromptField(
    raw.trim() ? t('提示词文件已更新') : t('已改回内置默认提示词'),
    { promptFile: raw },
  )
}

/**
 * ****（ that 1 ）。empty = this 1 、
 * file / default（`savePrompt` empty）。
 *
 * successreturn true， is 。
 */
export async function savePromptText(text: string): Promise<boolean> {
  const empty = !text.trim()
  return savePromptField(
    empty ? t('已清掉界面编辑的提示词（回落文件 / 内置默认）') : t('提示词正文已更新'),
    { promptText: empty ? '' : text },
  )
}

/* ─── ： by ─────────────────────── */

/** this 1 in up （ not to id —— and after 1 to ） */
export function promptProviderLabel(prompt: PromptState, id: string): string {
  return prompt.options.find(item => item.id === id)?.label || id
}

/**
 * 1 id is ** in **， not returnempty（ use not ）。
 *
 * 。 1 in （ down `onValueChange` null）：Base UI
 * Select in 「empty / 」 when `null`， `String(null)` is **
 * "null"** —— that will 1 `null` to after ， after 1 
 * 「unknown id：null」， use to is 1 its failure。
 * this 1 then ： not id from （ old response、 、 after add use ），
 * ** not ** —— and after `is_known_provider_id` 1 。
 */
function knownProviderId(id: unknown): string {
  const text = id == null ? '' : String(id).trim()
  if (!text) return ''
  return snapshot.prompt.options.some(item => item.id === text) ? text : ''
}

/** 1 （`patch` to by after ） */
async function saveProviderPrompt(
  label: string,
  id: string,
  patch: { promptMode?: string; promptFile?: string; promptText?: string } | null,
): Promise<boolean> {
  return savePromptField(label, { promptProviders: { [id]: patch } })
}

/**
 * 「 this 1 up that 」→ this 1 （****）。
 *
 * and file 1 ： this 1 can can 「」（ up is that ），
 * 1 when ， 1 if not to 「default」—— 1 ， after ， not 
 * （`saveProviderPromptMode` as this this ）。
 *
 * already has （ down ）； when 。
 * file and default not config，otherwise after file will this 。
 */
function providerPatch(
  prompt: PromptState,
  item: ProviderPromptState,
): { promptMode: string; promptFile: string; promptText: string } {
  const followGlobal = !item.configured
  return {
    promptMode: item.mode,
    promptFile: item.file,
    promptText: item.source === 'inline' || item.mode === 'passthrough' ? item.text
      : followGlobal && (prompt.source === 'inline' || prompt.mode === 'passthrough') ? prompt.text : '',
  }
}

/** to this 1 1 （ = that ： up current ） */
export async function addProviderPrompt(rawId: string): Promise<void> {
  const id = knownProviderId(rawId)
  if (!id) return
  const prompt = snapshot.prompt
  await saveProviderPrompt(
    t('已为「{label}」单独配置提示词', { label: promptProviderLabel(prompt, id) }),
    id,
    providerPatch(prompt, {
      id,
      mode: prompt.mode,
      file: prompt.file,
      text: prompt.text,
      source: prompt.source,
      lines: prompt.lines,
      fileError: prompt.fileError,
      gateway: prompt.gateway[id] ?? true,
      configured: false,
    }),
  )
}

export async function saveProviderPromptMode(item: ProviderPromptState, mode: string): Promise<void> {
  const id = knownProviderId(item.id)
  if (!id || !PROMPT_MODES.some(option => option.value === mode)) return
  const option = PROMPT_MODES.find(candidate => candidate.value === mode)
  await saveProviderPrompt(
    t('「{label}」的模式改为「{mode}」', {
      label: promptProviderLabel(snapshot.prompt, id),
      mode: option?.toastLabel ?? mode,
    }),
    id,
    { ...providerPatch(snapshot.prompt, item), promptMode: mode },
  )
}

export async function saveProviderPromptFile(item: ProviderPromptState, raw: string): Promise<void> {
  const id = knownProviderId(item.id)
  if (!id) return
  await saveProviderPrompt(
    raw.trim()
      ? t('「{label}」的提示词文件已更新', { label: promptProviderLabel(snapshot.prompt, id) })
      : t('「{label}」改用内置默认提示词', { label: promptProviderLabel(snapshot.prompt, id) }),
    id,
    // up ： and file 1 current ，「file 」
    { ...providerPatch(snapshot.prompt, item), promptFile: raw },
  )
}

/**
 * ** 1 ** （ that 1 ）。empty = this 1 、
 * file / default。
 *
 * and up 1 「」： 1 this / file / 
 * 1 ， when 。
 */
export async function saveProviderPromptText(
  item: ProviderPromptState,
  text: string,
): Promise<boolean> {
  const id = knownProviderId(item.id)
  if (!id) return false
  const empty = !text.trim()
  const label = promptProviderLabel(snapshot.prompt, id)
  return saveProviderPrompt(
    empty ? t('已清掉「{label}」界面编辑的正文', { label }) : t('「{label}」的提示词正文已更新', { label }),
    id,
    { ...providerPatch(snapshot.prompt, item), promptText: empty ? '' : text },
  )
}

/** this 1 （set； null is after 「delete this 1 」） */
export async function removeProviderPrompt(rawId: string): Promise<void> {
  const id = knownProviderId(rawId)
  if (!id) return
  await saveProviderPrompt(
    t('已取消「{label}」的单独配置', { label: promptProviderLabel(snapshot.prompt, id) }),
    id,
    null,
  )
}

/**
 * **** （ this 1 need not need that ）。
 *
 * and / file** 1 **（`promptGateway`）， by after 
 * `KEY_PROMPT_GATEWAY`： 1 down not this this 。
 *
 * is 「 use up need 」 ： in when after ， when 
 * is in —— for 1 will 。
 */
export async function saveProviderGatewayPrompt(rawId: string, enabled: boolean): Promise<void> {
  const id = knownProviderId(rawId)
  if (!id) return
  const label = promptProviderLabel(snapshot.prompt, id)
  if (busyScope) {
    await loadPrompt()
    return
  }
  beginBusy('prompt')
  try {
    const saved = await shared().workbuddyDesktop?.savePrompt({ promptGateway: { [id]: enabled } })
    renderPrompt(saved)
    toast(
      enabled
        ? t('✅ 已为「{label}」装上网关自带提示词', { label })
        : t('已关闭「{label}」的网关自带提示词：能否通过上游校验取决于上游当前口径', { label }),
      enabled ? 'ok' : 'err',
    )
  } catch (error) {
    toast(t('保存失败：{message}', { message: errorMessage(error) }), 'err')
    await loadPrompt() // 回滚到后端的真实值
  } finally {
    endBusy()
  }
}

/**
 * ** 1 ** ****（ 1 ；`null` = to ）。
 *
 * and （`saveProviderGatewayPrompt`）** 1 **（`promptGatewayText`）：
 * not this —— use in that 、 to when ，
 * not this 「 not 」 1 。
 *
 * 1 is has ： is 1 （ up is 「」 this ），
 * is and ， when by up to that down 。
 * successreturn true， is 。
 */
export async function saveProviderGatewayText(
  rawId: string,
  blocks: GatewayBlocks | null,
): Promise<boolean> {
  const id = knownProviderId(rawId)
  if (!id) return false
  const label = promptProviderLabel(snapshot.prompt, id)
  // empty = no has （ after is this ：empty not 、empty this ）。
  // 1 `null` is as ： use empty， to is 「 already 」。
  const emptied = !blocks
    || !(blocks.identity.trim() || blocks.stable.trim() || blocks.dynamic.trim())
  return savePromptField(
    emptied
      ? t('「{label}」的网关自带提示词已改回官方原文', { label })
      : t('「{label}」的网关自带提示词正文已更新', { label }),
    { promptGatewayText: { [id]: emptied ? null : blocks } },
  )
}

/** （ after ；config 1 not ） */
export async function clearDegrade(): Promise<void> {
  if (busyScope) return
  beginBusy('prompt')
  try {
    const saved = await shared().workbuddyDesktop?.savePrompt({ clearDegrade: true })
    renderPrompt(saved)
    toast(t('✅ 已解除内容拦截降级'))
  } catch (error) {
    toast(t('解除失败：{message}', { message: errorMessage(error) }), 'err')
    await loadPrompt()
  } finally {
    endBusy()
  }
}

/* ─── （） ─────────────────── */

/** 1 ： has 1 null（「—」） */
function finiteOrNull(value: unknown): number | null {
  if (value === null || value === undefined || value === '') return null
  const num = Number(value)
  return Number.isFinite(num) ? num : null
}

/**
 * 。 is after ****：
 * `{ configDir, database: { file, bytes, available, accounts, logs, requests, dailyDays, downstreamRaw, upstreamRaw } }`。
 * after not （ / failure） and not is ， for this 1 ： not to 
 * 「unavailable」 not is 1 0。
 */
export function renderStorage(data?: unknown): void {
  if (data === undefined) return
  const info = data && typeof data === 'object'
    ? (data as Record<string, unknown>).database
    : null
  if (!info || typeof info !== 'object') {
    publish({ storage: { ...snapshot.storage, status: 'unavailable' } })
    return
  }
  const record = info as Record<string, unknown>
  publish({
    storage: {
      status: 'ready',
      available: record.available !== false,
      file: String(record.file || ''),
      bytes: Number(record.bytes) || 0,
      accounts: finiteOrNull(record.accounts),
      logs: finiteOrNull(record.logs),
      requests: finiteOrNull(record.requests),
      dailyDays: finiteOrNull(record.dailyDays),
      downstreamRaw: finiteOrNull(record.downstreamRaw),
      upstreamRaw: finiteOrNull(record.upstreamRaw ?? record.debug),
    },
  })
}

async function loadStorage(): Promise<void> {
  try {
    renderStorage(await shared().workbuddyDesktop?.getStorage())
  } catch (error) {
    console.warn('读取数据存储概况失败:', errorMessage(error))
    renderStorage(null)
  }
}

/* ─── （only） ─────────────────── */

/**
 * 「」 will （30 1 and ）， its already not ；
 * success after 。return false failure（ by need ）。
 */
export async function panelLogout(): Promise<boolean> {
  try {
    await shared().workbuddyDesktop?.panelLogout()
    // 1 （panel_session），must 1 and ——
    // server will not ， down will ，
    // as 「 need 1 」。
    await shared().workbuddyDesktop?.panelSessionClear?.()
    // tauri://，path `/login` not in （ that is headless
    // path）。 。
    const desktop = shared().workbuddyDesktop?.platform !== 'web'
    window.location.href = desktop ? 'login.html' : '/login'
    return true
  } catch (error) {
    toast(t('退出失败：{message}', { message: errorMessage(error) }), 'err')
    return false
  }
}

/* ─── ─────────────────────────────── */

/**
 * set（app.js this when use ，upgrade-panel after ）。
 * and ，failure —— 1 not this empty。
 */
export async function load(): Promise<void> {
  restoreCategory()
  renderUnits()
  publish({ panelLogin: readPanelLogin() })
  window.dispatchEvent(new Event('wb:settings-load'))
  await Promise.all([
    loadSettings(),
    loadLanExtras(),
    loadRetention(),
    loadRetry(),
    loadTimeouts(),
    loadQueue(),
    loadImageCompress(),
    loadCaptchaPool(),
    loadDebug(),
    loadSanitize(),
    loadDeveloperRoleCompat(),
    loadClineHeaders(),
    loadZcodeVersion(),
    loadCors(),
    loadPrompt(),
    loadStorage(),
    loadCaptcha(),
    loadAccount(),
    loadModelsDev(),
    reconnectDbRestore(),
    // update is 1 （update-panel.tsx），set when new 1 
    shared().wbUpdatePanel?.load?.(),
    // 「」 is （system-info-pane.tsx）： in set when ——
    // / when / in request this will when ， 1 old no has 
    shared().wbSystemInfoPane?.load?.(),
  ])
}

/* ─── new by （ toast old ） ─────── */

export async function refreshRetention(): Promise<void> {
  await loadRetention()
  toast(t('数据保留设置已刷新'))
}

export async function refreshRetry(): Promise<void> {
  await loadRetry()
  toast(t('重试设置已刷新'))
}

export async function refreshTimeouts(): Promise<void> {
  await loadTimeouts()
  toast(t('超时设置已刷新'))
}

export async function refreshQueue(): Promise<void> {
  await loadQueue()
  toast(t('排队等待设置已刷新'))
}

export async function refreshImageCompress(): Promise<void> {
  await loadImageCompress()
  toast(t('会话图片压缩设置已刷新'))
}

export async function refreshCaptchaPool(): Promise<void> {
  await loadCaptchaPool()
  toast(t('验证码令牌池设置已刷新'))
}

export async function refreshRouting(): Promise<void> {
  await loadRouting()
  toast(t('路由设置已刷新'))
}

export async function refreshDebug(): Promise<void> {
  await loadDebug()
  toast(t('调试模式设置已刷新'))
}

export async function refreshSanitize(): Promise<void> {
  await loadSanitize()
  toast(t('指纹脱敏设置已刷新'))
}

export async function refreshDeveloperRoleCompat(): Promise<void> {
  await loadDeveloperRoleCompat()
  toast(t('开发者角色兼容设置已刷新'))
}

export async function refreshCors(): Promise<void> {
  await loadCors()
  toast(t('网关跨域访问设置已刷新'))
}

export async function refreshPrompt(): Promise<void> {
  await loadPrompt()
  toast(t('系统提示词设置已刷新'))
}

export async function refreshStorage(): Promise<void> {
  await loadStorage()
  toast(t('存储概况已刷新'))
}
