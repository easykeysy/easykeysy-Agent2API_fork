import { t } from '../i18n'
import type { AccountRecord, LimiterRule } from './accounts-shared'
import { isCreditUnit, usageUnitLabel } from '../../../ui-shared/usage-readings'
import type { LimiterCondition, LimiterGroup, LimiterGroupState, LimiterReading, LimiterSnapshot, LimiterWindow, QuotaSource } from './accounts-limiter-types'

let snapshot: LimiterSnapshot = {}

export function adoptLimiterSnapshot(next: LimiterSnapshot | null | undefined): boolean {
  if (!next?.limiterStates) return false
  const changed = JSON.stringify([snapshot.limiterStates, snapshot.quotaSources, snapshot.limiterModels])
    !== JSON.stringify([next.limiterStates, next.quotaSources, next.limiterModels])
  snapshot = { tokenAt: next.tokenAt, limiterStates: next.limiterStates, quotaSources: next.quotaSources, limiterModels: next.limiterModels }
  return changed
}

export function groupsOf(account: AccountRecord | null | undefined, legacy: LimiterRule[] = []): LimiterGroup[] {
  if (Array.isArray(account?.limiterGroups)) return account.limiterGroups
  return legacy.map(rule => ({
    enabled: rule.enabled !== false, action: rule.action, target: 'account',
    conditions: [{ join: 'or', sourceId: '', models: [], threshold: rule.threshold,
      metric: rule.type === 'balance' ? 'remaining' : 'tokens',
      comparison: rule.type === 'balance' ? 'lt' : 'gte',
      ...(rule.type === 'token' ? { window: { reset: rule.reset === 'daily' ? 'daily' : 'fixed', seconds: rule.period || 0 } as LimiterWindow } : {}),
    }],
  }))
}

export function sourcesOf(account: AccountRecord | null | undefined): QuotaSource[] {
  const sources = account ? snapshot.quotaSources?.[account.id] : undefined
  return Array.isArray(sources) ? sources.slice() : [
    { id: 'account', label: t('Account primary quota'), provider: String(account?.provider || '') },
    { id: 'local', label: t('Local usage protection'), provider: String(account?.provider || ''), localOnly: true },
  ]
}

export function modelsOf(account: AccountRecord | null | undefined): Array<{ value: string; label: string }> {
  const provider = String(account?.provider || '')
  const models = snapshot.limiterModels?.[provider] || []
  return models.map(model => ({ value: model.id, label: model.name && model.name !== model.id ? `${model.name}（${model.id}）` : model.id }))
}

export function statesOf(account: AccountRecord | null | undefined): LimiterGroupState[] {
  return account ? snapshot.limiterStates?.[account.id] || [] : []
}

export function limiterHit(account: AccountRecord | null | undefined, metric?: 'balance' | 'token', action = 'skip'): boolean {
  if (!account) return false
  const groups = groupsOf(account)
  const states = statesOf(account)
  return groups.some((group, index) => group.enabled && group.action === action && states[index]?.hit
    && (!metric || group.conditions.some(condition => metric === 'token'
      ? condition.metric === 'tokens' || condition.metric === 'requests'
      : condition.metric !== 'tokens' && condition.metric !== 'requests')))
}

export function sourceLabel(source: QuotaSource | null | undefined, id = source?.id || 'account'): string {
  if (id === 'local') return t('Local usage protection')
  if (id === 'account' || !id) return isCreditUnit(source?.unit || '') ? t('Total points') : t('Account primary quota')
  return source?.label ? t(source.label) : t('Account primary quota')
}

export function sourceIdentity(source: QuotaSource): QuotaSource {
  const { remaining: _remaining, total: _total, used: _used, remainingPercent: _percent, resetsAt: _reset, ...identity } = source
  return identity
}

export function groupHit(conditions: LimiterCondition[], readings: Array<{ hit: boolean }>): boolean {
  let any = false
  let term = true
  conditions.forEach((condition, index) => {
    if (index > 0 && condition.join === 'or') { any ||= term; term = true }
    term &&= readings[index]?.hit === true
  })
  return conditions.length > 0 && (any || term)
}

export function conditionText(condition: LimiterCondition, account?: AccountRecord): string {
  const id = condition.sourceId || (condition.metric === 'tokens' || condition.metric === 'requests' ? 'local' : 'account')
  const current = sourcesOf(account).find(source => source.id === id)
  const source = sourceLabel(current || condition.source, id)
  const models = condition.models.length ? condition.models.join(', ') : t('All models')
  const metric = metricLabel(condition.metric)
  const sign = condition.comparison === 'lt' ? '<' : condition.comparison === 'lte' ? '≤' : '≥'
  const unit = condition.metric.endsWith('percent') ? '%' : condition.metric === 'tokens' ? 'Token'
    : condition.metric === 'requests' ? t('times') : usageUnitLabel(current?.unit || condition.source?.unit || '', t)
  const quota = condition.metric === 'tokens' ? tokenCount(condition.threshold) : String(condition.threshold)
  return `${source} · ${models} · ${metric} ${sign} ${quota} ${unit}`.trim()
}

export function metricLabel(metric: LimiterCondition['metric']): string {
  switch (metric) {
    case 'remaining': return t('Remaining quota')
    case 'remaining_percent': return t('Remaining percentage')
    case 'used': return t('Used quota')
    case 'used_percent': return t('Used percentage')
    case 'tokens': return t('Local token usage')
    case 'requests': return t('Local request count')
  }
}

export function tokenCount(value: number): string {
  const api = (window as unknown as { wbUnits?: { formatTokens?: (value: number) => string } }).wbUnits
  return api?.formatTokens?.(value) || value.toLocaleString()
}

export function readingText(reading: LimiterReading | undefined, condition: LimiterCondition, account?: AccountRecord): string {
  if (!reading?.known || reading.value === null) return readingReason(reading?.reason)
  const source = sourcesOf(account).find(source => source.id === (condition.sourceId || 'account'))
  const unit = usageUnitLabel(source?.unit || condition.source?.unit || '', t)
  const format = (value: number) => condition.metric === 'tokens' ? `${tokenCount(value)} Token`
    : condition.metric.endsWith('percent') ? `${Number(value.toFixed(2))}%`
    : `${Number(value.toFixed(2))} ${condition.metric === 'requests' ? t('times') : unit}`.trim()
  const models = Object.entries(reading.modelValues || {})
  return models.length ? models.map(([model, value]) => `${model}：${format(value)}`).join(' · ') : format(reading.value)
}

function readingReason(reason: string | undefined): string {
  switch (reason) {
    case 'This quota does not belong to the account’s current channel': return t('This quota does not belong to the account’s current channel')
    case 'The plan is not active yet or has expired': return t('The plan is not active yet or has expired')
    case 'The upstream quota is unlimited': return t('The upstream quota is unlimited')
    case 'The counting period has not started or has ended': return t('The counting period has not started or has ended')
    default: return t('No reading available')
  }
}

export function expressionText(conditions: LimiterCondition[], account?: AccountRecord): string {
  const terms: string[][] = []
  conditions.forEach((condition, index) => {
    if (index === 0 || condition.join === 'or') terms.push([])
    terms[terms.length - 1].push(conditionText(condition, account))
  })
  return terms.map(term => term.length > 1 ? `（${term.join(` ${t(' and ')} `)}）` : term[0])
    .join(` ${t('OR')} `)
}
