import type { Translate } from './assistant-read-types'

export type ClientEvidence = {
  userAgent: string
  userAgentTruncated: boolean
  headers: Record<string, string>
  truncatedHeaders: string[]
  invalidHeaders: string[]
  method: string
  ruleId: string
  matchedHeader: string
  matchedPattern: string
  clientId: string
  clientName: string
  candidate: string
  declarationStatus: string
}
export type ClientRule = {
  id: string
  enabled: boolean
  header: string
  matchType: 'product' | 'prefix' | 'contains' | 'exact'
  pattern: string
  clientId: string
}
export type CustomClient = { id: string; name: string }
export type ClientSettings = { acceptDeclarations: boolean; clients: CustomClient[]; rules: ClientRule[] }
export type ClientIdentificationView = {
  config: ClientSettings
  revision: string
  /** `ua` 是该客户端一份代表性的 User-Agent（账号级自定义 UA 的预置项，见后端 catalog） */
  builtins: { id: string; name: string; value: string; products: string[]; ua?: string }[]
  headers: string[]
  declarationHeader: string
  maxRules: number
  maxClients: number
}
export type IdentificationSample = {
  id?: string
  ts?: number
  before: string
  client: string
  clientLabel: string
  evidence: ClientEvidence
  ruleMatches: boolean
  ruleApplied: boolean
}
export type IdentificationPreview = {
  scanned: number
  matched: number
  applied: number
  window: number
  samples: IdentificationSample[]
  sample: IdentificationSample | null
}
export type ClientIdentificationApi = {
  load(): Promise<ClientIdentificationView>
  save(payload: { config: ClientSettings; revision: string }): Promise<ClientIdentificationView>
  preview(payload: { config: ClientSettings; ruleId?: string; sample?: ClientEvidence }): Promise<IdentificationPreview>
}

export const IDENTIFICATION_CHANGED = 'wb:client-identification-changed'
export function notifyIdentificationChanged(): void {
  window.dispatchEvent(new Event(IDENTIFICATION_CHANGED))
}

export function clientMethodLabel(method: string | undefined, t: Translate): string {
  switch (method) {
    case 'builtin_ua': return t('Built-in User-Agent rule')
    case 'builtin_header': return t('Client-specific header')
    case 'custom_rule': return t('Custom identification rules')
    case 'declared': return t('Caller declaration')
    case 'unknown': return t('No known client matched')
    case 'ambiguous': return t('Conflicting client identifiers')
    case 'invalid': return t('Invalid or duplicate client identifier')
    case 'missing': return t('No client identifier provided')
    default: return t('Identification evidence was not recorded')
  }
}

export function blankRule(evidence?: ClientEvidence): ClientRule {
  const header = evidence?.matchedHeader || 'user-agent'
  const product = evidence?.matchedPattern || evidence?.candidate
    || evidence?.userAgent.trim().split('/')[0] || ''
  return {
    id: `rule-${randomId()}`, enabled: true, header,
    matchType: header === 'user-agent' ? 'product' : 'exact',
    pattern: header === 'user-agent' ? product : evidence?.headers[header] || '',
    clientId: evidence?.clientId || '',
  }
}

export function randomId(): string {
  const bytes = crypto.getRandomValues(new Uint8Array(12))
  return Array.from(bytes, byte => byte.toString(16).padStart(2, '0')).join('')
}

export function clientSettingsError(config: ClientSettings, view: ClientIdentificationView, t: Translate): string {
  if (config.rules.length > view.maxRules) return t('Identification rule limit reached')
  if (config.clients.length > view.maxClients) return t('Custom client limit reached')
  const names = new Set(view.builtins.flatMap(client => [client.name, client.id, client.value, ...client.products]).map(name => name.toLowerCase()))
  for (const client of config.clients) {
    if (!client.name.trim() || client.name.trim() !== client.name || client.name.length > 80) return t('Enter a client name of up to 80 characters')
    if (names.has(client.name.toLowerCase())) return t('Client name is duplicated or conflicts with a built-in name')
    names.add(client.name.toLowerCase())
  }
  const clients = new Set([...view.builtins, ...config.clients].map(client => client.id))
  for (const rule of config.rules) {
    if (!rule.pattern.trim() || rule.pattern.trim() !== rule.pattern || rule.pattern.length > 256) return t('Enter a match pattern of up to 256 characters')
    if (!clients.has(rule.clientId)) return t('Select the client to assign')
    if (!view.headers.includes(rule.header)) return t('Select an available identity header')
    if (rule.matchType === 'product' && (rule.header !== 'user-agent' || !/^[a-z0-9_. -]+$/i.test(rule.pattern))) return t('Product names cannot include slashes or versions. Use prefix matching.')
  }
  return ''
}
