import * as React from 'react'
import { Card, CardContent, CardHeader, CardTitle, Empty } from '@ui'
import { formatInt, formatTokens, heatmapView, heatLegendItems, heatThresholdsOf, type StatsSummary } from '../report-charts'
import { useWidth } from './use-width'
import { t } from '../../i18n'

export function ActivityPanel({ summary, loading }: { summary: StatsSummary | null; loading: React.ReactNode }) {
  const { ref, width } = useWidth()
  const days = summary?.heatmap || []
  const view = heatmapView(days, width)
  const active = days.filter(day => (day.requests || 0) > 0).length
  const tokens = days.reduce((sum, day) => sum + (day.tokens || 0), 0)
  return <Card render={<section />} className='report-card report-activity-card'>
    <CardHeader className='report-card-header'><CardTitle>{t('Activity heatmap')}</CardTitle>
      <span className='report-card-meta'>{t('Last 365 days · independent of the date filter')}</span>
    </CardHeader>
    <CardContent className='report-card-content'>
      {summary ? <div className='report-activity-summary'>
        <span><strong>{formatInt(active)}</strong> {t('active days')}</span>
        <span>{t('Current streak')} <strong>{formatInt(summary.overview?.streak)}</strong></span>
        <span>{t('Total')} <strong>{formatTokens(tokens)}</strong> Token</span>
      </div> : null}
      <div className='heat-wrap' id='report-heatmap' ref={ref}>
        {!summary ? loading : view ? <svg className='report-svg' width={view.width} height={view.height}
          viewBox={`0 0 ${view.width} ${view.height}`} role='img' aria-label={t('Last 365 days: activity heatmap')}>
          {view.months.map(item => <text className='hm-axis' x={item.x} y={10} key={item.key}>{item.text}</text>)}
          {view.weekdays.map(item => <text className='hm-axis' x={item.x} y={item.y} textAnchor='end' dominantBaseline='middle' key={item.key}>{item.text}</text>)}
          {view.cells.map(cell => <rect className={`hm-cell l${cell.level}`} x={cell.x} y={cell.y} width={cell.size}
            height={cell.size} rx={3} tabIndex={-1} data-tip={cell.tip} key={cell.key} />)}
        </svg> : <Empty>{t('No heatmap data')}</Empty>}
      </div>
      {summary ? <div className='report-activity-footer'>
        <span>{days[0]?.date} — {days[days.length - 1]?.date}</span>
        <div className='heat-legend' id='report-heat-legend'><span>Token</span>{heatLegendItems(heatThresholdsOf(days)).map(item =>
          <span className='heat-legend-item' key={item.level}><i className={`l${item.level}`} />{item.text}</span>)}</div>
      </div> : null}
    </CardContent>
  </Card>
}
