/** use and （none React ， and use ） */

import { upstreamCooldownOf } from '../../../ui-shared/account-upstream-state'

/** ：1.2 / 3.4 ； 1 */
export function formatCompact(value: number): string {
  if (!Number.isFinite(value)) return '—'
  const abs = Math.abs(value)
  if (abs >= 1e8) return `${(value / 1e8).toFixed(abs >= 1e9 ? 0 : 2)} 亿`
  if (abs >= 1e4) return `${(value / 1e4).toFixed(abs >= 1e6 ? 0 : 1)} 万`
  return String(Math.round(value))
}

/** */
export function formatInt(value: number): string {
  if (!Number.isFinite(value)) return '—'
  return Math.round(value).toLocaleString('zh-CN')
}

/** Token ：（ and use this 1 ， not 1 ） */
export function formatTokens(value: number): string {
  return formatCompact(value)
}

/** ： 0~1 or 0~100 by `asRatio` */
export function formatPercent(value: number | null | undefined, digits = 1, asRatio = true): string {
  if (value === null || value === undefined || !Number.isFinite(value)) return '—'
  const percent = asRatio ? value * 100 : value
  return `${percent.toFixed(digits)}%`
}

/** ms → can when （1.2s / 860ms / 2m 05s） */
export function formatDuration(ms: number | null | undefined): string {
  if (ms === null || ms === undefined || !Number.isFinite(ms) || ms < 0) return '—'
  if (ms < 1000) return `${Math.round(ms)}ms`
  if (ms < 60_000) return `${(ms / 1000).toFixed(ms < 10_000 ? 2 : 1)}s`
  const total = Math.floor(ms / 1000)
  return `${Math.floor(total / 60)}m ${String(total % 60).padStart(2, '0')}s`
}

/** when → HH:MM:SS */
export function formatClock(ts: number | null | undefined): string {
  if (!ts) return '—'
  const date = new Date(ts)
  if (Number.isNaN(date.getTime())) return '—'
  return date.toLocaleTimeString('zh-CN', { hour12: false })
}

/** when → MM-DD HH:MM */
export function formatStamp(ts: number | null | undefined): string {
  if (!ts) return '—'
  const date = new Date(ts)
  if (Number.isNaN(date.getTime())) return '—'
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  const time = date.toLocaleTimeString('zh-CN', { hour12: false, hour: '2-digit', minute: '2-digit' })
  return `${month}-${day} ${time}`
}

/** for when ：3 minutes after / 2 when after / 4 after */
export function formatRelative(ts: number | null | undefined): string {
  if (!ts) return '—'
  const diff = ts - Date.now()
  const abs = Math.abs(diff)
  const suffix = diff >= 0 ? '后' : '前'
  if (abs < 60_000) return diff >= 0 ? '即将' : '刚刚'
  if (abs < 3_600_000) return `${Math.round(abs / 60_000)} 分钟${suffix}`
  if (abs < 86_400_000) return `${Math.round(abs / 3_600_000)} 小时${suffix}`
  return `${Math.round(abs / 86_400_000)} 天${suffix}`
}

/**
 * valid： 「 for when 」 up for 。
 * expiredreturn「expired」；return「—」。
 */
export function formatExpiry(ts: number | null | undefined): string {
  if (!ts) return '—'
  const diff = ts - Date.now()
  if (diff <= 0) return '已过期'
  if (diff < 3_600_000) return `${Math.max(1, Math.round(diff / 60_000))} 分钟后`
  if (diff < 86_400_000) return `${Math.round(diff / 3_600_000)} 小时后`
  return `${Math.round(diff / 86_400_000)} 天后`
}

/** when （ and after to 1 ） */
export function beijingDayKey(ts = Date.now()): string {
  const date = new Date(ts + 8 * 3_600_000)
  return date.toISOString().slice(0, 10)
}

/** is already when （ when ） */
export function isToday(ts: number | null | undefined): boolean {
  if (!ts) return false
  return beijingDayKey(ts) === beijingDayKey()
}

/** use ， old when 。 */
export function accountName(account: {
  displayName?: string
  name?: string
  nickname?: string
  email?: string
  id?: string
}): string {
  return account.displayName || account.name || account.nickname || account.email || account.id || '未命名'
}

/** ： 3 ， in use • */
export function maskMiddle(text: string, keep = 3): string {
  const value = String(text || '')
  if (value.length <= keep * 2) return value
  return `${value.slice(0, keep)}${'•'.repeat(Math.min(8, value.length - keep * 2))}${value.slice(-keep)}`
}

/** when （） */
export function ellipsisMiddle(text: string, max = 26): string {
  const value = String(text || '')
  if (value.length <= max) return value
  const head = Math.ceil((max - 1) / 2)
  const tail = Math.floor((max - 1) / 2)
  return `${value.slice(0, head)}…${value.slice(-tail)}`
}

/**
 * **current** （`rateLimits[model]`， by when ）。
 *
 * and after `routing::rate_limit_reset_at`、 `activeLimits()` 1 ：
 * has `resetAt` in ****； in expired = ，
 * not this 「」。 after from not ， this must ——
 * 「 has no has 」 will all 。
 */
export function activeLimits(account: {
  rateLimits?: Record<string, { resetAt?: number; status?: number | string; code?: string | number; message?: string }>
}): Array<{ model: string; resetAt: number; status?: number | string; code?: string | number; message?: string }> {
  const limits = account.rateLimits
  if (!limits || typeof limits !== 'object') return []
  const now = Date.now()
  return Object.entries(limits)
    .map(([model, info]) => ({ model, resetAt: Number(info?.resetAt) || 0, ...(info || {}) }))
    .filter(item => item.resetAt > now)
    .sort((a, b) => a.resetAt - b.resetAt)
}

/** is in （ and {@link activeLimits} 1 ） */
export function isRateLimited(account: {
  upstreamCooldownUntil?: number | null
  upstreamBlockedReason?: string
  rateLimits?: Record<string, { resetAt?: number }>
}): boolean {
  return upstreamCooldownOf(account) !== null || activeLimits(account).length > 0
}

