import { useEffect, useState } from 'react'
import { Button , Card } from '@ui'
import { Badge, Empty, SectionHead } from '../../components/ui'
import { ProviderLogo } from '../../components/ProviderLogo'
import { claimOnboarding, getOnboarding, type CheckinCenter, type CheckinSection, type OnboardingEntry, type OnboardingResult } from '../../api/checkin'
import { claimedPlansToday } from '../../lib/checkin'
import { formatStamp } from '../../lib/format'
import type { CheckinAction } from '../CheckinPage'
import { CheckinGrowth } from './CheckinGrowth'
import { CheckinPlansSheet } from './CheckinPlansSheet'
import { store } from '../../state/store'
import { t, locale } from '../../i18n'
import { CampaignRewards } from '../../../../ui-shared/checkin-campaign-rewards'
import { OfficeaceSubscription } from '../../../../ui-shared/checkin-officeace-subscription'
import { activateOfficeaceSubscription, claimCampaignReward, getCampaignRewards, getOfficeaceSubscription, redeemCampaignBenefit } from '../../api/checkin'

const campaignApi = { query: getCampaignRewards, claim: claimCampaignReward, redeem: redeemCampaignBenefit }
const subscriptionApi = { query: getOfficeaceSubscription, activate: activateOfficeaceSubscription }

export function CheckinBenefits({ center, section, busy, action, onChange }: {
  center: CheckinCenter; section: CheckinSection; busy: string; action: CheckinAction; onChange: () => void
}) {
  const [planId, setPlanId] = useState('')
  const plan = center.extras?.plans?.find(item => item.id === planId)
  return (
    <>
      <div hidden={section !== 'onboarding'}>
        {center.extras?.onboarding?.length ? <div className='m-section'>
          <SectionHead title={t('Starter rewards')} />
          <Card className='m-card'>
            {center.extras.onboarding.map(row => <OnboardingRow key={row.id} row={row} busy={busy} action={action}
              active={section === 'onboarding'} onChange={onChange} />)}
          </Card>
        </div> : <Empty text={t('No accounts support starter rewards')} />}
      </div>
      <div hidden={section !== 'plans'}>
        {(center.extras?.plans?.length ?? 0) + (center.extras?.campaigns?.length ?? 0) > 0 ? <div className='m-section'>
          <SectionHead title={t('Event rewards')} />
          <p className='m-ck-note'>{t('Limited-time rewards and trial plans · Query and claim individually · Separate from daily check-in')}</p>
          <Card className='m-card'>
            {center.extras.campaigns?.map(row => <div className='m-card-pad m-ck-account' key={row.id}>
              <div className='m-ck-toolbar mb-3'>
                <ProviderLogo id={row.provider} />
                <div className='m-ck-title m-ck-grow'>{row.name || row.id}</div>
              </div>
              <CampaignRewards entry={row} api={campaignApi} t={t} locale={locale()} formatTime={formatStamp} onChange={onChange} />
            </div>)}
            {center.extras.plans.map(row => {
              const claimed = claimedPlansToday(row.claimPlans)
              return <div className='m-card-pad m-ck-account' key={row.id}>
                <div className='m-ck-toolbar'>
                  <ProviderLogo id={row.provider} />
                  <div className='m-ck-grow'>
                    <div className='m-ck-title'>{row.name || row.id}</div>
                    <div className='m-ck-note'>{claimed.length ? t('Claimed {n} items today', { n: claimed.length }) : t('Nothing has been claimed yet today')}</div>
                  </div>
                  <Button variant='outline' size='sm' disabled={Boolean(busy)} onClick={() => setPlanId(row.id)}>{t('Plan details')}</Button>
                </div>
              </div>
            })}
          </Card>
        </div> : <Empty text={t('No accounts support campaign rewards. Add an account and update the provider plugin.')} />}
      </div>
      <div hidden={section !== 'growth'}>
        {center.extras?.growth?.length ? <div className='m-section'>
          <SectionHead title={t('Growth tasks')} />
          <Card className='m-card'>
            {center.extras.growth.map(row => <CheckinGrowth key={row.id} row={row} busy={busy} action={action} />)}
          </Card>
        </div> : <Empty text={t('No accounts support growth tasks (WorkBuddy CN required).')} />}
      </div>
      {plan ? <CheckinPlansSheet key={plan.id} entry={plan} onClose={() => setPlanId('')} onChange={onChange} /> : null}
    </>
  )
}

function settledResult(row: OnboardingEntry): OnboardingResult | null {
  if (!row.settled) return null
  return {
    tasks: row.settled.tasks,
    earned: row.settled.earned ?? undefined,
    total: row.settled.total ?? undefined,
    unclaimed: 0,
    settled: true,
    settledAt: row.settled.at,
  }
}

function OnboardingRow({ row, busy, action, active, onChange }: {
  row: OnboardingEntry; busy: string; action: CheckinAction; active: boolean; onChange: () => void
}) {
  const [data, setData] = useState<OnboardingResult | null>(() => settledResult(row))
  const [error, setError] = useState('')
  const settled = data ? data.settled === true : Boolean(row.settled)
  const settledAt = data?.settledAt || row.settled?.at

  useEffect(() => {
    const snapshot = settledResult(row)
    setData(previous => snapshot ?? (previous?.settled ? null : previous))
  }, [row.settled])

  const query = async () => {
    setError('')
    const result = await action(`onboarding-query:${row.id}`, () => getOnboarding(row.id, true))
    if (result) setData(result)
  }

  const claim = async () => {
    const result = await action(`onboarding-claim:${row.id}`, () => claimOnboarding(row.id))
    if (!result) return
    setData(result)
    const failed = Number(result.failed) || 0
    const text = result.message || t('Claimed: {n} items, +{points} points, {failed} failed",', {
      n: Number(result.claimed) || 0, points: Number(result.claimedPoints) || 0, failed,
    })
    setError(failed > 0 ? text : '')
    store.toast(text, failed > 0 ? 'err' : 'ok')
  }

  return (
    <div className='m-card-pad m-ck-account'>
      <div className='m-ck-toolbar'>
        <ProviderLogo id={row.provider} />
        <div className='m-ck-title m-ck-grow'>{row.name || row.id}</div>
        {settled ? <Badge tone='ok'>{t('Claimed')}</Badge> : null}
      </div>
      <div className='m-ck-actions'>
        <Button variant='outline' size='sm' disabled={Boolean(busy)} onClick={() => void query()}>
          {busy === `onboarding-query:${row.id}` ? t('Checking…') : t('Query tasks')}
        </Button>
        <Button size='sm' disabled={Boolean(busy) || settled || !data || !data.tasks?.some(task => !task.done && !task.blocked)} onClick={() => void claim()}>
          {busy === `onboarding-claim:${row.id}` ? t('Claiming…') : t('Claim perks')}
        </Button>
      </div>
      {settled ? <p className='m-ck-note'>
        {t('One-time rewards have been claimed; no need to query again')}
        {settledAt ? ` · ${t('Settled at {time}', { time: formatStamp(settledAt) })}` : ''}
      </p> : null}
      {error ? <p className='m-ck-error'>{error}</p> : null}
      {data?.tasks?.map(task => <div className='m-ck-task' key={task.key}>
        <span className='m-ck-grow'>{task.title || task.key}{task.points ? ` · +${task.points}` : ''}</span>
        <Badge tone={task.done ? 'ok' : task.blocked ? 'mute' : 'warn'}>{task.done ? t('Claimed') : task.blocked ? t('Locked') : t('To claim')}</Badge>
      </div>)}
      {!data && !settled ? <p className='m-ck-note'>{t('After clicking query, view the tasks; the claim actions are executed by the gateway')}</p> : null}
      {row.provider === 'officeace' ? <div className='mt-3 border-t border-border pt-3'>
        <OfficeaceSubscription accountId={row.id} api={subscriptionApi} t={t} active={active}
          disabled={Boolean(busy)} onChange={() => { onChange(); void store.refresh(true) }} />
      </div> : null}
    </div>
  )
}
