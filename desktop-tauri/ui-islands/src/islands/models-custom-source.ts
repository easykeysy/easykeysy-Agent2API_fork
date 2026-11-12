/**
 * Agent2API · （ in when and ）。
 *
 * from ui/models-custom-source.js TS， window 。
 * ：
 *
 * 1. ****： this （`models` / `mappings` ） and 
 * `GET /api/models/manage` `{models, mappings}` —— is 、、
 * chip、 etc 、 can 、set use ， 1 not use as ；
 * 2. ****： this no has ， has 「」（`POST /api/custom-providers/models`）。
 * 「current → use this 1 → → new directory」， is up 
 * 1 （ / / etc / can / ）****， no has by 、 no has ；
 * 3. ****：`fetch-models` by server up ， new and after 。
 *
 * ── Why in ， not is and after manage_view ──
 * after `core/custom_providers.rs` 「modelRules disabled/hidden ** not used for**
 * 」： then in modelRules up （ is 、 1 can in 
 * 1 ）， is use 、 in 。
 * not ， and need ， is 「 1 before 」。 this is ****。
 *
 * ── （alias == target） is ──────────────
 * will 「 ID default」 mappings 。default by 
 * （ = `models[].enabled`、 etc = `models[].reasoning`），： when 
 * ** and default**， not in 1 ； when ** not **（ draftOf ）——
 * otherwise use default ， down 1 will that old 。
 *
 * directory（`wbProviders.customList`） is 1 ： by providers.js 1 and 
 * new ， not GET， in success after `refreshCustom()` 1 。
 */

import { activeModelBindings, disableOtherModelBindings, normalizeModelBindingStates } from '../../../ui-shared/models-bindings'
import { CAPABILITY_KEYS, normalizeCapabilities } from './model-capability'
import { t } from '../i18n'

/* ─── for （，models-page.tsx this ）────── */

/** 1 （ after `catalog::manage_view` models[] and ） */
export type ManageModel = {
  id: string
  name: string
  provider: string
  providerLabel: string
  /** 'remote' / 'builtin' / 'manual'；empty（ that by ） */
  source: string
  kind?: string
  supports_image_generation?: boolean
  supports_image_editing?: boolean
  output_modalities?: string[]
  enabled: boolean
  aliases: string[]
  /** ： this when （ms，0 = unknown）； no has */
  refreshedAt?: unknown
  credits?: unknown
  /**
   * for down can （： + use ）。 by after to 
   * 「、null = 」 ； by ****
   * （ no not ）—— 1 `model-capability` 1 /，
   * not will 。
   */
  capabilities?: unknown
  /** use can （ has to ； 1 is use ，empty） */
  capOverrides?: unknown
  /**
   * can ****（`{: 'upstream' | 'modelsdev' | 'user'}`， =
   * that 1 no has ）。 「 up / / use 」 ——
   * 1 、 can （ `core::model_meta` module header）。
   */
  capSources?: unknown
  /**
   * ** can **（， by after in 1 ： this 
   * > models.dev effort ； = ， use ）。
   * ****， not and ： use can is 「 use 1 」
   * （`ManageMapping.reasoning`）， not up 。
   */
  reasoningLevels?: unknown
  /** default（ and `reasoningLevels` ； = ） */
  reasoningDefaultLevel?: unknown
  reasoningFixedLevel?: unknown
  reasoningBindingSupported?: unknown
  reasoningSource?: unknown
  /**
   * ** can **： this in client is will 。
   *
   * `'shown'` / `'hidden'` / `'undeclared'`， by `models.rs` in 
   * （：Antigravity by 、Qoder up `enable`、Raccoon 
   * `visible`、CodeArts `display_enabled`）。
   *
   * ** / `null`** = this no has can （current is that all）。
   * this and `'shown'` is ： before is 「 this no this 」， after is 「 this can 」。
   * must ，otherwise use will as 「 no has 」 is 「 can 」。
   */
  visibility?: unknown
  upstreamStatus?: 'undeployed' | null
}

/** 1 （ default：alias == target） */
export type ManageMapping = {
  alias: string
  target: string
  provider: string
  enabled: boolean
  reasoning: string
  isDefault?: boolean
  dangling?: boolean
  carried?: boolean
}

export type ManageProvider = {
  provider: string
  providerLabel: string
  count: number
  source: 'none' | 'remote' | 'builtin'
  refreshedAt: number
}

/** 1 （ /api/models/manage， by buildView ） */
export type ManageView = {
  models?: ManageModel[]
  mappings?: ManageMapping[]
  providers?: ManageProvider[]
  /** etc （ has that response； models-reasoning.ts ） */
  reasoningLevels?: string[]
}

/** directory 1 （providers.js customList()） */
export type CustomProviderRecord = {
  id?: string
  name?: string
  createdAt?: unknown
  /** `capabilities` is this can （optional， `model-capability`） */
  models?: Array<{ id?: unknown; enabled?: unknown; reasoning?: unknown; capabilities?: unknown }>
  mappings?: Array<{ alias?: unknown; target?: unknown; enabled?: unknown; reasoning?: unknown }>
}

/** directory（providers.js） use to that */
type ProvidersBridge = {
  customList?: () => CustomProviderRecord[]
  refreshCustom?: () => Promise<unknown>
  absorbCustom?: (provider: CustomProviderRecord) => void
  customRequest?: (method: string, path: string, body?: unknown) => Promise<{ models?: unknown; provider?: CustomProviderRecord } | null | undefined>
}

/** ： and after 1 （ not to = not ） */
type BindingPatch = { enabled?: boolean; reasoning?: string; create?: boolean }

/* ─── window ─────────────────────────── */

/**
 * directory（providers.js） in index.html in ** after **（ need to 
 * use ， after ）， can **** —— will to undefined， is 
 * （「 no has 」）。
 */
type SharedWindow = {
  wbProviders?: ProvidersBridge
  /** for ；models-fetch-modal.tsx addModels */
  wbModelsCustom?: CustomSourceApi
}

function shared(): SharedWindow {
  return window as unknown as SharedWindow
}

/* ─── ─────────────────────────────────── */

/** 1 ：empty + not （ and after 1 ） */
const norm = (value: unknown): string => String(value ?? '').trim().toLowerCase()
const same = (left: unknown, right: unknown): boolean => norm(left) === norm(right)

export const ALL_CUSTOM = 'custom-all'

/** `custom-` before （ and after `is_custom_provider_id` 1 ） */
export function isCustom(id: unknown): boolean {
  return typeof id === 'string' && id !== ALL_CUSTOM && id.startsWith('custom-')
}

/** （directory ， by createdAt ） */
export function list(): CustomProviderRecord[] {
  return (shared().wbProviders?.customList?.() || []).filter(provider => isCustom(provider?.id))
}

/**
 * directory ** use **（ not 、 not ）： by 「 no 」。
 *
 * providers.js fetchCustomProviders customItems new for ，
 * use etc = directory no = valid； use 1 。 this is 
 * 1 can —— is ， for （add / delete
 * 、 will use ， this ）。
 */
export function listSource(): readonly CustomProviderRecord[] {
  return shared().wbProviders?.customList?.() || []
}

/** 1 （ not in return null —— can can ） */
export function record(id: string): CustomProviderRecord | null {
  return list().find(provider => provider.id === id) || null
}

/**
 * → can （`{models, mappings}`）。
 *
 * 1 and custom-models-modal that ： enabled 、
 * reasoning empty； and default after from mappings （file）。
 * from **directory current**， is etc ：， to 
 * is 1 after 。
 */
type DraftModel = { id: string; enabled: boolean; reasoning: string; capabilities?: Record<string, number | boolean> }
type DraftMapping = { alias: string; target: string; enabled: boolean; reasoning: string }

/**
 * can → can for （empty to `undefined`： not 
 * this ， and after 「empty not 」 1 ）。
 */
function sparseCapabilities(value: unknown): Record<string, number | boolean> | undefined {
  const normalized = normalizeCapabilities(value)
  const result: Record<string, number | boolean> = {}
  for (const key of CAPABILITY_KEYS) {
    const item = normalized[key]
    if (typeof item === 'number' || typeof item === 'boolean') result[key] = item
  }
  return Object.keys(result).length ? result : undefined
}

function draftOf(provider: CustomProviderRecord | null): { models: DraftModel[]; mappings: DraftMapping[] } {
  const models = (Array.isArray(provider?.models) ? provider.models : [])
    .map(model => {
      const draft: DraftModel = {
        id: String(model?.id ?? '').trim(),
        enabled: model?.enabled !== false,
        reasoning: typeof model?.reasoning === 'string' ? model.reasoning : '',
      }
      // can **must**： down ，， use 
      // can will 1 「」 
      draft.capabilities = sparseCapabilities(model?.capabilities)
      return draft
    })
    .filter(model => model.id)
  const mappings = (Array.isArray(provider?.mappings) ? provider.mappings : [])
    .map(mapping => ({
      alias: String(mapping?.alias ?? '').trim(),
      target: String(mapping?.target ?? '').trim(),
      enabled: mapping?.enabled !== false,
      reasoning: typeof mapping?.reasoning === 'string' ? mapping.reasoning : '',
    }))
    .filter(mapping => mapping.alias && mapping.target)

  for (const model of models) {
    const legacy = mappings.find(mapping => same(mapping.alias, model.id) && same(mapping.target, model.id))
    if (!legacy) continue
    model.enabled = model.enabled && legacy.enabled
    model.reasoning = legacy.reasoning || model.reasoning
  }
  const cleaned = mappings.filter(mapping =>
    !(same(mapping.alias, mapping.target) && models.some(model => same(model.id, mapping.target))))
  normalizeModelBindingStates(models, cleaned)
  return { models, mappings: cleaned }
}

/**
 * directory → 。return `null` = this already not in （ ）。
 *
 * and after `catalog::manage_view` for ， this ：
 *   · models: `{id, name, provider, providerLabel, source, enabled, aliases,
 *     capabilities, capOverrides}`
 * —— no has and ，`source` empty（ in when that by 
 * ， models-page visibleColumns）； can is **** ，
 * `capOverrides` empty（ no has 「 up 」 this ， `setCapabilities`）；
 *   · mappings: `{alias, target, provider, enabled, reasoning, isDefault, dangling, carried}`
 * —— default（alias == target） by ， and 1 。
 */
export function buildView(id: string): { models: ManageModel[]; mappings: ManageMapping[] } | null {
  const provider = record(id)
  if (!provider) return null
  const draft = draftOf(provider)
  const name = provider.name || id
  const models: ManageModel[] = []
  const mappings: ManageMapping[] = []
  for (const model of draft.models) {
    models.push({
      id: model.id,
      name: '',
      provider: id,
      providerLabel: name,
      source: '',
      enabled: model.enabled,
      aliases: [],
      // can is ** **（ use ）， no has 「 up 」 this ——
      // empty， by 「 / already 」
      capabilities: model.capabilities || {},
      capOverrides: [],
    })
    mappings.push({
      alias: model.id,
      target: model.id,
      provider: id,
      enabled: model.enabled,
      reasoning: model.reasoning,
      isDefault: true,
      dangling: false,
      carried: true,
    })
  }
  for (const mapping of draft.mappings) {
    const row = models.find(model => same(model.id, mapping.target))
    mappings.push({
      alias: mapping.alias,
      target: mapping.target,
      provider: id,
      enabled: mapping.enabled,
      reasoning: mapping.reasoning,
      isDefault: false,
      // not in = this not to 1 （ 「」）。
      // not this this （ will ）， file or 
      // can can down —— ， use not to 。
      dangling: !row,
      carried: Boolean(row),
    })
    if (row) row.aliases.push(mapping.alias)
  }
  return { models, mappings }
}

/* ─── ───────────────────────────────────── */

/**
 * ：current → to `mutate` → → new directory。
 * `mutate` return to use （used for toast ）。
 */
const writes = new Map<string, Promise<unknown>>()

function submit<T>(id: string, mutate: (draft: ReturnType<typeof draftOf>) => T): Promise<T> {
  const previous = writes.get(id) || Promise.resolve()
  const next = previous.catch(() => undefined).then(() => submitNow(id, mutate))
  writes.set(id, next)
  void next.then(() => { if (writes.get(id) === next) writes.delete(id) }, () => { if (writes.get(id) === next) writes.delete(id) })
  return next
}

async function submitNow<T>(id: string, mutate: (draft: ReturnType<typeof draftOf>) => T): Promise<T> {
  const providers = shared().wbProviders
  if (!providers?.customRequest) throw new Error(t('目录模块未就绪'))
  const provider = record(id)
  if (!provider) throw new Error(t('该自定义提供商已不存在（可能已被删除），请刷新后重试'))
  const draft = draftOf(provider)
  const result = mutate(draft)
  const saved = await providers.customRequest('POST', '/api/custom-providers/models', {
    providerId: id,
    models: draft.models,
    mappings: draft.mappings,
  })
  if (!saved?.provider || saved.provider.id !== id) throw new Error(t('网关未返回保存后的提供商记录'))
  if (providers.absorbCustom) providers.absorbCustom(saved.provider)
  else await providers.refreshCustom?.()
  return result
}

/**
 * 1 / etc —— that use （chip 、 etc ） ，
 * for ：
 * · `alias == target` and has this = **default**： ；
 * · otherwise by (alias, target) already has ， to 、 not to **add**（）；
 * · `patch.enabled === false` and not in = use in 1 not in ，。
 *
 * `patch` no to （ and after 「」 1 to ： not = not ）。
 */
export async function setBinding(id: string, alias: string, target: string, patch: BindingPatch = {}): Promise<void> {
  const { enabled, reasoning, create } = patch
  await submit(id, draft => {
    const existing = draft.mappings.find(item => same(item.alias, alias) && same(item.target, target))
    if (create && (same(alias, target) || existing)) {
      throw new Error(t('模型 ID「{id}」已绑定到此上游模型，请直接使用已有绑定', { id: alias }))
    }
    if (!existing && create === false && !same(alias, target)) {
      throw new Error(t('映射「{alias} → {target}」不存在', { alias, target }))
    }
    if (same(alias, target)) {
      const model = draft.models.find(item => same(item.id, target))
      if (!model) throw new Error(t('该提供商的清单里没有模型「{name}」', { name: target }))
      if (enabled === true) disableOtherModelBindings(draft.models, draft.mappings, alias, target)
      if (enabled !== undefined) model.enabled = enabled
      if (reasoning !== undefined) model.reasoning = String(reasoning ?? '')
      return null
    }
    if (existing) {
      if (enabled === true) disableOtherModelBindings(draft.models, draft.mappings, alias, target)
      if (enabled !== undefined) existing.enabled = enabled
      if (reasoning !== undefined) existing.reasoning = String(reasoning ?? '')
      return null
    }
    if (enabled === false) throw new Error(t('映射「{alias} → {target}」不存在', { alias, target }))
    if (!draft.models.some(model => same(model.id, target))) {
      throw new Error(t('该提供商的清单里没有模型「{name}」', { name: target }))
    }
    if (activeModelBindings(draft.models, draft.mappings, alias, target).length) {
      throw new Error(t('模型 ID「{id}」在该提供商已有启用绑定，请先关闭它再添加', { id: alias }))
    }
    draft.mappings.push({ alias, target, enabled: enabled ?? true, reasoning: String(reasoning ?? '') })
    return null
  })
}

/** delete 1 （default not can —— up no has delete by ） */
export async function removeMapping(id: string, alias: string, target: string): Promise<void> {
  await submit(id, draft => {
    // default（alias == target） in ： in ， can not can 。
    // already not in （、 or after ） when no has can —— and 
    // 1 （ after model_manage::remove_mapping use 「 has no has 」），delete。
    if (same(alias, target) && draft.models.some(model => same(model.id, target))) {
      throw new Error(t('原始 ID 的默认绑定不能删除，请关闭它的开关'))
    }
    const before = draft.mappings.length
    draft.mappings = draft.mappings.filter(item => !(same(item.alias, alias) && same(item.target, target)))
    if (draft.mappings.length === before) throw new Error(t('映射「{alias} → {target}」不存在', { alias, target }))
    return null
  })
}

/** 1 （， and 1 ） */
export async function addModel(id: string, modelId: string): Promise<void> {
  const value = String(modelId ?? '').trim()
  if (!value) throw new Error(t('请填写模型 ID'))
  await submit(id, draft => {
    if (draft.models.some(model => same(model.id, value))) {
      throw new Error(t('模型「{name}」已存在（忽略大小写判重）', { name: value }))
    }
    if (activeModelBindings(draft.models, draft.mappings, value, value).length) {
      throw new Error(t('模型 ID「{id}」在该提供商已有启用绑定，请先关闭它再添加', { id: value }))
    }
    draft.models.push({ id: value, enabled: true, reasoning: '' })
    return null
  })
}

/**
 * （「get」 by ）： already in ，returnadd。
 * and `fetchModels` in **** —— that is server up after and ， this is use 
 * in 。
 */
export async function addModels(id: string, modelIds: readonly unknown[]): Promise<number> {
  const values = (Array.isArray(modelIds) ? modelIds : [])
    .map(value => String(value ?? '').trim())
    .filter(Boolean)
  if (!values.length) throw new Error(t('没有选中任何模型'))
  return submit(id, draft => {
    let added = 0
    for (const value of values) {
      if (draft.models.some(model => same(model.id, value))) continue
      if (activeModelBindings(draft.models, draft.mappings, value, value).length) {
        throw new Error(t('模型 ID「{id}」在该提供商已有启用绑定，请先关闭它再添加', { id: value }))
      }
      draft.models.push({ id: value, enabled: true, reasoning: '' })
      added++
    }
    return added
  })
}

/**
 * can （ ； `writeCapabilities` 
 * `POST /api/models/capabilities`）。
 *
 * can in ** models[] **（ and / etc 1 
 * ）， is that has path：current → this 1 → 。
 * `patch` and after 1 ： = not 、`null` = this 1 、 has = ；
 * to 1 not when capabilities 1 and （ and after 「empty not 」 1 ）。
 */
export async function setCapabilities(
  id: string,
  modelId: string,
  patch: Record<string, number | boolean | null>,
): Promise<void> {
  await submit(id, draft => {
    const model = draft.models.find(item => same(item.id, modelId))
    if (!model) throw new Error(t('该提供商的清单里没有模型「{name}」', { name: modelId }))
    const next: Record<string, number | boolean> = { ...(model.capabilities || {}) }
    for (const [key, value] of Object.entries(patch)) {
      if (value === null || value === undefined) delete next[key]
      else next[key] = value
    }
    model.capabilities = Object.keys(next).length ? next : undefined
    return null
  })
}

/** 1 ： target to ，return（used for toast） */
export async function removeModel(id: string, modelId: string): Promise<{ removedMappings: number }> {
  return submit(id, draft => {
    if (!draft.models.some(model => same(model.id, modelId))) {
      throw new Error(t('该提供商的清单里没有模型「{name}」', { name: modelId }))
    }
    const before = draft.mappings.length
    draft.mappings = draft.mappings.filter(mapping => !same(mapping.target, modelId))
    const removedMappings = before - draft.mappings.length
    draft.models = draft.models.filter(model => !same(model.id, modelId))
    return { removedMappings }
  })
}

/**
 * from up and and （ for 「 new 」）：server、 not ， already in id
 * （ignore），add that 。
 */
export async function fetchModels(id: string): Promise<{ total: number; added: number }> {
  const providers = shared().wbProviders
  if (!providers?.customRequest) throw new Error(t('目录模块未就绪'))
  const data = await providers.customRequest('POST', '/api/custom-providers/fetch-models', { providerId: id })
  const ids = (Array.isArray(data?.models) ? data.models : [])
    .map(value => String(value ?? '').trim())
    .filter(Boolean)
  const added = await submit(id, draft => {
    let count = 0
    for (const value of ids) {
      if (draft.models.some(model => same(model.id, value))) continue
      if (activeModelBindings(draft.models, draft.mappings, value, value).length) {
        throw new Error(t('模型 ID「{id}」在该提供商已有启用绑定，请先关闭它再添加', { id: value }))
      }
      draft.models.push({ id: value, enabled: true, reasoning: '' })
      count++
    }
    return count
  })
  return { total: ids.length, added: Number(added) || 0 }
}

/* ─── for ：window.wbModelsCustom ───────────── */

export type CustomSourceApi = {
  isCustom: typeof isCustom
  list: typeof list
  listSource: typeof listSource
  record: typeof record
  buildView: typeof buildView
  setBinding: typeof setBinding
  removeMapping: typeof removeMapping
  addModel: typeof addModel
  addModels: typeof addModels
  removeModel: typeof removeModel
  setCapabilities: typeof setCapabilities
  fetchModels: typeof fetchModels
}

/**
 * to window up ：** not is ** use ——
 * `ui-islands/src/islands/models-fetch-modal.tsx` 「 in N 」
 * `wbModelsCustom.addModels(providerId, values)`（grep ）。
 * （current no to fetchModels）， use when 。
 *
 * not `declare global`： this is use ， declare 1 will 
 * not 1 TS2717（ and when ）。
 */
shared().wbModelsCustom = {
  isCustom,
  list,
  listSource,
  record,
  buildView,
  setBinding,
  removeMapping,
  addModel,
  addModels,
  removeModel,
  setCapabilities,
  fetchModels,
}
