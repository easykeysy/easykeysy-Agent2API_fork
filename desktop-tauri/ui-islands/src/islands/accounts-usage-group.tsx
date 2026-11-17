import * as React from 'react'
import { t } from '../i18n'
import { numberText, type AccountRecord } from './accounts-shared'
import { supportsCreditDetail } from './accounts-domain'
import { usageEntryOf, usageFailureOf } from './accounts-usage-cache'
import { usageWindows, type UsageWindowReading } from './accounts-usage-windows'
import {
  creditExpirySummaryOf, creditExpiryText, expiringCreditsOf,
  type CreditExpirySummary, type ExpiringCredit,
} from '../../../ui-shared/credit-expiry'
import { usageNumberOrNull, usageUnitLabel, usageWarningsOf } from '../../../ui-shared/usage-readings'

type GroupRatioReading = {
  label: string
  percent: number | null
  unlimited: boolean
}

type GroupRatioSummary = {
  percent: number
  hasShortWindow: boolean
  adjusted: boolean
  windows: GroupRatioReading[]
}

type GroupBalance = {
  counted: number
  total: number
  sum: number | null
  unlimited: number
  unit: string
  failed: number
  uncountable: number
  pending: number
  partial: number
  scope: 'group' | 'filtered'
  additive: boolean
  ratio: GroupRatioSummary | null
  expiring: CreditExpirySummary
}

function isShortWindow(row: UsageWindowReading): boolean {
  return row.windowCode === 'daily' || row.windowCode === '5h'
}

function groupRatioOf(windows: UsageWindowReading[], rawValue: number | null): GroupRatioSummary | null {
  const measured = windows.filter(row => !row.aside)
  const exhaustedGroups = new Set(measured.filter(row =>
    (row.windowCode === 'weekly' || row.windowCode === 'monthly') && !row.unlimited
      && ((row.percent !== null && row.percent <= 0) || (row.remaining !== null && row.remaining <= 0)),
  ).map(row => row.group))
  let adjusted = false
  const readings = measured.map(row => {
    const shortWindow = isShortWindow(row)
    // 周/月限制只约束同一模型组；修正汇总副本，不改写账号明细的原始读数。
    const blocked = shortWindow && exhaustedGroups.has(row.group)
    if (blocked && (row.unlimited || row.percent !== 0)) adjusted = true
    return {
      label: row.group ? `${row.group} · ${row.window}` : row.window,
      percent: blocked ? 0 : row.percent,
      unlimited: !blocked && row.unlimited,
      shortWindow,
    }
  })
  const shortReadings = readings.filter(row => row.shortWindow && (row.percent !== null || row.unlimited))
  const candidates = (shortReadings.length ? shortReadings : readings)
    .flatMap(row => row.unlimited ? [100] : row.percent === null ? [] : [row.percent])
  if (candidates.length) {
    return {
      percent: Math.min(...candidates),
      hasShortWindow: shortReadings.length > 0,
      adjusted,
      windows: readings,
    }
  }
  if (rawValue === null) return null
  const percent = Math.max(0, Math.min(100, rawValue))
  return { percent, hasShortWindow: false, adjusted: false, windows: [{ label: '', percent, unlimited: false }] }
}

/** 默认统计整组，筛选口径由调用方决定；禁用账号仍保有额度，分页不改变统计范围。 */
export function groupBalanceOf(members: AccountRecord[], scope: 'group' | 'filtered' = 'group', now = Date.now()): GroupBalance {
  const out: GroupBalance = {
    counted: 0, total: members.length, sum: null, unlimited: 0, unit: '',
    failed: 0, uncountable: 0, pending: 0, partial: 0, scope,
    additive: true, ratio: null, expiring: creditExpirySummaryOf([], now),
  }
  const expiring: ExpiringCredit[] = []
  let sum = 0
  let mode: 'additive' | 'ratio' | null = null
  for (const account of members) {
    const entry = usageEntryOf(account)
    if (entry === undefined || entry === null) { out.pending++; continue }
    const failure = usageFailureOf(entry)
    if (failure) {
      if (!failure.notConfigured) out.failed++
      continue
    }
    if (typeof entry !== 'object' || Array.isArray(entry)) { out.uncountable++; continue }
    if (entry.statisticsError || usageWarningsOf(entry).length > 0) out.partial++
    if (supportsCreditDetail(account)) expiring.push(...expiringCreditsOf(entry, now))
    if (entry.unlimited === true) { out.unlimited++; continue }
    const hasTotalLeft = Object.prototype.hasOwnProperty.call(entry, 'totalLeft')
    const rawValue = usageNumberOrNull(hasTotalLeft ? entry.totalLeft : entry.available)
    const ratio = groupRatioOf(usageWindows(entry), rawValue)
    const unit = String(entry.unit || t('Points')).trim()
    const entryMode: 'additive' | 'ratio' | null =
      rawValue === null && ratio === null ? null : unit === '%' || rawValue === null ? 'ratio' : 'additive'
    if (entryMode === null) { out.uncountable++; continue }
    if (mode === null) mode = entryMode
    // 绝对值与百分比、不同单位的余额不能混加。
    if (mode !== entryMode) { out.uncountable++; continue }
    if (entryMode === 'additive' && out.unit && unit !== out.unit) { out.uncountable++; continue }
    if (!out.unit) out.unit = unit
    out.counted++
    if (entryMode === 'additive') {
      if (rawValue !== null) sum += rawValue
    } else if (ratio && (!out.ratio || ratio.percent < out.ratio.percent)) {
      out.ratio = ratio
    }
  }
  out.additive = mode !== 'ratio'
  out.expiring = creditExpirySummaryOf(expiring, now)
  if (out.additive && out.counted) out.sum = Number(sum.toFixed(2))
  return out
}

type UnitsBridge = {
  formatTokens?: (value: unknown) => string
  formatInt?: (value: unknown) => string
}

function balanceAmountText(value: number): string {
  const api = (window as unknown as { wbUnits?: UnitsBridge }).wbUnits
  if (value < 10_000) {
    return Number.isInteger(value) && api?.formatInt ? api.formatInt(value) : numberText(value)
  }
  return api?.formatTokens ? api.formatTokens(value) : numberText(value)
}

function ratioText(row: GroupRatioReading): string {
  return row.unlimited ? '∞' : row.percent === null ? '—' : `${numberText(row.percent)}%`
}

function groupBalanceTitle(balance: GroupBalance): string {
  const parts = [
    balance.additive
      ? balance.scope === 'filtered'
        ? t('Total available balance of the {n} accounts under the current filter (changes with filter and search; not affected by pagination)",', { n: balance.total })
        : t('Sum of available balance across all {n} accounts in this group, counted over the whole group (unaffected by filters or search)', { n: balance.total })
      : balance.scope === 'filtered'
        ? t('Quota overview for the {n} accounts under the current filter (changes with filters and search, unaffected by paging)', { n: balance.total })
        : t('Quota overview for the {n} accounts in this group (whole group, unaffected by filters or search)', { n: balance.total }),
  ]
  parts.push(t('{n} counted', { n: balance.counted }))
  if (balance.unlimited) parts.push(t('includes {n} unlimited account(s); total shown as ∞', { n: balance.unlimited }))
  if (!balance.additive && balance.ratio) {
    parts.push(balance.ratio.hasShortWindow
      ? t('Shows the account with the lowest effective daily or short-window quota; all periods come from that account')
      : t('Selects the account with the lowest remaining percentage; all periods come from that account'))
    parts.push(t('Selected account\'s periods: {list}', {
      list: balance.ratio.windows.map(row => [row.label, ratioText(row)].filter(Boolean).join(' ')).join(' · '),
    }))
    if (balance.ratio.adjusted) {
      parts.push(t('The weekly or monthly quota is exhausted, so the effective short-window quota is treated as 0%; account details retain the original readings'))
    }
  }
  if (balance.pending) parts.push(t('{n} not read yet (not queried or in progress)', { n: balance.pending }))
  if (balance.failed) parts.push(t('{n} failed to query', { n: balance.failed }))
  if (balance.uncountable) {
    parts.push(balance.additive
      ? t('{n} have no absolute figure (percentage only or no quota), not counted', { n: balance.uncountable })
      : t('{n} readings had no usable percentage and were not counted', { n: balance.uncountable }))
  }
  if (balance.partial) parts.push(t('{n} only partially read (readable but incomplete)', { n: balance.partial }))
  const expiryText = creditExpiryText(balance.expiring, t, balanceAmountText, balance.unit === t('Points') ? 'Points' : balance.unit)
  if (expiryText.detail) parts.push(expiryText.detail)
  return parts.join('; ')
}

export function GroupBalanceBadge({ balance }: { balance: GroupBalance }) {
  const expiryText = creditExpiryText(balance.expiring, t, balanceAmountText, balance.unit === t('Points') ? 'Points' : balance.unit)
  const warned = balance.failed + balance.partial > 0
  const empty = balance.additive
    ? balance.sum === null && balance.unlimited === 0
    : balance.ratio === null && balance.unlimited === 0
  const low = !balance.additive && balance.ratio?.windows.some(row => !row.unlimited && row.percent !== null && row.percent < 20)
  const tone = warned || low ? 'warn' : empty ? 'muted' : 'ok'
  const amount = balance.unlimited > 0
    ? '∞'
    : balance.sum === null ? '—' : balanceAmountText(balance.sum)
  const unit = balance.additive && balance.unit && balance.unit !== 'Points' && balance.unit !== t('Points')
    ? usageUnitLabel(balance.unit, t)
    : ''
  const note = balance.unlimited === 0 && balance.counted < balance.total
    ? t('counted {counted}/{total}', { counted: balance.counted, total: balance.total })
    : ''
  return (
    <span className={`grp-balance ${tone}`} title={groupBalanceTitle(balance)}>
      <span className='k'>{balance.additive ? t('Total available') : t('Lowest available')}</span>
      {balance.additive ? (
        <>
          <b className='v'>{amount}</b>
          {unit ? <span className='u'>{unit}</span> : null}
        </>
      ) : balance.unlimited > 0 ? (
        <b className='v'>∞</b>
      ) : balance.ratio ? (
        balance.ratio.windows.map((row, index) => (
          <React.Fragment key={`${row.label}-${index}`}>
            {index > 0 ? <span className='k' aria-hidden='true'>·</span> : null}
            {row.label ? <span className='k'>{row.label}</span> : null}
            <b className={row.percent === null && !row.unlimited ? 'k' : 'v'}>{ratioText(row)}</b>
          </React.Fragment>
        ))
      ) : (
        <b className='v'>—</b>
      )}
      {note ? <span className='n'>{note}</span> : null}
      {expiryText.group ? <span className='n soon' title={expiryText.detail}>{expiryText.group}</span> : null}
      {warned ? <span className='warn-mark' aria-hidden>⚠</span> : null}
    </span>
  )
}
