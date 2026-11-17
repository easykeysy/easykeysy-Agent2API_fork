import * as React from 'react'
import {
  Badge, Button, Card, CardContent, CardFooter, CardHeader, Collapsible, CollapsibleContent,
  CollapsibleTrigger, Empty, Switch, confirmDialog,
} from '@ui'
import { t } from '../i18n'
import { patch, removeScheduleEntry, runScheduleEntry, saveScheduleEntry, store } from './checkin-store'
import type { CheckinCenter, ScheduleEntry } from './checkin-types'
import { ProviderLogo } from './checkin-row'
import { CheckinScheduleEditor } from './checkin-schedule-editor'
import { CheckinScheduleExtras } from './checkin-schedule-extras'
import { CheckinScheduleHeading, CheckinScheduleMaster, CheckinScheduleRules } from './checkin-schedule-common'
import {
  beijingScheduleTime, checkinEnd, checkinRangeText, goCheckinTab, groupScheduleRows, scheduleGroupLabel,
  scheduleName, scheduleOptions, schedulePayload, scheduleRows, type ScheduleOption, type ScheduleTimeGroup,
} from './checkin-schedule-model'

function ScheduleGroup({ group, entry, masterOn, options }: {
  group: ScheduleTimeGroup
  entry: ScheduleEntry
  masterOn: boolean
  options: ScheduleOption[]
}): React.ReactElement {
  const [open, setOpen] = React.useState(false)
  const fired = group.rows.filter(row => row.firedToday).length
  const overdue = group.rows.some(row => !row.firedToday && row.overdue)
  const paused = !masterOn || !entry.enabled
  const completed = fired === group.rows.length
  const accounts = group.rows.reduce((sum, row) => sum + (row.accountCount || 0), 0)
  const attempted = group.rows.reduce((sum, row) => sum + (row.attemptedCount || 0), 0)
  const partial = attempted > 0 && !completed
  const nextAccountAt = group.rows.filter(row => !row.firedToday).map(row => Number(row.nextAccountAt))
    .filter(at => Number.isFinite(at) && at > 0).sort((a, b) => a - b)[0]
  const status = accounts === 0 ? t('No eligible accounts') : completed ? t('Main check-in ran') : paused ? t('Paused')
    : partial ? t('Checking in across the window') : overdue ? t('Due, waiting to run') : t('Waiting for the check-in window')
  return <Collapsible open={open} onOpenChange={setOpen} className='ck-schedule-group'>
    <div className='ck-schedule-group-head'>
      <strong className='font-mono tabular-nums'>{checkinRangeText(group.time, group.timeEnd)}</strong>
      <Badge variant={completed && accounts > 0 ? 'success' : overdue && !paused ? 'brand' : 'outline'} shape='tag'>{status}</Badge>
    </div>
    <div className='ck-schedule-group-meta'><span>{scheduleGroupLabel(group, entry)}</span>
      <span>{t('{n} platforms', { n: group.rows.length })}</span>
    </div>
    <div className='ck-schedule-group-logos'>
      {group.rows.slice(0, 6).map(row => <ProviderLogo key={row.id} id={row.id} label={t(row.label)} />)}
      {group.rows.length > 6 ? <span>+{group.rows.length - 6}</span> : null}
    </div>
    {nextAccountAt && !paused ? <p className='ck-schedule-hint'>{t('Next account: {time}', { time: beijingScheduleTime(nextAccountAt) })}</p> : null}
    <div className='ck-schedule-group-foot'>
      <span>{accounts > 0 ? t('{n} / {total} accounts processed', { n: attempted, total: accounts })
        : t('{n} / {total} platforms ran', { n: fired, total: group.rows.length })}</span>
      <CollapsibleTrigger render={<Button variant='link' size='xs' />}>
        {open ? t('Hide platforms') : t('View platforms')}
      </CollapsibleTrigger>
    </div>
    <CollapsibleContent className='ck-schedule-group-details'>
      {group.rows.map(row => {
        const available = options.find(option => option.id === row.id)?.available
        return <div className='ck-schedule-group-detail' key={row.id}>
          <span>{t(row.label)}</span><small>{!available ? t('Plugin unavailable; scope retained') : row.accountCount === 0
            ? t('No eligible accounts') : row.firedToday ? t('Ran today') : paused ? t('Paused') : row.overdue ? t('Waiting to run') : t('Waiting for the check-in window')}
            {Number(row.accountCount) > 0 ? <span className='ck-schedule-detail-progress'>
              {t('{n} / {total} accounts processed', { n: row.attemptedCount || 0, total: row.accountCount || 0 })}
            </span> : null}
          </small>
        </div>
      })}
    </CollapsibleContent>
  </Collapsible>
}

function SchedulePlan({ entry, masterOn, options }: {
  entry: ScheduleEntry
  masterOn: boolean
  options: ScheduleOption[]
}): React.ReactElement {
  const rows = scheduleRows(entry, options)
  const groups = groupScheduleRows(rows)
  const fired = rows.filter(row => row.firedToday).length
  const accounts = rows.reduce((sum, row) => sum + (row.accountCount || 0), 0)
  const attempted = rows.reduce((sum, row) => sum + (row.attemptedCount || 0), 0)
  const name = scheduleName(entry)
  const result = entry.lastResult
  const running = store.runningScheduleId === entry.id
  const remove = async () => {
    if (!await confirmDialog(t('Delete “{name}”? Its catch-up and campaign retries will also be removed. Check-in records are kept.', { name }))) return
    void removeScheduleEntry(entry.id)
  }
  const run = async () => {
    if (!await confirmDialog(t('Run “{name}” now for the entire scope, without waiting for platform times? Closed windows may still prevent claims.', { name }), { destructive: false })) return
    void runScheduleEntry(entry.id)
  }
  return <Card render={<section />} className='ck-schedule-plan'>
    <CardHeader className='ck-schedule-plan-head'>
      <div className='ck-schedule-plan-name'><h3>{name}</h3>
        <Badge variant={entry.enabled ? 'brand' : 'outline'} shape='tag'>{entry.enabled ? t('Runs daily') : t('Disabled')}</Badge>
        <p>{t('Default {time} · {n} platforms · {groups} time groups', { time: checkinRangeText(entry.time, checkinEnd(entry.time, entry.timeEnd)), n: rows.length, groups: groups.length })}</p>
      </div>
      <div className='ck-schedule-plan-actions'>
        <Switch checked={entry.enabled} disabled={store.saving} aria-label={t('Enable check-in schedule {name}', { name })}
          onCheckedChange={enabled => void saveScheduleEntry(schedulePayload(entry, { enabled: enabled === true }), { closeEditor: false, silent: true })} />
        <Button size='sm' variant='outline' disabled={store.runningScheduleId !== null || store.center?.auto.running} onClick={() => void run()}>
          {running ? t('Running…') : t('Run now')}
        </Button>
        <Button size='sm' variant='ghost' disabled={store.saving} onClick={() => patch({ editingId: entry.id })}>{t('Edit')}</Button>
        <Button size='sm' variant='ghost' className='text-destructive' disabled={store.saving} onClick={() => void remove()}>{t('Delete')}</Button>
      </div>
    </CardHeader>
    <CardContent className='ck-schedule-plan-content grid gap-3'>
      <div className='ck-schedule-today'><strong>{t('Today\'s main check-in')}</strong>
        <span>{accounts > 0 ? t('{n} / {total} accounts processed', { n: attempted, total: accounts })
          : t('{n} / {total} platforms ran', { n: fired, total: rows.length })}</span>
        {rows.some(row => !row.firedToday && row.overdue) && masterOn && entry.enabled
          ? <Badge variant='warning' shape='tag'>{t('Some platforms are due and waiting')}</Badge> : null}
      </div>
      <div className='ck-schedule-time-groups'>
        {groups.map(group => <ScheduleGroup key={group.time} group={group} entry={entry} masterOn={masterOn} options={options} />)}
      </div>
      <div className='ck-schedule-plan-note'><span>{t('Only check-in times are managed here. Catch-up and campaign retries have a separate page.')}</span>
        <Button size='xs' variant='link' onClick={() => goCheckinTab('makeup')}>{t('Configure scheduled catch-up')}</Button>
      </div>
    </CardContent>
    <CardFooter className='ck-schedule-plan-footer'>
      <span>{result?.at ? t('Last run: {time}', { time: beijingScheduleTime(result.at) }) : t('Not run yet')}</span>
      {result ? <span>{t('Latest result: {n} successful · {active} kept active · {failed} failed', {
        n: Number(result.succeeded) || 0, active: Number(result.active) || 0, failed: Number(result.failedCount) || 0,
      })}</span> : null}
      <Button size='xs' variant='link' onClick={() => goCheckinTab('records')}>{t('View check-in records')}</Button>
    </CardFooter>
  </Card>
}

export function ScheduleTab({ center }: { center: CheckinCenter }): React.ReactElement {
  const auto = center.auto
  const options = scheduleOptions(auto)
  const entries = auto.schedules ?? []
  return <div className='ck-schedule-main'>
    <CheckinScheduleHeading title={t('Scheduled check-in')} description={t('Set the daily check-in window. Accounts run at random staggered times within it.')}
      action={<Button size='sm' disabled={store.saving || entries.length >= 10 || !options.some(option => option.available)}
        onClick={() => patch({ editingId: 'new' })}>{t('New check-in schedule')}</Button>} />
    <CheckinScheduleMaster auto={auto} />
    <div className='ck-schedule-section-heading'><h3>{t('Check-in schedules')}</h3><span>{t('{n} schedules · Beijing time UTC+8', { n: entries.length })}</span></div>
    {entries.map(entry => <SchedulePlan key={entry.id} entry={entry} options={options} masterOn={auto.enabled} />)}
    {!entries.length ? <Empty className='ck-schedule-empty'>
      <p>{t('No check-in schedules yet. Choose platforms and a default time first.')}</p>
      {!options.some(option => option.available) ? <Button variant='outline' size='sm' onClick={() => window.wbApp?.showPage?.('plugins')}>{t('Install check-in support on the Plugins page')}</Button> : null}
    </Empty> : null}
    <CheckinScheduleRules />
    <CheckinScheduleExtras center={center} />
    <CheckinScheduleEditor entries={entries} options={options} defaultRetryTime={auto.defaultRetryTime || '00:10'} />
  </div>
}
