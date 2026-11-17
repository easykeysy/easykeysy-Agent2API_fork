import { useState } from 'react'
import { Button, Input, Switch, Textarea , Card, Label } from '@ui'
import { SectionHead } from '../../components/ui'
import { runTravel, saveKeepalive, saveTravel, type CheckinCenter } from '../../api/checkin'
import { store } from '../../state/store'
import { formatStamp } from '../../lib/format'
import type { CheckinAction } from '../CheckinPage'
import { t } from '../../i18n'
import { apiCall } from '../../api/client'
import { ModelApiKeyField } from '../../../../ui-shared/model-api-key-field'
import { useModelApiKeys } from '../../../../ui-shared/use-model-api-keys'

export function CheckinRegionSettings({ center, busy, action }: {
  center: CheckinCenter
  busy: string
  action: CheckinAction
}) {
  const panels = new Set(center.adminPanels || [])
  return (
    <>
      {panels.has('daily_keepalive') && center.keepalive ? <KeepalivePanel center={center} busy={busy} action={action} /> : null}
      {panels.has('growth_tasks') && center.travel ? <TravelPanel center={center} busy={busy} action={action} /> : null}
    </>
  )
}

function KeepalivePanel({ center, busy, action }: { center: CheckinCenter; busy: string; action: CheckinAction }) {
  const [draft, setDraft] = useState<string | null>(null)
  const keyState = useModelApiKeys(() => apiCall('/api/model-access/keys'))
  const text = draft ?? center.keepalive.models.join('\n')
  const save = async () => {
    const models = [...new Set(text.split(/[\n,，、]+/).map(item => item.trim()).filter(Boolean))]
    const result = await action('keepalive', () => saveKeepalive(models), t('The model chain has been saved'))
    if (result) setDraft(null)
  }

  return (
    <div className='m-section'>
      <SectionHead title={t('Keep-alive model chain')} />
      <Card className='m-card m-card-pad'>
        <ModelApiKeyField {...keyState} value={center.keepalive.apiKeyId ?? null} disabled={Boolean(busy)}
          onRefresh={() => { void keyState.refresh().catch(() => {}) }} t={t} id='keepalive-api-key'
          onChange={apiKeyId => { void action('keepalive', () => saveKeepalive(center.keepalive.models, apiKeyId), t('Keepalive API Key saved')) }} />
        {center.keepalive.authError ? <p role='status' className='m-ck-error'>{center.keepalive.authError}</p> : null}
        <p className='m-ck-note'>{t('Try the free models in order, one per line; clear and save to restore the default chain.')}</p>
        <Textarea className='text-base' rows={4} value={text} disabled={Boolean(busy)}
          aria-label={t('Keep-alive model chain')} onChange={event => setDraft(event.currentTarget.value)} />
        <div className='m-ck-actions'>
          <Button size='sm' variant='outline' disabled={Boolean(busy)} onClick={() => setDraft(center.keepalive.defaultModels.join('\n'))}>
            {t('Fill in default chain')}
          </Button>
          <Button size='sm' disabled={Boolean(busy) || draft === null} onClick={() => void save()}>{busy === 'keepalive' ? t('Saving…') : t('Save')}</Button>
        </div>
      </Card>
    </div>
  )
}

function TravelPanel({ center, busy, action }: { center: CheckinCenter; busy: string; action: CheckinAction }) {
  const [draft, setDraft] = useState<string | null>(null)
  const travel = center.travel
  const text = draft ?? (travel.times || []).join(', ')

  const save = async () => {
    const times = text.split(/[,，\s]+/).map(item => item.trim()).filter(Boolean)
    if (!times.length || times.length > 6 || times.some(time => !/^([01]\d|2[0-3]):[0-5]\d$/.test(time))) {
      store.toast(t('Please enter 1–6 total HH:MM Times, comma-separated'), 'err')
      return
    }
    const result = await action('travel-save', () => saveTravel({ times }), t('Saved'))
    if (result) setDraft(null)
  }

  const run = async () => {
    const result = await action('travel-run', runTravel)
    if (!result) return
    const failed = Number(result.summary?.failed) || 0
    store.toast(result.outcome === 'skipped' ? result.message || t('Skipped this round')
      : t('Cat travel: {n} actions, +{credit} credits, {failed} failed', {
        n: result.summary?.actions?.length || 0, credit: Number(result.summary?.credit) || 0, failed,
      }), failed > 0 ? 'err' : 'ok')
  }

  return (
    <div className='m-section'>
      <SectionHead title={t('Cat travel schedule')} />
      <Card className='m-card m-card-pad'>
        <div className='m-ck-toolbar'>
          <div className='m-ck-grow'>
            <div className='m-ck-title'>{t('Automatic inspection')}</div>
            <div className='m-ck-note'>{t('Each trip advances one step, independent of the sign-in schedule')}</div>
          </div>
          <Switch checked={travel.enabled} disabled={Boolean(busy)} aria-label={t('Automatic inspection')}
            onCheckedChange={enabled => void action('travel-toggle', () => saveTravel({ enabled }), t('Saved'))} />
        </div>
        {travel.available === false ? <p className='m-ck-note'>{t('Automatic inspection is paused: {reason}', { reason: travel.pausedReason || '' })}</p> : null}
        <Label className='m-ck-field'>{t('Daily time (Beijing time)')}
          <Input className='text-base' value={text} placeholder='09:00, 21:00' disabled={Boolean(busy)}
            onChange={event => setDraft(event.currentTarget.value)} />
        </Label>
        <div className='m-ck-actions'>
          <Button size='sm' disabled={Boolean(busy) || draft === null} onClick={() => void save()}>{t('Save')}</Button>
          <Button size='sm' variant='outline' disabled={Boolean(busy) || travel.running || travel.available === false} onClick={() => void run()}>
            {busy === 'travel-run' || travel.running ? t('Inspecting…') : t('Inspect all now')}
          </Button>
        </div>
        {travel.firedToday?.length ? <p className='m-ck-note'>{t('Executed today: {times}', { times: travel.firedToday.join(' / ') })}</p> : null}
        {travel.lastResult?.at ? <p className='m-ck-note'>{t('Last run: {time}', { time: formatStamp(travel.lastResult.at) })}</p> : null}
        {travel.lastResult?.error ? <p className='m-ck-error'>{travel.lastResult.error}</p> : null}
        {travel.lastResult?.actions?.map((item, index) => <p className='m-ck-note' key={`${item.accountId}:${index}`}>
          {item.name || item.accountId}: {item.message || item.outcome}
        </p>)}
      </Card>
    </div>
  )
}
