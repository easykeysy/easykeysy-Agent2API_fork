import type { ScriptPackage, ScriptPluginsState, ScriptRecommendation, ScriptUpdate } from './plugins'
import type { PluginTranslate } from './plugins-store'

import { scriptPublisher, type ScriptPublisherInfo } from './plugins-filters'

export function scriptSourceLabel(item: ScriptPublisherInfo, t: PluginTranslate): string {
  switch (scriptPublisher(item)) {
    case 'official': return t('Magpie official')
    case 'recommended': return t('Magpie recommended')
    default: return t('Third-party')
  }
}

export function scriptOfficialBadge(item: ScriptPublisherInfo): boolean {
  return scriptPublisher(item) !== 'third-party'
}

/** 这个已安装插件有没有新版本（后台那份检查结果；没检查到就没有）。 */
export function scriptUpdate(state: ScriptPluginsState | undefined, id: string): ScriptUpdate | undefined {
  return (state?.updates?.updates || []).find(item => item.id === id && item.available === true)
}

/**
 * 「可更新」徽章的文案。
 *
 * npm 与 magpie 分发来源的最新版是版本号，就写出来（「可更新 v0.2.3」）；GitHub
 * 来源的最新版是一个提交短号，数字开头之外的情况不拼 `v` —— 「可更新 v6f3a1b」
 * 读起来像版本号，会让人以为它真有版本。
 */
export function updateLabel(update: ScriptUpdate, t: PluginTranslate): string {
  return /^\d/.test(update.latest) ? t('v{version} available', { version: update.latest }) : t('Updates available')
}

/**
 * 「按原来源更新／重装」要提交的东西。
 *
 * magpie 源与 npm 源把版本留空 = 让后端取最新（`magpie:` 走官方分发索引的
 * latest、npm 走 dist-tags.latest），GitHub 源带原来的分支/标签（后端按它的
 * 当前提交装）。三处入口（卡片上的「更新」、设置面板的「更新／重装」、
 * 推荐卡的「获取」）必须是同一份请求，别各写一遍。
 */
export function reinstallRequest(pack: ScriptPackage): { id: string; name: string; version: string } {
  return {
    id: pack.id,
    name: reinstallSource(pack),
    version: pack.source?.kind === 'github' ? pack.source.ref : '',
  }
}

export function scriptRecommendations(state: ScriptPluginsState): ScriptRecommendation[] {
  const items = state.recommendations ?? (state.recommended ? [state.recommended] : [])
  return items.filter(item => (!item.kind || item.kind === 'provider')
    && (item.providerVerified === true || item.providers.length > 0))
}

function catalogSummary(summary?: Record<string, string> | null): string {
  const language = document.documentElement.lang || 'zh-CN'
  return summary?.[language] || summary?.[language.split('-')[0]] || summary?.en || summary?.zh || ''
}

export function recommendationKey(item: ScriptRecommendation): string {
  return `${item.name}|${item.source || item.name}`
}

export function isRecommendationInstalled(item: ScriptRecommendation, packages: ScriptPackage[]): boolean {
  return packages.some(pack => {
    if (pack.name !== item.name || pack.source?.kind === 'local') return false
    if (pack.source?.kind === 'github') return Boolean(item.repository && pack.source.repository.toLowerCase() === item.repository.toLowerCase())
    if (pack.source?.kind === 'magpie') return scriptPublisher(item) !== 'third-party'
    return !item.source?.startsWith('github:')
  })
}

export function recommendedDescription(item: ScriptRecommendation, t: PluginTranslate): string {
  const summary = catalogSummary(item.summary)
  if (summary) return summary
  switch (item.providers[0]) {
    case 'agy-cli': return t('Uses the agy CLI installed and signed in on your gateway computer for models, tool calls and quota queries over ACP. This is different from the Antigravity ACP plugin that relies on the Antigravity IDE.')
    case 'minimax-code': return t('Device-code sign-in, chat, model catalog, balance and daily check-in for MiniMax Code. Enable the China and Global providers separately.')
    case 'deepaicode': return t('Connect DeepAI Code with web sign-in, local credentials, model catalogs, quota queries and check-in.')
    case 'cloudflare-workers-ai': return t('Connect Workers AI chat and its online model catalog using a Cloudflare Account ID and API Token.')
    case 'codar': return t('Connect Codar with local credentials, model catalogs, points and rolling-window check-in.')
    case 'joycode': return t('Connect JoyCode (JD) with web sign-in, enterprise tenants, model catalogs and quota details.')
    case 'agnes-code': return t('Connect Agnes Code using a local session or accessToken, with model catalogs, points and observed sign-in rewards.')
    case 'river': return t('Connect River chat and its online model catalog via an API Key. The plugin does not report a balance amount.')
    case 'agnes': return t('Connect Agnes AI international with browser sign-in, an online model catalog, plan usage and a USD balance. Separate from the Agnes Code desktop product.')
    case 'warp': return t('Use your Warp terminal sign-in to access the account\'s available models and view its AI request allowance.')
    case 'intern-inkstone': return t('Connect the Intern InkStone Token Plan with browser sign-in, an API key, a model catalog and ink-credit quota windows.')
    case 'gcp-ai': return t('Use gcloud / ADC credentials on the gateway machine to access Vertex AI models and view Google Cloud promotional credits.')
    case 'sap-aicore': return t('Connect existing SAP AI Core deployments by importing a credential file from the gateway machine and listing models per resource group. Does not create deployments or report quotas.')
    case 'mclaw': return t('Connect China Mobile Cloud MClaw with browser sign-in, an online model catalog, aitokens allowance and credential renewal.')
    case 'wps-lingxi': return t('Connect WPS Lingxi with web sign-in, client import, cookies, model catalogs and credits. Check-in and bundled tasks are allowed by default.')
    default: return item.providers.length
      ? t('Connects {providers} through a community plugin. Manage providers in settings and check-in schedules in Check-in Center.', { providers: t(item.label) })
      : t('The author has not provided a plugin description.')
  }
}

export function scriptPluginName(pack: ScriptPackage, recommended: ScriptRecommendation[], _t: PluginTranslate): string {
  return pack.displayName || recommended.find(item => isRecommendationInstalled(item, [pack]))?.label || pack.name
}

export function scriptPluginDescription(pack: ScriptPackage, recommended: ScriptRecommendation[], t: PluginTranslate): string {
  const summary = catalogSummary(pack.summary)
  if (summary) return summary
  const item = recommended.find(item => isRecommendationInstalled(item, [pack]))
  if (item) return recommendedDescription(item, t)
  if (pack.providers.length) {
    return t('Connects {providers} through a community plugin. Manage providers in settings and check-in schedules in Check-in Center.', {
      providers: pack.providers.map(provider => t(provider.label)).join(' / '),
    })
  }
  return t('The author has not provided a plugin description.')
}

export function matchesScriptPlugin(pack: ScriptPackage, recommended: ScriptRecommendation[], query: string, t: PluginTranslate): boolean {
  const keyword = query.trim().toLocaleLowerCase()
  return !keyword || [pack.id, pack.name, pack.repository, pack.source?.kind === 'github' ? pack.source.repository : '', scriptPluginName(pack, recommended, t),
    scriptPluginDescription(pack, recommended, t), scriptSourceLabel(pack, t), ...pack.providers.map(provider => `${provider.id} ${t(provider.label)}`)]
    .join(' ').toLocaleLowerCase().includes(keyword)
}

export function matchesRecommendedPlugin(item: ScriptRecommendation, query: string, t: PluginTranslate): boolean {
  const keyword = query.trim().toLocaleLowerCase()
  return !keyword || [item.name, item.source, item.repository, item.label, t(item.label), scriptSourceLabel(item, t), ...item.providers, recommendedDescription(item, t)]
    .join(' ').toLocaleLowerCase().includes(keyword)
}

export function reinstallSource(pack: ScriptPackage): string {
  if (pack.source?.kind === 'local') return `file:${pack.source.path}`
  if (pack.source?.kind === 'github') return `github:${pack.source.repository}`
  if (pack.source?.kind === 'magpie') return `magpie:${pack.name}`
  return pack.name
}
