import * as React from 'react'
import { Badge, Button, Dialog, DialogBody, DialogContent, DialogHeader, DialogTitle, Empty, Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@ui'
import { formatInt, formatTokens, type CustomRange, type StatsGroup, type StatsSummary } from '../report-charts'
import { openRequestLogs, sourceName } from './source-stats'
import { t } from '../../i18n'

export function ApiKeyUsageDialog({ target, range, dates, onClose }: {
  target: StatsGroup; range: string; dates?: CustomRange; onClose: () => void
}) {
  const [data, setData] = React.useState<StatsSummary | null>(null)
  const [error, setError] = React.useState('')
  const filterValue = target.filterValue || 'unknown'
  React.useEffect(() => {
    let cancelled = false
    setData(null)
    setError('')
    const api = (window as unknown as { workbuddyDesktop?: {
      getStatsSummary(range: string, dates?: CustomRange, options?: { apiKey?: string }): Promise<StatsSummary>
    } }).workbuddyDesktop
    if (!api) { setError(t('Backend bridge unavailable')); return }
    void api.getStatsSummary(range, dates, { apiKey: filterValue }).then(value => {
      if (!cancelled) setData(value)
    }).catch(failure => {
      if (!cancelled) setError(failure instanceof Error ? failure.message : String(failure))
    })
    return () => { cancelled = true }
  }, [range, dates?.start, dates?.end, filterValue])
  const detail = data?.apiKeyDetail
  const showRows = (rows: StatsGroup[] | undefined, clients: boolean) => {
    const total = Number(detail?.totalTokens) || 0
    return rows?.length ? (
      <Table><TableHeader><TableRow>
        <TableHead>{clients ? t('Client') : t('Model')}</TableHead>
        <TableHead className='text-right'>{t('Requests')}</TableHead>
        <TableHead className='text-right'>{t('Total tokens')}</TableHead>
        <TableHead className='text-right'>{t('Share')}</TableHead>
        <TableHead className='text-right'>{t('Actions')}</TableHead>
      </TableRow></TableHeader><TableBody>{rows.map((row, index) => (
        <TableRow key={row.id || row.model || index}>
          <TableCell>{clients ? sourceName(row, 'clients') : row.label || row.model || t('Unknown model')}</TableCell>
          <TableCell className='text-right'>{formatInt(row.requests)}</TableCell>
          <TableCell className='text-right'>{formatTokens(row.totalTokens)}</TableCell>
          <TableCell className='text-right'>{total ? `${((Number(row.totalTokens) || 0) / total * 100).toFixed(1)}%` : '—'}</TableCell>
          <TableCell className='text-right'><Button variant='ghost' size='sm' onClick={() => {
            openRequestLogs({ apiKey: filterValue, startDate: data?.startDate, endDate: data?.endDate,
              ...(clients ? { client: row.id || '' } : { model: row.model || '' }) })
            onClose()
          }}>{t('Request logs')}</Button></TableCell>
        </TableRow>
      ))}</TableBody></Table>
    ) : <Empty>{t('No recoverable breakdown')}</Empty>
  }
  return (
    <Dialog open onOpenChange={open => { if (!open) onClose() }}>
      <DialogContent className='max-w-[900px]'>
        <DialogHeader><DialogTitle>{t('API key usage details')} · {sourceName(target, 'apiKeys')}</DialogTitle></DialogHeader>
        <DialogBody>
          <div className='flex items-center gap-2 text-xs text-muted-foreground'>
            {data?.startDate} ～ {data?.endDate}
            {target.source !== 'key' ? <Badge variant='outline'>{sourceName(target, 'apiKeys')}</Badge> : null}
          </div>
          {error ? <Empty className='text-destructive'>{error}</Empty> : !data ? <Empty>{t('Loading…')}</Empty> : (
            <>
              {detail?.detailsComplete === false ? <p className='text-xs text-warning'>{t('Historical details are incomplete; input, output and cache readings cannot be recovered.')}</p> : null}
              <h3 className='mt-4 mb-2 font-semibold'>{t('Client breakdown')}</h3>
              {showRows(detail?.clients, true)}
              <h3 className='mt-4 mb-2 font-semibold'>{t('Model breakdown')}</h3>
              {showRows(detail?.models, false)}
              <p className='mt-3 text-xs text-muted-foreground'>{t('Breakdowns come from retained daily source aggregates; clients or models are not inferred for unrecoverable history.')}</p>
            </>
          )}
        </DialogBody>
      </DialogContent>
    </Dialog>
  )
}
