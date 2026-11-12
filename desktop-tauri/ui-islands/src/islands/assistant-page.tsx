/**
 * AI （「 can 」 that 1 ）。
 *
 * ── 1 ──────────────────────────────────────────────
 * will + for （ / / ）+ up set。
 * `POST /api/assistant/chat`（SSE， assistant-state），
 * and 「」 in this 1 （ assistant-tools module header）。
 *
 * ── agent loop （file need 1 ）────────────────
 * 1 = 1 up use 。 up `tool_calls` by ** **：
 * · → ，， down 1 ；
 * · / → by ：
 * `confirm` down （`suspendedRef`）， etc use 「」 or 「」；
 * `full` ****，「 already 」 in for ；
 * after from down 1 （`pump` ）；
 * · → down as `role:"tool"` ， down 1 。
 * limit `MAX_ROUNDS` is ：「」 when use not this 
 * none。
 *
 * ── Why use ref + setState ─────────────────────────
 * and is ****， not to setState after new ；
 * in `uiRef` / `wireRef` ref （UI / up ），
 * `syncUI()` ref state 。 this is in use 
 * （ accounts-page ）， in prev not 。
 *
 * ── and 「set」 ──────────────────────────────────────
 * config（default as / / ） already from set 
 * （assistant-settings-drawer）。 not ： `/api/assistant/config`。
 */

import * as React from 'react'
import { mountOnPage } from '../island-lazy'
import { createRoot } from 'react-dom/client'
import { t } from '../i18n'
import {
  AssistantRequestError,
  DEFAULT_SYSTEM_PROMPT,
  gatewayBase,
  loadSession,
  loadSessions,
  parseToolArguments,
  newSessionId,
  streamChat,
  toast,
  type AssistantConfig,
  type ChatMessage,
  type PermissionMode,
  type ToolCall,
} from './assistant-state'
import {
  ASSISTANT_TOOLS,
  errorText,
  toolById,
  toolsPayload,
  type ToolResult,
} from './assistant-tools'
import { selectedModel, unavailableModelMessage } from './assistant-models'
import { useAssistantModels } from './assistant-model-state'
import { useAssistantConfig } from './assistant-config-state'
import { titleFrom, useAssistantSessions } from './assistant-session-state'
import { AssistantComposer } from './assistant-composer'
import { useAssistantAttachments } from './assistant-attachments'
import { MessageList, type UiMessage } from './assistant-messages'
import { AssistantSettingsDrawer } from './assistant-settings-drawer'
import { AssistantEmpty, AssistantSidebar, AssistantTopbar } from './assistant-shell'
import { useAssistantMessageOperations } from './assistant-message-operations'
import { AssistantTitleSettings } from '../../../ui-shared/assistant-title-settings'
import { replayMessages } from './assistant-replay'
import { imageContext, type ImageAsset } from '../../../ui-shared/assistant-image-tools'
import { assistantFileContext, type AssistantTextFile } from '../../../ui-shared/assistant-files'
import { useModelApiKeys } from '../../../ui-shared/use-model-api-keys'
import { ModelApiKeyField } from '../../../ui-shared/model-api-key-field'
import { modelApiKeyAllows, modelApiKeyLabel, type ModelApiKey } from '../../../ui-shared/model-api-key'
import { callJson } from './assistant-state'

/** 1 for 「 up use → → up use 」 */
const MAX_ROUNDS = 12

/** ： after from `index` */
type Suspended = {
  calls: ToolCall[]
  index: number
  collected: Array<{ toolCallId: string; content: string; images?: ImageAsset[] }>
  cardIndex: number
}

/* ─── ─────────────────────────────────────────────── */

function AssistantPage() {
  const { config, configRef, status, failed: loadingFailed, load: loadConfig, save: saveConfig } = useAssistantConfig()
  const keyState = useModelApiKeys(() => callJson('GET', '/api/model-access/keys'))
  const selectedKey = keyState.keys.find(key => key.id === config?.apiKeyId)
  const requestKeyRef = React.useRef<ModelApiKey | undefined>(undefined)
  const { models, ready: modelsReady, loading: modelsLoading, error: modelsError, refresh: refreshModels } = useAssistantModels(selectedKey?.key)
  const [ui, setUi] = React.useState<UiMessage[]>([])
  const [busy, setBusy] = React.useState(false)
  const [awaiting, setAwaiting] = React.useState(false)
  const [draft, setDraft] = React.useState('')
  /** will （） and current that 1 */
  const [activeId, setActiveId] = React.useState<string>('')
  /** in down is （， CSS .asst-layout） */
  const [sidebarOpen, setSidebarOpen] = React.useState(false)
  /** up 「set」 */
  const [settingsOpen, setSettingsOpen] = React.useState(false)
  const [titleSettingsOpen, setTitleSettingsOpen] = React.useState(false)

  const uiRef = React.useRef<UiMessage[]>([])
  const wireRef = React.useRef<ChatMessage[]>([])
  const suspendedRef = React.useRef<Suspended | null>(null)
  const controllerRef = React.useRef<AbortController | null>(null)
  const scrollRef = React.useRef<HTMLDivElement | null>(null)
  const loadedRef = React.useRef(false)
  /** current will id（`uiRef` that 1 ； new when 1 ） */
  const activeIdRef = React.useRef<string>('')
  const { images, setImages, files, setFiles, addFiles, reading: readingAttachments } = useAssistantAttachments(activeIdRef)
  /** will （ use no 1 1 use ， `titleFrom`） */
  const titleRef = React.useRef<string>('')
  const { sessions, setSessions, persist, summarize, remove: removeSavedSession, clear: clearSavedSessions } = useAssistantSessions({
    config: configRef, messages: wireRef, ui: uiRef, activeId: activeIdRef, title: titleRef, apiKey: requestKeyRef,
  })
  const stoppedRef = React.useRef(false)
  const historyBusyRef = React.useRef(false)
  const [historyBusy, setHistoryBusy] = React.useState(false)
  /**
   * `loop` use （****）。
   *
   * `pump` and `loop` use （ need down 1 ）， React useCallback
   * 1 deps is 。 up can `pump` deps as empty、
   * this ref ** new ** `loop`： no has ，`pump` will has that 1 
   * `loop`， that 1 `enabledTools` is empty —— as 「
   * not to 」。
   */
  const loopRef = React.useRef<(() => Promise<void>) | null>(null)
  /** use already （： MAX_ROUNDS） */
  const sendingRef = React.useRef(false)
  const checkingModelRef = React.useRef(false)
  const preflightRef = React.useRef(0)
  const requestModelRef = React.useRef<string>('')
  const roundRef = React.useRef(0)

  const refreshModelOptions = React.useCallback(() => { void refreshModels().catch(() => {}) }, [refreshModels])

  const syncUI = React.useCallback(() => { setUi(uiRef.current.slice()) }, [])

  /** `persist` use （`loop` deps will ， loopRef ） */
  const persistRef = React.useRef(persist)
  persistRef.current = persist
  const summarizeRef = React.useRef(summarize)
  summarizeRef.current = summarize

  /* ： new when to （ use up when not —— in when ） */
  React.useEffect(() => {
    const el = scrollRef.current
    if (!el) return
    const near = el.scrollHeight - el.scrollTop - el.clientHeight < 200
    if (near) el.scrollTop = el.scrollHeight
  }, [ui])

  /* ：config + （activate when ； use 1 ）
   *
   * ****， not 1 try：config is HTTP
   * （`/api/assistant/config`）， is —— after 
   * or old when ， not this 「」 1 empty 。
   *
   * configfailure will ****（ has ）：dev when can can HTTP
   * ， that 1 failure「」， not 「」。 */
  const activate = React.useCallback(async () => {
    void loadConfig()
    void refreshModels().catch(() => {})
    if (loadedRef.current) return
    loadedRef.current = true
    try {
      setSessions(await loadSessions())
    } catch (error) {
      toast(t('读取历史会话失败：{message}', { message: errorText(error) }), 'err')
    }
    // 1 empty will （ not etc use 「 new for 」： can ）
    if (!activeIdRef.current) {
      const id = newSessionId()
      activeIdRef.current = id
      titleRef.current = ''
      setActiveId(id)
    }
  }, [loadConfig, refreshModels])

  React.useEffect(() => {
    const api = window as unknown as { wbAssistant?: { activate?: () => void } }
    api.wbAssistant = { activate: () => { void activate() } }
    void activate()
    return () => { delete api.wbAssistant }
  }, [activate])

  const toolEnabled = React.useMemo(() => {
    const map: Record<string, boolean> = {}
    for (const tool of ASSISTANT_TOOLS) map[tool.id] = config?.tools?.[tool.id] ?? false
    return map
  }, [config])
  const enabledTools = React.useMemo(() => toolsPayload(toolEnabled), [toolEnabled])
  const enabledCount = Object.values(toolEnabled).filter(Boolean).length

  const currentModel = config?.model ?? null
  const currentOption = selectedModel(models, currentModel)
  const modelSupportsImages = currentOption?.supportsImages === true
  const modelWarning = modelsError
    ? t('读取模型清单失败：{message}', { message: modelsError })
    : !modelsReady ? t('正在读取可用模型…')
    : currentModel && !currentOption ? unavailableModelMessage(currentModel) : ''
  const mode: PermissionMode = config?.mode ?? 'confirm'

  const runTool = React.useCallback(async (tool: NonNullable<ReturnType<typeof toolById>>, args: Record<string, unknown>, collected: Suspended['collected']): Promise<ToolResult> => {
    const controller = new AbortController()
    controllerRef.current = controller
    const session = activeIdRef.current
    try {
      if (tool.id === 'generate_images' || tool.id === 'edit_images') await persistRef.current(true, collected)
      if (controller.signal.aborted) throw new DOMException('Aborted', 'AbortError')
      const references = imageContext([...wireRef.current, ...collected.map(item => ({ role: 'tool', images: item.images }))])
      const attachmentFiles = assistantFileContext(wireRef.current)
      const key = requestKeyRef.current
      const result = await tool.run(args, { session, images: references, files: attachmentFiles, confirmed: true, signal: controller.signal, apiKey: key?.key, apiKeyName: key?.name })
      if (activeIdRef.current !== session) throw new DOMException('Aborted', 'AbortError')
      return result
    } finally {
      if (controllerRef.current === controller) controllerRef.current = null
    }
  }, [])

  /* ─── agent loop ───────────────────────────────────────── */

  /** 1 tool_calls（ from `index` ；confirm to will ） */
  const pump = React.useCallback(async (calls: ToolCall[], index: number, collected: Suspended['collected']) => {
    setBusy(true)
    const session = activeIdRef.current
    for (let i = index; i < calls.length; i += 1) {
      if (activeIdRef.current !== session) return
      if (stoppedRef.current) {
        for (const call of calls.slice(i)) {
          uiRef.current.push({ kind: 'tool', call, state: 'cancelled' })
          collected.push({ toolCallId: call.id, content: '用户已停止，未执行这次操作。' })
        }
        break
      }
      const call = calls[i]
      const tool = toolById(call.name)
      const args = parseToolArguments(call.arguments)
      if (!tool) {
        uiRef.current.push({
          kind: 'tool', call, state: 'failed',
          error: t('未知工具「{name}」：它在设置里可能刚被取消勾选，或这一版助手还不认识它。', { name: call.name }),
        })
        collected.push({ toolCallId: call.id, content: `未知工具 ${call.name}，未执行。` })
        syncUI()
        continue
      }
      if (configRef.current?.tools?.[tool.id] !== true) {
        const message = t('工具「{name}」未获授权，未执行。', { name: tool.label })
        uiRef.current.push({ kind: 'tool', call, state: 'failed', error: message })
        collected.push({ toolCallId: call.id, content: message })
        syncUI()
        continue
      }
      const refreshingBalance = tool.id === 'list_account_balance' && args.refresh === true
      if (tool.kind === 'read' && !refreshingBalance) {
        const cardIndex = uiRef.current.push({ kind: 'tool', call, state: 'running' }) - 1
        syncUI()
        try {
          const result = await runTool(tool, args, collected)
          uiRef.current[cardIndex] = { kind: 'tool', call, state: 'done', summary: result.summary, detail: result.text, images: result.images }
          collected.push({ toolCallId: call.id, content: result.text, images: result.images })
          if (result.images?.length) void persistRef.current(true, collected)
        } catch (error) {
          if (error instanceof DOMException && error.name === 'AbortError') {
            if (activeIdRef.current !== session) return
            stoppedRef.current = true
            uiRef.current[cardIndex] = { kind: 'tool', call, state: 'cancelled' }
            collected.push({ toolCallId: call.id, content: '用户已停止，未继续执行这次操作。' })
            syncUI()
            continue
          }
          const message = errorText(error)
          uiRef.current[cardIndex] = { kind: 'tool', call, state: 'failed', error: message }
          collected.push({ toolCallId: call.id, content: `工具执行失败：${message}` })
        }
        syncUI()
        continue
      }

      // / ：「」 is 「」
      const plan = tool.plan?.(args) || {
        lead: t('将执行 {label}。', { label: tool.label }),
        ...(refreshingBalance ? { note: t('实时余额查询会请求上游，并可能触发已配置的自动禁用规则。') } : {}),
      }
      const auto = (configRef.current?.mode ?? 'confirm') === 'full'
      if (!auto) {
        const cardIndex = uiRef.current.push({ kind: 'tool', call, state: 'confirm', plan }) - 1
        suspendedRef.current = { calls, index: i, collected, cardIndex }
        setAwaiting(true)
        setBusy(false)
        syncUI()
        return
      }
      // ： not down ， in （「 already 」）
      const cardIndex = uiRef.current.push({ kind: 'tool', call, state: 'running', plan, auto: true }) - 1
      syncUI()
      try {
        const result = await runTool(tool, args, collected)
        uiRef.current[cardIndex] = {
          kind: 'tool', call, state: 'done', auto: true,
          summary: result.summary, detail: result.text, images: result.images,
        }
        collected.push({ toolCallId: call.id, content: result.text, images: result.images })
        if (result.images?.length) void persistRef.current(true, collected)
      } catch (error) {
        if (error instanceof DOMException && error.name === 'AbortError') {
          if (activeIdRef.current !== session) return
          stoppedRef.current = true
          uiRef.current[cardIndex] = { kind: 'tool', call, state: 'cancelled' }
          collected.push({ toolCallId: call.id, content: '用户已停止，未继续执行这次操作。' })
          syncUI()
          continue
        }
        const message = errorText(error)
        uiRef.current[cardIndex] = { kind: 'tool', call, state: 'failed', auto: true, error: message }
        collected.push({ toolCallId: call.id, content: `工具执行失败：${message}` })
      }
      syncUI()
    }
    // 1 ：， down 1 （ ref new loop， ）
    for (const item of collected) {
      wireRef.current.push({ role: 'tool', toolCallId: item.toolCallId, content: item.content, images: item.images })
    }
    syncUI()
    await persistRef.current()
    if (stoppedRef.current) { setBusy(false); return }
    await loopRef.current?.()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  /** 1 up use （ delta / tool_calls） */
  const loop = React.useCallback(async () => {
    if (stoppedRef.current) { setBusy(false); return }
    const session = activeIdRef.current
    if (roundRef.current >= MAX_ROUNDS) {
      uiRef.current.push({
        kind: 'notice',
        text: t('这一轮来回已达 {n} 次上限，我先停下等你指示（可以换个说法，或拆成几步让我做）。', { n: MAX_ROUNDS }),
        bad: true,
      })
      syncUI()
      setBusy(false)
      return
    }
    roundRef.current += 1
    setBusy(true)
    const controller = new AbortController()
    controllerRef.current = controller
    const messageId = newSessionId()
    const botIndex = uiRef.current.push({ kind: 'bot', id: messageId, model: requestModelRef.current, text: '', streaming: true }) - 1
    syncUI()
    let calls: ToolCall[] = []
    let failed = false
    try {
      if (!requestKeyRef.current) throw new Error(t('请先选择一把 API Key'))
      await streamChat({
        model: requestModelRef.current,
        apiKey: requestKeyRef.current.key,
        messages: wireRef.current,
        tools: enabledTools,
        reasoning: configRef.current?.reasoning ?? null,
        signal: controller.signal,
        onEvent: event => {
          if (activeIdRef.current !== session) return
          if (event.type === 'delta') {
            const current = uiRef.current[botIndex]
            if (current && current.kind === 'bot') {
              uiRef.current[botIndex] = { ...current, text: current.text + event.text }
              syncUI()
            }
          }
          if (event.type === 'toolCalls') calls = event.calls
        },
      })
    } catch (error) {
      if (activeIdRef.current !== session) return
      failed = true
      if (error instanceof AssistantRequestError && error.code === 'model_not_found') {
        void refreshModels().catch(() => {})
      }
      const message = errorText(error)
      const current = uiRef.current[botIndex]
      const partial = current && current.kind === 'bot' ? current.text : ''
      if (!partial) uiRef.current.splice(botIndex, 1)
      else {
        uiRef.current[botIndex] = { kind: 'bot', id: messageId, model: requestModelRef.current, text: partial }
        wireRef.current.push({ role: 'assistant', id: messageId, model: requestModelRef.current, content: partial })
      }
      void persistRef.current()
      const aborted = controller.signal.aborted
      uiRef.current.push({
        kind: 'notice',
        text: aborted ? t('已停止这次回答。') : t('请求失败：{message}', { message }),
        bad: !aborted,
      })
      syncUI()
    } finally {
      if (controllerRef.current === controller) controllerRef.current = null
    }
    if (activeIdRef.current !== session) return
    if (failed) {
      setBusy(false)
      return
    }
    const current = uiRef.current[botIndex]
    const text = current && current.kind === 'bot' ? current.text : ''
    if (current && current.kind === 'bot') {
      if (!text && !calls.length) uiRef.current.splice(botIndex, 1)
      else uiRef.current[botIndex] = { ...current, streaming: false }
    }
    wireRef.current.push({ role: 'assistant', id: messageId, model: requestModelRef.current, content: text, toolCalls: calls.length ? calls : undefined })
    syncUI()
    // 1 （， persist ）： in ， not already 
    await persistRef.current()
    if (!calls.length) {
      setBusy(false)
      if (!stoppedRef.current) void summarizeRef.current(session)
      return
    }
    await pump(calls, 0, [])
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [enabledTools, pump, syncUI, refreshModels])
  loopRef.current = loop

  /** up 1 ： or ， after down use */
  const resolveConfirm = React.useCallback(async (run: boolean) => {
    const session = activeIdRef.current
    const suspended = suspendedRef.current
    if (!suspended) return
    suspendedRef.current = null
    setAwaiting(false)
    const call = suspended.calls[suspended.index]
    const tool = toolById(call.name)
    const args = parseToolArguments(call.arguments)
    if (!run || !tool) {
      uiRef.current[suspended.cardIndex] = { kind: 'tool', call, state: 'cancelled' }
      suspended.collected.push({ toolCallId: call.id, content: '用户取消了这次操作，没有做任何改动。' })
      syncUI()
    } else {
      uiRef.current[suspended.cardIndex] = { kind: 'tool', call, state: 'running' }
      setBusy(true)
      syncUI()
      try {
        if (configRef.current?.tools?.[tool.id] !== true) throw new Error(t('工具「{name}」未获授权，未执行。', { name: tool.label }))
        const result: ToolResult = await runTool(tool, args, suspended.collected)
        uiRef.current[suspended.cardIndex] = { kind: 'tool', call, state: 'done', summary: result.summary, detail: result.text, images: result.images }
        suspended.collected.push({ toolCallId: call.id, content: result.text, images: result.images })
        if (result.images?.length) void persistRef.current(true, suspended.collected)
      } catch (error) {
        if (error instanceof DOMException && error.name === 'AbortError') {
          if (activeIdRef.current !== session) return
          stoppedRef.current = true
          uiRef.current[suspended.cardIndex] = { kind: 'tool', call, state: 'cancelled' }
          suspended.collected.push({ toolCallId: call.id, content: '用户已停止，未继续执行这次操作。' })
          syncUI()
          await pump(suspended.calls, suspended.index + 1, suspended.collected)
          return
        }
        const message = errorText(error)
        uiRef.current[suspended.cardIndex] = { kind: 'tool', call, state: 'failed', error: message }
        suspended.collected.push({ toolCallId: call.id, content: `工具执行失败：${message}` })
      }
      syncUI()
    }
    await pump(suspended.calls, suspended.index + 1, suspended.collected)
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [pump, syncUI])

  /** before current，failure not not empty。 */
  const send = React.useCallback(async (text: string, attached: string[], textFiles: AssistantTextFile[]) => {
    const value = text.trim()
    if ((!value && attached.length === 0 && textFiles.length === 0) || busy || awaiting || readingAttachments || sendingRef.current || historyBusyRef.current) return
    const current = configRef.current
    if (!current?.model) {
      toast(current
        ? t('还没有选择对话模型：在输入框右下角选一个，或点右上角「助手设置」。')
        : t('助手设置没读出来，现在还不能发消息 —— 点右上角「助手设置」看看。'), 'err')
      return
    }
    const attempt = ++preflightRef.current
    sendingRef.current = true
    checkingModelRef.current = true
    setBusy(true)
    const model = current.model
    const session = activeIdRef.current
    try {
      const [latest, keys] = await Promise.all([refreshModels(), keyState.refresh()])
      if (attempt !== preflightRef.current || activeIdRef.current !== session || configRef.current?.model !== model) return
      const key = keys.find(key => key.id === current.apiKeyId)
      if (!key) {
        toast(t('请先选择一把 API Key'), 'err')
        return
      }
      if (!modelApiKeyAllows(key, model)) {
        toast(t('所选 API Key 无权调用此模型或提供商'), 'err')
        return
      }
      const option = selectedModel(latest, model)
      if (!option) {
        toast(unavailableModelMessage(model), 'err')
        return
      }
      checkingModelRef.current = false
      requestModelRef.current = option.name
      requestKeyRef.current = key
      if (!activeIdRef.current) {
        const id = newSessionId()
        activeIdRef.current = id
        setActiveId(id)
      }
      roundRef.current = 0
      stoppedRef.current = false
      const messageId = newSessionId()
      uiRef.current.push({
        kind: 'user', id: messageId, text: value,
        ...(attached.length ? { images: attached } : {}),
        ...(textFiles.length ? { files: textFiles } : {}),
      })
      wireRef.current.push({
        role: 'user', id: messageId, content: value,
        ...(attached.length ? { images: attached } : {}),
        ...(textFiles.length ? { files: textFiles } : {}),
      })
      setDraft('')
      setImages([])
      setFiles([])
      syncUI()
      await loop()
      await persistRef.current()
    } catch (error) {
      if (attempt === preflightRef.current) {
        toast(checkingModelRef.current
          ? t('读取模型清单失败：{message}', { message: errorText(error) })
          : t('请求失败：{message}', { message: errorText(error) }), 'err')
      }
    } finally {
      if (attempt === preflightRef.current) {
        checkingModelRef.current = false
        sendingRef.current = false
        setBusy(false)
      }
    }
  }, [awaiting, busy, readingAttachments, loop, syncUI, refreshModels, configRef, keyState.refresh, setFiles])

  const stop = React.useCallback(() => {
    if (checkingModelRef.current) {
      preflightRef.current += 1
      checkingModelRef.current = false
      sendingRef.current = false
      setBusy(false)
      return
    }
    stoppedRef.current = true
    controllerRef.current?.abort()
  }, [])

  /** 1 empty will （「 new for 」/ new when use ） */
  const resetToBlank = React.useCallback((id: string) => {
    preflightRef.current += 1
    checkingModelRef.current = false
    sendingRef.current = false
    controllerRef.current?.abort()
    suspendedRef.current = null
    uiRef.current = []
    wireRef.current = []
    activeIdRef.current = id
    titleRef.current = ''
    setActiveId(id)
    setAwaiting(false)
    setBusy(false)
    setImages([])
    setFiles([])
    syncUI()
  }, [syncUI, setImages, setFiles])

  const newChat = React.useCallback(() => {
    if (historyBusyRef.current || sendingRef.current) return
    // and openSession 1 by ： new for before old ， not 「 has new 」，
    // not this that to 1 。
    void persistRef.current(false)
    resetToBlank(newSessionId())
    setSidebarOpen(false)
  }, [resetToBlank])

  /**
   * 1 will 。
   *
   * in when not ： that 1 in `uiRef` ， after will 
   * ** new will ** （ 1 ref）—— is for in 1 。
   */
  const openSession = React.useCallback(async (id: string) => {
    if (busy || awaiting || sendingRef.current || historyBusyRef.current) return
    if (id === activeIdRef.current) { setSidebarOpen(false); return }
    historyBusyRef.current = true
    setHistoryBusy(true)
    try {
      await persistRef.current(false)
      const detail = await loadSession(id)
      uiRef.current = replayMessages(detail.messages, detail.model)
      wireRef.current = detail.messages
      activeIdRef.current = detail.id
      titleRef.current = detail.title
      setActiveId(detail.id)
      setImages([])
      setFiles([])
      setDraft('')
      suspendedRef.current = null
      setAwaiting(false)
      setBusy(false)
      syncUI()
      // will and etc ：（ use in is this will ）
      if (config && (detail.model || detail.reasoning)) {
        const next: AssistantConfig = {
          ...config,
          model: detail.model ?? config.model,
          reasoning: detail.reasoning,
        }
        saveConfig(next)
        refreshModelOptions()
      }
    } catch (error) {
      toast(t('打开会话失败：{message}', { message: errorText(error) }), 'err')
    } finally {
      historyBusyRef.current = false
      setHistoryBusy(false)
    }
    setSidebarOpen(false)
  }, [awaiting, busy, config, syncUI, saveConfig, refreshModelOptions])

  const removeSession = React.useCallback(async (id: string) => {
    if (busy || awaiting || sendingRef.current || historyBusyRef.current) return
    historyBusyRef.current = true
    setHistoryBusy(true)
    try {
      await removeSavedSession(id)
      if (id === activeIdRef.current) resetToBlank(newSessionId())
    } catch (error) {
      toast(t('删除会话失败：{message}', { message: errorText(error) }), 'err')
    } finally {
      historyBusyRef.current = false
      setHistoryBusy(false)
    }
  }, [busy, awaiting, resetToBlank, removeSavedSession])

  const clearHistory = React.useCallback(async () => {
    if (busy || awaiting || sendingRef.current || historyBusyRef.current) return
    historyBusyRef.current = true
    setHistoryBusy(true)
    try {
      await clearSavedSessions()
      resetToBlank(newSessionId())
    } catch (error) {
      toast(t('清空历史失败：{message}', { message: errorText(error) }), 'err')
    } finally {
      historyBusyRef.current = false
      setHistoryBusy(false)
    }
  }, [busy, awaiting, resetToBlank, clearSavedSessions])

  const pickModel = React.useCallback((name: string) => {
    if (!name) return
    if (!config) {
      toast(t('助手设置没读出来，暂时存不了对话模型。'), 'err')
      return
    }
    saveConfig({ ...config, model: name })
  }, [config, saveConfig])

  const pickReasoning = React.useCallback((level: string | null) => {
    if (!config) {
      toast(t('助手设置没读出来，暂时存不了思考等级。'), 'err')
      return
    }
    saveConfig({ ...config, reasoning: level })
  }, [config, saveConfig])

  const pickMode = React.useCallback((next: PermissionMode) => {
    if (!config) return
    saveConfig(
      { ...config, mode: next },
      next === 'full' ? t('已切换到完全访问') : t('已切换到变更前确认'),
    )
  }, [config, saveConfig])

  const messageActions = useAssistantMessageOperations({
    messages: wireRef, ui: uiRef, activeId: activeIdRef,
    busy: () => busy || awaiting || sendingRef.current || historyBusyRef.current,
    setBusy: value => { historyBusyRef.current = value; setHistoryBusy(value) },
    model: () => configRef.current?.model ?? null, sync: syncUI, persist: () => persistRef.current(),
    open: detail => {
      wireRef.current = detail.messages
      uiRef.current = replayMessages(detail.messages, detail.model)
      activeIdRef.current = detail.id
      titleRef.current = detail.title
      setActiveId(detail.id)
      setSessions(previous => [detail, ...previous])
      setImages([])
      setFiles([])
      setDraft('')
      syncUI()
      setSidebarOpen(false)
    },
  })

  /* ─── ─── */

  const activeTitle = titleRef.current || titleFrom(wireRef.current) || t('新对话')
  const empty = ui.length === 0

  return (
    <div className='asst-layout'>
      <AssistantSidebar open={sidebarOpen} onOpenChange={setSidebarOpen} sessions={sessions} activeId={activeId}
        disabled={busy || awaiting || historyBusy} settingsDisabled={!config} onNew={newChat}
        onOpen={id => { void openSession(id) }} onRemove={id => { void removeSession(id) }} onClear={() => { void clearHistory() }}
        onSettings={() => { setTitleSettingsOpen(true); refreshModelOptions() }} />

      <div className='asst-main'>
        <AssistantTopbar title={activeTitle} keyLabel={selectedKey ? modelApiKeyLabel(selectedKey) : t('请先选择一把 API Key')}
          onHistory={() => setSidebarOpen(open => !open)} onSettings={() => setSettingsOpen(true)} />

        <div className='asst-scroll' ref={scrollRef}>
          {empty ? (
            <AssistantEmpty noTools={config !== null && enabledCount === 0} failed={loadingFailed} status={status}
              onSend={text => { void send(text, [], []) }} />
          ) : (
            <div className='asst-flow'>
              <MessageList messages={ui} model={currentModel || ''} actions={{ ...messageActions, disabled: busy || awaiting || historyBusy }} onConfirm={run => void resolveConfirm(run)} />
            </div>
          )}
        </div>

        <AssistantComposer
          draft={draft}
          onDraftChange={setDraft}
          images={images}
          files={files}
          onAddFiles={addFiles}
          onRemoveImage={index => setImages(prev => prev.filter((_, i) => i !== index))}
          onRemoveFile={index => setFiles(prev => prev.filter((_, i) => i !== index))}
          modelSupportsImages={modelSupportsImages}
          imageToolsEnabled={toolEnabled.edit_images === true}
          modelAvailable={modelsReady && Boolean(currentOption) && keyState.ready && Boolean(selectedKey)}
          modelsReady={modelsReady}
          modelsLoading={modelsLoading}
          modelWarning={modelWarning}
          onRefreshModels={refreshModelOptions}
          models={models}
          model={currentModel}
          onPickModel={pickModel}
          reasoning={config?.reasoning ?? null}
          onPickReasoning={pickReasoning}
          mode={mode}
          onModeChange={pickMode}
          busy={busy || readingAttachments}
          awaiting={awaiting || historyBusy}
          onSend={() => void send(draft, images, files)}
          onStop={stop}
        />
      </div>

      <AssistantTitleSettings open={titleSettingsOpen} onOpenChange={setTitleSettingsOpen}
        model={config?.titleModel ?? null} prompt={config?.titlePrompt ?? ''} models={models.map(item => item.name)} t={t} disabled={!config}
        onSave={(titleModel, titlePrompt) => { if (config) saveConfig({ ...config, titleModel, titlePrompt }, t('标题总结设置已保存')) }} />
      <AssistantSettingsDrawer
        open={settingsOpen}
        onOpenChange={setSettingsOpen}
        config={config}
        models={models.map(item => item.name)}
        modelAvailable={modelsReady && Boolean(currentOption)}
        modelsReady={modelsReady}
        modelWarning={modelWarning}
        onRefreshModels={refreshModelOptions}
        busy={busy}
        failed={loadingFailed}
        status={status}
        onSave={saveConfig}
        apiKeyField={<ModelApiKeyField {...keyState} value={config?.apiKeyId ?? null} disabled={!config || busy || awaiting} rememberSelection
          onChange={apiKeyId => { if (config) saveConfig({ ...config, apiKeyId }) }}
          onRefresh={() => { void keyState.refresh().catch(() => {}) }} t={t} />}
      />
    </div>
  )
}

/* ─── ： index.html assistant section ─── */

const PAGE_SELECTOR = '.page[data-page="assistant"]'

let root: ReturnType<typeof createRoot> | null = null

function mount(): void {
  if (root) return
  const section = document.querySelector<HTMLElement>(PAGE_SELECTOR)
  if (!section) return
  section.replaceChildren()
  root = createRoot(section)
  root.render(<AssistantPage />)
  // for 1 ： in use 1 when 。
  void gatewayBase().catch(() => {})
}

mountOnPage('assistant', PAGE_SELECTOR, mount)


export { DEFAULT_SYSTEM_PROMPT }
