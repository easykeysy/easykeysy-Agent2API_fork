import { useEffect, useRef, useState } from 'react'
import { Badge, Button, Checkbox, Spinner } from '@ui'
import type { OfficeaceSubscriptionApi, OfficeaceSubscriptionState, OfficeaceSubscriptionStatus } from './checkin-officeace-types'

type Translate = (key: string, params?: Record<string, string | number>) => string
const STATES = new Set<OfficeaceSubscriptionStatus>([
  'active', 'inactive', 'expired', 'frozen', 'unknown', 'pending',
  'credentials_missing', 'credentials_expired', 'permission_denied', 'unsupported',
])

function statusLabel(status: OfficeaceSubscriptionStatus | undefined, t: Translate): string {
  switch (status) {
    case 'active': return t('Active')
    case 'inactive': return t('Not activated')
    case 'expired': return t('Plan expired')
    case 'frozen': return t('Plan frozen')
    case 'pending': return t('Activation awaiting confirmation')
    case 'credentials_missing': return t('Activation credentials missing')
    case 'credentials_expired': return t('Credentials need refreshing')
    case 'permission_denied': return t('Authorization denied')
    case 'unsupported': return t('Not supported yet')
    case 'unknown': return t('Status unconfirmed')
    default: return t('Not queried yet')
  }
}

function description(status: OfficeaceSubscriptionStatus | undefined, t: Translate): string {
  switch (status) {
    case 'active': return t('The plan is already active. No further action is needed.')
    case 'inactive': return t('No plan is active. You can activate the free trial plan.')
    case 'expired':
    case 'frozen': return t('The existing plan has expired or been frozen. Manage it in the official client; this page will not renew or change it.')
    case 'credentials_missing': return t('Plan activation credentials are missing. Sign in to OfficeAce through web login to restore authorization.')
    case 'credentials_expired': return t('The credentials for querying plans have expired. Refresh them on the Accounts page first; this query does not refresh credentials automatically.')
    case 'permission_denied': return t('The official service denied the plan query. Check account authorization; this does not necessarily mean the token has expired.')
    case 'unsupported': return t('This account does not use an official OfficeAce gateway address. The free plan cannot be activated here.')
    case 'unknown': return t('The official service returned no definite plan status. Activation is unavailable for now.')
    case 'pending': return t('Activation is still processing or awaiting confirmation. Check again later; do not submit another request.')
    default: return t('Querying only reads the plan status. Activation runs only when you click Activate.')
  }
}

export function OfficeaceSubscription({ accountId, api, t, active = true, disabled = false, onChange }: {
  accountId: string
  api: OfficeaceSubscriptionApi
  t: Translate
  active?: boolean
  disabled?: boolean
  onChange?: () => void
}) {
  const [data, setData] = useState<OfficeaceSubscriptionState | null>(null)
  const [busy, setBusy] = useState<'query' | 'activate' | ''>('')
  const [error, setError] = useState('')
  const [message, setMessage] = useState('')
  const [consent, setConsent] = useState(false)
  const lock = useRef(false)
  const pollCount = useRef(0)
  const generation = useRef(0)

  useEffect(() => {
    setData(null)
    setError('')
    setMessage('')
    setBusy('')
    setConsent(false)
    pollCount.current = 0
    lock.current = false
    const current = ++generation.current
    return () => { if (generation.current === current) generation.current += 1 }
  }, [accountId])

  const query = async () => {
    if (lock.current || disabled) return
    lock.current = true
    const current = generation.current
    setBusy('query')
    setError('')
    setMessage('')
    try {
      const result = await api.query(accountId)
      if (generation.current !== current) return
      if (!result || !STATES.has(result.status)) throw new Error(t('The gateway did not return a valid plan status'))
      setData(result)
      if (result.status === 'active') onChange?.()
    } catch (cause) {
      if (generation.current !== current) return
      setData(null)
      setError(cause instanceof Error ? cause.message : String(cause))
    } finally {
      if (generation.current === current) { lock.current = false; setBusy('') }
    }
  }

  useEffect(() => {
    if (active && !data && !error) void query()
  }, [active, accountId])

  useEffect(() => {
    if (!active || disabled || busy || error || data?.status !== 'pending' || pollCount.current >= 6) return
    const timer = window.setTimeout(() => {
      pollCount.current += 1
      void query()
    }, 5000)
    return () => window.clearTimeout(timer)
  }, [active, disabled, busy, error, data, accountId])

  const activate = async () => {
    if (lock.current || disabled || !consent || data?.status !== 'inactive' || data.canActivate !== true) return
    lock.current = true
    const current = generation.current
    setBusy('activate')
    setError('')
    setMessage('')
    pollCount.current = 0
    try {
      const result = await api.activate(accountId, consent)
      if (generation.current !== current) return
      if (!result || !['activated', 'already_active', 'skipped', 'pending'].includes(result.status)) {
        throw new Error(t('The gateway did not return a valid plan status'))
      }
      if (result.status === 'activated' || result.status === 'already_active') {
        setData({ ...result, status: 'active', canActivate: false })
        setMessage(result.message || (result.status === 'activated' ? t('The free plan has been activated and confirmed active') : t('The plan is already active. No further action is needed.')))
      } else {
        setData({ ...result, status: result.status === 'pending' ? 'pending' : 'unknown', canActivate: false })
        setMessage(result.message || (result.status === 'pending'
          ? t('Activation is still processing or awaiting confirmation. Check again later; do not submit another request.')
          : t('Activation was not performed. Query the plan status for details.')))
      }
      onChange?.()
    } catch (cause) {
      if (generation.current !== current) return
      setData({ status: 'unknown', canActivate: false })
      setError(cause instanceof Error ? cause.message : String(cause))
      try {
        const latest = await api.query(accountId)
        if (generation.current === current && latest && STATES.has(latest.status)) setData(latest)
      } catch {
        if (generation.current === current) setMessage(t('Activation was not confirmed. Check the plan status again; activation will not be retried automatically.'))
      }
    } finally {
      if (generation.current === current) { lock.current = false; setBusy('') }
    }
  }

  const status = data?.status
  const variant = status === 'active' ? 'success' : status === 'inactive' ? 'brand'
    : status === 'pending' ? 'warning' : 'outline'
  return <div className='min-w-0 space-y-2' aria-label={t('OfficeAce free trial plan')}>
    <div className='flex flex-wrap items-center gap-2'>
      <span className='text-sm font-medium text-foreground'>{t('Free trial plan')}</span>
      <Badge variant={variant} shape='tag'>{statusLabel(status, t)}</Badge>
      <div className='ml-auto flex flex-wrap gap-2'>
        <Button size='sm' variant='outline' disabled={Boolean(busy) || disabled} onClick={() => void query()}>
          {busy === 'query' ? <Spinner /> : null}{busy === 'query' ? t('Checking…') : t('Check plan')}
        </Button>
        <Button size='sm' disabled={Boolean(busy) || disabled || !consent || status !== 'inactive' || data?.canActivate !== true}
          onClick={() => void activate()}>
          {busy === 'activate' ? <Spinner /> : null}
          {busy === 'activate' ? t('Activating…') : status === 'active' ? t('Active') : t('Activate free plan')}
        </Button>
      </div>
    </div>
    <p className='text-xs text-muted-foreground break-words'>{data?.message ? t(data.message) : description(status, t)}</p>
    <p className='text-xs text-muted-foreground'>{t('Only activate the personal trial when the official price is CNY 0, with automatic renewal disabled. Rewards are claimed separately.')}</p>
    {status === 'inactive' && data?.canActivate ? <div className='space-y-1.5'>
      <label className='flex items-start gap-2 text-xs text-muted-foreground'>
        <Checkbox className='mt-0.5' checked={consent} onCheckedChange={checked => setConsent(checked === true)} disabled={Boolean(busy) || disabled} />
        <span>{t('I have read and accept the service statements below, and authorize only the zero-cost trial without automatic renewal.')}</span>
      </label>
      <div className='flex flex-wrap gap-3 pl-5 text-xs'>
        <a className='text-primary underline underline-offset-2' href='https://www.huaweicloud.com/declaration/officeace.html' target='_blank' rel='noreferrer'>{t('OfficeAce Service Statement')}</a>
        <a className='text-primary underline underline-offset-2' href='https://www.huaweicloud.com/declaration/agentarts.html' target='_blank' rel='noreferrer'>{t('AgentArts Service Statement')}</a>
      </div>
    </div> : null}
    {data?.autoRenew === false ? <Badge variant='success' shape='tag'>{t('Automatic renewal confirmed off')}</Badge> : null}
    {data?.autoRenew === true ? <p role='alert' className='text-sm text-destructive'>{t('The official service reports that automatic renewal is enabled. Check it in the official client.')}</p> : null}
    {error ? <p role='alert' className='text-sm text-destructive break-words'>{error}</p> : null}
    {message && message !== data?.message ? <p role='status' className='text-sm text-foreground break-words'>{t(message)}</p> : null}
  </div>
}
