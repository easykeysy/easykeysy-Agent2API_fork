/**
 * AI 助手的工具集：定义（给模型看的 JSON Schema）+ 执行器（调既有的管理接口）。
 *
 * ── 为什么工具全在前端 ───────────────────────────────────────
 * 每只工具做的都是界面本来就会做的事（模型启停、映射、能力位、顺序、余额、
 * 模型测试），而这些动作在**桥**上已有现成入口（`window.workbuddyDesktop`
 * 的同名方法，桌面壳与 headless 面板各有一份实现）。工具放在这里 = 加一只工具
 * 不用加 Rust 业务接口；执行走既有桥 = 与界面按钮走同一条链。
 * 后端校验工具 ID 与权限后转给上游、把 `tool_calls` 原样流回来（见
 * `api::assistant` 模块头）。
 *
 * ── 三类工具（决定确认流程）──────────────────────────────────
 *   read  只读：读完直接把结果回给模型，不打扰用户；
 *   write 改动：**先渲染确认卡**，用户点「执行」才落库；
 *   run   会真打上游 / 消耗额度（模型测试）：同 write，且卡上说清代价。
 * 「要不要确认」不看工具自己，看用户点的是哪张卡 —— 这里的 `kind` 是唯一的
 * 判据，别在页面里另写一套。
 *
 * ── 工具 id 是与设置页的契约 ─────────────────────────────────
 * `tools` 勾选表存的是这些 id（后端默认集见 `config::DEFAULT_ENABLED_TOOLS`，
 * 同一份字符串）。改名 = 用户存过的勾选失效，加工具 = 两边各加一处。
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
import {
  createModelListTool, type AssistantModelRow as ModelRow, type AssistantMappingRow as MappingRow,
  type AssistantModelView, type ModelProviderRow,
} from '../../../ui-shared/assistant-read-models'
import { CAPABILITY_KEYS } from '../../../ui-shared/model-capability'

export type { ModelOption } from './assistant-models'

/** 工具结果：`text` 回给模型（走 role:"tool" 的消息体），`summary` 给卡片显示 */
export type ToolResult = {
  text: string
  summary: string
  images?: ImageAsset[]
}

/** 确认卡上的计划（改动类工具在用户确认前渲染它） */
export type ToolPlan = {
  /** 一句话说清将要做什么（含数量） */
  lead: string
  /** 明细表（可选；列数任意，行数由调用方截断显示） */
  table?: { head: string[]; rows: string[][] }
  /** 底部的补充说明（风险 / 生效时机 / 失败语义） */
  note?: string
}

export type AssistantTool = {
  id: string
  label: string
  kind: 'read' | 'write' | 'run'
  /** 给模型看的说明（中文 —— 上游模型对中文的理解足够，且与界面文案同源） */
  desc: string
  params: Record<string, unknown>
  /** 改动类 / 消耗类：渲染确认卡（只读工具不实现） */
  plan?: (args: Record<string, unknown>) => ToolPlan
  /** 参数在工具卡里展开显示时的遮蔽形态（凭证原文不长期显示） */
  redactArgs?: (args: Record<string, unknown>) => Record<string, unknown>
  run: (args: Record<string, unknown>, context?: ImageToolContext) => Promise<ToolResult>
}

/* ─── 桥（只声明用到的那些方法，缺失时按「这条链在当前形态不可用」处理）── */

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
  if (!api) throw new Error(t('Desktop bridge unavailable (this page must be opened inside the Agent2API client)'))
  return api
}

/** 改动类工具落库后刷新全局状态（其它页面的数据依赖它） */
function refreshApp(): void {
  const app = (window as unknown as { wbApp?: { refresh?: () => void } }).wbApp
  void app?.refresh?.()
}

/* ─── 入参读取小工具（模型给的东西不一定守规矩，坏值一律当缺）── */

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

/* ─── 数据形状（只声明用到的字段）── */

type ManageViewLike = Partial<AssistantModelView>

/** 自定义提供商目录里的一条记录（`providers.js` 的 `customList`） */
type CustomProviderRecord = {
  id?: unknown
  name?: unknown
  models?: unknown
  mappings?: unknown
}

/** 目录模块（`ui/providers.js`）在本模块用到的部分 */
type ProvidersBridge = {
  customList?: () => CustomProviderRecord[]
  refreshCustom?: () => Promise<unknown>
}

/**
 * 自定义提供商的判定与「记录 → 管理视图」的适配都复用模型管理页那一份。
 *
 * ── 为什么要复用而不是在助手这边另写一份 ────────────────────
 * 那层适配（默认绑定现造、同名映射并入、能力位稀疏表、legacy 条目剔除）是
 * 模型管理页的**渲染契约**，助手读到的形状必须与它逐字一致，否则同一个模型
 * 在「管理页看到的启用状态」与「助手说它是启用的」之间会分叉。两份实现
 * 只会让这种分叉迟早发生。
 */
type CustomSourceModule = {
  isCustom: (id: unknown) => boolean
  buildView: (id: string) => { models?: ModelRow[]; mappings?: MappingRow[] } | null
  /** 改一条绑定的开关 / 思考等级（`alias == target` 即默认绑定） */
  setBinding: (
    id: string,
    alias: string,
    target: string,
    patch?: { enabled?: boolean; reasoning?: string },
  ) => Promise<void>
  /** 删除一条映射（默认绑定不可删 —— 与内置家同口径） */
  removeMapping: (id: string, alias: string, target: string) => Promise<void>
  /** 覆盖某条模型的能力位；`null` = 清除该项 */
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
 * 取自定义家的写入模块；缺了就明确报错。
 *
 * 不静默跳过：`window.wbModelsCustom` 由模型管理页那个岛在模块顶层挂上（同一个
 * bundle、加载即注册），缺它只可能是页面形态变了 —— 那时「说改动成功、其实
 * 什么都没写」比一条明确的错误糟得多。
 */
function customWriter(): CustomSourceModule {
  const source = customSource()
  if (!source) throw new Error(t('Custom provider module is not ready (refresh the UI and try again)'))
  return source
}

/** 改自定义家一条绑定的开关 / 思考等级 */
async function customBinding(
  provider: string,
  alias: string,
  target: string,
  patch: { enabled?: boolean; reasoning?: string },
): Promise<void> {
  await customWriter().setBinding(provider, alias, target, patch)
}

/** 目录缓存（`providers.js` 在页面 head 之后加载，所以只能运行期取） */
function customRecords(): CustomProviderRecord[] {
  const providers = (window as unknown as { wbProviders?: ProvidersBridge }).wbProviders
  const list = providers?.customList?.()
  return Array.isArray(list) ? list : []
}

/**
 * 读目录缓存；**空表时先补拉一次**。
 *
 * `providers.js` 在加载期就拉过一份，所以正常情况下缓存总是有值。但面板形态
 * 首屏那次请求可能还没回来（用户刚打开页面就问助手），此时空表会被读成
 * 「一家自定义提供商都没有」—— 那是一次静默的错误答案。补拉一次再读，
 * 失败就按空表处理（目录偶发打不通不该让整条工具失败）。
 */
async function customRecordsFresh(): Promise<CustomProviderRecord[]> {
  const cached = customRecords()
  if (cached.length) return cached
  const providers = (window as unknown as { wbProviders?: ProvidersBridge }).wbProviders
  try {
    await providers?.refreshCustom?.()
  } catch { /* 目录偶发打不通：按现有缓存（空）继续 */ }
  return customRecords()
}

/**
 * 自定义家是否在目录里（判据与后端 `is_custom_provider_id` 同向前缀）。
 *
 * 这里**不看** `isCustom`（它只判前缀）：目录里没有的 `custom-xxx` 是残留
 * 账号指向的家，管理页那边同样只认目录里存在的那些。
 */
function isCustomProvider(id: string): boolean {
  const source = customSource()
  if (source) return source.isCustom(id)
  return id.startsWith('custom-')
}

/**
 * 一家自定义提供商的管理视图；目录里没有这家时 `buildView` 自己回 null。
 *
 * 注意 `buildView` 读的是**同步**目录缓存 —— 调用方必须先 await 过一次
 * `customRecordsFresh()`（`readModels` 就是这么做的），否则首屏那一刻会拿到
 * 一份还没到的空目录。
 */
function customView(id: string): ManageViewLike | null {
  return customSource()?.buildView(id) ?? null
}

/**
 * 全部模型行 + 映射（内置家 + 自定义家）。
 *
 * ── 为什么必须合并 ──────────────────────────────────────────
 * `GET /api/models/manage` 只遍历内置注册表（`catalog::manage_view` 的
 * `active_manifests`），自定义家一个都不在里面 —— 它们的清单存在提供商记录里、
 * 由模型管理页在前端适配。助手若只读那一条接口，回答「有哪些模型」时会漏掉
 * 用户自建的全部上游，且 `set_model_*` 系列拿不到自定义家的启用状态。
 */
async function readModels(): Promise<AssistantModelView> {
  const read = bridge().getModelManage
  if (!read) throw new Error(t('This client is missing the read interface: {name}', { name: 'getModelManage' }))
  const data = await read() as ManageViewLike | undefined
  if (!Array.isArray(data?.models) || !Array.isArray(data?.mappings)) throw new Error(t('Failed to load the model list'))
  const models: ModelRow[] = data.models.slice()
  const mappings: MappingRow[] = data.mappings.slice()
  const providers: ModelProviderRow[] = Array.isArray(data.providers) ? data.providers.slice() : []
  // 自定义家逐个适配后追加。目录缓存缺失（面板形态未注入 / 还没拉到）时
  // 只是少这一块 —— 内置家那部分照常可用，不因此整条工具失败。
  const known = new Set(models.map(row => `${str(row.provider)}\u0001${str(row.id)}`.toLowerCase()))
  for (const record of await customRecordsFresh()) {
    const id = str(record.id)
    if (!id) continue
    const view = customView(id)
    if (!view) continue
    providers.push({ provider: id, providerLabel: str(record.name) || id })
    for (const row of view.models || []) {
      const key = `${id}\u0001${str(row.id)}`.toLowerCase()
      if (known.has(key)) continue
      known.add(key)
      models.push(row)
    }
    for (const mapping of view.mappings || []) mappings.push(mapping)
  }
  return { models, mappings, providers }
}

/* ─── 模型工具与余额查询 ───────────────────────────────────── */

const listModels: AssistantTool = createModelListTool(readModels, t)

const listAccountBalance: AssistantTool = createBalanceTool({
  accounts: async () => {
    const read = bridge().getAccounts
    if (!read) throw new Error(t('This client is missing the read interface: {name}', { name: 'getAccounts' }))
    return read()
  },
  usageSnapshot: async () => {
    const read = bridge().getBalancesSnapshot
    if (!read) throw new Error(t('This client is missing the read interface: {name}', { name: 'getBalancesSnapshot' }))
    return read()
  },
  refreshBalance: async id => {
    const query = bridge().getAllBalances
    if (!query) throw new Error(t('This client is missing the read interface: {name}', { name: 'getAllBalances' }))
    return query(id)
  },
}, t)

/** 3 · 启用 / 禁用模型（改动） */
const setModelEnabled: AssistantTool = {
  id: 'set_model_enabled',
  label: t('Enable / disable models'),
  kind: 'write',
  desc: 'Turn on or off the availability of one or more models on the gateway (at the granularity of "provider× model"). After disabling, downstream requests are no longer routed to it. Several can be changed at once. Both built-in and custom providers (with the custom- prefix) are supported.',
  params: {
    type: 'object',
    properties: {
      items: {
        type: 'array',
        description: 'The model list to change',
        items: {
          type: 'object',
          properties: {
            provider: { type: 'string', description: 'Provider id' },
            id: { type: 'string', description: 'Model ID' },
            enabled: { type: 'boolean', description: 'true = Enable, false= Disable' },
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
      lead: t('About to change the enabled state of {n} models:', { n: items.length }),
      table: {
        head: [t('Provider'), t('Model'), t('Change to')],
        rows: items.map(item => [
          str(item.provider),
          str(item.id),
          bool(item.enabled) === false ? t('Disable') : t('enabled'),
        ]),
      },
      note: t('Changes take effect immediately; once disabled, requests for it fall through to the next candidate.'),
    }
  },
  run: async (args, context) => {
    const api = bridge()
    const result = await runToolBatch(list(args.items), context, async item => {
      const provider = str(item.provider)
      const id = str(item.id)
      const enabled = bool(item.enabled)
      if (!provider || !id || enabled === undefined) throw new Error(t('Incomplete parameters (provider / id / enabled required)'))
      if (isCustomProvider(provider)) {
        await customBinding(provider, id, id, { enabled })
      } else {
        if (!api.setModelState) throw new Error(t('This client is missing the read interface: {name}', { name: 'setModelState' }))
        await api.setModelState({ provider, id, enabled })
      }
      return `${enabled ? 'enabled' : 'Disable'} ${provider}/${id}`
    }, t)
    refreshApp()
    return result
  },
}

/** 4 · 设置模型映射 ID（改动）—— 两类用法的公共入口：
 *   「同名合并」（多家各挂同一个新名字）与「前缀区分」（各家挂各自前缀的名字），
 *   都只是同一批 alias 的两组参数。 */
const setModelMapping: AssistantTool = {
  id: 'set_model_mapping',
  label: t('Set model mapping ID'),
  kind: 'write',
  desc: 'Add or remove mapping names (external names) for a model. The same external name may be registered on multiple providers: downstream clients request with the same name, and the gateway failovers between providers by account priority; you can also prefix same-named models from different providers (e.g. ca/claude-sonnet-4.5) to target one specific provider. Multiple mappings can be changed in one call. Removing a mapping name only makes it unavailable to external clients; no model is deleted. Both the built-in providers and custom providers (custom- prefix) are supported.",',
  params: {
    type: 'object',
    properties: {
      items: {
        type: 'array',
        description: 'The mapping list to change',
        items: {
          type: 'object',
          properties: {
            provider: { type: 'string', description: 'Provider id' },
            id: { type: 'string', description: 'Upstream model ID' },
            alias: { type: 'string', description: 'The mapping names (outward-facing names) to add / remove' },
            action: { type: 'string', enum: ['add', 'remove'], description: 'add = add mappings, remove= Delete mapping' },
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
    if (adds) parts.push(t('Add {n} entries', { n: adds }))
    if (removes) parts.push(t('Delete {n} entries', { n: removes }))
    return {
      lead: t('About to change {n} mappings ({parts}):', { n: items.length, parts: parts.join(', ') }),
      table: {
        head: [t('Provider'), t('Upstream model'), t('Actions'), t('Mapping name')],
        rows: items.map(item => [
          str(item.provider),
          str(item.id),
          str(item.action) === 'remove' ? t('Delete') : t('Added'),
          str(item.alias),
        ]),
      },
      note: t('Only the mapping name changes; the upstream model ID stays. Mappings take effect immediately (usable in a new chat).'),
    }
  },
  run: async (args, context) => {
    const api = bridge()
    const result = await runToolBatch(list(args.items), context, async item => {
      const provider = str(item.provider)
      const id = str(item.id)
      const alias = str(item.alias, 128)
      const action = str(item.action)
      if (!provider || !id || !alias || !['add', 'remove'].includes(action)) throw new Error(t('Incomplete parameters (provider / id / alias / action required)'))
      if (isCustomProvider(provider)) {
        if (action === 'remove') await customWriter().removeMapping(provider, alias, id)
        else await customBinding(provider, alias, id, { enabled: true })
      } else if (action === 'remove') {
        if (!api.removeModelMapping) throw new Error(t('This client is missing the read interface: {name}', { name: 'removeModelMapping' }))
        await api.removeModelMapping(alias, id, provider)
      } else {
        if (!api.addModelMapping) throw new Error(t('This client is missing the read interface: {name}', { name: 'addModelMapping' }))
        await api.addModelMapping(alias, id, provider)
      }
      return `${action === 'remove' ? 'Delete' : 'Added'} ${alias} → ${provider}/${id}`
    }, t)
    refreshApp()
    return result
  },
}

/** 5 · 修改模型能力（改动） */
const CAPABILITY_LABELS: Record<string, string> = {
  maxInputTokens: t('Context window'),
  maxOutputTokens: t('Max output tokens'),
  supportsToolCall: t('Tool calling'),
  supportsImages: t('Image recognition'),
  supportsVideo: t('Video recognition'),
  supportsReasoning: t('Deep reasoning'),
}

const setModelCapability: AssistantTool = {
  id: 'set_model_capability',
  label: t('Edit model capabilities'),
  kind: 'write',
  desc: 'Change the capability flags a model declares downstream: context window, max output tokens, and four booleans — tool calling / Images / Videos / deep reasoning. Only pass the keys you want to change; null restores the upstream original value for a boolean (custom providers have no "upstream original" — null on one just clears that item). Both the built-in providers and custom providers (custom- prefix) are supported.",',
  params: {
    type: 'object',
    properties: {
      provider: { type: 'string', description: 'Provider id' },
      id: { type: 'string', description: 'Model ID' },
      capabilities: {
        type: 'object',
        description: 'The capability bits to change (write only the keys to change)',
        properties: {
          maxInputTokens: { type: ['number', 'null'], description: 'Context window (token count)' },
          maxOutputTokens: { type: ['number', 'null'], description: 'Maximum output token count' },
          supportsToolCall: { type: ['boolean', 'null'], description: 'Whether tool calling is supported; null= Restore upstream values' },
          supportsImages: { type: ['boolean', 'null'], description: 'Whether image recognition is supported; null= Restore upstream values' },
          supportsVideo: { type: ['boolean', 'null'], description: 'Whether video recognition is supported; null= Restore upstream values' },
          supportsReasoning: { type: ['boolean', 'null'], description: 'Whether deep reasoning is supported; null= Restore upstream values' },
        },
      },
    },
    required: ['provider', 'id', 'capabilities'],
  },
  plan: args => {
    const caps = (args.capabilities && typeof args.capabilities === 'object' ? args.capabilities : {}) as Record<string, unknown>
    const keys = CAPABILITY_KEYS.filter(key => key in caps)
    return {
      lead: t('About to change {n} capabilities of {model}:', { model: `${str(args.provider)} / ${str(args.id)}`, n: keys.length }),
      table: {
        head: [t('Capabilities'), t('Change to')],
        rows: keys.map(key => {
          const value = caps[key]
          const text = value === null ? t('Restore upstream values') : typeof value === 'boolean' ? (value ? t('Supported') : t('Not supported')) : String(value)
          return [CAPABILITY_LABELS[key] || key, text]
        }),
      },
      note: t('Capability flags decide what downstream sees declared (image input, context size) without changing upstream behavior.'),
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
        if (n === undefined) throw new Error(t('{key} must be a number', { key: CAPABILITY_LABELS[key] }))
        caps[key] = Math.round(n)
      } else {
        const b = bool(value)
        if (b === undefined) throw new Error(t('{key} must be true / false / null', { key: CAPABILITY_LABELS[key] }))
        caps[key] = b
      }
    }
    if (!provider || !id || !Object.keys(caps).length) throw new Error(t('Incomplete parameters (provider / id / capabilities required)'))
    if (isCustomProvider(provider)) {
      // 自定义家的能力位存在提供商记录的 models[] 条目里，写入口是整表保存
      // （后端 `POST /api/models/capabilities` 对 custom- id 明确 400，见
      // `api::model_manage::set_capabilities` 的说明）。
      await customWriter().setCapabilities(provider, id, caps as Record<string, number | boolean | null>)
    } else {
      const write = bridge().setModelCapabilities
      if (!write) throw new Error(t('This client is missing the read interface: {name}', { name: 'setModelCapabilities' }))
      await write(provider, id, caps)
    }
    refreshApp()
    return { text: JSON.stringify({ ok: true, provider, id, capabilities: caps }), summary: t('Changed {n} capabilities', { n: Object.keys(caps).length }) }
  },
}

/** 6 · 发起模型测试（消耗额度） */
const testModel: AssistantTool = {
  id: 'test_model',
  label: t('Run a model test'),
  kind: 'run',
  desc: 'Send one real chat request to a given model and report connectivity, duration, and time to first token. This consumes the account\'s quota. Use it to answer "does this model work / is it fast". Both the built-in providers and custom providers (custom- prefix) are supported; the model name may be a model ID or a registered mapping alias.",',
  params: {
    type: 'object',
    properties: {
      provider: { type: 'string', description: 'Provider id' },
      id: { type: 'string', description: 'Model ID' },
      prompt: { type: 'string', description: 'The user prompt used for testing; leave blank= one greeting by default (minimal request)' },
    },
    required: ['provider', 'id'],
  },
  plan: args => ({
    lead: t('About to send one real request to {model}:', { model: `${str(args.provider)} / ${str(args.id)}` }),
    table: {
      head: [t('items'), t('Value')],
      rows: [
        [t('Prompt'), str(args.prompt) || t('(default greeting)')],
        [t('Cost'), t('Spends this account\'s quota')],
      ],
    },
    note: t('This is a real upstream request over the same path as production (it is billed but excluded from reports).'),
  }),
  run: async (args, context) => {
    checkToolSignal(context)
    const api = bridge()
    if (!api.testModel) throw new Error(t('This client is missing the read interface: {name}', { name: 'testModel' }))
    if (!context?.apiKey) throw new Error(t('Select an API Key first'))
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
      summary: ok ? t('Reachable · {ms}ms', { ms: String(slim.duration_ms ?? '') }) : t('Failed · {status}', { status: status || '—' }),
    }
  },
}

/** 附件读取与账号文件导入：定义在 `ui-shared`，桌面端只注入管理请求与刷新。 */
const readAttachment = createAttachmentReadTool(t)
const importAccount = createAccountImportTool({
  post: (path, body, signal) => callJson('POST', path, body, signal),
  refresh: refreshApp,
}, t)
const ACCOUNT_IMPORT_META = accountImportMeta(t)

/** 全部工具（顺序 = 设置页清单顺序：只读在前，改动在后，高危最后） */
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

/** 按勾选表组装 OpenAI tools 数组（顺序固定 = ASSISTANT_TOOLS 的顺序） */
export function toolsPayload(enabled: Record<string, boolean>): Array<Record<string, unknown>> {
  return ASSISTANT_TOOLS.filter(tool => enabled[tool.id]).map(tool => ({
    type: 'function',
    function: { name: tool.id, description: tool.desc, parameters: tool.params },
  }))
}

/** 设置页清单与文案（工具 id → 中文名 / 一句话 / 风险标签）。
 *  文案在这里就用 `t()` 包好（定义时求值）：页面直接取 `meta.label` 即可，
 *  不会再出现「把变量喂给 t()」那种扫描器提不到键的写法。 */
export const TOOL_META: Array<{ id: string; label: string; desc: string; tag?: 'danger' | 'cost' }> = [
  ...ASSISTANT_READ_META,
  ...ASSISTANT_IMAGE_META,
  { id: 'list_models', label: t('List models'), desc: t('Query models, capabilities, sources, and mappings by provider name, including custom providers.') },
  { id: 'list_account_balance', label: t('Query account balance'), desc: t('Reads each account\'s quota and remaining usage.') },
  { id: 'set_model_enabled', label: t('Enable / disable models'), desc: t('Toggles a model\'s outward availability without touching other attributes.') },
  { id: 'set_model_mapping', label: t('Set model mapping ID'), desc: t('Adds/removes mapping names, with bulk prefixing and merging a shared name across providers.') },
  { id: 'set_model_capability', label: t('Edit model capabilities'), desc: t('Edits capability flags such as context window, max output tokens, and tool calls.') },
  { id: 'test_model', label: t('Run a model test'), desc: t('Sends one real request to the given model and reports reachability and latency.'), tag: 'cost' },
  { id: 'set_provider_order', label: t('Reorder providers'), desc: t('Reorders a provider\'s whole group (which provider is tried first).') },
  { id: 'set_account_order', label: t('Adjust account priority'), desc: t('Changes an account\'s rank in the global queue, deciding primary/backup and failover order.') },
  { id: 'set_account_enabled', label: t('Enable / disable accounts'), desc: t('Disabling removes the account from the forwarding queue; the route may run dry as a result.'), tag: 'danger' },
  ...ACCOUNT_IMPORT_META,
]

export { errorText }

/** 助手选择器只列出网关当前对外提供的名字，不使用管理目录推算可用性。 */
export async function loadModelNames(): Promise<string[]> {
  return (await loadModelOptions()).map(option => option.name)
}

export async function loadModelOptions(): Promise<ModelOption[]> {
  const readState = bridge().getState
  if (!readState) throw new Error(t('Failed to load the model list'))
  const state = await readState() as { models?: unknown } | null | undefined
  if (!Array.isArray(state?.models)) throw new Error(t('Failed to load the model list'))
  return modelOptionsFrom(state.models)
}
