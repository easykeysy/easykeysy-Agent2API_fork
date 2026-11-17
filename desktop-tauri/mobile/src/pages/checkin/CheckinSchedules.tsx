import { useState } from 'react'
import {
  Button, Checkbox, Input, Sheet, SheetBody, SheetContent, SheetFooter,
  SheetHeader, SheetTitle, Switch,
  Card, Label } from '@ui'
import { Empty, SectionHead } from '../../components/ui'
import { store } from '../../state/store'
import {
  deleteCheckinSchedule, runCheckinSchedule, saveAutoCheckin, saveCheckinSchedule,
  type CheckinCenter, type ScheduleEntry,
} from '../../api/checkin'
import { checkinSummary } from '../../lib/checkin'
import { formatStamp } from '../../lib/format'
import type { CheckinAction } from '../CheckinPage'
import { CheckinRegionSettings } from './CheckinRegionSettings'
import { t } from '../../i18n'
import { checkinEnd, checkinMakeupTime, checkinRangeText, defaultCheckinEnd, providerCheckinEnd, validCheckinRange } from '../../../../ui-shared/checkin-window'

export function CheckinSchedules({ center, busy, action }: {
  center: CheckinCenter
  busy: string
  action: CheckinAction
}) {
  const [editing, setEditing] = useState<ScheduleEntry | 'new' | null>(null)
  const [remove, setRemove] = useState<ScheduleEntry | null>(null)
  const auto = center.auto
  const options = auto?.providerOptions || []
  const entries = auto?.schedules || []

  const toggle = (entry: ScheduleEntry, enabled: boolean) => {
    void action(`toggle:${entry.id}`, () => saveCheckinSchedule({
      id: entry.id, label: entry.label, enabled, time: entry.time, timeEnd: entry.timeEnd, providers: entry.providers,
    }), t('Saved'))
  }

  const run = async (id: string) => {
    const result = await action(`run:${id}`, () => runCheckinSchedule(id))
    if (result) {
      const summary = checkinSummary(result)
      store.toast(summary.text, summary.tone)
    }
  }

  return (
    <>
      <div className='m-section' style={{ marginTop: 0 }}>
        <SectionHead title={t('Scheduled check-in')} />
        <Card className='m-card m-card-pad'>
          <div className='m-ck-toolbar'>
            <div className='m-ck-grow'>
              <div className='m-ck-title'>{t('Master switch')}</div>
              <div className='m-ck-note'>{auto?.enabled ? t('Enabled schedules check in randomly within their windows') : t('All scheduled sign-in entries are paused')}</div>
            </div>
            <Switch checked={auto?.enabled === true} disabled={Boolean(busy)} aria-label={t('The master switch for scheduled sign-in')}
              onCheckedChange={enabled => void action('master', () => saveAutoCheckin(enabled), t('Saved'))} />
          </div>
          <p className='m-ck-note'>{t('Beijing time · Missed windows are caught up for unrun accounts. Restarting keeps today’s schedule. Tasks run on the gateway, so closing the mobile app does not affect them.')}</p>
          <Button size='sm' variant='outline' disabled={Boolean(busy) || !options.length} onClick={() => setEditing('new')}>
            {t('Add schedule')}
          </Button>
        </Card>
      </div>

      {entries.map(entry => (
        <div key={entry.id} className='m-section'>
          <Card className='m-card m-card-pad'>
            <div className='m-ck-toolbar'>
              <div className='m-ck-grow'>
                <div className='m-ck-title'>{entry.displayLabel || entry.label || entry.time}</div>
                <div className='m-ck-note'>{t('Daily at {time}', { time: checkinRangeText(entry.time, checkinEnd(entry.time, entry.timeEnd)) })}</div>
              </div>
              <Switch checked={entry.enabled} disabled={Boolean(busy)}
                aria-label={entry.displayLabel || entry.time} onCheckedChange={enabled => toggle(entry, enabled)} />
            </div>
            <p className='m-ck-note'>{entry.providers.map(id => t(options.find(item => item.id === id)?.label || id)).join(', ')}</p>
            {/* 各家按自己的窗口时刻签（Qoder 10:01 / ZCode 19:01），
                这一行说明「还没到点的是哪几家」——手机上最需要看到的一格 */}
            {(entry.providerRows || []).length
              ? <p className='m-ck-note'>
                  {entry.providerRows.map(row => {
                    const progress = Number(row.accountCount) > 0 ? t('{n} / {total} accounts processed', {
                      n: row.attemptedCount || 0, total: row.accountCount || 0,
                    }) : t('No eligible accounts')
                    return `${t(row.label)} ${checkinRangeText(row.time, row.timeEnd)} · ${progress}`
                  }).join(' · ')}
                  {entry.retryTime ? t(' · Catch-up runs after the main check-in window') : ''}
                  {Number(entry.retryCount) > 0 ? t(' · {n} accounts waiting for the window retry', { n: entry.retryCount }) : ''}
                  {/* 次数用完的账号已不在 retryCount 里：不单独说一句，这一行归零
                      会被读成「都签完了」 */}
                  {Number(entry.retryExhausted?.length ?? 0) > 0
                    ? t(' · {n} accounts used up today’s retries', { n: entry.retryExhausted?.length ?? 0 }) : ''}
                </p>
              : null}
            <p className='m-ck-note'>{entry.nextRunAt ? t('Next run: {time}', { time: formatStamp(entry.nextRunAt) })
              : t('No next run scheduled')}
              {entry.lastResult?.at ? t(' · Last run: {time}', { time: formatStamp(entry.lastResult.at) }) : ''}</p>
            {entry.lastResult ? <p className='m-ck-note'>{checkinSummary(entry.lastResult).text}</p> : null}
            <div className='m-ck-actions'>
              <Button size='sm' variant='outline' disabled={Boolean(busy) || auto?.running} onClick={() => void run(entry.id)}>
                {busy === `run:${entry.id}` ? t('Running…') : t('Run now')}
              </Button>
              <Button size='sm' variant='ghost' disabled={Boolean(busy)} onClick={() => setEditing(entry)}>{t('Edit')}</Button>
              <Button size='sm' variant='ghost' disabled={Boolean(busy)} className='text-destructive' onClick={() => setRemove(entry)}>{t('Delete')}</Button>
            </div>
          </Card>
        </div>
      ))}
      {!entries.length ? <Empty text={t('No scheduled entries yet; add one, or sign in manually from the sign-in center')} /> : null}

      <CheckinRegionSettings center={center} busy={busy} action={action} />

      {editing ? <ScheduleEditor key={typeof editing === 'string' ? editing : editing.id}
        entry={editing === 'new' ? null : editing} options={options} busy={busy} action={action} onClose={() => setEditing(null)} /> : null}
      <Sheet open={Boolean(remove)} onOpenChange={(open, details) => {
        if (open) return
        if (busy) { details.cancel(); return }
        setRemove(null)
      }}>
        <SheetContent showCloseButton={!busy}>
          <SheetHeader><SheetTitle>{t('Delete the scheduled sign-in entry')}</SheetTitle></SheetHeader>
          <SheetBody><p className='m-ck-note'>{t('Delete "{name}"? After deletion it will no longer run on schedule.', {
            name: remove?.displayLabel || remove?.time || '',
          })}</p></SheetBody>
          <SheetFooter>
            <Button variant='outline' disabled={Boolean(busy)} onClick={() => setRemove(null)}>{t('Cancel')}</Button>
            <Button variant='destructive' disabled={Boolean(busy)} onClick={async () => {
              if (!remove) return
              const result = await action(`delete:${remove.id}`, () => deleteCheckinSchedule(remove.id), t('Deleted'))
              if (result !== undefined) setRemove(null)
            }}>{busy ? t('Deleting…') : t('Delete')}</Button>
          </SheetFooter>
        </SheetContent>
      </Sheet>
    </>
  )
}

function ScheduleEditor({ entry, options, busy, action, onClose }: {
  entry: ScheduleEntry | null
  options: { id: string; label: string; builtinTime?: string | null }[]
  busy: string
  action: CheckinAction
  onClose: () => void
}) {
  const [label, setLabel] = useState(entry?.label || '')
  const [time, setTime] = useState(entry?.time || '00:01')
  const [timeEnd, setTimeEnd] = useState(checkinEnd(entry?.time || '00:01', entry?.timeEnd))
  const [providers, setProviders] = useState(entry?.providers || options.map(item => item.id))
  // 逐家时刻覆盖（空串 = 撤销）与补签轮：与桌面端同一套字段
  const [providerTimes, setProviderTimes] = useState<Record<string, string>>(entry?.providerTimes || {})
  const [retryTime, setRetryTime] = useState(entry?.retryTime ?? '00:10')
  const [retryGap, setRetryGap] = useState(String(entry?.retryGapMinutes ?? 10))
  const [retryLimit, setRetryLimit] = useState(String(entry?.retryLimit ?? 3))
  const [providerSearch, setProviderSearch] = useState('')
  const unavailable = providers.filter(id => !options.some(item => item.id === id))
  const available = [...options, ...unavailable.map(id => ({ id, label: id }))]
  const visibleProviders = available.filter(option => `${option.id} ${t(option.label)}`.toLowerCase().includes(providerSearch.trim().toLowerCase()))

  const submit = async () => {
    if (!validCheckinRange(time, timeEnd) || !providers.length) {
      store.toast(t('Enter a valid window with the end no earlier than the start, and select at least one provider.'), 'err')
      return
    }
    const overrides: Record<string, string> = {}
    for (const [id, value] of Object.entries(providerTimes)) {
      if (value) overrides[id] = value
    }
    const result = await action('schedule-save', () => saveCheckinSchedule({
      ...(entry ? { id: entry.id } : {}), label: label.trim(), time, timeEnd,
      enabled: entry ? entry.enabled : true, providers,
      providerTimes: overrides,
      retryTime: retryTime.trim(),
      retryGapMinutes: Number(retryGap) || 0,
      retryLimit: Number(retryLimit) || 0,
    }), t('Saved'))
    if (result) onClose()
  }

  return (
    <Sheet open onOpenChange={(open, details) => {
      if (open) return
      if (busy) { details.cancel(); return }
      onClose()
    }}>
      <SheetContent showCloseButton={!busy}>
        <SheetHeader><SheetTitle>{entry ? t('Edit the scheduled entry') : t('New scheduled entry')}</SheetTitle></SheetHeader>
        <SheetBody>
          <Label className='m-ck-field'>{t('Name (optional)')}
            <Input className='text-base' value={label} disabled={Boolean(busy)} onChange={event => setLabel(event.currentTarget.value)} />
          </Label>
          <Label className='m-ck-field'>{t('Random check-in start')}
            <Input type='time' className='text-base' value={time} disabled={Boolean(busy)} onChange={event => {
              const value = event.currentTarget.value
              setTime(value)
              if (!validCheckinRange(value, timeEnd)) setTimeEnd(defaultCheckinEnd(value))
            }} />
          </Label>
          <Label className='m-ck-field'>{t('Random check-in end')}
            <Input type='time' className='text-base' value={timeEnd} disabled={Boolean(busy)} onChange={event => setTimeEnd(event.currentTarget.value)} />
          </Label>
          <p className='m-ck-note'>{t('Each account runs once a day at a random time within the window. Equal start and end times mean a fixed-time run.')}</p>
          <div className='m-ck-title'>{t('Window start by provider')}</div>
          <p className='m-ck-note'>{t('Platform-specific or custom times set the start and use the default window length. Times are randomized daily; restarting keeps today’s schedule.')}</p>
          {available.filter(option => providers.includes(option.id)).map(option => {
            const builtin = options.find(item => item.id === option.id)?.builtinTime || ''
            const value = providerTimes[option.id] || ''
            return (
              <Label className='m-ck-field' key={`time-${option.id}`}>
                {t(option.label)}{builtin ? t('(exclusive {time})', { time: builtin }) : ''}
                <Input type='time' className='text-base' value={value || builtin || time}
                  disabled={Boolean(busy)}
                  onChange={event => setProviderTimes(prev => ({ ...prev, [option.id]: event.currentTarget.value }))} />
                <span className='m-ck-note font-mono'>{checkinRangeText(value || builtin || time, providerCheckinEnd(value || builtin || time, time, timeEnd))}</span>
              </Label>
            )
          })}
          <p className='m-ck-note'>{t('Catch-up runs after the main check-in window and only processes accounts not checked in.')}</p>
          {retryTime && validCheckinRange(time, timeEnd) ? <p className='m-ck-note'>
            {available.filter(option => providers.includes(option.id)).map(option => {
              const start = providerTimes[option.id] || options.find(item => item.id === option.id)?.builtinTime || time
              const end = providerCheckinEnd(start, time, timeEnd)
              const makeup = checkinMakeupTime(end, retryTime, Number(retryGap) || 0)
              return `${t(option.label)} · ${t('Actual catch-up')} ${makeup || '—'}`
            }).join(' · ')}
          </p> : null}
          <Label className='m-ck-field'>{t('The make-up sign-in time (only fills in accounts that missed today; clearing= off)')}
            <Input type='time' className='text-base' value={retryTime} disabled={Boolean(busy)}
              onChange={event => setRetryTime(event.currentTarget.value)} />
          </Label>
          <div className='m-ck-toolbar'>
            <Label className='m-ck-grow'>{t('Window retry interval (minutes, 0= off)')}
              <Input type='number' className='text-base' value={retryGap} disabled={Boolean(busy)}
                onChange={event => setRetryGap(event.currentTarget.value)} />
            </Label>
            <Label className='m-ck-grow'>{t('Maximum retry count (0= off)')}
              <Input type='number' className='text-base' value={retryLimit} disabled={Boolean(busy)}
                onChange={event => setRetryLimit(event.currentTarget.value)} />
            </Label>
          </div>
          <div className='m-ck-title'>{t('Check-in scope')}</div>
          <Input type='search' value={providerSearch} disabled={Boolean(busy)}
            placeholder={t('Search providers')} aria-label={t('Search providers')}
            onChange={event => setProviderSearch(event.currentTarget.value)} />
          {visibleProviders.map(option => <Label className='m-ck-toolbar' key={option.id}>
            <Checkbox checked={providers.includes(option.id)} disabled={Boolean(busy)} onCheckedChange={checked => {
              setProviders(list => checked ? [...new Set([...list, option.id])] : list.filter(id => id !== option.id))
            }} />
            <span className='m-ck-grow m-ck-note'>{t(option.label)}{unavailable.includes(option.id) ? t('(the plugin is not ready; the original configuration is kept)') : ''}</span>
          </Label>)}
          {!visibleProviders.length ? <p className='m-ck-note'>{t('No matching options')}</p> : null}
        </SheetBody>
        <SheetFooter>
          <Button variant='outline' disabled={Boolean(busy)} onClick={onClose}>{t('Cancel')}</Button>
          <Button disabled={Boolean(busy) || !validCheckinRange(time, timeEnd) || !providers.length} onClick={() => void submit()}>{busy ? t('Saving…') : t('Save')}</Button>
        </SheetFooter>
      </SheetContent>
    </Sheet>
  )
}
