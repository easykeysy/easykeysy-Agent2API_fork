/**
 * AI 助手的工具集（手机端）。
 *
 * ── 为什么手机端也要有工具表 ──────────────────────────────────
 * 后端的 `/api/assistant/chat` 有一条产品口径（见 `config::AssistantConfig::
 * system_prompt`）：**一只工具都没勾 → 不注入系统提示词**。而系统提示词正是
 * 告诉模型「你是 Agent2API 的内置助手，管的是本机模型转发配置」的那段话。
 * 手机端早先只发 messages、不带 tools，于是模型拿到的是一个没有身份设定的
 * 通用对话 —— 用户问「各账号余额」，它答成了银行账户余额。
 *
 * 原有九只操作/查询工具、共享的六只诊断工具、图片工具，以及附件读取 / 账号
 * 文件导入工具都走**网关的管理 API**（手机端本来就在用那些接口），不经过桌面
 * 壳的桥。
 *
 * ── 与桌面端的差异 ────────────────────────────────────────────
 *   · 执行层从 `window.workbuddyDesktop` 换成 `api/gateway` 的 HTTP 调用；
 *   · 自定义提供商（`custom-` 前缀）的读写走 `/api/custom-providers`；
 *   · 写操作与额度消耗遵守同一份 confirm / full 配置，手机端用确认弹窗展示参数；
 *     未勾选的工具不进入请求，执行入口仍须再次核对权限。
 */

import {
  fetchAccounts,
  fetchAccountUsage,
  fetchManage,
  fetchUsageSnapshot,
  setAccountEnabled,
  setModelEnabled,
  type Account,
} from '../api/gateway'
import { apiCall } from '../api/client'
import { accountName } from '../lib/format'
import { createBalanceTool } from '../../../ui-shared/assistant-read-balance'
import { t } from '../i18n'
import { ASSISTANT_READ_TOOLS } from './assistant-read-tools'
import { ASSISTANT_IMAGE_TOOLS } from './assistant-image-tools'
import {
  createAccountImportTool,
  createAttachmentReadTool,
} from '../../../ui-shared/assistant-account-import-tools'
import type { ImageAsset, ImageToolContext } from '../../../ui-shared/assistant-image-tools'
import { checkToolSignal, providerGroups, runToolBatch } from '../../../ui-shared/assistant-tool-execution'
import { safeText } from '../../../ui-shared/assistant-read-data'
import { createModelListTool, type AssistantModelView } from '../../../ui-shared/assistant-read-models'
import { CAPABILITY_KEYS } from '../../../ui-shared/model-capability'
import { isCustomModelProvider, readCustomModelView, removeCustomMapping, setCustomBinding, setCustomCapabilities } from './assistant-custom-models'

export type ToolResult = { text: string; summary: string; images?: ImageAsset[] }

export type AssistantTool = {
  id: string
  label: string
  kind: 'read' | 'write' | 'run'
  desc: string
  params: Record<string, unknown>
  plan?: (args: Record<string, unknown>) => { lead: string; table?: { head: string[]; rows: string[][] }; note?: string }
  redactArgs?: (args: Record<string, unknown>) => Record<string, unknown>
  run: (args: Record<string, unknown>, context?: ImageToolContext) => Promise<ToolResult>
}

/* ─── 入参读取（模型给的东西不一定守规矩，坏值一律当缺）── */

function str(value: unknown, max = 256): string {
  return typeof value === 'string' ? value.trim().slice(0, max) : ''
}
function bool(value: unknown): boolean | undefined {
  return typeof value === 'boolean' ? value : undefined
}
function num(value: unknown): number | undefined {
  return typeof value === 'number' && Number.isFinite(value) ? value : undefined
}
function rows(value: unknown): Array<Record<string, unknown>> {
  return Array.isArray(value)
    ? value.filter((item): item is Record<string, unknown> => Boolean(item) && typeof item === 'object')
    : []
}

/** 账号点名：id 精确 > 名字精确 > 名字子串（唯一命中才算） */
function findAccount(accounts: Account[], ref: string): Account {
  const needle = ref.trim()
  if (!needle) throw new Error(t('No account name given'))
  const byId = accounts.find(row => row.id === needle)
  if (byId) return byId
  const exact = accounts.filter(row => accountName(row) === needle)
  if (exact.length === 1) return exact[0]
  const fuzzy = accounts.filter(row => accountName(row).includes(needle))
  if (fuzzy.length === 1) return fuzzy[0]
  const candidates = (exact.length ? exact : fuzzy).map(accountName).slice(0, 6)
  if (!candidates.length) throw new Error(t('No account found named "{name}"', { name: needle }))
  throw new Error(
    t('"{name}" matches multiple accounts: {list}. Retry with the full account name.', {
      name: needle,
      list: candidates.join(', '),
    }),
  )
}

async function readAccounts(): Promise<Account[]> {
  const data = await fetchAccounts()
  // 全局队列 = priority 升序（后端 priority 越小越靠前）
  return (data.accounts || [])
    .slice()
    .sort((a, b) => Number(a.priority ?? 0) - Number(b.priority ?? 0))
}

async function readModels(): Promise<AssistantModelView> {
  const [view, custom] = await Promise.all([fetchManage(), readCustomModelView()])
  if (!Array.isArray(view.models) || !Array.isArray(view.mappings)) throw new Error(t('Failed to load the model list'))
  return {
    models: [...view.models, ...custom.models],
    mappings: [...view.mappings, ...custom.mappings],
    providers: [...(view.providers || []), ...custom.providers],
  }
}

/* ─── 原有工具（诊断读取由 assistant-read-tools 共享）── */

const listModels: AssistantTool = createModelListTool(readModels, t)

const listAccountBalance: AssistantTool = createBalanceTool({
  accounts: () => fetchAccounts(),
  usageSnapshot: () => fetchUsageSnapshot(),
  refreshBalance: id => fetchAccountUsage(id),
}, t)

/** 3 · 启用 / 禁用模型（改动） */
const setModelEnabledTool: AssistantTool = {
  id: 'set_model_enabled',
  label: t('Enable / disable models'),
  kind: 'write',
  desc: 'Turn on or off the availability of one or more models on the gateway (at the granularity of "provider× model"). Once disabled, downstream requests are no longer routed to it. Several can be changed at once.',
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
  run: async (args, context) => runToolBatch(rows(args.items), context, async item => {
    const provider = str(item.provider)
    const id = str(item.id)
    const enabled = bool(item.enabled)
    if (!provider || !id || enabled === undefined) throw new Error(t('Incomplete parameters (provider / id / enabled required)'))
    if (isCustomModelProvider(provider)) await setCustomBinding(provider, id, id, enabled, context)
    else await setModelEnabled({ provider, id, enabled })
    return `${enabled ? 'enabled' : 'Disable'} ${provider}/${id}`
  }, t),
}

/** 4 · 设置模型映射 ID（改动） */
const setModelMapping: AssistantTool = {
  id: 'set_model_mapping',
  label: t('Set model mapping ID'),
  kind: 'write',
  desc: 'Add or remove mapping names (external names) for a model. The same external name may be registered on multiple providers: downstream clients request with the same name, and the gateway failovers between providers by account priority; you can also prefix same-named models from different providers to target one specific provider. Multiple mappings can be changed in one call. Removing a mapping name only makes it unavailable to external clients; no model is deleted.",',
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
            action: { type: 'string', enum: ['add', 'remove'], description: 'add = add, remove= Delete' },
          },
          required: ['provider', 'id', 'alias', 'action'],
        },
      },
    },
    required: ['items'],
  },
  run: async (args, context) => runToolBatch(rows(args.items), context, async item => {
    const provider = str(item.provider)
    const target = str(item.id)
    const alias = str(item.alias, 128)
    const action = str(item.action)
    if (!provider || !target || !alias || !['add', 'remove'].includes(action)) throw new Error(t('Incomplete parameters (provider / id / alias / action required)'))
    if (isCustomModelProvider(provider)) {
      if (action === 'remove') await removeCustomMapping(provider, alias, target, context)
      else await setCustomBinding(provider, alias, target, true, context)
    } else {
      await apiCall(`/api/models/mappings${action === 'remove' ? '/remove' : ''}`, { method: 'POST', body: { provider, target, alias } })
    }
    return `${action === 'remove' ? 'Delete' : 'Added'} ${alias} → ${provider}/${target}`
  }, t),
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
  desc: 'Change the capability flags a model declares downstream: context window, max output tokens, and four booleans — tool calling / Images / Videos / deep reasoning. Only pass the keys you want to change; null restores the upstream original value for a boolean.",',
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
          supportsToolCall: { type: ['boolean', 'null'], description: 'Whether tool calling is supported' },
          supportsImages: { type: ['boolean', 'null'], description: 'Whether image recognition is supported' },
          supportsVideo: { type: ['boolean', 'null'], description: 'Whether video recognition is supported' },
          supportsReasoning: { type: ['boolean', 'null'], description: 'Whether deep reasoning is supported' },
        },
      },
    },
    required: ['provider', 'id', 'capabilities'],
  },
  run: async (args, context) => {
    checkToolSignal(context)
    const provider = str(args.provider)
    const id = str(args.id)
    const raw = (args.capabilities && typeof args.capabilities === 'object'
      ? args.capabilities
      : {}) as Record<string, unknown>
    const caps: Record<string, unknown> = {}
    for (const key of CAPABILITY_KEYS) {
      if (!(key in raw)) continue
      const value = raw[key]
      if (value === null) {
        caps[key] = null
        continue
      }
      if (key === 'maxInputTokens' || key === 'maxOutputTokens') {
        const value2 = num(value)
        if (value2 === undefined) throw new Error(t('{key} must be a number', { key: CAPABILITY_LABELS[key] }))
        caps[key] = Math.round(value2)
      } else {
        const value2 = bool(value)
        if (value2 === undefined) {
          throw new Error(t('{key} must be true / false / null', { key: CAPABILITY_LABELS[key] }))
        }
        caps[key] = value2
      }
    }
    if (!provider || !id || !Object.keys(caps).length) {
      throw new Error(t('Incomplete parameters (provider / id / capabilities required)'))
    }
    if (isCustomModelProvider(provider)) await setCustomCapabilities(provider, id, caps, context)
    else await apiCall('/api/models/capabilities', { method: 'POST', body: { provider, id, capabilities: caps } })
    return {
      text: JSON.stringify({ ok: true, provider, id, capabilities: caps }),
      summary: t('Changed {n} capabilities', { n: Object.keys(caps).length }),
    }
  },
}

/** 6 · 发起模型测试（消耗额度） */
const testModel: AssistantTool = {
  id: 'test_model',
  label: t('Run a model test'),
  kind: 'run',
  desc: 'Send one real chat request to a given model and report connectivity, duration, and time to first token. This consumes the account\'s quota. Use it to answer "does this model work / is it fast".",',
  params: {
    type: 'object',
    properties: {
      provider: { type: 'string', description: 'Provider id' },
      id: { type: 'string', description: 'Model ID' },
      prompt: { type: 'string', description: 'The user prompt used for testing; leave blank= One greeting by default' },
    },
    required: ['provider', 'id'],
  },
  run: async (args, context) => {
    checkToolSignal(context)
    if (!context?.apiKey) throw new Error(t('Select an API Key first'))
    const testId = crypto.randomUUID()
    const payload: Record<string, unknown> = { provider: str(args.provider), model: str(args.id), test_id: testId }
    const prompt = str(args.prompt, 4000)
    if (prompt) payload.prompt = prompt
    const cancel = () => { void apiCall(`/api/stats/requests/terminate?id=${encodeURIComponent(testId)}`, { method: 'POST' }).catch(() => {}) }
    context?.signal?.addEventListener('abort', cancel, { once: true })
    let result: Record<string, unknown> | undefined
    try {
      result = await apiCall('/api/models/test', { method: 'POST', body: payload, apiKey: context.apiKey }) as Record<string, unknown> | undefined
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
      summary: ok
        ? t('Reachable · {ms}ms', { ms: String(slim.duration_ms ?? '') })
        : t('Failed · {status}', { status: status || '—' }),
    }
  },
}

/** 7 · 调整提供商顺位（改动）—— 整组移动 */
const setProviderOrder: AssistantTool = {
  id: 'set_provider_order',
  label: t('Reorder providers'),
  kind: 'write',
  desc: 'Adjusts the **whole-group position** (which group number) of a given provider in the account global queue, deciding which provider is tried first at routing time. All of that provider\'s accounts move as one block. Positions are counted from 1.",',
  params: {
    type: 'object',
    properties: {
      provider: { type: 'string', description: 'Provider id' },
      position: { type: 'number', description: 'The target group position (1= at the very front)' },
    },
    required: ['provider', 'position'],
  },
  run: async (args, context) => {
    checkToolSignal(context)
    const provider = str(args.provider).toLowerCase()
    const position = num(args.position)
    if (!provider || position === undefined) {
      throw new Error(t('Incomplete parameters (provider / position required)'))
    }
    const accounts = await readAccounts()
    const groups = providerGroups(accounts)
    const from = groups.findIndex(group => group.provider === provider)
    if (from < 0) throw new Error(t('No accounts found for provider "{provider}"', { provider }))
    const to = Math.max(0, Math.min(groups.length - 1, Math.round(position) - 1))
    const [moved] = groups.splice(from, 1)
    groups.splice(to, 0, moved)
    checkToolSignal(context)
    await apiCall('/api/accounts/reorder', {
      method: 'POST',
      body: { queue: groups.flatMap(group => group.ids) },
    })
    return {
      text: JSON.stringify({ ok: true, provider, from: from + 1, to: to + 1, moved: moved.ids.length }),
      summary: t('Moved to position {n}', { n: to + 1 }),
    }
  },
}

/** 8 · 调整账号优先级（改动） */
const setAccountOrder: AssistantTool = {
  id: 'set_account_order',
  label: t('Adjust account priority'),
  kind: 'write',
  desc: 'Move a given account to position N in the account global queue (1= at the very front). Position decides the primary/backup and failover order: the earlier, the sooner it is used.',
  params: {
    type: 'object',
    properties: {
      account: { type: 'string', description: 'The account name or account id' },
      position: { type: 'number', description: 'The target position (1= at the very front)' },
    },
    required: ['account', 'position'],
  },
  run: async (args, context) => {
    checkToolSignal(context)
    const ref = str(args.account, 128)
    const position = num(args.position)
    if (!ref || position === undefined) {
      throw new Error(t('Incomplete parameters (account / position required)'))
    }
    const accounts = await readAccounts()
    const target = findAccount(accounts, ref)
    const ids = accounts.map(row => row.id).filter(Boolean)
    const from = ids.indexOf(target.id)
    if (from < 0) throw new Error(t('The account is not in the current queue'))
    ids.splice(from, 1)
    const to = Math.max(0, Math.min(ids.length, Math.round(position) - 1))
    ids.splice(to, 0, target.id)
    checkToolSignal(context)
    await apiCall('/api/accounts/reorder', { method: 'POST', body: { queue: ids } })
    return {
      text: JSON.stringify({ ok: true, account: accountName(target), from: from + 1, to: to + 1 }),
      summary: t('Moved to position {n}', { n: to + 1 }),
    }
  },
}

/** 9 · 启用 / 禁用账号（高危） */
const setAccountEnabledTool: AssistantTool = {
  id: 'set_account_enabled',
  label: t('Enable / disable accounts'),
  kind: 'write',
  desc: 'Turns the forwarding eligibility of one or more accounts on or off. Disabling only removes them from the queue (balance can still be queried), but if a line is left with only that account, disabling cuts it off outright. Several can be changed at once.",',
  params: {
    type: 'object',
    properties: {
      items: {
        type: 'array',
        description: 'The account list to change',
        items: {
          type: 'object',
          properties: {
            account: { type: 'string', description: 'The account name or account id' },
            enabled: { type: 'boolean', description: 'true = Enable, false= Disable' },
          },
          required: ['account', 'enabled'],
        },
      },
    },
    required: ['items'],
  },
  run: async (args, context) => {
    checkToolSignal(context)
    const accounts = await readAccounts()
    return runToolBatch(rows(args.items), context, async item => {
      const enabled = bool(item.enabled)
      if (enabled === undefined) throw new Error(t('Incomplete parameters (account / enabled required)'))
      const target = findAccount(accounts, str(item.account, 128))
      await setAccountEnabled(target.id, enabled)
      return `${enabled ? 'enabled' : 'Disable'} ${accountName(target)}`
    }, t)
  },
}

/** 附件读取与账号文件导入：与桌面端共用同一份定义，注入手机端的 HTTP 调用。 */
const readAttachment = createAttachmentReadTool(t)
const importAccount = createAccountImportTool({
  post: (path, body, signal) => apiCall(path, { method: 'POST', body, signal }),
}, t)

/** 全部工具（顺序 = 设置页清单顺序：只读在前，改动在后，高危最后） */
export const ASSISTANT_TOOLS: AssistantTool[] = [
  ...ASSISTANT_READ_TOOLS,
  readAttachment,
  listModels,
  listAccountBalance,
  setModelEnabledTool,
  setModelMapping,
  setModelCapability,
  testModel,
  ...ASSISTANT_IMAGE_TOOLS,
  setProviderOrder,
  setAccountOrder,
  setAccountEnabledTool,
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
