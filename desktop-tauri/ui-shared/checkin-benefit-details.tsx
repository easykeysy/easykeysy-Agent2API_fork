import { useId, useRef, useState } from 'react'
import { Badge, Button, Input, Label } from '@ui'
import type { CampaignApi, CampaignEntry, CampaignPreview, MonthlyGift } from './checkin-campaign-types'

type Translate = (key: string, params?: Record<string, string | number>) => string

function giftStatus(status: MonthlyGift['status'], t: Translate): string {
  switch (status) {
    case 'pending': return t('Pending issue')
    case 'granted': return t('Issued')
    case 'failed': return t('Issue failed')
    case 'cancelled': return t('Canceled')
    case 'processing': return t('Issuing')
    default: return t('Unknown')
  }
}

export function CampaignBenefitDetails({ entry, data, api, t, formatTime, onChange }: {
  entry: CampaignEntry
  data: CampaignPreview
  api: CampaignApi
  t: Translate
  formatTime: (value: number) => string
  onChange?: () => void
}) {
  const inputId = useId()
  const [code, setCode] = useState('')
  const [confirming, setConfirming] = useState(false)
  const [busy, setBusy] = useState(false)
  const [message, setMessage] = useState('')
  const [error, setError] = useState('')
  const lock = useRef(false)

  const redeem = async () => {
    if (lock.current || !api.redeem || !code.trim()) return
    lock.current = true
    setBusy(true)
    setError('')
    setMessage('')
    try {
      const result = await api.redeem(entry.id, code.trim())
      if (!result?.outcome) throw new Error(t('The gateway returned no redemption result'))
      setMessage(result.outcome === 'claimed' ? t('Benefit redeemed. Query again to see the latest records.')
        : result.outcome === 'already_claimed' ? t('This benefit was already redeemed. Do not submit it again.')
          : result.outcome === 'confirming' ? t('Redemption is awaiting confirmation. Check the official client; no duplicate request will be sent.')
            : result.message || t('Benefit redemption failed'))
      onChange?.()
    } catch {
      setError(t('The redemption request was not confirmed. Check the official client and do not submit it again.'))
    } finally {
      setCode('')
      setConfirming(false)
      setBusy(false)
      lock.current = false
    }
  }

  return <div className='space-y-4'>
    {data.notice ? <p className='text-sm text-muted-foreground break-words'>{data.notice}</p> : null}
    {data.redemption?.supported && api.redeem ? <div className='space-y-2 border-t pt-3'>
      <Label htmlFor={inputId}>{t('Redemption code / order number')}</Label>
      <Input id={inputId} type='password' autoComplete='new-password' spellCheck={false} maxLength={128}
        value={code} disabled={busy} placeholder={t('Enter a code or order number you already have')}
        onChange={event => { setCode(event.currentTarget.value); setConfirming(false) }} />
      <p className='text-xs text-muted-foreground'>{t('Only redeem the benefit you enter; no plan is purchased. Code activation may require the original client’s device identity.')}</p>
      {confirming ? <div className='space-y-2'>
        <p className='text-sm'>{t('Redeem this benefit to the current account? This may not be reversible.')}</p>
        <div className='flex gap-2'>
          <Button size='sm' disabled={busy || !entry.available} onClick={() => void redeem()}>
            {busy ? t('Redeeming…') : t('Confirm redemption')}
          </Button>
          <Button size='sm' variant='outline' disabled={busy} onClick={() => setConfirming(false)}>{t('Cancel')}</Button>
        </div>
      </div> : <Button size='sm' variant='outline' disabled={!code.trim() || !entry.available}
        onClick={() => setConfirming(true)}>{t('Redeem benefit')}</Button>}
      {message ? <p role='status' className='text-sm break-words'>{message}</p> : null}
      {error ? <p role='alert' className='text-sm text-destructive'>{error}</p> : null}
    </div> : null}
    {Array.isArray(data.monthlyGifts) ? <div className='space-y-2 border-t pt-3'>
      <h4 className='text-sm font-medium'>{t('Monthly gifts')}</h4>
      <p className='text-xs text-muted-foreground'>{t('Issued automatically by the official service on schedule. These records are read-only; no duplicate claims are made.')}</p>
      {data.monthlyGifts.length === 0 ? <p className='text-sm text-muted-foreground'>{t('No monthly gift records')}</p> : null}
      {data.monthlyGifts.map(gift => <div key={gift.id} className='flex flex-wrap items-center gap-x-3 gap-y-1 text-sm'>
        <span className='min-w-0 flex-1 break-words'>{t('Order {order} · Period {period}', { order: gift.orderId, period: gift.period })}</span>
        {gift.amount !== null ? <span className='tabular-nums'>{t('{points} points', { points: gift.amount })}</span> : null}
        <Badge variant={gift.status === 'granted' ? 'success' : 'outline'} shape='tag'>{giftStatus(gift.status, t)}</Badge>
        {gift.scheduledAt ? <span className='w-full text-xs text-muted-foreground'>
          {t('Scheduled issue: {time}', { time: formatTime(gift.scheduledAt) })}
        </span> : null}
        {gift.grantedAt ? <span className='w-full text-xs text-muted-foreground'>
          {t('Issued at: {time}', { time: formatTime(gift.grantedAt) })}
        </span> : null}
      </div>)}
    </div> : null}
  </div>
}
