import { useCallback, useEffect, useRef, useState } from 'react'
import { Button, Sheet, SheetBody, SheetContent, SheetFooter, SheetHeader, SheetTitle } from '@ui'
import { Badge } from '../../components/ui'
import { fetchAccountUsage, fetchUsageSnapshot, type UsageRow } from '../../api/gateway'
import { claimPlan, previewPlans, type CaptchaPhase, type ClaimResult, type Entitlement, type OwnedPlan, type PreviewResult } from '../../api/zcode'
import { planActivityEnded, planClaimResultApplies, planClaimState, planQueryErrors, unconfirmedPlanIds } from '../../../../ui-shared/zcode-plans'
import { useDisplayClock } from '../../../../ui-shared/use-display-clock'
import type { PlanEntry } from '../../api/checkin'
import { claimedPlansToday } from '../../lib/checkin'
import { formatCompact, formatStamp } from '../../lib/format'
import { serverIdentity } from '../../lib/servers'
import { useStore, store } from '../../state/store'
import { t } from '../../i18n'
import { ZcodeCaptchaPanel } from './ZcodeCaptchaPanel'

function stamp(seconds?: number | null): string {
  return seconds ? formatStamp(seconds * 1000) : ''
}

function entitlementText(item: Entitlement): string {
  const period = item.period === 'daily' ? t('Daily') : item.period === 'weekly' ? t('Weekly') : item.period === 'monthly' ? t('Monthly') : ''
  return [item.showName, item.grantUnits ? `${formatCompact(item.grantUnits)} ${item.unitType || ''}` : '', period].filter(Boolean).join(' · ')
}

function stateView(state?: string): { label: string; tone: 'ok' | 'warn' | 'mute' } {
  if (state === 'active') return { label: t('Active'), tone: 'ok' }
  if (state === 'pending') return { label: t('Pending'), tone: 'warn' }
  return { label: state === 'expired' ? t('Expired') : t('Unknown'), tone: 'mute' }
}

export function CheckinPlansSheet({ entry, onClose, onChange }: {
  entry: PlanEntry
  onClose: () => void
  onChange: () => void
}) {
  const { session } = useStore()
  const now = useDisplayClock()
  const [preview, setPreview] = useState<PreviewResult | null>(null)
  const [usage, setUsage] = useState<UsageRow | null>(null)
  const [previewError, setPreviewError] = useState('')
  const [receiptError, setReceiptError] = useState('')
  const [usageError, setUsageError] = useState('')
  const [loading, setLoading] = useState(true)
  const [querying, setQuerying] = useState(false)
  const [claiming, setClaiming] = useState('')
  const [claimPhase, setClaimPhase] = useState<CaptchaPhase | ''>('')
  const [verifyingChannel, setVerifyingChannel] = useState(false)
  const [claimedNow, setClaimedNow] = useState<string[]>([])
  const [claimResults, setClaimResults] = useState<Record<string, ClaimResult>>({})
  const previewRequest = useRef(0)
  const mounted = useRef(true)
  const claimController = useRef<AbortController | null>(null)
  const identity = serverIdentity()
  const account = session?.accounts?.accounts?.find(item => item.id === entry.id)
  const stored = account?.claimPlans as Record<string, unknown> | undefined
  const claimed = [...new Set([...claimedPlansToday(stored || entry.claimPlans), ...claimedNow])]
  const plans = preview?.plans || []
  const activityPlans = plans.filter(plan => !planActivityEnded(plan, now))
  const queryErrors = preview ? planQueryErrors(preview) : []
  const owned = Array.isArray(usage?.usage?.plans) ? [...usage.usage.plans as OwnedPlan[]] : []
  for (const plan of plans) {
    const item = plan.ownedPlan
    if (item && !owned.some(plan => item.userPlanId
      ? plan.userPlanId === item.userPlanId : plan.planId === item.planId)) owned.push(item)
  }
  const unconfirmed = unconfirmedPlanIds(claimed, plans, owned, now)

  const loadPreview = useCallback(async () => {
    if (!mounted.current || serverIdentity() !== identity) return
    const request = ++previewRequest.current
    setLoading(true)
    try {
      const next = await previewPlans(entry.id)
      if (mounted.current && serverIdentity() === identity && request === previewRequest.current) {
        setPreview(next); setPreviewError('')
        onChange()
      }
    } catch (cause) {
      if (mounted.current && serverIdentity() === identity && request === previewRequest.current) {
        setPreviewError(cause instanceof Error ? cause.message : String(cause))
      }
    } finally {
      if (mounted.current && serverIdentity() === identity && request === previewRequest.current) setLoading(false)
    }
  }, [entry.id, identity, onChange])

  const queryUsage = useCallback(async () => {
    if (!mounted.current || serverIdentity() !== identity) return
    setQuerying(true)
    try {
      const next = await fetchAccountUsage(entry.id)
      if (!mounted.current || serverIdentity() !== identity) return
      const row = next.results?.find(item => item.id === entry.id)
      setUsage(row || null)
      setUsageError(row?.error || (row ? '' : t('No balance data returned')))
    } catch (cause) {
      if (mounted.current && serverIdentity() === identity) setUsageError(cause instanceof Error ? cause.message : String(cause))
    } finally { if (mounted.current && serverIdentity() === identity) setQuerying(false) }
  }, [entry.id, identity])

  useEffect(() => {
    mounted.current = true
    const controller = new AbortController()
    void loadPreview()
    void fetchUsageSnapshot(controller.signal).then(snapshot => {
      if (controller.signal.aborted || !mounted.current || serverIdentity() !== identity) return
      const row = snapshot.results?.find(item => item.id === entry.id)
      if (row) { setUsage(row); setUsageError(row.error || '') }
      else void queryUsage()
    }).catch(() => {
      if (!controller.signal.aborted && mounted.current && serverIdentity() === identity) void queryUsage()
    })
    return () => {
      mounted.current = false
      controller.abort()
      claimController.current?.abort()
    }
  }, [entry.id, identity, loadPreview, queryUsage])

  const claim = async (id: string) => {
    if (claimController.current || verifyingChannel || !mounted.current || serverIdentity() !== identity) return
    const plan = activityPlans.find(item => item.planId === id)
    if (!plan || planActivityEnded(plan)) return
    const controller = new AbortController()
    claimController.current = controller
    setClaiming(id)
    setClaimPhase('preparing')
    try {
      const result = await claimPlan(entry.id, id, controller.signal, phase => { if (mounted.current) setClaimPhase(phase) })
      if (controller.signal.aborted || !mounted.current || serverIdentity() !== identity) return
      setClaimResults(results => ({ ...results, [result.planId || id]: result }))
      if (result.ok || result.failure === 'already_claimed') {
        setReceiptError(result.marketingError || '')
        const settled = result.planId || id
        setClaimResults(results => ({ ...results, [settled]: {
          ...result, claimedAt: result.claimedAt || Date.now(),
        } }))
        setClaimedNow(list => [...new Set([...list, settled])])
        store.toast(result.ok ? t('Claim succeeded: {planId}', { planId: settled }) : result.failureLabel || t('claimed today'))
        void store.refresh(true)
        onChange()
        void queryUsage()
        void loadPreview()
      } else {
        store.toast([result.failureLabel || t('Claim unsuccessful'), result.message].filter(Boolean).join(': '), 'err')
      }
    } catch (cause) {
      if (!controller.signal.aborted && mounted.current && serverIdentity() === identity && (cause as Error)?.name !== 'CaptchaCancelledError') {
        store.toast(cause instanceof Error ? cause.message : String(cause), 'err')
      }
    } finally {
      if (claimController.current === controller) claimController.current = null
      if (mounted.current && serverIdentity() === identity) { setClaiming(''); setClaimPhase('') }
    }
  }

  const cancelClaim = () => {
    if (claimPhase !== 'submitting') claimController.current?.abort()
  }

  const close = () => {
    if (claimPhase === 'submitting') return
    cancelClaim()
    onClose()
  }

  return (
    <Sheet open modal={!claiming && !verifyingChannel} onOpenChange={(open, details) => {
      if (open) return
      if (claimPhase === 'submitting' || ((claiming || verifyingChannel) && ['outside-press', 'focus-out'].includes(details.reason))) {
        details.cancel()
        return
      }
      close()
    }}>
      <SheetContent showCloseButton={claimPhase !== 'submitting'}>
        <SheetHeader><SheetTitle>{t('Plan details — {name}', { name: entry.name || entry.id })}</SheetTitle></SheetHeader>
        <SheetBody>
          <ZcodeCaptchaPanel accountId={entry.id} disabled={Boolean(claiming)} onBusy={setVerifyingChannel} />
          <div className='m-ck-toolbar'>
            <div className='m-ck-title m-ck-grow'>{t('Current events')}</div>
            <Button variant='outline' size='sm' disabled={loading || Boolean(claiming) || verifyingChannel} onClick={() => void loadPreview()}>{t('Refresh')}</Button>
          </div>
          {previewError ? <p className='m-ck-error'>{previewError}</p> : null}
          {loading ? <p className='m-ck-note'>{t('Checking current events…')}</p>
            : !previewError && !activityPlans.length ? <p className='m-ck-note'>
              {queryErrors.length
                ? t('Some sources could not be queried, so current events or claim status cannot be confirmed yet.')
                : t('No current events were found. This does not mean the account is ineligible.')}
            </p> : null}
          {activityPlans.map(plan => {
            const id = plan.planId || ''
            const savedResult = claimResults[id]
            const result = planClaimResultApplies(plan, savedResult) ? savedResult : undefined
            const locallySettled = result?.ok || result?.failure === 'already_claimed'
            const state = planClaimState(plan, locallySettled ? [id] : plan.claimState ? [] : claimed)
            const taken = state === 'claimed'
            const rejected = !taken && (result?.failure === 'ineligible' || state === 'ineligible')
            const statusLabel = taken ? t('Claimed') : rejected ? t('Currently ineligible')
              : state === 'unknown' ? t('Status unconfirmed') : t('Claimable')
            const accountPlan = plan.ownedPlan
            const period = [stamp(plan.startsAt), stamp(plan.endsAt)].filter(Boolean).join(' → ')
            const expiresAt = accountPlan?.endsAt || plan.claimEndsAt || result?.endsAt
            const message = result && !result.ok && result.failure !== 'already_claimed'
              ? result.message || result.failureLabel : plan.claimMessage
            return <div className='m-ck-account' key={id || plan.name}>
              <div className='m-ck-toolbar'>
                <div className='m-ck-title m-ck-grow'>{plan.name || id}</div>
                <Badge tone={taken ? 'ok' : rejected ? 'warn' : 'mute'}>{statusLabel}</Badge>
                {accountPlan?.state ? <Badge tone={stateView(accountPlan.state).tone}>{stateView(accountPlan.state).label}</Badge> : null}
              </div>
              {accountPlan?.pendingUntil ? <p className='m-ck-note'>{t('Effective from {time}', { time: stamp(accountPlan.pendingUntil) })}</p> : null}
              {period ? <p className='m-ck-note'>{t('Event period: {time}', { time: period })}</p> : null}
              {expiresAt ? <p className='m-ck-note'>{t('Account plan expires: {time}', { time: stamp(expiresAt) })}</p> : null}
              {message ? <p className='m-ck-note'>{message}</p> : null}
              {plan.entitlements?.map((item, index) => <p key={index} className='m-ck-note'>{entitlementText(item)}</p>)}
              {plan.description ? <p className='m-ck-note'>{plan.description}</p> : null}
              <div className='m-ck-actions'><Button size='sm' disabled={Boolean(claiming) || verifyingChannel || taken || !id} onClick={() => void claim(id)}>
                {claiming === id ? t('Claiming…') : taken ? t('Claimed')
                  : state === 'unknown' || rejected ? t('Try claiming') : t('Claim')}
              </Button></div>
            </div>
          })}
          {queryErrors.length > 0 ? <p className='m-ck-note'>
            {t('Some sources could not be queried; other results were retained: {message}', { message: queryErrors.join('; ') })}
          </p> : null}
          {receiptError ? <p className='m-ck-note'>
            {t('The plan has been claimed, but the marketing receipt is incomplete; it will not be claimed again: {message}', { message: receiptError })}
          </p> : null}
          {preview?.activationError ? <p className='m-ck-note'>{preview.activationError}</p> : null}
          <p className='m-ck-note'>{t('The event catalog combines account previews, marketing events, and anonymous discovery. Claimed status requires a claim record or an account-owned plan. If the status is unconfirmed, you can try claiming; eligibility is determined by the upstream response.')}</p>
          <div className='m-ck-toolbar'>
            <div className='m-ck-title m-ck-grow'>{t('My plans')}</div>
            <Button variant='outline' size='sm' disabled={querying || Boolean(claiming)} onClick={() => void queryUsage()}>
              {querying ? t('Checking…') : t('Query balance')}
            </Button>
          </div>
          {usageError ? <p className='m-ck-error'>{usageError}</p> : null}
          {owned.map(plan => {
            const state = stateView(plan.state)
            return <div className='m-ck-account' key={plan.userPlanId || plan.planId || plan.name}>
              <div className='m-ck-toolbar'><span className='m-ck-title m-ck-grow'>{plan.name || plan.planId}</span><Badge tone={state.tone}>{state.label}</Badge></div>
              {plan.pendingUntil ? <p className='m-ck-note'>{t('Effective from {time}', { time: stamp(plan.pendingUntil) })}</p> : null}
              {plan.endsAt ? <p className='m-ck-note'>{t('Expires {time}', { time: stamp(plan.endsAt) })}</p> : null}
              {plan.entitlements?.map((item, index) => <p className='m-ck-note' key={index}>{entitlementText(item)}</p>)}
            </div>
          })}
          {!owned.length ? <p className='m-ck-note'>{querying ? t('Checking…') : t('The current balance reading has no plan list')}</p> : null}
          {unconfirmed.length ? <p className='m-ck-note'>{t('Claimed today but the reading has not synced yet: {plans}', { plans: unconfirmed.join(', ') })}</p> : null}
        </SheetBody>
        <SheetFooter>
          {claiming ? <Button variant='outline' disabled={claimPhase === 'submitting'} onClick={cancelClaim}>{t('Cancel verification')}</Button> : null}
          <Button variant='outline' disabled={claimPhase === 'submitting'} onClick={close}>{t('Close')}</Button>
        </SheetFooter>
      </SheetContent>
    </Sheet>
  )
}
