/**
 * AI for （SSE）。
 *
 * ── request ──────────────────────────────────────────────
 * `POST /api/assistant/chat`，** by to up **。 、
 * delta —— by 、、429 、、
 * in that 1 （ `api::assistant` module header）。 not has up ，
 * not 。
 *
 * ── Why not use apiCall ──────────────────────────────────────
 * `apiCall` will response `{success, data}` ； for is SSE，
 * need 。 this 1 fetch， use client.ts 、
 * and 401 （ that must and its request 1 ，otherwise「」
 * in for will 1 ）。
 *
 * ── ────────────────────────────────────────────────────
 * and OpenAI 1 ：`data: {choices:[{delta:{content, tool_calls}}]}`，
 * `data: [DONE]` 。 by delta ；`tool_calls` by `index` 
 * （ 1 parameter will ， not can use ）。
 */

import { ApiError, apiCall, authHeaders, refreshPanelAuth, refreshPanelAuthIfExpiring, resolveUrl } from '../api/client'
import type { ChatMessage } from '../../../ui-shared/assistant-history'
import { parseAssistantMessages, assistantTitle } from '../../../ui-shared/assistant-history'
export type { ChatMessage } from '../../../ui-shared/assistant-history'
import { readAssistantStream, type ChatEvent, type ToolCall } from '../../../ui-shared/assistant-stream'
import { userContentWithFiles } from '../../../ui-shared/assistant-files'
import { t } from '../i18n'
import { serverIdentity, apiBase } from '../lib/servers'
import { readModelApiKeySelection, rememberModelApiKeySelection } from '../../../ui-shared/model-api-key-memory'
export type { ChatEvent, ToolCall } from '../../../ui-shared/assistant-stream'

export type ChatRole = 'user' | 'assistant' | 'tool'

/** config（ use to ） */
export type AssistantConfig = {
  model?: string | null
  apiKeyId?: string | null
  reasoning?: string | null
  mode?: 'confirm' | 'full'
  systemPromptEnabled?: boolean
  systemPrompt?: string
  titleModel?: string | null
  titlePrompt?: string
  /** ：tool id → is （ after already ） */
  tools?: Record<string, boolean>
}

export async function fetchAssistantConfig(): Promise<AssistantConfig> {
  const identity = serverIdentity()
  const scope = apiBase()
  const payload = await apiCall<{ config?: AssistantConfig }>('/api/assistant/config')
  if (serverIdentity() !== identity) throw new DOMException('服务器已切换', 'AbortError')
  if (!payload.config) throw new Error(t('配置响应不完整'))
  const apiKeyId = typeof payload.config.apiKeyId === 'string' && payload.config.apiKeyId.trim() ? payload.config.apiKeyId.trim()
    : readModelApiKeySelection(scope, 'assistant')
  if (apiKeyId) rememberModelApiKeySelection(scope, 'assistant', apiKeyId)
  return { ...payload.config, apiKeyId }
}

/**
 * 1 for and 。
 *
 * `signal` use 「」：abort after AbortError —— that is 
 * use ， not is error（ use by `signal.aborted` ）。
 */
export async function streamAssistantChat(options: {
  apiKey: string
  messages: ChatMessage[]
  /** （OpenAI tools ）；empty = */
  tools?: Array<Record<string, unknown>>
  model?: string | null
  reasoning?: string | null
  signal?: AbortSignal
  onEvent: (event: ChatEvent) => void
}): Promise<void> {
  const body: Record<string, unknown> = {
    messages: options.messages.map(wireMessage),
  }
  // not model when by use 「set」 default
  if (options.model) body.model = options.model
  if (options.reasoning) body.reasoning = options.reasoning
  // as empty when ** not need empty**： after by 「 has no has tools」 not 
  // （ config::AssistantConfig::system_prompt），empty and not not is 1 
  if (options.tools && options.tools.length) body.tools = options.tools

  const identity = serverIdentity()
  const target = resolveUrl('/api/assistant/chat')
  const send = async (): Promise<Response> => {
    if (serverIdentity() !== identity) throw new DOMException('服务器已切换', 'AbortError')
    return fetch(target, {
      method: 'POST',
      headers: { ...authHeaders(), 'x-api-key': options.apiKey, Authorization: `Bearer ${options.apiKey}`, 'Content-Type': 'application/json', Accept: 'text/event-stream' },
      body: JSON.stringify(body),
      signal: options.signal,
    })
  }

  // to before ： this not apiCall， not this 1 will in access that 1 
  // 1 401（ client.ts refreshPanelAuthIfExpiring）
  await refreshPanelAuthIfExpiring()
  let response = await send()
  // 401 1 （ and apiCall 1 ）： up ， not up to 
  if (response.status === 401 && (await refreshPanelAuth())) {
    response = await send()
  }

  if (!response.ok || !response.body) {
    const text = await response.text().catch(() => '')
    let message = text.trim()
    try {
      const parsed = JSON.parse(text)
      message = String(parsed?.error?.message || parsed?.error || message)
    } catch {
      /* JSON error： use */
    }
    if (response.status === 404 && message === 'Not found: POST /api/assistant/chat') {
      throw new ApiError(t('这一版网关还没有 AI 助手接口（HTTP 404），请升级网关后重试。'), 404)
    }
    throw new ApiError(message || `HTTP ${response.status}`, response.status)
  }

  await readAssistantStream(response, {
    onEvent: options.onEvent,
    signal: options.signal,
    incompleteMessage: t('模型响应未完整结束，请重试；未执行其中的工具调用。'),
  })
}

/**
 * 1 → to up wire 。
 *
 * and OpenAI for ：assistant use when need has `tool_calls`，
 * use `role: "tool"` + `tool_call_id` —— 1 ，
 * up will request（ or in down 1 up down ）。
 */
function wireMessage(message: ChatMessage): Record<string, unknown> {
  if (message.role === 'tool') {
    return { role: 'tool', tool_call_id: message.toolCallId, content: message.content }
  }
  if (message.role === 'assistant') {
    const wire: Record<string, unknown> = { role: 'assistant', content: message.content || '' }
    if (message.toolCalls?.length) {
      wire.tool_calls = message.toolCalls.map(call => ({
        id: call.id,
        type: 'function',
        function: { name: call.name, arguments: call.arguments },
      }))
    }
    return wire
  }
  const images = (message.images || []).filter((image): image is string => typeof image === 'string')
  const text = userContentWithFiles(message.content, message.files || [])
  if (!images.length) return { role: 'user', content: text }
  return { role: 'user', content: [
    { type: 'text', text },
    ...images.map(url => ({ type: 'image_url', image_url: { url } })),
  ] }
}

export type SessionSummary = { id: string; title: string; model: string | null; reasoning: string | null; createdAt: number; updatedAt: number; titleGenerated: boolean }
export type SessionDetail = SessionSummary & { messages: ChatMessage[] }

export async function saveAssistantSession(id: string, messages: ChatMessage[], model?: string | null, title?: string): Promise<SessionSummary> {
  const data = await apiCall<{ session: SessionSummary }>('/api/assistant/sessions', { method: 'POST', body: { id, title: title ?? assistantTitle(messages), messages, model: model || null } })
  return data.session
}

export async function listAssistantSessions(): Promise<SessionSummary[]> {
  const data = await apiCall<{ sessions: SessionSummary[] }>('/api/assistant/sessions')
  return data.sessions
}

export async function getAssistantSession(id: string): Promise<SessionDetail> {
  const data = await apiCall<{ session: SessionDetail }>(`/api/assistant/sessions/${encodeURIComponent(id)}`)
  return { ...data.session, messages: parseAssistantMessages(data.session.messages) }
}

export async function forkAssistantSession(id: string, messageId: string): Promise<SessionDetail> {
  const data = await apiCall<{ session: SessionDetail }>(`/api/assistant/sessions/${encodeURIComponent(id)}/fork`, { method: 'POST', body: { id: crypto.randomUUID(), messageId } })
  return { ...data.session, messages: parseAssistantMessages(data.session.messages) }
}

export async function summarizeAssistantSession(id: string, apiKey: string): Promise<SessionSummary> {
  const data = await apiCall<{ session: SessionSummary }>(`/api/assistant/sessions/${encodeURIComponent(id)}/title`, { method: 'POST', apiKey })
  return data.session
}

export async function saveAssistantConfig(config: AssistantConfig): Promise<AssistantConfig> {
  const identity = serverIdentity()
  const scope = apiBase()
  rememberModelApiKeySelection(scope, 'assistant', config.apiKeyId)
  const data = await apiCall<{ config: AssistantConfig }>('/api/assistant/config', { method: 'PUT', body: { config } })
  if (serverIdentity() !== identity) throw new DOMException('服务器已切换', 'AbortError')
  rememberModelApiKeySelection(scope, 'assistant', data.config.apiKeyId)
  return data.config
}

export async function saveAssistantTitleConfig(config: AssistantConfig, titleModel: string | null, titlePrompt: string): Promise<AssistantConfig> {
  return saveAssistantConfig({ ...config, titleModel, titlePrompt })
}

export async function deleteAssistantSession(id: string): Promise<void> {
  await apiCall(`/api/assistant/sessions/${encodeURIComponent(id)}`, { method: 'DELETE' })
}
