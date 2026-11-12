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

/**
 * set「」（React ）。
 *
 * ── ──────────────────────────────────────────
 * use when is 「 / 1 use 」。 this 
 * 「 is 1 、 in 、 and in 、 is 」 1 
 * ，** and can 1 ** —— can ， can issue。
 *
 * ── Why after ， not `platform` ─────────────────────
 * `workbuddyDesktop.platform` （windows / macos / linux / web）。
 * 「 when 」 after ， use that when 
 * `platform` is `'web'`， after is —— before will 
 * Docker 。 in after （`core::system_info::deployment`）， this 
 * `deployment` + `platform` 1 。
 *
 * ── 「 use 」Why is requestparameter not is before ───────────────
 * this use is ****。if in ，「」
 * is 1 （ ）， —— use as ，
 * is path。request（`?hideUsername=1`），
 * after return path already is `~\.agent2api`。** not **： is 「 this 
 * need to 」 1 ， will down 。
 *
 * ── ────────────────────────────────────────────────────
 * is （ this 1 to ）， down by 「 / /
 * / / 」。all use etc —— 、path、 in etc 
 * down for ， OCR / for 。
 */

/* ─── and ───────────────────────────── */

type Group = { title: string; rows: Array<[string, React.ReactNode]> }

/** → （ after `deployment` ） */
const DEPLOYMENT_LABEL: Record<string, string> = {
  desktop: t('桌面端'),
  'windows-service': t('Windows 服务'),
  docker: t('Docker 容器'),
  systemd: t('systemd 服务'),
  foreground: t('前台运行'),
}

/** → */
const OS_LABEL: Record<string, string> = {
  windows: 'Windows',
  macos: 'macOS',
  linux: 'Linux',
}

/** → */
const INSTALLER_LABEL: Record<string, string> = {
  nsis: t('Windows 安装包（NSIS）'),
  dmg: 'macOS 安装包（dmg）',
  deb: t('Linux 安装包（deb）'),
  appimage: 'AppImage',
  server: t('服务端压缩包'),
}

function updateSupportLabel(deployment: NonNullable<SystemInfo['deployment']>): string {
  const capability = deployment.updateCapability
  if (!capability) return onOff(deployment.selfUpdate)
  return capability.installSupported ? onOff(true) : `${onOff(false)} · ${updateUnavailableReason(capability, t)}`
}

/** current（ before ， after not use in ） */
function interfaceLabel(): string {
  const platform = shared().workbuddyDesktop?.platform
  if (platform === 'web') return t('浏览器面板')
  return t('应用窗口')
}

/**
 * + 1 。
 *
 * not can `${OS_LABEL[os]} ${osVersion}`：Windows `ProductName`
 * "Windows"（`Windows 11 Pro`），Linux `PRETTY_NAME` 
 * （`Ubuntu 24.04 LTS`）—— 1 before 「Windows Windows 11 Pro」。
 * is 「 is already 」， is not 。
 */
function formatOs(os: string | null | undefined, version: string | null | undefined): string {
  const name = OS_LABEL[String(os)] || text(os)
  const detail = text(version)
  if (detail === '—') return name
  if (name !== '—' && detail.toLowerCase().startsWith(name.toLowerCase())) return detail
  return `${name} ${detail}`
}

/** already when → 「3 4 when 」 this can */
function formatUptime(seconds: number | null | undefined): string {
  const total = Number(seconds)
  if (!Number.isFinite(total) || total <= 0) return '—'
  const days = Math.floor(total / 86400)
  const hours = Math.floor((total % 86400) / 3600)
  const minutes = Math.floor((total % 3600) / 60)
  if (days > 0) return t('{d} 天 {h} 小时', { d: days, h: hours })
  if (hours > 0) return t('{h} 小时 {m} 分钟', { h: hours, m: minutes })
  if (minutes > 0) return t('{m} 分钟', { m: minutes })
  return t('不到 1 分钟')
}

/** when （ms）→ when ；0 / to 「—」 */
function formatTime(ms: number | null | undefined): string {
  const value = Number(ms)
  if (!Number.isFinite(value) || value <= 0) return '—'
  const date = new Date(value)
  const pad = (n: number) => String(n).padStart(2, '0')
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())} `
    + `${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}`
}

/** → 「 / 」；unknown to 「—」 */
function onOff(value: boolean | null | undefined): string {
  if (value === true) return t('开')
  if (value === false) return t('关')
  return '—'
}

/** any → can （empty / null 1 「—」） */
function text(value: unknown): string {
  if (value === null || value === undefined) return '—'
  const result = String(value).trim()
  return result || '—'
}

/* ─── ─────────────────────────────── */

type State = {
  status: 'loading' | 'ready' | 'unavailable'
  info: SystemInfo | null
  /** failureReason（empty = no has failure） */
  error: string
  /** 「 use 」： 1 （ in after ） */
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
 * 1 。
 *
 * failure not empty already has `info`： and update 1 to —— not will as 1 
 * unknown， up 1 + 1 failureReason「—」 has use 。
 */
async function load(): Promise<void> {
  const api = shared().workbuddyDesktop
  if (!api?.getSystemInfo) {
    publish({ status: 'unavailable', error: t('当前环境不支持读取版本信息（桥接方法缺失）') })
    return
  }
  if (snapshot.status !== 'ready') publish({ status: 'loading' })
  try {
    const info = await api.getSystemInfo({ hideUsername: snapshot.hideUsername })
    if (!info) {
      publish({ status: 'unavailable', error: t('后端没有返回版本信息') })
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

/** 「 use 」：update（ use to path） */
function toggleHideUsername(next: boolean): void {
  publish({ hideUsername: next })
  void load()
}

/* ─── ─────────────────────────── */

/**
 * current「 + 」 。
 *
 * use is ** already ** `info`（requestparameter，file）， this not need need 
 * path。 use `: ` not is ： GitHub issue after ，
 * in will 1 ， is 。
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

  section(t('版本'), [
    [t('软件版本'), text(app.version)],
    [t('开源许可'), text(app.license)],
    [t('源代码'), text(app.sourceUrl)],
    [t('版权声明'), text(app.copyright)],
    [t('更新通道'), app.channel === 'both' ? t('正式版与 beta 版') : app.channel === 'beta' ? t('仅 beta 版') : t('仅正式版')],
    [t('数据库 schema'), text(app.schemaVersion)],
    [t('库中 schema'), text(app.dbSchemaVersion)],
  ])
  section(t('环境'), [
    [t('运行形态'), DEPLOYMENT_LABEL[String(runtime.deployment)] || text(runtime.deployment)],
    [t('当前界面'), interfaceLabel()],
    [t('操作系统'), formatOs(runtime.os, runtime.osVersion)],
    [t('架构'), text(runtime.arch)],
    [t('WebView / 浏览器内核'), text(runtime.webviewVersion)],
    [t('进程 PID'), text(runtime.pid)],
    [t('启动时间'), formatTime(runtime.startedAt)],
    [t('已运行'), formatUptime(runtime.uptimeSeconds)],
    [t('开发构建'), onOff(runtime.debugBuild)],
  ])
  section(t('位置与端口'), [
    [t('监听地址'), `${text(deployment.host)}:${text(deployment.port)}`],
    [t('面板端口'), text(deployment.panelPort)],
    [t('托管网页界面'), onOff(deployment.uiHosted)],
    [t('数据目录'), text(deployment.configDir)],
    [t('数据库文件'), `${text(deployment.dbFile)}（${formatBytes(Number(deployment.dbBytes) || 0)}）`],
    [t('日志文件'), text(deployment.logFile)],
    [t('程序路径'), text(deployment.exePath)],
    [t('安装包形态'), INSTALLER_LABEL[String(deployment.installerKind)] || text(deployment.installerKind)],
    [t('面板自动安装'), updateSupportLabel(deployment)],
  ])
  section(t('安全'), [
    [t('管理员已注册'), onOff(security.adminRegistered)],
    [t('面板登录'), onOff(security.panelAuth)],
    [t('网关 Key 数量'), text(security.apiKeyCount)],
    [t('/v1 拒绝无 Key 请求'), onOff(security.v1FailClosed)],
    [t('未注册闸门'), onOff(security.panelGate)],
    [t('允许无 Key 裸跑'), onOff(security.allowNoKey)],
    [t('GitHub 令牌'), security.githubTokenFilled ? t('已填写') : t('未填写')],
    [t('更新出网线路'), security.updateProxy ? t('已指定出口') : t('直连')],
  ])
  section(t('开关'), [
    [t('调试模式'), onOff(features.debugMode)],
    [t('指纹脱敏'), onOff(features.sanitizeFingerprints)],
    [t('跨域访问'), onOff(features.cors)],
    [t('开发者角色兼容'), onOff(features.developerRoleCompat)],
    [t('保活换行丢弃'), onOff(features.keepaliveStrip)],
    [t('自动续写'), onOff(features.autoContinue)],
    [t('models.dev 元数据'), onOff(features.modelsDev)],
    [t('登录人机验证'), onOff(features.captchaEnabled)],
  ])
  section(t('计数'), [
    [t('账号'), `${text(stats.enabledAccounts)} / ${text(stats.accounts)} ${t('启用')}`],
    [t('限流中账号'), text(stats.limitedAccounts)],
    [t('模型'), text(stats.models)],
    [t('在途请求'), text(stats.inFlight)],
    [t('累计请求'), formatCount(stats.requests)],
    [t('事件日志'), formatCount(stats.logs)],
    [t('待迁移旧数据'), onOff(stats.upgradePending)],
  ])
  return lines.join('\n').trimEnd()
}

/** （ and clipboard.js 1 ： up down down execCommand） */
async function copyPlainText(): Promise<void> {
  const info = snapshot.info
  if (!info) {
    toast(t('还没有读到版本信息'), 'err')
    return
  }
  const payload = toPlainText(info)
  try {
    await navigator.clipboard.writeText(payload)
    toast(t('已复制全部版本信息'))
    return
  } catch {
    /* to down */
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
    if (ok) toast(t('已复制全部版本信息'))
    else toast(t('复制失败，请手动选择复制'), 'err')
  } catch {
    toast(t('复制失败，请手动选择复制'), 'err')
  }
}

/* ─── ─────────────────────────────────── */

/** 1 「 —— 」：、 etc can （path not to ） */
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

  // （root use ），set when by wbSystemInfoPane.load() 
  React.useEffect(() => { void load() }, [])

  const info = state.info
  const app = info?.app || {}
  const runtime = info?.runtime || {}
  const deployment = info?.deployment || {}
  const security = info?.security || {}
  const features = info?.features || {}
  const stats = info?.stats || {}

  const deploymentLabel = DEPLOYMENT_LABEL[String(runtime.deployment)] || text(runtime.deployment)
  // can can already （Windows ProductName / Linux PRETTY_NAME），
  // before 1 ，「Windows Windows 11」 this （ formatOs ）
  const osFull = formatOs(runtime.os, runtime.osVersion)
  const schemaMismatch = app.schemaVersion != null
    && app.dbSchemaVersion != null
    && app.schemaVersion !== app.dbSchemaVersion

  const groups: Group[] = [
    {
      title: t('版本'),
      rows: [
        [t('软件版本'), <span className='sysinfo-mono'>{text(app.version)}</span>],
        [t('开源许可'), text(app.license || 'AGPL-3.0-only')],
        [t('更新通道'), app.channel === 'both'
          ? <Badge variant='warning' shape='tag'>{t('正式版与 beta 版')}</Badge>
          : app.channel === 'beta'
            ? <Badge variant='warning' shape='tag'>{t('仅 beta 版')}</Badge>
            : <Badge variant='outline' shape='tag'>{t('仅正式版')}</Badge>],
        [t('数据库 schema'), (
          <span className='sysinfo-mono'>
            {text(app.schemaVersion)}
            {app.dbSchemaVersion != null ? ` / ${app.dbSchemaVersion}` : ''}
            {schemaMismatch
              ? <Badge variant='destructive' shape='tag' className='ml-2'>{t('不一致')}</Badge>
              : null}
          </span>
        )],
      ],
    },
    {
      title: t('环境'),
      rows: [
        [t('运行形态'), <Badge variant='brand'>{deploymentLabel}</Badge>],
        [t('当前界面'), text(interfaceLabel())],
        [t('操作系统'), osFull],
        [t('架构'), <span className='sysinfo-mono'>{text(runtime.arch)}</span>],
        [t('WebView / 浏览器内核'), <span className='sysinfo-mono'>{text(runtime.webviewVersion)}</span>],
        [t('进程 PID'), <span className='sysinfo-mono'>{text(runtime.pid)}</span>],
        [t('启动时间'), <span className='sysinfo-mono'>{formatTime(runtime.startedAt)}</span>],
        [t('已运行'), formatUptime(runtime.uptimeSeconds)],
        [t('开发构建'), onOff(runtime.debugBuild)],
        [t('详细日志'), onOff(runtime.verbose)],
      ],
    },
    {
      title: t('位置与端口'),
      rows: [
        [t('监听地址'), <span className='sysinfo-mono'>{`${text(deployment.host)}:${text(deployment.port)}`}</span>],
        [t('面板端口'), <span className='sysinfo-mono'>{text(deployment.panelPort)}</span>],
        [t('托管网页界面'), onOff(deployment.uiHosted)],
        [t('数据目录'), <span className='sysinfo-mono'>{text(deployment.configDir)}</span>],
        [t('数据库文件'), (
          <span className='sysinfo-mono'>
            {text(deployment.dbFile)}
            <span className='sysinfo-dim'>{`（${formatBytes(Number(deployment.dbBytes) || 0)}）`}</span>
            {deployment.dbAvailable === false
              ? <Badge variant='destructive' shape='tag' className='ml-2'>{t('不可用')}</Badge>
              : null}
          </span>
        )],
        [t('日志文件'), <span className='sysinfo-mono'>{text(deployment.logFile)}</span>],
        [t('程序路径'), <span className='sysinfo-mono'>{text(deployment.exePath)}</span>],
        [t('安装包形态'), INSTALLER_LABEL[String(deployment.installerKind)] || text(deployment.installerKind)],
        [t('面板自动安装'), updateSupportLabel(deployment)],
      ],
    },
    {
      title: t('安全'),
      rows: [
        [t('管理员已注册'), onOff(security.adminRegistered)],
        [t('面板登录'), onOff(security.panelAuth)],
        [t('网关 Key 数量'), <span className='sysinfo-mono'>{text(security.apiKeyCount)}</span>],
        [t('/v1 拒绝无 Key 请求'), onOff(security.v1FailClosed)],
        [t('未注册闸门'), onOff(security.panelGate)],
        [t('允许无 Key 裸跑'), onOff(security.allowNoKey)],
        [t('GitHub 令牌'), security.githubTokenFilled ? t('已填写') : t('未填写')],
        [t('更新出网线路'), security.updateProxy ? t('已指定出口') : t('直连')],
      ],
    },
    {
      title: t('开关'),
      rows: [
        [t('调试模式'), onOff(features.debugMode)],
        [t('指纹脱敏'), onOff(features.sanitizeFingerprints)],
        [t('跨域访问'), onOff(features.cors)],
        [t('开发者角色兼容'), onOff(features.developerRoleCompat)],
        [t('保活换行丢弃'), onOff(features.keepaliveStrip)],
        [t('自动续写'), features.autoContinue
          ? t('开（最多 {n} 轮）', { n: text(features.autoContinueMaxRounds) })
          : t('关')],
        [t('models.dev 元数据'), onOff(features.modelsDev)],
        [t('登录人机验证'), onOff(features.captchaEnabled)],
      ],
    },
    {
      title: t('计数'),
      rows: [
        [t('账号'), <span className='sysinfo-mono'>{`${text(stats.enabledAccounts)} / ${text(stats.accounts)}`}</span>],
        [t('限流中账号'), <span className='sysinfo-mono'>{text(stats.limitedAccounts)}</span>],
        [t('模型'), <span className='sysinfo-mono'>{text(stats.models)}</span>],
        [t('在途请求'), <span className='sysinfo-mono'>{text(stats.inFlight)}</span>],
        [t('累计请求'), <span className='sysinfo-mono'>{formatCount(stats.requests)}</span>],
        [t('事件日志'), <span className='sysinfo-mono'>{formatCount(stats.logs)}</span>],
        [t('待迁移旧数据'), onOff(stats.upgradePending)],
      ],
    },
  ]

  return (
    <>
      {/* ： + 。 in up is —— 
           this 1 to is 「 in 」。 */}
      <Card render={<section />} className='panel'>
        <CardHeader className='panel-head'>
          <h2>{t('版本信息')}</h2>
          <div className='head-actions'>
            <Button variant='outline' onClick={() => void load()} disabled={state.status === 'loading'}>
              {state.status === 'loading' ? t('读取中…') : t('刷新')}
            </Button>
            <Button variant='default' onClick={() => void copyPlainText()} disabled={!info}>
              {t('复制全部信息')}
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
            {t('出问题时把这一页截图，或点「复制全部信息」把文本贴进反馈，一般就能定位到大致原因。')}
          </div>

          <div className='mt-3 flex flex-wrap items-center gap-2 text-[12px] text-subtle'>
            <span>{app.copyright || 'Copyright (c) 2026 Agent2API'}</span>
            <Button variant='outline' size='xs' onClick={() => void openExternal(
              app.licenseUrl || 'https://github.com/aimod-cc/Agent2API/blob/main/LICENSE',
            )}>AGPL-3.0</Button>
            <Button variant='outline' size='xs' onClick={() => void openExternal(
              app.sourceUrl || 'https://github.com/aimod-cc/Agent2API',
            )}>{t('源代码')}</Button>
          </div>
          <p className='mt-1.5 text-[12px] text-subtle'>
            {t('本软件不提供任何担保，可按 AGPL-3.0 修改与分发；通过网络提供修改版服务时须向用户提供对应源码。')}
          </p>

          {/* path use can ： will to ， this is to use */}
          <div className='sysinfo-toggle'>
            <div className='sysinfo-toggle-text'>
              <span>{t('隐藏用户名')}</span>
              <span className='hint'>
                {t('把路径里的主目录（如 C:\\Users\\你的用户名）显示成 ~，截图分享时不暴露用户名。只影响本页显示与复制，不改任何文件。')}
              </span>
            </div>
            <Switch
              checked={state.hideUsername}
              onCheckedChange={toggleHideUsername}
              aria-label={t('隐藏用户名')}
            />
          </div>

          {state.error ? (
            <div className='detail' style={{ color: 'var(--danger)' }}>
              {t('读取失败：{message}', { message: state.error })}
            </div>
          ) : null}
        </CardContent>
      </Card>

      {/* ：、 1 （ CSS .sysinfo-groups）。
           when this 1 has 、， need ； after 1 can ，
           and 「 / / and 」 in 、
          「 / / 」 in —— is 「 this is 」 and 
          「 in 」。 */}
      <div className='sysinfo-groups'>
        {groups.map(group => (
          <GroupCard key={group.title} title={group.title} rows={group.rows} />
        ))}
      </div>
    </>
  )
}

/* ─── ：set `.settings-pane[data-cat="system"]` ─── */

const PANE_SELECTOR = '.settings-pane[data-cat="system"]'

let paneRoot: ReturnType<typeof createRoot> | null = null

/**
 * React root in pane up （ update-panel ： not ，
 * and its 1 ）。 not in （）。
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

/* ─── for ─────────────────────────────── */

declare global {
  interface Window {
    /** set「」： to this when by settings-state load() 1 */
    wbSystemInfoPane?: {
      load(): Promise<void>
    }
  }
}

window.wbSystemInfoPane = { load }
