import { t } from '../i18n'
import type { AgentClient, AgentModel, AgentOperation, AgentProfile } from './agent-types'
import { modelLevels } from './agent-model-config'

export function protocolLabel(value: string): string {
  return { anthropic: 'Anthropic Messages', responses: 'OpenAI Responses', openai_chat: 'OpenAI Chat Completions', gemini: 'Gemini API', native: t('Native client configuration') }[value] || value
}
export function kindLabel(value: string): string {
  return { gateway: t('This gateway'), official: t('Official direct'), relay: t('Third-party relay'), local: t('Local service') }[value] || value
}
export function detectionLabel(client: AgentClient): string {
  return { missing: t('Not detected'), uninitialized: t('Not initialized'), read_error: t('Read failed'), unsupported: t('Read-only'), ready: t('Configuration readable and writable') }[client.detectionState]
}
export function defaultOperation(client: AgentClient): AgentOperation { return client.configMode === 'single' ? 'default' : 'apply' }
export function blankModel(): AgentModel { return { id: '', name: '', reasoningLevels: [], effort: '' } }
export function blankProfile(client: AgentClient, gateway: string): AgentProfile {
  const protocol = client.protocols[0] || ''
  return {
    id: `p-${crypto.randomUUID()}`, name: '', kind: client.supportsGateway ? 'gateway' : 'relay', protocol,
    baseUrl: client.supportsGateway ? protocol === 'anthropic' ? gateway : `${gateway}/v1` : '',
    model: '', provider: '', effort: '', enabled: true, default: false, note: '',
    models: [],
    claudeModels: { opus: '', sonnet: '', haiku: '', subagent: '' },
    authMode: client.id === 'zed' ? 'preserve' : client.supportsGateway ? 'gateway' : 'key',
    apiKey: '', envKey: '', gatewayKeyId: '',
  }
}

export function cloneProfile(profile: AgentProfile): AgentProfile { return structuredClone(profile) }

/**
 * 主模型变化后同步 Claude Code 的四档映射：只动「还跟着主模型走」的档（留空，或正好等于
 * 旧主模型），用户单独调过的档保持不动 —— 否则换个主模型就把分档配置一笔抹掉了。
 */
export function syncClaudeTiers(tiers: AgentProfile['claudeModels'], previous: string, next: string): AgentProfile['claudeModels'] {
  const follow = (value: string) => !value || value === previous ? next : value
  return { opus: follow(tiers.opus), sonnet: follow(tiers.sonnet), haiku: follow(tiers.haiku), subagent: follow(tiers.subagent) }
}

export function validateDraft(client: AgentClient, profile: AgentProfile): string {
  if (!profile.name.trim()) return t('Enter a configuration name')
  if (!client.protocols.includes(profile.protocol)) return t('This client does not support the selected protocol')
  const inherited = !profile.baseUrl && profile.authMode === 'preserve' && Boolean(profile.sourceId)
  if (!inherited) {
    try { const url = new URL(profile.baseUrl); if (!['http:', 'https:'].includes(url.protocol) || url.username || url.password || url.hash) return t('Enter a valid HTTP or HTTPS URL') }
    catch { return t('Enter a valid HTTP or HTTPS URL') }
  }
  if (profile.authMode === 'key' && !profile.apiKey && !profile.hasApiKey) return t('Enter an API key')
  if (profile.authMode === 'env' && !/^[A-Za-z_][A-Za-z0-9_]*$/.test(profile.envKey)) return t('Enter a valid environment variable name')
  if (client.supportsModels && !(['codex', 'claude-code'].includes(client.id) && inherited && !profile.models.length)) {
    if (!profile.models.length) return t('Add at least one model')
    if (profile.models.length > 200) return t('Select up to {n} models', { n: 200 })
    const ids = new Set<string>()
    for (const model of profile.models) {
      if (!model.id.trim() || ids.has(model.id.trim())) return t('Model IDs cannot be empty or duplicated')
      ids.add(model.id.trim())
      if (model.effort && model.effort !== 'default' && !modelLevels(client, model.reasoningLevels, profile.protocol).includes(model.effort)) return t('The model or client does not support the selected reasoning level')
      if (['zed', 'kimi-code', 'crush'].includes(client.id) && !model.contextWindow) return t('This client requires the model\'s context window')
      if (client.id === 'crush' && (model.maxOutputTokens == null || model.supportsImages == null || model.supportsReasoning == null)) return t('Crush requires explicit output limits, image support, and reasoning support')
      if ([model.contextWindow, model.maxOutputTokens].some(value => value != null && (!Number.isInteger(value) || value < 1 || value > 100_000_000))) return t('Token limits must be integers between 1 and 100 million')
    }
    if (client.supportsDefault && !profile.models.some(model => model.id === profile.model)) return t('The default model must be in the model list')
  } else if (client.supportsDefault && !profile.model && !inherited) return t('Enter a model ID')
  return ''
}

export function profileSignature(profile: AgentProfile): string { return JSON.stringify(profile) }
