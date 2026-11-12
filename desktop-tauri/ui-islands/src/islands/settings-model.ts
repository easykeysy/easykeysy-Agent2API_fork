/**
 * Agent2API · set ****（ / / / ）。
 *
 * from settings-page.tsx ： that 1 「 + + 1 」 
 * already file， this 1 —— no has JSX、 no has 。 to 
 * （settings-state.ts and settings-page.tsx import ， not ）。
 *
 * not is ：file is .ts， not will src/index.tsx `islands/*.tsx` glob ；set 
 * has settings-page.tsx 1 （ and file）。
 *
 * this is 「 1 」 ：
 * · （ / / when / ）—— must and after config.rs 1 ，
 * to 1 （ old this for in settings-panel.js ）；
 * · —— from index.html （ / / ），
 * delete， this is up 1 ；
 * · （ / 1 / ）—— and need use 。
 */

import type { SanitizePatch } from './settings-sanitize-model'
import { t } from '../i18n'


// ── from HEAD （ return）──────────────

/** （`/api/captcha`）： 1 + */
export type CaptchaSetting = {
  captchaDesktop?: boolean
  captchaWeb?: boolean
  /** true = is ；false = headless（Docker / server） */
  localPanel?: boolean
}

/** ： optional， that 1 not */
export type CaptchaPatch = {
  captchaDesktop?: boolean
  captchaWeb?: boolean
}

/** （`/api/panel/account`） */
export type PanelAccount = {
  registered?: boolean
  username?: string
  loginEnabled?: boolean
}

export type DatabaseTableStat = { table?: string; rows?: number }

/** （ export_database） */
export type DatabaseExportResult = {
  canceled?: boolean
  bytes?: number
  file?: string
  tables?: DatabaseTableStat[]
  encrypted?: boolean
  includedKeys?: string[]
}

export type DatabaseImportResult = {
  /** use in file（ / file can can is this ） */
  canceled?: boolean
  /** after */
  tables?: DatabaseTableStat[]
  /** before file（{config_dir}/backups/ down ） */
  safetyBackup?: string
  /**
   * not （** in has **， by after `db::backup` 
   * `account_key_warning` and to ）。
   *
   * ** not is failure**：， this is use 1 （ or when 
   * no ）， after not 、 will is empty 。**must**
   * （ ）—— seconds toast etc no ，
   * use after will for empty。
   */
  warning?: string
  encrypted?: boolean
  restoredKeys?: string[]
  reencryptedSecrets?: number
  localKeysPreserved?: boolean
}

/**
 * （`GET /api/backup/progress`， `db::backup` `RestoreProgress`）。
 *
 * by after to ：`phase` is 「 this 1 in 」 can ，`percent` is 
 * 。 before not —— when no has can （`execute_batch`
 * not ）， 1 not 1 。
 */
export type RestoreProgressSnapshot = {
  /** in */
  active?: boolean
  /** current */
  phase?: string
  /** （0-100） */
  percent?: number
  done?: boolean
  failed?: boolean
  /** failureReason（`failed` when has ） */
  error?: string | null
  /** （`done` when has ， DatabaseImportResult） */
  result?: unknown
  startedAt?: number
  finishedAt?: number
}

export type ModelsDevRefreshResult = {
  status?: string
  message?: string
  count?: number
  refreshedAt?: number
}

/** need that （ this import ） */
type ModelsDevView = {
  status: 'loading' | 'ready' | 'unavailable'
  enabled: boolean
  source: string
  updatedAt: number
  count: number
}

/**
 * models.dev ： 「 + + when + 」 1 。
 *
 * Why not 「 already 」： this in ** that is when 、 has 
 * ** —— ， use no 「 Why is 」 is no 、
 * is no to 、 is this old 。
 */
export function modelsDevStateText(state: ModelsDevView): string {
  if (state.status === 'loading') return STATES.appLoading
  if (state.status === 'unavailable') return STATES.modelsDevUnavailable
  const origin = state.source === 'builtin'
    ? STATES.modelsDevSourceBuiltin
    : state.source === 'cache'
      ? STATES.modelsDevSourceCache
      : state.source === 'network'
        ? STATES.modelsDevSourceNetwork
        // after will when ， not empty
        : state.source
  const stamp = state.updatedAt > 0
    // and when （app.js `formatTime`； from ）
    ? String(shared().wbApp?.formatTime?.(state.updatedAt) ?? '')
    : ''
  if (!origin && !stamp && !state.count) return STATES.modelsDevEmpty
  const at = stamp ? t('，{time}', { time: stamp }) : ''
  const mode = state.enabled ? t('正在联网更新') : t('仅用现有数据、不联网刷新')
  return t('{count} 条 · 来自{origin}{at} · {mode}', {
    count: Number(state.count).toLocaleString('zh-CN'),
    origin: origin || t('未知来源'),
    at,
    mode,
  })
}

/* ─── and ─────────────────────────── */

/** and set（ get_app_settings / save_app_settings） */
export type AppSettings = {
  closeToTray: boolean
  autostart: boolean
  /** ： 0.0.0.0（「 use 」， TIPS.lan） */
  lanAccess: boolean
  /** when is when （） */
  lanPanel: boolean
  /** ：（ WebView2 ）， by need */
  lightweightMode: boolean
}

/**
 * 「」 （`GET /api/system-info`，）。
 *
 * can can is null / （ after by ， `api::system_info`）：
 * 1 by 「—」， not 。`deployment` is after 
 * （desktop / windows-service / docker / systemd / foreground），
 * `runtime.webviewVersion` is WebView2 / WKWebView、 is 。
 */
export type SystemInfo = {
  app?: {
    version?: string | null
    license?: string | null
    sourceUrl?: string | null
    licenseUrl?: string | null
    copyright?: string | null
    channel?: string | null
    schemaVersion?: number | null
    dbSchemaVersion?: number | null
  }
  runtime?: {
    deployment?: string | null
    container?: boolean | null
    os?: string | null
    osVersion?: string | null
    arch?: string | null
    webviewVersion?: string | null
    startedAt?: number | null
    uptimeSeconds?: number | null
    pid?: number | null
    debugBuild?: boolean | null
    verbose?: boolean | null
  }
  deployment?: {
    host?: string | null
    port?: number | null
    panelPort?: number | null
    uiHosted?: boolean | null
    localPanel?: boolean | null
    configDir?: string | null
    dbFile?: string | null
    dbBytes?: number | null
    dbAvailable?: boolean | null
    logFile?: string | null
    exePath?: string | null
    installerKind?: string | null
    selfUpdate?: boolean | null
    updateCapability?: import('../../../ui-shared/updates').UpdateCapability
  }
  security?: {
    adminRegistered?: boolean | null
    panelAuth?: boolean | null
    apiKeyCount?: number | null
    allowNoKey?: boolean | null
    v1FailClosed?: boolean | null
    panelGate?: boolean | null
    githubTokenFilled?: boolean | null
    githubTokenOrigin?: string | null
    updateProxy?: boolean | null
  }
  features?: {
    debugMode?: boolean | null
    sanitizeFingerprints?: boolean | null
    cors?: boolean | null
    developerRoleCompat?: boolean | null
    keepaliveStrip?: boolean | null
    autoContinue?: boolean | null
    autoContinueMaxRounds?: number | null
    modelsDev?: boolean | null
    captchaEnabled?: boolean | null
    updateBeta?: boolean | null
  }
  stats?: {
    accounts?: number | null
    enabledAccounts?: number | null
    limitedAccounts?: number | null
    models?: number | null
    inFlight?: number | null
    requests?: number | null
    logs?: number | null
    upgradePending?: boolean | null
  }
}

/** failure：customProvider that not is ， is */
export type ImportError = { id?: string; message?: string; customProvider?: boolean }

/** （ import_accounts） */
export type ImportResult = {
  canceled?: boolean
  total?: number
  added?: number
  updated?: number
  skipped?: number
  failed?: number
  errors?: ImportError[]
  /** v2 file statistics */
  customProviders?: { added?: number; updated?: number }
}

/** （ export_accounts） */
export type ExportResult = {
  canceled?: boolean
  count?: number
  /** v2 file */
  customProviders?: number
  file?: string
}

/**
 * ****（ / / ）。
 *
 * is up for **** need （： and 1 → 405/3012，
 * after `zcode::OFFICIAL_PROMPT_NOTE`）， and config by ， not 
 * 「 1 」 。 （`{cwd}` etc ），
 * request when 。
 */
export type GatewayBlocks = { identity: string; stable: string; dynamic: string }

/** ： that 1 （ after ），clearDegrade is 1 up */
export type PromptPatch = {
  promptMode?: string
  promptFile?: string
  /**
   * （**** `promptFile` and default）。empty = 
   * this 1 、file / default； = this 1 not 。
   */
  promptText?: string
  /**
   * by ：`{ "<providerId>": {promptMode?, promptFile?, promptText?} | null }`
   * （ and that 、 and response 1 ， has 1 ）。
   * is `null` = this 1 （set）； = this 1 not 。
   */
  promptProviders?: Record<
    string,
    { promptMode?: string; promptFile?: string; promptText?: string } | null
  >
  /**
   * ：`{ "<providerId>": true | false | null }`。
   * `null` = to default（）； = this 1 not （ and up that 1 ）。
   */
  promptGateway?: Record<string, boolean | null>
  /**
   * ****：`{ "<providerId>": {identity?, stable?, dynamic?} | null }`。
   * is **update**（ ，empty = this 1 to ）； `null` =
   * this 1 to 。
   */
  promptGatewayText?: Record<string, Partial<GatewayBlocks> | null>
  clearDegrade?: boolean
}

/** Cline response（GET and PUT ， and after api/cline_headers.rs 1 ） */
export type ClineHeadersData = {
  /** use （；empty = this not ） */
  overrides: Record<string, string>
  /** default（ that 1 ；「default」） */
  defaults: Record<string, string>
  /** default ∪ and after （ will that ） */
  effective: Record<string, string>
}

/** use to / HTTP （ and bridge.rs 1 1 for ， 1 not can ） */
export type SettingsBridge = {
  /** 'desktop' | 'web'： that 1 in has */
  platform?: string
  getAppSettings(): Promise<AppSettings | null | undefined>
  saveAppSettings(patch: AppSettings): Promise<AppSettings | null | undefined>
  exportAccounts(): Promise<ExportResult | null | undefined>
  importAccounts(): Promise<ImportResult | null | undefined>
  getRetention(): Promise<unknown>
  saveRetention(patch: Record<string, number>): Promise<unknown>
  getRetry(): Promise<unknown>
  saveRetry(patch: Record<string, unknown>): Promise<unknown>
  getTimeouts(): Promise<unknown>
  saveTimeouts(patch: Record<string, number>): Promise<unknown>
  getQueue(): Promise<unknown>
  saveQueue(patch: Record<string, number>): Promise<unknown>
  getImageCompress(): Promise<unknown>
  saveImageCompress(patch: Record<string, number | boolean>): Promise<unknown>
  getDebug(): Promise<unknown>
  saveDebug(enabled: boolean): Promise<unknown>
  getSanitize(): Promise<unknown>
  saveSanitize(patch: boolean | SanitizePatch): Promise<unknown>
  getDeveloperRoleCompat(): Promise<unknown>
  saveDeveloperRoleCompat(enabled: boolean): Promise<unknown>
  getClineHeaders(): Promise<unknown>
  saveClineHeaders(overrides: Record<string, string>): Promise<unknown>
  getZcodeVersion(): Promise<unknown>
  saveZcodeVersion(version: string): Promise<unknown>
  getCors(): Promise<unknown>
  saveCors(enabled: boolean): Promise<unknown>
  getPrompt(): Promise<unknown>
  savePrompt(payload: PromptPatch): Promise<unknown>
  getStorage(): Promise<unknown>
  getCaptchaSetting(): Promise<CaptchaSetting | null | undefined>
  saveCaptchaSetting(patch: CaptchaPatch): Promise<CaptchaSetting | null | undefined>
  /** （set「」）； when registered=false */
  getPanelAccount(): Promise<PanelAccount | null | undefined>
  /** ： need old ；success after after its will ， will */
  changePanelPassword(oldPassword: string, newPassword: string): Promise<unknown>
  panelLogout(): Promise<unknown>
  /** will （ ）； by shim cookie */
  panelSessionStatus?(): Promise<{ registered?: boolean; authenticated?: boolean } | null | undefined>
  /** will （ when and panelLogout ） */
  panelSessionClear?(): Promise<unknown>
  /** after （ need ； its not ） */
  getBackendStatus?(): Promise<{ port?: number } | null | undefined>
  onBackendReady?(callback: () => void): (() => void) | void
  // ── （only，web shim not this ）──
  /** is already （ need not need ） */
  panelAdminStatus?(): Promise<{ registered?: boolean } | null | undefined>
  /** （path， not HTTP ） */
  panelRegister?(username: string, password: string): Promise<{ registered?: boolean; existed?: boolean }>
  /** （set and use ）；return createdKey empty = already Key */
  changeLanAccess?(enabled: boolean, panel: boolean): Promise<unknown>
  /** in （ not to as null） */
  localIp?(): Promise<string | null | undefined>
  /** 1 （ http(s)）：， shim is window.open */
  openReleasePage?(url: string): Promise<unknown>
  /**
   * set「」 1 （ / shim ）。
   *
   * `hideUsername` is ****parameter： in path in directory before `~`，
   * in after —— this 「」 to is after 。
   */
  getSystemInfo?(options?: { hideUsername?: boolean }): Promise<SystemInfo | null | undefined>
  // ── down from HEAD （ and settings-state.ts use ）──
    exportDatabase?(includeLogs: boolean, password: string): Promise<DatabaseExportResult | null | undefined>
    importDatabase?(password?: string): Promise<DatabaseImportResult | null | undefined>
    /** （`GET /api/backup/progress`）： 500ms 1 */
    getBackupProgress?(): Promise<RestoreProgressSnapshot | null | undefined>
    getCaptchaPool(): Promise<unknown>
    saveCaptchaPool(patch: Record<string, number>): Promise<unknown>
    getRoutingConfig(): Promise<unknown>
    setRoutingConfig(patch: Record<string, unknown>): Promise<unknown>
    getModelsDev(): Promise<ModelsDevState | null | undefined>
    saveModelsDev(on: boolean): Promise<ModelsDevState | null | undefined>
    refreshModelsDev(): Promise<{ result?: ModelsDevRefreshResult; status?: ModelsDevState } | null | undefined>
    // ── （set「 → 」， settings-cloud-sync.tsx）──
    // optional： shim and ，（ before ） no has this ——
    // 「unavailable」 not is 。
    getCloudSync?(): Promise<unknown>
    saveCloudSync?(payload: Record<string, unknown>): Promise<unknown>
    testCloudSync?(payload: Record<string, unknown>): Promise<unknown>
    getCloudSyncRemote?(): Promise<unknown>
    cloudSyncUpload?(): Promise<unknown>
    cloudSyncDownload?(): Promise<unknown>
    importCloudSyncKey?(password: string): Promise<unknown>
}

/**
 * window up by its / its 。
 *
 * use 「 + 」 not is declare global Window up ：
 * workbuddyDesktop / wbApp / wbUnits is use ， declare 1 will 
 * not 1 TS2717（ and when ）。 this use to ；
 * set wbSettingsPanel by settings-page.tsx declare。
 */
export type SharedWindow = {
  workbuddyDesktop?: SettingsBridge
  wbApp?: {
    toast?: (message: string, kind?: 'err' | 'ok') => void
    /** after （、） */
    refresh?: () => Promise<void> | void
    /** current： when if already in set， 1 load() */
    readonly currentPage?: string
    /** use （system / light / dark）： and in app.js， use */
    applyTheme?: (mode: string) => void
    /** use （， 105）： by app.js to WebView and */
    applyZoom?: (percent: number) => void
    /** when → when （app.js ；models.dev 「 is when 」 use ） */
    formatTime?: (value: unknown) => string
  }
  /** Token （units.js）：， in that */
  wbUnits?: { isChinese?: () => boolean; setChinese?: (on: boolean) => void }
  /** update （update-panel.tsx）： to set when new 1 */
  wbUpdatePanel?: { load?: () => Promise<void> | void }
  /** （system-info-pane.tsx）： up */
  wbSystemInfoPane?: { load?: () => Promise<void> | void }
  /** （icons.js）： by （return SVG ， use ） */
  wbIcons?: { icon?: (name: string, size?: number) => string }
}

export function shared(): SharedWindow {
  return window as unknown as SharedWindow
}

export function errorMessage(error: unknown): string {
  const message = error instanceof Error ? error.message : String(error ?? '')
  return message.trim() || t('未知错误')
}

/** toast 1 （ wbApp， not in ） */
export function toast(message: string, kind?: 'err' | 'ok'): void {
  shared().wbApp?.toast?.(message, kind)
}

/**
 * ： 1 to default（ `open_release_page`，
 * shim window.open）—— and update-shared is ****，
 * not import is ： this set（ and need ）， that update 1 ，
 * that ；， need use from this 。
 */
export async function openExternal(url: string): Promise<void> {
  try {
    await shared().workbuddyDesktop?.openReleasePage?.(url)
  } catch (error) {
    toast(t('打开链接失败：{error}', { error: errorMessage(error) }), 'err')
  }
}

/* ─── and ─────────────────────────── */

/**
 * 。 = ；showCategory use （ old is DOM，
 * this this —— not by HTML ， is ）。
 *
 * 「」「 when 」 from 「」（ in 1 ， in etc 
 * ）： this is ** will ** as parameter， when ， 1 
 * 1 。 not （settings-state load 1 and ），
 * is 。
 *
 * 「」「 use 」： / / is ** before **（ in 
 * localStorage ， and ）， not after config —— in before that 。
 *
 * 「 and need 」 is （ GitHub issue ， settings-page 
 * FeedbackPane）， in 「update」 up —— is 「 for 」 ，。
 *
 * `icon` is icons.js that set （， that ）；
 * not limit， in 1 ，。
 *
 * 「update」（「」）**id `about` not **： when is update-panel.tsx 
 * （`.settings-pane[data-cat="about"]`） and use localStorage ，
 * will —— use to is ，id is 。
 */
export const CATEGORIES = [
  { id: 'account', label: t('账户'), icon: 'accounts' },
  { id: 'general', label: t('通用'), icon: 'sliders' },
  // ：AI config（ for / / ） already to 
  // 「set」（ assistant-settings-drawer.tsx）—— that is 
  // ， in this need for 。 not ，
  // use localStorage 'assistant' by settings-state to 。
  { id: 'display', label: t('显示'), icon: 'display' },
  { id: 'gateway', label: t('网关'), icon: 'traffic' },
  { id: 'retry', label: t('重试'), icon: 'refresh' },
  { id: 'timeout', label: t('超时'), icon: 'timer' },
  { id: 'security', label: t('安全'), icon: 'shield' },
  { id: 'data', label: t('数据'), icon: 'database' },
  { id: 'feedback', label: t('反馈与需求'), icon: 'feedback' },
  // 「」： / / path / / ， can 1 
  // （ in system-info-pane.tsx）。「 and need 」—— 
  // is 1 ： this ， that issue。
  // `system` is new id， not and has / localStorage 。
  { id: 'system', label: t('版本信息'), icon: 'info' },
  { id: 'about', label: t('更新'), icon: 'download' },
] as const

/**
 * 「setcurrent」 localStorage ： before （ and 、），
 * not and 。 use old ， —— otherwise use up will 。
 */
export const SETTINGS_CAT_KEY = 'workbuddy-desktop-settings-cat'

/* ─── （「」） ───────────────── */

/**
 * 「」 ： / / 。
 *
 * ⚠ before ** use not in **， in ui/app.js（applyTheme / applyZoom）：
 * · need ， and 「」 when will 
 * WebView （ that in app.js）；
 * · need WebView （ set_zoom）， not is can 。
 * ：**** localStorage to current up ； when 
 * `wbApp.applyTheme / applyZoom`， 'wb:theme' / 'wb:zoom' ——
 * and this is 1 set ，must。
 *
 * and app.js is 1 （ 1 ， 1 ）；
 * （app.js ，）。
 */
export const THEME_KEY = 'workbuddy-desktop-theme'
export const ZOOM_KEY = 'workbuddy-desktop-zoom'
export const THEME_EVENT = 'wb:theme'
export const ZOOM_EVENT = 'wb:zoom'

/** and ： and app.js ZOOM_MIN / ZOOM_MAX / ZOOM_STEP */
export const ZOOM_MIN = 80
export const ZOOM_MAX = 130
export const ZOOM_STEP = 5

/** （80% ~ 130%，5% 1 11 ）： down value is */
export const ZOOM_PERCENTS: number[] = Array.from(
  { length: (ZOOM_MAX - ZOOM_MIN) / ZOOM_STEP + 1 },
  (_, index) => ZOOM_MIN + index * ZOOM_STEP,
)

/**
 * ：value and app.js / `data-theme` 1 （system / light / dark）。
 * that by title（ → ， is 1 ，
 * this use 1 ）。
 */
export const THEME_MODES = [
  { value: 'system', label: t('跟随系统') },
  { value: 'light', label: t('浅色') },
  { value: 'dark', label: t('深色') },
] as const

/** （SegmentedControl parameter need use ， in ） */
export type ThemeMode = (typeof THEME_MODES)[number]['value']

/**
 * already to when ： optional by ui/i18n.js `wbI18n.LOCALES`
 * （label is ）， workbuddy-desktop-locale。
 * this not LANGUAGES —— ， this has ui/i18n.js 1 。
 */

/** current（app.js is ）： to by */
export function readThemeMode(): ThemeMode {
  try {
    const mode = localStorage.getItem(THEME_KEY)
    return mode === 'light' || mode === 'dark' ? mode : 'system'
  } catch {
    return 'system'
  }
}

/** current（）： / 100， and app.js storedZoom 1 */
export function readZoomPercent(): number {
  try {
    const text = localStorage.getItem(ZOOM_KEY)
    // empty / need ：Number('') and Number(null) is 0（ not is NaN），
    // not will 「0%」 1 to 80% —— 「 no 」must etc default 100%
    if (text === null || text.trim() === '') return 100
    const raw = Number(text)
    if (!Number.isFinite(raw)) return 100
    const snapped = Math.round(raw / ZOOM_STEP) * ZOOM_STEP
    return Math.min(ZOOM_MAX, Math.max(ZOOM_MIN, snapped))
  } catch {
    return 100
  }
}

/* ─── ───────────────────────────── */

export type NumberField = {
  /** after config JSON （must 1 ，otherwise PUT ignore） */
  key: string
  /** id： old id （ no has by id this ） */
  id: string
  label: string
  /** */
  unit: string
  min: number
  max: number
  /** down （ index.html ） */
  hint: string
}

export { RETENTION_FIELDS, RETENTION_MIN, RETENTION_MAX } from './settings-retention-model'

/**
 * 。 `retryCrossProviderCount` is ** old **（config，
 * will config not to 、default）， after already KEY_RETRY_ACCOUNT_SWITCH_COUNT
 * —— this must use old ， has 。
 */
export const RETRY_FIELDS: NumberField[] = [
  {
    key: 'retryCount',
    id: 'settings-retry-count',
    label: t('同一账号重试次数'),
    unit: t('次'),
    min: 0,
    max: 10,
    hint: t('在同一个账号上原地重发几次（不含首发；整条请求共用一份，用完后换来的账号只发一次就继续换）。可填 0–10，0 表示失败立即换号。'),
  },
  {
    key: 'retryCrossProviderCount',
    id: 'settings-retry-cross-provider-count',
    label: t('切换账号重试次数'),
    unit: t('次'),
    min: 0,
    max: 10,
    hint: t('失败后最多再换几个账号试（填 N = 最多换 N 个，首发那个不算）。换到同一家名下另一个账号、或换到另一家，都各算一次；换满仍失败就返回错误。可填 0–10，0 表示失败立即报错。'),
  },
  {
    key: 'retryIntervalSeconds',
    id: 'settings-retry-interval',
    label: t('重试间隔'),
    unit: t('秒'),
    min: 0,
    max: 300,
    hint: t('两次重试之间的等待时间，两项共用，可填 0–300 秒。'),
  },
]

/** when ： and config.rs KEY_TIMEOUT_* to JSON 1 */
export const TIMEOUT_FIELDS: NumberField[] = [
  {
    key: 'connectTimeoutSeconds',
    id: 'settings-timeout-connect',
    label: t('连接中超时'),
    unit: t('秒'),
    min: 1,
    max: 3600,
    hint: t('建立上游 TCP/TLS 连接或代理隧道的最大等待时间（秒），默认 30 秒，范围 1~3600。'),
  },
  {
    key: 'headersTimeoutSeconds',
    id: 'settings-timeout-headers',
    label: t('等待响应超时'),
    unit: t('秒'),
    min: 1,
    max: 3600,
    hint: t('请求发出后等待上游响应头的最大时间（秒），默认 300 秒，范围 1~3600。'),
  },
  {
    key: 'streamIdleTimeoutSeconds',
    id: 'settings-timeout-stream-idle',
    label: t('流式响应空闲超时'),
    unit: t('秒'),
    min: 1,
    max: 3600,
    hint: t('流式响应相邻数据之间允许的最大空闲时间（秒），收到新数据后重新计时，默认 300 秒，范围 1~3600。'),
  },
  {
    key: 'bodyTimeoutSeconds',
    id: 'settings-timeout-body',
    label: t('非流式响应超时'),
    unit: t('秒'),
    min: 1,
    max: 3600,
    hint: t('读取完整非流式响应体允许的最大时间（秒），默认 300 秒，范围 1~3600。'),
  },
]

/** etc ： and config.rs KEY_QUEUE_* to JSON 1 */
export const QUEUE_FIELDS: NumberField[] = [
  {
    key: 'queueMaxWaits',
    id: 'settings-queue-max-waits',
    label: t('排队等待次数'),
    unit: t('次'),
    min: 0,
    max: 10,
    hint: t('上游模型繁忙时会先排队（回一句「建议 N 秒后再来」）：网关等一会儿再发同一请求，最多等几次。默认 2 次，可填 0–10；填 0 表示不等待，排队直接返回 503 并说明「不是登录态或额度问题」。'),
  },
  {
    key: 'queueWaitSeconds',
    id: 'settings-queue-wait-seconds',
    label: t('排队等待秒数'),
    unit: t('秒'),
    min: 0,
    max: 120,
    hint: t('每次等待的时长，默认 0 = 跟随上游建议（实测 9~30 秒，上游没给建议时用 15 秒）。可填 0–120 秒强制一个固定时长。'),
  },
]

/** will （ and after KEY_IMAGE_COMPRESS_ENABLED 1 ） */
export const IMAGE_COMPRESS_ENABLED_KEY = 'imageCompressEnabled'

/**
 * parameter： and after config.rs KEY_IMAGE_COMPRESS_* to JSON 1 。
 *
 * default（ 2048 / 80 / 512KB） and OmniProxy default 1 ；
 * in ， default after as 。
 */
export const IMAGE_COMPRESS_FIELDS: NumberField[] = [
  {
    key: 'imageCompressMaxEdge',
    id: 'settings-image-compress-max-edge',
    label: t('长边像素上限'),
    unit: t('像素'),
    min: 256,
    max: 8192,
    hint: t('图片最长边超过这个值就等比缩小（只缩不放）。默认 2048，可填 256–8192 —— 多数视觉模型在这个量级已经够用，调小更省流量、但细节会丢。'),
  },
  {
    key: 'imageCompressQuality',
    id: 'settings-image-compress-quality',
    label: t('重编码质量'),
    unit: '',
    min: 40,
    max: 95,
    hint: t('缩放后重新存成 JPEG 时用的质量。默认 80，可填 40–95；数值越小体积越小、画质越差。'),
  },
  {
    key: 'imageCompressThresholdKb',
    id: 'settings-image-compress-threshold',
    label: t('触发阈值'),
    unit: t('KB'),
    min: 0,
    max: 10240,
    hint: t('解码后小于这个体积的图片保持原样。默认 512KB，可填 0–10240；填 0 表示所有内嵌图片都过一遍。'),
  },
]

/** 「error」：、 and limit and after retry_api.rs */
export const NO_RETRY_CODES_KEY = 'noRetryStatusCodes'
export const RETRY_CODE_MIN = 100
export const RETRY_CODE_MAX = 599
export const RETRY_MAX_CODES = 50

/** ： down （「（default）」） and toast use —— old is */
export type PromptModeOption = { value: string; optionLabel: string; toastLabel: string }

export const PROMPT_MODES: PromptModeOption[] = [
  { value: 'passthrough', optionLabel: t('透传客户端 system（默认）'), toastLabel: t('透传客户端 system') },
  { value: 'custom', optionLabel: t('替换为网关提示词'), toastLabel: t('替换为网关提示词') },
  { value: 'append', optionLabel: t('追加网关提示词'), toastLabel: t('追加网关提示词') },
]

/* ─── and 1 ─────────────────────────── */

export type ParseResult = { ok: true; value: number } | { ok: false; message: string }

/**
 * before ：min–max （ after will ， this 1 ， and ）。
 * use /^\d+$/ not is Number() + Number.isInteger()： after will "1e2" 100、
 * "0x10" 16， this not is 「 use / 」 ， not 。
 * `noun` is ： use 「」， its use 。
 */
export function parseInteger(raw: unknown, min: number, max: number, noun: string): ParseResult {
  const text = String(raw ?? '').trim()
  // empty need ：empty not down then ， not （「 not can as empty」「must is 」）
  if (!text) return { ok: false, message: t('{noun}不能为空（可填 {min}–{max}）', { noun, min, max }) }
  if (!/^\d+$/.test(text)) return { ok: false, message: t('{noun}必须是整数', { noun }) }
  const value = Number(text)
  if (value < min || value > max) {
    return { ok: false, message: t('{noun}必须在 {min}–{max} 之间（当前填的是 {text}）', { noun, min, max, text }) }
  }
  return { ok: true, value }
}

/**
 * return 1 ： after by set， is ，
 * when use ， 「」。
 */
export function normalizeApp(saved: unknown, fallback: AppSettings): AppSettings {
  const result: AppSettings = { ...fallback }
  if (saved && typeof saved === 'object') {
    const record = saved as Record<string, unknown>
    if (typeof record.closeToTray === 'boolean') result.closeToTray = record.closeToTray
    if (typeof record.autostart === 'boolean') result.autostart = record.autostart
    if (typeof record.lanAccess === 'boolean') result.lanAccess = record.lanAccess
    if (typeof record.lanPanel === 'boolean') result.lanPanel = record.lanPanel
    if (typeof record.lightweightMode === 'boolean') result.lightweightMode = record.lightweightMode
  }
  return result
}

/**
 * response 1 （ / / when use 1 ）：
 * 「 」， / null / 1 use up 1 valid ——
 * not up no to empty， not "undefined" 。
 * return null unavailable（response not for ， or not to valid）。
 */
export function normalizeNumbers(
  fields: NumberField[],
  data: unknown,
  previous: Record<string, number> | null,
): Record<string, number> | null {
  if (!data || typeof data !== 'object') return null
  const record = data as Record<string, unknown>
  const next: Record<string, number> = { ...(previous || {}) }
  for (const field of fields) {
    const value = Number(record[field.key])
    if (Number.isInteger(value) && value >= field.min && value <= field.max) {
      next[field.key] = value
    }
  }
  // 1 valid not to （ after ）： by unavailable，
  // not 1 empty in up 
  if (!fields.some(field => Number.isInteger(next[field.key]))) return null
  return next
}

/* ─── （） ─────────────── */

/** → can （file KB to MB， use ） */
export function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes <= 0) return '0 B'
  if (bytes < 1024) return `${bytes} B`
  const kb = bytes / 1024
  if (kb < 1024) return `${kb.toFixed(1)} KB`
  const mb = kb / 1024
  if (mb < 1024) return `${mb.toFixed(1)} MB`
  return `${(mb / 1024).toFixed(2)} GB`
}

/** → （ use for ）； to 「—」 */
export function formatCount(value: unknown): string {
  const count = Number(value)
  if (!Number.isFinite(count) || count < 0) return '—'
  return count.toLocaleString('zh-CN')
}

/* ─── （ index.html ） ─── */

/** （`[data-tip]`） ：desktop-tooltip.tsx in up ， old this */
export const TIPS = {
  displayTheme: t('控制界面的深浅色，与左侧边栏底部的三个主题按钮是同一个设置（改哪一处，另一处立刻跟上）。「跟随系统」会随操作系统当前的浅色 / 深色自动切换，并在系统主题变化时即时跟上；选「浅色」或「深色」则把界面固定在该模式，不再随系统变化。窗口标题栏的深浅色会一并同步，不会出现深色界面配一条浅色标题栏的情况。'),
  displayZoom: t('等比放大或缩小整个界面（文字、控件、间距一起变），效果与浏览器按 Ctrl +/- 相同：80%–130%、5% 一档。窗口本身不缩放，变的是页面内容的显示比例，设置立即生效并记住，下次启动直接按这个比例打开。放得越大可视范围越小，窗口较窄或表格较宽时不建议调得太大。'),
  displayLanguage: t('界面语言。可选简体中文、繁體中文、English、日本語、한국어、Português (Brasil)，或「跟随系统」（按浏览器 / 系统的语言列表自动选）。切换后整页立即刷新；该偏好只存在本机（localStorage），不影响账号、密钥与转发。'),
  accountList: t('账号页表格的两项显示偏好，只存在本机（localStorage），不影响账号数据与转发。「积分合计只统计当前筛选出来的账号」：账号页每一家的分组带右侧有一枚「可用合计」，默认按整组求和 —— 不受筛选、搜索影响，被筛掉的账号也计入（余额是资产，禁用也只表示不参与转发，额度仍在账上）。打开这一项后改为只累加当前可见的行：例如筛出 1 个 100 积分的账号，合计就是 100，而不是同组两个账号之和 1,000。分页不参与这个口径 —— 翻页看的是同一份结果的不同屏，合计不随页码变化。「隐藏账号名」与表头「账号」旁边那颗眼睛是同一个开关：打开后账号列的名字整体显示成星号，适合截图分享时遮挡账号。') + ' ' + t('百分比类读数（如 OpenCode Go、Antigravity）不能相加，组头显示的是组内最低剩余比例。'),
  tray: t('默认关闭窗口不会退出程序，而是把窗口缩到系统托盘，转发继续在后台运行（OpenAI 客户端不受影响）；要彻底退出程序，请在托盘图标上右键选「退出」。关掉这个开关后，点关闭按钮即退出程序、转发随之中断。「开机自动启动」开启后，登录系统时会自动启动本程序（通常直接驻留托盘），不需要手动打开。'),
  lightweight: t('开启后，关闭窗口会销毁界面并释放 WebView 进程，网关转发、定时签到、余额查询仍在后台继续。ZCode 模型请求免验证时不受影响；上游要求人机验证时，需要重新打开界面，在账号的「套餐明细」中主动完成验证。此开关只在「关闭窗口时最小化到托盘」开启时有意义。'),
  lan: t('默认网关只监听 127.0.0.1，只有本机能访问。开启后网关改听所有网卡，同一局域网内的设备可以把 API 地址指向本机 IP 一起使用。出于安全考虑，开启前必须先注册一个面板管理员：管理接口从此要求管理员会话或网关 Key，模型额度不会对局域网裸奔；一把启用的网关 Key 都没有时，转发接口也会拒绝服务，直到创建第一把。改动需要重启应用生效。「同时开放网页管理面板」把管理界面也出给局域网（其他设备的浏览器打开本机 IP 即可管理），默认关闭 —— 桌面端的面板仍只由本程序自己出。'),
  units: t('控制报表与请求日志里 Token 读数的写法：开启后按中文量级显示（1.2亿 / 8400万），关闭则用 k / M / B 缩写（与上游文档、接口字段的写法一致）。这只影响显示口径，不改变任何统计与存储的数值。'),
  queue: t('只对「排队制」的上游生效：模型繁忙时上游不报错，只回一句「建议 N 秒后再来」，网关按建议时长等一会儿再发同一请求，等满次数仍排不上才把「排队中」作为错误返回（HTTP 503，文案会说明这不是登录态或额度问题）。等待发生在首个字节之前，吃的是「等待响应超时」那份预算 —— 两项设置一起决定一次请求最多卡多久；排队不会标记账号限额、也不会换账号（换谁都一样在排队）。保存后对下一个请求立即生效，不用重启。'),
  captchaPool: t('模型请求是否需要验证码由 ZCode 上游配置决定，领取验证与模型请求免验证分别处理。后台只检查配置和库存，不自动运行验证码或弹出拼图；需要验证时，在账号的「套餐明细」中点击「开始验证」。验证令牌一次性使用，短时间内过期，失败或取消后不会自动重试。'),
  captchaSides: t('两张开关分别管桌面端（应用内窗口，也包含开启局域网访问后从其他设备访问的那份面板）与网页端（Docker / 服务端自托管的浏览器面板）。默认都开启；桌面端只在本机使用、嫌验证耗时可以关掉，网页端暴露在网络上，建议保持开启。'),
  timeouts: t('上游请求四个阶段各自的等待上限（秒，1~3600）：①「连接中超时」= 建立 TCP/TLS 连接或代理隧道的最大等待，默认 30 秒；②「等待响应超时」= 请求发出后等上游响应头的最大时间，默认 300 秒 —— 网关请求上游恒带 stream，响应头在 SSE 建立时就到达，与模型思考多久无关；③「流式响应空闲超时」= 流式响应相邻两块数据之间允许的最大空闲，收到新数据即重新计时，默认 300 秒 —— 上游长时间不吐数据即判定连接僵死并断开；④「非流式响应超时」= 读完整份非流式响应体的总预算（一次性计时、不重置），默认 300 秒。四项都是「等待上限」，只要数据还在来，正常的流式回答就一直往下传。保存后对下一个请求立即生效，不用重启。'),
  retry: t('请求转发到上游失败时（服务器瞬时错误、连接失败，以及 WorkBuddy 的 11-128 敏感词拦截），网关会等一段间隔后再发一次。次数分两档，管的是两件事：①「同一账号重试次数」在同一账号上原地重发几次 —— 一般是提示词被上游拦截、或链路瞬时抖动，等一会儿重发往往就好了；②「切换账号重试次数」这份请求最多再换几个账号试 —— 首发的那个账号不算，每换一个扣一次，不分是不是同一家（换到同一家的下一个账号也算一次），换满还失败就把错误返回给客户端。两项都设为 0 表示失败不重试、直接报错。注意账号级限额（429）不占这里的换号次数 —— 那类失败走「冷却该账号并换下一个账号」的降级，与这里的重试是两条路。另有「指定错误码直接换号」名单（默认 402）：命中名单的上游状态码不在同一账号重发，直接按队列换下一个账号继续试（换号也占「切换账号重试次数」），换满仍失败才把错误返回给客户端。'),
  routing: t('转发一个请求时按什么顺序挑账号：①「路由策略」决定排队规则 —— 顺序优先是排第 1 的接走所有请求、直到它不能答（限流 / 故障 / 额度尽）才轮到下一位；负载均衡是每条请求换下一位，把负载摊到各个账号上。②「会话粘性」让同一会话（相同的模型 + 相同的开头消息）尽量跟住同一个账号 —— 多轮对话的上下文缓存能复用、少换号少浪费；自动档粘 5 分钟、会话内档粘 6 小时、每轮与关闭都不粘。账号的排队位次（账号表的第一列）在所有策略下都是地基，策略在它之上再排。此外路由还有三层自愈保护：转发失败的账号退避休息一会儿（15 分钟起步翻倍、封顶 1 小时）、被 429 的账号垫底排到队尾 30 分钟、余额不足 / Token 周期用量到顶的账号由限制器拦下 —— 前两层可以在下面按需开关，限制器由账号的「限制器」设置管。'),
  sanitize: t('指纹脱敏会在转发前按词表顺序改写请求消息中的文本，包括对话内容、思考内容和工具调用参数。默认词表包含客户端模板句与计费表头的精确替换、正则删除规则，可以新增、编辑、删除、启停和调整顺序；替换内容留空表示删除匹配内容。默认应用全部提供商，也可指定多家。改动只作用于发给上游的副本，响应内容不变。编辑后点击「保存设置」，下一次请求生效；清空词表表示不做任何指纹改写，恢复默认只重置词表。'),
  developerRoleCompat: t('OpenAI 新版 API 用 developer 角色取代 system 承载指令，新版 OpenAI SDK 与部分客户端发的对话里会出现 role 为 developer 的消息；不少 OpenAI 兼容上游（尤其是自定义提供商的透传路径）不认识这个角色，收到就报 400。开启本项后，网关在每次转发前把 developer 角色的消息改写为 system —— 两类角色在 OpenAI 语义上等价（OpenAI 自己的迁移指南也是这么映射的），改写不影响对话内容与语义；对本来就只认 system 的各家内置上游同样无害。默认开启；关掉后 developer 消息会原样发往上游，不认识它的上游会报「角色不存在」一类的错误。'),
  imageCompress: t('对话里内嵌的 base64 大图（截图、照片）既撑大请求体，也拉高按像素折算的上游计费。开启后，网关在转发前把超过阈值的图等比缩到长边上限、重新存成 JPEG —— 只改发给上游的副本，请求日志里保存的仍是你发来的原图。同一张原图在多轮对话里重复出现时复用上一次的压缩结果，不会每轮重算。http 链接图片、GIF 动图、SVG 一律不动；压完反而更大的图也保留原图。只作用在对话场景，生图与图片编辑不受影响。默认开启；不想要的话把开关关掉即可。'),
  clineHeaders: t('Cline 上游按请求头判定「调用方是不是 Cline 自家产品」：X-CLIENT-TYPE: cline-sdk 是硬门槛（缺了它免费池模型一律 403），版本号与平台标识让请求更贴近官方客户端的形态。这里列出的是网关默认会发的头：改某一行的值 = 覆盖默认值（也可以新增默认清单之外的自定义头）；把值清空保存 = 这个头不发送；在界面删掉被改过的行 = 回落默认值。改完点「保存」，下一个 Cline 请求立即生效。不懂某一行的含义就不要动它 —— 伪装头的价值在「与官方客户端逐字一致」。'),
  cors: t('网关面（/v1/*）default not request： will down ，「none API」。Note is * —— 「 Key」 when can up ，default； to use ， and （ /v1）。（/api/*） not 。'),
  zcodeVersion: t('ZCode （ when ） by up by **client** down not down ： in to when return， after when 1 empty（ not is ， up 「 no has 」）。2026-10 that ：3.16.0 and down empty，3.16.1 has 。 default will update， up can can 1 —— that when ifclient can to 、 this as empty， in this clientcurrent （ 3.16.2）。 ZCODE_APP_VERSION 。'),
  prompt: t('client（Claude Code / Codex etc CLI） will in system ， up by ， in （HTTP 400）。 can in that ， 1 client can can new 。 this can system that ：①「」= not client system（default， as and before 1 ）；②「」= clientall system / developer ， （client）；③「」= in system after 1 ， has not （client and and use ）。 that has ：file（empty use default）， or in up —— that as （file），empty then to file / default。， / down （ is ） when ， will 1 in 1 ， and 「」 to 00:00 —— allrequest in ， not 1 400； this can and before 。 up is **default**： to ， use down 「 by 」 1 （「」 default in that ）—— not up for this not 1 。'),
  debug: t(' after ， will ** to up request**（request + request） and ** up return response**（ + response + response） to ，request 「」。request Authorization、Cookie、API Key etc 1 [redacted]， not will ；request and response by （ can can for ）。 500 ， after old 。 this is use when ， not need need when 。'),
  io: t(' will and 1 JSON file， can to 1 up after use 。 use and ： down by （UID / userId / apiKey etc ） —— already in update， has ； new to ， not will current in use ； by id and ， when 。'),
  backup: t(' will and need 1 as .a2ab ， can in Windows、macOS or Linux up ，none need 。 when 、config、 and set， and use new ， not already has 。 can file； old .sql can 。 before ， after need need new 。'),
  retention: t(' and request when and limitlimit。default 30 、10000 ， by ； 3 。requestdefault 99999 、1000000 （100 ）， after from old ， and delete for 。 by default 99999 ， by ， not request。 can 1–99999， can 1–100000000。 1 set and after ， and not can 。'),
  storage: t('（、、request、、set） 1 in configdirectory down agent2api.db in ， in 。 use 「 / 」， not need 。set AGENT2API_PROXY_HOME after 。'),
  modelsDev: t('models.dev is 1 （）， this use 「 up no 」 can ： up down 、、 use 、、、 is supports， and this 。 is ： in 「 can 」 > up > models.dev —— up no empty， from not up already 。 1 ，、、 not when has ； when will by 「directory new 」 update（ 1 1 and new ， can in this new ）。 new ， already to use 。'),
} as const

/** （`.hint.retention-note`） and */
export const NOTES = {
  timeouts: t('保存后立即生效，不用重启；超时按上游失败处理（计入请求日志的「错误」列，并按「请求重试」的设置决定是否重试）。'),
  queue: t('等待期间请求日志的状态列会显示「排队中」阶段，详情弹窗的「内部重试」里能逐次看到等待时长；等待用尽后返回 HTTP 503（不是 401/429：既不会刷新凭证，也不会把账号标成限额）。'),
  captchaPool: t('保存后状态读取立即生效。后台不会自动生成令牌；需要人机验证时，在账号的「套餐明细」中主动开始验证。模型请求免验证时无需保持界面打开。'),
  retry: t('保存后立即生效，不用重启；次数用尽仍失败时，错误原样返回给客户端。'),
  routing: t('保存后立即生效，不用重启：策略与粘性作用于**下一次**转发的选路，进行中的请求不受影响。'),
  retryCodes: t('命中这些上游状态码的失败不在同一账号重发，直接换下一个账号继续试（换号占用「切换账号重试次数」，换满仍失败才把错误原样返回给客户端）。输入 100–599 的状态码后回车添加，点 × 或退格删除；默认 402（积分不足）。清空全部表示任何错误都照常重试。'),
  sanitize: t('默认开启并应用全部提供商。编辑词表或应用范围后点击「保存设置」，下一次请求立即生效，不用重启。命中明细仍记进请求日志的「敏」标签。'),
  developerRoleCompat: t('默认开启。只改 role 字段本身，消息内容与顺序都不动；本项只改发给上游的副本，客户端看到的响应内容不变。'),
  imageCompress: t('默认开启。只改发给上游的副本，客户端看到的响应与原图都不变；请求日志里保存的是客户端发来的原图，不是压缩后的版本。只作用在对话链路，生图 / 图片编辑不经过这里。改完对下一个请求立即生效，不用重启。'),
  clineHeaders: t('值与默认值相同的行不占覆盖（存的是「改过的键」，不是全量配置）；值留空保存表示这个头不发送，删除改过的行表示回落默认值。改完立即生效，不用重启。'),
  zcodeVersion: t('留空 = 用插件内置的默认版本号（每期活动由插件跟随官方更新）。只有遇到「官方客户端里能看到活动、这里探测为空」时才需要填：那种情况几乎都是上游抬高了版本门槛，而版本落后的表现是一份空列表而不是报错。按官方客户端当前版本填（形如 3.16.2），保存后回到套餐明细刷新一次即刻生效。环境变量 ZCODE_APP_VERSION 若已设置，优先级高于这里。'),
  cors: t('默认关闭。改动立即生效，不用重启；关闭期间的跨源请求会被预检拦下。'),
  debug: t('开启后，新发生的请求会保存上游调试报文与下游正文，历史请求无法补录。上游调试报文只保留最近 500 条；两侧条数可在「数据 → 数据存储」查看。关闭开关不会删除已保存的报文，可在请求日志中选择「仅清空报文原文」一起清理。'),
  promptMode: t('替换 = 客户端 system / developer 消息全部删掉，换成网关的提示词；追加 = 保留客户端内容，只在开头 system 块之后多插一条。两者都只改发给上游的副本，客户端看到的响应不变。'),
  promptFile: t('UTF-8 文本文件路径，留空用内置默认提示词（通用编码助手提示词，不含任何会被上游拦截的模板句）。只在「替换 / 追加」模式下生效，透传模式不读它。'),
  promptText: t('在这里直接编辑提示词正文：保存后以这份文本为准（优先于上面的提示词文件），清空则回到文件 / 内置默认。改一个字不必再去编辑器里开文件。'),
  promptGatewayText: t('上游按「结构」校验身份：三段必须各自成块，所以这里分开编辑三段（不能合成一段）。动态段里的运行值写成占位符，发请求时才替换：{cwd} 工作目录、{platform} 平台、{shell} shell、{os_version} 系统版本、{git} 是否 git 仓库、{provider} 提供方标识、{model} 模型名。清空某一段 = 那一段恢复官方原文。第三段发出时前面会自带一个空行（官方形状）。'),
  prompt: t('改完立即生效，不用重启（下一个请求就用新模式）。降级是自动的临时状态，解除后本模式自己的提示词立刻恢复。'),
  promptProviders: t('这里只列出已保存的提供商配置；未配置的提供商沿用全局设置，可从下拉框手动添加。用户保存的模式、文件和正文优先于全局设置，「跟随默认」只取消该提供商的模式、文件和正文覆盖。ZCode 套餐池默认由插件按三块装配，客户端系统正文合入第三块，不会自动创建替换规则；手动修改的网关提示词段或关闭开关仍以用户配置为准，能否被接受取决于上游规则。'),
  modelsDev: t('「模型管理」里每个能力的来源标记（上游 / models.dev / 已覆盖）读的就是这份数据；映射弹窗的思考档位下拉也按这里的数据只列该模型真实认识的档位。'),
  retention: t('保存后立即生效：保留天数和数量上限同时生效，调小任一项会立即清理超出的历史数据，且不可恢复。'),
  storageFile: t('配置目录下的 agent2api.db（账号、日志、请求记录、调试报文与设置都在里面）。'),
  storageSize: t('数据库主文件与 WAL 文件的总占用，与请求日志清理弹窗采用同一口径；清理后可压缩数据库回收空闲空间。'),
  storage: t('数据统一保存在配置目录下的 agent2api.db；如需更换位置，请设置环境变量 AGENT2API_PROXY_HOME 后重启程序。'),
  captcha: t('开启后，登录页在浏览器后台自动完成验证（对真人无感），而脚本每次尝试都要先算一道题 —— 暴力破解与抢注的成本显著上升。仅影响面板的登录 / 注册，与 API 客户端的 API Key 无关。桌面端与网页端各自独立：桌面端主要服务本机与局域网，网页端（Docker / 服务端）直接暴露在网络上的登录页风险更高，建议保持开启。'),
  panelLogin: t('当前浏览器以管理员身份登录着本面板。「退出登录」会撤销这台设备的登录会话（30 天内的自动续期一并失效），需要重新输入账号密码；其他已登录的设备不受影响。'),
  exportDanger: t('导出文件内含 accessToken / refreshToken / apiKey 等凭证与自定义提供商定义，可直接用于登录。请妥善保管，不要外传或上传到公共位置。'),
  backupDanger: t('备份包含账号凭证、网关 Key、GitHub 令牌与云同步凭据的恢复密钥，整个文件受备份密码保护。请分开保管文件与密码；忘记密码无法恢复。'),
} as const

/** （`.settings-state`） ： and old 1 */
export const STATES = {
  appLoading: '—',
  appUnavailable: t('主进程未返回启动设置，请更新桌面端后重试'),
  appTrayOn: t('关闭窗口时程序不退出，转发继续在后台运行；退出请用托盘菜单'),
  appTrayOff: t('关闭窗口即退出程序，后台转发随之中断'),
  appLightOn: t('轻量模式已开启：关窗即销毁界面进程（释放约 200MB 内存），网关在后台继续；界面销毁期间 ZCode 套餐池（Start Plan）通道不可用，相关请求会在请求日志里注明原因'),
  appLightOff: t('未开启轻量模式：关窗仅隐藏窗口，界面进程常驻内存，随时打开不等待'),
  unitsOn: t('当前显示为「1.2亿 / 8400万」这类中文量级。'),
  unitsOff: t('当前显示为「1.20M / 8.4k / 1.2B」这类英文缩写。'),
  debugUnavailable: t('未能读取调试模式设置，请稍后重试'),
  debugOn: t('正在保存上游原始报文：'),
  debugOff: t('未开启，转发时不保存任何原始报文。'),
  sanitizeUnavailable: t('未能读取指纹脱敏设置，请稍后重试'),
  sanitizeOn: t('正在剥离出站请求里的审核指纹：表头键值整段删除，模板句最小改写。'),
  sanitizeOff: t('未开启，客户端 system 模板会原样发往上游，可能被内容审核误拦（400）。'),
  roleCompatUnavailable: t('未能读取开发者角色兼容设置，请稍后重试'),
  roleCompatOn: t('正在归一角色：developer 角色消息转发前改写为 system。'),
  roleCompatOff: t('未开启，developer 角色消息会原样发往上游，不认识该角色的上游会报错（400）。'),
  corsUnavailable: t('未能读取网关跨域访问设置，请稍后重试'),
  corsOn: t('正在应答跨源请求：浏览器跨域请求可直连 /v1/*。'),
  corsOff: t('，none。'),
  modelsDevUnavailable: t(' can models.dev ， after '),
  /** ：（ when that ）/ up new / */
  modelsDevSourceBuiltin: t('随包内嵌的快照（发版时的那份）'),
  modelsDevSourceCache: t('上次成功刷新的结果（存在本机数据库里）'),
  modelsDevSourceNetwork: t('刚刚从 models.dev 拉到的最新数据'),
  modelsDevEmpty: t('还没有装载任何数据。'),
  promptUnavailable: t('未能读取系统提示词设置，请稍后重试'),
  degradeUntilFallback: t('次日 00:00'),
  // ── （add： not ， is new ）──
  themeSystem: t('当前跟随操作系统的深浅色设置，系统切换时界面会自动跟上。'),
  themeLight: t('当前固定为浅色模式，不随系统变化。'),
  themeDark: t('当前固定为深色模式，不随系统变化。'),
  zoomWeb: t('网页端的界面缩放由浏览器自己控制（Ctrl + / Ctrl -，或浏览器菜单里的缩放），此项不可调。'),
  zoomDefault: t('当前按 100% 显示（默认比例）。'),
  languageHint: t('界面语言只保存在本机（localStorage）。切换后整页立即刷新，不影响账号、密钥与转发。'),
  groupTotalFilteredOn: t('合计只累加当前筛选后可见的账号，筛选、搜索一改数字就跟着变。'),
  groupTotalFilteredOff: t('合计按整组统计（默认）：不受筛选、搜索影响，被筛掉与已禁用的账号也计入。'),
} as const

// ── down from HEAD ──────────────────────────────
// settings-state.ts（ up 1 before ） import ； and 
// no import， in this 。 not is that import，
// etc that after can 。

export const ROUTE_STRATEGIES = [
  {
    value: 'order',
    label: t('顺序优先'),
    tip: t('排第 1 的接走所有请求，直到它不能答（限流 / 故障 / 额度尽）才轮到下一位 —— 就是「从上往下」的现状行为。'),
  },
  {
    value: 'rotate',
    label: t('负载均衡'),
    tip: t('每条请求轮换到下一个可用账号；新会话交给当前最闲的那位。适合想把手里的号摊开用、不让某一个一直扛。'),
  },
  {
    value: 'provider',
    label: t('家内轮询'),
    tip: t('家之间仍按优先级，只有同一家内部轮流用各账号 —— 想优先用某几家、又不愿把某一家的某个账号焊死时用它。'),
  },
] as const

/** will ： 1 will after request is 1 （ will = + use ） */

export const ROUTE_STICKY_OPTIONS = [
  {
    value: 'auto',
    label: t('自动'),
    tip: t('同一会话在 5 分钟内尽量跟住同一个账号；停了 5 分钟再发就算新会话。多数场景用它 —— 多轮对话少换号，也不至于把账号焊死。'),
  },
  {
    value: 'session',
    label: t('会话内'),
    tip: t('同一会话在 6 小时内跟住同一个账号；适合一坐几个小时的长会话，上游的上下文缓存能吃到更多。'),
  },
  {
    value: 'turn',
    label: t('每轮'),
    tip: t('每条请求都重新按策略选路 —— 行为与「关闭」一致，仅保留会话识别的记录。'),
  },
  {
    value: 'off',
    label: t('关闭'),
    tip: t('不做会话识别，每条请求都重新按策略选路。'),
  },
] as const

/** by （SegmentedControl parameter use ， not ） */
export type RouteStrategyValue = (typeof ROUTE_STRATEGIES)[number]['value']
export type RouteStickyValue = (typeof ROUTE_STICKY_OPTIONS)[number]['value']

/**
 * 「 by 」 （ after `/api/routing` ）。
 *
 * `key` is after （PUT ），`defaultOn` used for「current is default」，
 * in after 。
 */
export const ROUTE_PROTECTION_SWITCHES = [
  {
    key: 'routingFailoverBackoff',
    label: t('失败退避'),
    defaultOn: true,
    tip: t('转发失败的账号休息一会儿再回来：15 分钟起步，连续失败翻倍，封顶 1 小时；成功一次立即恢复。目的是「让这次失败先凉一会儿」，免得一条条地反复撞同一个坏账号。'),
  },
  {
    key: 'routingSink',
    label: t('限流垫底'),
    defaultOn: true,
    tip: t('还有额度时被 429 的账号排到队尾，30 分钟后自动回到队列（成功转发一次也会立即回来）。它标记的是「疑似共享池被别人抢」，不是「这个账号本身有问题」。⚠️ 注意它与路由策略的配合：在「家内轮询」下，队尾意味着这一家整体被往后排 —— 如果某个提供商的账号被批量限流，它们会集体沉到队尾，那个家可能长时间轮不到。这时可以关掉本项，让 429 过的账号只受冷却约束（冷却到点自动回来）。'),
  },
  {
    key: 'routingBreakerEnabled',
    label: t('熔断'),
    defaultOn: false,
    tip: t('连续失败到阈值后把这个账号整个打开（默认关）。与失败退避的分工：退避管「单次失败先凉一会儿」，惩罚轻、自动恢复；熔断管「这个账号现在整体不对劲，别再一条条地试」，要连续成功若干次才关。打开期间该账号被跳过，除非全场只剩它 —— 那时仍放行试一次。'),
  },
] as const

export type RouteProtectionKey = (typeof ROUTE_PROTECTION_SWITCHES)[number]['key']

export const CAPTCHA_POOL_FIELDS: NumberField[] = [
  {
    key: 'zcodeCaptchaPoolTarget',
    id: 'settings-captcha-pool-target',
    label: t('库存目标'),
    unit: t('个'),
    min: 1,
    max: 10,
    hint: t('验证码库存的参考目标，默认 3 个，可填 1–10。只影响库存状态展示，不会因为低于目标自动生成令牌或弹出验证码。'),
  },
  {
    key: 'zcodeCaptchaPoolPollMs',
    id: 'settings-captcha-pool-poll-ms',
    label: t('轮询间隔'),
    unit: t('毫秒'),
    min: 1000,
    max: 30000,
    hint: t('有近期套餐池请求时读取库存状态的间隔，默认 2500 毫秒，可填 1000–30000。没有近期请求时降低读取频率，轮询不会发起验证码。'),
  },
  {
    key: 'zcodeCaptchaPoolMaxPerRound',
    id: 'settings-captcha-pool-max-per-round',
    label: t('单轮铸造上限'),
    unit: t('个'),
    min: 1,
    max: 5,
    hint: t('保留旧版配置兼容，当前不会自动连铸。每次点击「开始验证」只取得一枚令牌，失败或取消后停止。'),
  },
]

/** 「error」：、 and limit and after retry_api.rs */

export type ModelsDevState = {
  /** new （ after already use ， after `KEY_MODELS_DEV_ENABLED`） */
  modelsDevEnabled?: boolean
  /** this ：`builtin`（）/ `cache`（ up new ）/ `network`（ ） */
  source?: string
  /** this when （ms；0 = unknown） */
  updatedAt?: number
  /** */
  count?: number
}

/** new 1 （ and directory new ， after `api::model_meta`）。 */
