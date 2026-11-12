import * as React from 'react'
import { mountOnPage } from '../island-lazy'
import { createRoot } from 'react-dom/client'
import { Badge, Button, Progress , Card, CardHeader, CardContent } from '@ui'
import {
  downloadAsset, errorMessage, handleExternalClick, isSelfUpdate, isServiceUpdate, isManualUpdate, manualUpdateReason, markdownHtml,
  missingAssetReasonFor, safeExternal, shared, toast, openExternal, serviceUpdateTask,
  type DownloadTask, type UpdateInfo,
} from './update-shared'
import { showUpdateModal } from './update-modal'
import { startServiceUpdate, watchServiceUpdate, serviceTaskActive, type ServiceProgress } from './update-systemd'
import { UpdateSettingsDialog } from './update-settings'
import { QqGroupDialog } from './update-qq-group'
import { SupportDialog } from './update-support-dialog'
import { t } from '../i18n'
import { HtmlContent } from '@ui'
import { Empty } from '@ui'



/* ─── ─────────────────────────────────── */

/** ： down 25MB ，1 seconds not will （ old ） */
const POLL_MS = 1000

/**
 * update（Linux headless / Docker） down and not ：
 * server 10MB ， 1 seconds 1 。
 */
const RESTART_SECONDS = 8

/** and ： 1-3 seconds， to to 10 seconds */
const HEALTH_RETRIES = 10
const HEALTH_INTERVAL_MS = 1000

/** （app.js renderTopbarStatus data-tone， its mirror ） */
type BadgeTone = '' | 'ok' | 'warn' | 'bad'

/** → Badge variant（ and old .badge.ok / .warn / .bad 1 1 for ） */
const BADGE_VARIANT: Record<BadgeTone, 'outline' | 'success' | 'warning' | 'destructive'> = {
  '': 'outline',
  ok: 'success',
  warn: 'warning',
  bad: 'destructive',
}

/** down / ： that by and 。
 * `restarting` has update will to （ → → new ）。 */
type DownloadPhase = 'idle' | 'downloading' | 'failed' | 'ready' | 'restarting'

/** ：React ， */
type Snapshot = {
  /** 1 **success**check （null = no success；update by ）。
   *
   * checkfailure when ** not empty** —— and update up 1 successcheck，valid，
   * is not new ；「 this Why no 」 by down checkError 。 */
  info: UpdateInfo | null
  /** current：** info**。
   *
   * checkfailure when info will up 1 success ，「current」 need 1 ：
   * （ when ， or after ）， 1 check no 
   * success when has —— / when not this is 1 not 。 */
  currentVersion: string
  /** up check when （0 = no has ）。 1 after that `checkedAt`： and when is 
   * 1 、 1 （ after 「 1 **success**check」， not ），
   * before not when —— that will minutes before 「 」 */
  checkedAt: number
  /** check failureReason（empty = no has failure）。
   *
   * and `info` ：checkfailure when `info` up 1 success （ and update
   * valid， is not new ）， this 1 「 this Why no 」。
   * down 1 successcheck（ or when ）empty。 */
  checkError: string
  /** （owner/repo），： and 「」 by 。
   * Note** not is ** `info.repository` —— that is in ，
   * Actions up no has ， etc */
  sourceRepository: string
  badgeText: string
  badgeTone: BadgeTone
  /** （ old #update-state） */
  stateText: string
  /** is by error（ old is style.color = var(--danger)） */
  stateError: boolean
  /** down 0-100； old in 0 < percent < 100 when */
  percent: number
  phase: DownloadPhase
  /** already path（empty = no has ）；empty when by 「 and / 」 */
  readyPath: string
  /** checkupdate in （ by 「check in …」 and use ） */
  checking: boolean
  /** down / request in ： by when use （ and old DOM disabled etc ） */
  actionBusy: boolean
  /** when （only phase === 'restarting' when has ） */
  restartSeconds: number
  /** 「GitHub no has 」： that 。
   * `null` = no to （ not ）；`false` = no has available（）。
   * `filled` this 1 ， and （ / ） by that */
  tokenFilled: boolean | null
}

const INITIAL_SNAPSHOT: Snapshot = {
  info: null,
  currentVersion: '',
  checkedAt: 0,
  checkError: '',
  sourceRepository: '',
  // and index.html 1 ： in load() before this 
  badgeText: t('未检查'),
  badgeTone: '',
  stateText: t('点击「检查更新」查询最新发布版本。'),
  stateError: false,
  percent: 0,
  phase: 'idle',
  readyPath: '',
  checking: false,
  actionBusy: false,
  restartSeconds: 0,
  // not 「」（ no to ， not ）： to false that 
  tokenFilled: null,
}

/* ─── ─────────────────────────────────── */
/* toast / errorMessage / markdownHtml / （openExternal、handleExternalClick）
 * in update-shared.ts —— use ， 1 。 */

/** 。 and logs-panel when 1 （ pad + when ）， not use 
 * toLocaleString： set， its when is */
function pad2(value: number): string {
  return String(value).padStart(2, '0')
}

/** / when → `YYYY-MM-DD HH:mm`（publishedAt is UTC ISO ， when ） */
function formatDateTime(value: unknown): string {
  const date = new Date(String(value ?? ''))
  if (!value || Number.isNaN(date.getTime())) return ''
  return `${date.getFullYear()}-${pad2(date.getMonth() + 1)}-${pad2(date.getDate())}`
    + ` ${pad2(date.getHours())}:${pad2(date.getMinutes())}`
}

/**
 * 「 up check」 when ： `HH:mm:ss`， up —— check / after no 
 * when ， when minutes will or before check ， this 1 is 「 up check is 
 * when 」。
 */
function formatClock(value: number): string {
  const date = new Date(value)
  if (!value || Number.isNaN(date.getTime())) return ''
  const today = new Date()
  const sameDay = date.getFullYear() === today.getFullYear()
    && date.getMonth() === today.getMonth()
    && date.getDate() === today.getDate()
  const clock = `${pad2(date.getHours())}:${pad2(date.getMinutes())}:${pad2(date.getSeconds())}`
  return sameDay ? clock : `${pad2(date.getMonth() + 1)}-${pad2(date.getDate())} ${clock}`
}

/**
 * ****：`dmg`（macOS，）/ `deb`（Linux， to 
 * ）/ `appimage`（Linux，file）/ `nsis`（Windows，）。
 *
 * ── by after to （checkUpdate installerKind）， not is ──────
 * is `isMacInstaller()` this 1 「 is not is dmg」 —— when use 
 * （ macOS by Windows UAC ）， Linux after none
 * ：deb not not ， use 1 is 。
 *
 * ── Why UA ─────────────────────────────────────────
 * load() 「 up down 」 that can can 1 checkUpdate（ that when 
 * info is null）， no has will macOS use to 「 will UAC 」 this in that 
 * up not in 。 has after when 1 after as 。
 */
type InstallerKind = 'dmg' | 'deb' | 'appimage' | 'nsis'

function installerKindOf(info: UpdateInfo | null): InstallerKind {
  const raw = info?.installerKind
  if (raw === 'dmg' || raw === 'deb' || raw === 'appimage' || raw === 'nsis') return raw
  // after no to （ / info as null）： by UA 1 —— Apple 1 dmg，
  // Linux deb（）， its by Windows nsis。
  const ua = navigator.userAgent || ''
  if (/Mac|iPhone|iPad/.test(ua)) return 'dmg'
  if (/Linux/.test(ua)) return 'deb'
  return 'nsis'
}

/**
 * need down and 「 is 」 in update-shared.ts
 * （isSelfUpdate / downloadAsset —— need use ， 1 ）。
 */

/** 「 down 」 when ： down will */
function downloadHint(info: UpdateInfo | null): string {
  if (isSelfUpdate(info)) {
    return t('正在下载服务端更新包…（下载完成后由面板完成替换，重启容器内的服务后生效）')
  }
  switch (installerKindOf(info)) {
    case 'dmg':
      return t('正在下载安装包…（下载完成后会挂载磁盘映像，把应用拖进「应用程序」即可完成安装）')
    case 'deb':
      return t('正在下载安装包…（下载完成后会交给系统安装器，按提示输入密码即可完成安装）')
    case 'appimage':
      return t('正在下载安装包…（下载完成后会启动新的 AppImage；把它替换掉旧文件即完成升级）')
    default:
      return t('正在下载安装包…（下载完成后启动安装程序需要管理员权限，会弹出 UAC 确认框）')
  }
}

/** 「 already 」 ： down 1 this */
function readyHint(info: UpdateInfo | null, name: string): string {
  if (isSelfUpdate(info)) {
    return t('更新包已就绪：{name}。点「安装并重启」后，面板会校验并替换容器内的程序与界面文件，然后重启服务（页面会自动刷新）。', { name })
  }
  switch (installerKindOf(info)) {
    case 'dmg':
      return t('安装包已就绪：{name}。点击「打开安装包」后会挂载磁盘映像，把应用拖进「应用程序」即可完成安装。', { name })
    case 'deb':
      return t('安装包已就绪：{name}。点击「打开安装包」后会交给系统安装器，按提示输入密码即可完成安装（本程序无需退出）。', { name })
    case 'appimage':
      return t('安装包已就绪：{name}。点击「安装并重启」后会启动新的 AppImage；把它移动到你的应用目录并替换旧文件即完成升级（本程序会先退出以释放文件占用）。', { name })
    default:
      return t('安装包已就绪：{name}。点击「安装并重启」后需要管理员权限，会弹出 UAC 确认框。', { name })
  }
}

/**
 * by 。macOS and Linux deb ** not **： and in no has file
 * （dmg is + ，deb is directory， in use directory），
 * and when by use —— Windows that will none in in 。
 */
function installButtonText(info: UpdateInfo | null): string {
  if (isSelfUpdate(info)) return t('安装并重启')
  const kind = installerKindOf(info)
  return kind === 'dmg' || kind === 'deb' ? t('打开安装包') : t('安装并重启')
}

/* ─── （ store）──────────────────── */

let snapshot: Snapshot = INITIAL_SNAPSHOT

/** （current has that 1 root） */
const subscribers = new Set<() => void>()

function getSnapshot(): Snapshot {
  return snapshot
}

function subscribe(listener: () => void): () => void {
  subscribers.add(listener)
  return () => { subscribers.delete(listener) }
}

/** and ：useSyncExternalStore use ，must new for */
function publish(patch: Partial<Snapshot>): void {
  snapshot = { ...snapshot, ...patch }
  for (const listener of subscribers) listener()
}

/**
 * 1 「GitHub no has 」， to that 。
 *
 * ── Why is `filled` this 1 ─────────────────────────────
 * after `/api/update/token` 「 has no has 、」（ not ，
 * core::update::token）， and that is 1 ， not will 。
 * this need is 「check is that 60 / when 」： and 
 * `GITHUB_TOKEN` （）， has no has this 。
 * not （file / no ） and no has when ， after to 
 * `filled` is false —— this ；**Reason**（「 new 1 can 
 * 」） in that ， this not Reason。
 *
 * ── when ──────────────────────────────────────────────
 * 、set（load）、 and 「updateset」 after —— in that 
 * （ / ）， this 1 down is 1 will this when 。failure****：
 * this 1 as 1 ， not this as 1 failure in （ or ） 1 ；
 * checkupdate that has error。
 */
async function refreshTokenHint(): Promise<void> {
  try {
    const status = await shared().workbuddyDesktop?.getUpdateToken()
    publish({ tokenFilled: status?.filled === true })
  } catch {
    /* not to （ to is not ） */
  }
}

/* ─── ─────────────────────────── */

/**
 * 「 1 1 」 （ old busy）。
 *
 * in not is ：openAndDownload and need **** to 
 * （ 1 down 、 and ）。 up use use checking / actionBusy
 * ， not 。
 */
let busy = false

/**
 * already path。
 *
 * down （「 down and 」 is need update， not this ），
 * none down is not is 、 in has no has 。 this ****：
 * UAC / no when in 「 already 」，（load） will to ，
 * no has this will 1 1 UAC。 1 after as to by ， by use 。
 */
let autoInstalledPath: string | null = null

/** down when ；null = no in */
let pollTimer: number | null = null

function stopPolling(): void {
  if (pollTimer !== null) {
    window.clearInterval(pollTimer)
    pollTimer = null
  }
}

/* ─── （ old setBadge / setState / renderXxx）── */

/**
 * checkfailure when ， up that **success when **（-1 = no has failure need ）。
 *
 * Why need need ：check / failure down when ， after 1 「request
 * …」 not `lastAttemptAt`（ that 1 no request， `check.rs` Deferred
 * ）。60 seconds after to no has 「failure」 —— by after ，
 * this failure will in down 1 。
 *
 * is 「 after success when this new 」， is ** after when **， not 
 * when （ when can can minutes）。
 */
let failedOnCheckedAt = -1

function setBadge(text: string, tone: BadgeTone = ''): void {
  publish({ badgeText: text, badgeTone: tone })
}

function setState(text: string, isError = false): void {
  publish({ stateText: text, stateError: isError })
}

/**
 * 1 check to up ：checksuccess after and 「 new set」 use 1 。
 *
 * old renderVersions / renderChangelog / toggleActions in file no has for 
 * —— need （ 、update、、 down by ） is info /
 * checkedAt / repository / phase ****，React by 。 this that 
 * 「 not is 」 ： and 。
 */
function acceptServiceProgress(progress: ServiceProgress): void {
  stopPolling()
  publish({ info: progress.info, currentVersion: String(progress.info.currentVersion || snapshot.currentVersion),
    percent: progress.percent, readyPath: '', phase: progress.active ? 'restarting' : progress.failed ? 'failed' : 'idle' })
  setBadge(progress.badge, progress.failed ? 'bad' : progress.active ? 'warn' : 'ok')
  setState(progress.text, progress.failed)
}

function renderCheckResult(): void {
  const { info, checkedAt, checkError } = snapshot
  if (!info) {
    // no has can old ：「 this Why no 」
    if (checkError) {
      setBadge(t('检查失败'), 'bad')
      setState(t('检查更新失败：{error}', { error: checkError }), true)
    }
    return
  }
  const serviceTask = serviceUpdateTask(info)
  if (serviceTask
    && (serviceTaskActive(serviceTask)
      || (serviceTask.phase === 'failed' && (!info.hasUpdate || serviceTask.version === info.latestVersion))
      || (!info.hasUpdate && serviceTask.version === info.currentVersion))) {
    watchServiceUpdate(info, acceptServiceProgress)
    return
  }
  const at = checkedAt ? t('（检查于 {time}）', { time: formatClock(checkedAt) }) : ''
  // already use 、： this 1 「 has new 」—— up already is new ，
  // 「 can down 」 will up （ after will 400 ）。
  // 「checkfailure」： this 1 need is 「 by 1 down 」， that is can ；
  // this checkfailure in after 
  if (info.pendingRestart === true) {
    setBadge(t('待重启'), 'warn')
    setState(checkError
      ? t('更新已应用到容器内，重启服务后生效。{at}（本次检查失败：{error}）', { at, error: checkError })
      : t('更新已应用到容器内，重启服务后生效。{at}', { at }))
    return
  }
  // checkfailure： by errorReason，** not ** info ——
  // and update up 1 successcheck， is valid， is not new 。
  // 1 「 up successcheck」， old this to 
  if (checkError) {
    setBadge(t('检查失败'), 'bad')
    setState(t('检查更新失败：{error}（以下版本与更新日志来自上次成功检查，检查于 {time}）', {
      error: checkError,
      time: formatClock(checkedAt) || t('未知'),
    }), true)
    return
  }
  if (info.hasUpdate === true) {
    setBadge(t('有新版本'), 'warn')
    if (isManualUpdate(info)) {
      const reason = manualUpdateReason(info)
      setState(t('发现新版本 {latest}（当前 {current}）。{reason}{at}', {
        latest: String(info.latestVersion || ''), current: String(info.currentVersion || ''), reason, at,
      }))
      return
    }
    // update： this no serverupdate when （ Release / ），
    // by not will —— Reason and ， for no has by 
    if (isSelfUpdate(info) && !info.serverAsset) {
      setState(t('发现新版本 {latest}（当前 {current}），但该版本没有提供容器内更新包（{reason}）；请在宿主机执行 docker compose pull && docker compose up -d 升级。{at}', {
        latest: String(info.latestVersion || ''),
        current: String(info.currentVersion || ''),
        reason: missingAssetReasonFor(info),
        at,
      }))
      return
    }
    // ： this 1 no （ / not for / update）
    // when by not ，must in this Reason and 「」 —— otherwise use for 
    // 「 new 」 none from down ， up its has down 
    if (!downloadAsset(info)?.url) {
      setState(t('发现新版本 {latest}（当前 {current}），但这一版没有提供本机安装包（{reason}）；可到 GitHub 发布页手动下载。{at}', {
        latest: String(info.latestVersion || ''),
        current: String(info.currentVersion || ''),
        reason: missingAssetReasonFor(info),
        at,
      }))
      return
    }
    setState(t('发现新版本 {latest}（当前 {current}）。{at}', {
      latest: String(info.latestVersion || ''),
      current: String(info.currentVersion || ''),
      at,
    }))
  } else if (info.hasUpdate === false) {
    setBadge(t('已是最新'), 'ok')
    setState(t('当前已是最新版本（{current}）。{at}', { current: String(info.currentVersion || ''), at }))
  } else {
    // hasUpdate as null：none（ is or tag ）
    setBadge(t('无法比较'), 'warn')
    setState(info.latestVersion
      ? t('最新发布版本为 {latest}，但当前版本号「{current}」无法解析，未做新旧判断。{at}', {
        latest: info.latestVersion,
        current: info.currentVersion || t('未知'),
        at,
      })
      : t('仓库暂无发布版本。{at}', { at }))
  }
}

/**
 * ** to 1 check**（ not request、 not ）。
 *
 * use ：app.js 60 seconds（pollUpdateStatus） and when 「」—— to 
 * after after `wbApp.updateUpdateBadge` to this ；file syncFromCache use （ from 
 * ， is 「 already in up 」）。 and 「 to update」 that ，
 * （ / update / check when ），** not need ** updateUpdateBadge
 * —— that is 。 from before no has this 1 when ， after when to in up not to ， use 
 * 1 「checkupdate」： can has ， is no 。
 */
function applyStatus(info?: UpdateInfo | null): void {
  if (info && serviceTaskActive(serviceUpdateTask(info))) {
    watchServiceUpdate(info, acceptServiceProgress)
    return
  }
  if (info?.channelChanged && info.checked === false) {
    failedOnCheckedAt = -1
    publish({ info: null, checkedAt: 0, checkError: '' })
    fillCurrentVersion(info)
    if (snapshot.phase === 'idle') {
      setBadge(t('未检查'))
      setState(t('更新通道已改变，请重新检查更新。'))
    }
    return
  }
  // checked === false： after no has success， not can empty up already has 
  if (!info || info.checked === false) {
    // 「current」 can ： this no successcheck after ，
    // use empty —— check（ when package_info） after 
    fillCurrentVersion(info)
    // from no success、failure： no has and can ， Reason。
    // up already has 1 **update** failure（failedOnCheckedAt -1） when not ——
    // that 「 need etc seconds」， after down up 1 failure has use 
    if (info?.lastError && failedOnCheckedAt === -1) {
      publish({ checkError: describeBackendError(info.lastError) })
      if (snapshot.phase === 'idle') renderCheckResult()
    }
    return
  }
  publish({ info, checkedAt: Number(info.checkedAt) || 0 })
  fillCurrentVersion(info)
  if (info.sourceRepository) applySourceRepository(info.sourceRepository)
  // ── failure and ────────────────────────────────────
  // not can lastError empty：check / failure down when ， after 
  // no this 1 （Deferred not request）， that will down 1 。
  // ：failure down 「 when that success when 」（failedOnCheckedAt）， after 
  // has after **update** success when —— is after when ，
  // not when （ when can can minutes）。
  // when that 1 failure by after ： 1 1 success、 and not in 。
  const successAt = Number(info.checkedAt) || 0
  if (failedOnCheckedAt !== -1) {
    // has failure in ： etc 「 1 success」， not use after old failure
    if (successAt > failedOnCheckedAt) {
      failedOnCheckedAt = -1
      publish({ checkError: '' })
    }
  } else {
    const failedBySchedule = info.checking !== true
      && !!info.lastError
      && Number(info.lastAttemptAt) > successAt
    if (failedBySchedule) {
      publish({ checkError: describeBackendError(info.lastError) })
    } else if (snapshot.checkError) {
      publish({ checkError: '' })
    }
  }
  // down in / already / down failure this down ， and down （、、failure
  // Reason）， 1 after not this ； and update is ， new 
  if (snapshot.phase === 'idle') renderCheckResult()
}

/**
 * after `lastError` → Reason。
 *
 * after in `RunGuard::finish` is `checkfailure：{Reason}`（ this 1 ），
 * already has 「checkupdatefailure：」 before ， will 「failure：checkfailure：」。
 */
function describeBackendError(value: string | null | undefined): string {
  return errorMessage(String(value || '').trim().replace(/^检查失败[：:]\s*/, ''))
}

/** （owner/repo）； not for not use ， 。
 * 1 is `sourceRepository`：`repository` is in ，
 * not is （ Snapshot ）。 */
function applySourceRepository(value: unknown): void {
  const repo = String(value || '').trim()
  if (!/^[\w.-]+\/[\w.-]+$/.test(repo)) return
  publish({ sourceRepository: repo })
}

/**
 * 「current」。update， no —— ** not empty**：
 * this 1 is ， not this 1 check 1 （ Snapshot ）。
 */
function rememberCurrentVersion(info: UpdateInfo | null | undefined): void {
  const version = String(info?.currentVersion || '').trim()
  if (version && version !== snapshot.currentVersion) publish({ currentVersion: version })
}

/**
 * in no has when 1 。
 *
 * after （`/api/update/status`、60 seconds、 ） is 
 * after ****； `checkUpdate` to is when package_info，
 * 1 after as 。 that empty，checksuccess when 。
 */
function fillCurrentVersion(info: UpdateInfo | null | undefined): void {
  if (!snapshot.currentVersion) rememberCurrentVersion(info)
}

/**
 * currentempty when ， after 1 。
 *
 * use in ：checkfailure after 。failure when `checkUpdate` is error， no has for can ，
 * `/api/update/status` is （ not 、 not ）， 
 * `currentVersion` after —— after no successcheck when has 。
 * already not ： that 1 not will as 1 failure。
 */
async function refreshCurrentVersion(): Promise<void> {
  if (snapshot.currentVersion) return
  try {
    fillCurrentVersion(await shared().workbuddyDesktop?.getUpdateStatus())
  } catch {
    /* after ：empty（「—」）， down set will */
  }
}

/* ─── down ───────────────────────────────────── */

/**
 * down → （ old renderTask，）。
 *
 * return false this not is in 、 no has （e.g. after empty），
 * 、。
 */
function renderTask(task: DownloadTask | null | undefined): boolean {
  if (!task) return false

  if (task.active) {
    const percent = Math.max(0, Math.min(100, Number(task.percent) || 0))
    const mb = (Number(task.received) || 0) / 1024 / 1024
    const totalMb = (Number(task.total) || 0) / 1024 / 1024
    publish({ phase: 'downloading', percent })
    setBadge(t('下载中'), 'warn')
    const text = task.selecting ? t('正在选择下载通道…') : t('正在下载 {name}：{percent}%（{received} / {total} MB）', {
      name: task.filename || t('安装包'),
      percent,
      received: mb.toFixed(1),
      total: totalMb.toFixed(1),
    })
    setState(task.source ? `${text} · ${t('当前下载通道：{source}', { source: task.source === '官方源' ? t('官方源') : task.source })}` : text)
    return true
  }

  stopPolling()
  publish({ percent: 0 })

  if (task.error) {
    setBadge(t('下载失败'), 'bad')
    setState(t('下载失败：{error}', { error: errorMessage(task.error) }), true)
    publish({ phase: 'failed' })
    return true
  }
  if (task.canceled) {
    setBadge(t('已取消'), 'warn')
    setState(isSelfUpdate(snapshot.info)
      ? t('下载已取消，可重新点击「下载并更新」。')
      : t('下载已取消，可重新点击「下载并安装」。'))
    // to idle： by 「 down and / down and update」（ old ， not ）
    publish({ phase: 'idle' })
    return true
  }
  if (task.done && task.path) {
    const path = task.path
    const name = task.filename || path
    setBadge(t('可安装'), 'ok')
    publish({ phase: 'ready', readyPath: path })
    // update：** not **。 down is as 「 down is need update」，
    // is 1 UAC； this will file and （、 need 
    // new ）， use by down that 1 down ， no use when 。
    if (isSelfUpdate(snapshot.info)) {
      setState(readyHint(snapshot.info, name))
      return true
    }
    // down （「 down in 、 when already down 」 this path）。 has 1 
    // path already 1 not —— that when in 「 already 」（UAC /
    // no ）， etc UAC。
    if (path !== autoInstalledPath) {
      autoInstalledPath = path
      // down after by to （deb to 、dmg 、
      // AppImage / NSIS ）
      const kind = installerKindOf(snapshot.info)
      setState(kind === 'dmg'
        ? t('安装包已下载完成，正在挂载磁盘映像…')
        : kind === 'deb'
          ? t('安装包已下载完成，正在打开系统安装器…')
          : kind === 'appimage'
            ? t('安装包已下载完成，正在启动新的 AppImage…')
            : t('安装包已下载完成，正在启动安装程序（需要管理员权限，会弹出 UAC 确认框）…'))
      void install(path)
    } else {
      setState(readyHint(snapshot.info, name))
    }
    return true
  }
  return false
}

/**
 * 1 。
 *
 * and old 1 ** has **： as empty（ after / ） when ， not is 
 * old that seconds 1 —— old renderTask(null) return false， no 
 * ， will 1 empty to 。 after 1 ， use available by 。
 */
async function pollProgress(): Promise<void> {
  try {
    const task = await shared().workbuddyDesktop?.updateProgress()
    if (!task) {
      stopPolling()
      return
    }
    renderTask(task)
  } catch (error) {
    stopPolling()
    setState(t('读取下载进度失败：{error}', { error: errorMessage(error) }), true)
  }
}

function startPolling(): void {
  stopPolling()
  pollTimer = window.setInterval(() => { void pollProgress() }, POLL_MS)
}

/**
 * ：；Windows up by ，file use 。
 *
 * return restart is ** by **：Windows before must；macOS dmg
 * and Linux deb not need need （ not this ） —— this not ， by 
 * （otherwise macOS / Linux use will etc 1 not will ）。
 */
async function install(path: string): Promise<void> {
  const api = shared().workbuddyDesktop
  if (!api) return
  try {
    const result = await api.runInstaller(path, true)
    const willRestart = result?.restart !== false
    // not when by ：dmg is 「 use 」，deb is can 
    const noRestartHint = installerKindOf(snapshot.info) === 'deb'
      ? t('已打开系统安装器，请在它弹出的窗口里完成安装。安装完成后重新打开本程序即可。')
      : t('安装包已挂载，请在弹出的窗口里把应用拖进「应用程序」完成安装。安装完成后重新打开本程序即可。')
    setState(willRestart
      ? t('安装程序已启动，本程序将退出以便完成覆盖安装。安装程序需要管理员权限，会弹出 UAC 确认框，请选择「是」。')
      : noRestartHint)
  } catch (error) {
    setBadge(t('启动失败'), 'bad')
    // UAC when return already 「 can new and 」， this ， not 
    // —— use can success（ by in ， phase === 'ready'）
    setState(t('启动安装程序失败：{error}', { error: errorMessage(error) }), true)
  }
}

/* ─── update（Linux headless / Docker）────── */

/**
 * and ：server and file → request → when + → new 。
 *
 * and install() not ： that is to （UAC / dmg），
 * this serverfile、 after by restart —— before 「 etc 
 * 」 this 1 。 when and sub2api ：，
 * and its use for ， not new 「 new already in 」 that 1 。
 */
async function installSelfUpdate(): Promise<void> {
  const api = shared().workbuddyDesktop
  if (!api?.installUpdate) return
  try {
    setBadge(t('安装中'), 'warn')
    setState(t('正在校验并替换容器内的程序与界面文件…'))
    const result = await api.installUpdate()
    const version = String(result?.version || '').trim()
    setState(version
      ? t('已替换为 {version}，正在重启服务…', { version })
      : t('替换完成，正在重启服务…'))
    // request response can can as not to （ ）—— that not is failure，
    // when can ， this 
    await api.restartService?.().catch(() => undefined)
    beginRestartCountdown(version)
  } catch (error) {
    setBadge(t('安装失败'), 'bad')
    setState(t('安装失败：{error}', { error: errorMessage(error) }), true)
  }
}

/** when ： */
function beginRestartCountdown(version: string): void {
  stopPolling()
  publish({ phase: 'restarting', restartSeconds: RESTART_SECONDS, readyPath: '' })
  setBadge(t('重启中'), 'warn')
  let seconds = RESTART_SECONDS
  const tick = (): void => {
    setState(version
      ? t('{version} 已就位，服务正在重启，页面会在恢复后自动刷新…（{seconds}s）', {
        version,
        seconds: Math.max(0, seconds),
      })
      : t('服务正在重启，页面会在恢复后自动刷新…（{seconds}s）', { seconds: Math.max(0, seconds) }))
    if (seconds <= 0) {
      window.clearInterval(timer)
      void waitForServiceThenReload()
      return
    }
    seconds -= 1
    publish({ restartSeconds: seconds })
  }
  const timer = window.setInterval(tick, 1000)
  tick()
}

/**
 * after new 。
 *
 * use （`getUpdateStatus`） not is fetch /health：（AGENT2API_PANEL_PORT）
 * down and not in 1 ，/health in that up ； is 
 * /api/*，。 not new —— already ， new is use need 
 * that 1 ， in up in old 。
 */
async function waitForServiceThenReload(): Promise<void> {
  for (let attempt = 0; attempt < HEALTH_RETRIES; attempt += 1) {
    try {
      const probe = await shared().workbuddyDesktop?.getUpdateStatus()
      if (probe) {
        window.location.reload()
        return
      }
    } catch {
      /* no ： etc down 1 */
    }
    await new Promise<void>(resolve => { window.setTimeout(resolve, HEALTH_INTERVAL_MS) })
  }
  window.location.reload()
}

/* ─── （ for ）────────────────── */

/** checkupdate： use 1 request（ after and failure） */
async function check(): Promise<UpdateInfo | null> {
  if (busy) return null
  busy = true
  publish({ checking: true })
  setBadge(t('检查中'), 'warn')
  setState(t('正在查询 GitHub 上的最新发布版本…'))
  let result: UpdateInfo | null = null
  try {
    const api = shared().workbuddyDesktop
    if (!api) throw new Error(t('后端桥不可用'))
    const info = await api.checkUpdate()
    result = info ?? null
    // when after return as ： this 1 down can can in when up ， use when will 
    // minutes before 「 」
    publish({ info: result, checkedAt: Number(result?.checkedAt) || Date.now(), checkError: '' })
    failedOnCheckedAt = -1
    rememberCurrentVersion(result)
    if (result?.sourceRepository) applySourceRepository(result.sourceRepository)
    renderCheckResult()
  } catch (error) {
    result = null
    // failure：**** up 1 success （ / updatevalid， is not new ），
    // by checkError this failure Reason。empty will in / when 
    // 「 up to is 1 」 not to —— that is this not this 。
    // down 「failure when up that success when 」： after after update success when ，
    // （ failedOnCheckedAt ）。 no has when -1 —— that 
    // down up no has can old ， down success can 
    failedOnCheckedAt = snapshot.checkedAt
    publish({ checkError: errorMessage(error) })
    renderCheckResult()
    // 「current」 not empty（ is ）： already ； no has 
    // （ will no successcheck） when 1 —— not this no has 
    void refreshCurrentVersion()
  } finally {
    busy = false
    publish({ checking: false })
  }
  // ： has has new ，failure and 「 already is new 」。
  // failure（result as null） when ** not ** —— that will up 1 successcheck 「 new 」，
  // that and no has （ up ）。 old null is as failure will empty
  // ，must 1 ； in not ， need 1 。
  if (result) shared().wbApp?.updateUpdateBadge?.(result)
  showUpdateModal(result, openAndDownload)
  return result
}

/**
 * ：** after ** 1 check， not GitHub。
 *
 * by after when （ in 「updateset」，）。 and when use 1 
 * ， this to is 「 1 successcheck」 when and ， and none。 before 
 * 1 is ，：dev down 1 need ，
 * （60 / when ， by IP ）， after when 1 failure。 not ，
 * **set 1 **（ load）。 no has when by 「check」 —— in 
 * no has when ： already 1 ，empty。
 */
async function syncFromCache(): Promise<void> {
  let cached: UpdateInfo | null = null
  try {
    cached = (await shared().workbuddyDesktop?.getUpdateStatus()) ?? null
  } catch {
    /* after ： up already has ， not empty */
  }
  if (cached) {
    // has no has success to applyStatus：「current」、failureReason and 
    // 「 already 」 this ， 1 
    applyStatus(cached)
  }
  if (cached?.channelChanged) {
    shared().wbApp?.updateUpdateBadge?.(cached)
    return
  }
  if (cached && cached.checked !== false) {
    // and check() 1 ： when to new when can 
    shared().wbApp?.updateUpdateBadge?.(cached)
    // this path is 「 when new → 1 」 1 （ old app.js ）
    showUpdateModal(cached, openAndDownload)
    return
  }
  // no has success check（ or failure）：current can from this ——
  // after in successcheck when will up ， this 1 not this 「check」 1 empty
  fillCurrentVersion(cached)
  // up already has （ old / ， or 1 failureReason），empty
  if (snapshot.info || snapshot.checkError) return
  setBadge(t('未检查'))
  setState(t('点击「检查更新」查询 GitHub 上的最新发布版本。'))
}

/**
 * that by ： down / ， and after 。
 *
 * old toggleActions： has update and has can down when to 。「 down in /
 * already 」 down —— that when info use can or （ old 
 * 「 not toggleActions」 1 ， this ）。
 */
async function downloadOrCancel(): Promise<void> {
  if (busy) return
  const current = getSnapshot()
  if (serviceTaskActive(serviceUpdateTask(current.info))) return
  if (isManualUpdate(current.info)) {
    const url = downloadAsset(current.info)?.url || current.info?.pageUrl
    if (url) await openExternal(url)
    return
  }
  if (isServiceUpdate(current.info) && current.info) {
    busy = true
    publish({ actionBusy: true })
    try { await startServiceUpdate(current.info, acceptServiceProgress) }
    catch (error) { setState(t('无法开始更新：{error}', { error: errorMessage(error) }), true) }
    finally { busy = false; publish({ actionBusy: false }) }
    return
  }

  // update already use to 、（ can can in up 1 after ：
  // 、request no ， or use in when new ）—— this 1 
  if (current.info?.pendingRestart === true && current.phase !== 'downloading') {
    busy = true
    publish({ actionBusy: true })
    try {
      await shared().workbuddyDesktop?.restartService?.().catch(() => undefined)
      beginRestartCountdown('')
    } finally {
      busy = false
      publish({ actionBusy: false })
    }
    return
  }

  // already has update： by in this 1 is 「 and / 」。
  // update installSelfUpdate（ + + ）， install（ to ）
  if (current.phase !== 'downloading' && current.readyPath) {
    busy = true
    publish({ actionBusy: true })
    try {
      if (isSelfUpdate(current.info)) await installSelfUpdate()
      else await install(current.readyPath)
    } finally {
      busy = false
      publish({ actionBusy: false })
    }
    return
  }

  // in down when by 「 down 」
  if (current.phase === 'downloading') {
    try {
      await shared().workbuddyDesktop?.cancelUpdate()
      setState(t('正在取消下载…'))
    } catch (error) {
      toast(t('取消失败：{error}', { error: errorMessage(error) }), 'err')
    }
    return
  }

  const asset = downloadAsset(current.info)
  if (!asset?.url) {
    // （ in up not to this 1 ）： **Why** no has 1 and ——
    // 1 「 no has can down 」 etc is no 、 is 
    const reason = missingAssetReasonFor(current.info)
    toast(isSelfUpdate(current.info)
      ? t('这一版没有提供容器内更新包：{reason}', { reason })
      : t('没有可下载的安装包：{reason}', { reason }), 'err')
    return
  }
  // by not can not this from this down （openAndDownload this ）
  if (!downloadButton(current)) return

  busy = true
  publish({ actionBusy: true })
  try {
    await shared().workbuddyDesktop?.downloadUpdate({ url: asset.url, name: asset.name })
    // new 1 down ： up 1 「 already 」 —— 1 new down after in 
    // down when 
    autoInstalledPath = null
    publish({ phase: 'downloading', readyPath: '', percent: 0 })
    setBadge(t('下载中'), 'warn')
    setState(downloadHint(current.info))
    startPolling()
  } catch (error) {
    setBadge(t('下载失败'), 'bad')
    setState(t('下载失败：{error}', { error: errorMessage(error) }), true)
  } finally {
    busy = false
    publish({ actionBusy: false })
  }
}

/** （set when by settings-panel.js use ） */
async function load(): Promise<void> {
  // and ： by current 1 ， when not will 1 etc 
  renderCheckResult()

  // that 「GitHub Token 」 ：、 not ，set 1 
  // （ and down and ， not ）
  void refreshTokenHint()

  // 1 after that 1 （ when by 、check 1 ）：****，
  // not as 「 will already has 」 —— after to new 、 new check when need to 
  // up （， not ， by syncFromCache）。 in down before ： down that 
  // will and down ， 「 down 」
  await syncFromCache()
  if (isServiceUpdate(snapshot.info) || isManualUpdate(snapshot.info)) return

  // has no has up down （ when not ）
  try {
    const task = await shared().workbuddyDesktop?.updateProgress()
    if (task?.active) {
      // （ autoInstalledPath ）： this not 「 will 
      // 」 and 「 up 」， down has 1 
      renderTask(task)
      startPolling()
      return
    }
    if (task?.done && task.path) {
      renderTask(task)
      return
    }
  } catch {
    /* after ： by check */
  }
}

/**
 * 「 to update」「update」 when ： already has check， and after 
 * ** down **。
 *
 * Why not in this 1 check()： 、update is that checkUpdate ，
 * 1 1 ， can can 「 has new 、 to no has 」 not 1 。
 * ，。
 *
 * already has / already down when not down ， to load() has —— down will 
 * in down 。
 */
async function openAndDownload(checkedInfo?: UpdateInfo | null): Promise<void> {
  // already use 、： no has can down —— to that 「」，
  // not is down 1 （ after will 400 ）
  if (checkedInfo?.pendingRestart === true) {
    await downloadOrCancel()
    return
  }
  if (checkedInfo?.hasUpdate === true) {
    // when use after to that 1 ： this is after 1 check（ ），
    // use when will when before check「 」—— need is up check when 
    publish({ info: checkedInfo, checkedAt: Number(checkedInfo.checkedAt) || snapshot.checkedAt })
    // empty： this can can after （）， up already has if 
    // check（ when package_info）， after as 
    fillCurrentVersion(checkedInfo)
    if (checkedInfo.sourceRepository) applySourceRepository(checkedInfo.sourceRepository)
    renderCheckResult()
    shared().wbApp?.updateUpdateBadge?.(checkedInfo)
    // already 「update」 to set， this use will 「 already in set」（ old path）
    showUpdateModal(checkedInfo, openAndDownload)
  }
  await load()
  // already has down in or already ： that down by is 「 down 」 and 「 and 」，
  // 1 
  const current = getSnapshot()
  if (current.phase === 'downloading' || current.readyPath) return
  if (!current.info?.hasUpdate) return
  // downloadOrCancel has `if (busy) return`， busy in ** **check / down in when 
  // as （ after when check to ， in this ， this 1 down will ， to 
  // is 「 no 」）。 etc ， to in that when ； use ——
  // by in up ， use 1 down can 。
  for (let attempt = 0; attempt < 3; attempt += 1) {
    if (busy) {
      await new Promise<void>(resolve => { window.setTimeout(resolve, 400) })
      continue
    }
    await downloadOrCancel()
    return
  }
}

/* ─── 1 ：set ─────────────── */

/** up that by ；null = not （ 1 ， by file） */
function downloadButton(snap: Snapshot): { label: string; disabled: boolean } | null {
  if (serviceTaskActive(serviceUpdateTask(snap.info))) return { label: t('更新进行中'), disabled: true }
  if (snap.info?.hasUpdate && isManualUpdate(snap.info)) {
    return { label: downloadAsset(snap.info)?.url ? t('下载服务端包') : t('查看发布页'), disabled: snap.actionBusy }
  }
  if (snap.info?.hasUpdate && isServiceUpdate(snap.info) && downloadAsset(snap.info)?.url) {
    return { label: t('一键更新'), disabled: snap.actionBusy }
  }
  // when not to by ： no has （ up need new ）
  if (snap.phase === 'restarting') return null
  // already use 、： by （ = ， downloadOrCancel ）
  if (snap.info?.pendingRestart === true && snap.phase === 'idle') {
    return { label: t('立即重启'), disabled: snap.actionBusy }
  }
  // has update and has can down when to by ： asset（），
  // serverAsset（server）—— this no for when 
  const actionable = snap.info?.hasUpdate === true && !!downloadAsset(snap.info)?.url
  if (!actionable && snap.phase !== 'downloading' && snap.phase !== 'ready') return null
  const label = snap.phase === 'downloading'
    ? t('取消下载')
    : snap.phase === 'failed'
      ? t('重试下载')
      : snap.phase === 'ready'
        ? installButtonText(snap.info)
        : isSelfUpdate(snap.info) ? t('下载并更新') : t('下载并安装')
  return { label, disabled: snap.actionBusy }
}

/** update（ new 1 ）： and 「 in GitHub 」 in 1 ， not 。 */
function changelogCard(snap: Snapshot): React.ReactNode {
  const info = snap.info
  const notes = String(info?.notes || '').trim()
  if (!notes) {
    return (
      <div className='update-log-hint'>
        {info ? t('这个版本没有填写发布说明。') : t('点击「检查更新」后，这里会显示最新版本的更新说明。')}
      </div>
    )
  }
  const html = markdownHtml(notes)
  const pageUrl = safeExternal(info?.pageUrl)
  const published = formatDateTime(info?.publishedAt)
  return (
    <div className='rel-note'>
      {/* ： + + 「 in GitHub 」+ 。
           this not is —— is （.rel-note-body 46vh），
           1 ， that need to ， etc no has 。 not and ，
           in 1 。 */}
      <div className='rel-note-head'>
        <span className='rel-tag'>{String(info?.latestVersion || t('最新版本'))}</span>
        {/* old .badge.tag.warn → Badge tag + warning */}
        {info?.prerelease === true ? <Badge shape='tag' variant='warning'>{t('预发布')}</Badge> : null}
        {pageUrl ? (
          <a
            className='rel-head-link'
            href={pageUrl}
            data-external={pageUrl}
            target='_blank'
            rel='noopener'
            title={t('在浏览器中打开 GitHub 发布页')}
          >
            {t('在 GitHub 查看完整说明')}
          </a>
        ) : null}
        {published ? <span className='rel-date'>{published}</span> : null}
      </div>
      <div className='rel-note-body'>
        {/* is wbMarkdown HTML（ use ， not ）： can 
            dangerouslySetInnerHTML。 by React ， / / is 
            ， is old that esc() in this not need need 。 */}
        {html ? <HtmlContent className='md-body' html={html} /> : (
          <div className='md-body'>
            <Empty className='md-body-empty'>{t('这个版本没有填写发布说明。')}</Empty>
          </div>
        )}
      </div>
    </div>
  )
}

/** 。root in `.settings-pane[data-cat="about"]` up ， is 
 * that `.panel` —— and 1 1 for ， is */
function UpdatePanel() {
  const snap = React.useSyncExternalStore(subscribe, getSnapshot)
  const button = downloadButton(snap)
  // 「updateset」 ： is （Dialog Portal body），
  // in that （ update-settings.tsx ）， not 
  const [settingsOpen, setSettingsOpen] = React.useState(false)
  // 「」： in 「」 that 1 by ， in this 1 
  // （ Portal body， and 1 React can ）
  const [qqGroupOpen, setQqGroupOpen] = React.useState(false)
  const [supportOpen, setSupportOpen] = React.useState(false)

  // （root use ）， not will ； 1 will （ / 
  // ）， —— when will 1 by 1 seconds after 
  React.useEffect(() => () => { stopPolling() }, [])
  // that 「GitHub Token 」 （ after by load and new ）
  React.useEffect(() => { void refreshTokenHint() }, [])

  // and to ****： in down ，
  // that no has no has issue， not can 「」 
  const repo = snap.sourceRepository ? `https://github.com/${snap.sourceRepository}` : ''
  const owner = snap.sourceRepository ? `https://github.com/${snap.sourceRepository.split('/')[0]}` : ''
  const selfUpdate = isSelfUpdate(snap.info)
  const latestLabel = snap.info?.latestVersion
    ? (snap.info.prerelease
      ? t('{version}（预发布）', { version: snap.info.latestVersion })
      : snap.info.latestVersion)
    : '—'
  const tag = String(snap.info?.latestVersion || '').trim()
  const logMeta = tag
    ? `${tag}${snap.checkedAt ? t(' · 检查于 {time}', { time: formatClock(snap.checkedAt) }) : ''}`
    : ''

  return (
    <Card render={<section />} className='panel' onClick={handleExternalClick}>
      <CardHeader className='panel-head'>
        <h2>{t('软件更新')}</h2>
        <span
          className='tip-q'
          data-tip={snap.info?.updateCapability?.deployment === 'windows-service'
            ? t('Windows 服务一键更新会核验官方更新包，再由独立升级进程停止服务、备份替换并重新启动。账号与配置保留，确认新版运行后页面自动刷新。')
            : snap.info?.updateCapability?.deployment === 'systemd'
            ? t('systemd 一键更新由受限升级程序完成：下载、核验官方校验值、备份替换、重启并确认版本。网关保持普通账号运行，账号与配置不变。')
            : isManualUpdate(snap.info) ? manualUpdateReason(snap.info) : selfUpdate
            ? t('容器内更新：更新包从 GitHub 发布页下载，校验 sha256 后由面板替换容器内的程序与界面文件，再重启服务（容器由 restart 策略拉起，页面会自动刷新）。更新的是容器里的文件，compose 里的镜像标签不变 —— 之后执行 docker compose pull && up -d 会使用镜像里的版本。检查更新与下载的出网线路、GitHub 令牌都在「更新设置」里。')
            : t('更新包从 GitHub Release 获取。自动模式按实际请求选择可用通道，失败后换源；镜像安装包必须匹配官方校验值，取不到时仅使用原始源。加速源、代理和令牌在「更新设置」中配置，指定代理失效不会静默直连。')}
        ></span>
        {/* id ：app.js renderTopbarStatus by id and （
            data-tone ；set that before not ， 1 no ） */}
        <Badge id='update-badge' variant={BADGE_VARIANT[snap.badgeTone]} data-tone={snap.badgeTone}>
          {snap.badgeText}
        </Badge>
        <div className='head-actions'>
          {/* 「 GitHub 」 （ in no has available when ，）：
               60 / when 、 5000，「checkupdate」 by in —— in this 
               has ；「updateset」 can ， 1 。
               after filled this 1 （ refreshTokenHint）， to and not 
               its ：null（ no to ） not ， 1 down */}
          {snap.tokenFilled === false ? (
            <span
              className='text-[12px] font-semibold text-warning'
              title={t('填写后检查限额 60 → 5000 次/小时（创建令牌无需勾选任何权限）')}
            >
              {t('GitHub Token 未填写')}
            </span>
          ) : null}
          {/* 「updateset」： + GitHub in （update-settings.tsx），
              「checkupdate」 —— and check， */}
          <Button
            id='btn-update-settings'
            variant='outline'
            title={t('更新设置')}
            onClick={() => setSettingsOpen(true)}
          >
            {t('更新设置')}
          </Button>
          {/* old class="primary" → variant="default"；none class → variant="outline" */}
          <Button id='btn-update-check' variant='outline' disabled={snap.checking || snap.phase === 'restarting'} onClick={() => void check()}>
            {snap.checking ? t('检查中…') : t('检查更新')}
          </Button>
          {button ? (
            <Button
              id='btn-update-download'
              variant='default'
              disabled={button.disabled}
              onClick={() => void downloadOrCancel()}
            >
              {button.label}
            </Button>
          ) : null}
        </div>
      </CardHeader>
      <CardContent className='panel-body'>
        {/* current → new 1 ， not down when .update-version wrap */}
        <div className='update-version'>
          <span className='version-pill'>
            <span className='k'>{t('当前版本')}</span>
            <span className='v' id='update-current'>{snap.currentVersion || '—'}</span>
          </span>
          <span className='version-arrow'>→</span>
          <span className='version-pill new'>
            <span className='k'>{t('最新版本')}</span>
            <span className='v' id='update-latest'>{latestLabel}</span>
          </span>
        </div>

        {/* error use （ old is style.color）：.detail is ，
             no has can */}
        <div className='detail' id='update-state' style={snap.stateError ? { color: 'var(--danger)' } : undefined}>
          {snap.stateText}
        </div>

        {/* ： old in 0 < percent < 100 when ， this */}
        {(snap.phase === 'downloading' || serviceUpdateTask(snap.info)?.phase === 'downloading') && snap.percent > 0 && snap.percent < 100
          ? <Progress value={snap.percent} />
          : null}

        <div className='update-author' id='update-author'>
          <div className='update-author-text'>
            <strong>{t('关于作者')}</strong>
            {owner && repo ? (
              <span>
                {t('本工具由')}{' '}
                <a id='update-author-link' href={owner} data-external={owner} target='_blank' rel='noopener'>
                  {snap.sourceRepository.split('/')[0]}
                </a>{' '}
                {t('个人开发并开源维护，免费使用，也不会收集你的账号数据。如果它帮到了你，欢迎到项目仓库点个 Star 支持一下。')}
              </span>
            ) : (
              <span>{t('Agent2API 一直在更新，是因为有人在背后持续投入。如果它帮到了你，欢迎请作者喝杯咖啡。')}</span>
            )}
          </div>
          <div className='update-author-actions'>
            <Button
              type='button'
              size='sm'
              variant='outline'
              title={t('QQ 群 / Telegram 都在这里')}
              onClick={() => setQqGroupOpen(true)}
            >
              {t('加入交流群')}
            </Button>
            {owner && repo ? (
              <Button
                id='btn-update-favorite'
                type='button'
                size='sm'
                variant='outline'
                data-external={repo}
                title={t('在浏览器中打开项目仓库，点 Star 支持作者')}
              >
                {t('收藏项目')}
              </Button>
            ) : null}
            <Button
              id='btn-update-support'
              type='button'
              size='sm'
              variant='outline'
              onClick={() => setSupportOpen(true)}
            >
              <svg viewBox='0 0 24 24' width='14' height='14' fill='none' stroke='currentColor' strokeWidth='1.8' strokeLinecap='round' strokeLinejoin='round' aria-hidden='true'>
                <path d='M20.8 4.6a5.5 5.5 0 0 0-7.8 0L12 5.7l-1.1-1.1a5.5 5.5 0 0 0-7.8 7.8L12 21l8.8-8.6a5.5 5.5 0 0 0 0-7.8Z' />
              </svg>
              {t('支持作者')}
            </Button>
          </div>
        </div>

        {/* update： new 1 ，「checkupdate」 
            （ and up 「 new 」） */}
        <div className='update-log' id='update-notes'>
          <div className='update-log-head'>
            <span className='update-log-title'>{t('更新日志')}</span>
            <span className='update-log-meta' id='update-log-meta'>{logMeta}</span>
          </div>
          <div className='update-log-body' id='update-log-body'>
            {changelogCard(snap)}
          </div>
        </div>
      </CardContent>

      {/* 「updateset」：（Dialog Portal body， in section 
           is ， not ）；， in 。
           when 1 ： in this / ， that need */}
      {settingsOpen ? (
        <UpdateSettingsDialog
          open
          onClose={() => {
            setSettingsOpen(false)
            void refreshTokenHint()
          }}
        />
      ) : null}
      {/* 「」：QQ （ can ）+ Telegram */}
      {qqGroupOpen ? <QqGroupDialog open onClose={() => setQqGroupOpen(false)} /> : null}
      {supportOpen ? <SupportDialog open onClose={() => setSupportOpen(false)} /> : null}
    </Card>
  )
}

/* ─── ： index.html has set ─── */

const PANE_SELECTOR = '.settings-pane[data-cat="about"]'

let paneRoot: ReturnType<typeof createRoot> | null = null

/**
 * React root in `.settings-pane[data-cat="about"]` up （ not ）。
 *
 * （ that 1 `.panel`）： down JSX will by new ，
 * will and React 。createRoot not ， replaceChildren()；
 * class / data-cat by settings-panel.js ，。
 *
 * not in （）：， to set when is no has this 1 
 * ， not will set。
 */
function mountPanel(): void {
  if (paneRoot) return
  const pane = document.querySelector<HTMLElement>(PANE_SELECTOR)
  if (!pane) return
  pane.replaceChildren()
  paneRoot = createRoot(pane)
  paneRoot.render(<UpdatePanel />)
}

// to when （ island-lazy）： not in ， not 。
mountOnPage('settings', PANE_SELECTOR, mountPanel)

/* ─── ： for ─────────────────────────── */

declare global {
  interface Window {
    /** update（ ui/update-panel.js， and 1 ） */
    wbUpdatePanel?: {
      /** settings-panel.js to set when use */
      load(): Promise<void>
      /** 「checkupdate」 by */
      check(): Promise<UpdateInfo | null>
      /** after （ not ） */
      syncFromCache(): Promise<void>
      /** 「update」/ when check new after ： and down */
      openAndDownload(info?: UpdateInfo | null): Promise<void>
      /** to check（app.js 60 seconds、 when 「」）：
       * 。 not request、 not 、 not —— that use that */
      applyStatus(info?: UpdateInfo | null): void
      /** fileadd（ old this in app.js maybeShowUpdateModal）： already 「
       * / will already / already in set」 ， use checkUpdate /
       * getUpdateStatus can 。app.js when （pollUpdateStatus）
       * if wbApp.updateUpdateBadge， need need in that to this ，otherwise「 after 
       * 1 」 will */
      showUpdateModal(info?: UpdateInfo | null): void
    }
  }
}

window.wbUpdatePanel = {
  load, check, syncFromCache, openAndDownload, applyStatus,
  showUpdateModal: (info) => showUpdateModal(info, openAndDownload),
}
