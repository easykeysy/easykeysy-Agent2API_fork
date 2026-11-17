import { useMemo, useState } from 'react'
import { Card, Empty, SegmentedControl } from '@ui'
import { SearchableSelect } from '../../../ui-shared/searchable-select'
import type { StatsSummary } from '../api/gateway'
import { formatInt, formatTokens } from '../lib/format'
import { reportSourceName } from './SourceUsageSection'
import { t } from '../i18n'

export function MobileReportTrend({ summary }: { summary: StatsSummary }) {
  const [dimension, setDimension] = useState<'models' | 'clients' | 'apiKeys'>('models')
  const [metric, setMetric] = useState<'tokens' | 'requests'>('tokens')
  const [selected, setSelected] = useState('__all__')
  const options = useMemo(() => [{ value: '__all__', label: t('All') }, ...(summary[dimension] || []).map(row => ({
    value: dimension === 'apiKeys' ? row.filterValue || 'unknown' : row.id || row.model || '__unknown__',
    label: dimension === 'models' ? row.label || row.model || t('Unknown model') : reportSourceName(row, dimension === 'apiKeys'),
  }))], [summary, dimension])
  const displayedOptions = options.some(option => option.value === selected) ? options : [{ value: selected, label: selected }, ...options]
  const trend = (summary.dailyTrend || []).map(day => ({
    date: day.date || '',
    value: selected === '__all__' ? Number(day[metric]) || 0 : (day[dimension] || []).reduce((sum, row) => {
      const id = row.id || row.model || '__unknown__'
      return sum + (id === selected ? Number(row[metric]) || 0 : 0)
    }, 0),
  }))
  const max = Math.max(...trend.map(row => row.value), 1)
  const width = 300
  const height = 84
  const gap = trend.length > 100 ? 0 : 2
  const barWidth = Math.max(0.1, (width - gap * (trend.length - 1)) / Math.max(1, trend.length))
  return <div className='m-section'><Card className='m-card p-3'>
    <b className='text-sm'>{metric === 'tokens' ? t('Daily token trend') : t('Daily request trend')}</b>
    <div className='flex flex-wrap gap-2 my-3'>
      <SegmentedControl value={dimension} aria-label={t('Trend dimension')} onValueChange={value => {
        setDimension(value as typeof dimension); setSelected('__all__')
      }} options={[{ value: 'models', label: t('Model') }, { value: 'clients', label: t('Client') }, { value: 'apiKeys', label: t('API Key') }]} />
      <SegmentedControl value={metric} aria-label={t('Trend metric')} onValueChange={value => setMetric(value as typeof metric)}
        options={[{ value: 'tokens', label: t('Token usage') }, { value: 'requests', label: t('Requests') }]} />
      <SearchableSelect value={selected} options={displayedOptions} onValueChange={setSelected}
        className='w-full' aria-label={t('Filter report items')}
        searchPlaceholder={dimension === 'models' ? t('Search models') : t('Search…')} />
    </div>
    {!trend.length ? <Empty>{t('No trend data')}</Empty> : (
      <><svg viewBox={`0 0 ${width} ${height}`} width='100%' height={height} role='img'
        aria-label={metric === 'tokens' ? t('Daily token trend') : t('Daily request trend')}>
        {trend.map((row, index) => {
          const size = row.value / max * height
          return <rect key={row.date} x={index * (barWidth + gap)} y={height - size} width={barWidth} height={size}
            rx={Math.min(3, barWidth / 2)} fill='var(--m-primary)' opacity={row.value === max ? 1 : 0.4}>
            <title>{row.date} · {metric === 'tokens' ? formatTokens(row.value) : formatInt(row.value)}</title>
          </rect>
        })}
      </svg><div className='flex justify-between text-xs text-muted-foreground mt-2'>
        <span>{trend[0]?.date}</span><span>{trend[trend.length - 1]?.date}</span>
      </div></>
    )}
  </Card></div>
}
