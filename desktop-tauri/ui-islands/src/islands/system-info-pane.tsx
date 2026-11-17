import * as React from 'react'
import { mountOnPage } from '../island-lazy'
import { createRoot } from 'react-dom/client'
import { Badge, Button, Switch , Card, CardHeader, CardContent } from '@ui'
import {
  formatBytes,
  formatCount,
  shared,
  toast,
  type SystemInfo,
} from './settings-model'
import { t } from '../i18n'
import { openExternal } from './update-shared'
import { updateUnavailableReason } from '../../../ui-shared/updates'
import { ThirdPartyLicensesButton } from '../../../ui-shared/settings-third-party-licenses'

/**
 * 设置页「版本信息」面板（React 岛）。
 *
 * ── 它解决什么问题 ──────────────────────────────────────────
 * 用户出问题时最常见的开场是「截个图 / 说一句我用的哪版」。这个页面把
 * 「你装的是哪一版、跑在什么环境、数据与日志在哪、哪些开关是开的」一次
 * 摆全，**且能一键复制成纯文本** —— 截图只能看，文本能直接贴进 issue。
 *
 * ── 为什么环境判定读后端，不读 `platform` ─────────────────────
 * `workbuddyDesktop.platform` 只说明编译目标（windows / macos / linux / web）。
 * 桌面端开了「同时开放网页管理面板」后，别人用浏览器访问那个面板时
 * `platform` 是 `'web'`，但后端进程仍是桌面端 —— 只看前端会把它误报成
 * Docker 面板。真正的判据在后端（`core::system_info::deployment`），这里只
 * 负责把 `deployment` + `platform` 组合成一句人话。
 *
 * ── 「隐藏用户名」为什么是请求参数而不是前端替换 ───────────────
 * 这个页面的核心用途是**分享**。如果脱敏只做在渲染层，「复制全部信息」
 * 走的是另一条路（把数据序列化成文本），很容易漏掉 —— 用户以为隐藏了，
 * 实际复制出去的是完整路径。所以开关直接进请求（`?hideUsername=1`），
 * 后端返回的路径本身就已经是 `~\.agent2api`。开关**不落库**：它是「我这次
 * 要发给别人」的一次性决定，记住反而会让下次截图悄悄少信息。
 *
 * ── 布局 ────────────────────────────────────────────────────
 * 顶部是环境徽章（截图里最该被一眼看到的东西），下面按「版本 / 环境 /
 * 位置 / 状态 / 计数」分组。所有值用等宽字体 —— 版本号、路径、端口在等宽
 * 字体下更容易逐字核对，也更容易被 OCR / 人工比对。
 */

/* ─── 类型与常量 ───────────────────────────── */

type Group = { title: string; rows: Array<[string, React.ReactNode]> }

/** 运行形态 → 展示文案（后端 `deployment` 的取值集合） */
const DEPLOYMENT_LABEL: Record<string, string> = {
  desktop: t('Desktop app'),
  'windows-service': t('Windows service'),
  docker: t('Docker container'),
  systemd: t('systemd service'),
  foreground: t('Foreground process'),
}

/** 系统标识 → 展示文案 */
const OS_LABEL: Record<string, string> = {
  windows: 'Windows',
  macos: 'macOS',
  linux: 'Linux',
}

/** 安装包形态 → 展示文案 */
const INSTALLER_LABEL: Record<string, string> = {
  nsis: t('Windows installer (NSIS)'),
  dmg: 'macOS Install package (dmg)',
  deb: t('Linux package (deb)'),
  appimage: 'AppImage',
  server: t('Server archive'),
}

function updateSupportLabel(deployment: NonNullable<SystemInfo['deployment']>): string {
  const capability = deployment.updateCapability
  if (!capability) return onOff(deployment.selfUpdate)
  return capability.installSupported ? onOff(true) : `${onOff(false)} · ${updateUnavailableReason(capability, t)}`
}

/** 当前界面形态（前端补充，后端不知道用户在哪个窗口里） */
function interfaceLabel(): string {
  const platform = shared().workbuddyDesktop?.platform
  if (platform === 'web') return t('Browser panel')
  return t('App window')
}

/**
 * 系统名 + 版本拼成一句。
 *
 * 不能直接 `${OS_LABEL[os]} ${osVersion}`：Windows 的 `ProductName`
 * 自带 "Windows"（`Windows 11 Pro`），Linux 的 `PRETTY_NAME` 也自带发行版名
 * （`Ubuntu 24.04 LTS`）—— 再拼一次前缀就成了「Windows Windows 11 Pro」。
 * 判据是「版本描述是否已经以系统名开头」，是就不加。
 */
function formatOs(os: string | null | undefined, version: string | null | undefined): string {
  const name = OS_LABEL[String(os)] || text(os)
  const detail = text(version)
  if (detail === '—') return name
  if (name !== '—' && detail.toLowerCase().startsWith(name.toLowerCase())) return detail
  return `${name} ${detail}`
}

/** 已运行时长 → 「3 天 4 小时」这种可读文本 */
function formatUptime(seconds: number | null | undefined): string {
  const total = Number(seconds)
  if (!Number.isFinite(total) || total <= 0) return '—'
  const days = Math.floor(total / 86400)
  const hours = Math.floor((total % 86400) / 3600)
  const minutes = Math.floor((total % 3600) / 60)
  if (days > 0) return t('{d}d {h}h', { d: days, h: hours })
  if (hours > 0) return t('{h}h {m}m', { h: hours, m: minutes })
  if (minutes > 0) return t('{m}m', { m: minutes })
  return t('under a minute')
}

/** 时间戳（毫秒）→ 本地时间串；0 / 非法给「—」 */
function formatTime(ms: number | null | undefined): string {
  const value = Number(ms)
  if (!Number.isFinite(value) || value <= 0) return '—'
  const date = new Date(value)
  const pad = (n: number) => String(n).padStart(2, '0')
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())} `
    + `${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}`
}

/** 布尔开关 → 「开 / 关」；未知给「—」 */
function onOff(value: boolean | null | undefined): string {
  if (value === true) return t('On')
  if (value === false) return t('Off')
  return '—'
}

/** 任意值 → 可展示文本（空 / null 一律「—」） */
function text(value: unknown): string {
  if (value === null || value === undefined) return '—'
  const result = String(value).trim()
  return result || '—'
}

/* ─── 数据读取 ─────────────────────────────── */

type State = {
  status: 'loading' | 'ready' | 'unavailable'
  info: SystemInfo | null
  /** 本次读取的失败原因（空串 = 没有失败） */
  error: string
  /** 「隐藏用户名」：改了立即重拉一次（脱敏在后端） */
  hideUsername: boolean
}

const EMPTY: State = { status: 'loading', info: null, error: '', hideUsername: false }

let snapshot: State = EMPTY
const listeners = new Set<() => void>()

function publish(patch: Partial<State>): void {
  snapshot = { ...snapshot, ...patch }
  listeners.forEach(listener => listener())
}

function subscribe(listener: () => void): () => void {
  listeners.add(listener)
  return () => listeners.delete(listener)
}

function getSnapshot(): State {
  return snapshot
}

/**
 * 拉一次读数。
 *
 * 失败不清空已有的 `info`：与更新面板同一个取向 —— 环境信息不会因为一次
 * 网络抖动就变成未知，留着上一次的值 + 一句失败原因比整页「—」有用。
 */
async function load(): Promise<void> {
  const api = shared().workbuddyDesktop
  if (!api?.getSystemInfo) {
    publish({ status: 'unavailable', error: t('This environment cannot read version info (bridge method missing)') })
    return
  }
  if (snapshot.status !== 'ready') publish({ status: 'loading' })
  try {
    const info = await api.getSystemInfo({ hideUsername: snapshot.hideUsername })
    if (!info) {
      publish({ status: 'unavailable', error: t('The backend returned no version info') })
      return
    }
    publish({ status: 'ready', info, error: '' })
  } catch (error) {
    publish({
      status: 'unavailable',
      error: error instanceof Error ? error.message : String(error),
    })
  }
}

/** 拨「隐藏用户名」：先更新开关再重拉（让用户看到路径立刻变化） */
function toggleHideUsername(next: boolean): void {
  publish({ hideUsername: next })
  void load()
}

/* ─── 复制成纯文本 ─────────────────────────── */

/**
 * 把当前读数摊平成「分组 + 键值」的纯文本。
 *
 * 用的是**已经脱敏**的 `info`（开关走请求参数，见文件头），所以这里不需要
 * 再处理路径。行格式刻意用 `键: 值` 而不是表格：贴进 GitHub issue 后，
 * 表格语法在移动端会挤成一团，纯文本行反而是最好读的。
 */
function toPlainText(info: SystemInfo): string {
  const app = info.app || {}
  const runtime = info.runtime || {}
  const deployment = info.deployment || {}
  const security = info.security || {}
  const features = info.features || {}
  const stats = info.stats || {}

  const lines: string[] = []
  const section = (title: string, rows: Array<[string, string]>) => {
    lines.push(`[${title}]`)
    rows.forEach(([key, value]) => lines.push(`${key}: ${value}`))
    lines.push('')
  }

  section(t('Version'), [
    [t('Software version'), text(app.version)],
    [t('Open-source license'), text(app.license)],
    [t('Source code'), text(app.sourceUrl)],
    [t('Copyright notice'), text(app.copyright)],
    [t('Update channel'), app.channel === 'both' ? t('Stable and beta') : app.channel === 'beta' ? t('Beta only') : t('Stable only')],
    [t('Database schema'), text(app.schemaVersion)],
    [t('Schema in database'), text(app.dbSchemaVersion)],
  ])
  section(t('Environment'), [
    [t('Deployment'), DEPLOYMENT_LABEL[String(runtime.deployment)] || text(runtime.deployment)],
    [t('Current interface'), interfaceLabel()],
    [t('Operating system'), formatOs(runtime.os, runtime.osVersion)],
    [t('Architecture'), text(runtime.arch)],
    [t('WebView / browser engine'), text(runtime.webviewVersion)],
    [t('Process PID'), text(runtime.pid)],
    [t('Started at'), formatTime(runtime.startedAt)],
    [t('Uptime'), formatUptime(runtime.uptimeSeconds)],
    [t('Debug build'), onOff(runtime.debugBuild)],
  ])
  section(t('Location and ports'), [
    [t('Listen address'), `${text(deployment.host)}:${text(deployment.port)}`],
    [t('Panel port'), text(deployment.panelPort)],
    [t('Serves web UI'), onOff(deployment.uiHosted)],
    [t('Data directory'), text(deployment.configDir)],
    [t('Database file'), `${text(deployment.dbFile)}（${formatBytes(Number(deployment.dbBytes) || 0)}）`],
    [t('Log file'), text(deployment.logFile)],
    [t('Executable path'), text(deployment.exePath)],
    [t('Installer kind'), INSTALLER_LABEL[String(deployment.installerKind)] || text(deployment.installerKind)],
    [t('Automatic installation from panel'), updateSupportLabel(deployment)],
  ])
  section(t('Security'), [
    [t('Admin registered'), onOff(security.adminRegistered)],
    [t('Panel login'), onOff(security.panelAuth)],
    [t('Gateway key count'), text(security.apiKeyCount)],
    [t('/v1 rejects keyless requests'), onOff(security.v1FailClosed)],
    [t('Unregistered gate'), onOff(security.panelGate)],
    [t('Allow running without keys'), onOff(security.allowNoKey)],
    [t('GitHub Token'), security.githubTokenFilled ? t('Saved') : t('Not set')],
    [t('Update egress route'), security.updateProxy ? t('Pinned egress') : t('Direct')],
  ])
  section(t('Toggles'), [
    [t('Debug mode'), onOff(features.debugMode)],
    [t('Fingerprint sanitizing'), onOff(features.sanitizeFingerprints)],
    [t('CORS'), onOff(features.cors)],
    [t('Developer role compat'), onOff(features.developerRoleCompat)],
    [t('Collapse third-party model work after the answer'), onOff(features.thirdPartyReasoningCollapse)],
    [t('Strip keepalive newlines'), onOff(features.keepaliveStrip)],
    [t('Auto continuation'), onOff(features.autoContinue)],
    [t('models.dev metadata'), onOff(features.modelsDev)],
    [t('Login CAPTCHA'), onOff(features.captchaEnabled)],
  ])
  section(t('Counts'), [
    [t('Accounts'), `${text(stats.enabledAccounts)} / ${text(stats.accounts)} ${t('enabled')}`],
    [t('Rate-limited accounts'), text(stats.limitedAccounts)],
    [t('Model'), text(stats.models)],
    [t('In-flight requests'), text(stats.inFlight)],
    [t('Total requests'), formatCount(stats.requests)],
    [t('Event logs'), formatCount(stats.logs)],
    [t('Legacy data pending migration'), onOff(stats.upgradePending)],
  ])
  return lines.join('\n').trimEnd()
}

/** 复制（与 clipboard.js 同一套兜底：非安全上下文下 execCommand） */
async function copyPlainText(): Promise<void> {
  const info = snapshot.info
  if (!info) {
    toast(t('Version info has not been loaded yet'), 'err')
    return
  }
  const payload = toPlainText(info)
  try {
    await navigator.clipboard.writeText(payload)
    toast(t('Copied all version info'))
    return
  } catch {
    /* 落到下面的兜底 */
  }
  try {
    const area = document.createElement('textarea')
    area.value = payload
    area.style.position = 'fixed'
    area.style.opacity = '0'
    document.body.appendChild(area)
    area.select()
    const ok = document.execCommand('copy')
    area.remove()
    if (ok) toast(t('Copied all version info'))
    else toast(t('Copy failed, please select and copy manually'), 'err')
  } catch {
    toast(t('Copy failed, please select and copy manually'), 'err')
  }
}

/* ─── 视图 ─────────────────────────────────── */

/** 一行「键 —— 值」：键定宽、值等宽可折行（长路径不撑出横向滚动） */
function Row({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className='sysinfo-row'>
      <span className='sysinfo-key'>{label}</span>
      <span className='sysinfo-value'>{children}</span>
    </div>
  )
}

function GroupCard({ title, rows }: { title: string; rows: Array<[string, React.ReactNode]> }) {
  if (!rows.length) return null
  return (
    <Card render={<section />} className='panel'>
      <CardHeader className='panel-head'>
        <h2>{title}</h2>
      </CardHeader>
      <CardContent className='panel-body'>
        {rows.map(([label, value], index) => (
          <Row key={`${label}-${index}`} label={label}>
            {value}
          </Row>
        ))}
      </CardContent>
    </Card>
  )
}

function SystemInfoPanel() {
  const state = React.useSyncExternalStore(subscribe, getSnapshot)

  // 面板常驻（root 随应用生命周期），进设置页时由 wbSystemInfoPane.load() 重拉
  React.useEffect(() => { void load() }, [])

  const info = state.info
  const app = info?.app || {}
  const runtime = info?.runtime || {}
  const deployment = info?.deployment || {}
  const security = info?.security || {}
  const features = info?.features || {}
  const stats = info?.stats || {}

  const deploymentLabel = DEPLOYMENT_LABEL[String(runtime.deployment)] || text(runtime.deployment)
  // 系统描述本身可能已带系统名（Windows 的 ProductName / Linux 的 PRETTY_NAME），
  // 拼之前先判一次，避免「Windows Windows 11」这种重复（见 formatOs 的说明）
  const osFull = formatOs(runtime.os, runtime.osVersion)
  const schemaMismatch = app.schemaVersion != null
    && app.dbSchemaVersion != null
    && app.schemaVersion !== app.dbSchemaVersion

  const groups: Group[] = [
    {
      title: t('Version'),
      rows: [
        [t('Software version'), <span className='sysinfo-mono'>{text(app.version)}</span>],
        [t('Open-source license'), text(app.license || 'AGPL-3.0-only')],
        [t('Update channel'), app.channel === 'both'
          ? <Badge variant='warning' shape='tag'>{t('Stable and beta')}</Badge>
          : app.channel === 'beta'
            ? <Badge variant='warning' shape='tag'>{t('Beta only')}</Badge>
            : <Badge variant='outline' shape='tag'>{t('Stable only')}</Badge>],
        [t('Database schema'), (
          <span className='sysinfo-mono'>
            {text(app.schemaVersion)}
            {app.dbSchemaVersion != null ? ` / ${app.dbSchemaVersion}` : ''}
            {schemaMismatch
              ? <Badge variant='destructive' shape='tag' className='ml-2'>{t('Mismatch')}</Badge>
              : null}
          </span>
        )],
      ],
    },
    {
      title: t('Environment'),
      rows: [
        [t('Deployment'), <Badge variant='brand'>{deploymentLabel}</Badge>],
        [t('Current interface'), text(interfaceLabel())],
        [t('Operating system'), osFull],
        [t('Architecture'), <span className='sysinfo-mono'>{text(runtime.arch)}</span>],
        [t('WebView / browser engine'), <span className='sysinfo-mono'>{text(runtime.webviewVersion)}</span>],
        [t('Process PID'), <span className='sysinfo-mono'>{text(runtime.pid)}</span>],
        [t('Started at'), <span className='sysinfo-mono'>{formatTime(runtime.startedAt)}</span>],
        [t('Uptime'), formatUptime(runtime.uptimeSeconds)],
        [t('Debug build'), onOff(runtime.debugBuild)],
        [t('Verbose logs'), onOff(runtime.verbose)],
      ],
    },
    {
      title: t('Location and ports'),
      rows: [
        [t('Listen address'), <span className='sysinfo-mono'>{`${text(deployment.host)}:${text(deployment.port)}`}</span>],
        [t('Panel port'), <span className='sysinfo-mono'>{text(deployment.panelPort)}</span>],
        [t('Serves web UI'), onOff(deployment.uiHosted)],
        [t('Data directory'), <span className='sysinfo-mono'>{text(deployment.configDir)}</span>],
        [t('Database file'), (
          <span className='sysinfo-mono'>
            {text(deployment.dbFile)}
            <span className='sysinfo-dim'>{`（${formatBytes(Number(deployment.dbBytes) || 0)}）`}</span>
            {deployment.dbAvailable === false
              ? <Badge variant='destructive' shape='tag' className='ml-2'>{t('Unavailable')}</Badge>
              : null}
          </span>
        )],
        [t('Log file'), <span className='sysinfo-mono'>{text(deployment.logFile)}</span>],
        [t('Executable path'), <span className='sysinfo-mono'>{text(deployment.exePath)}</span>],
        [t('Installer kind'), INSTALLER_LABEL[String(deployment.installerKind)] || text(deployment.installerKind)],
        [t('Automatic installation from panel'), updateSupportLabel(deployment)],
      ],
    },
    {
      title: t('Security'),
      rows: [
        [t('Admin registered'), onOff(security.adminRegistered)],
        [t('Panel login'), onOff(security.panelAuth)],
        [t('Gateway key count'), <span className='sysinfo-mono'>{text(security.apiKeyCount)}</span>],
        [t('/v1 rejects keyless requests'), onOff(security.v1FailClosed)],
        [t('Unregistered gate'), onOff(security.panelGate)],
        [t('Allow running without keys'), onOff(security.allowNoKey)],
        [t('GitHub Token'), security.githubTokenFilled ? t('Saved') : t('Not set')],
        [t('Update egress route'), security.updateProxy ? t('Pinned egress') : t('Direct')],
      ],
    },
    {
      title: t('Toggles'),
      rows: [
        [t('Debug mode'), onOff(features.debugMode)],
        [t('Fingerprint sanitizing'), onOff(features.sanitizeFingerprints)],
        [t('CORS'), onOff(features.cors)],
        [t('Developer role compat'), onOff(features.developerRoleCompat)],
        [t('Collapse third-party model work after the answer'), onOff(features.thirdPartyReasoningCollapse)],
        [t('Strip keepalive newlines'), onOff(features.keepaliveStrip)],
        [t('Auto continuation'), features.autoContinue
          ? t('On (up to {n} rounds)', { n: text(features.autoContinueMaxRounds) })
          : t('Off')],
        [t('models.dev metadata'), onOff(features.modelsDev)],
        [t('Login CAPTCHA'), onOff(features.captchaEnabled)],
      ],
    },
    {
      title: t('Counts'),
      rows: [
        [t('Accounts'), <span className='sysinfo-mono'>{`${text(stats.enabledAccounts)} / ${text(stats.accounts)}`}</span>],
        [t('Rate-limited accounts'), <span className='sysinfo-mono'>{text(stats.limitedAccounts)}</span>],
        [t('Model'), <span className='sysinfo-mono'>{text(stats.models)}</span>],
        [t('In-flight requests'), <span className='sysinfo-mono'>{text(stats.inFlight)}</span>],
        [t('Total requests'), <span className='sysinfo-mono'>{formatCount(stats.requests)}</span>],
        [t('Event logs'), <span className='sysinfo-mono'>{formatCount(stats.logs)}</span>],
        [t('Legacy data pending migration'), onOff(stats.upgradePending)],
      ],
    },
  ]

  return (
    <>
      {/* 头部：环境徽章 + 三个动作。徽章放在最上面是刻意的 —— 截图里
          最该被一眼看到的就是「他跑在什么环境」。 */}
      <Card render={<section />} className='panel'>
        <CardHeader className='panel-head'>
          <h2>{t('Version info')}</h2>
          <div className='head-actions'>
            <Button variant='outline' onClick={() => void load()} disabled={state.status === 'loading'}>
              {state.status === 'loading' ? t('Loading…') : t('Refresh')}
            </Button>
            <Button variant='default' onClick={() => void copyPlainText()} disabled={!info}>
              {t('Copy all info')}
            </Button>
          </div>
        </CardHeader>
        <CardContent className='panel-body'>
          <div className='sysinfo-headline'>
            <Badge variant='brand'>{deploymentLabel}</Badge>
            <span className='sysinfo-headline-version'>
              {app.version ? `v${app.version}` : '—'}
            </span>
            <span className='sysinfo-headline-os'>
              {OS_LABEL[String(runtime.os)] || text(runtime.os)} {text(runtime.arch)}
            </span>
          </div>
          <div className='detail'>
            {t('When something goes wrong, screenshot this page or use "Copy all info" and paste the text into your report — that is usually enough to narrow down the cause.')}
          </div>

          <div className='mt-3 flex flex-wrap items-center gap-2 text-[12px] text-subtle'>
            <span>{app.copyright || 'Copyright (c) 2026 Agent2API'}</span>
            <Button variant='outline' size='xs' onClick={() => void openExternal(
              app.licenseUrl || 'https://github.com/aimod-cc/Agent2API/blob/main/LICENSE',
            )}>AGPL-3.0</Button>
            <Button variant='outline' size='xs' onClick={() => void openExternal(
              app.sourceUrl || 'https://github.com/aimod-cc/Agent2API',
            )}>{t('Source code')}</Button>
            <ThirdPartyLicensesButton t={t} openExternal={openExternal} />
          </div>
          <p className='mt-1.5 text-[12px] text-subtle'>
            {t('This software comes without warranty and may be modified and distributed under AGPL-3.0. Modified network services must offer their Corresponding Source to users.')}
          </p>

          {/* 路径里的用户名可以隐藏：截图会发到公开的地方，这是给用户的选择 */}
          <div className='sysinfo-toggle'>
            <div className='sysinfo-toggle-text'>
              <span>{t('Hide username')}</span>
              <span className='hint'>
                {t('Replace the home directory in the path (e.g. C:\\Users\\<user>\\) with\\Users\\your username) is shown as~, does not expose the username in screenshot sharing. Only affects the display and copy on this page; does not modify any file.')}
              </span>
            </div>
            <Switch
              checked={state.hideUsername}
              onCheckedChange={toggleHideUsername}
              aria-label={t('Hide username')}
            />
          </div>

          {state.error ? (
            <div className='detail' style={{ color: 'var(--danger)' }}>
              {t('Failed to read: {message}', { message: state.error })}
            </div>
          ) : null}
        </CardContent>
      </Card>

      {/* 分组卡片：宽屏左右两栏、窄屏自动退回一栏（见 CSS 的 .sysinfo-groups）。
          单栏时这一页有六张卡片、四十多行，要滚两屏多；两栏之后一屏能扫完，
          而且浏览器的自动平衡恰好把「版本 / 环境 / 位置与端口」放在左栏、
          「安全 / 开关 / 计数」放在右栏 —— 正好是「这台机器是什么」与
          「它现在怎么跑」两组语义。 */}
      <div className='sysinfo-groups'>
        {groups.map(group => (
          <GroupCard key={group.title} title={group.title} rows={group.rows} />
        ))}
      </div>
    </>
  )
}

/* ─── 挂载：接管设置页 `.settings-pane[data-cat="system"]` ─── */

const PANE_SELECTOR = '.settings-pane[data-cat="system"]'

let paneRoot: ReturnType<typeof createRoot> | null = null

/**
 * 把 React root 直接建在 pane 元素上（照 update-panel 的手法：不套宿主，
 * 布局与其它分类完全一致）。容器不存在（骨架变了）就静默跳过。
 */
function mountPanel(): void {
  if (paneRoot) return
  const pane = document.querySelector<HTMLElement>(PANE_SELECTOR)
  if (!pane) return
  pane.replaceChildren()
  paneRoot = createRoot(pane)
  paneRoot.render(<SystemInfoPanel />)
}

mountOnPage('settings', PANE_SELECTOR, mountPanel)

/* ─── 对外契约 ─────────────────────────────── */

declare global {
  interface Window {
    /** 设置页「版本信息」面板：切到该分类时由 settings-state 调 load() 重拉一次 */
    wbSystemInfoPane?: {
      load(): Promise<void>
    }
  }
}

window.wbSystemInfoPane = { load }
