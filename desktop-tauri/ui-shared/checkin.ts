export type DailyCheckinStatus = 'claimed' | 'done' | 'confirming' | 'inactive' | 'ineligible' | 'captcha' | 'failed'

export type DailyCheckinState = {
  at: number
  completed: boolean
  completedCount: number
  totalCount: number
  sourcesComplete: boolean
  kind?: 'plans'
  status?: DailyCheckinStatus
  message?: string
  retryAt?: number | null
  queryOnly?: boolean
}

export type CheckinAccount = {
  id: string
  name: string
  available: boolean
  checkinAt: number | null
  checkedInToday: boolean
  dailyCheckin?: DailyCheckinState | null
  edition: string | null
  streakDays: number
}

export type ProviderGroup = {
  id: string
  label: string
  accounts: CheckinAccount[]
  doneCount: number
  totalCount: number
  description?: string | null
  modes?: string[] | null
  uiHooks?: string[] | null
  manualPlans?: boolean
}

export type OutOfScope = { label: string; reason: string; count: number }

export type CheckinSection = 'daily' | 'onboarding' | 'plans' | 'growth'

export const CHECKIN_SECTION_OPTIONS = [
  { value: 'daily', label: '每日签到' },
  { value: 'onboarding', label: '新手福利' },
  { value: 'plans', label: '活动福利' },
  { value: 'growth', label: '成长任务' },
] as const

export type OnboardingSettlement = {
  at: number
  tasks: OnboardingTask[]
  earned: number | null
  total: number | null
}

export type OnboardingEntry = {
  id: string
  name: string
  provider: string
  settled: OnboardingSettlement | null
}

export type OnboardingTask = {
  key: string
  title: string
  points: number
  done: boolean
  blocked: boolean
  claiming: boolean
  error: string
}

export type PlanEntry = {
  id: string
  name: string
  provider: string
  claimAt: number | null
  claimPlans: Record<string, number> | null
}

export type GrowthEntry = { id: string; name: string; provider: string }

type GrowthTaskState = {
  status?: string
  current?: number
  target?: number
  supported?: boolean
  unforgeable?: boolean
  locked?: boolean
}

export function growthTaskCompleted(task: GrowthTaskState): boolean {
  return task.status === 'completed' || (Number.isFinite(task.current) && Number.isFinite(task.target)
    && Number(task.target) > 0 && Number(task.current) >= Number(task.target))
}

export function growthTaskRunnable(task: GrowthTaskState, mode: 'full' | 'claim'): boolean {
  if (!task.supported || task.locked || task.status === 'claimed') return false
  const completed = growthTaskCompleted(task)
  return mode === 'claim' ? completed : !task.unforgeable || completed
}


/** 1 ： when + no has （`builtin` has = this when ） */
export type ScheduleProviderRow = {
  id: string
  label: string
  time: string
  timeEnd?: string
  builtin: string | null
  firedToday: boolean
  accountCount?: number
  attemptedCount?: number
  pendingCount?: number
  nextAccountAt?: number | null
  /** to no （`false` and `firedToday=false` = in etc ） */
  overdue?: boolean
}

export type ScheduleEntry = {
  id: string
  label: string
  displayLabel: string
  enabled: boolean
  time: string
  /** to when ， and time when */
  timeEnd?: string
  providers: string[]
  /** when （`{providerId: "HH:MM"}`；empty = ） */
  providerTimes: Record<string, string>
  /** when and （ after ，） */
  providerRows: ScheduleProviderRow[]
  /** when （empty = ） */
  retryTime: string
  /** when （ to + etc ， not retryTime） */
  providerRetryTimes?: Record<string, string>
  /** in （minutes；0 = not ） */
  retryGapMinutes: number
  /** limit（0 = not ） */
  retryLimit: number
  lastFiredDate: string | null
  lastFiredToday: boolean
  /** no （「」） */
  pendingProviders: string[]
  /** in 、 etc */
  retryCount: number
  /**
   * already use id（ after by `retryLimit` ）。
   * this already from `retryCount` ，must 1 
   * 「 not 」，otherwise「 no 」。
   */
  retryExhausted?: string[]
  nextRunAt: number | null
  lastResult: CheckinResult | null
}

export type CheckinResultRow = {
  id?: string
  name?: string
  error?: string | null
  mode?: string
  claim?: {
    success?: boolean
    alreadyCompleted?: boolean
    outcome?: string
    retryAt?: number | null
    msg?: string
    error?: string
    points?: number
  } | null
  activity?: {
    error?: string
    pokeSucceeded?: boolean
    report?: { succeeded?: boolean }
    webConversation?: { succeeded?: boolean }
  } | null
}

export type CheckinResult = {
  at?: number
  reason?: string
  succeeded?: number
  active?: number
  total?: number
  skipped?: number
  failedCount?: number
  failed?: string[] | number
  results?: CheckinResultRow[]
  retryAccounts?: Record<string, number>
  outcome?: 'completed' | 'failed'
  error?: string
  accounts?: number
  credit?: number
  actions?: TravelAction[]
}

export type TravelAction = {
  accountId?: string
  name?: string
  outcome?: string
  message?: string
  credit?: number
}

export type AutoState = {
  enabled: boolean
  lastFiredToday: boolean
  running: boolean
  providerOptions: { id: string; label: string; builtinTime?: string | null }[]
  /** when （「 when 」 placeholder） */
  defaultRetryTime?: string
  schedules: ScheduleEntry[]
}

export type KeepaliveState = {
  models: string[]
  defaultModels: string[]
  apiKeyId?: string | null
  /** no 、 use is 1 available Key（ can 1 ） */
  apiKeyDefaulted?: boolean
  authError?: string | null
}

export type TravelState = {
  enabled: boolean
  available?: boolean
  pausedReason?: string | null
  times: string[]
  firedToday: string[]
  lastFiredDate: string | null
  lastFiredTime: string | null
  lastResult: CheckinResult | null
  running: boolean
}

export type TravelRunResult = {
  outcome: 'completed' | 'skipped'
  message?: string
  summary: CheckinResult | null
  schedule: TravelState
}

export type HistoryRow = {
  at: number
  date?: string
  reason?: string
  succeeded?: number
  total?: number
  skipped?: number
  failed?: number
  failedCount?: number
}

export type RecordRow = {
  rowId?: number
  ts: number
  date?: string
  accountId?: string
  accountName?: string
  /** （ after by provider id ） */
  provider?: string
  /** id（ ， ） */
  providerId?: string
  source?: string
  outcome?: string
  reason?: string
  message?: string
  error?: string
  keptActive?: boolean
}

export type CheckinCenter = {
  daily: {
    providers: ProviderGroup[]
    outOfScope: OutOfScope[]
    todayDone: number
    todayEligible: number
  }
  extras: {
    onboarding: OnboardingEntry[]
    plans: PlanEntry[]
    growth: GrowthEntry[]
  }
  auto: AutoState
  keepalive: KeepaliveState
  travel: TravelState
  history: HistoryRow[]
  adminPanels: string[]
  checkinFeatures: Record<string, {
    description?: string | null
    modes?: string[] | null
    uiHooks?: string[] | null
  }>
}

export type RecordsPage = {
  records: RecordRow[]
  total: number
}

export function modesOf(group: ProviderGroup | undefined): string[] {
  const modes = group?.modes
  if (!Array.isArray(modes) || modes.length === 0) return ['checkin']
  return modes.filter(mode => typeof mode === 'string' && mode)
}
