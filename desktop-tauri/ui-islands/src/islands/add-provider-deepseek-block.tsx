import * as React from 'react'
import { createPortal } from 'react-dom'
import {
  Alert, AlertDescription, Button, Collapsible, CollapsibleContent, CollapsibleTrigger,
  Input, InputGroup, InputGroupAddon, InputGroupButton, InputGroupInput, Label,
  SegmentedControl, Spinner,
} from '@ui'
import { t } from '../i18n'
import { afterAdd, addedLabelOf, describeError, normalizeLoginResult, postAccount, runtimeProviderOf, shared, startLogin, toast } from './add-account-bridge'
import { draftOfProxy, ProxyForm, readProxyDraft, type ProxyDraft } from './accounts-proxy-form'
import type { WebLoginController, WebLoginView } from './add-account-bridge'

function LockIcon() {
  return <svg viewBox='0 0 24 24' aria-hidden='true'><rect x='5' y='10' width='14' height='11' rx='3' /><path d='M8 10V7a4 4 0 0 1 8 0v3' /></svg>
}

function GlobeIcon() {
  return <svg viewBox='0 0 24 24' aria-hidden='true'><circle cx='12' cy='12' r='9' /><ellipse cx='12' cy='12' rx='4' ry='9' /><path d='M3 12h18' /></svg>
}

function ChevronIcon() {
  return <svg viewBox='0 0 24 24' aria-hidden='true'><path d='m9 5 7 7-7 7' /></svg>
}

function CredentialInput({ id, label, optional, placeholder, value, onChange, disabled, maxLength, children }: {
  id: string; label: string; optional?: boolean; placeholder: string; value: string
  onChange: (value: string) => void; disabled: boolean; maxLength: number; children?: React.ReactNode
}) {
  const [visible, setVisible] = React.useState(false)
  return <div className='compact-login-field'>
    <div className='compact-login-label'><Label htmlFor={id}>{label}{optional ? <span>{t('Optional')}</span> : null}</Label>{children}</div>
    <InputGroup>
      <InputGroupInput id={id} type={visible ? 'text' : 'password'} value={value} disabled={disabled}
        maxLength={maxLength} placeholder={placeholder} autoComplete='new-password' spellCheck={false}
        onChange={event => onChange(event.currentTarget.value)} />
      <InputGroupAddon align='inline-end'><InputGroupButton size='icon-xs' disabled={disabled}
        aria-label={visible ? t('Hide credentials') : t('Show credentials')} onClick={() => setVisible(previous => !previous)}>
        <svg viewBox='0 0 24 24' aria-hidden='true'><path d='M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7S2 12 2 12Z' /><circle cx='12' cy='12' r='3' /></svg>
      </InputGroupButton></InputGroupAddon>
    </InputGroup>
  </div>
}

function proxyLabel(draft: ProxyDraft): string {
  if (draft.mode === 'none') return t('Direct')
  if (draft.mode === 'pool') return t('Saved proxy')
  if (draft.mode === 'custom') return draft.host ? `${draft.host}:${draft.port}` : t('Manual proxy')
  return t('Select a proxy')
}

export function DeepseekAccountBlock({ active, footerHost, onClose }: {
  active: boolean; footerHost: HTMLElement | null; onClose: () => void
}): React.ReactElement {
  const id = `${React.useId()}-deepseek-login`
  const desktop = shared().workbuddyDesktop?.platform !== 'web'
  const [method, setMethod] = React.useState<'web' | 'token'>(() => desktop ? 'web' : 'token')
  const [token, setToken] = React.useState('')
  const [cookie, setCookie] = React.useState('')
  const [name, setName] = React.useState('')
  const [proxy, setProxy] = React.useState(() => draftOfProxy(undefined))
  const [saving, setSaving] = React.useState(false)
  const [error, setError] = React.useState('')
  const [guideOpen, setGuideOpen] = React.useState(false)
  const [proxyOpen, setProxyOpen] = React.useState(false)
  const [view, setView] = React.useState<WebLoginView>({ label: t('Open web login'), hint: '', disabled: false, waiting: false })
  const controller = React.useRef<WebLoginController | null>(null)
  const alive = React.useRef(false)
  const savingRef = React.useRef(false)
  const snapshot = React.useRef({ name, proxy })
  snapshot.current = { name, proxy }
  const locked = saving || view.disabled
  const tokenValid = Boolean(token.trim()) && !token.trim().startsWith('sk-') && !/[\r\n]/.test(token)

  React.useEffect(() => {
    alive.current = true
    controller.current = shared().wbWebLogin?.create({
      provider: 'deepseek-web', buttonId: `${id}-start`, cancelId: `${id}-cancel`, hintId: `${id}-hint`,
      busyText: t('Waiting for sign-in…'), onStateChange: setView,
      texts: () => ({ button: t('Open web login'), hint: '' }),
      start: async () => {
        const current = snapshot.current
        if (!runtimeProviderOf('deepseek-web')) throw new Error(t('This provider is no longer in the runtime catalog. Return to the list and select again.'))
        try {
          return await startLogin('deepseek-web', '', 'embedded', false, {
            methodIndex: 1, inputs: current.name.trim() ? { name: current.name.trim() } : {}, proxy: readProxyDraft(current.proxy),
          })
        } catch (failure) {
          if (alive.current) setError(describeError(failure))
          throw failure
        }
      },
      onSuccess: async raw => {
        const result = normalizeLoginResult(raw)
        if (!result.ok || result.canceled || !(result.account?.id || result.session?.accountUid || result.session?.accountId)) {
          throw new Error(t('The sign-in bridge did not return a completed account or session. Try again.'))
        }
        await afterAdd(addedLabelOf(result.account), t('DeepSeek Web'))
        if (result.warning) toast(result.warning, 'warn')
      },
    }) || null
    return () => {
      alive.current = false
      controller.current?.dispose()
      controller.current = null
      void shared().wbWebLogin?.cancelIfActive?.('deepseek-web')
    }
  }, [id])

  React.useEffect(() => {
    if (active) return
    setToken('')
    setCookie('')
    setError('')
    setProxy(previous => ({ ...previous, password: '' }))
    void shared().wbWebLogin?.cancelIfActive?.('deepseek-web')
  }, [active])

  async function addToken(): Promise<void> {
    if (!tokenValid || locked || savingRef.current) return
    try {
      if (!runtimeProviderOf('deepseek-web')) throw new Error(t('This provider is no longer in the runtime catalog. Return to the list and select again.'))
      const selectedProxy = readProxyDraft(proxy)
      savingRef.current = true
      setSaving(true)
      setError('')
      const result = normalizeLoginResult(await postAccount({
        provider: 'deepseek-web', accessToken: token.trim(), cookie: cookie.trim(), name: name.trim() || undefined, proxy: selectedProxy,
      }))
      if (!result.account?.id) throw new Error(t('The sign-in bridge did not return a completed account or session. Try again.'))
      if (alive.current) { setToken(''); setCookie('') }
      await afterAdd(addedLabelOf(result.account), t('DeepSeek Web'))
      if (result.warning) toast(result.warning, 'warn')
    } catch (failure) {
      if (alive.current) setError(describeError(failure))
    } finally {
      savingRef.current = false
      if (alive.current) setSaving(false)
    }
  }

  function start(): void {
    if (locked) return
    setError('')
    try {
      readProxyDraft(proxy)
      if (!controller.current) throw new Error(t('The shared sign-in bridge is unavailable. Reopen the panel.'))
      controller.current.start()
    } catch (failure) { setError(describeError(failure)) }
  }

  function changeMethod(next: 'web' | 'token'): void {
    setMethod(next)
    setToken('')
    setCookie('')
    setError('')
  }

  const footer = <>
    <span className='compact-login-security'><LockIcon />{t('Credentials sent only to this gateway')}</span>
    <div className='compact-login-actions'>
      <Button id={`${id}-cancel`} variant='outline' disabled={saving} onClick={() => {
        if (view.waiting) controller.current?.cancel()
        else onClose()
      }}>{view.waiting ? t('Cancel sign-in') : t('Cancel')}</Button>
      {method === 'web' ? <Button id={`${id}-start`} disabled={locked} onClick={start}>
        {view.waiting ? <Spinner /> : <GlobeIcon />}{view.label}
      </Button> : <Button disabled={locked || !tokenValid} onClick={() => void addToken()}>
        {saving ? <Spinner /> : null}{saving ? t('Adding…') : t('Add account')}
      </Button>}
    </div>
  </>

  return <div className='compact-account-login' hidden={!active}>
    {desktop ? <SegmentedControl className='compact-login-methods' value={method} disabled={locked}
      aria-label={t('Add method')} options={[
        { value: 'web', label: <>{t('Web login')}<span className='compact-login-recommend'>{t('Recommended')}</span></> },
        { value: 'token', label: t('Paste token') },
      ]} onValueChange={changeMethod} /> : null}
    {method === 'web' ? <div className='compact-login-description'>
      {view.waiting ? <p role='status'>{t('Waiting for you to sign in on the official page. Your account will be added automatically on success.')}</p>
        : <p>{t('Sign in on the official DeepSeek website to add your account automatically.')}<br /><span>{t('No token copying needed. Complete verification on the official page.')}</span></p>}
    </div> : <>
      <CredentialInput id={`${id}-token`} label={t('Web access token')} placeholder={t('Paste userToken.value')}
        value={token} onChange={setToken} disabled={locked} maxLength={16384}>
        <Button variant='link' size='xs' onClick={() => setGuideOpen(previous => !previous)} aria-expanded={guideOpen}>
          {t('How to get it')}
        </Button>
      </CredentialInput>
      <p className='compact-login-hint'>{t('Use a web session token, not a paid API key.')}</p>
      {guideOpen ? <p className='compact-login-guide'>{t('Sign in at chat.deepseek.com, open Developer Tools → Application → Local Storage, and copy the value field of userToken.')}</p> : null}
      {token.trim().startsWith('sk-') ? <p className='compact-login-error'>{t('Use a web access token, not an API key starting with sk-.')}</p> : null}
      <CredentialInput id={`${id}-cookie`} label='Cookie' optional placeholder={t('Paste the complete Cookie if needed')}
        value={cookie} onChange={setCookie} disabled={locked} maxLength={32768} />
      {!desktop ? <p className='compact-login-hint'>{t('The browser panel cannot read sign-in sessions from other sites. Use the token from your own web account.')}</p> : null}
    </>}
    <div className='compact-login-field'>
      <Label htmlFor={`${id}-name`}>{t('Label')}<span>{t('Optional')}</span></Label>
      <Input id={`${id}-name`} value={name} disabled={locked} maxLength={100}
        placeholder={t('Leave blank to use the default account name')} onChange={event => setName(event.currentTarget.value)} />
    </div>
    <Collapsible open={proxyOpen} onOpenChange={setProxyOpen} className='compact-login-proxy'>
      <CollapsibleTrigger className='compact-login-proxy-trigger' disabled={locked}>
        <span>{t('Outbound proxy')}</span><span className='compact-login-proxy-summary'>{proxyLabel(proxy)}</span><ChevronIcon />
      </CollapsibleTrigger>
      <CollapsibleContent><div className='compact-login-proxy-body' inert={locked}>
        <ProxyForm draft={proxy} onChange={setProxy} idPrefix={`${id}-proxy`} />
      </div></CollapsibleContent>
    </Collapsible>
    {view.hint && !view.waiting ? <p id={`${id}-hint`} className='compact-login-hint'>{view.hint}</p> : null}
    {error ? <Alert variant='destructive'><AlertDescription>{error}</AlertDescription></Alert> : null}
    {footerHost && active ? createPortal(footer, footerHost) : null}
  </div>
}
