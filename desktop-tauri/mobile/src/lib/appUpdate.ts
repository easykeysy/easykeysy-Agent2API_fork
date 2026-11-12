/**
 * use update。
 *
 * ── ， ────────────────────────────────────────────
 * · **PWA / **：update Service Worker（ lib/pwa.ts）， this not and 。
 * 「 new 」 is `ui/m/*` ， new can 。
 * · **APK**： in ，must。 this is this ：
 * 1. `GET /api/update/check` new and `androidAsset`；
 * 2. to down ， by down and APK；
 * 3. 。
 * down not in ，GitHub not to or 。
 * · **iOS（IPA）**： not App down and can ， this 
 * ** can ** —— `pageUrl` GitHub Release ， by use 
 * TestFlight / App Store / AltStore。`downloadApk` / `installApk` in iOS
 * up ， not that by （ AppUpdatePanel）。
 *
 * ── Why down not is fetch ────────────────────────────────
 * APK has MB，`fetch` + `blob` will WebView ；`CapacitorHttp`
 * 1 （ response ）。 use HttpURLConnection 
 * file， use has ， can when up 。
 *
 * ── from ────────────────────────────────────────────────
 * `__APP_VERSION__` is by vite from `mobile/package.json` 
 * （ vite.config.ts）， and APK `versionName`（android/variables.gradle
 * 1 ） 1 ；iOS `MARKETING_VERSION` by CI from 1 
 * xcodebuild parameter， not will 。
 *
 * ── check ────────────────────────────────────────────────────
 * and ** 「check」config**（`/api/scheduled-tasks`
 * `updateCheck` ）， not 1 set —— use in 1 ， 1 。
 * 1 「 not check」 （ in localStorage），
 * used for「 not 」 this 。
 */

import { Capacitor, registerPlugin } from '@capacitor/core'
import { ApiError, apiCall } from '../api/client'
import { activeServerId, nativePlatform } from '../lib/servers'
import { t } from '../i18n'
import type { AssetProbe, ForceUpdatePolicy } from '../../../ui-shared/updates'

/** */
declare const __APP_VERSION__: string

export const APP_VERSION: string = typeof __APP_VERSION__ === 'string' ? __APP_VERSION__ : '0.0.0'

/** updatecheck that */
export type ApkUpdateInfo = {
  forceUpdate?: ForceUpdatePolicy | null
  currentVersion: string
  latestVersion: string | null
  hasUpdate: boolean | null
  notes: string
  publishedAt: string
  pageUrl: string
  prerelease: boolean
  /** this 1 APK ；null = this no Android */
  asset: { name?: string; size?: number; url?: string } | null
  /** this 1 to file —— no has APK when Reason（ assetMissingReason） */
  assetProbe?: AssetProbe | null
}

type RawCheckResult = {
  forceUpdate?: ForceUpdatePolicy | null
  currentVersion?: string | null
  latestVersion?: string | null
  hasUpdate?: boolean | null
  notes?: string | null
  publishedAt?: string | null
  pageUrl?: string | null
  prerelease?: boolean
  androidAsset?: { name?: string; size?: number; url?: string } | null
  assetProbe?: AssetProbe | null
  checked?: boolean
  channelChanged?: boolean
}

/**
 * checkupdate。
 *
 * not is server： can can in up 、 and not ，
 * 「 has no has new 」 has current is 。
 *
 * `/api/update/check` not is GitHub API： this can ** use 
 * already config GitHub **（ 60 / when ， 5000），
 * not 1 。Note this use 「」， down is 。
 */
export async function checkApkUpdate(): Promise<ApkUpdateInfo> {
  const raw = await apiCall<RawCheckResult>(
    `/api/update/check?current=${encodeURIComponent(APP_VERSION)}`,
  )
  if (raw.channelChanged && raw.checked === false) throw new Error(t('更新通道已改变，请重新检查更新。'))
  const asset = raw.androidAsset && raw.androidAsset.url ? raw.androidAsset : null
  return {
    forceUpdate: raw.forceUpdate,
    currentVersion: APP_VERSION,
    latestVersion: raw.latestVersion || null,
    // has new no APK when ， for etc 「 no has can update 」：
    // not 「 has update」 use to 1 not by up 。
    hasUpdate: raw.hasUpdate === true && asset !== null ? true : raw.hasUpdate === false ? false : null,
    notes: raw.notes || '',
    publishedAt: raw.publishedAt || '',
    pageUrl: raw.pageUrl || '',
    prerelease: Boolean(raw.prerelease),
    asset,
    assetProbe: raw.assetProbe ? { ...raw.assetProbe, expectedSuffixes: ['.apk'], expectedName: null } : null,
  }
}

/* ─── check config（） ─────────────────────────────── */

/** 「 not check」（， not up ） */
const AUTO_CHECK_KEY = 'agent2api.mobile.updateAutoCheck'

/** is check（default） */
export function autoCheckEnabled(): boolean {
  try {
    return localStorage.getItem(AUTO_CHECK_KEY) !== 'off'
  } catch {
    return true
  }
}

export function setAutoCheckEnabled(enabled: boolean): void {
  try {
    if (enabled) localStorage.removeItem(AUTO_CHECK_KEY)
    else localStorage.setItem(AUTO_CHECK_KEY, 'off')
  } catch {
    /* ： in */
  }
}

/**
 * from 「check」check（ms）。
 *
 * this in 「updateset」（ when not ），config
 * by `/api/scheduled-tasks` return —— 1 ， not set。
 * when return 0（= not check）， and 1 。
 *
 * not to （ / requestfailure） when 20 minutes： that is 
 * （`DEFAULT_UPDATE_CHECK_MINUTES`）， as and 「 no 」 1 。
 */
export async function autoCheckIntervalMs(): Promise<number> {
  try {
    const payload = await apiCall<{
      tasks?: Array<{ id?: string; enabled?: boolean; interval?: number; unit?: string }>
    }>('/api/scheduled-tasks')
    const task = (payload.tasks || []).find(item => item.id === 'updateCheck')
    if (!task) return 20 * 60_000
    if (task.enabled === false) return 0
    const interval = Number(task.interval) || 0
    if (interval <= 0) return 20 * 60_000
    // this is minutes（ scheduled_tasks.rs TaskDef）
    return interval * 60_000
  } catch {
    return 20 * 60_000
  }
}

/* ─── （ in APK in ） ─────────────────────────────── */

type PluginListenerHandle = { remove: () => Promise<void> }

type ApkUpdaterPlugin = {
  sourceSelectionSupported(): Promise<{ supported: boolean }>
  selectSources(options: { urls: string[]; sha256?: string; scope: string }): Promise<{ urls: string[]; selected: string; elapsedMs: number; cached: boolean }>
  download(options: { url: string; sha256?: string; size?: number }): Promise<{ path: string; size: number }>
  cancel(): Promise<void>
  canInstall(): Promise<{ allowed: boolean }>
  openInstallPermissionSettings(): Promise<void>
  install(): Promise<void>
  downloaded(): Promise<{ exists: boolean; size: number }>
  addListener(
    event: 'downloadProgress',
    handler: (data: { received: number; total: number; percent: number }) => void,
  ): Promise<PluginListenerHandle>
}

const ApkUpdater = registerPlugin<ApkUpdaterPlugin>('ApkUpdater')

/** ； not in APK （ or no ） when return null */
function plugin(): ApkUpdaterPlugin | null {
  if (nativePlatform() !== 'android' || !Capacitor.isPluginAvailable('ApkUpdater')) return null
  return ApkUpdater
}

/** current can not can in use down and （ has Android can ） */
export function canSelfUpdate(): boolean {
  return nativePlatform() === 'android' && plugin() !== null
}

export type DownloadProgress = {
  received: number
  total: number
  /** 0–100；totalunknown when as -1 */
  percent: number
  phase?: 'selecting' | 'downloading'
  source?: string
}

let downloadCanceled = false
let downloadTask: Promise<{ path: string; size: number }> | null = null
let nativeSelection: Promise<boolean> | null = null

function sourceSelectionSupported(api: ApkUpdaterPlugin): Promise<boolean> {
  nativeSelection ??= api.sourceSelectionSupported().then(value => value.supported === true).catch(() => false)
  return nativeSelection
}

type DownloadPlan = { urls: string[]; sha256?: string | null; size?: number; mode?: 'auto' | 'official' | 'accelerated' }

/** APK by down ； and API 。 */
export async function downloadApk(
  url: string,
  onProgress?: (progress: DownloadProgress) => void,
): Promise<{ path: string; size: number }> {
  if (downloadTask) throw new Error('下载或探测正在进行')
  downloadCanceled = false
  const task = runApkDownload(url, onProgress)
  downloadTask = task
  try { return await task }
  finally { if (downloadTask === task) downloadTask = null }
}

async function runApkDownload(
  url: string,
  onProgress?: (progress: DownloadProgress) => void,
): Promise<{ path: string; size: number }> {
  const api = plugin()
  if (!api) {
    // iOS and will to this ： not use 
    throw new Error(
      nativePlatform() === 'ios'
        ? 'iOS 不支持应用内安装，请到 GitHub Release 页面下载'
        : '当前不是 APK 形态，无法在应用内更新',
    )
  }
  if (!url) throw new Error('缺少下载地址')

  let plan: DownloadPlan
  try {
    plan = await apiCall<DownloadPlan>('/api/update/download-plan', { method: 'POST', body: { url } })
  } catch (error) {
    if (!(error instanceof ApiError) || error.status !== 404) throw error
    plan = { urls: [url] }
  }
  if (!plan.urls?.length) throw new Error('没有可用的下载源')
  if (downloadCanceled) throw new Error('已取消下载')
  const candidates = [...plan.urls]
  const hasNativeSelection = await sourceSelectionSupported(api)
  if (plan.mode === 'auto' && candidates.length > 1 && hasNativeSelection) {
    if (downloadCanceled) throw new Error('已取消下载')
    onProgress?.({ received: 0, total: 0, percent: -1, phase: 'selecting' })
    const selected = await api.selectSources({ urls: candidates, sha256: plan.sha256 || undefined, scope: activeServerId() || 'local' })
    if (downloadCanceled) throw new Error('已取消下载')
    const valid = selected.urls.filter(candidate => candidates.includes(candidate))
    candidates.splice(0, candidates.length, ...new Set([...valid, ...candidates]))
  }
  let handle: PluginListenerHandle | null = null
  let currentSource = ''
  if (onProgress) {
    handle = await api.addListener('downloadProgress', data => onProgress({ ...data, phase: 'downloading', source: currentSource }))
  }
  try {
    let lastError: unknown = new Error('没有可用的下载源')
    for (const candidate of candidates) {
      if (downloadCanceled) throw new Error('已取消下载')
      try {
        currentSource = new URL(candidate).host
        onProgress?.({ received: 0, total: plan.size || 0, percent: -1, phase: 'downloading', source: currentSource })
        const result = await api.download({ url: candidate, sha256: plan.sha256 || undefined, size: plan.size })
        return result
      } catch (error) {
        if (downloadCanceled) throw new Error('已取消下载')
        const message = error instanceof Error ? error.message : String(error)
        if (/限流|HTTP 429\b/.test(message) || (!hasNativeSelection && /HTTP 403\b/.test(message))) throw error
        lastError = error
      }
    }
    throw lastError
  } finally {
    if (handle) void handle.remove()
  }
}

export async function cancelDownload(): Promise<void> {
  downloadCanceled = true
  const task = downloadTask
  await plugin()?.cancel()
  await task?.catch(() => undefined)
}

/** is already 「unknown use 」 */
export async function canInstallApk(): Promise<boolean> {
  const api = plugin()
  if (!api) return false
  const result = await api.canInstall()
  return Boolean(result.allowed)
}

/** to 「unknown use 」 */
export async function openInstallPermission(): Promise<void> {
  await plugin()?.openInstallPermissionSettings()
}

/** （APK already down when ） */
export async function installApk(): Promise<void> {
  const api = plugin()
  if (!api) {
    throw new Error(
      nativePlatform() === 'ios'
        ? 'iOS 不支持应用内安装，请到 GitHub Release 页面下载'
        : '当前不是 APK 形态，无法安装',
    )
  }
  await api.install()
}

/** is already has down （set when 「 already down ，」） */
export async function hasDownloadedApk(): Promise<{ exists: boolean; size: number }> {
  const api = plugin()
  if (!api) return { exists: false, size: 0 }
  return api.downloaded()
}
