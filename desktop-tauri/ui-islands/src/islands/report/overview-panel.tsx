import * as React from 'react'
import { Card, MetricCard, MetricGrid, MetricLabel, MetricValue } from '@ui'
import { formatInt, formatPercent, formatTokens, type StatsSummary } from '../report-charts'
import { t } from '../../i18n'
import { measured, sumUsage } from './usage-data'
import { CompactValue } from './compact-value'

export function ReportOverview({ summary, loading }: { summary: StatsSummary | null; loading: React.ReactNode }) {
  const overview = summary?.overview
  const fallback = sumUsage((summary?.clients || []).map(row => ({ ...row })))
  const sourceComplete = summary?.clients && summary.clients.reduce((sum, row) => sum + (row.requests || 0), 0) === overview?.requests
  const cached = measured(overview?.cacheReadTokens) ?? (sourceComplete ? measured(fallback.cacheReadTokens) : null)
  const input = measured(overview?.promptTokens) ?? (sourceComplete ? measured(fallback.promptTokens) : null)
  const rate = input && cached !== null ? formatPercent(cached / input) : '—'
  const requests = overview?.requests || 0
  const success = overview?.successful || 0
  const cells = [
    { key: 'tokens', label: t('Total tokens'), value: <CompactValue value={overview?.tokens} />, sub: t('Input + output') },
    { key: 'requests', label: t('Total requests'), value: <CompactValue value={requests} />, sub: t('{n} requests', { n: formatInt(requests) }) },
    { key: 'success', label: t('Successful requests'), value: <CompactValue value={success} />, sub: t('成功率 {rate} · 失败 {n} 次', { rate: requests ? formatPercent(success / requests) : '—', n: formatInt(Math.max(0, requests - success)) }) },
    { key: 'cache', label: t('Cache reads'), value: cached === null ? '—' : <CompactValue value={cached} />, sub: cached === null ? t('Historical details are incomplete') : t('{rate} of input tokens', { rate }) },
  ]
  return <Card render={<section />} className='report-overview-block' aria-label={t('Statistics overview')}>
    <MetricGrid className='report-overview-grid' id='report-overview'>
      {summary ? cells.map(cell => <MetricCard className='report-stat' key={cell.key}>
        <MetricLabel>{cell.label}</MetricLabel><MetricValue>{cell.value}</MetricValue><div className='report-stat-caption'>{cell.sub}</div>
      </MetricCard>) : loading}
    </MetricGrid>
    {summary ? <div className='report-overview-secondary'>
      <span>{t('Active days')} <strong>{formatInt(overview?.activeDays)}</strong><small>/ {formatInt(summary.dailyTrend?.length || 0)}</small></span>
      <span>{t('Current streak')} <strong>{formatInt(overview?.streak)}</strong></span>
      <span className='report-top-model'>{t('Top models')} <strong title={overview?.topModel?.model}>{overview?.topModel?.model || '—'}</strong>
        {overview?.topModel ? <small>{formatTokens(overview.topModel.tokens)} · {formatPercent(overview.topModel.percentage)}</small> : null}</span>
    </div> : null}
  </Card>
}
