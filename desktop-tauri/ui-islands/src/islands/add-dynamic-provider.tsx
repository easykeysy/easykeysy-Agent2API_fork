/** 未在内置表单登记的运行时 provider：动态方法、纯文本说明与旧插件兼容入口。 */
import * as React from 'react'
import {
  Alert, AlertDescription, AlertTitle, Button, Input, Label, SegmentedControl,
  Select, SelectContent, SelectItem, SelectTrigger, SelectValue, Spinner, Textarea,
} from '@ui'
import { AddFoot, AddGroup } from './add-account-kit'
import { t } from '../i18n'
import { MAX_NAME_LENGTH } from './add-account-configs'
import {
  accountRequest, addedLabelOf, afterAdd, describeError, isMethodsUnavailable,
  normalizeLoginResult, postAccount, runtimeProviderOf, saveAccountProxySelection, shared, startLogin, toast,
  type LoginProgress, type LoginResult, type RuntimeProviderRecord,
  type WebLoginController, type WebLoginView,
} from './add-account-bridge'
import { draftOfProxy, readProxyDraft } from './accounts-proxy-form'
import { AccountProxySection } from './add-account-proxy'
import { UserAgentInput, useUserAgentPresets, userAgentEditable, applyUserAgentAfterCreate, createdAccountIdOf } from './add-account-user-agent'
import { openExternal } from './update-shared'
import { PluginLoginInteraction } from '../../../ui-shared/plugin-login-interaction'
import { SearchableSelect } from '../../../ui-shared/searchable-select'
import { usePluginPrompts } from './add-account-plugin-prompts'

import {
  loginText, mergeLoginInstructions, methodLabel, parseCallbackInput, parseMethods, pluginBrowserInstructions, promptEnabled, promptValue, readInputs,
  type Method, type MethodsState,
} from './add-account-dynamic-fields'

const MAX_JSON_LENGTH = 262_144
const UNSAFE_KEYS = new Set(['__proto__', 'prototype', 'constructor'])
const RESERVED_KEYS = new Set([
  'provider', 'name', 'proxy', 'methodIndex', 'inputs', 'importDesktop', 'guest',
  'authFileContent', 'authFileB64', 'authFileName', 'edition', 'mode', 'socialRestore',
  'options', 'state', 'flow', 'callbackUrl', 'signal',
])
const textOf = (value: unknown): string => typeof value === 'string' ? value : ''
const objectOf = (value: unknown): Record<string, unknown> | undefined =>
  value && typeof value === 'object' && !Array.isArray(value)
    ? value as Record<string, unknown> : undefined

/** 501 才使用已有握手 meta；不臆造方法索引，也不把 token/sms 当作 OAuth。 */
function legacyMethods(provider: RuntimeProviderRecord | undefined): MethodsState {
  const source = provider?.source || textOf(provider?.meta?.source)
  const builtin = source === 'builtin' || provider?.id?.endsWith('-subscription') === true
  if (builtin) return {
    phase: 'error', methods: [], instructions: '',
    reason: t('The native subscription returned no dynamic sign-in methods (501). Import endpoints will not be guessed and no plugin manual-credentials form will be shown. Update the gateway and retry.'),
  }
  const meta: Record<string, unknown> | undefined = provider?.meta || (provider && { ...provider })
  const declared = meta?.login_methods ?? meta?.loginMethods
  const methods: Method[] = []
  for (const [index, value] of (Array.isArray(declared) ? declared : []).entries()) {
    const raw = objectOf(value)
    if (!raw) continue
    const kind = textOf(raw.kind)
    const method: Method = {
      id: `legacy:${index}`, type: 'unsupported', prompts: [], localOnly: false,
      manualCallback: false, callbackInput: 'url', embeddedOnly: false,
      label: textOf(raw.label) || textOf(raw.button) || kind,
      instructions: textOf(raw.hint) || textOf(raw.placeholder),
    }
    if (kind === 'browser') {
      method.type = 'oauth'
      method.label ||= t('Web sign-in (legacy declaration)')
    } else if (kind === 'token') {
      // 旧 token 声明没有主字段的 key，不能猜成 token/apiKey；由 JSON 手动明确字段。
      method.type = 'manual'
      method.label ||= t('Manual credentials JSON')
      const extraFields = Array.isArray(raw.extra_fields) ? raw.extra_fields : []
      method.instructions = [method.instructions, ...extraFields.map(value => {
        const field = objectOf(value)
        return [textOf(field?.label), textOf(field?.key), textOf(field?.placeholder)].filter(Boolean).join(' · ')
      })].filter(Boolean).join('\n')
    } else if (kind === 'desktop') {
      method.type = 'desktop'
      method.localOnly = true
      method.label ||= t('Import from a client on the gateway machine')
    } else if (kind === 'guest') {
      method.type = 'guest'
      method.label ||= t('Guest mode')
    } else {
      method.label ||= t('Legacy sign-in method')
      method.reason = kind === 'sms'
        ? t('The plugin only declared SMS sign-in but did not provide a usable dynamic form. This interface will not guess SMS endpoints.')
        : t('This legacy sign-in declaration cannot be safely mapped to a dynamic form. Contact the plugin author.')
    }
    methods.push(method)
  }
  if (!methods.some(method => method.type === 'manual')) {
    methods.push({
      id: 'manual:compat', type: 'manual', label: t('Manual credentials JSON (compatibility entry)'),
      prompts: [], localOnly: false, manualCallback: false, callbackInput: 'url', embeddedOnly: false, instructions: '',
    })
  }
  return {
    phase: 'legacy', methods, instructions: textOf(meta?.instructions),
    reason: Array.isArray(declared) && declared.length
      ? t('The plugin does not implement auth.methods (501). The entries below are shown solely based on its legacy sign-in declarations.')
      : t('The plugin does not implement auth.methods (501) and has no usable legacy sign-in declarations. Only the manual credentials JSON entry remains, subject to validation by the plugin.'),
  }
}

function credentialPayload(text: string): Record<string, unknown> {
  if (!text.trim() || text.length > MAX_JSON_LENGTH) {
    throw new Error(t('Enter a non-empty credentials JSON object no larger than 256 KiB.'))
  }
  let value: unknown
  try { value = JSON.parse(text) } catch {
    // JSON.parse 的错误可能包含原文片段，不能把凭证回显进错误提示。
    throw new Error(t('The credentials JSON is not valid. Check the brackets and quotation marks.'))
  }
  const record = objectOf(value)
  if (!record || !Object.keys(record).length) throw new Error(t('The credentials JSON must be a non-empty object, not an array or a string.'))
  const inspect = (value: unknown, depth: number): void => {
    if (depth > 20) throw new Error(t('The credentials JSON is nested too deeply. Paste only the credentials object required by this plugin.'))
    if (!value || typeof value !== 'object') return
    for (const [key, child] of Object.entries(value)) {
      if (UNSAFE_KEYS.has(key) || (depth === 0 && RESERVED_KEYS.has(key))) {
        throw new Error(t('The credentials JSON contains a disallowed field: {key}', { key }))
      }
      inspect(child, depth + 1)
    }
  }
  inspect(record, 0)
  return record
}

export function DynamicProviderBlock({
  provider, label, active, version, footerHost, hideMethodPicker = false,
}: {
  provider: string
  label: string
  active: boolean
  version: number
  footerHost: HTMLElement | null
  hideMethodPicker?: boolean
}): React.ReactElement {
  const prefix = `${React.useId()}-dynamic-login`
  const [methodsState, setMethodsState] = React.useState<MethodsState>({
    phase: 'loading', methods: [], instructions: '', reason: '',
  })
  const [selected, setSelected] = React.useState('')
  const [values, setValues] = React.useState<Record<string, Record<string, string>>>({})
  const [credentialJson, setCredentialJson] = React.useState('')
  const [name, setName] = React.useState('')
  // 账号级自定义 User-Agent（可选）：只对走 POST /api/accounts 的方式生效
  // （api / manual / desktop / guest）；OAuth 那条由共享登录桥建号，建好后
  // 到「账号设置」里改 —— 这里不给一个提交了却不生效的输入框。
  const [userAgent, setUserAgent] = React.useState('')
  const userAgentRef = React.useRef(userAgent)
  userAgentRef.current = userAgent
  const uaPresets = useUserAgentPresets()
  // 会话式家由插件自己拼出站头，网关覆盖不到 —— 不显示输入框
  const uaEditable = userAgentEditable(provider)
  const [proxyDraft, setProxyDraft] = React.useState(() => draftOfProxy(undefined))
  const [readingSystemProxy, setReadingSystemProxy] = React.useState(false)
  const [mode, setMode] = React.useState<'external' | 'embedded'>('external')
  const [saving, setSaving] = React.useState(false)
  const [error, setError] = React.useState('')
  const [progress, setProgress] = React.useState<LoginProgress>({})
  const [hasCallback, setHasCallback] = React.useState(false)
  const [callbackBusy, setCallbackBusy] = React.useState(false)
  const [callbackError, setCallbackError] = React.useState('')
  const [callbackNotice, setCallbackNotice] = React.useState('')
  const [nextAuthUrl, setNextAuthUrl] = React.useState('')
  const callbackInputRef = React.useRef<HTMLInputElement>(null)
  const callbackRequest = React.useRef(false)
  const callbackGeneration = React.useRef(0)
  const progressRef = React.useRef<LoginProgress>({})
  const [loginView, setLoginView] = React.useState<WebLoginView>({
    label: t('Sign in and add'), hint: '', disabled: false, waiting: false,
  })
  const controllerRef = React.useRef<WebLoginController | null>(null)
  const alive = React.useRef(true)
  const savingRef = React.useRef(false)
  const requestingRef = React.useRef(false)
  const loadedRef = React.useRef(false)
  const declaredMethod = methodsState.methods.find(item => item.id === selected) || methodsState.methods[0]
  const inputs = (declaredMethod && values[declaredMethod.id]) || {}
  const resolvedPrompts = usePluginPrompts(provider, declaredMethod, inputs, active && !loginView.waiting,
    (path, body) => accountRequest('POST', path, body))
  const method = resolvedPrompts.method
  const methodAvailable = method && methodsState.phase !== 'loading' && method.type !== 'unsupported'
  const selectedRef = React.useRef(method?.id)
  selectedRef.current = method?.id
  const locked = saving || loginView.disabled || readingSystemProxy
  const submitLocked = locked || resolvedPrompts.pending || Boolean(resolvedPrompts.error)
  const legacyBrowser = method?.type === 'oauth' && method.methodIndex === undefined
  const snapshot = React.useRef({ method, inputs, mode, proxyDraft })
  snapshot.current = { method, inputs, mode, proxyDraft }
  const callbackType = progress.callbackInput ?? method?.callbackInput ?? 'url'
  // 网页端的粘贴框归 web shim；本岛仅补原生桌面等待态，避免重复回调输入。
  const manualWaiting = active && shared().workbuddyDesktop?.platform !== 'web'
    && loginView.waiting && method?.type === 'oauth' && !method.reason
    && progress.active === true && progress.provider === provider && Boolean(progress.state)
    && (progress.manualCallback === true || method.manualCallback)

  function clearCallback(preserveError = false): void {
    if (callbackInputRef.current) callbackInputRef.current.value = ''
    callbackGeneration.current += 1
    callbackRequest.current = false
    if (!alive.current) return
    setHasCallback(false)
    setCallbackBusy(false)
    if (!preserveError) setCallbackError('')
    setCallbackNotice('')
    setNextAuthUrl('')
  }

  React.useEffect(() => {
    clearCallback(true)
    if (!active) {
      // 关掉弹窗即作废这次读取；下次打开会重新问插件要登录方式
      //（原来的「重新读取登录方式」按钮已移除，靠这里保证每次打开都是新清单）。
      loadedRef.current = false
      setCallbackError('')
      progressRef.current = {}
      setProgress({})
      setValues({})
      setCredentialJson('')
      setProxyDraft(previous => ({ ...previous, password: '' }))
    }
  }, [active, method?.id, progress.state, progress.callbackInput, manualWaiting])

  React.useEffect(() => {
    if (!active || savingRef.current || requestingRef.current || loginView.waiting) return
    if (loadedRef.current && runtimeProviderOf(provider)) return
    let current = true
    const previousSelection = selectedRef.current
    const previousInputs = snapshot.current.inputs
    const accept = (next: MethodsState): void => {
      if (!current) return
      const retained = next.methods.find(item => item.id === previousSelection)
      const nonSecret = retained ? Object.fromEntries(retained.prompts
        .filter(prompt => !prompt.secret && Object.hasOwn(previousInputs, prompt.key))
        .map(prompt => [prompt.key, previousInputs[prompt.key]])) : {}
      setValues(retained ? { [retained.id]: nonSecret } : {})
      setCredentialJson('')
      setSelected(retained?.id || '')
      setMethodsState(next)
      loadedRef.current = next.phase !== 'error'
    }
    requestingRef.current = true
    loadedRef.current = false
    setMethodsState(previous => ({ ...previous, phase: 'loading', reason: '' }))
    void (async () => {
      try {
        if (!runtimeProviderOf(provider)) throw new Error(t('This provider is no longer in the runtime catalog. Return to the list and select again.'))
        const locale = window.wbI18n?.locale() || 'zh-CN'
        const result = await accountRequest('GET', `/api/session/login/methods?provider=${encodeURIComponent(provider)}&locale=${encodeURIComponent(locale)}`)
        const parsed = parseMethods(result, provider)
        if (shared().workbuddyDesktop?.platform === 'web') parsed.methods = parsed.methods.filter(method => !method.desktopOnly)
        accept(parsed)
      } catch (failure) {
        if (!current) return
        accept(isMethodsUnavailable(failure) ? legacyMethods(runtimeProviderOf(provider)) : {
          phase: 'error', methods: [], instructions: '', reason: describeError(failure),
        })
      } finally {
        if (current) requestingRef.current = false
      }
    })()
    return () => { current = false; requestingRef.current = false }
    // 等待状态由共享引擎掌管，不以等待态重建方法或重新发起登录。
  }, [provider, active, version])

  React.useEffect(() => {
    alive.current = true
    const engine = shared().wbWebLogin
    if (engine) {
      controllerRef.current = engine.create({
        provider, buttonId: `${prefix}-start`, cancelId: `${prefix}-cancel`, hintId: `${prefix}-hint`,
        busyText: t('Waiting for account sign-in to complete…'), onStateChange: setLoginView,
        texts: () => ({ button: t('Sign in and add'), hint: '' }),
        start: async () => {
          const current = snapshot.current
          if (!current.method || current.method.type !== 'oauth') throw new Error(t('Select a valid OAuth sign-in method first.'))
          setError('')
          clearCallback()
          progressRef.current = {}
          setProgress({})
          const inputs = readInputs(current.method, current.inputs)
          // 插件申报 embeddedOnly 时一律走内嵌窗口（不信任残留的 mode 状态）。
          const mode = current.method.embeddedOnly ? 'embedded' : current.method.browserMode === 'external' ? 'external' : current.mode
          // 旧 browser 声明没有索引，只走已存在的默认网页登录入口。
          try {
            return await startLogin(provider, undefined, mode, undefined, {
              ...(current.method.methodIndex === undefined ? {} : { methodIndex: current.method.methodIndex, inputs }),
              proxy: readProxyDraft(current.proxyDraft),
            })
          } catch (failure) {
            if (alive.current) setError(loginText(describeError(failure)))
            throw failure
          }
        },
        onSuccess: async raw => {
          const result = raw as LoginResult
          if (!result?.ok || result.canceled) throw new Error(t('The sign-in bridge has not confirmed that the account was added.'))
          // OAuth 建号也走共享登录桥，UA 同样只能建好后补写
          await applyUserAgentAfterCreate(createdAccountIdOf(result), provider, userAgentRef.current)
          await finish(result)
        },
      }) ?? null
    }
    const receiveProgress = (next: LoginProgress): void => {
      if (!alive.current) return
      if (next.active === false || next.provider && next.provider !== provider) {
        progressRef.current = {}
        setProgress({})
        clearCallback()
        setValues({})
        setCredentialJson('')
        setProxyDraft(previous => ({ ...previous, password: '' }))
        return
      }
      if (next.provider !== provider || next.active !== true) return
      const current = snapshot.current.method
      const state = textOf(next.state)
      if (!state) {
        // start 的预约态 / 旧 LoginState 查询不携带 flow，不能覆盖已收到的真实进度。
        if (!progressRef.current.state) setProgress({ active: true, provider })
        return
      }
      let callbackInput: LoginProgress['callbackInput']
      try {
        callbackInput = parseCallbackInput(next.callbackInput === undefined ? current?.callbackInput : next.callbackInput)
        if (next.manualCallback !== undefined && typeof next.manualCallback !== 'boolean') throw new Error()
        if (current?.manualCallback && (next.manualCallback === false || callbackInput !== current.callbackInput)) throw new Error()
        if (callbackInput === 'code' && !(next.manualCallback === true || current?.manualCallback)) throw new Error()
      } catch {
        progressRef.current = {}
        setProgress({})
        clearCallback()
        setCallbackError(t('The gateway did not return a valid manual callback input contract. Update the gateway and start again.'))
        return
      }
      // 只摘公开进度字段，不保存整个 payload 或凭证；flow 必须属于当前 provider。
      const safe: LoginProgress = {
        active: true, provider, state,
        manualCallback: next.manualCallback === true, callbackInput,
        instructions: loginText(next.instructions), hint: loginText(next.hint), userCode: textOf(next.userCode),
      }
      if (progressRef.current.state !== safe.state || progressRef.current.callbackInput !== safe.callbackInput) clearCallback()
      progressRef.current = safe
      setProgress(safe)
    }
    const bridge = shared().workbuddyDesktop
    const unsubscribeState = bridge?.onLoginState?.(receiveProgress)
    const unsubscribeProgress = bridge?.onLoginProgress?.(receiveProgress)
    return () => {
      alive.current = false
      clearCallback()
      progressRef.current = {}
      controllerRef.current?.dispose()
      controllerRef.current = null
      if (typeof unsubscribeState === 'function') unsubscribeState()
      if (typeof unsubscribeProgress === 'function') unsubscribeProgress()
      void shared().wbWebLogin?.cancelIfActive?.(provider)
    }
  }, [provider, prefix])

  async function finish(result: LoginResult): Promise<void> {
    clearCallback()
    progressRef.current = {}
    if (!alive.current) { await shared().wbApp?.refresh?.(); return }
    setProgress({})
    setValues({})
    setCredentialJson('')
    setUserAgent('')
    setProxyDraft(draftOfProxy(undefined))
    setError('')
    const nickname = textOf(result.session?.nickname)
    await afterAdd(addedLabelOf(result.account) || nickname, label)
    if (result.warning) toast(t('Account added, but note: {warning}', { warning: result.warning }), 'warn')
  }

  async function submit(): Promise<void> {
    if (!method || submitLocked || savingRef.current || requestingRef.current || methodsState.phase === 'loading') return
    try {
      if (!runtimeProviderOf(provider)) throw new Error(t('This provider is no longer in the runtime catalog. Return to the list and select again.'))
      if (method.reason) throw new Error(method.reason)
      const fields = readInputs(method, inputs)
      const proxy = readProxyDraft(proxyDraft)
      if (proxy && 'source' in proxy && proxy.source === 'rotate') {
        throw new Error(t('Sign-in does not support rotating proxies. Select one saved proxy.'))
      }
      const remark = method.prompts.some(prompt => prompt.key === 'name') ? fields.name : name.trim()
      const payload: Record<string, unknown> = { provider, proxy, ...(remark ? { name: remark } : {}) }
      if (userAgent.trim()) payload.userAgent = userAgent.trim()
      if (method.type === 'api') {
        payload.methodIndex = method.methodIndex
        payload.inputs = fields
      } else if (method.type === 'manual') {
        Object.assign(payload, credentialPayload(credentialJson))
      } else if (method.type === 'desktop') {
        payload.importDesktop = true
      } else if (method.type === 'guest') {
        payload.guest = true
      } else {
        throw new Error(t('This sign-in method must be started through the shared sign-in bridge.'))
      }
      savingRef.current = true
      setSaving(true)
      setError('')
      // API 方法必须进入 account.normalize，不能把 Key/Token 交给 OAuth start。
      const result = await saveAccountProxySelection(normalizeLoginResult(await postAccount(payload)), proxy)
      await finish(result)
    } catch (failure) {
      if (alive.current) setError(describeError(failure))
    } finally {
      savingRef.current = false
      if (alive.current) setSaving(false)
    }
  }

  function selectMethod(value: string): void {
    clearCallback()
    progressRef.current = {}
    setValues({})
    setCredentialJson('')
    setSelected(value)
    setError('')
    setProgress({})
  }

  function startOauth(): void {
    if (submitLocked || !method || requestingRef.current || methodsState.phase === 'loading') return
    try {
      if (!runtimeProviderOf(provider)) throw new Error(t('This provider is no longer in the runtime catalog. Return to the list and select again.'))
      readInputs(method, inputs)
      readProxyDraft(proxyDraft)
      if (!controllerRef.current) throw new Error(t('The shared sign-in bridge is unavailable. Reopen the panel.'))
      controllerRef.current.start()
    } catch (failure) { setError(describeError(failure)) }
  }

  function cancelOauth(): void {
    clearCallback()
    setValues({})
    setCredentialJson('')
    setProxyDraft(previous => ({ ...previous, password: '' }))
    controllerRef.current?.cancel()
  }

  async function sendCallback(): Promise<void> {
    const current = snapshot.current.method
    const flow = progressRef.current
    const engine = shared().wbWebLogin
    if (callbackRequest.current || !manualWaiting || !current || flow.active !== true
      || flow.provider !== provider || !flow.state || current.reason
      || !(flow.manualCallback === true || current.manualCallback)) return
    const raw = callbackInputRef.current?.value || ''
    if (!raw) return
    const generation = callbackGeneration.current
    const isCurrent = () => alive.current && callbackGeneration.current === generation
      && progressRef.current.state === flow.state && progressRef.current.provider === provider
    callbackRequest.current = true
    setCallbackBusy(true)
    setCallbackError('')
    setCallbackNotice('')
    try {
      if (!runtimeProviderOf(provider)) throw new Error(t('This provider is no longer in the runtime catalog. Return to the list and select again.'))
      if (!engine?.isActive?.() || engine.activeProvider?.() !== provider) throw new Error()
      const bridge = shared().workbuddyDesktop
      if (!bridge?.submitLoginCallback) throw new Error()
      // URL / 裸 code 只在输入框与这次受保护 POST 中存在，不 trim、解析或伪造 URL。
      const reply = objectOf(await bridge.submitLoginCallback(flow.state, raw))
      if (!isCurrent()) return
      if (reply?.success === false || reply?.ok === false || reply?.error) throw new Error()
      if (reply?.retryable === true) {
        clearCallback()
        setCallbackError(loginText(reply.hint) || t('Callback not accepted. Copy the complete callback URL for this sign-in again.'))
        return
      }
      const nextUrl = textOf(reply?.nextUrl)
      if (nextUrl) {
        const parsed = new URL(nextUrl)
        if (parsed.protocol !== 'https:' || parsed.username || parsed.password || /[\u0000-\u0020\u007f]/.test(nextUrl)) throw new Error()
        clearCallback()
        setNextAuthUrl(nextUrl)
        setCallbackNotice(t('More authorization is required. Open the next authorization page, then paste the new complete callback URL.'))
        return
      }
      clearCallback()
      setCallbackNotice(t('Callback submitted. Wait for the gateway to confirm account sign-in is complete.'))
      // accepted / Pending / accountId 都不触发 finish；终态只由现有 start/wait 链收尾。
    } catch {
      if (isCurrent()) setCallbackError(t('Authorization callback submission failed. Check the input and retry; sign-in is still waiting.'))
    } finally {
      if (isCurrent()) {
        callbackRequest.current = false
        setCallbackBusy(false)
      }
    }
  }

  const instructions = mergeLoginInstructions(methodsState.instructions, method?.instructions,
    method?.browserMode === 'plugin' ? pluginBrowserInstructions(provider) : '',
    loginText(progress.instructions), loginText(progress.hint), loginText(loginView.hint))
  const commonFields = method && method.type !== 'unsupported'

  return <div className='add-provider-block' hidden={!active}>
    <AddGroup label={t('Add method')} hidden={hideMethodPicker && methodsState.phase === 'ready' && !methodsState.reason}>
      {methodsState.phase === 'loading' ? <p role='status'><Spinner /> {t('Loading plugin sign-in methods…')}</p>
        : methodsState.reason ? <Alert variant={methodsState.phase === 'error' ? 'destructive' : 'warning'}>
          <AlertTitle>{methodsState.phase === 'error' ? t('Unable to load sign-in methods') : t('Sign-in method details')}</AlertTitle>
          <AlertDescription className='whitespace-pre-wrap break-words'>{methodsState.reason}</AlertDescription>
        </Alert> : null}
      {!hideMethodPicker && method && methodsState.phase !== 'loading' ? (
        methodsState.methods.length <= 3 ? <SegmentedControl
          aria-label={t('How to add a {label} account', { label })} disabled={locked}
          className='max-w-full flex-wrap [&_[data-slot=segmented-item]]:h-7'
          value={method.id} options={methodsState.methods.map(item => ({ value: item.id, label: methodLabel(item) }))}
          onValueChange={selectMethod}
        /> : <div className='min-w-0'>
          <Select value={method.id} disabled={locked}
            onValueChange={value => { if (value !== null) selectMethod(value) }}>
            <SelectTrigger className='w-full max-w-full' aria-label={t('Account sign-in methods')} title={methodLabel(method)}>
              <SelectValue>{methodLabel(method)}</SelectValue>
            </SelectTrigger>
            <SelectContent className='w-[min(var(--anchor-width),var(--available-width))]'>
              {methodsState.methods.map(item => <SelectItem key={item.id} value={item.id}
                className='[&>:first-child]:whitespace-normal [&>:first-child]:break-words'>
                {methodLabel(item)}
              </SelectItem>)}
            </SelectContent>
          </Select>
        </div>
      ) : null}
    </AddGroup>

    {progress.active && progress.state ? <PluginLoginInteraction key={progress.state} flow={progress.state} t={t}
      request={async <T,>(method: string, path: string, body?: unknown) => await accountRequest(method, path, body) as T} /> : null}
    {instructions || progress.userCode ? <Alert>
      <AlertTitle>{t('Sign-in instructions / Device-code authorization')}</AlertTitle>
      <AlertDescription className='whitespace-pre-wrap break-words'>
        {instructions}
        {progress.userCode ? <div className='mt-2'>{t('Device authorization code: {code}', { code: progress.userCode })}</div> : null}
      </AlertDescription>
    </Alert> : null}

    {method && methodsState.phase !== 'loading' ? <AddGroup label={methodLabel(method)}>
      {method.type === 'oauth' && !method.localOnly ? <p>{t('Enter the required information and start authorization; progress and errors appear here.')}</p> : null}
      {method.type === 'api' ? <p>{t('API methods submit to the plugin\'s account normalization entry point; the key is not used to start OAuth sign-in.')}</p> : null}
      {method.localOnly ? <Alert variant='warning'>
        <AlertTitle>{t('Requires the gateway machine\'s local environment')}</AlertTitle>
        <AlertDescription>{t('“Local” means the machine running the gateway, not the computer displaying the panel. A remote browser can also start this, but the gateway machine may need the client installed and signed in. Availability depends on the gateway\'s response.')}</AlertDescription>
      </Alert> : null}
      {method.manualCallback ? <Alert>
        <AlertDescription>{method.callbackInput === 'code'
          ? t('This method requires an unchanged authorization code or token. The input is hidden and is not parsed as a URL or API credential. Wait for gateway confirmation after submitting.')
          : t('This method requires the complete callback URL. Paste it unchanged, without splitting parameters or rebuilding the URL. Wait for gateway confirmation after submitting.')}</AlertDescription>
      </Alert> : null}
      {resolvedPrompts.error ? <Alert variant='destructive'><AlertDescription>{resolvedPrompts.error}</AlertDescription></Alert> : null}
      {method.reason ? <Alert variant='warning'><AlertDescription>{method.reason}</AlertDescription></Alert> : null}
      {method.prompts.map((prompt, index) => {
        if (!promptEnabled(prompt, method, inputs)) return null
        const id = `${prefix}-prompt-${index}`
        const current = promptValue(prompt, inputs)
        return <div key={`${method.id}:${prompt.key}`} className='flex flex-col gap-1.5'>
          <Label htmlFor={id}>{prompt.label}{prompt.optional ? t(' (Optional)') : t(' (Required)')}</Label>
          {prompt.type === 'select' ? <SearchableSelect value={current} disabled={locked || !prompt.options.length}
            onValueChange={value => setValues(previous => ({
              ...previous, [method.id]: { ...previous[method.id], [prompt.key]: value },
            }))}
            id={id} className='w-full' searchable={prompt.options.length > 8}
            displayValue={prompt.options.find(option => option.value === current)?.label || prompt.placeholder || t('Select an option')}
            options={[
              ...(prompt.optional ? [{ value: '', label: t('Leave blank (optional)') }] : []),
              ...prompt.options,
            ]} /> : <Input id={id} type={prompt.secret ? 'password' : 'text'} disabled={locked}
            autoComplete={prompt.secret ? 'new-password' : 'off'} spellCheck={false}
            placeholder={prompt.placeholder} value={current} onChange={event => {
              const value = event.currentTarget.value
              setValues(previous => ({ ...previous, [method.id]: { ...previous[method.id], [prompt.key]: value } }))
            }} />}
        </div>
      })}
      {method.type === 'manual' ? <>
        <Alert variant='warning'><AlertDescription>
          {t('This is a manual compatibility entry for legacy plugins, not support for arbitrary credentials or sign-in methods. Paste only credentials JSON explicitly accepted by the plugin. The gateway passes it to account.normalize for validation and does not infer Token, Cookie or OAuth credentials.')}
        </AlertDescription></Alert>
        <Label htmlFor={`${prefix}-credential`}>{t('Credentials JSON (required)')}</Label>
        <Textarea id={`${prefix}-credential`} rows={6} maxLength={MAX_JSON_LENGTH} disabled={locked}
          autoComplete='off' spellCheck={false} value={credentialJson}
          onChange={event => setCredentialJson(event.currentTarget.value)}
          placeholder={t('Paste the JSON object required by the plugin, without provider, name, proxy or sign-in flow control fields.')} />
        <p>{t('Credentials are sent to the gateway only on submission. Use the fields below for the label and proxy; parse errors will not display credential contents.')}</p>
      </> : null}
      {commonFields ? <>
        {method.type !== 'oauth' && !method.prompts.some(prompt => prompt.key === 'name') ? <div className='flex flex-col gap-1.5'>
          <Label htmlFor={`${prefix}-name`}>{t('Label')}</Label>
          <Input id={`${prefix}-name`} maxLength={MAX_NAME_LENGTH} placeholder={t('Optional; leave blank to use the account nickname.')}
            value={name} disabled={locked} onChange={event => setName(event.currentTarget.value)} />
        </div> : null}
        {uaEditable ? <div className='flex flex-col gap-1.5'>
          <Label htmlFor={`${prefix}-user-agent`}>{t('User-Agent (optional)')}</Label>
          <UserAgentInput id={`${prefix}-user-agent`} value={userAgent} onChange={setUserAgent}
            presets={uaPresets} disabled={locked} />
        </div> : null}
      </> : null}
      {method.type === 'oauth' && !legacyBrowser && !method.prompts.some(prompt => prompt.key === 'name')
        ? <p>{t('The plugin fills in the account name automatically; you can change it after adding.')}</p> : null}
      {method.type === 'oauth' && !method.localOnly && !method.embeddedOnly && (!method.browserMode || method.browserMode === 'choice') && shared().workbuddyDesktop?.platform !== 'web' ? <>
        <Label>{t('Open with')}</Label>
        <SegmentedControl aria-label={t('How to open web login')} value={mode} disabled={locked}
          options={[{ value: 'external', label: t('System browser') }, { value: 'embedded', label: t('Embedded window') }]}
          onValueChange={setMode} />
      </> : null}
      {method.type === 'oauth' ? <AddFoot host={footerHost} active={active}>
        <Button id={`${prefix}-start`} disabled={submitLocked || Boolean(method.reason)} onClick={startOauth}>
          {loginView.waiting ? <Spinner /> : null}{loginView.label}
        </Button>
        <Button id={`${prefix}-cancel`} variant='outline' hidden={!loginView.waiting}
          onClick={cancelOauth}>{t('Cancel sign-in')}</Button>
      </AddFoot> : method.type !== 'unsupported' ? <AddFoot host={footerHost} active={active}>
        <Button disabled={submitLocked || Boolean(method.reason)}
          onClick={() => void submit()}>{saving ? <Spinner /> : null}{saving ? t('Adding…') : t('Add account')}</Button>
      </AddFoot> : null}
      {manualWaiting ? <div className='flex flex-col gap-2'>
        {nextAuthUrl ? <Button variant='outline' className='self-start' disabled={callbackBusy}
          onClick={() => {
            if (progressRef.current.active && progressRef.current.provider === provider) void openExternal(nextAuthUrl)
          }}>{t('Open the next authorization page')}</Button> : null}
        <Label htmlFor={`${prefix}-callback`}>{callbackType === 'code'
          ? t('Authorization code or token (submitted unchanged)') : t('Complete callback URL (submitted unchanged)')}</Label>
        <Input key={`${progress.state}:${callbackType}`} id={`${prefix}-callback`} ref={callbackInputRef}
          type='password' defaultValue=''
          autoComplete='new-password' spellCheck={false} autoCapitalize='none'
          disabled={callbackBusy} placeholder={callbackType === 'code'
            ? t('Authorization code or token (submitted unchanged)') : t('Complete callback URL (submitted unchanged)')}
          onChange={event => {
            setHasCallback(event.currentTarget.value.length > 0)
            setCallbackError('')
            setCallbackNotice('')
          }} />
        <Button variant='outline' className='self-start' disabled={!hasCallback || callbackBusy}
          onClick={() => void sendCallback()}>{callbackBusy ? <Spinner /> : null}{t('Submit authorization callback')}</Button>
        {callbackNotice ? <p role='status'>{callbackNotice}</p> : null}
      </div> : null}
      {callbackError ? <Alert variant='destructive'><AlertDescription>{callbackError}</AlertDescription></Alert> : null}
      {error ? <Alert variant='destructive'><AlertTitle>{t('Failed to add')}</AlertTitle>
        <AlertDescription className='whitespace-pre-wrap break-words'>{error}</AlertDescription>
      </Alert> : null}
    </AddGroup> : null}
    {error && !methodAvailable ? <Alert variant='destructive'><AlertDescription>{error}</AlertDescription></Alert> : null}
    {methodAvailable ? <AccountProxySection draft={proxyDraft} onChange={setProxyDraft}
      idPrefix={`${prefix}-proxy`} disabled={locked} onSystemImportBusy={setReadingSystemProxy} /> : null}
  </div>
}
