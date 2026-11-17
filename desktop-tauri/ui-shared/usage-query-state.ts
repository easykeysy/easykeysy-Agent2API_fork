type Translate = (key: string, params?: Record<string, string | number>) => string

export const USAGE_NOT_CONFIGURED_CODE = 'usage_not_configured'

export type UsageQueryRow = {
  id?: string
  name?: string | null
  usage?: Record<string, unknown> | null
  error?: string | null
  code?: string | number | null
  at?: number
  /** 最近一次成功查询的毫秒时间戳；失败不会推进。 */
  lastSuccessAt?: number
  /** 仅失败时携带的历史读数，不代表本次查询成功。 */
  lastUsage?: Record<string, unknown> | null
  requestId?: string
  stale?: boolean
  queryStatus?: import('./usage-query-task').UsageQueryStatus
}

function reading(value: unknown): Record<string, unknown> | null {
  return value && typeof value === 'object' && !Array.isArray(value)
    ? value as Record<string, unknown> : null
}

function timestamp(value: unknown): number {
  const time = Number(value)
  return Number.isFinite(time) && time > 0 && time <= 8.64e15 ? time : 0
}

/** 最新尝试与上次成功读数分别保存；失败不能刷新历史读数的时间。 */
export function mergeUsageQueryRow(row: UsageQueryRow, previous?: UsageQueryRow): UsageQueryRow {
  if (!row.error) {
    return { ...row, lastUsage: null, lastSuccessAt: timestamp(row.lastSuccessAt) || timestamp(row.at) }
  }
  const hasHistory = Object.prototype.hasOwnProperty.call(row, 'lastUsage')
  const lastUsage = hasHistory ? reading(row.lastUsage)
    : reading(previous?.error ? previous.lastUsage : previous?.usage)
  const lastSuccessAt = hasHistory ? timestamp(row.lastSuccessAt)
    : timestamp(row.lastSuccessAt) || timestamp(previous?.lastSuccessAt) || (previous?.error ? 0 : timestamp(previous?.at))
  return { ...row, usage: null, lastUsage, lastSuccessAt: lastUsage ? lastSuccessAt : 0 }
}

export function usageAgeText(at: number, t: Translate, now: number): string {
  const elapsed = Math.max(0, now - at)
  if (elapsed < 10_000) return t('Updated just now')
  if (elapsed < 60_000) return t('Updated {n} seconds ago', { n: Math.floor(elapsed / 1000) })
  if (elapsed < 3_600_000) return t('Updated {n} minutes ago', { n: Math.floor(elapsed / 60_000) })
  if (elapsed < 86_400_000) return t('Updated {n} hours ago', { n: Math.floor(elapsed / 3_600_000) })
  return t('Updated {n} days ago', { n: Math.floor(elapsed / 86_400_000) })
}

export function usageQueryFailureStatus(
  row: UsageQueryRow,
  t: Translate,
  formatTime: (at: number) => string,
  now = Date.now(),
): { text: string; detail: string } | null {
  if (!row.error || row.code === USAGE_NOT_CONFIGURED_CODE) return null
  const at = reading(row.lastUsage) ? timestamp(row.lastSuccessAt) : 0
  const text = at ? `${t('Update failed')} · ${usageAgeText(at, t, now)}` : t('Query failed')
  const fullTime = at ? formatTime(at) : ''
  return {
    text,
    detail: [t(row.error), fullTime ? t('Last successful query: {time}', { time: fullTime }) : ''].filter(Boolean).join('\n'),
  }
}
