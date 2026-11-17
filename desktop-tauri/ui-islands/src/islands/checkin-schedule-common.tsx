import * as React from 'react'
import { Badge, Button, Card, CardContent, CardHeader, Switch } from '@ui'
import { t } from '../i18n'
import { store, toggleMaster } from './checkin-store'
import type { AutoState } from './checkin-types'
import { beijingScheduleTime, goCheckinTab } from './checkin-schedule-model'

export function CheckinScheduleMaster({ auto }: { auto: AutoState }): React.ReactElement {
  const enabledCount = (auto.schedules ?? []).filter(entry => entry.enabled).length
  const next = (auto.schedules ?? []).filter(entry => entry.enabled)
    .map(entry => Number(entry.nextRunAt)).filter(value => Number.isFinite(value) && value > 0)
    .sort((a, b) => a - b)[0]
  return (
    <Card render={<section />} className='ck-schedule-master'>
      <CardContent className='ck-schedule-master-content flex items-center'>
        <div className='ck-schedule-master-copy'>
          <div className='ck-schedule-title'>
            <h2>{t('Automatic run master switch')}</h2>
            <Badge variant={auto.enabled ? 'success' : 'outline'} shape='tag'>
              {auto.enabled ? t('Enabled') : t('Paused')}
            </Badge>
          </div>
          <p>{t('Controls scheduled check-ins, catch-up and campaign retries together. Cat travel runs independently.')}</p>
        </div>
        <div className='ck-schedule-next'>
          <span>{auto.enabled ? t('Next automatic run') : t('Automatic runs paused')}</span>
          <strong>{auto.enabled && next ? beijingScheduleTime(next) : '—'}</strong>
          <small>{t('Beijing time · {n} enabled check-in schedules', { n: enabledCount })}</small>
        </div>
        <Switch checked={auto.enabled} disabled={store.saving}
          aria-label={t('Automatic run master switch')}
          onCheckedChange={enabled => void toggleMaster(enabled === true)} />
      </CardContent>
      {!auto.enabled ? <div className='ck-schedule-paused'>
        {t('Check-in and catch-up schedules are paused, with settings retained. Manual runs are not affected by the master switch.')}
      </div> : null}
    </Card>
  )
}

export function CheckinScheduleHeading({ title, description, action }: {
  title: string
  description: string
  action?: React.ReactNode
}): React.ReactElement {
  return <div className='ck-schedule-heading'>
    <div><h2>{title}</h2><p>{description}</p></div>{action}
  </div>
}

export function CheckinScheduleRules(): React.ReactElement {
  return <Card render={<section />} className='ck-schedule-rules'>
    <CardHeader><h3>{t('Execution rules')}</h3></CardHeader>
    <CardContent>
      <p>{t('All times use Beijing time. Each account gets one random time per day, retained across restarts. If a window is missed, unrun accounts are caught up.')}</p>
      <p>{t('A completed main run does not mean every account claimed successfully. See the check-in records for account results.')}</p>
      <Button size='sm' variant='link' onClick={() => goCheckinTab('records')}>{t('View check-in records')}</Button>
    </CardContent>
  </Card>
}
