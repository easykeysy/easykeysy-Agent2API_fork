/**
 * 「update」（update-panel.tsx） and 「updateset」（update-settings.tsx）
 * use ** and **（：`.ts` not import.meta.glob ）。
 *
 * this file Reason： and （update-settings / update-modal） is 
 * React （ Portal to body）， 1 window 。 if declare 1 
 * `SharedWindow` / `UpdateBridge`， and will not 1 TS2717
 * （ accounts-shared.ts is 1 by ）。 this ** 1 **：、、
 * and in this 。
 *
 * （ / / ） this ： already not down 
 * （「updateset」 by ）， this has use ， not React ，
 * in this can update-settings.tsx 「 and 」。
 *
 * when （tasks-panel.tsx） import this 1 ： 「check ·
 * 」 to new when forwardUpdateResult and / check 1 ——
 * this 。
 */

import type * as React from 'react'
import type { AssetProbe, UpdateChannels, ForceUpdatePolicy, UpdateCapability, ServiceUpdateTask } from '../../../ui-shared/updates'
import { assetMissingReason, githubErrorMessage, updateUnavailableReason } from '../../../ui-shared/updates'
export type { UpdateChannels, AssetProbe } from '../../../ui-shared/updates'
// and before and use 1 —— down must 1 
// （`pool:<id>`）， 1 will 。accounts-shared is + 
// （ not import ）， this use not 。
import { POOL_VALUE_PREFIX, poolItemLabel, type PoolItem } from './accounts-shared'
import { acceptProxyPool } from './proxy-pool-cache'
import { t } from '../i18n'

/* ─── ─────────────────────────────────── */

/** updatecheck（checkUpdate return / getUpdateStatus ）。 by optional：
 * after in 「none」「none」 etc down will to null */
export type UpdateInfo = {
  updateCapability?: UpdateCapability
  systemdTask?: ServiceUpdateTask | null
  serviceTask?: ServiceUpdateTask | null
  forceUpdate?: ForceUpdatePolicy | null
  currentVersion?: string | null
  latestVersion?: string | null
  /** true / false / null（none when after to null，「none」） */
  hasUpdate?: boolean | null
  /** new 1 （Markdown ， to wbMarkdown） */
  notes?: string | null
  /** UTC ISO */
  publishedAt?: string | null
  /** GitHub （， to openReleasePage ） */
  pageUrl?: string | null
  prerelease?: boolean
  /** can down ； no has when 「 down and 」 not to */
  asset?: { url?: string; name?: string } | null
  /** in （ down ），「update from 」；
   * ** not need ** / Star —— that is `sourceRepository` */
  repository?: string
  /** （`owner/repo`）： and 「」 by 。
   * and `repository` is as in down ， that 
   * no has 、 no has issue， etc */
  sourceRepository?: string
  /** 'nsis'（Windows， need UAC ）/ 'dmg'（macOS， after ） */
  installerKind?: string
  /** check when （ after to ； before not use when ） */
  checkedAt?: number
  /** after 1 checkfailureReason（success when as empty）。 checkUpdate not ，
   * has /api/update/status has —— 「 is not is already 1 failure」 */
  lastError?: string | null
  /** 1 ****check when （success and failure）。 and checkedAt（ 1 success），
   * after failure 1 */
  lastAttemptAt?: number
  /** after that 1 check is in （ in when not old lastError 「 this failure」） */
  checking?: boolean
  /** only getUpdateStatus has ： after when no when as false */
  checked?: boolean
  channelChanged?: boolean
  channels?: UpdateChannels
  /* ── update（Linux headless / Docker has ， as false/null）──
   * and up asset is ：asset is （ down by ），
   * down this is 「 in and 」 。 */
  /** can not can update（ + AGENT2API_SELF_UPDATE ）， and none */
  selfUpdate?: boolean
  /** this 1 release for update；null = this no （） */
  serverAsset?: { url?: string; name?: string; size?: number } | null
  /** update already use to 、（ by 「」） */
  pendingRestart?: boolean
  /** this 1 ** to file**（ after / total / has no has update ）：
   * no has can down when ，Reason， missingAssetReasonFor。
   * before check no has —— use must can 「 need 」 */
  assetProbe?: AssetProbe | null
}

/**
 * update（Linux headless / Docker） and is ** not **，file
 * by this 1 ： selfUpdate as false， down old as 。
 *
 * · ： after to `asset`（NSIS / dmg / deb / AppImage ）→ down → by 
 * （ run_installer）；
 * · ： after to `serverAsset`（serverupdate）→ down → server
 * and ui directory → （ restart ）。
 */
export function isSelfUpdate(info: UpdateInfo | null | undefined): boolean {
  return info?.updateCapability ? info.updateCapability.mode === 'docker' : info?.selfUpdate === true
}

export function isServiceUpdate(info: UpdateInfo | null | undefined): boolean {
  return info?.updateCapability?.mode === 'systemd' || info?.updateCapability?.mode === 'windows-service'
}

export function serviceUpdateTask(info: UpdateInfo | null | undefined): ServiceUpdateTask | null {
  return info?.serviceTask ?? info?.systemdTask ?? null
}

export function isManualUpdate(info: UpdateInfo | null | undefined): boolean {
  return info?.updateCapability?.installSupported === false
}

export function manualUpdateReason(info: UpdateInfo | null | undefined): string {
  if (!info?.updateCapability) return ''
  const reason = updateUnavailableReason(info.updateCapability, t)
  return info.updateCapability.deployment === 'docker'
    ? `${reason} ${t('在宿主机执行 docker compose pull && docker compose up -d 即可升级。')}` : reason
}

/** not ， to server。 */
export function downloadAsset(info: UpdateInfo | null | undefined): { url?: string; name?: string } | null {
  if (!info) return null
  const server = info.updateCapability?.packageKind === 'server' || isSelfUpdate(info)
  return server ? (info.serverAsset ?? null) : (info.asset ?? null)
}

/** installerKind → this after 。 in after no to assetProbe when use 
 * （ after expectedSuffixes as ， is pick_installer 1 ） */
function installerSuffixOf(info: UpdateInfo | null | undefined): string {
  switch (String(info?.installerKind || '')) {
    case 'dmg': return '.dmg'
    case 'deb': return '.deb'
    case 'appimage': return '.AppImage'
    case 'nsis': return '.exe'
    default: return ''
  }
}

/** [assetMissingReason] ： up 「」 that （ is 、 need after ）。
 * 1 Reason， in use 1 —— that is */
export function missingAssetReasonFor(info: UpdateInfo | null | undefined): string {
  return assetMissingReason(info?.assetProbe, {
    selfUpdate: isSelfUpdate(info),
    serverPackage: info?.updateCapability?.packageKind === 'server',
    fallbackSuffix: installerSuffixOf(info),
  }, t)
}

/** down （downloadUpdate / updateProgress return） */
export type DownloadTask = {
  active?: boolean
  done?: boolean
  canceled?: boolean
  error?: string | null
  filename?: string
  path?: string | null
  received?: number
  total?: number
  percent?: number
  selecting?: boolean
  source?: string | null
}

export type UpdateProxyChoice = {
  source?: string
  label?: string
  error?: string
  config?: { source?: string; listenerUid?: string; proxyId?: string } | null
} | null

/** GitHub （`/api/update/token` return）：** no has **， has has no has 、 */
export type UpdateTokenStatus = {
  filled?: boolean
  /** 'stored'（）/ 'env'（）· null（ no ） */
  origin?: string | null
  /** already not when Reason（ new 1 ） */
  error?: string | null
  /** only POST has ： is success（false = already ， will ） */
  saved?: boolean
}

import type { GithubAccessSettings, GithubAccessPatch } from '../../../ui-shared/github-access'
export type { GithubAccessSettings } from '../../../ui-shared/github-access'

/** 「update」 this 1 use to （ bridge.rs 「update」 1 ） */
export type UpdateBridge = {
  /** ：current has ， by up to after */
  checkUpdate(): Promise<UpdateInfo | null | undefined>
  /** after 1 check（ when ） */
  getUpdateStatus(): Promise<UpdateInfo | null | undefined>
  /** update currentset（proxy as null = ） */
  getUpdateProxy(): Promise<{ proxy?: UpdateProxyChoice } | null | undefined>
  
  setUpdateProxy(payload: { proxy: unknown }): Promise<{
    proxy?: UpdateProxyChoice
    saved?: boolean
  } | null | undefined>
  getGithubAccess(): Promise<GithubAccessSettings | null | undefined>
  setGithubAccess(payload: GithubAccessPatch): Promise<GithubAccessSettings | null | undefined>
  /** GitHub （filled / origin， not ） */
  getUpdateToken(): Promise<UpdateTokenStatus | null | undefined>
  /** / GitHub （token null = ） */
  setUpdateToken(payload: { token: string | null }): Promise<UpdateTokenStatus | null | undefined>
  /** 「 beta 」 （`/api/update/beta`）： has enabled 1 */
  getUpdateBeta(): Promise<{ enabled?: boolean } | null | undefined>
  /** 「 beta 」：true = （ after check GitHub 1 and ）。
   * response saved = is success（false = already ， to ） */
  setUpdateBeta(payload: { enabled: boolean }): Promise<{
    enabled?: boolean
    saved?: boolean
  } | null | undefined>
  getUpdateChannels(): Promise<UpdateChannels | null | undefined>
  setUpdateChannels(payload: Partial<Pick<UpdateChannels, 'stable' | 'beta'>>): Promise<UpdateChannels | null | undefined>
  /** ：「checkupdate」（ id 'updateCheck'） and ——
   * that config「updateset」（update-settings.tsx） */
  getScheduledTasks?: () => Promise<{
    tasks?: Array<{ id?: string; enabled?: boolean; interval?: number }>
  } | null | undefined>
  /** config（PATCH /api/scheduled-tasks/{id}，response is that ） */
  saveScheduledTask?: (
    id: string,
    patch: Record<string, unknown>,
  ) => Promise<{ id?: string; enabled?: boolean; interval?: number; saved?: boolean } | null | undefined>
  /** 「」 （ down ， and 1 ） */
  getProxyPool(): Promise<{ items?: PoolItem[] } | null | undefined>
  downloadUpdate(payload: { url: string; name?: string }): Promise<DownloadTask | null | undefined>
  updateProgress(): Promise<DownloadTask | null | undefined>
  cancelUpdate(): Promise<{ canceled?: boolean } | null | undefined>
  runInstaller(
    path: string,
    restart: boolean,
  ): Promise<{ launched?: boolean; path?: string; restart?: boolean } | null | undefined>
  /* ── update（Linux headless / Docker）： this has ──
   * no has （ is runInstaller ）， use use 
   * optional use + selfUpdate 。 */
  /** and use already down update（ and ui directory； after ） */
  installUpdate?(): Promise<{
    applied?: boolean
    version?: string
    needRestart?: boolean
    uiUpdated?: boolean
  } | null | undefined>
  startSystemdUpdate?(payload: { version: string }): Promise<ServiceUpdateTask>
  startWindowsServiceUpdate?(payload: { version: string }): Promise<ServiceUpdateTask>
  /** request（ by restart ， after new ） */
  restartService?(): Promise<{ restarting?: boolean } | null | undefined>
  /** 1 （ http(s)， commands.rs） */
  openReleasePage(url: string): Promise<{ url?: string } | null | undefined>
}

/**
 * window up by its / its 。
 *
 * use 「 + 」 not is declare global Window up ：
 * workbuddyDesktop / wbApp / wbMarkdown is use ， declare 1 will 
 * not 1 TS2717。update-panel wbUpdatePanel
 * in update-panel.tsx declare。
 */
export type SharedWindow = {
  wbConfirm?: { ask(options: { title: string; text: string; okText: string; cancelText: string; bodyClass?: string }): Promise<boolean> }
  workbuddyDesktop?: UpdateBridge
  wbApp?: {
    toast?: (message: string, kind?: 'err' | 'ok') => void
    /** 「 has new 」「set」 up 「 new 」（app.js ） */
    updateUpdateBadge?: (info: unknown) => void
    /** ：「update」 need to set（ and old app.js 1 ） */
    showPage?: (name: string, options?: { persist?: boolean }) => void
    /** current： already in set when not （ old app.js ） */
    readonly currentPage?: string
  }
  /** set （「update」 need to 「update」） */
  wbSettingsPanel?: { showCategory?: (category: string) => void }
  /** update Markdown （ use ， not in this ） */
  wbMarkdown?: { render?: (text: string) => string }
  /** SVG （ui/icons.js ； from this ） */
  wbIcons?: { icon?: (name: string, size?: number) => string }
}

export function shared(): SharedWindow {
  return window as unknown as SharedWindow
}

/* ─── （ / error / ）───────────────── */

/** toast 1 （ wbApp， not in ） */
export function toast(message: string, kind?: 'err' | 'ok'): void {
  shared().wbApp?.toast?.(message, kind)
}

export function errorMessage(error: unknown): string {
  return githubErrorMessage(error, t, 'update')
}

/** ： http(s)（ and after open_release_page 1 ， this 1 ） */
export function safeExternal(value: unknown): string {
  const url = String(value || '').trim()
  return /^https?:\/\//i.test(url) ? url : ''
}

/**
 * ： 1 to default。webview will ， and has 
 * ， not this in use （「updateset」 GitHub ）。
 */
export async function openExternal(url: string): Promise<void> {
  const target = safeExternal(url)
  if (!target) {
    toast(t('链接地址不受支持'), 'err')
    return
  }
  try {
    await shared().workbuddyDesktop?.openReleasePage(target)
  } catch (error) {
    toast(t('打开链接失败：{error}', { error: errorMessage(error) }), 'err')
  }
}

/**
 * QQ （ ）。 and can ； 1、 2 already ，
 * when to 3 —— this in ， not in up 。
 *
 * Why not ： is ， / 1 。
 * in will （ 1 、 after can can ）， + by ， 1 can 。
 *
 * 1 ： QQ can in QQ that 1 —— `mqqapi://`
 * QQ NT not ，`qm.qq.com/q/…` that QQ 。
 * this to （ / to QQ ）， not can 1 。
 */
export const QQ_GROUPS = [
  { no: '1127656296', nearlyFull: true },
  { no: '1130894674', nearlyFull: true },
  { no: '1108276647', nearlyFull: false },
] as const

/**
 * its 。 is https ， to default（ and update 
 * GitHub 1 ：`data-external` + openReleasePage）。
 * not in this —— need ， by use t() 。
 */
export const COMMUNITY_LINKS = [
  { id: 'telegram', url: 'https://t.me/+9gM8YPvgRL02NGRl' },
] as const

/** update： to wbMarkdown（ not ）， */
export function markdownHtml(notes: string): string {
  return shared().wbMarkdown?.render?.(notes) || ''
}

/**
 * ：update and Markdown is ， 
 * 。 and onClick in up （React to 
 * that ）， Portal in body up 、 not to ， 1 。
 */
export function handleExternalClick(event: React.MouseEvent<HTMLElement>): void {
  const target = event.target as HTMLElement | null
  const trigger = target?.closest?.('[data-external]') as HTMLElement | null
  if (!trigger) return
  event.preventDefault() // 掐掉 webview 自己的导航（跳过去只会白屏）
  void openExternal(trigger.dataset.external || '')
}

/**
 * ** 1 check** to update：「set」 up 「 new 」， and in this 
 * when 「 to update」（update and 「update」）。 is after 
 * （`/api/update/status`）， not GitHub。
 *
 * ── Why in this ──────────────────────────────────────
 * when 「check · 」 and app.js 20 seconds（`pollUpdateStatus`，
 * 「 when to → to new 」 that ） is ** 1 **： app.js 
 * `updateUpdateBadge`， and not （update / will already / already in set）
 * in update-modal.tsx。 in file is as 「update 」 can to 
 * 1 ， not declare 1 、 1 （ that ）。
 *
 * failure（ after unavailable / not to ）： use （ 1 「」） already 
 * ， not this as this 1 down failure failure。
 */
export async function forwardUpdateResult(): Promise<void> {
  try {
    const info = await shared().workbuddyDesktop?.getUpdateStatus?.()
    // must old and ；none then has 。
    if (info && (info.checked !== false || info.channelChanged)) shared().wbApp?.updateUpdateBadge?.(info)
  } catch (error) {
    console.warn('读取版本检查结果失败:', errorMessage(error))
  }
}

/* ─── （「updateset」 that 1 ）────── */

export const PROXY_OTHER_CURRENT = '__proxy_other__'

/** 1 （ is ） */
export type ProxySelection = {
  proxyChoice: UpdateProxyChoice
  proxyLoaded?: boolean
  
  pool: PoolItem[] | null
  /** failure Reason（empty = no failure， and 「 no 」） */
  poolError: string
}

export const EMPTY_PROXY_SELECTION: ProxySelection = {
  proxyChoice: null,
  proxyLoaded: false,
  pool: null,
  poolError: '',
}

export async function fetchProxySelection(): Promise<ProxySelection> {
  const api = shared().workbuddyDesktop
  const selection: ProxySelection = { ...EMPTY_PROXY_SELECTION }
  if (!api?.getUpdateProxy) return selection
  try {
    const data = await api.getUpdateProxy()
    if (!data || !Object.prototype.hasOwnProperty.call(data, 'proxy')) throw new Error('更新线路响应异常')
    selection.proxyChoice = data.proxy ?? null
    selection.proxyLoaded = true
  } catch {
    /* set not to ： by ， will */
  }
  try {
    if (typeof api.getProxyPool !== 'function') throw new Error('桥未提供代理池方法')
    const data = await api.getProxyPool()
    const items = data?.items
    // when can can resolve undefined not is reject： not will 「 1 no has 」
    if (!Array.isArray(items)) throw new Error('代理池响应异常')
    selection.pool = items
    acceptProxyPool(items)
  } catch (error) {
    selection.pool = []
    selection.poolError = errorMessage(error)
  }
  return selection
}

export async function saveProxy(value: string): Promise<{ choice: UpdateProxyChoice; saved: boolean } | null> {
  if (value === PROXY_OTHER_CURRENT) return null
  const api = shared().workbuddyDesktop
  if (!api?.setUpdateProxy) return null
  const proxy = value === ''
    ? null
    : { source: 'pool', proxyId: value.slice(POOL_VALUE_PREFIX.length) }
  try {
    const result = await api.setUpdateProxy({ proxy })
    // after as （failure when error）， not from 
    const choice = result?.proxy ?? null
    const saved = result?.saved !== false
    if (!saved) toast(t('出网线路已切换，但写入磁盘失败（重启后会恢复原线路）'), 'err')
    else toast(choice
      ? t('✅ 更新出网已切换为 {label}', { label: choice.label || t('指定代理') })
      : t('✅ 更新出网已切换为直连'))
    return { choice, saved }
  } catch (error) {
    toast(t('保存失败：{error}', { error: errorMessage(error) }), 'err')
    return null
  }
}

export function buildProxyPick(selection: ProxySelection): {
  current: string
  items: Array<{ value: string; label: string; disabled?: boolean }>
  selected: { value: string; label: string } | undefined
  broken: boolean
  title: string
} {
  const proxy = selection.proxyChoice
  const source = proxy?.config?.source || proxy?.source
  const label = proxy?.label || (source === 'custom' ? t('自定义代理') : t('已设置'))
  const poolItems = Array.isArray(selection.pool) ? selection.pool : []
  const broken = Boolean(proxy?.error)
  // ） to —— not can 「」
  let current = ''
  if (source === 'pool' && proxy?.config?.proxyId) current = `${POOL_VALUE_PREFIX}${proxy.config.proxyId}`
  else if (proxy) current = PROXY_OTHER_CURRENT

  const items: Array<{ value: string; label: string; disabled?: boolean }> = [{ value: '', label: t('直连') }]
  for (const item of poolItems) {
    // 「（ :）」—— and 、 1 。
    // use ： updatefailure when not will 
    items.push({
      value: `${POOL_VALUE_PREFIX}${item.id}`,
      label: poolItemLabel(item),
      disabled: item.enabled === false,
    })
  }
  if (selection.pool === null) {
    // no to （、request in ）： to 1 「 in 」 not is 
    // 「 no has 」—— after will use as is empty 
    items.push({ value: '__hint_pool__', label: t('正在读取代理列表…'), disabled: true })
  } else if (!poolItems.length) {
    items.push({
      value: '__hint_pool__',
      label: selection.poolError ? t('代理列表读取失败') : t('还没有代理（去「网络代理」页添加）'),
      disabled: true,
    })
  }
  if (current === PROXY_OTHER_CURRENT) {
    // in no has for optional（ 1 ）， need —— 
    // ， use in this 1 or 
    const prefix = source === 'clash' ? t('Clash 出口：') : source === 'custom' ? t('自定义：') : ''
    items.push({ value: PROXY_OTHER_CURRENT, label: `${prefix}${label}${broken ? t('（不可用）') : ''}` })
  } else if (source === 'pool' && !poolItems.some(item => `${POOL_VALUE_PREFIX}${item.id}` === current)) {
    // after failure Reason in title 
    items.push({ value: current, label: `${label}${broken ? t('（不可用）') : ''}` })
  }
  return {
    current,
    items,
    selected: items.find(item => item.value === current),
    broken,
    title: broken
      ? t('当前线路不可用：{error}；请重新选择代理或切回直连', { error: String(proxy?.error || '') })
      : t('检查更新与下载安装包使用所选线路；未选择代理时先直连，失败后尝试 Clash Verge 混合端口；指定代理不可用时不会自动改走直连。'),
  }
}
