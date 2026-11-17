import * as React from 'react'
import {
  Button, Card, CardContent, CardHeader, CardTitle, Empty, Label,
  Popover, PopoverContent, PopoverTrigger, Select, SelectContent, SelectItem, SelectTrigger, SelectValue, Switch,
} from '@ui'
import { DEFAULT_DONUT_SETTINGS, DONUT_THRESHOLDS, donutView, formatInt, type DonutSettings, type DonutView, type StatsGroup } from '../report-charts'
import { UsageDonut } from './report-recharts'
import { t } from '../../i18n'

const SETTINGS_KEY = 'workbuddy-desktop-report-donut-settings'
const TIP_GROUPING = t('Grouping affects only the donuts. Slices below the threshold are grouped as Other; every item remains in the list, with totals and shares unchanged.')
const TIP_MODELS = t('Sorted by token usage, with each slice\'s angle showing its share. Slices of at least 3% show a percentage; full readings for every item remain in the list and hover details.')
const TIP_PROVIDERS = t('How many tokens each provider used in the selected range, sorted by usage from high to low; each sector\'s angle is its share. The same model name may be offered by several providers; the Gateway tries them in priority order as configured in the Forwarding route setting, so the share also reflects where requests actually landed. The center shows the total for this dimension (equal to Total tokens in the overview), and each row on the right gives that provider\'s token usage and percentage. "Unknown" means those requests did not record the provider that served them—mainly detail rows from before provider recording existed, and requests that failed before an upstream was chosen.')

function readSettings(): DonutSettings {
  try {
    const saved: unknown = JSON.parse(localStorage.getItem(SETTINGS_KEY) || 'null')
    if (!saved || typeof saved !== 'object') return DEFAULT_DONUT_SETTINGS
    const record = saved as Record<string, unknown>
    return {
      enabled: typeof record.enabled === 'boolean' ? record.enabled : DEFAULT_DONUT_SETTINGS.enabled,
      threshold: DONUT_THRESHOLDS.some(value => value === record.threshold) ? record.threshold as number : DEFAULT_DONUT_SETTINGS.threshold,
    }
  } catch { return DEFAULT_DONUT_SETTINGS }
}

function DonutPanel({ panelId, listId, title, tip, view, ariaLabel, itemLabel, countText, emptyText }: {
  panelId: string; listId: string; title: string; tip: string; ariaLabel: string; itemLabel: string
  view: DonutView | null; countText: string; emptyText: string
}) {
  return <Card render={<section />} className='report-card report-donut-card' id={panelId}>
    <CardHeader className='report-card-header'>
      <CardTitle>{title}</CardTitle><span className='tip-q' data-tip={`${tip}\n\n${TIP_GROUPING}`} />
      <span className='report-card-meta ml-auto'>{countText}</span>
    </CardHeader>
    <CardContent className='report-donut-content'><div id={listId}>
      {view ? <UsageDonut view={view} ariaLabel={ariaLabel} itemLabel={itemLabel} /> : <Empty>{emptyText}</Empty>}
    </div></CardContent>
  </Card>
}

export function ReportUsageDonuts({ models, providers }: { models: StatsGroup[] | null; providers: StatsGroup[] | null }) {
  const [settings, setSettings] = React.useState<DonutSettings>(readSettings)
  const switchId = React.useId()
  const thresholdId = React.useId()
  const changeSettings = (patch: Partial<DonutSettings>) => {
    setSettings(previous => {
      const next = { ...previous, ...patch }
      try { localStorage.setItem(SETTINGS_KEY, JSON.stringify(next)) } catch { /* 本次会话照常生效。 */ }
      return next
    })
  }
  const modelView = donutView(models || undefined, t('Unknown model'), settings)
  const providerView = donutView(providers || undefined, t('Unknown'), settings)
  if (!models && !providers) return null
  return <section className='report-donut-section' aria-label={t('Usage breakdown')}>
    <div className='report-section-heading'>
      <div><h2>{t('Usage breakdown')}</h2><span>{t('By tokens')}</span></div>
      <Popover><PopoverTrigger render={<Button variant='ghost' size='sm' className='report-donut-settings-button' />}>
        <svg width='14' height='14' viewBox='0 0 16 16' fill='none' aria-hidden='true'><path d='M2 4h12M2 12h12M6 2v4M10 10v4' stroke='currentColor' strokeWidth='1.5' strokeLinecap='round' /></svg>
        {settings.enabled ? t('Group donut slices below {threshold}%', { threshold: settings.threshold }) : t('Show all donut slices')}
      </PopoverTrigger><PopoverContent align='end' className='report-donut-settings'>
        <div className='report-setting-title'>{t('Donut display')}</div>
        <p>{t('Only the two donuts change; the lists always show every item')}</p>
        <div className='report-setting-row'><Label htmlFor={switchId}>{t('Group small slices')}</Label>
          <Switch id={switchId} checked={settings.enabled} onCheckedChange={enabled => changeSettings({ enabled })} /></div>
        <div className='report-setting-row'><Label htmlFor={thresholdId}>{t('Group shares below')}</Label>
          <Select value={String(settings.threshold)} disabled={!settings.enabled} onValueChange={value => {
            const threshold = Number(value)
            if (DONUT_THRESHOLDS.some(option => option === threshold)) changeSettings({ threshold })
          }}>
            <SelectTrigger id={thresholdId} className='w-[96px]' aria-label={t('Small-slice grouping threshold')}><SelectValue>{`${settings.threshold}%`}</SelectValue></SelectTrigger>
            <SelectContent>{DONUT_THRESHOLDS.map(value => <SelectItem key={value} value={String(value)}>{`${value}%`}</SelectItem>)}</SelectContent>
          </Select>
        </div>
      </PopoverContent></Popover>
    </div>
    <div className='report-donut-grid'>
      {models ? <DonutPanel panelId='report-models-panel' listId='report-models-donut'
        title={t('Model usage')} tip={TIP_MODELS} ariaLabel={t('Model usage share')} itemLabel={t('Model')} view={modelView}
        countText={t('{n} models', { n: formatInt(modelView?.count || 0) })} emptyText={t('No model usage in the selected range')} /> : null}
      {providers ? <DonutPanel panelId='report-providers-pie-panel' listId='report-providers-donut'
        title={t('Provider usage')} tip={TIP_PROVIDERS} ariaLabel={t('Provider usage share')} itemLabel={t('Provider')} view={providerView}
        countText={t('{n} providers', { n: formatInt(providerView?.count || 0) })} emptyText={t('No provider usage in the selected range')} /> : null}
    </div>
    <p className='report-distribution-note'>{TIP_GROUPING}</p>
  </section>
}
