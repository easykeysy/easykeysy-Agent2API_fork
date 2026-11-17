import * as React from 'react'
import { Card, CardContent, CardHeader, CardTitle, MetricCard, MetricGrid, MetricLabel, MetricValue } from '@ui'
import { cacheRateCells, formatInt, formatPercent, formatTokens, type StatsHour, type StatsSummary } from '../report-charts'
import { CacheTrendChart } from './report-recharts'
import { amount, sumUsage } from './usage-data'
import { t } from '../../i18n'

const TIP_CACHE = t('Hit rate = cache-read tokens / input tokens (promptTokens). The shorter the window, the closer it is to "right now": when an upstream reports cache reads separately from input, this ratio can legitimately exceed 100%; it is shown as-is, without clamping.')

export function CacheWindows({ summary, loading }: { summary: StatsSummary | null; loading: React.ReactNode }) {
  return <Card render={<section />} className='report-card report-cache-windows'>
    <CardHeader className='report-card-header'><CardTitle>{t('Cache hit rate')}</CardTitle><span className='tip-q' data-tip={TIP_CACHE} /></CardHeader>
    <CardContent className='report-cache-window-content'>
      <MetricGrid className='report-cache-window-grid' id='report-cache-rates'>
        {summary ? cacheRateCells(summary.cacheRates).map(cell => <MetricCard className='report-cache-window' key={cell.key}>
          <MetricLabel>{cell.label}</MetricLabel><MetricValue>{cell.value.endsWith('%') ? <>{cell.value.slice(0, -1)}<small>%</small></> : cell.value}</MetricValue><div className='report-cache-window-reading'>{cell.sub}</div>
        </MetricCard>) : loading}
      </MetricGrid>
    </CardContent>
  </Card>
}

export function CacheTrendPanel({ summary, loading, colors }: { summary: StatsSummary | null; loading: React.ReactNode; colors: Map<string, string> }) {
  const hours: StatsHour[] = summary?.cacheTrend24h || []
  const latest = hours[hours.length - 1]
  const totals = sumUsage(hours.map(hour => ({ ...hour, promptTokens: hour.inputTokens, cacheReadTokens: hour.hitTokens })))
  const input = amount(totals.promptTokens)
  const cached = amount(totals.cacheReadTokens)
  const hasRequests = hours.every(hour => typeof hour.requests === 'number')
  const cells = [
    { label: t('Latest hourly hit rate'), value: latest?.inputTokens ? formatPercent(latest.rate) : '—' },
    { label: t('24-hour overall hit rate'), value: input ? formatPercent(cached / input) : '—' },
    { label: t('24-hour total usage'), value: `${formatTokens(totals.totalTokens)} Token` },
    { label: t('24-hour requests'), value: hasRequests ? formatInt(totals.requests) : '—' },
  ]
  return <Card render={<section />} className='report-card report-cache-card'>
    <CardHeader className='report-card-header'><CardTitle>{t('Last 24 hours: cache hit rate trend')}</CardTitle>
      <span className='tip-q' data-tip={TIP_CACHE} /><span className='report-card-meta ml-auto'>{t('按本地整点统计')}</span>
    </CardHeader>
    <CardContent className='report-card-content'>
      {summary ? <div className='report-cache-inline-stats'>{cells.map(cell => <div key={cell.label}><span>{cell.label}</span><strong>{cell.value}</strong></div>)}</div> : null}
      <div className='chart-wrap' id='report-cache-trend'>{summary ? <CacheTrendChart series={hours} colors={colors} /> : loading}</div>
      <div className='report-chart-footer'>
        <div className='report-cache-legend'>
          <span><i className='report-rate-swatch' />{t('Cache hit rate')} · {t('Left axis')}</span>
          <span><i className='report-token-swatch' />{t('Total tokens')} · {t('Right axis')}</span>
        </div>
        <span>{t('Hover for full readings and a breakdown')}</span>
      </div>
    </CardContent>
  </Card>
}
