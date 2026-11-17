import type { GithubFailure } from './github-access'

export type ForceUpdatePolicy = { required: boolean; minimumVersion: string; targetVersion: string; message: string; source?: string }

export type UpdateCapability = {
  deployment: 'desktop' | 'docker' | 'systemd' | 'windows-service' | 'foreground'
  mode: 'desktop' | 'docker' | 'systemd' | 'windows-service' | 'manual'
  packageKind: 'desktop' | 'server'
  installSupported: boolean
  reason?: 'disabled' | 'helper_missing' | 'helper_unavailable' | 'custom_installation' | 'unsupported_arch' | 'windows_service' | 'windows_setup_required' | 'windows_update_permissions' | 'unmanaged' | null
}

export type ServiceUpdateTask = {
  id: string
  version: string
  phase: 'downloading' | 'verifying' | 'installing' | 'restarting' | 'completed' | 'failed'
  startedAt: number
  updatedAt: number
  error?: string | null
}

export type SystemdUpdateTask = ServiceUpdateTask

export function updateUnavailableReason(capability: UpdateCapability, t: Translator): string {
  switch (capability.reason) {
    case 'windows_setup_required': return t('One-click updates are not set up for this Windows service. Extract the new server package and run install-service.ps1 once as administrator. Existing accounts, directories, ports and settings will be preserved.')
    case 'windows_update_permissions': return t('The Windows service permissions or directory protection do not meet update requirements. Run the new install-service.ps1 as administrator. Custom service accounts or externally managed environments should use their existing upgrade procedure.')
    case 'disabled': return t('An administrator has disabled one-click updates. Please upgrade manually.')
    case 'helper_missing': return t('One-click updates are not installed yet. Download the new server package and run sudo bash install.sh --upgrade once. Future updates can then be installed from the panel.')
    case 'helper_unavailable': return t('The update helper is unavailable or its permissions are incorrect. Run sudo bash install.sh --upgrade from the new package to repair it.')
    case 'custom_installation': return t('Custom program or panel paths were detected. One-click updates will not overwrite them. Use your existing deployment procedure.')
    case 'unsupported_arch': return t('Automatic installation is not supported on this system or architecture. Check the release page.')
    case 'windows_service': return t('Windows services require manual upgrades. Extract the new server package and run install-service.ps1 as administrator. Accounts and settings are preserved.')
    default: return t('This server runs without a service manager. Stop it, replace the server files and start it again.')
  }
}

export type UpdateChannels = {
  stable: boolean
  beta: boolean
  saved?: boolean
}

export type UpdateCheckPlan = {
  enabled: boolean
  interval: number
  saved?: boolean
}

export type UpdateSettingsMode = 'follow' | 'custom'

type Translator = (key: string, params?: Record<string, string | number>) => string

/**
 * 「有新版本，但没有可下载的包」时后端给的诊断（check 响应的 `assetProbe`）。
 *
 * ── 为什么需要它 ────────────────────────────────────────────
 * 光凭 `asset` 为空说不清原因，而三种原因的处置办法完全不同：上游压根没传包
 * （等构建 / 自己去发布页下）、只传了别的平台（同上）、被更新源的访问方式挡掉了
 * （去「更新设置」改访问方式）。这三个字段就是用来分开它们的。
 *
 * 字段全可选：升级前落库的那份检查结果里没有它，界面必须能退化到「不说原因」。
 */
export type AssetProbe = {
  /** 这一版 release 上传的文件数（按更新源白名单过滤**前**的清单） */
  assetsTotal?: number
  /** 这些文件出现过的后缀（小写、带点、去重） */
  suffixes?: string[]
  /** 有文件因更新源的访问方式（加速前缀 / 自定义 API）没通过而被丢掉 */
  filteredBySource?: boolean
  /** 本机要找的后缀（桌面安装包；容器内更新是空表） */
  expectedSuffixes?: string[]
  /** 本机要找的完整文件名（容器内更新包；桌面是 null） */
  expectedName?: string | null
}

/**
 * 「为什么没有可下载的包」——toast / 弹窗 / 面板状态行共用这一处口径。
 *
 * 返回的是一句能独立看懂的**原因**，由调用方拼进自己的句子里（桌面与容器的
 * 前半句不同，见 update-panel 与 update-modal）。取不到诊断时退化成「本机要
 * 什么」这一层 —— 至少让排查有方向，而不是只丢一句「没有包」。
 */
export function assetMissingReason(
  probe: AssetProbe | null | undefined,
  options: { selfUpdate?: boolean; fallbackSuffix?: string; serverPackage?: boolean },
  t: Translator,
): string {
  // ① 被更新源的访问方式过滤掉：文件其实传了，是我们这边把它们挡掉了。
  // 先说这条 —— 它的处置办法（去「更新设置」改访问方式）与另外两种完全不同
  if (probe?.filteredBySource) {
    return t('The files on the release page were filtered out by your update source settings. Check the GitHub access mode in Update settings.')
  }
  // ② 一个文件都没传：构建失败，或者还在打包上传
  if (probe?.assetsTotal === 0) {
    return t('No files have been uploaded for this release yet (the build may have failed, or uploads are still in progress)')
  }
  // ③ 传了文件，但没有本机要的那一份 —— 把「有什么」与「要什么」都摆出来。
  // 后端给的 expected 优先（Linux 桌面壳是 [deb, AppImage] 两个候选），
  // 老后端没给时退到调用方按 installerKind 推的那个后缀
  const seen = (probe?.suffixes || []).filter(Boolean).slice(0, 6)
  const serverPackage = options.serverPackage || options.selfUpdate
  const wanted = serverPackage
    ? String(probe?.expectedName || '')
    : ((probe?.expectedSuffixes ?? [options.fallbackSuffix || '']).filter(Boolean).join(' / '))
  if (seen.length && wanted) {
    return t('No suitable update package was found. Required: {wanted}. File types on the release page: {seen}.', { seen: seen.join(' / '), wanted })
  }
  // ④ 兜底：连「有什么」都不知道，至少说清「本机要什么」
  if (serverPackage) {
    return wanted
      ? t('The release page has no file named {wanted}', { wanted })
      : t('The release page has no server update package for this machine\'s architecture')
  }
  return wanted
    ? t('This release did not publish the installer this machine needs ({wanted})', { wanted })
    : t('This release provides no installer for the current platform')
}

export function githubFailureMessage(value: GithubFailure, t: Translator, settings: 'update' | 'plugin', now = Date.now()): string {
  if (!value.rateLimited && !value.rateLimits?.length) return githubErrorMessage(value.reason, t, settings)
  const active = (value.rateLimits || []).filter(limit => limit.retryAt > now)
  if (!active.length) return t('The previous GitHub rate-limit wait has ended. You can check again.')
  const first = active.reduce((left, right) => left.retryAt <= right.retryAt ? left : right)
  const source = [...new Set(active.map(limit => limit.source === 'official' ? t('Official source') : limit.label))].join(' / ')
  const params = { source, seconds: Math.ceil((first.retryAt - now) / 1000) }
  const message = first.retrySource === 'fallback'
    ? t('GitHub source “{source}” is rate-limited. Retry in {seconds} seconds (no reset time was provided).', params)
    : t('GitHub source “{source}” is rate-limited. Try again in {seconds} seconds.', params)
  return value.canRetry ? t('{message} Other routes can be checked again.', { message }) : message
}

export function githubErrorMessage(error: unknown, t: Translator, settings: 'update' | 'plugin'): string {
  const message = error instanceof Error ? error.message : String(error ?? '')
  const sourceLimit = /^GitHub 源「(.+)」限流，(?:请在 )?(\d+) 秒后(?:重试|可重试)(（上游未提供恢复时间）)?(；其他线路可重新检查)?$/.exec(message)
  if (sourceLimit) {
    const params = { source: sourceLimit[1] === 'Official source' ? t('Official source') : sourceLimit[1], seconds: sourceLimit[2] }
    const text = sourceLimit[3]
      ? t('GitHub source “{source}” is rate-limited. Retry in {seconds} seconds (no reset time was provided).', params)
      : t('GitHub source “{source}” is rate-limited. Try again in {seconds} seconds.', params)
    return sourceLimit[4] ? t('{message} Other routes can be checked again.', { message: text }) : text
  }
  if (message === 'The previous GitHub rate-limit wait has ended. You can check again.') return t('The previous GitHub rate-limit wait has ended. You can check again.')
  if (message === 'The GitHub API is rate-limited. Enter your credentials under GitHub Token in Update settings to increase the request limit.'
    || message === 'The GitHub API is rate-limited. Enter your credentials under GitHub Token in Plugin settings to increase the request limit.') {
    return settings === 'plugin'
      ? t('The GitHub API is rate-limited. Enter your credentials under GitHub Token in Plugin settings to increase the request limit.')
      : t('The GitHub API is rate-limited. Enter your credentials under GitHub Token in Update settings to increase the request limit.')
  }
  const cooldown = /^GitHub 接口限流中，请在 (\d+) 秒后重试$/.exec(message)
  if (cooldown) return t('The GitHub API is rate-limited. Try again in {seconds} seconds.', { seconds: cooldown[1] })
  if (message === 'The GitHub API is rate-limited. Try again later.') return t('The GitHub API is rate-limited. Try again later.')
  return message
}
