/**
 * 。
 *
 * after `usage` for by to not （workbuddy `totalLeft` has 、
 * 1 `available` + `wallets`、 and to `availableView`）。
 * this ** by ** not is by provider ：provider ，
 * not 。 `accounts-panels.tsx` `usageSummary`
 * is 1 ，「 1 + 1 」。
 *
 * failure 1 in （`muted`）， not —— not to is ，
 * not this 。
 */

import type { UsageRow, UsageSnapshot } from '../api/gateway'
import { t, locale } from '../i18n'
import { creditStatisticsWalletsOf, recordReadingsOf, snapshotReadingsOf, usageAggregateOf, usageNumberOrNull, usageSupplementReadingsOf, usageUnitLabel, usageWarningsOf, usageWindowLabel, usageWindowView } from '../../../ui-shared/usage-readings'
import { formatStamp } from './format'
import { mergeUsageQueryRow, usageQueryFailureStatus } from '../../../ui-shared/usage-query-state'
import { creditExpirySummaryOf, creditExpiryText, creditExpiryUnitOf, expiringCreditsOf } from '../../../ui-shared/credit-expiry'

export type UsageView = {
  kind: 'ok' | 'warn' | 'bad' | 'muted'
  /** ，e.g.「available 12.3 」「」「failure」 */
  text: string
  /** （ to / by use ） */
  detail: string
  failureStatus?: string
  failureDetail?: string
  expiry?: ReturnType<typeof creditExpiryText>
}

/**
 * ：， 1 use 「」、 1 use 「」。
 *
 * not import `lib/format` can ： that ， 
 * use will ； this 1 ， 1 。
 */
function compact(value: unknown): string {
  // null / empty not can Number() 0： that will 「 not to 」「available 0」
  const num = usageNumberOrNull(value)
  if (num === null) return '—'
  const abs = Math.abs(num)
  // 「 / 」 1~2 ： need ， for no has 
  const chinese = locale().startsWith('zh')
  if (abs >= 1e8 && chinese) return `${(num / 1e8).toFixed(2)} ${locale() === 'zh-Hant' ? '億' : '亿'}`
  if (abs >= 1e4 && chinese) return `${(num / 1e4).toFixed(1)} ${locale() === 'zh-Hant' ? '萬' : '万'}`
  if (!chinese && abs >= 1e6) return `${(num / 1e6).toFixed(1)}M`
  if (!chinese && abs >= 1e4) return `${(num / 1000).toFixed(1)}k`
  const language = locale() === 'zh-Hans' ? 'zh-CN' : locale()
  if (Number.isInteger(num)) return num.toLocaleString(language)
  return num.toLocaleString(language, { maximumFractionDigits: 2 })
}

/** usage can 「」： when by */
function unitOf(usage: Record<string, unknown>): string {
  const unit = usage.unit
  return typeof unit === 'string' && unit.trim() ? usageUnitLabel(unit.trim(), t) : t('积分')
}

/** in failure。 */
function partialErrors(usage: Record<string, unknown>): string[] {
  const parts = usageWarningsOf(usage).map(line => t(line))
  if (usage.statisticsError) {
    parts.push(t('订阅统计未读到：{error}', { error: String(usage.statisticsError) }))
  }
  return parts
}

/** ： `usage` / `error` 1 + */
export function usageText(row?: UsageRow, now = Date.now()): UsageView {
  // ① no （ no has this 1 ）
  if (row === undefined) {
    return { kind: 'muted', text: t('未查询'), detail: t('尚未查询该账号的余额') }
  }

  // ② this 1 failure：「not configured」 is in ， its is failure
  if (row.error) {
    if (row.code === 'usage_not_configured') {
      return { kind: 'muted', text: t('未配置'), detail: t(row.error) }
    }
    const status = usageQueryFailureStatus(row, t, formatStamp)
    const previous = row.lastUsage ? usageText({ usage: row.lastUsage }, now) : null
    return {
      kind: 'bad',
      text: previous?.text || t('查询失败'),
      detail: [status?.detail || t(row.error), previous?.detail].filter(Boolean).join(' · '),
      failureStatus: previous ? status?.text : undefined,
      failureDetail: previous ? status?.detail : undefined,
      expiry: previous?.expiry,
    }
  }

  const usage = row.usage
  if (!usage || typeof usage !== 'object' || Array.isArray(usage)) {
    return { kind: 'muted', text: t('无数据'), detail: t('未返回可识别的余额数据') }
  }

  const data = usage as Record<string, unknown>
  if (data.error) return { kind: 'bad', text: t('查询失败'), detail: t(String(data.error)) }
  const errors = partialErrors(data)
  const aggregate = usageAggregateOf(data, t)
  const expiry = creditExpiryText(creditExpirySummaryOf(expiringCreditsOf(data, now), now), t, compact, creditExpiryUnitOf(data))
  const readings = [
    ...(aggregate ? [`${aggregate.label} ${aggregate.unlimited ? t('不限') : compact(aggregate.remaining)}${aggregate.unit && !aggregate.unlimited ? ` ${aggregate.unit}` : ''}`] : []),
    ...usageSupplementReadingsOf(data, t, compact),
    ...snapshotReadingsOf(data, t, compact).map(reading => `${reading.name} ${reading.view}`),
    ...recordReadingsOf(data, t, compact, formatStamp),
  ]
  const baseKind: UsageView['kind'] = errors.length ? 'warn' : 'ok'
  const withErrors = (view: UsageView): UsageView => ({
    kind: errors.length ? 'warn' : view.kind,
    text: view.text,
    detail: [...new Set([view.detail, ...readings, expiry.detail, ...errors].filter(Boolean))].join(' · '),
    expiry: expiry.badge ? expiry : undefined,
  })

  if (Array.isArray(data.windows) && data.windows.length) {
    const windows = data.windows.filter((item): item is Record<string, unknown> => Boolean(item) && typeof item === 'object')
    const detail = windows.map(window => {
      const label = usageWindowLabel(window.label, t)
      const view = usageWindowView(window, t, compact)
      const reset = typeof window.resetsAt === 'number' && window.resetsAt > 0
        ? t('重置 {date}', { date: formatStamp(window.resetsAt) }) : ''
      return [label, view, reset].filter(Boolean).join(' ')
    }).join(' · ')
    return withErrors({ kind: baseKind, text: t('分窗口额度'), detail })
  }

  // ④ workbuddy has ：totalLeft / planLeft / bonusLeft
  if (Object.prototype.hasOwnProperty.call(data, 'totalLeft')) {
    const total = data.unlimited === true ? '∞' : compact(data.totalLeft)
    const parts = [t('总剩余 {total}', { total })]
    if (data.planLeft !== undefined && data.planLeft !== null) {
      parts.push(t('套餐 {plan}', { plan: compact(data.planLeft) }))
    }
    if (data.bonusLeft !== undefined && data.bonusLeft !== null) {
      parts.push(t('奖励 {bonus}', { bonus: compact(data.bonusLeft) }))
    }
    return withErrors({ kind: baseKind, text: t('可用 {total}', { total }), detail: parts.join(' · ') })
  }

  // ⑤ after already to （ZCode 「1 token」）
  if (typeof data.availableView === 'string' && data.availableView.trim()) {
    const text = data.availableView.trim()
    const parts = [text]
    if (data.available !== undefined && data.available !== null) {
      parts.push(t('可用 {value} {unit}', { value: compact(data.available), unit: unitOf(data) }))
    }
    return withErrors({ kind: baseKind, text, detail: parts.join(' · ') })
  }

  // ⑥ 1 ：available is 
  if (typeof data.available === 'number' && Number.isFinite(data.available)) {
    return withErrors({
      kind: baseKind,
      text: t('可用 {value} {unit}', { value: compact(data.available), unit: unitOf(data) }),
      detail: t('可用 {value} {unit}', { value: compact(data.available), unit: unitOf(data) }),
    })
  }

  // ⑦ has ： 1 balance ， detail
  if (Array.isArray(data.wallets)) {
    const wallets = creditStatisticsWalletsOf(data) ?? data.wallets.filter(
      (item): item is Record<string, unknown> => Boolean(item) && typeof item === 'object',
    )
    const lead = wallets.find(item => usageNumberOrNull(item.balance) !== null)
    if (lead) {
      const walletUnit = typeof lead.unit === 'string' && lead.unit.trim() ? usageUnitLabel(lead.unit.trim(), t) : unitOf(data)
      const detail = wallets
        .map(item => {
          const name =
            (typeof item.displayName === 'string' && item.displayName) ||
            (typeof item.type === 'string' && item.type) ||
            t('明细')
          const balance =
            item.balance !== undefined && item.balance !== null
              ? compact(item.balance)
              : typeof item.balanceView === 'string' && item.balanceView
                ? item.balanceView
                : '—'
          return `${name} ${balance}`
        })
        .join(' · ')
      return withErrors({
        kind: baseKind,
        text: t('可用 {value} {unit}', { value: compact(lead.balance), unit: walletUnit }),
        detail,
      })
    }
  }

  const supplements = usageSupplementReadingsOf(data, t, compact)
  if (supplements.length) return withErrors({ kind: 'muted', text: supplements[0], detail: supplements.join(' · ') })
  // ⑧ not 
  return withErrors({ kind: 'muted', text: t('无数据'), detail: t('未返回可识别的余额数据') })
}

/** by id （ no has id ） */
export function usageIndex(snapshot: UsageSnapshot | null): Map<string, UsageRow> {
  const map = new Map<string, UsageRow>()
  const rows = snapshot?.results
  if (!Array.isArray(rows)) return map
  rows.forEach(row => {
    if (row && typeof row.id === 'string' && row.id) map.set(row.id, row)
  })
  return map
}

export function mergeUsageRow(row: UsageRow, previous?: UsageRow): UsageRow {
  return mergeUsageQueryRow({ ...row, at: row.at || Date.now() }, previous)
}
