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
    case 'windows_setup_required': return t('当前 Windows 服务尚未接入一键更新。请解压新版服务端包，以管理员身份运行一次 install-service.ps1，原有账号、目录、端口和配置会保留。')
    case 'windows_update_permissions': return t('Windows 服务升级权限或目录保护不符合要求。请以管理员身份重跑新版 install-service.ps1；自定义服务账号或外部托管环境请按原方式升级。')
    case 'disabled': return t('管理员已关闭面板一键更新，请使用手动升级方式。')
    case 'helper_missing': return t('当前安装尚未接入一键更新。请下载新版服务端包，执行一次 sudo bash install.sh --upgrade；之后即可在面板更新。')
    case 'helper_unavailable': return t('一键更新组件不可用或权限配置不正确。请使用新版安装包执行 sudo bash install.sh --upgrade 修复。')
    case 'custom_installation': return t('检测到自定义程序或界面路径，一键更新不会覆盖这些位置。请按原部署方式升级。')
    case 'unsupported_arch': return t('当前系统或架构暂不支持面板自动安装，请查看发布页。')
    case 'windows_service': return t('Windows 服务部署需要手动升级。请解压新版服务端包，以管理员身份运行 install-service.ps1；账号与配置会保留。')
    default: return t('当前为直接运行的服务端，请停止程序后替换服务端包，再重新启动。')
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
 * 「 has new ， no has can down 」 when after to （check response `assetProbe`）。
 *
 * ── Why need need ────────────────────────────────────────────
 * `asset` as empty not Reason，Reason not ： up no 
 * （ etc / down ）、 （ up ）、 update 
 * （「updateset」）。 this is use 。
 *
 * optional： before that check no has ，must can to 「 not Reason」。
 */
export type AssetProbe = {
  /** this 1 release up file（ by update** before ** ） */
  assetsTotal?: number
  /** this file after （、、） */
  suffixes?: string[]
  /** has fileupdate （ before / API） no */
  filteredBySource?: boolean
  /** need after （；update is empty） */
  expectedSuffixes?: string[]
  /** need file（update； is null） */
  expectedName?: string | null
}

/**
 * 「Why no has can down 」——toast / / use this 1 。
 *
 * return is 1 can **Reason**， by use （ and 
 * before not ， update-panel and update-modal）。 not to when 「 need 
 * 」 this 1 —— has to ， not is 1 「 no has 」。
 */
export function assetMissingReason(
  probe: AssetProbe | null | undefined,
  options: { selfUpdate?: boolean; fallbackSuffix?: string; serverPackage?: boolean },
  t: Translator,
): string {
  // ① update ：file its ， is this 。
  // this —— （「updateset」） and not 
  if (probe?.filteredBySource) {
    return t('发布页上的文件被更新源的访问方式过滤掉了，请检查「更新设置」里的 GitHub 访问方式')
  }
  // ② 1 file no ：failure，or in up 
  if (probe?.assetsTotal === 0) {
    return t('这一版还没有上传任何文件（构建可能失败，或仍在打包上传）')
  }
  // ③ file， no has need that 1 —— 「 has 」 and 「 need 」。
  // after to expected （Linux is [deb, AppImage] ），
  // after no to when to use by installerKind that after 
  const seen = (probe?.suffixes || []).filter(Boolean).slice(0, 6)
  const serverPackage = options.serverPackage || options.selfUpdate
  const wanted = serverPackage
    ? String(probe?.expectedName || '')
    : ((probe?.expectedSuffixes ?? [options.fallbackSuffix || '']).filter(Boolean).join(' / '))
  if (seen.length && wanted) {
    return t('未找到适用的更新包。需要：{wanted}；发布页包含的文件类型：{seen}。', { seen: seen.join(' / '), wanted })
  }
  // ④ ：「 has 」 not ，「 need 」
  if (serverPackage) {
    return wanted
      ? t('发布页上没有 {wanted} 这个文件', { wanted })
      : t('发布页上没有对应本机架构的服务端更新包')
  }
  return wanted
    ? t('这一版没有发布本机需要的安装包（{wanted}）', { wanted })
    : t('这一版没有为当前平台提供安装包')
}

export function githubErrorMessage(error: unknown, t: Translator, settings: 'update' | 'plugin'): string {
  const message = error instanceof Error ? error.message : String(error ?? '')
  if (message === 'GitHub 接口限流中，请在「更新设置」的「GitHub 令牌」中填写凭证，以提高请求限额。'
    || message === 'GitHub 接口限流中，请在「插件设置」的「GitHub 令牌」中填写凭证，以提高请求限额。') {
    return settings === 'plugin'
      ? t('GitHub 接口限流中，请在「插件设置」的「GitHub 令牌」中填写凭证，以提高请求限额。')
      : t('GitHub 接口限流中，请在「更新设置」的「GitHub 令牌」中填写凭证，以提高请求限额。')
  }
  const cooldown = /^GitHub 接口限流中，请在 (\d+) 秒后重试$/.exec(message)
  if (cooldown) return t('GitHub 接口限流中，请在 {seconds} 秒后重试', { seconds: cooldown[1] })
  if (message === 'GitHub 接口限流中，请稍后重试') return t('GitHub 接口限流中，请稍后重试')
  return message
}
