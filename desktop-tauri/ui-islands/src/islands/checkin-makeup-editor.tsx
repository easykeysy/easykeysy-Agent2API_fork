import * as React from 'react'
import {
  Button, Dialog, DialogBody, DialogContent, DialogFooter, DialogHeader, DialogTitle,
  Input, Label, Switch, Table, TableBody, TableCell, TableHead, TableHeader, TableRow,
} from '@ui'
import { t } from '../i18n'
import { saveScheduleEntry, store } from './checkin-store'
import type { ScheduleEntry } from './checkin-types'
import {
  activityRetryEnabled, checkinRangeText, groupScheduleRows, previewMakeupTime, scheduleName, schedulePayload,
  scheduleRows, validScheduleTime, type ScheduleOption,
} from './checkin-schedule-model'

export type MakeupEditorKind = 'makeup' | 'retry'

export function CheckinMakeupEditor({ entry, kind, options, defaultRetryTime, onClose }: {
  entry: ScheduleEntry
  kind: MakeupEditorKind
  options: ScheduleOption[]
  defaultRetryTime: string
  onClose(): void
}): React.ReactElement {
  const isMakeup = kind === 'makeup'
  const [enabled, setEnabled] = React.useState(isMakeup ? !!entry.retryTime : activityRetryEnabled(entry))
  const [time, setTime] = React.useState(entry.retryTime || defaultRetryTime)
  const [gap, setGap] = React.useState(String(Math.max(0, entry.retryGapMinutes)))
  const [limit, setLimit] = React.useState(String(entry.retryLimit || 3))
  const gapValue = Number(gap)
  const limitValue = Number(limit)
  const gapValid = gap.trim() !== '' && Number.isInteger(gapValue) && gapValue >= 0 && gapValue <= 120
  const limitValid = limit.trim() !== '' && Number.isInteger(limitValue) && limitValue >= 1 && limitValue <= 20
  const canSave = !store.saving && (isMakeup
    ? !enabled || validScheduleTime(time) && gapValid
    : !enabled || gapValid && gapValue > 0 && limitValid)
  const rows = scheduleRows(entry, options)
  const groups = groupScheduleRows(rows)
  const submit = async () => {
    if (!canSave) return
    const changes = isMakeup
      ? { retryTime: enabled ? time : '', ...(enabled ? { retryGapMinutes: gapValue } : {}) }
      : { retryLimit: enabled ? limitValue : 0, ...(enabled ? { retryGapMinutes: gapValue } : {}) }
    const saved = await saveScheduleEntry(schedulePayload(entry, changes), { closeEditor: false })
    if (saved) onClose()
  }

  return <Dialog open onOpenChange={open => { if (!open && !store.saving) onClose() }}>
    <DialogContent className='ck-makeup-editor-dialog w-[min(700px,calc(100vw-48px))] max-[767px]:w-[calc(100vw-24px)]'>
      <DialogHeader><DialogTitle>{isMakeup ? t('Configure scheduled catch-up') : t('Configure campaign-window retries')}</DialogTitle></DialogHeader>
      <DialogBody>
        <p className='ck-schedule-hint'>{t('Linked check-in schedule: {name}', { name: scheduleName(entry) })}</p>
        <div className='ck-makeup-setting'>
          <div><h3>{isMakeup ? t('Enable scheduled catch-up') : t('Enable campaign-window retries')}</h3>
            <p>{isMakeup ? t('Checks once daily, only for accounts not checked in today.') : t('Only retries accounts waiting for a window or campaign; it does not repeat every error.')}</p>
          </div>
          <Switch checked={enabled} disabled={store.saving} onCheckedChange={next => {
            setEnabled(next === true)
            if (!isMakeup && next && gapValue <= 0) setGap('10')
          }} aria-label={isMakeup ? t('Enable scheduled catch-up') : t('Enable campaign-window retries')} />
        </div>
        <div className='ck-makeup-editor-fields'>
          <Label className='ck-schedule-field grid items-stretch'><span>{isMakeup ? t('Wait after window ends (minutes)') : t('Retry interval (minutes)')}</span>
            <Input type='number' value={gap} min={isMakeup ? 0 : 1} max={120} step={1}
              disabled={!enabled || store.saving} onChange={event => setGap(event.currentTarget.value)} />
          </Label>
          {isMakeup ? <Label className='ck-schedule-field grid items-stretch'><span>{t('Catch-up no earlier than')}</span>
            <Input type='time' value={time} className='font-mono tabular-nums' disabled={!enabled || store.saving}
              onChange={event => setTime(event.currentTarget.value)} />
          </Label> : <Label className='ck-schedule-field grid items-stretch'><span>{t('Maximum retries per account per day')}</span>
            <Input type='number' value={limit} min={1} max={20} step={1} disabled={!enabled || store.saving}
              onChange={event => setLimit(event.currentTarget.value)} />
          </Label>}
        </div>
        {enabled && !canSave && !store.saving ? <p className='ck-schedule-error'>{t('Check that the time, interval and count are within the allowed ranges.')}</p> : null}
        {isMakeup ? <>
          <p className='ck-schedule-hint'>{t('Catch-up uses the later of the main window end plus the wait interval and the earliest catch-up time. Accounts still waiting for their turn are not processed early.')}</p>
          <Table className='ck-makeup-preview-table'>
            <TableHeader><TableRow><TableHead>{t('Platform group')}</TableHead><TableHead>{t('Check-in window')}</TableHead><TableHead>{t('Actual catch-up')}</TableHead></TableRow></TableHeader>
            <TableBody>{groups.map(group => <TableRow key={group.time}>
              <TableCell>{group.rows.map(row => t(row.label)).join(', ')}</TableCell>
              <TableCell className='font-mono tabular-nums'>{checkinRangeText(group.time, group.timeEnd)}</TableCell>
              <TableCell className='font-mono tabular-nums'>{enabled && gapValid
                ? previewMakeupTime(group.timeEnd || group.time, time, gapValue) || '—' : t('Disabled')}</TableCell>
            </TableRow>)}</TableBody>
          </Table>
        </> : null}
        <div className='ck-schedule-editor-note'>{t('The current gateway shares the catch-up wait and campaign retry interval. Changing it affects both; switches and retry counts are still separate.')}</div>
        {isMakeup ? <p className='ck-schedule-hint'>{t('An interval of 0 disables campaign retries. Scheduled catch-up still waits at least 1 minute and remains enabled.')}</p>
          : <>
            <p className='ck-schedule-hint'>{t('The limit is counted per account, per day: an account that reaches it is not retried again today and resets tomorrow.')}</p>
            <p className='ck-schedule-hint'>{t('Disabling campaign-window retries does not disable scheduled catch-up. If the plugin supplies a retry time, that time takes precedence.')}</p>
          </>}
      </DialogBody>
      <DialogFooter><Button size='sm' variant='outline' disabled={store.saving} onClick={onClose}>{t('Cancel')}</Button>
        <Button size='sm' disabled={!canSave} onClick={() => void submit()}>{store.saving ? t('Saving…') : t('Save settings')}</Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
}
