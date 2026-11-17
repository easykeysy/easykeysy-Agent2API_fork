/** 动态登录入口；列表只从 session 实际运行时摘要派生，不登记第二份 provider。 */
import { useEffect, useRef, useState } from 'react'
import {
  Alert, AlertDescription, AlertTitle, Badge, Button, Card, Input, Label, Select,
  SelectContent, SelectItem, SelectTrigger, SelectValue, Sheet, SheetBody, SheetContent,
  SheetHeader, SheetTitle, Spinner,
} from '@ui'
import { ProviderLogo } from '../components/ProviderLogo'
import { providerLabel } from '../lib/providers'
import { serverIdentity } from '../lib/servers'
import { store, useStore } from '../state/store'
import { fetchProxyPool, type PoolItem } from '../api/gateway'
import {
  addDynamicApiAccount, cancelDynamicLogin, dynamicLoginError, methodsUnavailable,
  readDynamicMethods, saveDynamicAccountProxy, startDynamicLogin, submitDynamicCallback, waitDynamicLogin,
} from '../api/add-account-dynamic'
import {
  loginText, mergeLoginInstructions, methodLabel, parseCallbackInput, parseMethods, pluginBrowserInstructions, promptEnabled, promptValue, readInputs,
  type CallbackInput, type MethodsState,
} from '../../../ui-islands/src/islands/add-account-dynamic-fields'
import { usePluginPrompts } from '../../../ui-islands/src/islands/add-account-plugin-prompts'
import { isAddAccountProviderHidden } from '../../../ui-shared/provider-availability'
import { ANTIGRAVITY_ACP_PROVIDER, isAntigravityAccountProvider, groupAntigravityCards, antigravityLoginProviders } from '../../../ui-shared/antigravity-login'
import { PluginLoginInteraction } from '../../../ui-shared/plugin-login-interaction'
import { SearchableSelect } from '../../../ui-shared/searchable-select'
import { readSystemProxy } from '../../../ui-shared/proxy-system'
import { apiCall } from '../api/client'
import { t } from '../i18n'

export function DynamicProviderSheet({ onClose, onAdded }: { onClose: () => void; onAdded: () => void }) {
  const { session, sessionError } = useStore()
  const providers = sessionError ? [] : (session?.accounts?.providers || [])
    .filter(item => Boolean(item.id) && !isAddAccountProviderHidden(item.id || '') && item.loginAvailable !== false)
  const [selected, setSelected] = useState('')
  const [search, setSearch] = useState('')
  const keyword = search.trim().toLowerCase()
  const selectedProvider = providers.find(item => item.id === selected)
  const cards = groupAntigravityCards(providers)
  const identity = serverIdentity()

  return <Sheet open onOpenChange={open => { if (!open) onClose() }}>
    <SheetContent>
      <SheetHeader><SheetTitle>{selectedProvider
        ? t('Sign in / add {name} account', { name: isAntigravityAccountProvider(selected) ? 'Antigravity' : providerLabel(selected, selectedProvider.label) })
        : t('Select provider')}</SheetTitle></SheetHeader>
      <SheetBody>
        {selectedProvider ? <>
          <Button size='sm' variant='outline' className='self-start' onClick={() => setSelected('')}>{t('Back to providers')}</Button>
          {isAntigravityAccountProvider(selected) ? <AntigravityMethods key={`${identity}:${selected}`}
            initialProvider={selected} providers={providers} onAdded={onAdded} />
            : <DynamicForm key={`${identity}:${selected}`} provider={selected} label={selectedProvider.label || selected}
              source={selectedProvider.source} onAdded={onAdded} />}
        </> : <>
          <p className='text-xs text-subtle'>{t('Providers appear only after being enabled and loaded. Sign-in and local references apply to the connected gateway, not this phone.')}</p>
          <Input type='search' value={search} placeholder={t('Search providers…')}
            onChange={event => setSearch(event.currentTarget.value)} />
          {sessionError ? <Alert variant='destructive'><AlertDescription>{sessionError}</AlertDescription></Alert> : null}
          <div className='grid grid-cols-1 gap-2 min-[420px]:grid-cols-2'>
            {cards.filter(item => !keyword || `${item.id} ${providerLabel(item.id, item.label)}`.toLowerCase().includes(keyword))
              .map(item => <Button key={item.id} variant='outline' className='h-auto min-h-16 justify-start whitespace-normal text-left'
                onClick={() => setSelected(item.id || '')}>
                <ProviderLogo id={item.id} label={item.label} />
                <span className='flex min-w-0 flex-col gap-1'>
                  <span>{providerLabel(item.id, item.label)}</span>
                  <span className='text-[11px] text-muted-foreground'>{t('{n} accounts', { n: item.count || 0 })}</span>
                </span>
              </Button>)}
          </div>
          {!providers.length ? <p className='text-xs text-subtle'>{t('Install and enable the corresponding provider plugin. If the list is empty, refresh and try again.')}</p> : null}
          <Button variant='outline' onClick={() => void store.refresh(true)}>{t('Reload providers')}</Button>
        </>}
      </SheetBody>
    </SheetContent>
  </Sheet>
}

function AntigravityMethods({ initialProvider, providers, onAdded }: {
  initialProvider: string
  providers: Array<{ id?: string; label?: string; source?: string; loginAvailable?: boolean }>
  onAdded: () => void
}) {
  const [selected, setSelected] = useState(initialProvider)
  const available = antigravityLoginProviders(providers)
  const provider = available.some(item => item.id === selected) ? selected : available[0]?.id || ''
  const current = available.find(item => item.id === provider)
  const label = provider === ANTIGRAVITY_ACP_PROVIDER ? current?.label || provider : t('Web sign-in / Enter credentials')
  return <>
    {available.length > 1 ? <>
      <Label>{t('Add method')}</Label>
      <Select value={provider} onValueChange={value => { if (value) setSelected(value) }}>
        <SelectTrigger className='w-full'><SelectValue>{label}</SelectValue></SelectTrigger>
        <SelectContent>{available.map(item => <SelectItem key={item.id} value={item.id!}>
          {item.id === ANTIGRAVITY_ACP_PROVIDER ? item.label || item.id : t('Web sign-in / Enter credentials')}
        </SelectItem>)}</SelectContent>
      </Select>
    </> : null}
    {current ? <DynamicForm key={provider} provider={provider} label={current.label || provider}
      source={current.source} onAdded={onAdded} hideMethodPicker={provider === ANTIGRAVITY_ACP_PROVIDER} />
      : <Alert variant='warning'><AlertDescription>
        {t('Install and enable the corresponding provider plugin. If the list is empty, refresh and try again.')}
      </AlertDescription></Alert>}
  </>
}

function DynamicForm({ provider, label, source, onAdded, hideMethodPicker = false }: {
  provider: string; label: string; source?: string; onAdded: () => void; hideMethodPicker?: boolean
}) {
  const prefix = `mobile-add-${provider}`
  const [methods, setMethods] = useState<MethodsState>({ phase: 'loading', methods: [], instructions: '', reason: '' })
  const [selected, setSelected] = useState('')
  const [values, setValues] = useState<Record<string, Record<string, string>>>({})
  const [name, setName] = useState('')
  const [proxyMode, setProxyMode] = useState('direct')
  const [readingSystemProxy, setReadingSystemProxy] = useState(false)
  const [systemProxyError, setSystemProxyError] = useState('')
  const [proxyId, setProxyId] = useState('')
  const [proxyPool, setProxyPool] = useState<PoolItem[] | null>(null)
  const [proxyPoolError, setProxyPoolError] = useState('')
  const [poolReload, setPoolReload] = useState(0)
  const [protocol, setProtocol] = useState('http')
  const [host, setHost] = useState('')
  const [port, setPort] = useState('')
  const [username, setUsername] = useState('')
  const [password, setPassword] = useState('')
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState('')
  const [progress, setProgress] = useState('')
  const [flow, setFlow] = useState<{ state: string; url: string; manual: boolean; callbackInput: CallbackInput } | null>(null)
  const [hasCallback, setHasCallback] = useState(false)
  const [callbackBusy, setCallbackBusy] = useState(false)
  const [callbackError, setCallbackError] = useState('')
  const [callbackNotice, setCallbackNotice] = useState('')
  const callbackInputRef = useRef<HTMLInputElement>(null)
  const callbackRequest = useRef(false)
  const callbackGeneration = useRef(0)
  const [canceling, setCanceling] = useState(false)
  const flowRef = useRef('')
  const loginProxy = useRef<unknown>(undefined)
  const mounted = useRef(false)
  const submitting = useRef(false)
  const identity = serverIdentity()
  const declaredMethod = methods.methods.find(item => item.id === selected) || methods.methods[0]
  const inputs = declaredMethod ? values[declaredMethod.id] || {} : {}
  const resolvedPrompts = usePluginPrompts(provider, declaredMethod, inputs, !flow && !busy,
    (path, body) => apiCall(path, { method: 'POST', body }))
  const method = resolvedPrompts.method
  const hasNamePrompt = Boolean(method?.prompts.some(prompt => prompt.key === 'name'))
  const locked = busy || Boolean(flow) || readingSystemProxy
  const submitLocked = locked || resolvedPrompts.pending || Boolean(resolvedPrompts.error)

  function clearCallback(): void {
    if (callbackInputRef.current) callbackInputRef.current.value = ''
    callbackGeneration.current += 1
    callbackRequest.current = false
    if (!mounted.current) return
    setHasCallback(false)
    setCallbackBusy(false)
    setCallbackError('')
    setCallbackNotice('')
  }

  useEffect(() => {
    mounted.current = true
    return () => {
      mounted.current = false
      clearCallback()
      if (flowRef.current && serverIdentity() === identity) void cancelDynamicLogin(flowRef.current).catch(() => {})
      flowRef.current = ''
      loginProxy.current = undefined
    }
  }, [identity])

  useEffect(() => {
    const controller = new AbortController()
    setMethods({ phase: 'loading', methods: [], instructions: '', reason: '' })
    setSelected('')
    setValues({})
    clearCallback()
    setError('')
    void readDynamicMethods(provider, controller.signal).then(result => {
      if (controller.signal.aborted) return
      const parsed = parseMethods(result, provider)
      parsed.methods = parsed.methods.filter(method => !method.desktopOnly)
      if (provider === 'deepseek-web') parsed.methods = parsed.methods.filter(method => method.type !== 'oauth')
      setMethods(parsed)
    }).catch(failure => {
      if (controller.signal.aborted) return
      const builtin = source === 'builtin' || provider.endsWith('-subscription')
      setMethods({ phase: 'error', methods: [], instructions: '', reason: methodsUnavailable(failure)
        ? builtin ? t('The native subscription returned no dynamic sign-in methods (501). Import endpoints will not be guessed and no plugin manual-credentials form will be shown. Update the gateway and retry.')
          : t('This provider declares no dynamic sign-in methods. Use its existing entry point in the gateway desktop panel; the mobile app will not guess credential fields.')
        : dynamicLoginError(failure) })
    })
    return () => controller.abort()
  }, [provider, source])

  useEffect(() => {
    if (!flow) return
    let live = true
    let timer: ReturnType<typeof setTimeout> | undefined
    const state = flow.state
    const poll = async () => {
      if (!live || !mounted.current || flowRef.current !== state || serverIdentity() !== identity) return
      try {
        const reply = await waitDynamicLogin(state)
        if (!live || !mounted.current || flowRef.current !== state || serverIdentity() !== identity) return
        if (reply.error) throw new Error(reply.error)
        if (reply.done === true) {
          if (!reply.session || !(reply.session.accountUid || reply.session.accountId)
            || reply.session.provider && reply.session.provider !== provider) {
            throw new Error(t('The sign-in bridge returned an empty session, so account addition cannot be confirmed.'))
          }
          const warning = await saveDynamicAccountProxy(
            reply.session.accountUid || reply.session.accountId!, loginProxy.current,
            reply.session.proxySaved === true, reply.session.warning || '', identity,
          )
          if (!live || !mounted.current || serverIdentity() !== identity) return
          flowRef.current = ''
          loginProxy.current = undefined
          setFlow(null)
          await finish(warning)
          return
        }
        if (reply.pending !== true) throw new Error(t('The sign-in bridge did not return a completed account or session. Try again.'))
        setProgress(mergeLoginInstructions(loginText(reply.instructions), loginText(reply.hint)))
        timer = setTimeout(() => void poll(), 1000)
      } catch (failure) {
        if (!live || !mounted.current || serverIdentity() !== identity) return
        if (flowRef.current && flowRef.current !== state) return
        flowRef.current = ''
        loginProxy.current = undefined
        clearCallback()
        setValues({})
        setPassword('')
        setFlow(null)
        setProgress('')
        setError(dynamicLoginError(failure))
        void cancelDynamicLogin(state).catch(() => {})
      }
    }
    void poll()
    return () => { live = false; if (timer) clearTimeout(timer) }
  }, [flow, identity, provider])

  useEffect(() => {
    if (proxyMode !== 'pool') return
    const controller = new AbortController()
    setProxyPoolError('')
    void fetchProxyPool(controller.signal).then(result => {
      if (!controller.signal.aborted && serverIdentity() === identity) setProxyPool(result.items || [])
    }).catch(failure => {
      if (!controller.signal.aborted && serverIdentity() === identity) {
        setProxyPool([])
        setProxyPoolError(dynamicLoginError(failure))
      }
    })
    return () => controller.abort()
  }, [proxyMode, identity, poolReload])

  async function importSystemProxy() {
    if (locked || submitting.current) return
    submitting.current = true
    setReadingSystemProxy(true)
    setSystemProxyError('')
    try {
      const proxy = readSystemProxy(await apiCall('/api/proxies/system'), t)
      if (!mounted.current || serverIdentity() !== identity) return
      setProxyMode('custom')
      setProtocol(proxy.protocol)
      setHost(proxy.host)
      setPort(String(proxy.port))
      setUsername(proxy.username)
      setPassword(proxy.password)
    } catch (failure) {
      if (mounted.current && serverIdentity() === identity) setSystemProxyError(dynamicLoginError(failure))
    } finally {
      submitting.current = false
      if (mounted.current && serverIdentity() === identity) setReadingSystemProxy(false)
    }
  }

  function proxyPayload(): unknown {
    if (proxyMode === 'direct') return null
    if (proxyMode === 'pool') {
      if (!proxyId) throw new Error(t('Select a saved proxy first'))
      const selected = proxyPool?.find(item => item.id === proxyId)
      if (!selected || selected.enabled === false || selected.resolveError) {
        throw new Error(t('The current proxy is unavailable. Choose another proxy.'))
      }
      return { source: 'pool', proxyId }
    }
    const number = Number(port)
    if (!host.trim()) throw new Error(t('Enter the proxy host address'))
    if (!Number.isInteger(number) || number < 1 || number > 65535) throw new Error(t('Proxy port must be an integer from 1 to 65535'))
    return { source: 'custom', protocol, host: host.trim(), port: number, username: username.trim(), password }
  }

  async function finish(warning = '') {
    clearCallback()
    loginProxy.current = undefined
    setValues({})
    setPassword('')
    setProgress('')
    await store.refresh(true)
    if (serverIdentity() !== identity) return
    store.toast(t('{label} account added', { label: t(label) }))
    if (warning) store.toast(warning, 'warn')
    if (store.get().sessionError) store.toast(t('Account added, but syncing the account list failed. Refresh and retry.'), 'warn')
    if (mounted.current) onAdded()
  }

  async function submit() {
    if (submitLocked || submitting.current || !method || methods.phase !== 'ready') return
    submitting.current = true
    setBusy(true)
    setError('')
    setProgress('')
    clearCallback()
    try {
      const fields = readInputs(method, inputs)
      const proxy = proxyPayload()
      const input = { provider, methodIndex: method.methodIndex!, inputs: fields, proxy }
      if (method.type === 'api') {
        if (method.localOnly) throw new Error(t('The plugin returned an invalid sign-in method. Update the plugin or contact its author.'))
        const remark = hasNamePrompt ? fields.name : name.trim()
        await finish(await addDynamicApiAccount({ ...input, ...(remark ? { name: remark } : {}) }))
        return
      }
      if (method.type !== 'oauth') throw new Error(t('Select a valid OAuth sign-in method first.'))
      loginProxy.current = proxy
      const started = await startDynamicLogin(input)
      const state = started.state || started.flow
      if (!state || typeof state !== 'string') throw new Error(t('The sign-in bridge did not return a completed account or session. Try again.'))
      if (!mounted.current || serverIdentity() !== identity) {
        if (serverIdentity() === identity) await cancelDynamicLogin(state)
        return
      }
      const rawUrl = started.authUrl ?? started.url ?? ''
      let callbackInput: CallbackInput
      try {
        callbackInput = parseCallbackInput(started.callbackInput === undefined ? method.callbackInput : started.callbackInput)
        if (started.manualCallback !== undefined && typeof started.manualCallback !== 'boolean') throw new Error()
        if (method.manualCallback && (started.manualCallback === false || callbackInput !== method.callbackInput)) throw new Error()
        if (callbackInput === 'code' && !(started.manualCallback === true || method.manualCallback)) throw new Error()
      } catch {
        await cancelDynamicLogin(state)
        throw new Error(t('The gateway did not return a valid manual callback input contract. Update the gateway and start again.'))
      }
      const manual = started.manualCallback === true || method.manualCallback
      if ((started.provider && started.provider !== provider) || typeof rawUrl !== 'string'
        || (!rawUrl && !(started.localOnly === true || method.localOnly || manual || (started.pollIntervalMs || 0) > 0))) {
        await cancelDynamicLogin(state)
        throw new Error(t('The sign-in authorization URL is invalid; this wait was canceled.'))
      }
      if (rawUrl) {
        try { const url = new URL(rawUrl); if (url.protocol !== 'https:' || url.username || url.password) throw new Error() }
        catch { await cancelDynamicLogin(state); throw new Error(t('The sign-in authorization URL is invalid; this wait was canceled.')) }
      }
      flowRef.current = state
      setProgress(loginText(started.instructions))
      setFlow({ state, url: rawUrl, manual, callbackInput })
    } catch (failure) {
      if (mounted.current && serverIdentity() === identity) setError(dynamicLoginError(failure))
    } finally {
      submitting.current = false
      if (mounted.current && serverIdentity() === identity) setBusy(false)
    }
  }

  async function cancel() {
    if (!flow || canceling) return
    const state = flow.state
    clearCallback()
    setValues({})
    setPassword('')
    setCanceling(true)
    try {
      await cancelDynamicLogin(state)
      if (!mounted.current || serverIdentity() !== identity || flowRef.current !== state) return
      flowRef.current = ''
      loginProxy.current = undefined
      setFlow(null)
      setProgress('')
      store.toast(t('Pending sign-in canceled'))
    } catch (failure) {
      if (mounted.current && serverIdentity() === identity) setError(dynamicLoginError(failure))
    } finally { if (mounted.current && serverIdentity() === identity) setCanceling(false) }
  }

  async function sendCallback() {
    if (!flow?.manual || callbackRequest.current || canceling || flowRef.current !== flow.state) return
    const raw = callbackInputRef.current?.value || ''
    if (!raw) return
    const state = flow.state
    const generation = callbackGeneration.current
    const isCurrent = () => mounted.current && serverIdentity() === identity && flowRef.current === state
      && callbackGeneration.current === generation
    callbackRequest.current = true
    setCallbackBusy(true)
    setCallbackError('')
    setCallbackNotice('')
    try {
      const next = await submitDynamicCallback(state, raw)
      if (!isCurrent()) return
      if (callbackInputRef.current) callbackInputRef.current.value = ''
      setHasCallback(false)
      if (next.retryable === true) {
        setCallbackError(loginText(next.hint) || t('Callback not accepted. Copy the complete callback URL for this sign-in again.'))
        return
      }
      if (next.nextUrl) {
        const url = new URL(next.nextUrl)
        if (url.protocol !== 'https:' || url.username || url.password || /[\u0000-\u0020\u007f]/.test(next.nextUrl)) throw new Error()
        setFlow(current => current?.state === state ? { ...current, url: next.nextUrl! } : current)
        setCallbackNotice(t('More authorization is required. Open the next authorization page, then paste the new complete callback URL.'))
        return
      }
      setCallbackNotice(t('Callback submitted. Wait for the gateway to confirm account sign-in is complete.'))
      // 回调收到不等于完成；账号添加只认 wait 的 done+session。
    } catch {
      if (isCurrent()) setCallbackError(t('Authorization callback submission failed. Check the input and retry; sign-in is still waiting.'))
    } finally {
      if (isCurrent()) {
        callbackRequest.current = false
        setCallbackBusy(false)
      }
    }
  }

  const instructions = mergeLoginInstructions(methods.instructions, method?.instructions,
    method?.browserMode === 'plugin' ? pluginBrowserInstructions(provider) : '', progress)

  return <div className='flex min-w-0 flex-col gap-3'>
    {flow?.state ? <PluginLoginInteraction key={flow.state} flow={flow.state} t={t}
      request={(method, path, body) => apiCall(path, { method, body })} /> : null}
    {methods.phase === 'loading' ? <p><Spinner /> {t('Loading plugin sign-in methods…')}</p> : null}
    {methods.reason ? <Alert variant={methods.phase === 'error' ? 'destructive' : 'warning'}>
      <AlertDescription>{methods.reason}</AlertDescription></Alert> : null}
    {method ? <>
      {!hideMethodPicker ? <Select value={method.id} disabled={locked} onValueChange={value => {
        setSelected(value || ''); setValues({}); setPassword(''); clearCallback(); setError('')
      }}>
        <SelectTrigger className='w-full max-w-full' aria-label={t('Account sign-in methods')} title={methodLabel(method)}><SelectValue>{methodLabel(method)}</SelectValue></SelectTrigger>
        <SelectContent className='w-[min(var(--anchor-width),var(--available-width))]'>
          {methods.methods.map(item => <SelectItem key={item.id} value={item.id}
            className='[&>:first-child]:whitespace-normal [&>:first-child]:break-words'>{methodLabel(item)}</SelectItem>)}
        </SelectContent>
      </Select> : null}
      {method.reason ? <Alert variant='warning'><AlertDescription>{method.reason}</AlertDescription></Alert> : null}
      {instructions ? <Alert>
        <AlertTitle>{t('Sign-in instructions / Device-code authorization')}</AlertTitle>
        <AlertDescription className='whitespace-pre-wrap break-words'>{instructions}</AlertDescription>
      </Alert> : null}
      {method.localOnly ? <Alert variant='warning'><AlertTitle>{t('Requires the gateway machine\'s local environment')}</AlertTitle>
        <AlertDescription>{t('“Local” means the machine running the gateway, not the computer displaying the panel. A remote browser can also start this, but the gateway machine may need the client installed and signed in. Availability depends on the gateway\'s response.')}</AlertDescription></Alert> : null}
      {method.prompts.map((prompt, index) => {
        if (!promptEnabled(prompt, method, inputs)) return null
        const id = `${prefix}-${index}`
        const value = promptValue(prompt, inputs)
        const change = (value: string) => setValues(previous => ({ ...previous, [method.id]: { ...previous[method.id], [prompt.key]: value } }))
        return <div key={`${method.id}:${prompt.key}`} className='flex flex-col gap-1.5'>
          <Label htmlFor={id}>{prompt.label}{prompt.optional ? t(' (Optional)') : t(' (Required)')}</Label>
          {prompt.type === 'select' ? <SearchableSelect value={value} disabled={locked} onValueChange={change}
            id={id} className='w-full' searchable={prompt.options.length > 8}
            displayValue={prompt.options.find(option => option.value === value)?.label || prompt.placeholder || t('Select an option')}
            options={[
              ...(prompt.optional ? [{ value: '', label: t('Leave blank (optional)') }] : []),
              ...prompt.options,
            ]} /> : <Input id={id} value={value} type={prompt.secret ? 'password' : 'text'} disabled={locked} spellCheck={false}
            autoComplete={prompt.secret ? 'new-password' : 'off'} placeholder={prompt.placeholder} onChange={event => change(event.currentTarget.value)} />}
        </div>
      })}
      {method.type === 'api' && !hasNamePrompt ? <Label className='flex flex-col items-start gap-1.5'>{t('Label')}
        <Input value={name} maxLength={100} disabled={locked} onChange={event => setName(event.currentTarget.value)} placeholder={t('Optional; leave blank to use the account nickname.')} />
      </Label> : method.type === 'oauth' && !hasNamePrompt ? <p className='text-xs text-subtle'>{t('The plugin fills in the account name automatically; you can change it after adding.')}</p> : null}
      <Label>{t('Account / sign-in outbound proxy')}</Label>
      <Select value={proxyMode} disabled={locked} onValueChange={value => setProxyMode(value || 'direct')}>
        <SelectTrigger className='w-full'><SelectValue>{proxyMode === 'pool' ? t('Saved proxy') : proxyMode === 'custom' ? t('Custom') : t('No proxy (direct)')}</SelectValue></SelectTrigger>
        <SelectContent><SelectItem value='direct'>{t('No proxy (direct)')}</SelectItem><SelectItem value='pool'>{t('Saved proxy')}</SelectItem><SelectItem value='custom'>{t('Custom')}</SelectItem></SelectContent>
      </Select>
      <Button size='sm' variant='outline' disabled={locked} className='self-start' onClick={() => void importSystemProxy()}>
        {readingSystemProxy ? <Spinner /> : null}{t('Fill from system proxy')}
      </Button>
      <p className='text-xs text-subtle'>{t('Copies the gateway machine\'s current HTTPS system proxy into the custom fields. Later system changes are not synced automatically.')}</p>
      {systemProxyError ? <p className='text-xs text-destructive' role='alert'>{systemProxyError}</p> : null}
      {proxyMode === 'pool' ? <div className='flex flex-col gap-2'>
        <Select value={proxyId} disabled={locked || !proxyPool?.length} onValueChange={value => setProxyId(value || '')}>
          <SelectTrigger className='w-full' aria-label={t('Proxy')}><SelectValue>{proxyPool?.find(item => item.id === proxyId)?.name
            || (proxyId ? t('The current proxy is unavailable. Choose another proxy.') : proxyPool === null ? t('Loading…') : t('Select a proxy'))}</SelectValue></SelectTrigger>
          <SelectContent>{(proxyPool || []).map(item => <SelectItem key={item.id} value={item.id}
            disabled={item.enabled === false || Boolean(item.resolveError)}>{item.name}</SelectItem>)}</SelectContent>
        </Select>
        {proxyPoolError ? <p className='text-xs text-destructive'>{proxyPoolError}</p>
          : proxyPool?.length === 0 ? <p className='text-xs text-subtle'>{t('No saved proxies yet')}</p> : null}
        <Button size='sm' variant='outline' disabled={locked} className='self-start'
          onClick={() => setPoolReload(value => value + 1)}>{t('Reload')}</Button>
      </div> : null}
      {proxyMode === 'custom' ? <Card className='flex flex-col gap-2 p-3'>
        <Select value={protocol} disabled={locked} onValueChange={value => setProtocol(value || 'http')}>
          <SelectTrigger className='w-full' aria-label={t('Protocol')}><SelectValue>{protocol === 'socks5' ? 'SOCKS5' : 'HTTP'}</SelectValue></SelectTrigger>
          <SelectContent><SelectItem value='http'>HTTP</SelectItem><SelectItem value='socks5'>SOCKS5</SelectItem></SelectContent>
        </Select>
        <Label className='flex flex-col items-start gap-1'>{t('Host')}<Input value={host} disabled={locked} onChange={event => setHost(event.currentTarget.value)} /></Label>
        <Label className='flex flex-col items-start gap-1'>{t('Port')}<Input value={port} type='number' min={1} max={65535} disabled={locked} onChange={event => setPort(event.currentTarget.value)} /></Label>
        <Label className='flex flex-col items-start gap-1'>{t('Username')}<Input value={username} disabled={locked} autoComplete='off' onChange={event => setUsername(event.currentTarget.value)} /></Label>
        <Label className='flex flex-col items-start gap-1'>{t('Password')}<Input value={password} type='password' disabled={locked} autoComplete='new-password' onChange={event => setPassword(event.currentTarget.value)} /></Label>
      </Card> : null}
      <Button disabled={submitLocked || Boolean(method.reason)} onClick={() => void submit()}>{busy || resolvedPrompts.pending ? <Spinner /> : null}{method.type === 'api' ? t('Add account') : t('Sign in and add')}</Button>
      <p className='text-xs text-subtle'>{t('The selected proxy will be saved in account settings and used for subsequent requests.')}</p>
    </> : null}
    {flow ? <Alert><AlertTitle>{t('Waiting for account sign-in to complete…')}</AlertTitle>
      <AlertDescription className='flex flex-col gap-2 whitespace-pre-wrap break-words'>
        {flow.url ? <Button variant='outline' onClick={() => window.open(flow.url, '_blank', 'noopener,noreferrer')}>{t('Open the official authorization page')}</Button> : null}
        {flow.manual ? <>
          <Badge variant='outline'>{t('Manual authorization callback required')}</Badge>
          <p>{flow.callbackInput === 'code'
            ? t('This method requires an unchanged authorization code or token. The input is hidden and is not parsed as a URL or API credential. Wait for gateway confirmation after submitting.')
            : t('This method requires the complete callback URL. Paste it unchanged, without splitting parameters or rebuilding the URL. Wait for gateway confirmation after submitting.')}</p>
          <Label htmlFor={`${prefix}-callback`}>{flow.callbackInput === 'code'
            ? t('Authorization code or token (submitted unchanged)') : t('Complete callback URL (submitted unchanged)')}</Label>
          <Input key={`${flow.state}:${flow.callbackInput}`} id={`${prefix}-callback`} ref={callbackInputRef}
            type='password' defaultValue=''
            autoComplete='new-password' spellCheck={false} autoCapitalize='none'
            placeholder={flow.callbackInput === 'code' ? t('Authorization code or token (submitted unchanged)') : t('Complete callback URL (submitted unchanged)')}
            onChange={event => { setHasCallback(event.currentTarget.value.length > 0); setCallbackError(''); setCallbackNotice('') }}
            disabled={callbackBusy || canceling} />
          <Button variant='outline' disabled={!hasCallback || callbackBusy || canceling} onClick={() => void sendCallback()}>
            {callbackBusy ? <Spinner /> : null}{t('Submit authorization callback')}</Button>
          {callbackNotice ? <p role='status'>{callbackNotice}</p> : null}
          {callbackError ? <Alert variant='destructive'><AlertDescription>{callbackError}</AlertDescription></Alert> : null}
        </> : flow.url ? <p>{t('If the official page redirects to a gateway-local address, complete the callback in the desktop panel. Mobile does not guess undeclared callback inputs.')}</p> : null}
        <Button variant='outline' disabled={canceling} onClick={() => void cancel()}>{t('Cancel sign-in')}</Button>
      </AlertDescription>
    </Alert> : null}
    {error || resolvedPrompts.error ? <Alert variant='destructive'><AlertDescription className='break-words'>{error || resolvedPrompts.error}</AlertDescription></Alert> : null}
  </div>
}
