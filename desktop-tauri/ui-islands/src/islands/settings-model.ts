/**
 * Agent2API · 设置页的**模型层**（桥类型 / 字段表 / 纯函数 / 页面文案）。
 *
 * 从 settings-page.tsx 拆出来：那一份「五个分类 + 十来个面板 + 一个确认框」的视图层装完
 * 已超过项目约定的单文件体量，而这一层的边界很清楚 —— 没有 JSX、没有状态。依赖单向
 * （settings-state.ts 与 settings-page.tsx import 它，它谁都不认识）。
 *
 * 它不是岛：文件名是 .ts，不会被 src/index.tsx 的 `islands/*.tsx` glob 加载；设置页的岛
 * 只有 settings-page.tsx 一个（页面级说明与取舍见它的文件头）。
 *
 * 这里的东西都是「只允许一处说了算」的口径：
 *   · 字段表（保留期 / 重试 / 超时 / 提示词模式）—— 键名必须与后端 config.rs 的常量逐字一致，
 *     散到视图里各写一遍必然漂（旧实现把这份对齐写在 settings-panel.js 的注释里）；
 *   · 页面文案 —— 从 index.html 的静态骨架逐字搬来（提示语 / 说明行 / 开关文字），
 *     静态骨架随本次迁移删除，这些字是页面上唯一的来源；
 *   · 纯函数（校验 / 归一化 / 文案派生）—— 状态层与视图层都要用。
 */

import type { SanitizePatch } from './settings-sanitize-model'
import type { CaptchaSetting, CaptchaPatch } from '../../../ui-shared/panel-captcha'
export type { CaptchaSetting, CaptchaPatch } from '../../../ui-shared/panel-captcha'
import { t } from '../i18n'


// ── 从 HEAD 恢复的类型（桥成员的返回形状）──────────────

/** 面板账户信息（`/api/panel/account`） */
export type PanelAccount = {
  registered?: boolean
  username?: string
  loginEnabled?: boolean
}

export type DatabaseTableStat = { table?: string; rows?: number }

/** 整库导出结果（壳命令 export_database） */
export type DatabaseExportResult = {
  canceled?: boolean
  bytes?: number
  file?: string
  tables?: DatabaseTableStat[]
  encrypted?: boolean
  includedKeys?: string[]
}

export type DatabaseImportResult = {
  /** 用户在文件框里点了取消（桌面壳 / 网页端选文件都可能是这个出口） */
  canceled?: boolean
  /** 恢复后的各表行数 */
  tables?: DatabaseTableStat[]
  /** 替换前自动建的安全备份文件名（{config_dir}/backups/ 下） */
  safetyBackup?: string
  /**
   * 密钥指纹不匹配的警告（**只在有问题的恢复里出现**，文案由后端 `db::backup` 的
   * `account_key_warning` 生成并给全）。
   *
   * 它**不是失败**：导入照常完成，但这份备份是用另一把加密密钥做的（或备份时本机
   * 还没生成过密钥），导入后的账号数据解不开、账号列表会是空的。所以界面**必须**
   * 把它常驻显示出来（面板内的警告框）—— 只塞进几秒即逝的 toast 等于没提示，
   * 用户错过后只会对着空账号列表发懵。
   */
  warning?: string
  encrypted?: boolean
  restoredKeys?: string[]
  reencryptedSecrets?: number
  localKeysPreserved?: boolean
}

/**
 * 恢复进度快照（`GET /api/backup/progress`，见 `db::backup` 的 `RestoreProgress`）。
 *
 * 字段全部由后端给：`phase` 是「这一步在做什么」的可读文案，`percent` 是阶段
 * 起点的大致位置。前端不自己算百分比 —— 各阶段耗时没有可靠计量（`execute_batch`
 * 不回调），一个只增不减的粗进度比一个乱跳的假进度诚实。
 */
export type RestoreProgressSnapshot = {
  /** 正在跑 */
  active?: boolean
  /** 当前阶段文案 */
  phase?: string
  /** 大致百分比（0-100） */
  percent?: number
  done?: boolean
  failed?: boolean
  /** 失败原因（`failed` 时才有） */
  error?: string | null
  /** 结果（`done` 时才有，形状同 DatabaseImportResult） */
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

/** 面板状态行要的那几个字段（避免这里 import 状态层的类型造成循环依赖） */
type ModelsDevView = {
  status: 'loading' | 'ready' | 'unavailable'
  enabled: boolean
  source: string
  updatedAt: number
  count: number
}

/**
 * models.dev 面板的状态行：把「开关 + 数据来历 + 生成时刻 + 条数」写成一句话。
 *
 * 为什么不只写「已开启」：这个开关的价值全在**那份元数据是什么时候的、有多少
 * 条** —— 只报开关状态，用户没法判断「我的档位为什么还是老样子」是没开、
 * 是没拉到、还是这份快照本来就旧。
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
        // 后端将来加了档位时原样回显，不显示成空白
        : state.source
  const stamp = state.updatedAt > 0
    // 与别处的时间展示同源（app.js 提供的 `formatTime`；岛里从共享桥取）
    ? String(shared().wbApp?.formatTime?.(state.updatedAt) ?? '')
    : ''
  if (!origin && !stamp && !state.count) return STATES.modelsDevEmpty
  const at = stamp ? t(', {time}', { time: stamp }) : ''
  const mode = state.enabled ? t('Updating online') : t('Using existing data only, no online refresh')
  return t('{count} entries · from {origin}{at} · {mode}', {
    count: Number(state.count).toLocaleString('zh-CN'),
    origin: origin || t('Unknown source'),
    at,
    mode,
  })
}

/* ─── 桥与共享全局 ─────────────────────────── */

/** 启动与托盘设置（壳命令 get_app_settings / save_app_settings） */
export type AppSettings = {
  closeToTray: boolean
  autostart: boolean
  /** 局域网访问：监听 0.0.0.0（改动随「应用重启」生效，见 TIPS.networkAccess） */
  lanAccess: boolean
  /** 局域网访问开启时是否同时托管网页管理面板（同样随重启生效） */
  lanPanel: boolean
  /** 轻量模式：关窗销毁界面进程（释放 WebView2 内存），托盘按需重建 */
  lightweightMode: boolean
}

/**
 * 「版本信息」的读数（`GET /api/system-info`，桌面壳命令同名）。
 *
 * 每个字段都可能是 null / 缺失（后端按分项降级，见 `api::system_info`）：
 * 界面一律按「—」显示，不猜。`deployment` 是后端判定的运行形态
 * （desktop / windows-service / docker / systemd / foreground），
 * `runtime.webviewVersion` 桌面端是 WebView2 / WKWebView、网页端是浏览器内核。
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
    thirdPartyReasoningCollapse?: boolean | null
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

/** 导入失败项：customProvider 标记的那条不是账号，是自定义提供商定义 */
export type ImportError = { id?: string; message?: string; customProvider?: boolean }

/** 导入结果（壳命令 import_accounts） */
export type ImportResult = {
  canceled?: boolean
  importNotice?: string
  warning?: string
  total?: number
  added?: number
  updated?: number
  skipped?: number
  failed?: number
  errors?: ImportError[]
  /** v2 导出文件附带的自定义提供商定义统计 */
  customProviders?: { added?: number; updated?: number }
}

/** 导出结果（壳命令 export_accounts） */
export type ExportResult = {
  canceled?: boolean
  count?: number
  /** v2 导出文件附带的自定义提供商定义条数 */
  customProviders?: number
  file?: string
}

/**
 * 网关自带提示词的**三段正文**（身份句 / 稳定段 / 动态段）。
 *
 * 三段各自成块是上游对身份的**结构**要求（实测：三段并成一段 → 405/3012，
 * 见后端 `zcode::OFFICIAL_PROMPT_NOTE`），所以界面与配置都按段来，不做
 * 「合成一整段」的编辑方式。动态段里的运行值写成占位符（`{cwd}` 等），
 * 发请求时才换成真实值。
 */
export type GatewayBlocks = { identity: string; stable: string; dynamic: string }

/** 提示词写入载荷：只传变化的那一项（后端允许部分字段），clearDegrade 是同一端点上的动作位 */
export type PromptPatch = {
  promptMode?: string
  promptFile?: string
  /**
   * 界面里编辑的提示词正文（**优先于** `promptFile` 与内置默认）。空串 = 清除
   * 这一份、回落文件 / 内置默认；缺失 = 这一项不改。
   */
  promptText?: string
  /**
   * 按提供商的覆盖：`{ "<providerId>": {promptMode?, promptFile?, promptText?} | null }`
   * （键名与全局那几项、以及响应里每家的字段完全一致，三处只有一套名字）。
   * 值是 `null` = 删掉这一家的覆盖（回落全局设置）；整张表缺失 = 这一维不改。
   */
  promptProviders?: Record<
    string,
    { promptMode?: string; promptFile?: string; promptText?: string } | null
  >
  /**
   * 网关自带提示词的逐家开关：`{ "<providerId>": true | false | null }`。
   * 值 `null` = 删键回到默认（装）；整张表缺失 = 这一维不改（与上面那张表同一约定）。
   */
  promptGateway?: Record<string, boolean | null>
  /**
   * 网关自带提示词的**正文覆盖**：`{ "<providerId>": {identity?, stable?, dynamic?} | null }`。
   * 段是**部分更新**（未出现的段保持原值，空串 = 这一段回到官方原文）；值 `null` =
   * 这一家整个回到官方原文。
   */
  promptGatewayText?: Record<string, Partial<GatewayBlocks> | null>
  clearDegrade?: boolean
}

/** Cline 伪装头接口的响应（GET 与 PUT 同形，键与后端 api/cline_headers.rs 一致） */
export type ClineHeadersData = {
  /** 用户改过的键（原样回显；空值 = 这个头不发送） */
  overrides: Record<string, string>
  /** 默认值（代码里内置的那一套；界面据此判断「改回了默认」） */
  defaults: Record<string, string>
  /** 默认值 ∪ 覆盖合并后的生效值（适配器实际会发的那份） */
  effective: Record<string, string>
}

/** 本页用到的壳 / HTTP 桥（方法名与 bridge.rs 一一对应，一个都不能改） */
export type SettingsBridge = {
  /** 'desktop' | 'web'：面板登录那一块只在网页端有意义 */
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
  getReasoningCollapse(): Promise<unknown>
  saveReasoningCollapse(enabled: boolean): Promise<unknown>
  getClineHeaders(): Promise<unknown>
  saveClineHeaders(overrides: Record<string, string>): Promise<unknown>
  getCors(): Promise<unknown>
  saveCors(enabled: boolean): Promise<unknown>
  getPrompt(): Promise<unknown>
  savePrompt(payload: PromptPatch): Promise<unknown>
  getStorage(): Promise<unknown>
  getCaptchaSetting(): Promise<CaptchaSetting | null | undefined>
  saveCaptchaSetting(patch: CaptchaPatch): Promise<CaptchaSetting | null | undefined>
  /** 面板账户信息（设置页「账户」分类）；未注册时 registered=false */
  getPanelAccount(): Promise<PanelAccount | null | undefined>
  /** 改密码：要求旧密码；成功后后端撤销其他会话链，本会话保留 */
  changePanelPassword(oldPassword: string, newPassword: string): Promise<unknown>
  panelLogout(): Promise<unknown>
  /** 面板会话状态（桌面壳的登录通道）；网页端由 shim 以 cookie 状态回答 */
  panelSessionStatus?(): Promise<{ registered?: boolean; authenticated?: boolean } | null | undefined>
  /** 清除本机的面板会话令牌（登出时与 panelLogout 配套） */
  panelSessionClear?(): Promise<unknown>
  /** 后端就绪状态（局域网地址展示要拼网关端口；其余字段本页不消费） */
  getBackendStatus?(): Promise<{ port?: number } | null | undefined>
  onBackendReady?(callback: () => void): (() => void) | void
  // ── 局域网访问（仅桌面端渲染，web 端 shim 不提供这几个方法）──
  /** 面板管理员是否已注册（开启流程据此决定要不要先走注册步） */
  panelAdminStatus?(): Promise<{ registered?: boolean } | null | undefined>
  /** 注册面板管理员（受信本地路径，不走公开的 HTTP 注册端点） */
  panelRegister?(username: string, password: string): Promise<{ registered?: boolean; existed?: boolean }>
  /** 切换局域网访问（写设置并重启应用）；返回里的 createdKey 非空 = 已自动补了首把 Key */
  changeLanAccess?(enabled: boolean, panel: boolean): Promise<unknown>
  /** 本机在局域网里的地址（查不到为 null） */
  localIp?(): Promise<string | null | undefined>
  /** 打开外链的唯一出口（只放行 http(s)）：桌面走系统浏览器，网页端 shim 是 window.open */
  openReleasePage?(url: string): Promise<unknown>
  /**
   * 设置页「版本信息」的一次性读数（桌面壳命令 / 网页端 shim 同名方法）。
   *
   * `hideUsername` 是**纯展示**参数：命中的路径在主目录前缀处换成 `~`，
   * 脱敏在后端做 —— 这样「复制全部信息」拿到的也是脱敏后的文本。
   */
  getSystemInfo?(options?: { hideUsername?: boolean }): Promise<SystemInfo | null | undefined>
  // ── 以下成员从 HEAD 恢复（并行改动移走了定义但 settings-state.ts 仍引用）──
    exportDatabase?(includeLogs: boolean, password: string): Promise<DatabaseExportResult | null | undefined>
    importDatabase?(password?: string): Promise<DatabaseImportResult | null | undefined>
    /** 恢复进度（`GET /api/backup/progress`）：弹窗每 500ms 拉一次 */
    getBackupProgress?(): Promise<RestoreProgressSnapshot | null | undefined>
    getCaptchaPool(): Promise<unknown>
    saveCaptchaPool(patch: Record<string, number>): Promise<unknown>
    getRoutingConfig(): Promise<unknown>
    setRoutingConfig(patch: Record<string, unknown>): Promise<unknown>
    getModelsDev(): Promise<ModelsDevState | null | undefined>
    saveModelsDev(on: boolean): Promise<ModelsDevState | null | undefined>
    refreshModelsDev(): Promise<{ result?: ModelsDevRefreshResult; status?: ModelsDevState } | null | undefined>
    // ── 云同步（设置页「数据 → 云同步」，面板见 settings-cloud-sync.tsx）──
    // 全部可选：网页端 shim 与桌面桥都提供，但老壳（升级前）没有这几个方法 ——
    // 面板据此显示「不可用」而不是崩掉。
    getCloudSync?(): Promise<unknown>
    saveCloudSync?(payload: Record<string, unknown>): Promise<unknown>
    testCloudSync?(payload: Record<string, unknown>): Promise<unknown>
    getCloudSyncRemote?(): Promise<unknown>
    cloudSyncUpload?(): Promise<unknown>
    cloudSyncDownload?(): Promise<unknown>
    importCloudSyncKey?(password: string): Promise<unknown>
}

/**
 * window 上由其它脚本 / 其它岛挂载的共享桥。
 *
 * 刻意用「局部窄类型 + 转型」而不是 declare global 往 Window 上加属性：
 * workbuddyDesktop / wbApp / wbUnits 是多个岛共用的桥，各岛各 declare 一份会因同名属性
 * 类型不一致直接报 TS2717（并行迁移时必然撞车）。这里只认本页用到的成员；
 * 设置页独占的 wbSettingsPanel 由 settings-page.tsx 自己 declare。
 */
export type SharedWindow = {
  workbuddyDesktop?: SettingsBridge
  wbApp?: {
    toast?: (message: string, kind?: 'err' | 'ok' | 'warn') => void
    /** 账号被导入改动后让主界面重绘（账号列表、导航计数） */
    refresh?: () => Promise<void> | void
    /** 当前页标识：脚本加载时若已停在设置页，补一次 load() */
    readonly currentPage?: string
    /** 应用显示模式（system / light / dark）：实现与持久化都在 app.js，本页只调用 */
    applyTheme?: (mode: string) => void
    /** 应用界面缩放（传百分数，如 105）：由 app.js 落到 WebView 层并记档 */
    applyZoom?: (percent: number) => void
    /** 时间戳 → 本地时间串（app.js 的全局实现；models.dev 面板的「数据是什么时候的」用它） */
    formatTime?: (value: unknown) => string
  }
  /** Token 计量单位的展示口径（units.js）：本页只负责拨开关，格式化在那边 */
  wbUnits?: { isChinese?: () => boolean; setChinese?: (on: boolean) => void }
  /** 软件更新面板的岛（update-panel.tsx）：切到设置页时让它自己刷新一次 */
  wbUpdatePanel?: { load?: () => Promise<void> | void; check?: () => Promise<unknown> }
  /** 版本信息面板的岛（system-info-pane.tsx）：同上 */
  wbSystemInfoPane?: { load?: () => Promise<void> | void }
  /** 内联图标集（icons.js）：左栏分类图标由它渲染（返回 SVG 串，注入用） */
  wbIcons?: { icon?: (name: string, size?: number) => string }
}

export function shared(): SharedWindow {
  return window as unknown as SharedWindow
}

export function errorMessage(error: unknown): string {
  const message = error instanceof Error ? error.message : String(error ?? '')
  return message.trim() || t('Unknown error')
}

/** toast 的统一出口（运行期读 wbApp，不在模块顶层解构） */
export function toast(message: string, kind?: 'err' | 'ok' | 'warn'): void {
  shared().wbApp?.toast?.(message, kind)
}

/**
 * 打开外链：一律交给系统默认浏览器（桌面走壳命令 `open_release_page`，网页端
 * shim 把它映射成 window.open）—— 与 update-shared 的同名函数是**两份实现**，
 * 不跨族 import 是刻意的：这里服务设置页（反馈与需求），那边服务更新面板一族，
 * 各自只依赖自己那份桥类型；但也别再加第三份，要用先从这两处挑。
 */
export async function openExternal(url: string): Promise<void> {
  try {
    await shared().workbuddyDesktop?.openReleasePage?.(url)
  } catch (error) {
    toast(t('Failed to open the link: {error}', { error: errorMessage(error) }), 'err')
  }
}

/* ─── 分类与偏好键 ─────────────────────────── */

/**
 * 左侧分类。顺序 = 界面顺序；showCategory 用它校验传进来的值（旧实现是查 DOM，
 * 这里改成查这份表 —— 分类不再由 HTML 声明，而是本页渲染出来的）。
 *
 * 「重试」「超时」从「网关」里拆出来独立成菜单（原先挤在网关一栏里，排在排队等待
 * 两侧）：这两组是**每次转发都会读**的网络行为参数，出问题时最常被翻，单独一栏
 * 少一次翻找。数据加载本就不分分类（settings-state 的 load 一次并行取全部），
 * 拆分只是视图层的两段搬迁。
 *
 * 「显示」紧跟「通用」：显示模式 / 界面缩放 / 语言都是**纯前端偏好**（存在
 * localStorage 里，与主题同族），不碰后端配置 —— 放在最前面那几栏里最顺手。
 *
 * 「反馈与需求」是纯跳转面板（三个 GitHub issue 表单入口，见 settings-page 的
 * FeedbackPane），排在「更新」上面 —— 都是「对外」的两栏，挨着放。
 *
 * `icon` 是 icons.js 里那组设置页分类图标的键（描边风格，见那边的说明）；标签
 * 不再受两字限制，图标负责在窄栏里一眼认出，文字负责说清。
 *
 * 「更新」（原「关于」）**id 保持 `about` 不变**：它同时是 update-panel.tsx 的挂载点
 * 选择器（`.settings-pane[data-cat="about"]`）与用户 localStorage 里存着的分类值，
 * 改名会让两者当场失配 —— 用户看到的只是标签，id 是内部契约。
 */
export const CATEGORIES = [
  { id: 'account', label: t('Account'), icon: 'accounts' },
  { id: 'general', label: t('General'), icon: 'sliders' },
  // 注：AI 助手的配置（对话模型 / 系统提示词 / 工具勾选）已搬到助手页自己的
  // 「助手设置」抽屉（见 assistant-settings-drawer.tsx）—— 那几项都是聊着聊着
  // 就想改的，留在这里意味着每次调整都要离开对话页再回来。分类不再保留，
  // 老用户 localStorage 里存着的 'assistant' 由 settings-state 的兜底回落到首项。
  { id: 'display', label: t('Shown'), icon: 'display' },
  { id: 'gateway', label: t('Gateway'), icon: 'traffic' },
  { id: 'providers', label: t('Provider'), icon: 'sliders' },
  { id: 'retry', label: t('Retry'), icon: 'refresh' },
  { id: 'timeout', label: t('Timeout'), icon: 'timer' },
  { id: 'security', label: t('Security'), icon: 'shield' },
  { id: 'data', label: t('Data'), icon: 'database' },
  { id: 'feedback', label: t('Feedback & Requests'), icon: 'feedback' },
  // 「版本信息」：软件版本 / 运行环境 / 路径 / 开关状态 / 关键计数，可一键
  // 复制成纯文本（面板在 system-info-pane.tsx）。紧挨「反馈与需求」—— 两者
  // 是同一件事的两端：出问题了先来这儿截图，再去那儿提 issue。
  // `system` 是新的内部 id，不与任何既有分类 / localStorage 取值冲突。
  { id: 'system', label: t('Version info'), icon: 'info' },
  { id: 'about', label: t('Update'), icon: 'download' },
] as const

/**
 * 「设置页当前分类」的 localStorage 键：纯前端偏好（与主题、计量单位同类），
 * 主进程不参与。键名沿用旧实现的取值，别改 —— 否则用户上次停留的分类会丢。
 */
export const SETTINGS_CAT_KEY = 'workbuddy-desktop-settings-cat'

/* ─── 显示偏好（「显示」分类） ───────────────── */

/**
 * 「显示」分类的三项偏好：显示模式 / 界面缩放 / 语言。
 *
 * ⚠ 前两项的**应用入口都不在本页**，而在 ui/app.js（applyTheme / applyZoom）：
 *   · 主题还要同步操作系统标题栏的深浅色，并处理「跟随系统」时窗口主题会污染
 *     WebView 颜色偏好的问题（那段长注释在 app.js）；
 *   · 缩放要走 WebView 层（壳命令 set_zoom），不是页面自己能做完的事。
 * 本页只做两件事：**读** localStorage 把控件摆到当前值上；改动时调
 * `wbApp.applyTheme / applyZoom`，再靠 'wb:theme' / 'wb:zoom' 事件跟随 ——
 * 侧边栏的主题三键与这里的档位是同一个设置的两个入口，必须互相同步。
 *
 * 键名与 app.js 里的字面量是同一份契约（两侧各写一份，改一处必然漂）；
 * 事件名同理（app.js 派发，本页监听）。
 */
export const THEME_KEY = 'workbuddy-desktop-theme'
export const ZOOM_KEY = 'workbuddy-desktop-zoom'
export const THEME_EVENT = 'wb:theme'
export const ZOOM_EVENT = 'wb:zoom'

/** 缩放档位边界与步长：与 app.js 的 ZOOM_MIN / ZOOM_MAX / ZOOM_STEP 同源 */
export const ZOOM_MIN = 80
export const ZOOM_MAX = 130
export const ZOOM_STEP = 5

/** 缩放候选项（80% ~ 130%，5% 一档共 11 档）：下拉的 value 就是百分数本身 */
export const ZOOM_PERCENTS: number[] = Array.from(
  { length: (ZOOM_MAX - ZOOM_MIN) / ZOOM_STEP + 1 },
  (_, index) => ZOOM_MIN + index * ZOOM_STEP,
)

/**
 * 显示模式三档：value 与 app.js / `data-theme` 的取值逐字一致（system / light / dark）。
 * 文案取侧边栏那三个按钮的 title（跟随设备 → 跟随系统，是同一件事的两种叫法，
 * 这里用了更书面的一种）。
 */
export const THEME_MODES = [
  { value: 'system', label: t('System') },
  { value: 'light', label: t('Light') },
  { value: 'dark', label: t('Dark') },
] as const

/** 显示模式取值（SegmentedControl 的泛型参数要用它，免得在视图里转字面量联合） */
export type ThemeMode = (typeof THEME_MODES)[number]['value']

/**
 * 语言选项已迁到运行时：界面语言的可选列表由 ui/i18n.js 暴露的 `wbI18n.LOCALES`
 * 提供（label 是各语言自己的写法），持久化键 workbuddy-desktop-locale。
 * 这里不再维护 LANGUAGES —— 两份列表放两处必然漂，语言这件事只有 ui/i18n.js 一个出处。
 */

/** 读当前显示模式（app.js 是写入方）：读到非法值按跟随系统 */
export function readThemeMode(): ThemeMode {
  try {
    const mode = localStorage.getItem(THEME_KEY)
    return mode === 'light' || mode === 'dark' ? mode : 'system'
  } catch {
    return 'system'
  }
}

/** 读当前缩放（百分数）：非法 / 越界值回落 100，与 app.js 的 storedZoom 同一口径 */
export function readZoomPercent(): number {
  try {
    const text = localStorage.getItem(ZOOM_KEY)
    // 空串 / 缺失要单独挡：Number('') 与 Number(null) 都是 0（不是 NaN），
    // 不挡就会被当成「0%」一路夹到 80% —— 「没设过」必须等于默认的 100%
    if (text === null || text.trim() === '') return 100
    const raw = Number(text)
    if (!Number.isFinite(raw)) return 100
    const snapped = Math.round(raw / ZOOM_STEP) * ZOOM_STEP
    return Math.min(ZOOM_MAX, Math.max(ZOOM_MIN, snapped))
  } catch {
    return 100
  }
}

/* ─── 数字字段表 ───────────────────────────── */

export type NumberField = {
  /** 后端 config 的 JSON 键（大小写必须逐字一致，否则 PUT 被静默忽略） */
  key: string
  /** 控件 id：保留旧 id 便于排查（没有任何外部脚本再按 id 读这些控件） */
  id: string
  label: string
  /** 输入框右侧的单位 */
  unit: string
  min: number
  max: number
  /** 行下方的说明（逐字来自 index.html 静态骨架） */
  hint: string
}

export { RETENTION_FIELDS, RETENTION_MIN, RETENTION_MAX } from './settings-retention-model'

/**
 * 三项重试字段。第二项的键名 `retryCrossProviderCount` 是**旧措辞**（配置兼容，
 * 改名会让老配置读不到、静默回落默认值），后端常量已经改叫 KEY_RETRY_ACCOUNT_SWITCH_COUNT
 * —— 这里必须沿用旧字符串，只有展示名跟着真语义走。
 */
export const RETRY_FIELDS: NumberField[] = [
  {
    key: 'retryCount',
    id: 'settings-retry-count',
    label: t('Same-Account Retries'),
    unit: t('times'),
    min: 0,
    max: 10,
    hint: t('How many times to resend in place on the same account (not counting the first send; the whole request shares one pool, and once it is used up, any account switched to gets only one send before switching continues). Accepts 0–10; 0 means switch accounts immediately on failure.'),
  },
  {
    key: 'retryCrossProviderCount',
    id: 'settings-retry-cross-provider-count',
    label: t('Account-Switch Retries'),
    unit: t('times'),
    min: 0,
    max: 10,
    hint: t('How many more accounts to try on failure (N = switch to at most N accounts; the account that made the first attempt does not count). Switching to another account under the same provider and switching to a different provider each count as one; once the limit is reached and the request still fails, an error is returned. Accepts 0–10; 0 means report the error immediately on failure.'),
  },
  {
    key: 'retryIntervalSeconds',
    id: 'settings-retry-interval',
    label: t('Retry Interval'),
    unit: t('seconds'),
    min: 0,
    max: 300,
    hint: t('Wait time between two retries, shared by both settings. Range: 0–300 seconds.'),
  },
]

/** 超时字段：键名与 config.rs 的 KEY_TIMEOUT_* 指向的 JSON 键逐字一致 */
export const TIMEOUT_FIELDS: NumberField[] = [
  {
    key: 'connectTimeoutSeconds',
    id: 'settings-timeout-connect',
    label: t('Connect Timeout'),
    unit: t('seconds'),
    min: 1,
    max: 3600,
    hint: t('Maximum time to establish the upstream TCP/TLS connection or proxy tunnel (seconds). Default 30 seconds, range 1–3600.'),
  },
  {
    key: 'headersTimeoutSeconds',
    id: 'settings-timeout-headers',
    label: t('Response Headers Timeout'),
    unit: t('seconds'),
    min: 1,
    max: 3600,
    hint: t('Maximum time to wait for the upstream\'s response headers after the request is sent (seconds). Default 300 seconds, range 1–3600.'),
  },
  {
    key: 'streamIdleTimeoutSeconds',
    id: 'settings-timeout-stream-idle',
    label: t('Stream Idle Timeout'),
    unit: t('seconds'),
    min: 1,
    max: 3600,
    hint: t('Maximum idle time allowed between adjacent chunks of a streaming response (seconds); the timer restarts on each new chunk. Default 300 seconds, range 1–3600.'),
  },
  {
    key: 'bodyTimeoutSeconds',
    id: 'settings-timeout-body',
    label: t('Non-Streaming Timeout'),
    unit: t('seconds'),
    min: 1,
    max: 3600,
    hint: t('Maximum time allowed to read a complete non-streaming response body (seconds). Default 300 seconds, range 1–3600.'),
  },
]

/** 排队等待字段：键名与 config.rs 的 KEY_QUEUE_* 指向的 JSON 键逐字一致 */
export const QUEUE_FIELDS: NumberField[] = [
  {
    key: 'queueMaxWaits',
    id: 'settings-queue-max-waits',
    label: t('Queue Wait Count'),
    unit: t('times'),
    min: 0,
    max: 10,
    hint: t('Busy upstream models queue first (replying "suggest waiting N seconds"): the gateway waits a while and resends the same request, up to a set number of waits. Default 2; accepts 0–10. 0 means no waiting: a queued response returns 503 directly and states that this is not a sign-in or quota problem.'),
  },
  {
    key: 'queueWaitSeconds',
    id: 'settings-queue-wait-seconds',
    label: t('Queue Wait Seconds'),
    unit: t('seconds'),
    min: 0,
    max: 120,
    hint: t('Duration of each wait. Default 0 = follow the upstream\'s suggestion (measured 9–30 seconds; 15 seconds when the upstream gives none). Accepts 0–120 seconds to force a fixed duration.'),
  },
]

/** 会话图片压缩的开关键名（与后端 KEY_IMAGE_COMPRESS_ENABLED 逐字一致） */
export const IMAGE_COMPRESS_ENABLED_KEY = 'imageCompressEnabled'

/**
 * 图片压缩参数：键名与后端 config.rs 的 KEY_IMAGE_COMPRESS_* 指向的 JSON 键逐字一致。
 *
 * 三项默认值（长边 2048 / 质量 80 / 阈值 512KB）与 OmniProxy 的默认值一致；
 * 它们只在本面板的输入框里作提示，真正的默认值以后端解析层为准。
 */
export const IMAGE_COMPRESS_FIELDS: NumberField[] = [
  {
    key: 'imageCompressMaxEdge',
    id: 'settings-image-compress-max-edge',
    label: t('Max long-edge pixels'),
    unit: t('px'),
    min: 256,
    max: 8192,
    hint: t('Images whose longest edge exceeds this value are scaled down proportionally (never enlarged). Default 2048, range 256-8192 — most vision models are fine at this size; a smaller value saves bandwidth but loses detail.'),
  },
  {
    key: 'imageCompressQuality',
    id: 'settings-image-compress-quality',
    label: t('Re-encode quality'),
    unit: '',
    min: 40,
    max: 95,
    hint: t('JPEG quality used when re-encoding the scaled image. Default 80, range 40-95; lower means a smaller file and lower quality.'),
  },
  {
    key: 'imageCompressThresholdKb',
    id: 'settings-image-compress-threshold',
    label: t('Trigger threshold'),
    unit: t('KB'),
    min: 0,
    max: 10240,
    hint: t('Images smaller than this size after decoding are kept as-is. Default 512KB, range 0-10240; 0 processes every inline image.'),
  },
]

/** 「指定错误码直接换号」：键名、状态码范围与名单上限与后端 retry_api.rs 逐字同源 */
export const NO_RETRY_CODES_KEY = 'noRetryStatusCodes'
export const RETRY_CODE_MIN = 100
export const RETRY_CODE_MAX = 599
export const RETRY_MAX_CODES = 50

/** 提示词模式：下拉的展示文案（含「（默认）」）与 toast 里用的短名分开 —— 旧实现是两份表 */
export type PromptModeOption = { value: string; optionLabel: string; toastLabel: string }

export const PROMPT_MODES: PromptModeOption[] = [
  { value: 'passthrough', optionLabel: t('Passthrough client system (default)'), toastLabel: t('Passthrough client system') },
  { value: 'custom', optionLabel: t('Replace with gateway prompt'), toastLabel: t('Replace with gateway prompt') },
  { value: 'append', optionLabel: t('Append gateway prompt'), toastLabel: t('Append gateway prompt') },
]

/* ─── 校验与归一化 ─────────────────────────── */

export type ParseResult = { ok: true; value: number } | { ok: false; message: string }

/**
 * 前端校验：min–max 的整数（后端也会挡，这里先挡省一次往返，且提示更短）。
 * 用 /^\d+$/ 而不是 Number() + Number.isInteger()：后者会把 "1e2" 认成 100、
 * "0x10" 认成 16，这些都不是「用户填了几天 / 几次」的直觉答案，不如直接判非法。
 * `noun` 是提示语的主语：保留期用「天数」，其余面板用自己的标签。
 */
export function parseInteger(raw: unknown, min: number, max: number, noun: string): ParseResult {
  const text = String(raw ?? '').trim()
  // 空串要单独挡：空值过不了下面的正则，但提示语不同（「不能为空」比「必须是整数」更准）
  if (!text) return { ok: false, message: t('{noun} cannot be empty (allowed range: {min}–{max})', { noun, min, max }) }
  if (!/^\d+$/.test(text)) return { ok: false, message: t('{noun} must be an integer', { noun }) }
  const value = Number(text)
  if (value < min || value > max) {
    return { ok: false, message: t('{noun} must be between {min} and {max} (currently entered: {text})', { noun, min, max, text }) }
  }
  return { ok: true, value }
}

/**
 * 保存返回值归一化：后端按契约应回传完整设置，但只认它是布尔才采纳，
 * 缺字段时沿用本次提交的值，避免把开关误渲染成「关闭」。
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
 * 数字面板的响应归一化（保留期 / 重试 / 超时共用同一口径）：
 * 只采纳「范围内的整数」，缺字段 / null / 字符串一概沿用上一轮的有效值 ——
 * 不把线上没给的项清空，也不写 "undefined" 进输入框。
 * 返回 null 表示整块不可用（响应形状不对，或三项都拿不到有效值）。
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
  // 一项有效值都拿不到（换壳后接口形状变了之类）：按不可用处理，
  // 不让一排空输入框留在页面上
  if (!fields.some(field => Number.isInteger(next[field.key]))) return null
  return next
}

/* ─── 读数格式化（数据存储面板） ─────────────── */

/** 字节数 → 可读大小（库主文件通常几百 KB 到几十 MB，四档够用） */
export function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes <= 0) return '0 B'
  if (bytes < 1024) return `${bytes} B`
  const kb = bytes / 1024
  if (kb < 1024) return `${kb.toFixed(1)} KB`
  const mb = kb / 1024
  if (mb < 1024) return `${mb.toFixed(1)} MB`
  return `${(mb / 1024).toFixed(2)} GB`
}

/** 条数 → 带千分位的文本（用户对着看更省事）；非法值给「—」 */
export function formatCount(value: unknown): string {
  const count = Number(value)
  if (!Number.isFinite(count) || count < 0) return '—'
  return count.toLocaleString('zh-CN')
}

/* ─── 页面文案（逐字来自 index.html 静态骨架） ─── */

/** 标题右侧问号（`[data-tip]`）的说明全文：desktop-tooltip.tsx 仍在页面上跑，照旧服务这些元素 */
export const TIPS = {
  displayTheme: t('Controls the UI\'s light / dark colors; it is the same setting as the three theme buttons at the bottom of the left sidebar (change either one and the other follows immediately). "System" switches automatically with the operating system\'s current light / dark setting and follows changes as they happen; choosing "Light" or "Dark" pins the UI to that mode regardless of the system. The window title bar\'s light / dark color stays in sync too, so you never end up with a dark UI under a light title bar.'),
  displayZoom: t('Zoom the whole interface proportionally (text, controls, and spacing scale together), equivalent to Ctrl +/- in a browser: 80%–130% in 5% steps. The window itself does not resize — what changes is the page content\'s display scale; the setting applies immediately and is remembered, so the next launch opens at this scale. The larger the scale, the less you can see at once; with a narrow window or wide tables, do not set it too high.'),
  displayLanguage: t('Interface language. Choices: Simplified Chinese, Traditional Chinese, English, Japanese, Korean, Portugueseês (Brasil), or System (picked automatically from the browser / system language list). Switching reloads the whole page immediately; the preference is stored only on this machine (localStorage) and does not affect accounts, keys, or forwarding.'),
  accountList: t('Two display preferences for the accounts table. They are stored only on this machine (localStorage) and never affect account data or forwarding. "Only sum accounts visible under the current filter": each provider band on the accounts page shows a "Total available" badge, and by default it sums the whole group — filtering and search do not change it, and accounts filtered out still count (balance is an asset; disabling an account only keeps it out of forwarding, the credit stays). Turn this on to add up only the rows currently visible: filter down to one account with 100 credits and the total reads 100, not 1,000 (the sum of two accounts in that group). Paging does not take part in this rule — pages are just different screens of the same result, so the total does not change as you flip pages. "Hide account names" is the same switch as the eye next to the "Account" column header: when on, names in the account column are shown as asterisks, handy for screenshots.') + ' ' + t('Percentage readings (such as OpenCode Go or Antigravity) cannot be summed; the band shows the group\'s lowest remaining percentage.'),
  tray: t('By default, closing the window does not quit the app; it shrinks the window to the system tray and forwarding keeps running in the background (OpenAI clients are unaffected). To quit the app completely, right-click the tray icon and choose "Exit". With this switch off, clicking the close button quits the app and forwarding stops with it. With "Launch at startup" on, the app starts automatically when you sign in to the system (usually staying in the tray), so you do not need to open it manually.'),
  lightweight: t('When enabled, closing the window destroys the interface and releases WebView processes. Forwarding, scheduled check-ins and balance queries continue in the background. ZCode requests exempt from verification are unaffected. If verification is required, reopen the interface and complete it in the account’s Plan details. This setting applies only when closing minimizes the app to the tray.'),
  networkAccess: t('By default, only this machine can connect (127.0.0.1). Enabling this listens on all IPv4 interfaces (0.0.0.0); access depends on your network and firewall, is not limited to the LAN, and is not automatically opened to the internet. A panel administrator must be registered first. Management endpoints require an admin session or gateway key; API calls require an enabled gateway key, with a default key created automatically if needed. The web management panel is off by default. Enable it to sign in from another device\'s browser; disabling it does not affect API forwarding or local desktop management. Either switch requires an app restart.'),
  units: t('Controls how token counts read in reports and request logs: on shows Chinese magnitudes (1.2100M / 840010k), off uses k / M / B abbreviations (consistent with upstream docs and API fields). This only changes the display; no statistics or stored numbers are affected.'),
  queue: t('Applies only to queueing upstreams : when a model is busy the upstream raises no error, it just replies "suggest waiting N seconds"; the gateway waits for the suggested duration and resends the same request, and only when the wait count is exhausted is "queueing" returned as an error (HTTP 503, with text explaining that this is not a sign-in or quota problem). The wait happens before the first byte and comes out of the "Response Headers Timeout" budget — together the two settings determine how long one request can stall at most; queueing does not mark the account as rate-limited and does not switch accounts (everyone is queueing). Takes effect on the next request after saving, no restart needed.'),
  captchaPool: t('ZCode upstream settings determine whether model requests need verification, independently of plan claims. The background only checks settings and stock; it never runs verification or opens a puzzle automatically. Start required verification in the account’s Plan details. Tokens are single-use, expire shortly, and are not retried automatically after failure or cancellation.'),
  captchaSides: t('The two groups apply to the web panel hosted by the desktop app and the browser panel hosted by Docker / server. Access from other devices depends on network access and firewall settings and is not limited to the LAN. Human verification is enabled by default for both; keeping it enabled is recommended.'),
  timeouts: t('The upstream wait limit for each of the four request stages (seconds, 1–3600): (1) "Connect Timeout" = the maximum wait to establish the TCP/TLS connection or proxy tunnel, default 30 seconds; (2) "Response Headers Timeout" = the maximum time to wait for the upstream\'s response headers after the request is sent, default 300 seconds — gateway requests always use stream, and the response headers arrive as soon as the SSE stream is established, regardless of how long the model thinks; (3) "Stream Idle Timeout" = the maximum idle gap allowed between adjacent chunks of a streaming response, the timer restarting on each new chunk, default 300 seconds — if the upstream sends no data for a long time, the connection is judged dead and disconnected; (4) "Non-Streaming Timeout" = the total budget for reading a complete non-streaming response body (a single timer that is not reset), default 300 seconds. All four are wait limits: as long as data keeps arriving, a normal streaming answer keeps flowing. Changes take effect on the next request without a restart.'),
  retry: t('When forwarding a request to an upstream fails (transient server errors, connection failures, and WorkBuddy\'s 11-128 sensitive-word blocking), the gateway waits for an interval and sends it once more. The counts come in two tiers covering two different things: (1) "Same-Account Retries" resends in place on the same account a few times — usually the prompt was blocked by the upstream or the link flickered, and resending after a wait tends to succeed; (2) "Account-Switch Retries" is how many more accounts this request may try — the account that sent it first does not count, each switch costs one, regardless of whether it is the same provider (switching to the next account of the same provider also counts), and when the switches are exhausted and it still fails, the error is returned to the client. Setting both to 0 means no retries on failure — error out immediately. Note that account-level throttling (429) does not consume these account switches — that kind of failure goes through the "cool down the account and move to the next one" fallback, a separate path from the retries here. There is also the "Switch Accounts on Specified Error Codes" list (default 402): upstream status codes on the list are not retried on the same account; the gateway switches to the next account in the queue and keeps trying (the switch also consumes an "Account-Switch Retries" slot), and only when those are exhausted is the error returned to the client.'),
  routing: t('How accounts are picked when forwarding a request: ① "Routing strategy" sets the queueing rule — Strict order means position 1 takes every request until it can\'t answer (rate-limited / failing / out of quota) before the next one steps in; Load balancing rotates to the next account on every request, spreading the load across accounts. ② "Session stickiness" keeps the same session (same model + same opening messages) on one account wherever possible — multi-turn conversations reuse the upstream context cache, fewer account switches, less waste; Auto sticks for 5 minutes, In-session for 6 hours, Per turn and Off don\'t stick. The queue position (first column of the accounts table) is the foundation under every strategy; strategies are layered on top of it. Routing also has three self-healing guards: accounts that fail to forward back off for a while (starting at 15 minutes, doubling per failure, capped at 1 hour), accounts 429\'d despite having quota go to the back of the queue for 30 minutes, and accounts out of quota or over their per-period token cap are held by the limiter — the first two can be toggled below as needed; the limiter is governed by each account\'s "Limiter" settings.'),
  sanitize: t('Before forwarding, fingerprint masking applies rules in list order to message text, including conversation content, reasoning content and tool-call arguments. The default rules contain exact replacements and regular-expression deletions for client templates and billing headers. You can add, edit, delete, enable, disable and reorder rules; an empty replacement deletes the matched text. All providers are selected by default, or you can select specific providers. Only the outbound copy is modified; responses are unchanged. Click Save settings to apply changes to the next request. An empty rule list performs no fingerprint rewriting, and restoring defaults resets only the rules.'),
  developerRoleCompat: t('The newer OpenAI API carries instructions in a developer role instead of system, so conversations sent by current OpenAI SDKs and some clients contain messages with role "developer". Many OpenAI-compatible upstreams (especially the passthrough path of custom providers) do not recognize that role and reject the whole request with 400. When enabled, the gateway rewrites developer-role messages to system before every forward — the two roles are equivalent in OpenAI semantics (OpenAI\'s own migration guide maps them the same way), so the rewrite changes neither content nor meaning, and it is harmless to the built-in upstreams that only accept system anyway. Enabled by default; when off, developer messages go upstream as-is and upstreams that do not know the role return "unknown role"-style errors.'),
  reasoningCollapse: t('Let Codex collapse intermediate work after the final answer. Phase markers are added only to strictly identified Codex Responses requests, independently of report categories and client declarations. Only non-empty, normally completed replies without tool calls receive final_answer; text before tool calls receives commentary. Errors, truncation, missing completion signals and unknown finish reasons never receive a final-answer marker. Off by default; answer text, model reasoning and usage are unchanged.'),
  imageCompress: t('Large base64 images embedded in a conversation (screenshots, photos) both bloat the request body and raise upstream billing, which is computed from pixels. When enabled, the gateway scales images above the threshold down to the long-edge limit and re-encodes them as JPEG before forwarding — only the copy sent upstream is changed; the request log still stores the original you sent. When the same original image reappears across turns, the previous compressed result is reused instead of recomputed. Images referenced by http links, animated GIFs and SVGs are never touched; an image that would grow after re-encoding is kept as-is. Applies to chat only — image generation and editing are unaffected. On by default; turn the switch off if you do not want it.'),
  clineHeaders: t('The Cline upstream decides from the request headers whether the caller is one of Cline\'s own products: X-CLIENT-TYPE: cline-sdk is a hard requirement (without it, free-pool models always return 403), and the version number and platform identifier make the request look closer to the official client. Listed here are the headers the gateway sends by default: editing a row\'s value overrides the default (you can also add custom headers beyond the default list); saving an empty value means this header is not sent; deleting an edited row in the UI reverts it to the default. Click "Save" when done and the next Cline request picks it up immediately. Do not touch a row you do not understand — the value of spoofed headers lies in being word-for-word identical to the official client.'),
  cors: t('The gateway endpoints (/v1/*) do not answer browser cross-origin requests by default: a page connecting directly is stopped by the preflight and only reports "cannot connect to the API". Note that the allowed origin is * — with no gateway key configured, any web page could use this machine\'s gateway to hit upstreams, which is why the default is off; if you only use it yourself, serving the page from the same origin as the gateway (a local reverse proxy for /v1) is safer. The panel API (/api/*) is unaffected.'),
  prompt: t('Clients (CLI tools such as Claude Code / Codex) inject dozens of fixed template sentences into the system prompt, and the upstream reviews them by word-for-word matching — a hit blocks the whole request (HTTP 400). Fingerprint masking can rewrite the sentences observed to be hit, but a different client version may introduce new ones. Here you can replace the whole system prompt with the gateway\'s own one: (1) "Passthrough" = leave the client\'s system prompt untouched (the default, identical to the previous behavior); (2) "Replace" = delete all client system / developer messages and substitute the gateway\'s prompt (the client\'s project instructions disappear with them); (3) "Append" = insert one gateway prompt after the leading consecutive system blocks, leaving existing messages word-for-word intact (the client\'s instructions and the gateway prompt are used together). That prompt has two sources: a prompt file (leave empty for the built-in default), or editing the text right in the UI — an edited text takes precedence (over the file), and clearing it goes back to the file / built-in default. Also, in Passthrough / Append mode, if content blocking is hit (usually a fingerprint false positive), the gateway automatically retries once with a minimal neutral prompt and extends the "fallback period" to 00:00 the next day — during that period every request goes out with the neutral prompt directly, with no 400 hit first; you can see it here and clear it early. The items above are **defaults**: to configure a single provider, add a row under "Per-Provider Configuration" below (providers with "Gateway-Provided Prompt" are listed there by default) — different upstreams tolerate these differently.'),
  debug: t('When on, the gateway stores every forward\'s **request to the upstream** (headers + body) and the **upstream response** (status code + headers + body) locally in full, for the "Details" view on the request logs page. Credential fields in request headers — Authorization, Cookie, API Key, and the like — are always replaced with [redacted] and never hit the disk in plain text; request and response bodies are stored as-is (they may contain your conversations). Only the most recent 500 payloads are kept; the oldest are dropped beyond that. This is a temporary troubleshooting switch — keep it off when you don\'t need it.'),
  io: t('Export writes all accounts and custom provider definitions to a JSON file that can be copied to another machine and imported for continued use. Import merges: within the same provider, accounts are deduplicated by business identity (UID / userId / apiKey, etc.) — existing accounts only get their credentials updated and keep this machine\'s existing priority order; new accounts are appended to the end of the forwarding order and never displace the account currently in use; custom provider definitions are merged by id and created automatically when missing locally.'),
  backup: t('Save the database and required recovery keys in a password-encrypted .a2ab backup. Restore it on Windows, macOS, or Linux without manually copying keys. Restoration replaces accounts, configuration, logs, and desktop settings, and re-encrypts credentials with the destination keys without overwriting them. Exclude logs to reduce file size. Legacy .sql imports remain supported. A safety backup is created before replacement; sign in again afterward.'),
  retention: t('Logs and request logs are limited by both retention days and record count. Logs default to 30 days and 10,000 records, with capacity allocated proportionally by level; debug details are kept for at most 3 days. Request logs default to 99999 days and 1,000,000 records; excess records are removed oldest first together with their raw bodies. Daily aggregates default to 99999 days and are limited only by days; request log count cleanup does not affect them. Days accept 1–99999 and counts accept 1–100000000. Lowering any setting and confirming immediately removes excess data, which cannot be recovered.'),
  storage: t('Accounts, event logs, request records, debug payloads, and settings are stored in agent2api.db. Encryption keys are stored separately. To move between machines, use Full database backup / restore rather than copying only the database. To change the directory, set AGENT2API_PROXY_HOME and restart.'),
  modelsDev: t('models.dev is an open-source, community-maintained model database. It fills in what the upstream never told us: context window, max output, tool calling, images, video, reasoning support, and which reasoning levels the model actually accepts. Priority: values you edited by hand under "Model capabilities" > what the upstream catalog declares > models.dev — it only fills gaps the upstream left, and never overrides a declared value. A snapshot ships inside the app, so offline, first-run and blocked-network cases still have data; while enabled it refreshes online on the "Model catalog refresh" schedule (in the same round, or on demand here). Turning it off only stops the online refresh; existing data stays in use.'),
} as const

/** 面板底注（`.hint.retention-note`）与各面板内的说明行 */
export const NOTES = {
  timeouts: t('Takes effect immediately after saving, no restart needed; a timeout is treated as an upstream failure (counted in the "Error" column of the request logs, and retried according to the "Request Retries" settings).'),
  queue: t('While waiting, the status column in the request logs shows a "Queueing" phase, and the "Internal Retries" section of the detail dialog lists each wait duration; when the waits run out, HTTP 503 is returned (not 401/429: credentials are not refreshed and the account is not marked as rate-limited).'),
  captchaPool: t('Status polling settings take effect immediately after saving. No tokens are generated automatically. Start required verification in the account’s Plan details. You do not need to keep the interface open when model requests are exempt from verification.'),
  retry: t('Takes effect immediately after saving, no restart needed; if all attempts are exhausted and the request still fails, the error is returned to the client as-is.'),
  routing: t('Takes effect on save, no restart: strategy and stickiness apply to the **next** forwarded request; in-flight requests are unaffected.'),
  retryCodes: t('Failures matching these upstream status codes are not retried on the same account — the gateway switches to the next account and keeps trying (an account switch consumes an "Account-Switch Retries" slot; only when those are exhausted is the error returned to the client as-is). Type a status code from 100–599 and press Enter to add it; click × or press Backspace to delete one. Default: 402 (insufficient points). Clearing the list means any error is retried as usual.'),
  sanitize: t('Enabled for all providers by default. Edit rules or provider scope and click Save settings; the next request uses the changes without restarting. Rule hits are still recorded in the request log\'s masking tag.'),
  developerRoleCompat: t('Enabled by default. Only the role field changes — message content and order stay untouched; this only rewrites the copy sent upstream, and what the client sees never changes.'),
  reasoningCollapse: t('Off by default. Enabling takes effect on the next Codex request without a restart; phase markers are only added to Codex-origin /v1/responses requests, other clients and /v1/chat/completions are unaffected.'),
  imageCompress: t('On by default. Only the copy sent upstream is changed; the response and the original image seen by the client stay the same, and the request log stores the original image from the client rather than the compressed version. Applies to the chat path only — image generation and editing do not go through it. Changes take effect on the next request, no restart needed.'),
  clineHeaders: t('Rows whose value equals the default do not count as overrides (only changed keys are stored, not the whole configuration); saving an empty value means this header is not sent, and deleting an edited row reverts it to the default. Changes take effect immediately, no restart needed.'),
  cors: t('Off by default. Changes take effect immediately, no restart needed; while off, cross-origin requests are stopped by the preflight.'),
  debug: t('New requests save upstream debug payloads and downstream bodies after this is enabled; past requests cannot be backfilled. Only the most recent 500 upstream debug payloads are kept. Counts for both sides are shown under Data → Data storage. Turning this off does not delete saved payloads; choose Clear raw payloads only in request logs to clear both sides together.'),
  promptMode: t('Replace = delete all client system / developer messages and substitute the gateway\'s prompt; Append = keep the client\'s content and just insert one extra prompt after the leading system blocks. Both only change the copy sent upstream; the response the client sees is unchanged.'),
  promptFile: t('Path to a UTF-8 text file. Leave empty to use the built-in default prompt (a general coding-assistant prompt with no template sentences the upstream would block). Only applies in Replace / Append mode; passthrough mode does not read it.'),
  promptText: t('Edit the prompt text right here: after saving, this text takes precedence (over the prompt file above), and clearing it falls back to the file / built-in default. No need to open a file in an editor just to change one word.'),
  promptGatewayText: t('The upstream verifies identity by "structure": the three blocks must each be a separate block, so they are edited separately here (they cannot be merged into one). Runtime values in the dynamic block are written as placeholders and substituted only when a request is sent: {cwd} working directory, {platform} platform, {shell} shell, {os_version} OS version, {git} whether this is a git repository, {provider} provider identifier, {model} model name. Clearing a block = that block reverts to the official text. A blank line is automatically prepended to the third block when it is sent (the official shape).'),
  prompt: t('Takes effect immediately after the change, no restart needed (the next request uses the new mode). The fallback is an automatic temporary state; once cleared, this mode\'s own prompt is restored immediately.'),
  promptProviders: t('Only saved provider settings are listed here. Unconfigured providers follow the global settings and can be added manually from the dropdown. Saved modes, files and prompt text override the global settings; Follow Default removes only that provider\'s mode, file and text overrides. For ZCode\'s Start Plan, the plugin assembles three blocks by default and merges the client\'s system text into the third, without creating a replacement rule. Manually edited gateway prompt blocks and a disabled assembly switch still take precedence; acceptance depends on the upstream rules.'),
  modelsDev: t('The per-capability source marks in "Model management" (upstream / models.dev / overridden) come from this data, and the reasoning-level dropdown in the mapping dialog lists only the levels the model actually accepts, based on it.'),
  retention: t('Takes effect immediately after saving. Both retention days and record limits apply; lowering either immediately removes excess historical data and cannot be undone.'),
  storageFile: t('agent2api.db in the configuration directory (it holds accounts, logs, request records, debug payloads, and settings).'),
  storageSize: t('Total size of the database main file and WAL file, using the same calculation as the request cleanup dialog. After clearing data, vacuum the database to reclaim unused space.'),
  storage: t('Data is kept in agent2api.db in the configuration directory; to change the location, set the AGENT2API_PROXY_HOME environment variable and restart the app.'),
  captcha: t('When on, the login page solves the challenge in the background (invisible to humans) while every scripted attempt must compute one first — brute force and account squatting become far more expensive. It only affects panel sign-in / registration, not API keys used by clients. Desktop and web are independent: the desktop page mostly serves this machine and the LAN, while the web page (Docker / server) is exposed to the network and benefits more from staying on.'),
  panelLogin: t('This browser is currently signed in to the panel as an administrator. "Sign Out" revokes this device\'s session (automatic renewals within 30 days are voided too) and the account and password must be entered again; other signed-in devices are unaffected.'),
  exportDanger: t('The export file contains credentials such as accessToken / refreshToken / apiKey along with custom provider definitions, and can be used to sign in directly. Keep it safe; do not share it or upload it to a public location.'),
  backupDanger: t('The backup contains recovery keys for account credentials, gateway keys, GitHub tokens, and cloud sync credentials. The whole file is protected by the backup password. Store the file and password separately; without the password, restoration is impossible.'),
} as const

/** 状态行（`.settings-state`）的派生文案：与旧实现的赋值逐字一致 */
export const STATES = {
  appLoading: '—',
  appUnavailable: t('The main process did not return startup settings. Update the desktop app and try again.'),
  appTrayOn: t('Closing the window does not quit the app; forwarding keeps running in the background. To quit, use the tray menu'),
  appTrayOff: t('Closing the window quits the app, and background forwarding stops with it'),
  appLightOn: t('Lightweight mode is on: closing the window destroys the UI process (freeing about 200 MB of memory) while the gateway continues in the background; while the UI is destroyed, the ZCode plan-pool (Start Plan) channel is unavailable and affected requests will note the reason in the request log'),
  appLightOff: t('Lightweight mode off: closing the window only hides it, the UI process stays in memory, and it opens instantly'),
  unitsOn: t('Currently shown as "1.2100M / 840010k"-style Chinese magnitudes.'),
  unitsOff: t('Currently shown with k / M / B abbreviations (e.g. 1.20M / 8.4k / 1.2B).'),
  debugUnavailable: t('Could not read the debug mode settings. Please try again later.'),
  debugOn: t('Saving raw upstream payloads:'),
  debugOff: t('Off: no raw payloads are saved while forwarding.'),
  sanitizeUnavailable: t('Could not read the fingerprint masking settings. Please try again later.'),
  sanitizeOn: t('Stripping review fingerprints from outbound requests: header key/value pairs are deleted entirely, and template sentences get minimal rewrites.'),
  sanitizeOff: t('Off: client system templates go upstream as-is and may be falsely blocked by content review (400).'),
  roleCompatUnavailable: t('Could not read the developer role compatibility setting, please try again later'),
  roleCompatOn: t('Normalizing roles: developer-role messages are rewritten to system before forwarding.'),
  roleCompatOff: t('Off: developer-role messages are sent upstream as-is; upstreams that do not know the role return an error (400).'),
  reasoningCollapseUnavailable: t('Could not read the third-party reasoning collapse setting, please try again later'),
  reasoningCollapseOn: t('Enabled: add phase markers to normally completed Codex replies.'),
  reasoningCollapseOff: t('Disabled: no extra phase markers; the client controls how work is displayed.'),
  corsUnavailable: t('Could not read the gateway CORS settings. Please try again later.'),
  corsOn: t('Answering cross-origin requests: browsers can reach /v1/* directly cross-origin.'),
  corsOff: t('Off: browsers cannot reach the gateway cross-origin.'),
  modelsDevUnavailable: t('Could not read the models.dev metadata state. Please try again later.'),
  /** 数据来历：内嵌快照（发版时那份）/ 上次刷新结果 / 刚拉的 */
  modelsDevSourceBuiltin: t('Snapshot bundled with the app (the one from the last release)'),
  modelsDevSourceCache: t('Result of the last successful refresh (stored in the local database)'),
  modelsDevSourceNetwork: t('Fresh data just fetched from models.dev'),
  modelsDevEmpty: t('No data loaded yet.'),
  promptUnavailable: t('Could not read the system prompt settings. Please try again later.'),
  degradeUntilFallback: t('00:00 the next day'),
  // ── 显示分类（本次新增：不来自静态骨架，是新写的文案）──
  themeSystem: t('Currently following the operating system\'s light / dark setting; the UI follows along automatically when the system switches.'),
  themeLight: t('Currently fixed to Light mode; it does not follow the system.'),
  themeDark: t('Currently fixed to Dark mode; it does not follow the system.'),
  zoomWeb: t('In the web build, interface zoom is controlled by the browser itself (Ctrl + / Ctrl -, or the zoom entry in the browser menu); this setting is not adjustable.'),
  zoomDefault: t('Currently displayed at 100% (the default).'),
  languageHint: t('The interface language is stored only on this machine (localStorage). Switching reloads the whole page immediately and does not affect accounts, keys, or forwarding.'),
  groupTotalFilteredOn: t('Totals only add up accounts visible under the current filter; changing the filter or search changes the number.'),
  groupTotalFilteredOff: t('Totals cover the whole group (default): filtering and search do not change them, and filtered-out or disabled accounts still count.'),
} as const

// ── 以下成员从 HEAD 恢复 ──────────────────────────────
// settings-state.ts（上一版拆分前的整体状态层）仍 import 它们；并行改动
// 移走了定义但没改 import，构建因此断在这里。恢复定义而不是动那份 import，
// 等那边的迁移完成后可整体搬走。

export const ROUTE_STRATEGIES = [
  {
    value: 'order',
    label: t('Strict order'),
    tip: t('Position 1 takes every request until it can\'t answer (rate-limited / failing / out of quota) before the next one steps in — the same top-down behavior as today.'),
  },
  {
    value: 'rotate',
    label: t('Load balancing'),
    tip: t('Rotates to the next usable account on every request; a new session goes to whoever is least busy. For spreading usage across accounts instead of letting one carry everything.'),
  },
  {
    value: 'provider',
    label: t('Per-provider rotation'),
    tip: t('Providers still follow priority; only accounts within the same provider take turns. For preferring a few providers without pinning any single account.'),
  },
] as const

/** 会话粘性档位：同一会话的后续请求是否尽量跟住同一个账号（会话 = 模型 + 首条用户消息） */

export const ROUTE_STICKY_OPTIONS = [
  {
    value: 'auto',
    label: t('Auto'),
    tip: t('The same session tries to stay on one account for 5 minutes; pausing longer than 5 minutes starts a new session. The choice for most cases — fewer account switches in multi-turn chats, without pinning an account.'),
  },
  {
    value: 'session',
    label: t('In-session'),
    tip: t('The same session stays on one account for up to 6 hours; suited to marathon sessions — the upstream context cache gets reused much more.'),
  },
  {
    value: 'turn',
    label: t('Per turn'),
    tip: t('Every request is routed by strategy from scratch — same behavior as "Off", only the session records are kept.'),
  },
  {
    value: 'off',
    label: t('Close'),
    tip: t('No session detection: every request is routed by strategy from scratch.'),
  },
] as const

/** 路由档位的取值（SegmentedControl 的泛型参数用它，视图里不必再写字面量联合） */
export type RouteStrategyValue = (typeof ROUTE_STRATEGIES)[number]['value']
export type RouteStickyValue = (typeof ROUTE_STICKY_OPTIONS)[number]['value']

/**
 * 「路由」分类的三个自愈开关（后端 `/api/routing` 的布尔字段）。
 *
 * `key` 是后端字段名（PUT 的键），`defaultOn` 只用于界面提示「当前是默认值吗」，
 * 真正的兜底在后端。
 */
export const ROUTE_PROTECTION_SWITCHES = [
  {
    key: 'routingFailoverBackoff',
    label: t('Failure backoff'),
    defaultOn: true,
    tip: t('An account that fails to forward rests for a while before coming back: starting at 15 minutes, doubling per consecutive failure, capped at 1 hour; one success restores it immediately. The point is to let a failure cool off instead of hammering the same bad account request after request.'),
  },
  {
    key: 'routingSink',
    label: t('Rate-limit demotion'),
    defaultOn: true,
    tip: t('An account with quota left that gets 429\'d goes to the back of the queue and returns automatically after 30 minutes (one successful forward also brings it right back). This marks "the shared pool is probably being drained by someone else", not "this account is broken". ⚠️ Mind the interplay with the routing strategy: under "Per-provider rotation", the back of the queue means the whole provider gets pushed back — if one provider\'s accounts are rate-limited en masse, they all sink to the back and that provider may not be reached for a long time. Turn this off in that case, so 429\'d accounts are only bound by cooldown (they come back automatically when it expires).'),
  },
  {
    key: 'routingBreakerEnabled',
    label: t('Circuit breaker'),
    defaultOn: false,
    tip: t('Trips this account off entirely after consecutive failures reach a threshold (off by default). Division of labor with failure backoff: backoff means "let a single failure cool off" — light penalty, automatic recovery; the circuit breaker means "this account is misbehaving as a whole, stop trying it one request at a time" and takes several consecutive successes to close. While open, the account is skipped — unless it is the only one left, in which case it still gets one try.'),
  },
] as const

export type RouteProtectionKey = (typeof ROUTE_PROTECTION_SWITCHES)[number]['key']

export const CAPTCHA_POOL_FIELDS: NumberField[] = [
  {
    key: 'zcodeCaptchaPoolTarget',
    id: 'settings-captcha-pool-target',
    label: t('Stock target'),
    unit: t('total'),
    min: 1,
    max: 10,
    hint: t('Reference stock target: 3 by default, from 1 to 10. It only affects the stock display; falling below it never generates tokens or opens verification automatically.'),
  },
  {
    key: 'zcodeCaptchaPoolPollMs',
    id: 'settings-captcha-pool-poll-ms',
    label: t('Poll interval'),
    unit: t('ms'),
    min: 1000,
    max: 30000,
    hint: t('Stock polling interval while Start Plan requests are recent: 2500 ms by default, from 1000 to 30000. Polling slows down when idle and never triggers verification.'),
  },
  {
    key: 'zcodeCaptchaPoolMaxPerRound',
    id: 'settings-captcha-pool-max-per-round',
    label: t('Per-round mint cap'),
    unit: t('total'),
    min: 1,
    max: 5,
    hint: t('Kept for compatibility with older settings. Tokens are not minted automatically in batches. Each Start verification action obtains one token and stops on failure or cancellation.'),
  },
]

/** 「指定错误码直接换号」：键名、状态码范围与名单上限与后端 retry_api.rs 逐字同源 */

export type ModelsDevState = {
  /** 联网刷新开关（关闭后已装载的数据继续用，见后端 `KEY_MODELS_DEV_ENABLED`） */
  modelsDevEnabled?: boolean
  /** 这份索引的来历：`builtin`（内嵌快照）/ `cache`（上次刷新结果）/ `network`（刚拉的） */
  source?: string
  /** 这份数据的生成时刻（毫秒；0 = 未知） */
  updatedAt?: number
  /** 收录的模型条数 */
  count?: number
}

/** 立即刷新的一行结果（字段与各家目录刷新的结果行同形，见后端 `api::model_meta`）。 */
