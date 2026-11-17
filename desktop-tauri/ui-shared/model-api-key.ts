import type { ClientEvidence } from './client-identification'

export type ModelApiKey = {
  id: string
  name: string
  key: string
  masked: string
  source: 'key' | 'env'
  allowedProviders: string[]
  allowedModels: string[]
}

export function parseModelApiKeys(raw: unknown): ModelApiKey[] {
  const payload = raw && typeof raw === 'object' ? raw as { keys?: unknown } : {}
  if (!Array.isArray(payload.keys)) throw new Error('API Key The list response is incomplete')
  return payload.keys.flatMap(value => {
    if (!value || typeof value !== 'object') return []
    const entry = value as Record<string, unknown>
    if (entry.enabled === false || typeof entry.id !== 'string' || !entry.id
      || typeof entry.key !== 'string' || !entry.key) return []
    const strings = (items: unknown) => Array.isArray(items)
      ? items.filter((item): item is string => typeof item === 'string') : []
    return [{
      id: entry.id,
      name: typeof entry.name === 'string' ? entry.name : '',
      key: entry.key,
      masked: typeof entry.masked === 'string' ? entry.masked : '',
      source: entry.source === 'env' ? 'env' as const : 'key' as const,
      allowedProviders: strings(entry.allowedProviders),
      allowedModels: strings(entry.allowedModels),
    }]
  })
}

export function modelApiKeyLabel(key: ModelApiKey): string {
  return [key.name || key.id, key.masked].filter(Boolean).join(' · ')
}

export function modelApiKeyAllows(key: ModelApiKey, model: string, provider?: string): boolean {
  const includes = (values: string[], value: string) => !values.length
    || values.some(item => item.trim().toLowerCase() === value.trim().toLowerCase())
  return includes(key.allowedModels, model) && (!provider || includes(key.allowedProviders, provider))
}

export type RequestIdentity = {
  client?: string
  clientLabel?: string
  clientEvidence?: ClientEvidence
  isTest?: boolean
  apiKeyId?: string
  apiKeyName?: string
  apiKeySource?: string
}

type Translate = (text: string) => string

export function requestClientLabel(entry: RequestIdentity, t: Translate): string {
  if (entry.clientEvidence?.method === 'missing') return t('No client identifier provided')
  if (entry.client === 'other' && entry.clientEvidence?.candidate) return `${entry.clientEvidence.candidate} · ${t('Pending confirmation')}`
  switch (entry.client) {
    case 'model_test': return t('Model test')
    case 'assistant': return t('Model chat')
    case 'assistant_images': return t('Assistant image tools')
    case 'assistant_title': return t('Assistant title summary')
    case 'image-generation': return `${t('Image generation')} MCP`
    case 'checkin_keepalive': return t('Daily activity keepalive')
    case 'other': return entry.clientEvidence ? t('Unknown client') : t('Other')
    default: return entry.clientLabel || entry.clientEvidence?.clientName || entry.client || (entry.isTest ? t('Model test') : t('Unrecorded client'))
  }
}

export function requestApiKeyLabel(entry: RequestIdentity, t: Translate): string {
  if (entry.apiKeySource === 'key') return entry.apiKeyName || entry.apiKeyId || t('Untitled')
  if (entry.apiKeySource === 'env') return t('Environment variable key')
  if (entry.apiKeySource === 'none') return t('Unauthenticated calls')
  if (entry.isTest) return requestClientLabel(entry, t)
  return t('Key not recorded in history')
}
