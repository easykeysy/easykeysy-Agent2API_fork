import * as React from 'react'
import {
  Button, Dialog, DialogBody, DialogContent, DialogFooter, DialogHeader, DialogTitle,
  Input, MetricCard, MultiSelect, Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
  Field, FieldGroup, FieldLabel,
} from '@ui'
import { t } from '../i18n'
import { numberText, type AccountRecord } from './accounts-shared'
import { providerOf } from './accounts-domain'
import type { LimiterCondition, LimiterMetric, LimiterWindow, QuotaSource } from './accounts-limiter-types'
import { metricLabel, modelsOf, sourceIdentity, sourceLabel, sourcesOf } from './accounts-limiter-model'
import { isCreditUnit, usageUnitLabel } from '../../../ui-shared/usage-readings'

const RESET_OPTIONS: Array<{ value: LimiterWindow['reset']; label: () => string }> = [
  { value: 'fixed', label: () => t('Fixed period') },
  { value: 'rolling', label: () => t('Rolling window') },
  { value: 'daily', label: () => t('Calendar day') },
  { value: 'weekly', label: () => t('Calendar week') },
  { value: 'monthly', label: () => t('Calendar month') },
  { value: 'total', label: () => t('Cumulative from activation to expiry (no automatic refill)') },
]

type Draft = {
  condition: LimiterCondition
  threshold: string
  unit: 'unit' | 'wan' | 'yi'
  period: string
  anchor: string
  end: string
}

export function blankCondition(): LimiterCondition {
  return { join: 'or', sourceId: 'local', models: [], metric: 'tokens', comparison: 'gte',
    threshold: 500_000, window: { reset: 'daily', utcOffsetMinutes: null } }
}

function dateInput(ms: number | undefined): string {
  if (!ms || !Number.isFinite(ms)) return ''
  const date = new Date(ms)
  return new Date(ms - date.getTimezoneOffset() * 60_000).toISOString().slice(0, 16)
}

function draftOf(condition: LimiterCondition): Draft {
  const unit = condition.metric === 'tokens' && condition.threshold % 100_000_000 === 0 ? 'yi'
    : condition.metric === 'tokens' && condition.threshold % 10_000 === 0 ? 'wan' : 'unit'
  return { condition: { ...structuredClone(condition), window: condition.window || condition.source?.window,
    sourceId: condition.sourceId || (condition.metric === 'tokens' || condition.metric === 'requests' ? 'local' : 'account') }, unit,
    threshold: String(condition.threshold / (unit === 'yi' ? 100_000_000 : unit === 'wan' ? 10_000 : 1)),
    period: String(((condition.window || condition.source?.window)?.seconds || 3600) / 3600),
    anchor: dateInput((condition.window || condition.source?.window)?.startsAt),
    end: dateInput((condition.window || condition.source?.window)?.endsAt),
  }
}

function metricsFor(source: QuotaSource | undefined): LimiterMetric[] {
  if (!source || source.id === 'local' || source.localOnly) return ['tokens', 'requests']
  if (source.id === 'account') return ['remaining']
  const result: LimiterMetric[] = []
  if (!source.unlimited) {
    if (source.remaining !== null && source.remaining !== undefined) result.push('remaining')
    if (source.used !== null && source.used !== undefined) result.push('used')
    if (source.remainingPercent !== null && source.remainingPercent !== undefined || (source.total || 0) > 0) {
      result.push('remaining_percent', 'used_percent')
    }
  }
  if (source.window) result.push('tokens', 'requests')
  return result.length ? result : ['tokens', 'requests']
}

export function LimiterConditionDialog({ account, value, allowBalance = true, onSave, onClose }: {
  account?: AccountRecord
  value: LimiterCondition
  allowBalance?: boolean
  onSave: (condition: LimiterCondition) => void
  onClose: () => void
}) {
  const [draft, setDraft] = React.useState(() => draftOf(value))
  const [error, setError] = React.useState('')
  const condition = draft.condition
  const sources = sourcesOf(account).filter(source => (allowBalance || source.id === 'local')
    && (providerOf(account) !== 'codearts' || source.id === 'account' || isCreditUnit(source.unit || '')))
  if (condition.source && !sources.some(source => source.id === condition.sourceId)) sources.push(condition.source)
  const source = sources.find(source => source.id === condition.sourceId)
  const aggregate = sources.find(source => source.id === 'account' && isCreditUnit(source.unit || ''))
  const plans = [...new Map(sources.filter(source => source.planId).map(source => [source.planId!, source.planName || source.planId!])).entries()]
  const plan = source?.planId || ''
  const options = sources.filter(source => source.id === 'account' || (plan ? source.planId === plan : !source.planId))
  const local = condition.metric === 'tokens' || condition.metric === 'requests'
  const convertedThreshold = Number(draft.threshold) * (draft.unit === 'yi' ? 100_000_000 : draft.unit === 'wan' ? 10_000 : 1)
  const thresholdPreview = condition.metric === 'tokens' && draft.unit !== 'unit' && draft.threshold.trim()
    && Number.isSafeInteger(convertedThreshold) && convertedThreshold > 0 ? convertedThreshold.toLocaleString('en-US') : ''
  const aggregateView = aggregate?.unlimited ? t('Unlimited') : aggregate?.remaining == null ? t('No reading available')
    : `${numberText(aggregate.remaining)} ${t('Points')}`
  const modelOptions = source?.models?.length
    ? source.models.map(model => ({ value: model, label: model })) : modelsOf(account)
  for (const model of condition.models) if (!modelOptions.some(option => option.value === model)) modelOptions.push({ value: model, label: model })
  const metrics = [...new Set([...metricsFor(source), condition.metric])]
    .filter(metric => providerOf(account) !== 'codearts' || metric !== 'tokens' && metric !== 'requests')
  const update = (next: Partial<LimiterCondition>) => {
    setError('')
    setDraft(current => ({ ...current, condition: { ...current.condition, ...next } }))
  }
  const selectSource = (id: string) => {
    const source = sources.find(source => source.id === id)
    if (!source) return
    const metric = source.localOnly || source.localEstimate ? 'tokens' : metricsFor(source)[0]
    const threshold = metric === 'tokens' ? source.total || 500_000 : metric.endsWith('percent') ? 5 : 1
    setDraft(draftOf({ ...condition, sourceId: id, source: sourceIdentity(source),
      models: source.models?.length === 1 ? source.models.slice() : [], metric,
      comparison: metric.startsWith('remaining') ? 'lte' : 'gte', threshold,
      window: source.window || { reset: 'daily', utcOffsetMinutes: null },
    }))
    setError('')
  }
  const save = () => {
    const raw = Number(draft.threshold)
    const threshold = condition.metric === 'tokens'
      ? Math.round(raw * (draft.unit === 'yi' ? 100_000_000 : draft.unit === 'wan' ? 10_000 : 1)) : raw
    if (!draft.threshold.trim() || !Number.isFinite(threshold) || threshold < 0) {
      setError(t('The threshold must be a non-negative finite number')); return
    }
    if (local && (!Number.isSafeInteger(threshold) || threshold <= 0)) {
      setError(t('The local usage limit must be a positive safe integer')); return
    }
    if (condition.metric.endsWith('percent') && threshold > 100) {
      setError(t('The percentage threshold must be between 0 and 100')); return
    }
    let window = condition.window
    if (local) {
      window = { ...window, reset: window?.reset || 'daily' }
      if (window.reset === 'fixed' || window.reset === 'rolling') {
        const seconds = Math.round(Number(draft.period) * 3600)
        if (!Number.isFinite(seconds) || seconds < 60 || seconds > 2_678_400) {
          setError(t('The counting window must be between 1 minute and 31 days')); return
        }
        window.seconds = seconds
      }
      if (window.reset === 'total' || window.reset === 'fixed') {
        window.startsAt = draft.anchor ? new Date(draft.anchor).getTime() : 0
        window.endsAt = draft.end ? new Date(draft.end).getTime() : 0
        if (!Number.isFinite(window.startsAt) || !Number.isFinite(window.endsAt)
          || window.endsAt > 0 && window.endsAt <= window.startsAt) {
          setError(t('The counting end time must be later than the start time')); return
        }
      }
    }
    onSave({ ...condition, threshold, window,
      source: source && source.id !== 'account' && source.id !== 'local' ? sourceIdentity(source) : null,
    })
  }

  return <Dialog open onOpenChange={next => { if (!next) onClose() }}>
    <DialogContent className='w-[min(680px,calc(100vw-48px))]' showCloseButton overlayForceRender>
      <DialogHeader><DialogTitle>{t('Edit limit condition')}</DialogTitle></DialogHeader>
      <DialogBody className='space-y-3'>
        {plans.length ? <Field orientation='horizontal' className='field-row'>
          <FieldLabel>{t('Plan')}</FieldLabel>
          <Select value={plan} onValueChange={id => {
            const next = sources.find(source => id ? source.planId === id : !source.planId)
            if (next) selectSource(next.id)
          }}>
            <SelectTrigger className='min-w-0 flex-1'><SelectValue>{plans.find(([id]) => id === plan)?.[1] || t('General quota / local protection')}</SelectValue></SelectTrigger>
            <SelectContent>
              <SelectItem value=''>{t('General quota / local protection')}</SelectItem>
              {plans.map(([id, label]) => <SelectItem key={id} value={id}>{label}</SelectItem>)}
            </SelectContent>
          </Select>
        </Field> : null}
        <Field orientation='horizontal' className='field-row'>
          <FieldLabel>{t('Quota source')}</FieldLabel>
          <Select value={condition.sourceId || 'account'} onValueChange={id => { if (id) selectSource(id) }}>
            <SelectTrigger className='min-w-0 flex-1'><SelectValue>{sourceLabel(source, condition.sourceId || 'account')}</SelectValue></SelectTrigger>
            <SelectContent>{options.map(source => <SelectItem key={source.id} value={source.id}>
              {sourceLabel(source)}
            </SelectItem>)}</SelectContent>
          </Select>
        </Field>
        {aggregate ? <MetricCard className='space-y-2'>
          <div className='flex flex-wrap items-center gap-2'>
            <span className='text-xs text-subtle'>{t('Total points')}</span>
            <span className='text-[13px] font-semibold tabular-nums'>{aggregateView}</span>
            <Button variant='outline' size='sm' className='ml-auto'
              disabled={source?.id === aggregate.id && condition.metric === 'remaining'}
              onClick={() => selectSource(aggregate.id)}>{t('Limit by total points')}</Button>
          </div>
          <p className='detail'>{t('Total points is the aggregate of currently available points, matching the balance column. Summary rows are not counted twice, and tokens, request counts, and quotas in other units are excluded.')}</p>
        </MetricCard> : null}
        <Field orientation='horizontal' className='field-row'>
          <FieldLabel>{t('Model scope')}</FieldLabel>
          <MultiSelect value={condition.models} options={modelOptions}
            onValueChange={models => update({ models })} placeholder={source?.models?.length ? t('All models in this quota') : t('All models')}
            className='min-w-0 flex-1' aria-label={t('Model scope')} />
        </Field>
        <Field orientation='horizontal' className='field-row'>
          <FieldLabel>{t('Limit metric')}</FieldLabel>
          <Select value={condition.metric} onValueChange={metric => {
            if (!metric) return
            update({ metric: metric as LimiterMetric, comparison: metric.startsWith('remaining') ? 'lte' : 'gte' })
            setDraft(current => ({ ...current, threshold: '', unit: 'unit' }))
          }}>
            <SelectTrigger className='min-w-[190px]'><SelectValue>{metricLabel(condition.metric)}</SelectValue></SelectTrigger>
            <SelectContent>{metrics.map(metric => <SelectItem key={metric} value={metric}>{metricLabel(metric)}</SelectItem>)}</SelectContent>
          </Select>
        </Field>
        <FieldGroup orientation='horizontal' className='field-row'>
          <span className='text-[12.5px] text-subtle'>{t('Condition')}</span>
          <Select value={condition.comparison} onValueChange={comparison => update({ comparison: comparison as LimiterCondition['comparison'] })}>
            <SelectTrigger className='w-[92px]' aria-label={t('Condition')}><SelectValue>{condition.comparison === 'lt' ? '<' : condition.comparison === 'lte' ? '≤' : '≥'}</SelectValue></SelectTrigger>
            <SelectContent><SelectItem value='lt'>{'<'}</SelectItem><SelectItem value='lte'>≤</SelectItem><SelectItem value='gte'>≥</SelectItem></SelectContent>
          </Select>
          <Input type='number' min={0} className='max-w-[160px]' value={draft.threshold}
            onChange={event => setDraft({ ...draft, threshold: event.currentTarget.value })} placeholder={t('Threshold')} aria-label={t('Threshold')} />
          {condition.metric === 'tokens' ? <Select value={draft.unit} onValueChange={unit => setDraft({ ...draft, unit: unit as Draft['unit'] })}>
            <SelectTrigger className='w-[84px]' aria-label={t('Token unit')}><SelectValue>{draft.unit === 'yi' ? t('100M') : draft.unit === 'wan' ? t('10k') : t('total')}</SelectValue></SelectTrigger>
            <SelectContent><SelectItem value='yi'>{t('100M')}</SelectItem><SelectItem value='wan'>{t('10k')}</SelectItem><SelectItem value='unit'>{t('total')}</SelectItem></SelectContent>
          </Select> : <span className='detail'>{condition.metric.endsWith('percent') ? '%' : condition.metric === 'requests' ? t('times') : usageUnitLabel(source?.unit || '', t)}</span>}
        </FieldGroup>
        {thresholdPreview ? <p className='detail'>{t('Actual threshold: {value} tokens', { value: thresholdPreview })}</p> : null}
        {local ? <>
          <Field orientation='horizontal' className='field-row'>
            <FieldLabel>{t('Counting period')}</FieldLabel>
            <Select value={condition.window?.reset || 'daily'} onValueChange={reset => update({ window: { ...condition.window, reset: reset as LimiterWindow['reset'] } })}>
              <SelectTrigger className='min-w-0 flex-1'><SelectValue>{RESET_OPTIONS.find(option => option.value === (condition.window?.reset || 'daily'))!.label()}</SelectValue></SelectTrigger>
              <SelectContent>{RESET_OPTIONS.map(option => <SelectItem key={option.value} value={option.value}>{option.label()}</SelectItem>)}</SelectContent>
            </Select>
          </Field>
          {condition.window?.reset === 'fixed' || condition.window?.reset === 'rolling' ? <Field orientation='horizontal' className='field-row'>
            <FieldLabel>{t('Window length')}</FieldLabel>
            <Input type='number' min={1 / 60} max={744} step='any' value={draft.period} className='max-w-[130px]'
              onChange={event => setDraft({ ...draft, period: event.currentTarget.value })} /><span className='detail'>{t('小时')}</span>
          </Field> : null}
          {condition.window?.reset === 'total' || condition.window?.reset === 'fixed' ? <>
            <Field orientation='horizontal' className='field-row'><FieldLabel>{t('Counting start')}</FieldLabel><Input type='datetime-local' value={draft.anchor}
              onChange={event => setDraft({ ...draft, anchor: event.currentTarget.value })} className='min-w-0 flex-1' /></Field>
            <Field orientation='horizontal' className='field-row'><FieldLabel>{t('Expiry time')}</FieldLabel><Input type='datetime-local' value={draft.end}
              onChange={event => setDraft({ ...draft, end: event.currentTarget.value })} className='min-w-0 flex-1' /></Field>
          </> : null}
          {['daily', 'weekly', 'monthly'].includes(condition.window?.reset || 'daily') ? <Field orientation='horizontal' className='field-row'>
            <FieldLabel>{t('Reset time zone')}</FieldLabel>
            <Select value={condition.window?.utcOffsetMinutes === 480 ? 'beijing' : condition.window?.utcOffsetMinutes === 0 ? 'utc' : 'local'}
              onValueChange={zone => update({ window: { ...condition.window, reset: condition.window?.reset || 'daily', utcOffsetMinutes: zone === 'beijing' ? 480 : zone === 'utc' ? 0 : null } })}>
              <SelectTrigger className='min-w-[190px]'><SelectValue>{condition.window?.utcOffsetMinutes === 480 ? t('Beijing time (UTC+8)') : condition.window?.utcOffsetMinutes === 0 ? 'UTC' : t('Server local time zone')}</SelectValue></SelectTrigger>
              <SelectContent><SelectItem value='local'>{t('Server local time zone')}</SelectItem><SelectItem value='beijing'>{t('Beijing time (UTC+8)')}</SelectItem><SelectItem value='utc'>UTC</SelectItem></SelectContent>
            </Select>
          </Field> : null}
        </> : null}
        <p className='detail'>{local
          ? t('Local counters include only requests through this gateway, not the upstream bill. Clearing request logs does not reset them. Selecting a plan scopes protection; it does not force the upstream to charge that plan.')
          : t('Evaluated using the upstream quota reading. Selecting models does not split the balance of a shared quota pool.')}</p>
        {local && modelOptions.length > 0 ? <p className='detail'>{t('When multiple models are selected, usage and thresholds are evaluated separately for each model. Usage is not combined.')}</p> : null}
        {source?.localEstimate ? <p className='detail text-warning'>{t('The upstream does not provide usage for this quota bucket, so protection uses a local estimate. Usage in other clients is not counted.')}</p> : null}
        {source?.channel && source.channel !== String(account?.quotaChannel || account?.zcodePlan || '') ? <p className='detail text-warning'>
          {t('This quota does not belong to the account’s current channel. It is evaluated only after switching to its channel.')}
        </p> : null}
        {error ? <p className='detail text-destructive'>{error}</p> : null}
      </DialogBody>
      <DialogFooter><Button variant='outline' onClick={onClose}>{t('Cancel')}</Button><Button onClick={save}>{t('Save condition')}</Button></DialogFooter>
    </DialogContent>
  </Dialog>
}
