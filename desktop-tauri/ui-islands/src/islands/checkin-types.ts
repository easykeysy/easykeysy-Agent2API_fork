export type {
  AutoState, CheckinAccount, CheckinCenter, CheckinResult, CheckinSection, GrowthEntry, HistoryRow,
  KeepaliveState, OnboardingEntry, OnboardingSettlement, OnboardingTask, OutOfScope, PlanEntry,
  ProviderGroup, RecordsPage, RecordRow, ScheduleEntry, TravelAction,
  TravelRunResult, TravelState,
} from '../../../ui-shared/checkin'
export { CHECKIN_SECTION_OPTIONS, modesOf } from '../../../ui-shared/checkin'

/** 档位 → 按钮文案（`checkin` 是兜底档，不是插件申报值） */
export const MODE_LABEL: Record<string, string> = {
  checkin: 'Check in',
  keepalive: 'Keep-alive',
  claim: 'Claim',
  full: 'Keep-alive + Claim',
}

/** 档位 → 悬停说明 */
export const MODE_TITLE: Record<string, string> = {
  checkin: 'Go through this provider\'s daily sign-in endpoint',
  keepalive: 'Keep the account active with free models and a web-app conversation · no rewards claimed',
  claim: 'Only claim that day\'s reward· Do not keep alive',
  full: 'Keep alive and collect the reward',
}

/** 签到来源 → 展示名 */
export const SOURCE_LABEL: Record<string, string> = {
  schedule: 'Scheduled',
  makeup: 'Catch-up',
  startup: 'Catch-up',
  manual: 'Manual',
  single: 'Single',
  activity: 'Campaign',
}

/** 结果 → 展示名与色调 */
export const OUTCOME_VIEW: Record<string, { label: string; variant: 'success' | 'brand' | 'warning' | 'destructive' | 'outline' }> = {
  success: { label: 'Claimed', variant: 'success' },
  already: { label: 'claimed today', variant: 'brand' },
  pending: { label: 'Not claimed', variant: 'warning' },
  captcha: { label: 'Human verification required', variant: 'warning' },
  failed: { label: 'Failed', variant: 'destructive' },
}

/** 毫秒时间戳 → 本地可读时间；缺失 / 非法给 `—` */
export function formatTime(value: unknown): string {
  const stamp = Number(value)
  if (!Number.isFinite(stamp) || stamp <= 0) return '—'
  const date = new Date(stamp)
  if (Number.isNaN(date.getTime())) return '—'
  return date.toLocaleString('zh-CN', { hour12: false })
}

/** 毫秒 → `HH:MM`（只关心当天时刻的场合） */
export function formatClock(value: unknown): string {
  const stamp = Number(value)
  if (!Number.isFinite(stamp) || stamp <= 0) return ''
  const date = new Date(stamp)
  if (Number.isNaN(date.getTime())) return ''
  const pad = (n: number) => String(n).padStart(2, '0')
  return `${pad(date.getHours())}:${pad(date.getMinutes())}`
}

/** 毫秒 → `MM-DD HH:MM`（记录表的时间列） */
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
    /** 提供商目录（providers.js）：把 provider id 翻成展示名 */
    wbProviders?: { labelOf?: (id: string) => string }
  }
}
