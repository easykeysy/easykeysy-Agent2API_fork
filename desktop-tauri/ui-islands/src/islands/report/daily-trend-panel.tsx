import * as React from 'react'
import { Card, CardContent, CardHeader, CardTitle, SegmentedControl } from '@ui'
import { SearchableSelect } from '../../../../ui-shared/searchable-select'
import { DailyTrendChart } from './report-recharts'
import { sourceName } from './source-stats'
import { formatInt, formatPercent, formatTokens, SLICE_COLORS, type StatsDay, type StatsGroup, type StatsSummary } from '../report-charts'
import { t } from '../../i18n'
import { amount, sumUsage } from './usage-data'
import { CompactValue } from './compact-value'

export type TrendDimension = 'models' | 'clients' | 'apiKeys'
export type TrendMetric = 'tokens' | 'requests'

const DIMENSIONS = [
  { value: 'models', label: t('Model') }, { value: 'clients', label: t('Client') }, { value: 'apiKeys', label: t('API Key') },
]
const METRICS = [{ value: 'tokens', label: t('Token usage') }, { value: 'requests', label: t('Requests') }]
const TYPES = [{ value: 'bar', label: t('Bars') }, { value: 'line', label: t('Line') }]

export function ReportDailyTrend({ summary, loading }: { summary: StatsSummary | null; loading: React.ReactNode }) {
  const [dimension, setDimension] = React.useState<TrendDimension>('models')
  const [metric, setMetric] = React.useState<TrendMetric>('tokens')
  const [type, setType] = React.useState<'bar' | 'line'>('bar')
  const [selected, setSelected] = React.useState('__all__')
  const { series, colors, options } = React.useMemo(() => {
    const group: StatsGroup[] = summary?.[dimension] || []
    const names = new Map<string, string>()
    const groupId = (row: StatsGroup) => String(dimension === 'apiKeys' ? row.filterValue || 'unknown' : row.id || row.model || '__unknown__')
    for (const row of group) names.set(groupId(row), dimension === 'models'
      ? String(row.label || row.model || t('Unknown model')) : sourceName(row, dimension))
    if (dimension === 'apiKeys') {
      const counts = new Map<string, number>()
      for (const name of names.values()) counts.set(name, (counts.get(name) || 0) + 1)
      for (const [id, name] of names) if ((counts.get(name) || 0) > 1) names.set(id, `${name} · ${id.replace(/^key:/, '')}`)
    }
    const totals = new Map<string, number>()
    const series: StatsDay[] = (summary?.dailyTrend || []).map(day => {
      const items = (day[dimension] || []).map(item => {
        const id = String(item.id || item.model || '__unknown__')
        const label = names.get(id) || String(item.label || id || t('Unrecorded'))
        if (!names.has(id)) names.set(id, label)
        totals.set(id, (totals.get(id) || 0) + amount(item.tokens))
        return { ...item, id, label, totalTokens: amount(item.tokens) }
      }).filter(item => selected === '__all__' || item.id === selected)
      if (selected === '__all__') return { ...day, models: items }
      const usage = sumUsage(items)
      return { ...day, ...usage, tokens: usage.totalTokens, models: items }
    })
    const colors = new Map<string, string>()
    const order = [...group].sort((left, right) => amount(right.totalTokens ?? right.tokens) - amount(left.totalTokens ?? left.tokens))
    for (const row of order) {
      const name = names.get(groupId(row))
      if (name && !colors.has(name)) colors.set(name, SLICE_COLORS[colors.size % SLICE_COLORS.length])
    }
    for (const [id] of [...totals.entries()].sort((left, right) => right[1] - left[1])) {
      const name = names.get(id) || id
      if (!colors.has(name)) colors.set(name, SLICE_COLORS[colors.size % SLICE_COLORS.length])
    }
    const options = [{ value: '__all__', label: t('All') }, ...[...names].map(([value, label]) => ({ value, label }))]
    if (!options.some(option => option.value === selected)) options.push({ value: selected, label: selected })
    return { series, colors, options }
  }, [summary, dimension, selected])
  const totals = sumUsage(series.map(day => ({ ...day, totalTokens: amount(day.tokens) })))
  const total = amount(metric === 'tokens' ? totals.totalTokens : totals.requests)
  const average = Math.round(total / Math.max(series.length, 1))
  const unknown = dimension === 'models' ? t('Unknown model') : dimension === 'clients' ? t('Unrecorded client') : t('Key not recorded in history')
  const breakdownLabel = dimension === 'models' ? t('Model breakdown') : dimension === 'clients' ? t('Client breakdown') : t('API key breakdown')
  return <Card render={<section />} className='report-card report-daily-card'>
    <CardHeader className='report-card-header'>
      <CardTitle>{t('Usage trend')}</CardTitle><span className='report-card-meta'>{t('Grouped by day')}</span>
      <div className='report-header-actions'>
        <SegmentedControl options={METRICS} value={metric} onValueChange={value => setMetric(value as TrendMetric)} aria-label={t('Trend metric')} />
        <SegmentedControl options={TYPES} value={type} onValueChange={value => setType(value as 'bar' | 'line')} aria-label={t('Chart type')} />
      </div>
    </CardHeader>
    <CardContent className='report-card-content'>
      <div className='report-trend-filters'>
        <SegmentedControl options={DIMENSIONS} value={dimension} onValueChange={value => { setDimension(value as TrendDimension); setSelected('__all__') }} aria-label={t('Trend dimension')} />
        <SearchableSelect value={selected} options={options} onValueChange={setSelected}
          className='report-trend-select' aria-label={t('Filter report items')}
          searchPlaceholder={dimension === 'models' ? t('Search models') : t('Search…')} />
      </div>
      {summary ? <div className='report-chart-reading'>
        <div><strong><CompactValue value={total} /></strong><span>{metric === 'tokens' ? t('Token') : t('Requests')}</span></div>
        <span>{t('Daily average {value}', { value: metric === 'tokens' ? `${formatTokens(average)} Token` : formatInt(average) })} · {t('Success rate {rate}', { rate: totals.successRate == null ? '—' : formatPercent(totals.successRate) })}</span>
      </div> : null}
      <div className='chart-wrap' id='report-daily-trend'>
        {summary ? <DailyTrendChart series={series} colors={colors} metric={metric} type={type} unknownLabel={unknown} breakdownLabel={breakdownLabel} /> : loading}
      </div>
    </CardContent>
  </Card>
}
