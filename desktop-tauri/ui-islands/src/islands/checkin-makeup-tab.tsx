import * as React from 'react'
import { Badge, Button, Card, CardContent, CardFooter, CardHeader, Collapsible, CollapsibleContent, CollapsibleTrigger, Empty, Switch } from '@ui'
import { t } from '../i18n'
import { saveScheduleEntry, store } from './checkin-store'
import type { CheckinCenter, ScheduleEntry } from './checkin-types'
import {
  activityRetryEnabled, checkinRangeText, goCheckinTab, groupScheduleRows, makeupScheduleRows, scheduleName,
  scheduleOptions, schedulePayload, scheduleRows, type ScheduleOption, type ScheduleTimeGroup,
} from './checkin-schedule-model'
import { CheckinMakeupEditor, type MakeupEditorKind } from './checkin-makeup-editor'
import { CheckinScheduleHeading, CheckinScheduleMaster } from './checkin-schedule-common'

function MakeupGroup({ group, entry, options, paused }: {
  group: ScheduleTimeGroup
  entry: ScheduleEntry
  options: ScheduleOption[]
  paused: boolean
}): React.ReactElement {
  const [open, setOpen] = React.useState(false)
  const rows = scheduleRows(entry, options)
  const mainTimes = [...new Set(group.rows.map(row => rows.find(item => item.id === row.id))
    .filter(Boolean).map(row => checkinRangeText(row!.time, row!.timeEnd)))]
  return <Collapsible open={open} onOpenChange={setOpen} className='ck-schedule-group ck-makeup-group'>
    <div className='ck-schedule-group-head'><strong className='font-mono tabular-nums'>{group.time}</strong>
      <Badge variant='outline' shape='tag'>{paused ? t('Paused') : t('Catch-up check')}</Badge>
    </div>
    <div className='ck-schedule-group-meta'><span>{t('{n} platforms', { n: group.rows.length })}</span>
      <span>{t('One catch-up check daily')}</span>
    </div>
    <div className='ck-makeup-group-providers'>{group.rows.slice(0, 3).map(row => t(row.label)).join(', ')}
      {group.rows.length > 3 ? ` · +${group.rows.length - 3}` : ''}
    </div>
    <div className='ck-schedule-group-foot'><span>{t('Linked main check-in: {times}', { times: mainTimes.join(' / ') })}</span>
      <CollapsibleTrigger render={<Button variant='link' size='xs' />}>{open ? t('Hide platforms') : t('View platforms')}</CollapsibleTrigger>
    </div>
    <CollapsibleContent className='ck-schedule-group-details'>
      {group.rows.map(row => <div className='ck-schedule-group-detail' key={row.id}>
        <span>{t(row.label)}</span><small className='font-mono tabular-nums'>{row.time}</small>
      </div>)}
    </CollapsibleContent>
  </Collapsible>
}

function MakeupPlan({ entry, options, masterOn, defaultRetryTime, onEdit }: {
  entry: ScheduleEntry
  options: ScheduleOption[]
  masterOn: boolean
  defaultRetryTime: string
  onEdit(): void
}): React.ReactElement {
  const enabled = !!entry.retryTime
  const paused = !masterOn || !entry.enabled
  const groups = groupScheduleRows(makeupScheduleRows(entry, options))
  const name = scheduleName(entry)
  return <Card render={<section />} className='ck-schedule-plan'>
    <CardHeader className='ck-schedule-plan-head'>
      <div className='ck-schedule-plan-name'><h3>{name}</h3>
        <Badge variant={enabled ? 'success' : 'outline'} shape='tag'>{enabled ? t('Catch-up enabled') : t('Catch-up disabled')}</Badge>
        {paused && enabled ? <Badge variant='outline' shape='tag'>{t('Paused with the check-in schedule')}</Badge> : null}
        <p>{t('Linked schedule · Same {n} platforms, only accounts not checked in', { n: entry.providers.length })}</p>
      </div>
      <div className='ck-schedule-plan-actions'>
        <Switch checked={enabled} disabled={store.saving} aria-label={t('Scheduled catch-up for {name}', { name })}
          onCheckedChange={next => void saveScheduleEntry(schedulePayload(entry, { retryTime: next ? defaultRetryTime : '' }), { closeEditor: false, silent: true })} />
        <Button size='sm' variant='outline' disabled={store.saving} onClick={onEdit}>{t('Configure catch-up')}</Button>
      </div>
    </CardHeader>
    <CardContent className='ck-schedule-plan-content grid gap-3'>
      <div className='ck-makeup-rule'>
        <span>{t('Wait {n} minutes after the main check-in window ends', { n: Math.max(1, entry.retryGapMinutes) })}</span>
        {enabled ? <span>{t('Catch-up no earlier than {time}', { time: entry.retryTime })}</span> : null}
        <Badge variant='outline' shape='tag'>{t('Checked-in accounts are skipped')}</Badge>
      </div>
      {enabled ? <div className='ck-schedule-time-groups'>
        {groups.map(group => <MakeupGroup key={group.time} group={group} entry={entry} options={options} paused={paused} />)}
      </div> : <p className='ck-schedule-hint'>{t('Catch-up is off; main check-ins are unaffected. Enabling it generates catch-up times for each platform.')}</p>}
    </CardContent>
    <CardFooter className='ck-schedule-plan-footer'><span>{t('The gateway calculates the actual times. Later windows delay catch-up, which only handles accounts not checked in today.')}</span></CardFooter>
  </Card>
}

function ActivityRetryPlan({ entry, masterOn, onEdit }: {
  entry: ScheduleEntry
  masterOn: boolean
  onEdit(): void
}): React.ReactElement {
  const enabled = activityRetryEnabled(entry)
  const active = masterOn && entry.enabled && enabled
  // 重试关着时这两个数字都不必提：`retryAccounts` 里可能还留着旧名单，
  // 但它们不会被消费 —— 说出来只会让「重试关闭」的徽标自相矛盾。
  const count = enabled ? Number(entry.retryCount) || 0 : 0
  // 次数用完的账号已经不在 `retryCount` 里了（后端摘掉了），所以数字归零不等于
  // 「没事了」—— 这两种状态必须分开说，否则用户以为签到已经完成。
  const exhausted = enabled ? (entry.retryExhausted?.length ?? 0) : 0
  return <Card render={<section />} className='ck-activity-retry'>
    <CardHeader className='ck-schedule-plan-head'>
      <div className='ck-schedule-plan-name'><h3>{scheduleName(entry)}</h3>
        <Badge variant={active ? 'brand' : 'outline'} shape='tag'>{!enabled ? t('Retries disabled') : active ? t('Retries enabled') : t('Retries paused')}</Badge>
        <p>{enabled ? t('Every {gap} minutes · Up to {limit} retries per account per day', { gap: entry.retryGapMinutes, limit: entry.retryLimit })
          : t('Waiting campaigns are not retried automatically')}</p>
      </div>
      <Button size='sm' variant='ghost' disabled={store.saving} onClick={onEdit}>{t('Adjust retries')}</Button>
    </CardHeader>
    <CardContent className='grid gap-3'>
      {!enabled ? <p className='ck-schedule-hint'>{t('Window retry is off; accounts waiting for a window or campaign are not retried automatically.')}</p> : <>
        {count > 0 ? <div className='ck-activity-retry-notice' role='status'>
          <Badge variant='warning' shape='tag'>{t('{n} accounts waiting for a window or campaign', { n: count })}</Badge>
          <span>{t('Only these accounts are retried, not the whole schedule. See the records for details.')}</span>
          <Button size='xs' variant='link' onClick={() => goCheckinTab('records')}>{t('View check-in records')}</Button>
        </div> : null}
        {exhausted > 0 ? <div className='ck-activity-retry-notice' role='status'>
          <Badge variant='outline' shape='tag'>{t('{n} accounts have used up today’s retry limit', { n: exhausted })}</Badge>
          <span>{t('These accounts are not retried again today; the counter resets tomorrow. To try once more right now, use “Run now” in the check-in schedule.')}</span>
          <Button size='xs' variant='link' onClick={() => goCheckinTab('records')}>{t('View check-in records')}</Button>
        </div> : null}
        {count === 0 && exhausted === 0
          ? <p className='ck-schedule-hint'>{t('No accounts are waiting for a window or campaign.')}</p> : null}
      </>}
    </CardContent>
  </Card>
}

export function MakeupTab({ center }: { center: CheckinCenter }): React.ReactElement {
  const [editing, setEditing] = React.useState<{ id: string; kind: MakeupEditorKind } | null>(null)
  const auto = center.auto
  const options = scheduleOptions(auto)
  const entries = auto.schedules ?? []
  const entry = editing ? entries.find(item => item.id === editing.id) : undefined
  const close = () => setEditing(null)
  return <div className='ck-schedule-main'>
    <CheckinScheduleHeading title={t('Scheduled catch-up')} description={t('Configure catch-up separately; only checks accounts not checked in today.')}
      action={<Button size='sm' variant='outline' onClick={() => goCheckinTab('schedule')}>{t('Manage scheduled check-ins')}</Button>} />
    <CheckinScheduleMaster auto={auto} />
    <div className='ck-schedule-section-heading'><h3>{t('Catch-up schedules')}</h3>
      <span>{t('Each check-in schedule has its own catch-up settings')}</span>
    </div>
    <div className='ck-schedule-editor-note'>{t('Scheduled check-ins and catch-up are managed separately. Catch-up uses the linked schedule\'s platforms and skips completed accounts.')}</div>
    {entries.map(entry => <MakeupPlan key={entry.id} entry={entry} options={options} masterOn={auto.enabled}
      defaultRetryTime={auto.defaultRetryTime || '00:10'} onEdit={() => setEditing({ id: entry.id, kind: 'makeup' })} />)}
    {!entries.length ? <Empty className='ck-schedule-empty'>
      <p>{t('Create a check-in schedule first, then configure its scheduled catch-up.')}</p>
      <Button size='sm' variant='outline' onClick={() => goCheckinTab('schedule')}>{t('Manage scheduled check-ins')}</Button>
    </Empty> : null}
    {entries.length ? <>
      <CheckinScheduleHeading title={t('Campaign-window retries')} description={t('Unlike scheduled catch-up, only handles windows not yet open or campaigns not yet issued.')} />
      {entries.map(entry => <ActivityRetryPlan key={entry.id} entry={entry} masterOn={auto.enabled}
        onEdit={() => setEditing({ id: entry.id, kind: 'retry' })} />)}
    </> : null}
    {entry && editing ? <CheckinMakeupEditor key={`${entry.id}:${editing.kind}`} entry={entry} kind={editing.kind}
      options={options} defaultRetryTime={auto.defaultRetryTime || '00:10'} onClose={close} /> : null}
  </div>
}
