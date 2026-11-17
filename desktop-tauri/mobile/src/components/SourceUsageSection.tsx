import { useEffect, useState } from 'react'
import { Badge, Button, Card, Empty, SegmentedControl, Sheet, SheetBody, SheetContent, SheetHeader, SheetTitle } from '@ui'
import { fetchStatsSummary, type StatsDimension, type StatsSummary } from '../api/gateway'
import { store } from '../state/store'
import { formatInt, formatPercent, formatTokens } from '../lib/format'
import { t } from '../i18n'

export function reportSourceName(row: StatsDimension, key: boolean): string {
  if (!key) return !row.id ? t('Unrecorded client') : row.id === 'other' ? t('Other') : String(row.label || row.id)
  if (row.source === 'none') return t('Unauthenticated calls')
  if (row.source === 'env') return t('Environment variable key')
  if (row.source !== 'key') return t('Key not recorded in history')
  return row.label || row.id || t('Untitled')
}

function value(amount: number | null | undefined): string {
  return typeof amount === 'number' ? formatTokens(amount) : '—'
}

export function openMobileRequestLogs(summary: StatsSummary, query: Record<string, string>): void {
  store.push('requests', { ...query, startDate: summary.startDate || '', endDate: summary.endDate || '', productionOnly: '1', completedOnly: '1' })
}

export function SourceUsageSection({ summary, apiKeys = false }: { summary: StatsSummary; apiKeys?: boolean }) {
  const [sort, setSort] = useState('tokens')
  const [expanded, setExpanded] = useState(false)
  const [target, setTarget] = useState<StatsDimension | null>(null)
  const [detail, setDetail] = useState<StatsSummary | null>(null)
  const [error, setError] = useState('')
  const list = [...(apiKeys ? summary.apiKeys || [] : summary.clients || [])].sort((left, right) => {
    const key = sort === 'tokens' ? 'totalTokens' : 'requests'
    return (Number(right[key]) || 0) - (Number(left[key]) || 0)
  })
  useEffect(() => {
    if (!target) return
    const controller = new AbortController()
    setDetail(null)
    setError('')
    void fetchStatsSummary('custom', controller.signal, {
      apiKey: target.filterValue || 'unknown', startDate: summary.startDate, endDate: summary.endDate,
    }).then(setDetail).catch(failure => { if (!controller.signal.aborted) setError(String(failure)) })
    return () => controller.abort()
  }, [target, summary.startDate, summary.endDate])
  const query = (row: StatsDimension): Record<string, string> => apiKeys ? { apiKey: row.filterValue || 'unknown' } : { client: row.id || '', clientLabel: row.label || '' }
  return (
    <div className='m-section'>
      <div className='flex flex-wrap justify-between items-center gap-2 mb-2'>
        <b className='text-sm'>{apiKeys ? t('API key statistics') : t('Client statistics')}</b>
        <SegmentedControl value={sort} onValueChange={setSort} aria-label={t('Statistics sort order')}
          options={[{ value: 'tokens', label: t('By tokens') }, { value: 'requests', label: t('By requests') }]} />
      </div>
      <Card className='m-card p-3'>
        {!list.length ? <Empty>{t('No requests in the selected range')}</Empty> : (expanded ? list : list.slice(0, 5)).map(row => (
          <div className='border-b border-border py-2 last:border-0' key={row.filterValue || row.id || 'unknown'}>
            <div className='flex items-center gap-2'>
              <Button variant='link' size='sm' className='px-0 min-w-0 truncate' onClick={() => openMobileRequestLogs(summary, query(row))}>
                {reportSourceName(row, apiKeys)}
              </Button>
              {apiKeys && row.source === 'key' ? <Badge variant={row.state === 'enabled' ? 'success' : 'outline'}>
                {row.state === 'enabled' ? t('enabled') : row.state === 'disabled' ? t('Disabled') : t('Deleted')}
              </Badge> : null}
              <span className='ml-auto text-xs whitespace-nowrap'>{formatTokens(row.totalTokens || 0)}</span>
            </div>
            <div className='text-xs text-muted-foreground leading-6'>
              {t('{n} requests', { n: formatInt(row.requests || 0) })} · {t('Success / failure')} {formatInt(row.success || 0)} / {formatInt(row.failures || 0)}
              <br />{t('Success rate')} {typeof row.successRate === 'number' ? formatPercent(row.successRate) : '—'} · {t('Share')} {summary.overview?.tokens ? formatPercent((row.totalTokens || 0) / summary.overview.tokens) : '—'}
              <br />{t('Input / output')} {value(row.promptTokens)} / {value(row.completionTokens)}
              <br />{t('Cache reads')} {value(row.cacheReadTokens)} · {t('Cache hit rate')} {typeof row.cacheRate === 'number' ? formatPercent(row.cacheRate) : '—'}
              {apiKeys ? <><br />{t('Last call')} {row.lastUsedAt ? new Date(row.lastUsedAt).toLocaleString() : '—'}</> : null}
            </div>
            {apiKeys ? <Button variant='ghost' size='sm' onClick={() => setTarget(row)}>{t('Usage details')}</Button> : null}
          </div>
        ))}
        {list.length > 5 ? <Button variant='ghost' size='sm' className='w-full' onClick={() => setExpanded(!expanded)}>
          {expanded ? t('Collapse') : t('View all')}
        </Button> : null}
      </Card>
      <Sheet open={!!target} onOpenChange={open => { if (!open) setTarget(null) }}>
        <SheetContent><SheetHeader><SheetTitle>{t('API key usage details')} · {target && reportSourceName(target, true)}</SheetTitle></SheetHeader>
          <SheetBody>
            {error ? <Empty>{error}</Empty> : !detail ? <Empty>{t('Loading…')}</Empty> : (
              <>{(['clients', 'models'] as const).map(section => (
                <div key={section} className='mb-4'>
                  <b>{section === 'clients' ? t('Client breakdown') : t('Model breakdown')}</b>
                  {detail.apiKeyDetail?.[section]?.length ? detail.apiKeyDetail[section]?.map((row, index) => (
                    <Button key={row.id || row.model || index} variant='ghost' className='w-full justify-between my-1' onClick={() => {
                      openMobileRequestLogs(summary, { apiKey: target?.filterValue || 'unknown',
                        ...(section === 'clients' ? { client: row.id || '', clientLabel: row.label || '' } : { model: row.model || '' }) })
                      setTarget(null)
                    }}><span className='truncate'>{section === 'clients' ? reportSourceName(row, false) : row.label || row.model}</span>
                      <span>{formatTokens(row.totalTokens || 0)} · {formatInt(row.requests || 0)}</span></Button>
                  )) : <Empty>{t('No recoverable breakdown')}</Empty>}
                </div>
              ))}</>
            )}
          </SheetBody>
        </SheetContent>
      </Sheet>
    </div>
  )
}
