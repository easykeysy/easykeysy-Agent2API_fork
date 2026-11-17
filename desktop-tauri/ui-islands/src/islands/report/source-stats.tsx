import * as React from 'react'
import { Badge, Button, Card, CardContent, CardHeader, CardTitle, Empty, SegmentedControl, Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@ui'
import { formatInt, formatPercent, formatTokens, type StatsGroup, type StatsSummary } from '../report-charts'
import { t } from '../../i18n'
import { TableFooter, useClientPaging } from '../table-shell'

export type SourceKind = 'clients' | 'apiKeys'
export type RequestDrilldown = { client?: string; apiKey?: string; startDate?: string; endDate?: string; model?: string }

export function sourceName(row: StatsGroup, kind: SourceKind): string {
  if (kind === 'clients') {
    if (!row.id) return t('Unrecorded client')
    return row.id === 'other' ? t('Other') : String(row.label || row.id)
  }
  if (row.source === 'none') return t('Unauthenticated calls')
  if (row.source === 'env') return t('Environment variable key')
  if (row.source !== 'key') return t('Key not recorded in history')
  return String(row.label || row.id || t('Untitled'))
}

export function openRequestLogs(query: RequestDrilldown): void {
  const app = window as unknown as {
    wbRequestsPanel?: { openFiltered?: (query: RequestDrilldown) => void }
  }
  app.wbRequestsPanel?.openFiltered?.(query)
}

function rate(value: number | null | undefined): string {
  return typeof value === 'number' && Number.isFinite(value) ? formatPercent(value) : '—'
}

function tokens(value: number | null | undefined): string {
  return typeof value === 'number' && Number.isFinite(value) ? formatTokens(value) : '—'
}

function KeyState({ row }: { row: StatsGroup }) {
  if (row.source !== 'key') return null
  const label = row.state === 'enabled' ? t('enabled') : row.state === 'disabled' ? t('Disabled') : t('Deleted')
  return <Badge shape='tag' variant={row.state === 'enabled' ? 'success' : 'outline'}>{label}</Badge>
}

export function SourceStatsPanel({ kind, rows, summary, onInspect }: {
  kind: SourceKind; rows: StatsGroup[]; summary: StatsSummary
  onInspect?: (row: StatsGroup) => void
}) {
  const [sort, setSort] = React.useState('tokens')
  const sorted = React.useMemo(() => [...rows].sort((left, right) => {
    const main = sort === 'requests' ? 'requests' : 'totalTokens'
    const other = sort === 'requests' ? 'totalTokens' : 'requests'
    return (Number(right[main]) || 0) - (Number(left[main]) || 0)
      || (Number(right[other]) || 0) - (Number(left[other]) || 0)
      || String(left.filterValue || left.id || '').localeCompare(String(right.filterValue || right.id || ''))
  }), [rows, sort])
  const paging = useClientPaging(sorted.length, `report-${kind}`)
  const list = paging.paged ? paging.slice(sorted) : sorted
  const total = Number(summary.overview?.tokens) || 0
  const isKey = kind === 'apiKeys'
  const queryOf = (row: StatsGroup): RequestDrilldown => ({
    startDate: summary.startDate, endDate: summary.endDate,
    ...(isKey ? { apiKey: row.filterValue || 'unknown' } : { client: row.id || '' }),
  })
  const sortOptions = React.useMemo(() => [
    { value: 'tokens', label: t('By tokens') }, { value: 'requests', label: t('By requests') },
  ], [])
  return (
    <Card render={<section />} className='report-card report-source-panel'>
      <CardHeader className='report-card-header'>
        <CardTitle>{isKey ? t('API key statistics') : t('Client statistics')}</CardTitle>
        <span className='report-card-meta'>{t('{n} items', { n: formatInt(rows.length) })}</span>
        <span className='tip-q' data-tip={isKey
          ? t('Production requests grouped by the gateway API key, excluding assistant chats and model tests. Disabling or deleting a key preserves its historical usage. Reports never include the secret value. Requests without a recorded identity are listed separately and cannot be attributed to a key.')
          : t('Statistics use per-request software identifiers and identification rules, independently of API keys. They do not count devices or users. Unconfirmed software is grouped under Other; rule changes do not reclassify historical requests.')} />
        <div className='head-actions'>
          <SegmentedControl options={sortOptions} value={sort} onValueChange={value => { setSort(value); paging.goto(1) }}
            aria-label={t('Statistics sort order')} />
        </div>
      </CardHeader>
      <CardContent className='report-card-content report-source-content'>
        <div className='report-source-table-wrap'>
          <Table container={false} className='report-source-table'>
            <TableHeader><TableRow>
              <TableHead>{isKey ? t('API Key') : t('Client')}</TableHead>
              <TableHead className='text-right'>{t('Requests')}</TableHead>
              <TableHead className='text-right'>{t('Success / failure')}</TableHead>
              <TableHead className='text-right'>{t('Success rate')}</TableHead>
              <TableHead className='text-right'>{t('Total tokens')}</TableHead>
              <TableHead className='text-right'>{t('Share')}</TableHead>
              <TableHead className='text-right'>{t('Input / output')}</TableHead>
              <TableHead className='text-right'>{t('Cache reads')}</TableHead>
              <TableHead className='text-right'>{t('Cache hit rate')}</TableHead>
              {isKey ? <TableHead>{t('Last call')}</TableHead> : null}
              <TableHead className='text-right'>{t('Actions')}</TableHead>
            </TableRow></TableHeader>
            <TableBody>
              {list.length ? list.map(row => (
                <TableRow key={row.filterValue || row.id || 'unknown'}>
                  <TableCell>
                    <div className='flex items-center gap-2'>
                      <Button variant='link' size='sm' className='max-w-[240px] truncate px-0' title={isKey && row.id ? `${sourceName(row, kind)}（${row.id}）` : sourceName(row, kind)}
                        onClick={() => openRequestLogs(queryOf(row))}>{sourceName(row, kind)}</Button>
                      {isKey ? <KeyState row={row} /> : null}
                    </div>
                  </TableCell>
                  <TableCell className='report-source-num'>{formatInt(row.requests)}</TableCell>
                  <TableCell className='report-source-num'>{formatInt(row.success)} / {formatInt(row.failures)}</TableCell>
                  <TableCell className='report-source-num'>{rate(row.successRate)}</TableCell>
                  <TableCell className='report-source-num' title={formatInt(row.totalTokens)}>{tokens(row.totalTokens)}</TableCell>
                  <TableCell className='report-source-num'>{total ? formatPercent((Number(row.totalTokens) || 0) / total) : '—'}</TableCell>
                  <TableCell className='report-source-num' title={row.tokenDetailsComplete === false ? t('Historical details are incomplete; input, output and cache readings cannot be recovered.') : undefined}>
                    {tokens(row.promptTokens)} / {tokens(row.completionTokens)}
                  </TableCell>
                  <TableCell className='report-source-num'>{tokens(row.cacheReadTokens)}</TableCell>
                  <TableCell className='report-source-num'>{rate(row.cacheRate)}</TableCell>
                  {isKey ? <TableCell className='whitespace-nowrap text-xs'>{row.lastUsedAt ? new Date(row.lastUsedAt).toLocaleString() : '—'}</TableCell> : null}
                  <TableCell className='text-right whitespace-nowrap'>
                    {isKey && onInspect ? <Button variant='ghost' size='sm' onClick={() => onInspect(row)}>{t('Usage details')}</Button> : null}
                    <Button variant='ghost' size='sm' onClick={() => openRequestLogs(queryOf(row))}>{t('Request logs')}</Button>
                  </TableCell>
                </TableRow>
              )) : <TableRow><TableCell colSpan={isKey ? 11 : 10}><Empty>{t('No requests in the selected range')}</Empty></TableCell></TableRow>}
            </TableBody>
          </Table>
        </div>
        <div className='report-source-note'>{t('Only completed production requests are counted. Failed requests still count as requests; token usage follows the existing successful-request policy.')}</div>
        <div className='report-source-note'>{t('Request logs show only retained details; clearing them does not reduce historical usage in daily reports.')}</div>
      </CardContent>
      <TableFooter total={rows.length} range={paging.paged ? { start: paging.rangeStart, end: paging.rangeEnd } : null}
        page={paging.page} pageCount={paging.pageCount} size={paging.size}
        onSizeChange={paging.setSize} onPageChange={paging.goto} />
    </Card>
  )
}
