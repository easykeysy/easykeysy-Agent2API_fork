export type {
  AutoState, CheckinAccount, CheckinCenter, CheckinResult, CheckinSection, GrowthEntry, HistoryRow,
  KeepaliveState, OnboardingEntry, OnboardingSettlement, OnboardingTask, OutOfScope, PlanEntry,
  ProviderGroup, RecordsPage, RecordRow, ScheduleEntry, TravelAction,
  TravelRunResult, TravelState,
} from '../../../ui-shared/checkin'
export { CHECKIN_SECTION_OPTIONS, modesOf } from '../../../ui-shared/checkin'

/** → by （`checkin` is ， not is ） */
export const MODE_LABEL: Record<string, string> = {
  checkin: '签到',
  keepalive: '保活',
  claim: '领取',
  full: '保活 + 领取',
}

/** → */
export const MODE_TITLE: Record<string, string> = {
  checkin: '走这一家的每日签到接口',
  keepalive: '用免费模型与网页会话维持账号活跃 · 不领取奖励',
  claim: '只领取当天的奖励 · 不做保活',
  full: '保活并领取奖励',
}

/** to → */
export const SOURCE_LABEL: Record<string, string> = {
  schedule: '定时',
  makeup: '补签',
  startup: '补签',
  manual: '手动',
  single: '单签',
  activity: '活动',
}

/** → and */
export const OUTCOME_VIEW: Record<string, { label: string; variant: 'success' | 'brand' | 'warning' | 'destructive' | 'outline' }> = {
  success: { label: '成功领取', variant: 'success' },
  already: { label: '今日已领', variant: 'brand' },
  pending: { label: '未领取', variant: 'warning' },
  captcha: { label: '需要人工验证', variant: 'warning' },
  failed: { label: '失败', variant: 'destructive' },
}

/** ms when → can when ； / to `—` */
export function formatTime(value: unknown): string {
  const stamp = Number(value)
  if (!Number.isFinite(stamp) || stamp <= 0) return '—'
  const date = new Date(stamp)
  if (Number.isNaN(date.getTime())) return '—'
  return date.toLocaleString('zh-CN', { hour12: false })
}

/** ms → `HH:MM`（ when ） */
export function formatClock(value: unknown): string {
  const stamp = Number(value)
  if (!Number.isFinite(stamp) || stamp <= 0) return ''
  const date = new Date(stamp)
  if (Number.isNaN(date.getTime())) return ''
  const pad = (n: number) => String(n).padStart(2, '0')
  return `${pad(date.getHours())}:${pad(date.getMinutes())}`
}

/** ms → `MM-DD HH:MM`（ when ） */
export function formatShort(value: unknown): string {
  const stamp = Number(value)
  if (!Number.isFinite(stamp) || stamp <= 0) return '—'
  const date = new Date(stamp)
  if (Number.isNaN(date.getTime())) return '—'
  const pad = (n: number) => String(n).padStart(2, '0')
  return `${pad(date.getMonth() + 1)}-${pad(date.getDate())} ${pad(date.getHours())}:${pad(date.getMinutes())}`
}

declare global {
  interface Window {
    wbApp?: {
      showPage?: (name: string) => void
      toast?: (message: string, tone?: string) => void
      readonly currentPage?: string
    }
    wbZcodePlans?: { open(options: { id: string; name?: string }): void }
  }
}
