import * as React from 'react'
import {
  Badge, Button, Checkbox, Dialog, DialogBody, DialogContent, DialogFooter, DialogHeader,
  DialogTitle, Input, Label,
} from '@ui'
import { t } from '../i18n'
import { patch, saveScheduleEntry, store } from './checkin-store'
import type { ScheduleEntry } from './checkin-types'
import type { ScheduleProviderRow } from '../../../ui-shared/checkin'
import { ProviderLogo } from './checkin-row'
import {
  checkinEnd, checkinRangeText, effectiveScheduleTime, groupScheduleRows, providerCheckinEnd,
  schedulePayload, validScheduleTime, type ScheduleOption,
} from './checkin-schedule-model'
import { defaultCheckinEnd, validCheckinRange } from '../../../ui-shared/checkin-window'

function ScheduleEditorForm({ entry, options, defaultRetryTime, onClose }: {
  entry: ScheduleEntry | null
  options: ScheduleOption[]
  defaultRetryTime: string
  onClose(): void
}): React.ReactElement {
  const [name, setName] = React.useState(entry?.label ?? '')
  const [time, setTime] = React.useState(entry?.time || '00:01')
  const [timeEnd, setTimeEnd] = React.useState(checkinEnd(entry?.time || '00:01', entry?.timeEnd))
  const [providers, setProviders] = React.useState(entry ? [...entry.providers] : options.filter(item => item.available).map(item => item.id))
  const [providerTimes, setProviderTimes] = React.useState<Record<string, string>>({ ...entry?.providerTimes })
  const [search, setSearch] = React.useState('')
  const rows: ScheduleProviderRow[] = options.filter(option => providers.includes(option.id)).map(option => ({
    id: option.id, label: option.label, time: effectiveScheduleTime(time, providerTimes, option),
    timeEnd: providerCheckinEnd(effectiveScheduleTime(time, providerTimes, option), time, timeEnd),
    builtin: option.builtinTime ?? null, firedToday: false,
  }))
  const groups = groupScheduleRows(rows)
  const canSave = !store.saving && validCheckinRange(time, timeEnd) && providers.length > 0
    && Object.values(providerTimes).every(value => !value || validScheduleTime(value))
  const visible = options.filter(option => t(option.label).toLocaleLowerCase().includes(search.trim().toLocaleLowerCase()))
  const toggle = (id: string, checked: boolean) => setProviders(previous => checked
    ? [...new Set([...previous, id])] : previous.filter(item => item !== id))
  const resetTime = (id: string) => setProviderTimes(previous => {
    const next = { ...previous }
    delete next[id]
    return next
  })
  const submit = () => {
    if (!canSave) return
    const times = Object.fromEntries(Object.entries(providerTimes).filter(([, value]) => value))
    void saveScheduleEntry(entry
      ? schedulePayload(entry, { label: name.trim(), time, timeEnd, providers, providerTimes: times })
      : {
          label: name.trim(), time, timeEnd, providers, providerTimes: times, enabled: true,
          retryTime: defaultRetryTime, retryGapMinutes: 10, retryLimit: 3,
        })
  }

  return <>
    <DialogHeader><DialogTitle>{entry ? t('Edit check-in schedule') : t('New check-in schedule')}</DialogTitle></DialogHeader>
    <DialogBody className='ck-schedule-editor-body grid gap-0 p-0'>
      <div className='ck-schedule-editor-fields'>
        <section className='ck-schedule-form-section'>
          <h3>{t('Basic settings')}</h3>
          <div className='ck-schedule-basic-fields'>
            <Label className='ck-schedule-field grid items-stretch'>
              <span>{t('Schedule name')} <small>{t('Optional')}</small></span>
              <Input value={name} disabled={store.saving} placeholder={t('For example: Daily automatic check-in')}
                onChange={event => setName(event.currentTarget.value)} />
            </Label>
            <Label className='ck-schedule-field grid items-stretch'>
              <span>{t('Random check-in start')}</span>
              <Input type='time' value={time} disabled={store.saving} className='font-mono tabular-nums'
                onChange={event => {
                  const value = event.currentTarget.value
                  setTime(value)
                  if (!validCheckinRange(value, timeEnd)) setTimeEnd(defaultCheckinEnd(value))
                }} />
            </Label>
            <Label className='ck-schedule-field grid items-stretch'>
              <span>{t('Random check-in end')}</span>
              <Input type='time' value={timeEnd} disabled={store.saving} className='font-mono tabular-nums'
                onChange={event => setTimeEnd(event.currentTarget.value)} />
            </Label>
          </div>
          <p className='ck-schedule-hint'>{t('Each account runs once a day at a random time within the window. Equal start and end times mean a fixed-time run.')}</p>
          {!validCheckinRange(time, timeEnd) ? <p className='ck-schedule-error'>{t('The end cannot be earlier than the start. Overnight windows are not supported yet.')}</p> : null}
        </section>
        <section className='ck-schedule-form-section'>
          <div className='ck-schedule-form-title'><h3>{t('Platforms and random check-in windows')}</h3>
            <span>{t('{n} / {total} platforms selected', { n: providers.length, total: options.length })}</span>
          </div>
          <div className='ck-schedule-picker-tools'>
            <Input type='search' value={search} disabled={store.saving} placeholder={t('Search platforms')}
              aria-label={t('Search check-in platforms')} onChange={event => setSearch(event.currentTarget.value)} />
            <Button variant='link' size='xs' disabled={store.saving}
              onClick={() => setProviders(options.map(item => item.id))}>{t('Select all')}</Button>
            <Button variant='ghost' size='xs' disabled={store.saving}
              onClick={() => setProviders([])}>{t('Clear')}</Button>
          </div>
          <div className='ck-schedule-picker'>
            {visible.map(option => {
              const checked = providers.includes(option.id)
              const override = providerTimes[option.id] ?? ''
              return <div className={`ck-schedule-picker-row${checked ? ' selected' : ''}`} key={option.id}>
                <Label className='ck-schedule-picker-label'>
                  <Checkbox checked={checked} disabled={store.saving}
                    aria-label={t('Select {name}', { name: t(option.label) })}
                    onCheckedChange={next => toggle(option.id, next === true)} />
                  <ProviderLogo id={option.id} label={t(option.label)} />
                  <span className='ck-schedule-picker-name'>{t(option.label)}
                    {!option.available ? <small>{t('Plugin unavailable; scope retained')}</small> : null}
                  </span>
                </Label>
                <div className='ck-schedule-picker-time'>
                  <Input type='time' value={override || effectiveScheduleTime(time, providerTimes, option)}
                    disabled={!checked || store.saving} className='w-[108px] font-mono tabular-nums'
                    aria-label={t('Check-in time for {name}', { name: t(option.label) })}
                    onChange={event => setProviderTimes(previous => ({ ...previous, [option.id]: event.currentTarget.value }))} />
                  <small className='font-mono tabular-nums'>{checkinRangeText(
                    effectiveScheduleTime(time, providerTimes, option),
                    providerCheckinEnd(effectiveScheduleTime(time, providerTimes, option), time, timeEnd),
                  )}</small>
                  {override ? <Button size='xs' variant='link' disabled={store.saving} onClick={() => resetTime(option.id)}>
                    {t('Restore Defaults')}
                  </Button> : <span>{option.builtinTime ? t('Platform-specific') : t('Follow Default')}</span>}
                </div>
              </div>
            })}
            {!visible.length ? <p className='ck-schedule-hint'>{t('No matching platforms')}</p> : null}
          </div>
          {!providers.length ? <p className='ck-schedule-error'>{t('Select at least one check-in platform')}</p> : null}
        </section>
        <div className='ck-schedule-editor-note'>{entry
          ? t('Only scheduled check-ins are edited here. Catch-up and campaign retry settings are preserved and configured separately on the catch-up tab.')
          : t('New schedules enable catch-up and campaign retries by default. After saving, adjust them on the Scheduled catch-up tab.')}</div>
      </div>
      <aside className='ck-schedule-preview'>
        <h3>{t('Actual check-in schedule')}</h3><p>{t('Beijing time · Repeats daily · Updates with settings')}</p>
        <div className='ck-schedule-preview-groups'>
          {groups.map(group => <div className='ck-schedule-preview-group' key={group.time}>
            <strong className='font-mono tabular-nums'>{checkinRangeText(group.time, group.timeEnd) || '—'}</strong>
            <Badge variant='outline' shape='tag'>{t('{n} platforms', { n: group.rows.length })}</Badge>
            <div>{group.rows.map(row => t(row.label)).join(', ')}</div>
          </div>)}
          {!groups.length ? <p>{t('Select platforms to see actual check-in times')}</p> : null}
        </div>
        <p className='ck-schedule-preview-note'>{t('Platform-specific or custom times set the start and use the default window length. Times are randomized daily; restarting keeps today’s schedule.')}</p>
      </aside>
    </DialogBody>
    <DialogFooter>
      <span className='ck-schedule-footer-note'>{t('{n} platforms · {groups} time groups', { n: providers.length, groups: groups.length })}</span>
      <Button variant='outline' size='sm' disabled={store.saving} onClick={onClose}>{t('Cancel')}</Button>
      <Button size='sm' disabled={!canSave} onClick={submit}>{store.saving ? t('Saving…') : t('Save schedule')}</Button>
    </DialogFooter>
  </>
}

export function CheckinScheduleEditor({ entries, options, defaultRetryTime }: {
  entries: ScheduleEntry[]
  options: ScheduleOption[]
  defaultRetryTime: string
}): React.ReactElement | null {
  const id = store.editingId
  if (!id) return null
  const entry = id === 'new' ? null : entries.find(item => item.id === id)
  if (entry === undefined) return null
  const close = () => patch({ editingId: null })
  return <Dialog open onOpenChange={open => { if (!open && !store.saving) close() }}>
    <DialogContent className='ck-schedule-editor-dialog w-[min(1000px,calc(100vw-48px))] max-[767px]:w-[calc(100vw-24px)]'>
      <ScheduleEditorForm key={id} entry={entry} options={options}
        defaultRetryTime={defaultRetryTime} onClose={close} />
    </DialogContent>
  </Dialog>
}
