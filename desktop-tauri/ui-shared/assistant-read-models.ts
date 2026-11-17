import type { AssistantReadTool, Translate } from './assistant-read-types'
import { booleanArg, enumArg, number, object, page, result, safeText, stringArg, strings, text } from './assistant-read-data'
import { CAPABILITY_KEYS, capSourceOf, isTokenKey, normalizeCapabilities, type CapabilityKey, type CapabilitySource } from './model-capability'
import { modelReasoning, type ReasoningModel } from './models-reasoning'

export type ModelProviderRow = { provider?: unknown; providerLabel?: unknown }
export type AssistantModelRow = ModelProviderRow & ReasoningModel & {
  id?: unknown
  name?: unknown
  enabled?: unknown
  aliases?: unknown
  kind?: unknown
  supports_image_generation?: unknown
  supports_image_editing?: unknown
  capabilities?: unknown
  capSources?: unknown
}
export type AssistantMappingRow = ModelProviderRow & {
  alias?: unknown
  target?: unknown
  enabled?: unknown
  reasoning?: unknown
  isDefault?: unknown
  dangling?: unknown
}
export type AssistantModelView = {
  models: AssistantModelRow[]
  mappings: AssistantMappingRow[]
  providers?: ModelProviderRow[]
}

type Provider = { id: string; label: string }
const PAGE_DEFAULT = 100
const PAGE_MAX = 500

function providerDirectory(data: AssistantModelView): Provider[] {
  const providers = new Map<string, Provider>()
  for (const row of [...(data.providers || []), ...data.models, ...data.mappings]) {
    const id = text(row.provider)
    if (!id) continue
    const key = id.toLowerCase()
    const label = text(row.providerLabel) || id
    const current = providers.get(key)
    if (!current || current.label === current.id) providers.set(key, { id, label })
  }
  return [...providers.values()]
}

function resolveProvider(providers: Provider[], wanted: string, t: Translate): string {
  const key = wanted.toLowerCase()
  const byId = providers.find(provider => provider.id.toLowerCase() === key)
  if (byId) return byId.id.toLowerCase()
  const exact = providers.filter(provider => provider.label.toLowerCase() === key)
  const matches = exact.length ? exact : providers.filter(provider =>
    provider.id.toLowerCase().includes(key) || provider.label.toLowerCase().includes(key))
  if (!matches.length) throw new Error(t('Provider "{name}" was not found. Query without provider first, or search with keyword', { name: safeText(wanted) }))
  if (matches.length > 1) throw new Error(t('Provider "{name}" matches multiple entries. Use a provider ID: {list}', {
    name: safeText(wanted), list: matches.map(provider => `${safeText(provider.label)} (${provider.id})`).join(', '),
  }))
  return matches[0].id.toLowerCase()
}

function modelCapabilities(row: AssistantModelRow) {
  const raw = object(row.capabilities)
  // 管理接口只接受数字和布尔值；不能把 true 或字符串误读成上下文大小。
  const values: Record<string, number | boolean> = {}
  for (const key of CAPABILITY_KEYS) {
    const value = raw[key]
    if (isTokenKey(key) ? typeof value === 'number' : typeof value === 'boolean') values[key] = value as number | boolean
  }
  const capabilities = normalizeCapabilities(values)
  const capSources: Partial<Record<CapabilityKey, CapabilitySource>> = {}
  for (const key of CAPABILITY_KEYS) {
    if (!(key in capabilities)) continue
    // 自定义清单的能力值来自用户保存的覆盖，不是上游探测结果。
    const source = text(row.provider).startsWith('custom-') ? 'user' : capSourceOf(row, key)
    if (source) capSources[key] = source
  }
  const reasoning = modelReasoning(row)
  return {
    capabilities,
    capSources,
    ...(reasoning.declared ? { reasoningLevels: reasoning.levels } : {}),
    ...(reasoning.defaultLevel ? { reasoningDefaultLevel: reasoning.defaultLevel } : {}),
    ...(reasoning.fixedLevel ? { reasoningFixedLevel: reasoning.fixedLevel } : {}),
    ...(reasoning.bindingSupported !== null ? { reasoningBindingSupported: reasoning.bindingSupported } : {}),
    ...(reasoning.source ? { reasoningSource: reasoning.source } : {}),
  }
}

function modelPage<T>(rows: T[], offset: number, limit: number) {
  const start = Math.min(offset, rows.length)
  return page(rows.slice(start), start, limit, rows.length)
}

export function createModelListTool(read: () => Promise<AssistantModelView>, t: Translate): AssistantReadTool {
  return {
    id: 'list_models',
    label: t('List models'),
    kind: 'read',
    desc: 'Paginated read of the built-in and custom providers\' model lists, display names, enabled state, and mappings, plus the context window, max output tokens, tool calling, image input, video input, and reasoning capabilities from capabilities. Query this before answering questions about model capabilities, context size, or provider ownership. capabilities contains only declared values: false means explicitly unsupported; a missing value means unknown and must not be treated as false or 0",; capSources — see the upstream / modelsdev / user  refer respectively to the upstream catalog / community data / user configuration, and are not measured at runtime. reasoningLevels are the reasoning tiers the model has declared; do not guess when they are missing — the fixed tiers, the default tier, and whether the tier is configurable are returned together with the declared data. Image input is different from the generation and editing capabilities in supports_image_generation / supports_image_editing. provider Supported: an ID, a display name, or a unique name fragment; when names collide, use the ID",; keyword You can search both model and provider names. Models and mappings are paginated separately; when hasMore is true, keep reading with nextOffset — do not estimate totals from a single page.",',
    params: {
      type: 'object',
      properties: {
        provider: { type: 'string', description: 'Provider ID, display name (including Chinese), or a uniquely matching name fragment; prefer exact matches, use the ID when names collide; leave blank",= All' },
        keyword: { type: 'string', description: 'Per model ID, model name, mapping name, provider ID or display name as a substring filter; if empty= No filter' },
        enabledOnly: { type: 'boolean', description: 'true = Only show enabled models or mappings' },
        section: { type: 'string', enum: ['models', 'mappings', 'both'], description: 'models = model, mappings= mapping, both= two segments (default); the two segments paginate separately' },
        offset: { type: 'number', description: 'The starting point of this page (from 0); when paging, pass the previous page\'s nextOffset' },
        limit: { type: 'number', description: `每页最多条数（默认 ${PAGE_DEFAULT}，上限 ${PAGE_MAX}）；能力信息较多时会缩小本页，请以 nextOffset 为准` },
      },
    },
    run: async args => {
      const wanted = stringArg(args, 'provider', t)
      const keyword = stringArg(args, 'keyword', t).toLowerCase()
      const enabledOnly = booleanArg(args, 'enabledOnly', t) === true
      const section = enumArg(args, 'section', ['models', 'mappings', 'both'], 'both', t)
      const offset = Math.max(0, Math.round(number(args.offset) ?? 0))
      const limit = Math.max(1, Math.min(PAGE_MAX, Math.round(number(args.limit) ?? PAGE_DEFAULT)))
      const data = await read()
      const providers = providerDirectory(data)
      const provider = wanted ? resolveProvider(providers, wanted, t) : ''
      const labels = new Map(providers.map(item => [item.id.toLowerCase(), item.label]))
      const labelOf = (row: ModelProviderRow) => labels.get(text(row.provider).toLowerCase()) || text(row.providerLabel) || text(row.provider)
      const matches = (row: ModelProviderRow & { enabled?: unknown }, values: string[]) => {
        if (provider && text(row.provider).toLowerCase() !== provider) return false
        if (enabledOnly && row.enabled !== true) return false
        return !keyword || [...values, text(row.provider), labelOf(row)].join(' ').toLowerCase().includes(keyword)
      }
      const models = data.models.filter(row => matches(row, [text(row.id), text(row.name), ...strings(row.aliases)]))
      const mappings = data.mappings.filter(row => matches(row, [text(row.alias), text(row.target)]))
      const payload: Record<string, unknown> = {}
      if (section !== 'mappings') {
        payload.models = modelPage(models.map(row => ({
          provider: text(row.provider), providerLabel: labelOf(row),
          id: text(row.id), name: text(row.name), enabled: row.enabled === true,
          kind: text(row.kind) || 'chat',
          supports_image_generation: row.supports_image_generation === true,
          supports_image_editing: row.supports_image_editing === true,
          aliases: strings(row.aliases),
          ...modelCapabilities(row),
        })), offset, limit)
      }
      if (section !== 'models') {
        payload.mappings = modelPage(mappings.map(row => ({
          provider: text(row.provider), providerLabel: labelOf(row),
          alias: text(row.alias), target: text(row.target), enabled: row.enabled === true,
          isDefault: row.isDefault === true, dangling: row.dangling === true,
        })), offset, limit)
      }
      return result(payload, section === 'mappings'
        ? t('{n} mappings', { n: mappings.length }) : t('{n} models', { n: models.length }))
    },
  }
}
