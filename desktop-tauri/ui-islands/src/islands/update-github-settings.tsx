import * as React from 'react'
import { Badge, Button, Input, Select, SelectContent, SelectItem, SelectTrigger, SelectValue, cn } from '@ui'
import {
  buildProxyPick, EMPTY_PROXY_SELECTION, errorMessage, fetchProxySelection, openExternal,
  saveProxy, shared, toast, type ProxySelection, type UpdateTokenStatus,
} from './update-shared'
import { GithubAccessSettingsPanel } from './update-github-access'
import { t } from '../i18n'

function tokenStatus(token: UpdateTokenStatus | null): { badge: React.ReactNode; hint: string } {
  if (!token) return { badge: <Badge shape='tag' variant='outline'>{t('Loading')}</Badge>, hint: '' }
  if (token.error) return {
    badge: <Badge shape='tag' variant='destructive' title={token.error}>{t('Cannot decrypt')}</Badge>,
    hint: t('Paste and save it again to recover'),
  }
  if (token.filled && token.origin === 'stored') return {
    badge: <Badge shape='tag' variant='success'>{t('Saved')}</Badge>,
    hint: t('Stored encrypted locally and never displayed; paste a new Token to overwrite it'),
  }
  if (token.filled) return {
    badge: <Badge shape='tag' variant='secondary'>{t('Environment variable')}</Badge>,
    hint: t('The GITHUB_TOKEN environment variable is set; a Token saved in the UI takes precedence over it'),
  }
  return {
    badge: <Badge shape='tag' variant='outline'>{t('Not set')}</Badge>,
    hint: t('Fill it in to raise the check rate limit from 60 to 5000 requests/hour (no scopes needed to create the Token)'),
  }
}

export function GithubUpdateSettings({ plugin = false, pane }: {
  plugin?: boolean
  pane: 'network' | 'token' | null
}) {
  const [selection, setSelection] = React.useState<ProxySelection>(EMPTY_PROXY_SELECTION)
  const [token, setToken] = React.useState<UpdateTokenStatus | null>(null)
  const [draft, setDraft] = React.useState('')
  const [busy, setBusy] = React.useState(false)
  const [proxyBusy, setProxyBusy] = React.useState(false)
  const proxyRevision = React.useRef(0)
  const pick = buildProxyPick(selection)

  React.useEffect(() => {
    let alive = true
    const revision = proxyRevision.current
    void fetchProxySelection().then(value => {
      if (alive && revision === proxyRevision.current) setSelection(value)
    })
    void shared().workbuddyDesktop?.getUpdateToken().then(value => {
      if (alive) setToken(value ?? null)
    }).catch(error => toast(t('Could not load settings: {error}', { error: errorMessage(error) }), 'err'))
    return () => { alive = false }
  }, [])

  async function handlePick(value: string): Promise<void> {
    if (proxyBusy) return
    proxyRevision.current += 1
    setProxyBusy(true)
    try {
      const result = await saveProxy(value)
      if (result) {
        setSelection(previous => ({ ...previous, proxyChoice: result.choice }))
        const next = await fetchProxySelection()
        setSelection(previous => ({
          ...next,
          proxyChoice: next.proxyLoaded ? next.proxyChoice : result.choice,
          pool: next.poolError ? previous.pool : next.pool,
        }))
      }
    } finally { setProxyBusy(false) }
  }

  async function saveToken(clear = false): Promise<void> {
    if (busy) return
    const value = draft.trim()
    if (!clear && !value) return
    setBusy(true)
    try {
      const result = await shared().workbuddyDesktop?.setUpdateToken({ token: clear ? null : value })
      if (!result) throw new Error(t('Request failed'))
      setToken(result)
      setDraft('')
      if (result.saved === false) toast(t('The token is active but could not be written to disk (it will be lost on restart)'), 'err')
      else toast(clear
        ? t('✅ Removed the saved GitHub token (an environment variable, if set, still applies)')
        : t('✅ GitHub token saved (stored encrypted, never shown again)'))
    } catch (error) {
      toast(t('Save failed: {error}', { error: errorMessage(error) }), 'err')
    } finally { setBusy(false) }
  }

  const status = tokenStatus(token)
  const stored = Boolean(token?.error) || token?.origin === 'stored'
  return (
    <>
      <div hidden={pane !== 'network'}>
        <div className='settings-dlg-pane-head'>
          <h3>{t('Network')}</h3>
          {plugin ? <p>{t('Shares the egress proxy and GitHub credentials with software updates. Changes in either place take effect in both.')}</p> : null}
        </div>
        <div className='settings-dlg-fields'>
          <GithubAccessSettingsPanel />
          <div className='h-px bg-hairline' />
          <div>
            <div className='text-[13px] font-semibold text-foreground'>{t('Outbound proxy')}</div>
            <div className='mt-1.5'>
              <Select value={pick.current} disabled={proxyBusy}
                onValueChange={value => void handlePick(String(value))}>
                <SelectTrigger className={cn('w-full', pick.broken && 'border-destructive-bd')}
                  title={pick.title} aria-label={t('Outbound proxy for updates')}>
                  <SelectValue className='min-w-0 truncate'>{pick.selected?.label || t('Direct')}</SelectValue>
                </SelectTrigger>
                <SelectContent>
                  {pick.items.map(item => (
                    <SelectItem key={item.value} value={item.value} disabled={item.disabled}>{item.label}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>
        </div>
      </div>
      <div hidden={pane !== 'token'}>
        <div className='settings-dlg-pane-head'>
          <h3>{t('GitHub Token')}</h3>
          {plugin ? <p>{t('Shares the egress proxy and GitHub credentials with software updates. Changes in either place take effect in both.')}</p> : null}
        </div>
        <div className='settings-dlg-fields'>
          <div className='flex flex-wrap items-center gap-2'>
            {status.badge}<span className='text-[12px] text-subtle'>{status.hint}</span>
          </div>
          <div className='flex flex-wrap items-center gap-2'>
            <Input type='password' className='min-w-0 flex-1' autoComplete='new-password' spellCheck={false}
              aria-label={t('GitHub Token')}
              placeholder={token?.filled && token.origin !== 'env'
                ? t('Filled in (paste a new token to replace it)') : t('Paste a GitHub token (ghp_… / github_pat_…)')}
              value={draft} onChange={event => setDraft(event.currentTarget.value)}
              onKeyDown={event => {
                if (event.key === 'Enter') { event.preventDefault(); void saveToken() }
              }} />
            <Button size='sm' disabled={busy || !draft.trim()} onClick={() => void saveToken()}>{t('Save')}</Button>
            {stored ? <Button variant='outline' size='sm' disabled={busy} onClick={() => void saveToken(true)}>
              {t('Clear')}
            </Button> : null}
          </div>
          <div>
            <Button variant='outline' size='sm'
              title={t('Open GitHub\'s Token creation page in your default browser (the note is prefilled, no scopes need to be selected—just click Generate token)')}
              onClick={() => void openExternal('https://github.com/settings/tokens/new?description='
                + encodeURIComponent('Agent2API Update check'))}>
              {t('Open GitHub Token page')}
            </Button>
          </div>
        </div>
      </div>
    </>
  )
}
