import { Button, Input } from '@ui'
import { dayKey, updateCustomRange, type CustomDates } from '../../../ui-shared/date-range'
import { t } from '../i18n'

export function DateRangeFields<T extends CustomDates>({ dates, onChange, onClear }: {
  dates: T; onChange: (dates: T) => void; onClear?: () => void
}) {
  const today = dayKey(new Date())
  const change = (patch: Partial<CustomDates>) => {
    const next = updateCustomRange(dates, patch)
    if (next !== dates) onChange(next)
  }
  return <span className='flex flex-wrap items-center gap-1.5'>
    <Input type='date' value={dates.start} max={dates.end < today ? dates.end : today} className='w-[132px]'
      aria-label={t('Start date')} title={t('Start date (inclusive)')}
      onChange={event => change({ start: event.currentTarget.value })} />
    <span className='text-muted-foreground' aria-hidden='true'>～</span>
    <Input type='date' value={dates.end} min={dates.start} max={today} className='w-[132px]'
      aria-label={t('End date')} title={t('End date (inclusive)')}
      onChange={event => change({ end: event.currentTarget.value })} />
    {onClear ? <Button variant='ghost' size='sm' onClick={onClear}>{t('Clear date filter')}</Button> : null}
  </span>
}
