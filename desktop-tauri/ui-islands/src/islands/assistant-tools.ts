/**
 * AI ：（ to JSON Schema）+ （ has ）。
 *
 * ── Why in before ───────────────────────────────────────
 * is will （、、 can 、、、
 * ）， this in **** up already has （`window.workbuddyDesktop`
 * ， and headless has 1 ）。 in this = 1 
 * not use Rust ； has = and by 1 。
 * after ID and after to up 、 `tool_calls` （
 * `api::assistant` module header）。
 *
 * ── （）──────────────────────────────────
 * read ： to ， not use ；
 * write ：****， use 「」；
 * run will up / （）： write， and up 。
 * 「 need not need 」 not ， use is —— this `kind` is 1 
 * ， in 1 。
 *
 * ── id is and set ─────────────────────────────────
 * `tools` is this id（ after default `config::DEFAULT_ENABLED_TOOLS`，
 * 1 ）。 = use ， = 1 。
 */

import { t } from '../i18n'
import { modelOptionsFrom, type ModelOption } from './assistant-models'
import { ASSISTANT_READ_META, ASSISTANT_READ_TOOLS } from './assistant-read-tools'
import { ASSISTANT_ACCOUNT_TOOLS } from './assistant-account-tools'
import { createBalanceTool } from '../../../ui-shared/assistant-read-balance'
import { ASSISTANT_IMAGE_TOOLS, ASSISTANT_IMAGE_META } from './assistant-image-tools'
import type { ImageAsset, ImageToolContext } from '../../../ui-shared/assistant-image-tools'
import {
  accountImportMeta,
  createAccountImportTool,
  createAttachmentReadTool,
} from '../../../ui-shared/assistant-account-import-tools'
import { callJson } from './assistant-api'
import { checkToolSignal, runToolBatch } from '../../../ui-shared/assistant-tool-execution'
import { safeText } from '../../../ui-shared/assistant-read-data'

export type { ModelOption } from './assistant-models'

/** ：`text` to （ role:"tool" ），`summary` to */
export type ToolResult = {
  text: string
  summary: string
  images?: ImageAsset[]
}

/** up （ in use before ） */
export type ToolPlan = {
  /** 1 will need （） */
  lead: string
  /** （optional；any， by use ） */
  table?: { head: string[]; rows: string[][] }
  /** （ / when / failure） */
  note?: string
}

export type AssistantTool = {
  id: string
  label: string
  kind: 'read' | 'write' | 'run'
  /** to （ in —— up for in ， and and ） */
  desc: string
  params: Record<string, unknown>
  /** / ：（ not ） */
  plan?: (args: Record<string, unknown>) => ToolPlan
  /** parameter in when （ not ） */
  redactArgs?: (args: Record<string, unknown>) => Record<string, unknown>
  run: (args: Record<string, unknown>, context?: ImageToolContext) => Promise<ToolResult>
}

/* ─── （ use to that ， when by 「 this in currentunavailable」）── */

type Bridge = {
  platform?: string
  getModelManage?: () => Promise<unknown>
  getState?: () => Promise<unknown>
  setModelState?: (payload: Record<string, unknown>) => Promise<unknown>
  addModelMapping?: (alias: string, target: string, provider: string, reasoning?: string, enabled?: boolean) => Promise<unknown>
  removeModelMapping?: (alias: string, target: string, provider: string) => Promise<unknown>
  setModelCapabilities?: (provider: string, id: string, capabilities: Record<string, unknown>) => Promise<unknown>
  testModel?: (payload: Record<string, unknown>, apiKey: string) => Promise<unknown>
  terminateStatsRequest?: (id: string) => Promise<unknown>
  getAllBalances?: (id?: string) => Promise<unknown>
  getAccounts?: () => Promise<unknown>
  getBalancesSnapshot?: () => Promise<unknown>
}

function bridge(): Bridge {
  const api = (window as unknown as { workbuddyDesktop?: Bridge }).workbuddyDesktop
  if (!api) throw new Error(t('桌面桥不可用（本页需要在 Agent2API 客户端里打开）'))
  return api
}

/** after new （ its ） */
function refreshApp(): void {
  const app = (window as unknown as { wbApp?: { refresh?: () => void } }).wbApp
  void app?.refresh?.()
}

/* ─── （ to not 1 ， 1 ）── */

function str(value: unknown, max = 256): string {
  return typeof value === 'string' ? value.trim().slice(0, max) : ''
}
function bool(value: unknown): boolean | undefined {
  return typeof value === 'boolean' ? value : undefined
}
function num(value: unknown): number | undefined {
  return typeof value === 'number' && Number.isFinite(value) ? value : undefined
}
function list(value: unknown): Array<Record<string, unknown>> {
  return Array.isArray(value) ? value.filter((item): item is Record<string, unknown> => Boolean(item) && typeof item === 'object') : []
}
function errorText(error: unknown): string {
  const message = error instanceof Error ? error.message : String(error)
  return message.replace(/^Error:\s*/, '')
}

/* ─── （ use to ）── */

type ModelRow = {
  id?: unknown
  name?: unknown
  provider?: unknown
  providerLabel?: unknown
  enabled?: unknown
  aliases?: unknown
  kind?: unknown
  supports_image_generation?: unknown
  supports_image_editing?: unknown
  /** can （`/api/models/manage` capabilities， core::providers::catalog::view） */
  capabilities?: unknown
}
type MappingRow = {
  alias?: unknown
  target?: unknown
  provider?: unknown
  providerLabel?: unknown
  enabled?: unknown
  isDefault?: unknown
  dangling?: unknown
}

/**
 * 1 and `/api/models/manage` （ after ， by 
 * `models-custom-source` `buildView` —— that its file）。
 */
type ManageViewLike = {
  models?: ModelRow[]
  mappings?: MappingRow[]
}

/** directory 1 （`providers.js` `customList`） */
type CustomProviderRecord = {
  id?: unknown
  name?: unknown
  models?: unknown
  mappings?: unknown
}

/** directory（`ui/providers.js`） in use to */
type ProvidersBridge = {
  customList?: () => CustomProviderRecord[]
  refreshCustom?: () => Promise<unknown>
}

/**
 * and 「 → 」 use that 1 。
 *
 * ── Why need use not is in this 1 ────────────────────
 * that （default、 and 、 can 、legacy ） is 
 * ****， to must and 1 ，otherwise 1 
 * in 「 to enabled」 and 「 is enabled 」 will 。
 * will this 。
 */
type CustomSourceModule = {
  isCustom: (id: unknown) => boolean
  buildView: (id: string) => { models?: ModelRow[]; mappings?: MappingRow[] } | null
  /** 1 / etc （`alias == target` default） */
  setBinding: (
    id: string,
    alias: string,
    target: string,
    patch?: { enabled?: boolean; reasoning?: string },
  ) => Promise<void>
  /** delete 1 （default not can —— and ） */
  removeMapping: (id: string, alias: string, target: string) => Promise<void>
  /** can ；`null` = this */
  setCapabilities: (
    id: string,
    modelId: string,
    patch: Record<string, number | boolean | null>,
  ) => Promise<void>
}

function customSource(): CustomSourceModule | undefined {
  return (window as unknown as { wbModelsCustom?: CustomSourceModule }).wbModelsCustom
}

/**
 * ；。
 *
 * not ：`window.wbModelsCustom` by that in up （ 1 
 * bundle、）， can can is —— that when 「success、 its 
 * no 」 1 error。
 */
function customWriter(): CustomSourceModule {
  const source = customSource()
  if (!source) throw new Error(t('自定义提供商模块未就绪（请刷新界面后重试）'))
  return source
}

/** 1 / etc */
async function customBinding(
  provider: string,
  alias: string,
  target: string,
  patch: { enabled?: boolean; reasoning?: string },
): Promise<void> {
  await customWriter().setBinding(provider, alias, target, patch)
}

/** directory（`providers.js` in head after ， can ） */
function customRecords(): CustomProviderRecord[] {
  const providers = (window as unknown as { wbProviders?: ProvidersBridge }).wbProviders
  const list = providers?.customList?.()
  return Array.isArray(list) ? list : []
}

/**
 * directory；**empty when 1 **。
 *
 * `providers.js` in 1 ， down total is has 。
 * that request can can no （ use ）， when empty will 
 * 「 1 no has 」—— that is 1 error。 1 ，
 * failure by empty（directory not not this failure）。
 */
async function customRecordsFresh(): Promise<CustomProviderRecord[]> {
  const cached = customRecords()
  if (cached.length) return cached
  const providers = (window as unknown as { wbProviders?: ProvidersBridge }).wbProviders
  try {
    await providers?.refreshCustom?.()
  } catch { /* directory not ： by has （empty） */ }
  return customRecords()
}

/**
 * is in directory（ and after `is_custom_provider_id` to before ）。
 *
 * this ** not ** `isCustom`（ before ）：directory no has `custom-xxx` is 
 * to ， that directory in that 。
 */
function isCustomProvider(id: string): boolean {
  const source = customSource()
  if (source) return source.isCustom(id)
  return id.startsWith('custom-')
}

/**
 * 1 ；directory no has this when `buildView` null。
 *
 * Note `buildView` is ****directory —— use must await 1 
 * `customRecordsFresh()`（`readModels` is this ），otherwise that 1 will to 
 * 1 no to emptydirectory。
 */
function customView(id: string): ManageViewLike | null {
  return customSource()?.buildView(id) ?? null
}

/**
 * + （ + ）。
 *
 * ── Whymust and ──────────────────────────────────────────
 * `GET /api/models/manage` （`catalog::manage_view` 
 * `active_manifests`）， 1 not in —— in 、
 * by in before 。 if that 1 ，「 has 」 when will 
 * use up ， and `set_model_*` not to enabled。
 */
async function readModels(): Promise<{ models: ModelRow[]; mappings: MappingRow[] }> {
  const read = bridge().getModelManage
  if (!read) throw new Error(t('当前客户端缺少读取接口：{name}', { name: 'getModelManage' }))
  const data = await read() as ManageViewLike | undefined
  if (!Array.isArray(data?.models) || !Array.isArray(data?.mappings)) throw new Error(t('读取模型清单失败'))
  const models: ModelRow[] = data.models.slice()
  const mappings: MappingRow[] = data.mappings.slice()
  // after 。directory（ / no to ） when 
  // is this 1 —— that available， not failure。
  const known = new Set(models.map(row => `${str(row.provider)}\u0001${str(row.id)}`.toLowerCase()))
  for (const record of await customRecordsFresh()) {
    const id = str(record.id)
    if (!id) continue
    const view = customView(id)
    if (!view) continue
    for (const row of view.models || []) {
      const key = `${id}\u0001${str(row.id)}`.toLowerCase()
      if (known.has(key)) continue
      known.add(key)
      models.push(row)
    }
    for (const mapping of view.mappings || []) mappings.push(mapping)
  }
  return { models, mappings }
}

/** can （ 1 ， down not is 1 ） */
function modelLabel(row: ModelRow): string {
  const provider = str(row.providerLabel) || str(row.provider)
  return provider ? `${provider} / ${str(row.id)}` : str(row.id)
}
/* ─── and ───────────────────────────────────── */

/** default and limit（`list_models` use ；limit 1 up down ） */
const PAGE_DEFAULT = 100
const PAGE_MAX = 500

/** 1 ：total / / is has down 1 （ and 1 ） */
type PageMeta = {
  total: number
  offset: number
  limit: number
  returned: number
  hasMore: boolean
  /** down 1 offset（`hasMore` as false when not ） */
  nextOffset?: number
}

function paginate<T>(rows: T[], offset: number, limit: number): { items: T[]; meta: PageMeta } {
  const start = Math.max(0, Math.min(offset, rows.length))
  const items = rows.slice(start, start + limit)
  const end = start + items.length
  const hasMore = end < rows.length
  return {
    items,
    meta: {
      total: rows.length,
      offset: start,
      limit,
      returned: items.length,
      hasMore,
      ...(hasMore ? { nextOffset: end } : {}),
    },
  }
}

/** 1 · get（，） */
const listModels: AssistantTool = {
  id: 'list_models',
  label: t('获取模型列表'),
  kind: 'read',
  desc: '分页读取各提供商的模型清单（模型 ID、所属提供商、是否启用、已挂的映射名）以及映射表。**内置家（Agent）与自定义提供商都在结果里**：自定义家的 provider 以 custom- 开头，可用 providerLabel 读它的展示名。回答「有哪些模型」「哪些模型在多家重复」「某个名字挂在哪」这类问题之前先调用它。\n\n结果分页：模型与映射各自一页，响应里带 total / offset / limit / returned / hasMore / nextOffset。hasMore 为 true 时用 nextOffset 作为下一次的 offset 继续取，直到取完；不要只看第一页就下结论。',
  params: {
    type: 'object',
    properties: {
      provider: { type: 'string', description: '只看某一家（提供商 id，如 codearts / workbuddy / custom-xxxx）；不填 = 全部' },
      keyword: { type: 'string', description: '按模型 ID 或映射名做子串过滤；不填 = 不过滤' },
      enabledOnly: { type: 'boolean', description: 'true = 只看已启用的模型' },
      section: {
        type: 'string',
        enum: ['models', 'mappings', 'both'],
        description: '取哪一段：models = 只要模型行、mappings = 只要映射表、both = 两段都要（默认）。翻页时建议先按段取，免得重复传另一段',
      },
      offset: { type: 'number', description: '本页起点（从 0 开始）；翻页时传上一页的 nextOffset' },
      limit: { type: 'number', description: `每页条数（默认 ${PAGE_DEFAULT}，最多 ${PAGE_MAX}）` },
    },
  },
  run: async args => {
    const provider = str(args.provider, 64).toLowerCase()
    const keyword = str(args.keyword, 128).toLowerCase()
    const enabledOnly = bool(args.enabledOnly) === true
    const section = str(args.section, 16).toLowerCase() || 'both'
    if (section !== 'models' && section !== 'mappings' && section !== 'both') {
      throw new Error(t('section 只能是 models / mappings / both'))
    }
    const offset = Math.max(0, Math.round(num(args.offset) ?? 0))
    const limit = Math.max(1, Math.min(PAGE_MAX, Math.round(num(args.limit) ?? PAGE_DEFAULT)))
    const data = await readModels()
    const payload: Record<string, unknown> = {}

    if (section !== 'mappings') {
      const rows = data.models.filter(row => {
        if (provider && str(row.provider).toLowerCase() !== provider) return false
        if (enabledOnly && row.enabled !== true) return false
        if (keyword) {
          const hay = [str(row.id), str(row.name), ...(Array.isArray(row.aliases) ? row.aliases.map(item => str(item)) : [])].join(' ').toLowerCase()
          if (!hay.includes(keyword)) return false
        }
        return true
      })
      const page = paginate(rows.map(row => ({
        provider: str(row.provider),
        providerLabel: str(row.providerLabel) || str(row.provider),
        id: str(row.id),
        enabled: row.enabled === true,
        kind: str(row.kind) || 'chat',
        supports_image_generation: row.supports_image_generation === true,
        supports_image_editing: row.supports_image_editing === true,
        aliases: Array.isArray(row.aliases) ? row.aliases.map(item => str(item)) : [],
      })), offset, limit)
      payload.models = { ...page.meta, items: page.items }
    }

    if (section !== 'models') {
      // 1 （ not in 「」）： up aliases 
      // up that ，「 in 」 this need is 。
      const rows = data.mappings.filter(row => {
        if (provider && str(row.provider).toLowerCase() !== provider) return false
        if (enabledOnly && row.enabled !== true) return false
        if (!keyword) return true
        return [str(row.alias), str(row.target)].join(' ').toLowerCase().includes(keyword)
      })
      const page = paginate(rows.map(row => ({
        provider: str(row.provider),
        alias: str(row.alias),
        target: str(row.target),
        enabled: row.enabled === true,
        isDefault: row.isDefault === true,
        dangling: row.dangling === true,
      })), offset, limit)
      payload.mappings = { ...page.meta, items: page.items }
    }

    const modelsMeta = payload.models as PageMeta | undefined
    const mappingsMeta = payload.mappings as PageMeta | undefined
    const summary = modelsMeta
      ? t('{n} 个模型', { n: modelsMeta.total })
      : t('{n} 条映射', { n: mappingsMeta?.total ?? 0 })
    return { text: JSON.stringify(payload), summary }
  },
}

const listAccountBalance: AssistantTool = createBalanceTool({
  accounts: async () => {
    const read = bridge().getAccounts
    if (!read) throw new Error(t('当前客户端缺少读取接口：{name}', { name: 'getAccounts' }))
    return read()
  },
  usageSnapshot: async () => {
    const read = bridge().getBalancesSnapshot
    if (!read) throw new Error(t('当前客户端缺少读取接口：{name}', { name: 'getBalancesSnapshot' }))
    return read()
  },
  refreshBalance: async id => {
    const query = bridge().getAllBalances
    if (!query) throw new Error(t('当前客户端缺少读取接口：{name}', { name: 'getAllBalances' }))
    return query(id)
  },
}, t)

/** 3 · enabled / use （） */
const setModelEnabled: AssistantTool = {
  id: 'set_model_enabled',
  label: t('启用 / 禁用模型'),
  kind: 'write',
  desc: '开启或关闭某几个模型在网关上的可用性（粒度是「提供商 × 模型」）。禁用后下游请求不再路由到它。一次可以改多个。内置家与自定义提供商（custom- 前缀）都支持。',
  params: {
    type: 'object',
    properties: {
      items: {
        type: 'array',
        description: '要改的模型列表',
        items: {
          type: 'object',
          properties: {
            provider: { type: 'string', description: '提供商 id' },
            id: { type: 'string', description: '模型 ID' },
            enabled: { type: 'boolean', description: 'true = 启用，false = 禁用' },
          },
          required: ['provider', 'id', 'enabled'],
        },
      },
    },
    required: ['items'],
  },
  plan: args => {
    const items = list(args.items)
    return {
      lead: t('将修改 {n} 个模型的启停状态：', { n: items.length }),
      table: {
        head: [t('提供商'), t('模型'), t('改成')],
        rows: items.map(item => [
          str(item.provider),
          str(item.id),
          bool(item.enabled) === false ? t('禁用') : t('启用'),
        ]),
      },
      note: t('改动立即生效；禁用后指向它的请求会落到下一个候选。'),
    }
  },
  run: async (args, context) => {
    const api = bridge()
    const result = await runToolBatch(list(args.items), context, async item => {
      const provider = str(item.provider)
      const id = str(item.id)
      const enabled = bool(item.enabled)
      if (!provider || !id || enabled === undefined) throw new Error(t('参数不完整（需要 provider / id / enabled）'))
      if (isCustomProvider(provider)) {
        await customBinding(provider, id, id, { enabled })
      } else {
        if (!api.setModelState) throw new Error(t('当前客户端缺少读取接口：{name}', { name: 'setModelState' }))
        await api.setModelState({ provider, id, enabled })
      }
      return `${enabled ? '启用' : '禁用'} ${provider}/${id}`
    }, t)
    refreshApp()
    return result
  },
}

/** 4 · set ID（）—— use ：
 * 「 and 」（ 1 new ） and 「 before 」（ before ），
 * is 1 alias parameter。 */
const setModelMapping: AssistantTool = {
  id: 'set_model_mapping',
  label: t('设置模型映射 ID'),
  kind: 'write',
  desc: '给模型新增或删除映射名（对外名）。同一对外名可以在多个提供商各挂一条：下游用同一个名字请求，网关按账号优先级主备切换；也可以给不同家的同名模型各加前缀（如 ca/claude-sonnet-4.5）来精确指定某一家。一次可以改多条。删掉的映射名只是不再对外可用，不删任何模型。内置家与自定义提供商（custom- 前缀）都支持。',
  params: {
    type: 'object',
    properties: {
      items: {
        type: 'array',
        description: '要改的映射列表',
        items: {
          type: 'object',
          properties: {
            provider: { type: 'string', description: '提供商 id' },
            id: { type: 'string', description: '上游模型 ID' },
            alias: { type: 'string', description: '要新增 / 删除的映射名（对外名）' },
            action: { type: 'string', enum: ['add', 'remove'], description: 'add = 新增映射，remove = 删除映射' },
          },
          required: ['provider', 'id', 'alias', 'action'],
        },
      },
    },
    required: ['items'],
  },
  plan: args => {
    const items = list(args.items)
    const adds = items.filter(item => str(item.action) !== 'remove').length
    const removes = items.length - adds
    const parts: string[] = []
    if (adds) parts.push(t('新增 {n} 条', { n: adds }))
    if (removes) parts.push(t('删除 {n} 条', { n: removes }))
    return {
      lead: t('将改动 {n} 条映射（{parts}）：', { n: items.length, parts: parts.join('、') }),
      table: {
        head: [t('提供商'), t('上游模型'), t('操作'), t('映射名')],
        rows: items.map(item => [
          str(item.provider),
          str(item.id),
          str(item.action) === 'remove' ? t('删除') : t('新增'),
          str(item.alias),
        ]),
      },
      note: t('只改映射名，不动上游模型 ID；映射立即生效（新对话即可用）。'),
    }
  },
  run: async (args, context) => {
    const api = bridge()
    const result = await runToolBatch(list(args.items), context, async item => {
      const provider = str(item.provider)
      const id = str(item.id)
      const alias = str(item.alias, 128)
      const action = str(item.action)
      if (!provider || !id || !alias || !['add', 'remove'].includes(action)) throw new Error(t('参数不完整（需要 provider / id / alias / action）'))
      if (isCustomProvider(provider)) {
        if (action === 'remove') await customWriter().removeMapping(provider, alias, id)
        else await customBinding(provider, alias, id, { enabled: true })
      } else if (action === 'remove') {
        if (!api.removeModelMapping) throw new Error(t('当前客户端缺少读取接口：{name}', { name: 'removeModelMapping' }))
        await api.removeModelMapping(alias, id, provider)
      } else {
        if (!api.addModelMapping) throw new Error(t('当前客户端缺少读取接口：{name}', { name: 'addModelMapping' }))
        await api.addModelMapping(alias, id, provider)
      }
      return `${action === 'remove' ? '删除' : '新增'} ${alias} → ${provider}/${id}`
    }, t)
    refreshApp()
    return result
  },
}

/** 5 · can （） */
const CAPABILITY_KEYS = ['maxInputTokens', 'maxOutputTokens', 'supportsToolCall', 'supportsImages', 'supportsVideo', 'supportsReasoning'] as const
const CAPABILITY_LABELS: Record<string, string> = {
  maxInputTokens: t('上下文窗口'),
  maxOutputTokens: t('最大输出 Token'),
  supportsToolCall: t('工具调用'),
  supportsImages: t('图片识别'),
  supportsVideo: t('视频识别'),
  supportsReasoning: t('深度推理'),
}

const setModelCapability: AssistantTool = {
  id: 'set_model_capability',
  label: t('修改模型能力'),
  kind: 'write',
  desc: '修改某个模型对下游声明的能力位：上下文窗口、最大输出 Token、工具调用 / 图片 / 视频 / 深度推理四个布尔位。只传要改的键；布尔位给 null 表示恢复上游原值（自定义提供商没有「上游原值」，给 null 等于清除这一项）。内置家与自定义提供商（custom- 前缀）都支持。',
  params: {
    type: 'object',
    properties: {
      provider: { type: 'string', description: '提供商 id' },
      id: { type: 'string', description: '模型 ID' },
      capabilities: {
        type: 'object',
        description: '要改的能力位（只写要改的键）',
        properties: {
          maxInputTokens: { type: ['number', 'null'], description: '上下文窗口（token 数）' },
          maxOutputTokens: { type: ['number', 'null'], description: '最大输出 token 数' },
          supportsToolCall: { type: ['boolean', 'null'], description: '是否支持工具调用；null = 恢复上游原值' },
          supportsImages: { type: ['boolean', 'null'], description: '是否支持图片识别；null = 恢复上游原值' },
          supportsVideo: { type: ['boolean', 'null'], description: '是否支持视频识别；null = 恢复上游原值' },
          supportsReasoning: { type: ['boolean', 'null'], description: '是否支持深度推理；null = 恢复上游原值' },
        },
      },
    },
    required: ['provider', 'id', 'capabilities'],
  },
  plan: args => {
    const caps = (args.capabilities && typeof args.capabilities === 'object' ? args.capabilities : {}) as Record<string, unknown>
    const keys = CAPABILITY_KEYS.filter(key => key in caps)
    return {
      lead: t('将修改 {model} 的 {n} 项能力：', { model: `${str(args.provider)} / ${str(args.id)}`, n: keys.length }),
      table: {
        head: [t('能力'), t('改成')],
        rows: keys.map(key => {
          const value = caps[key]
          const text = value === null ? t('恢复上游原值') : typeof value === 'boolean' ? (value ? t('支持') : t('不支持')) : String(value)
          return [CAPABILITY_LABELS[key] || key, text]
        }),
      },
      note: t('能力位决定下游看到的声明（能不能贴图、能塞多少上下文），不改上游的真实行为。'),
    }
  },
  run: async (args, context) => {
    checkToolSignal(context)
    const provider = str(args.provider)
    const id = str(args.id)
    const raw = (args.capabilities && typeof args.capabilities === 'object' ? args.capabilities : {}) as Record<string, unknown>
    const caps: Record<string, unknown> = {}
    for (const key of CAPABILITY_KEYS) {
      if (!(key in raw)) continue
      const value = raw[key]
      if (value === null) { caps[key] = null; continue }
      if (key === 'maxInputTokens' || key === 'maxOutputTokens') {
        const n = num(value)
        if (n === undefined) throw new Error(t('{key} 需要数字', { key: CAPABILITY_LABELS[key] }))
        caps[key] = Math.round(n)
      } else {
        const b = bool(value)
        if (b === undefined) throw new Error(t('{key} 需要 true / false / null', { key: CAPABILITY_LABELS[key] }))
        caps[key] = b
      }
    }
    if (!provider || !id || !Object.keys(caps).length) throw new Error(t('参数不完整（需要 provider / id / capabilities）'))
    if (isCustomProvider(provider)) {
      // can in models[] ， is 
      // （ after `POST /api/models/capabilities` for custom- id 400，
      // `api::model_manage::set_capabilities` ）。
      await customWriter().setCapabilities(provider, id, caps as Record<string, number | boolean | null>)
    } else {
      const write = bridge().setModelCapabilities
      if (!write) throw new Error(t('当前客户端缺少读取接口：{name}', { name: 'setModelCapabilities' }))
      await write(provider, id, caps)
    }
    refreshApp()
    return { text: JSON.stringify({ ok: true, provider, id, capabilities: caps }), summary: t('已修改 {n} 项能力', { n: Object.keys(caps).length }) }
  },
}

/** 6 · （） */
const testModel: AssistantTool = {
  id: 'test_model',
  label: t('发起模型测试'),
  kind: 'run',
  desc: '对某个模型发一次真实的对话请求，回报连通性、耗时与首字延迟。会消耗该账号的额度。回答「这个模型通不通 / 快不快」时用它。内置家与自定义提供商（custom- 前缀）都支持，模型名可以是模型 ID 或已登记的映射别名。',
  params: {
    type: 'object',
    properties: {
      provider: { type: 'string', description: '提供商 id' },
      id: { type: 'string', description: '模型 ID' },
      prompt: { type: 'string', description: '测试用的用户提示词；不填 = 默认一句问候（最小请求）' },
    },
    required: ['provider', 'id'],
  },
  plan: args => ({
    lead: t('将对 {model} 发起一次真实请求：', { model: `${str(args.provider)} / ${str(args.id)}` }),
    table: {
      head: [t('项'), t('值')],
      rows: [
        [t('提示词'), str(args.prompt) || t('（默认问候）')],
        [t('代价'), t('消耗该账号额度')],
      ],
    },
    note: t('这是真实的上游请求，与生产走同一条链路（会记账，报表排除）。'),
  }),
  run: async (args, context) => {
    checkToolSignal(context)
    const api = bridge()
    if (!api.testModel) throw new Error(t('当前客户端缺少读取接口：{name}', { name: 'testModel' }))
    if (!context?.apiKey) throw new Error(t('请先选择一把 API Key'))
    const testId = crypto.randomUUID()
    const payload: Record<string, unknown> = { provider: str(args.provider), model: str(args.id), test_id: testId }
    const prompt = str(args.prompt, 4000)
    if (prompt) payload.prompt = prompt
    const cancel = () => { void api.terminateStatsRequest?.(testId).catch(() => {}) }
    context?.signal?.addEventListener('abort', cancel, { once: true })
    let result: Record<string, unknown> | undefined
    try {
      result = await api.testModel(payload, context.apiKey) as Record<string, unknown> | undefined
    } finally {
      context?.signal?.removeEventListener('abort', cancel)
    }
    const status = String(result?.status ?? '')
    const ok = result?.success === true
    const slim = {
      ok,
      status,
      error: result?.error ? safeText(result.error, 3000) : null,
      reply: safeText(result?.reply, 1500),
      duration_ms: result?.duration_ms ?? null,
      ttfb_ms: result?.ttfb_ms ?? null,
      account: result?.account_name ?? null,
    }
    return {
      text: JSON.stringify(slim),
      summary: ok ? t('连通 · {ms}ms', { ms: String(slim.duration_ms ?? '') }) : t('失败 · {status}', { status: status || '—' }),
    }
  },
}

/** and file： in `ui-shared`，request and new 。 */
const readAttachment = createAttachmentReadTool(t)
const importAccount = createAccountImportTool({
  post: (path, body, signal) => callJson('POST', path, body, signal),
  refresh: refreshApp,
}, t)
const ACCOUNT_IMPORT_META = accountImportMeta(t)

/** （ = set： in before ， in after ， after ） */
export const ASSISTANT_TOOLS: AssistantTool[] = [
  ...ASSISTANT_READ_TOOLS,
  readAttachment,
  listModels,
  listAccountBalance,
  setModelEnabled,
  setModelMapping,
  setModelCapability,
  testModel,
  ...ASSISTANT_IMAGE_TOOLS,
  ...ASSISTANT_ACCOUNT_TOOLS,
  importAccount,
]

export function toolById(id: string): AssistantTool | undefined {
  return ASSISTANT_TOOLS.find(tool => tool.id === id)
}

/** by OpenAI tools （ = ASSISTANT_TOOLS ） */
export function toolsPayload(enabled: Record<string, boolean>): Array<Record<string, unknown>> {
  return ASSISTANT_TOOLS.filter(tool => enabled[tool.id]).map(tool => ({
    type: 'function',
    function: { name: tool.id, description: tool.desc, parameters: tool.params },
  }))
}

/** set and （ id → in / 1 / ）。
 * in this use `t()` （ when ）： `meta.label` can ，
 * not will 「 to t()」 that not to 。 */
export const TOOL_META: Array<{ id: string; label: string; desc: string; tag?: 'danger' | 'cost' }> = [
  ...ASSISTANT_READ_META,
  ...ASSISTANT_IMAGE_META,
  { id: 'list_models', label: t('获取模型列表'), desc: t('读各提供商的模型清单、映射名与启用状态。') },
  { id: 'list_account_balance', label: t('查询账号余额'), desc: t('读各账号的额度 / 剩余用量。') },
  { id: 'set_model_enabled', label: t('启用 / 禁用模型'), desc: t('开关模型的对外可用性，不改其它属性。') },
  { id: 'set_model_mapping', label: t('设置模型映射 ID'), desc: t('新增 / 删除映射名，支持批量加前缀与同名合并到多家。') },
  { id: 'set_model_capability', label: t('修改模型能力'), desc: t('改上下文窗口 / 最大输出 Token / 工具调用等能力位。') },
  { id: 'test_model', label: t('发起模型测试'), desc: t('对指定模型发一次真实请求，回报连通性与耗时。'), tag: 'cost' },
  { id: 'set_provider_order', label: t('调整提供商顺位'), desc: t('改某一家的整组顺位（先试哪一家）。') },
  { id: 'set_account_order', label: t('调整账号优先级'), desc: t('改账号在全局队列里的名次，决定主备与切换顺序。') },
  { id: 'set_account_enabled', label: t('启用 / 禁用账号'), desc: t('禁用会把账号移出转发队列，线路可能因此断供。'), tag: 'danger' },
  ...ACCOUNT_IMPORT_META,
]

export { errorText }

/** current for ， not use directoryavailable。 */
export async function loadModelNames(): Promise<string[]> {
  return (await loadModelOptions()).map(option => option.name)
}

export async function loadModelOptions(): Promise<ModelOption[]> {
  const readState = bridge().getState
  if (!readState) throw new Error(t('读取模型清单失败'))
  const state = await readState() as { models?: unknown } | null | undefined
  if (!Array.isArray(state?.models)) throw new Error(t('读取模型清单失败'))
  return modelOptionsFrom(state.models)
}
