import * as React from 'react'
import {
  Alert, Badge, Button, Card, CardContent, CardHeader, CardTitle, Input, Label,
  Select, SelectContent, SelectItem, SelectTrigger, SelectValue, Switch, Textarea,
} from '@ui'
import type { CaptchaPatch, CaptchaProvider, CaptchaSetting } from '../../../ui-shared/panel-captcha'
import { errorMessage, shared, toast, TIPS } from './settings-model'
import { t } from '../i18n'

export function CaptchaSettingsPanel({ disabled = false }: { disabled?: boolean }) {
  const [saved, setSaved] = React.useState<CaptchaSetting | null>(null)
  const [provider, setProvider] = React.useState<CaptchaProvider>('altcha')
  const [siteKey, setSiteKey] = React.useState('')
  const [secretKey, setSecretKey] = React.useState('')
  const [hostnames, setHostnames] = React.useState('')
  const [busy, setBusy] = React.useState(false)
  const [error, setError] = React.useState('')
  const pending = React.useRef(false)
  const prefix = React.useId()
  const locked = disabled || busy || !saved

  const accept = React.useCallback((value: CaptchaSetting | null | undefined) => {
    if (!value || typeof value.captchaDesktop !== 'boolean' || typeof value.captchaWeb !== 'boolean') {
      throw new Error(t('Could not read bot verification settings'))
    }
    setSaved(value)
    setProvider(value.captchaProvider || 'altcha')
    setSiteKey(value.turnstileSiteKey || '')
    setSecretKey('')
    setHostnames((value.turnstileHostnames || []).join('\n'))
  }, [])

  const load = React.useCallback(async () => {
    if (pending.current) return
    pending.current = true
    setBusy(true)
    setError('')
    try { accept(await shared().workbuddyDesktop?.getCaptchaSetting()) }
    catch (failure) { setError(errorMessage(failure)) }
    finally { pending.current = false; setBusy(false) }
  }, [accept])

  React.useEffect(() => { void load() }, [load])

  async function save(patch: CaptchaPatch, resetDraft: boolean) {
    if (pending.current || disabled) return
    pending.current = true
    setBusy(true)
    setError('')
    try {
      const result = await shared().workbuddyDesktop?.saveCaptchaSetting(patch)
      if (!result) throw new Error(t('Could not read bot verification settings'))
      if (resetDraft) accept(result)
      else setSaved(result)
      toast(t('Bot verification settings saved'))
    } catch (failure) { setError(errorMessage(failure)) }
    finally { pending.current = false; setBusy(false) }
  }

  const providerName = provider === 'turnstile' ? 'Cloudflare Turnstile' : t('ALTCHA (local verification)')
  const incomplete = provider === 'turnstile' && (!siteKey.trim() || !hostnames.trim()
    || (!secretKey.trim() && !saved?.turnstileSecretSet))

  return <Card render={<section />} className='panel'>
    <CardHeader className='flex-row items-center justify-between gap-3'>
      <CardTitle>{t('Captcha')}</CardTitle>
      <Button variant='ghost' size='sm' disabled={busy || disabled} onClick={() => void load()}>{t('Refresh')}</Button>
    </CardHeader>
    <CardContent className='panel-body flex flex-col gap-4'>
      <div className='settings-switches'>
        {(['desktop', 'web'] as const).map(side => {
          const key = side === 'desktop' ? 'captchaDesktop' : 'captchaWeb'
          const current = saved && (side === 'desktop' ? saved.localPanel : !saved.localPanel)
          return <Label key={side} className='flex items-center justify-between gap-3' htmlFor={`${prefix}-${side}`}>
            <span>{side === 'desktop' ? t('Desktop sign-in / registration verification') : t('Web sign-in / registration verification')}
              {current ? <Badge variant='outline' className='ml-2'>{t('Currently in use')}</Badge> : null}</span>
            <Switch id={`${prefix}-${side}`} checked={saved?.[key] === true} disabled={locked}
              onCheckedChange={value => void save({ [key]: value }, false)} />
          </Label>
        })}
      </div>
      <p className='m-0 text-xs leading-relaxed text-muted-foreground'>{TIPS.captchaSides}</p>
      <div className='space-y-2'>
        <Label htmlFor={`${prefix}-provider`}>{t('Verification method')}</Label>
        <Select value={provider} disabled={locked} onValueChange={value => setProvider(value as CaptchaProvider)}>
          <SelectTrigger id={`${prefix}-provider`}><SelectValue>{providerName}</SelectValue></SelectTrigger>
          <SelectContent>
            <SelectItem value='altcha'>{t('ALTCHA (local verification)')}</SelectItem>
            <SelectItem value='turnstile'>Cloudflare Turnstile</SelectItem>
          </SelectContent>
        </Select>
        <p className='m-0 text-xs leading-relaxed text-muted-foreground'>
          {provider === 'altcha' ? t('The browser performs the computation locally. No third-party keys or external verification service are required.')
            : t('Verification is provided by Cloudflare. Both the browser and gateway must be able to reach Cloudflare. Failed verification or an unavailable service will not allow access.')}
        </p>
      </div>
      {provider === 'turnstile' ? <>
        <Alert>{t('Create a Turnstile widget in Cloudflare first. Managed mode is recommended. Add the actual domain names used to access the panel.')}</Alert>
        <div className='space-y-2'>
          <Label htmlFor={`${prefix}-site`}>{t('Site Key')}</Label>
          <Input id={`${prefix}-site`} value={siteKey} disabled={locked} autoComplete='off' maxLength={512}
            onChange={event => setSiteKey(event.target.value)} />
        </div>
        <div className='space-y-2'>
          <Label htmlFor={`${prefix}-secret`}>{t('Secret Key')}</Label>
          <Input id={`${prefix}-secret`} type='password' value={secretKey} disabled={locked} autoComplete='new-password' maxLength={512}
            placeholder={saved?.turnstileSecretSet ? t('Saved; leave blank to keep unchanged') : t('Enter the secret key')}
            onChange={event => setSecretKey(event.target.value)} />
          <p className='m-0 text-xs text-muted-foreground'>{t('The secret key is stored encrypted and never returned to the browser. It is kept when switching back to ALTCHA so you can enable Turnstile again later.')}</p>
        </div>
        <div className='space-y-2'>
          <Label htmlFor={`${prefix}-hosts`}>{t('Allowed hostnames')}</Label>
          <Textarea id={`${prefix}-hosts`} value={hostnames} disabled={locked} rows={3} placeholder='panel.example.com'
            onChange={event => setHostnames(event.target.value)} />
          <p className='m-0 text-xs leading-relaxed text-muted-foreground'>{t('Enter one hostname per line, without a scheme, port or path. Match your Cloudflare configuration; the server requires an exact hostname match. The mobile app uses the connected gateway\'s hostname.')}</p>
        </div>
      </> : null}
      {error || saved?.error ? <Alert variant='destructive' className='whitespace-pre-wrap break-words'>{error || saved?.error}</Alert> : null}
      <div className='flex items-center justify-between gap-3'>
        <p className='m-0 text-xs text-muted-foreground'>{t('Only affects panel sign-in and registration, not API key requests. Changes take effect after saving and do not sign out existing sessions.')}</p>
        <Button disabled={locked || incomplete} onClick={() => void save({
          captchaProvider: provider,
          ...(provider === 'turnstile' ? {
            turnstileSiteKey: siteKey.trim(), turnstileHostnames: hostnames.split(/[\n,，]/).map(value => value.trim()).filter(Boolean),
            ...(secretKey.trim() ? { turnstileSecretKey: secretKey.trim() } : {}),
          } : {}),
        }, true)}>{busy ? t('Saving…') : t('Save')}</Button>
      </div>
    </CardContent>
  </Card>
}
