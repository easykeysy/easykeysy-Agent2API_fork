import { useCallback, useEffect, useRef, useState } from 'react'
import {
  Button, Input, Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
  Sheet, SheetBody, SheetContent, SheetFooter, SheetHeader, SheetTitle,
  Card } from '@ui'
import { Badge, Empty, ErrorNote, SectionHead } from '../../components/ui'
import { clearCheckinRecords, getCheckinRecords, type RecordsPage } from '../../api/checkin'
import { recordRangeStart } from '../../lib/checkin'
import { formatStamp } from '../../lib/format'
import { serverIdentity } from '../../lib/servers'
import { store } from '../../state/store'
import { t } from '../../i18n'
import { SearchableSelect } from '../../../../ui-shared/searchable-select'

const PAGE_SIZE = 30

function outcomeView(outcome?: string): { label: string; tone: 'ok' | 'warn' | 'bad' | 'mute' | 'brand' } {
  switch (outcome) {
    case 'success': return { label: t('Claimed'), tone: 'ok' }
    case 'already': return { label: t('claimed today'), tone: 'brand' }
    case 'pending': return { label: t('Not claimed'), tone: 'warn' }
    case 'captcha': return { label: t('Human verification required'), tone: 'warn' }
    case 'failed': return { label: t('Failed'), tone: 'bad' }
    default: return { label: outcome || '—', tone: 'mute' }
  }
}

function sourceLabel(source?: string): string {
  switch (source) {
    case 'schedule': return t('Scheduled')
    case 'makeup': return t('Catch-up')
    case 'startup': return t('Catch-up')
    case 'manual': return t('Manual')
    case 'single': return t('Single')
    case 'activity': return t('Campaign')
    default: return source || '—'
  }
}

export function CheckinRecords({ providers }: { providers: { id: string; label: string }[] }) {
  const [range, setRange] = useState('all')
  const [outcome, setOutcome] = useState('all')
  const [kind, setKind] = useState('all')
  const [provider, setProvider] = useState('all')
  const [keyword, setKeyword] = useState('')
  const [search, setSearch] = useState('')
  const [offset, setOffset] = useState(0)
  const [page, setPage] = useState<RecordsPage | null>(null)
  const [error, setError] = useState('')
  const [loading, setLoading] = useState(false)
  const [clearing, setClearing] = useState(false)
  const [confirm, setConfirm] = useState<'filtered' | 'all' | null>(null)
  const controller = useRef<AbortController | null>(null)
  const identity = serverIdentity()
  const ranges = [
    { value: 'all', label: t('All time') }, { value: 'today', label: t('Today') },
    { value: '7', label: t('7 days') }, { value: '30', label: t('30 days') }, { value: 'month', label: t('This month') },
  ]
  const outcomes = [
    { value: 'all', label: t('All outcomes') }, { value: 'success', label: t('Claimed') },
    { value: 'already', label: t('claimed today') }, { value: 'pending', label: t('Not claimed') },
    { value: 'captcha', label: t('Human verification required') }, { value: 'failed', label: t('Failed') },
  ]
  const kinds = [
    { value: 'all', label: t('All types') }, { value: 'manual', label: t('Manual check-in') },
    { value: 'schedule', label: t('Scheduled check-in') }, { value: 'makeup', label: t('Scheduled catch-up') },
  ]
  const providerOptions = [{ value: 'all', label: t('All providers') }, ...providers.map(item => ({ value: item.id, label: t(item.label) }))]
  const filters = useCallback(() => ({
    start: recordRangeStart(range), outcome: outcome === 'all' ? undefined : outcome,
    kind: kind === 'all' ? undefined : kind,
    provider: provider === 'all' ? undefined : provider, keyword: search || undefined,
  }), [range, outcome, kind, provider, search])

  const load = useCallback(async () => {
    controller.current?.abort()
    const request = new AbortController()
    controller.current = request
    setLoading(true)
    try {
      const data = await getCheckinRecords({ ...filters(), offset, limit: PAGE_SIZE }, request.signal)
      if (request.signal.aborted || serverIdentity() !== identity) return
      if (offset > 0 && offset >= data.total) { setOffset(Math.max(0, Math.ceil(data.total / PAGE_SIZE) - 1) * PAGE_SIZE); return }
      setPage(data)
      setError('')
    } catch (cause) {
      if (!request.signal.aborted && serverIdentity() === identity) setError(cause instanceof Error ? cause.message : String(cause))
    } finally {
      if (!request.signal.aborted && serverIdentity() === identity) setLoading(false)
    }
  }, [filters, offset, identity])

  useEffect(() => { void load(); return () => controller.current?.abort() }, [load])

  const change = (setter: (value: string) => void, value: string | null) => {
    setter(value || 'all')
    setOffset(0)
  }

  const remove = async () => {
    if (!confirm || clearing) return
    setClearing(true)
    try {
      const result = await clearCheckinRecords(filters(), confirm === 'all')
      if (serverIdentity() !== identity) return
      setConfirm(null)
      store.toast(t('Deleted {n} sign-in records', { n: result.removed }))
      if (offset > 0) setOffset(0)
      else await load()
    } catch (cause) {
      if (serverIdentity() === identity) store.toast(cause instanceof Error ? cause.message : String(cause), 'err')
    } finally { setClearing(false) }
  }

  const total = page?.total || 0
  const filtered = range !== 'all' || outcome !== 'all' || kind !== 'all' || provider !== 'all' || Boolean(search)
  const controls = [
    { value: range, options: ranges, label: t('Time range'), setter: setRange },
    { value: outcome, options: outcomes, label: t('Result'), setter: setOutcome },
    { value: provider, options: providerOptions, label: t('Provider'), setter: setProvider, searchable: true },
    { value: kind, options: kinds, label: t('Check-in type'), setter: setKind },
  ]

  return (
    <>
      <div className='m-section' style={{ marginTop: 0 }}>
        <SectionHead title={t('Check-in records')} />
        <Card className='m-card m-card-pad'>
          <div className='m-ck-filters'>
            {controls.map(control => control.searchable
              ? <SearchableSelect key={control.label} value={control.value} options={control.options} disabled={clearing}
                onValueChange={value => change(control.setter, value)} className='h-10 text-sm'
                aria-label={control.label} searchPlaceholder={t('Search providers')} />
              : <Select key={control.label} value={control.value} disabled={clearing}
                onValueChange={value => change(control.setter, value)}>
                <SelectTrigger className='h-10 text-sm' aria-label={control.label}>
                  <SelectValue>{control.options.find(item => item.value === control.value)?.label || control.value}</SelectValue>
                </SelectTrigger>
                <SelectContent>{control.options.map(item => <SelectItem key={item.value} value={item.value}>{item.label}</SelectItem>)}</SelectContent>
              </Select>)}
          </div>
          <form className='m-ck-toolbar' onSubmit={event => { event.preventDefault(); setSearch(keyword.trim()); setOffset(0) }}>
            <Input type='search' className='text-base' value={keyword} placeholder={t('Search accounts / Note')} disabled={clearing}
              aria-label={t('Search accounts / Note')} onChange={event => setKeyword(event.currentTarget.value)} />
            <Button type='submit' size='sm' variant='outline' disabled={clearing}>{t('Search')}</Button>
          </form>
          <div className='m-ck-actions'>
            <Button size='sm' variant='outline' disabled={loading || clearing} onClick={() => void load()}>{t('Refresh')}</Button>
            {filtered ? <Button size='sm' variant='ghost' className='text-destructive' disabled={clearing || !total}
              onClick={() => setConfirm('filtered')}>{t('Clear the filter results')}</Button> : null}
            <Button size='sm' variant='ghost' className='text-destructive' disabled={clearing}
              onClick={() => setConfirm('all')}>{t('Clear all')}</Button>
          </div>
          <p className='m-ck-note'>{t('Total {n} entries', { n: total })}</p>
        </Card>
      </div>
      {error ? <ErrorNote text={error} /> : null}
      {!page && loading ? <Empty text={t('Reading the sign-in records…')} /> : !page?.records?.length ? <Empty text={t('这段时间里还没有签到记录')} /> : null}
      {page?.records?.map((row, index) => {
        const view = outcomeView(row.outcome)
        return <div className='m-section' key={`${row.ts}:${row.accountId}:${index}`}>
          <Card className='m-card m-card-pad'>
            <div className='m-ck-toolbar'>
              <div className='m-ck-title m-ck-grow'>{row.accountName || row.accountId || '—'}</div>
              <Badge tone={view.tone}>{view.label}</Badge>
            </div>
            <p className='m-ck-note'>{formatStamp(row.ts)} · {providers.find(item => item.id === row.provider)?.label || row.provider || '—'} · {sourceLabel(row.source)}</p>
            <p className={row.error ? 'm-ck-error' : 'm-ck-note'}>{row.message || row.error || (row.keptActive ? t('Keep-alive succeeded') : '—')}</p>
            {row.reason ? <p className='m-ck-note'>{row.reason}</p> : null}
          </Card>
        </div>
      })}
      {total > PAGE_SIZE ? <div className='m-section m-ck-toolbar'>
        <Button variant='outline' size='sm' disabled={loading || offset === 0} onClick={() => setOffset(value => Math.max(0, value - PAGE_SIZE))}>{t('Previous')}</Button>
        <span className='m-ck-note'>{Math.floor(offset / PAGE_SIZE) + 1} / {Math.ceil(total / PAGE_SIZE)}</span>
        <Button variant='outline' size='sm' disabled={loading || offset + PAGE_SIZE >= total} onClick={() => setOffset(value => value + PAGE_SIZE)}>{t('Next')}</Button>
      </div> : null}
      <Sheet open={Boolean(confirm)} onOpenChange={(open, details) => {
        if (open) return
        if (clearing) { details.cancel(); return }
        setConfirm(null)
      }}>
        <SheetContent showCloseButton={!clearing}>
          <SheetHeader><SheetTitle>{confirm === 'all' ? t('Clear all sign-in records') : t('Clear the filter results')}</SheetTitle></SheetHeader>
          <SheetBody><p className='m-ck-note'>{confirm === 'all' ? t('Clear all sign-in records? Once cleared, they cannot be recovered.')
            : t('Delete the {n} sign-in records matched by the current filter? Once cleared, they cannot be recovered.', { n: total })}</p></SheetBody>
          <SheetFooter>
            <Button variant='outline' disabled={clearing} onClick={() => setConfirm(null)}>{t('Cancel')}</Button>
            <Button variant='destructive' disabled={clearing} onClick={() => void remove()}>{clearing ? t('Deleting…') : t('Delete')}</Button>
          </SheetFooter>
        </SheetContent>
      </Sheet>
    </>
  )
}
