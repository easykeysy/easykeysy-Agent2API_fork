/**
 * to in ：、、 and 「 this 1 current in 」 。
 *
 * ── Why is store not is React state ────────────────
 * 1 （`daily` / `auto` / `keepalive` / `travel` in 
 * `/api/checkin-center` 1 response）。 React state will 1 ，
 * and 。 this is 「 1 、 」—— and 
 * `accounts-store.ts` 1 to 。
 *
 * ── Why not use useSyncExternalStore ────────────────────────
 * is 「」，（、 ）， after 
 * 。`useSyncExternalStore` in ， this 
 * use not up 。
 *
 * ── error ────────────────────────────────────────────────
 * all `run()`： 1 `saving` 、error、success after 。
 * by not 1 try/catch —— that is old iframe 40 。
 */

import type {
  AutoState, CheckinCenter, CheckinSection, KeepaliveState, OnboardingTask, RecordsPage, RecordRow, TravelRunResult,
  TravelState,
} from './checkin-types'
import { t } from '../i18n'
import { DEFAULT_PAGE_SIZE, SERVER_PAGE_SIZES, readPageSize } from './table-shell'
import type { CheckinResultRow } from '../../../ui-shared/checkin'
import { growthTaskRunnable } from '../../../ui-shared/checkin'
import { rowCompleted, rowPending, rowMessage as resultMessage } from '../../../ui-shared/checkin-result'
import { onboardingFromCenter } from './checkin-onboarding-cache'

/** new by （ need up ， 1 1 ） */
export type OnboardingCache = {
  status: 'idle' | 'loading' | 'loaded' | 'error'
  tasks: OnboardingTask[]
  earned: number
  claiming: boolean
  settled: boolean
  settledAt?: number
  checkedAt?: number
  error?: string
}

/**
 * by （WorkBuddy in ）。
 *
 * to 「 has can 」，（ / can / / / ）
 * need 1 up —— and new 1 （must up request）。
 */
export type GrowthCache = {
  status: 'idle' | 'loading' | 'loaded' | 'error'
  checkedAt?: number
  /** can （null = no to ） */
  energy: number | null
  /** （null = no to ） */
  streak: number | null
  tasks: GrowthTask[]
  travel: unknown
  school: unknown
  running: boolean
  runningSchool: boolean
  /** （ by code ， in up ） */
  results: { code: string; outcome?: string; message?: string }[]
  error?: string
}

/** 1 */
export type GrowthTask = {
  code: string
  title: string
  description: string
  status: string
  current: number
  target: number
  locked: boolean
  supported: boolean
  /** none （ need ），「 need 」 and not */
  unforgeable: boolean
  kind: string
  source: string
}

/* ─── ───────────────────────────────────── */

/** use to （`bridge.rs` to that 1 ） */
type CheckinBridge = {
  getCheckinCenter?: () => Promise<CheckinCenter | null | undefined>
  checkinAllAccounts?: (id?: string) => Promise<CheckinRunResult | null | undefined>
  runCheckinActivity?: (id: string, mode: string) => Promise<CheckinRunResult | null | undefined>
  getCheckinRecords?: (params: Record<string, unknown>) => Promise<RecordsPage | null | undefined>
  clearCheckinRecords?: (params: Record<string, unknown>) => Promise<unknown>
  getCheckinSchedules?: () => Promise<AutoState | null | undefined>
  saveCheckinSchedule?: (patch: Record<string, unknown>) => Promise<AutoState | null | undefined>
  deleteCheckinSchedule?: (id: string) => Promise<unknown>
  runCheckinSchedule?: (id: string) => Promise<CheckinRunResult | null | undefined>
  saveAutoCheckin?: (patch: Record<string, unknown>) => Promise<AutoState | null | undefined>
  getCheckinKeepalive?: () => Promise<KeepaliveState | null | undefined>
  saveCheckinKeepalive?: (models: string[], apiKeyId?: string | null) => Promise<KeepaliveState | null | undefined>
  getTravelSchedule?: () => Promise<TravelState | null | undefined>
  saveTravelSchedule?: (patch: Record<string, unknown>) => Promise<TravelState | null | undefined>
  runTravelScheduleNow?: () => Promise<TravelRunResult | null | undefined>
  getOnboardingTasks?: (id: string, refresh?: boolean) => Promise<OnboardingPayload | null | undefined>
  claimOnboardingTasks?: (id: string) => Promise<OnboardingPayload | null | undefined>
  getQoderUmid?: () => Promise<UmidState | null | undefined>
  installQoderUmid?: () => Promise<unknown>
  getGrowthStatus?: (id: string) => Promise<GrowthPayload | null | undefined>
  runGrowthTask?: (id: string, code: string, mode: string) => Promise<GrowthRunRow | null | undefined>
  runGrowthSchool?: (id: string, lottery: boolean) => Promise<{ message?: string } | null | undefined>
  runGrowthTravel?: (id: string) => Promise<{ message?: string } | null | undefined>
}

/** `/api/accounts/{id}/growth` return（ / can / / / ） */
type GrowthPayload = {
  energy?: number | null
  streak?: number | null
  tasks?: { tasks?: unknown; error?: string }
  travel?: unknown
  school?: unknown
}

/** */
type GrowthRunRow = { code?: string; outcome?: string; message?: string }

/** new return（ and ） */
type OnboardingPayload = {
  tasks?: unknown
  earned?: number
  total?: number
  settled?: boolean
  settledAt?: number | null
  failed?: number
  message?: string
}

/** Qoder UMID （， and has no has none） */
export type UmidState = {
  available?: boolean
  installing?: boolean
  installSupported?: boolean
}

/** to returntotal and results，return。 */
type CheckinRunResult = CheckinResultRow & {
  succeeded?: number
  active?: number
  total?: number
  skipped?: number
  failedCount?: number
  failed?: string[]
  results?: CheckinResultRow[]
  reason?: string
  state?: AutoState
}

type SharedWindow = {
  workbuddyDesktop?: CheckinBridge
  wbApp?: {
    /** / new when by use ； in can when */
    readonly currentPage?: string
    toast?: (message: string, tone?: string) => void
    refresh?: () => void
  }
}

function bridge(): CheckinBridge {
  return (window as unknown as SharedWindow).workbuddyDesktop ?? {}
}

/** to use （`checkin-center-tab` need use ） */
export function bridgeApi(): CheckinBridge {
  return bridge()
}

function app(): SharedWindow['wbApp'] {
  return (window as unknown as SharedWindow).wbApp
}

/** ： toast（ and its 1 ）， no has `console` */
export function toast(message: string, tone: 'ok' | 'err' | 'warn' = 'ok'): void {
  const host = app()
  if (host?.toast) {
    host.toast(message, tone)
    return
  }
  // no toast when not this —— use by no is 
  if (tone === 'err') console.error('[checkin]', message)
  else console.info('[checkin]', message)
}

/** error → can （ is `Error`， is / for ） */
export function errorMessage(error: unknown): string {
  if (error instanceof Error) return error.message
  if (typeof error === 'string') return error
  if (error && typeof error === 'object' && 'message' in error) {
    return String((error as { message: unknown }).message)
  }
  return String(error)
}

/* ─── ─────────────────────────────────── */

export type Store = {
  /** （null = no to ） */
  center: CheckinCenter | null
  centerSection: CheckinSection
  loaded: boolean
  /** / new error（error） */
  loadError: string
  /** use 「 in 」：all use （ 1 this has 1 in ） */
  saving: boolean
  /** in when id */
  runningScheduleId: string | null
  /** id；`'new'` = new */
  editingId: string | null
  /** in ：`<accountId>:<mode>` */
  signing: Set<string>
  /** in to */
  runningAll: boolean
  /** （null = ） */
  keepaliveDraft: string[] | null
  /** when （null = ） */
  travelTimesDraft: string | null
  /** to （： has and ） */
  records: RecordRow[]
  recordsTotal: number
  recordsLoaded: boolean
  recordsError: string
  recordsLoading: boolean
  recordsQuery: {
    limit: number
    offset: number
    outcome: string
    /** to ：'' = ，manual / schedule / makeup */
    kind: string
    provider: string
    keyword: string
    /** when down （ms；undefined = not ） */
    start?: number
  }
  /** new ： id → （ not ） */
  onboarding: Map<string, OnboardingCache>
  /** ： id → */
  growth: Map<string, GrowthCache>
}

const recordsPageSize = readPageSize('checkin-records', SERVER_PAGE_SIZES)

export const store: Store = {
  center: null,
  centerSection: 'daily',
  loaded: false,
  loadError: '',
  saving: false,
  runningScheduleId: null,
  editingId: null,
  signing: new Set(),
  runningAll: false,
  keepaliveDraft: null,
  travelTimesDraft: null,
  records: [],
  recordsTotal: 0,
  recordsLoaded: false,
  recordsError: '',
  recordsLoading: false,
  recordsQuery: {
    limit: typeof recordsPageSize === 'number' ? recordsPageSize : DEFAULT_PAGE_SIZE,
    offset: 0, outcome: '', kind: '', provider: '', keyword: '',
  },
  onboarding: new Map(),
  growth: new Map(),
}

const listeners = new Set<() => void>()

export function subscribe(listener: () => void): () => void {
  listeners.add(listener)
  return () => {
    listeners.delete(listener)
  }
}

/** store and （allpath this ） */
export function patch(next: Partial<Store>): void {
  Object.assign(store, next)
  for (const notify of listeners) notify()
}

/* ─── ───────────────────────────────────── */

let centerRequest = 0

/** （ / after ） */
export async function loadCenter(): Promise<void> {
  const request = ++centerRequest
  try {
    const data = await bridge().getCheckinCenter?.()
    if (request !== centerRequest) return
    if (!data) {
      patch({ loaded: true, loadError: '后端未返回签到中心数据' })
      return
    }
    patch({
      center: data,
      onboarding: onboardingFromCenter(data, store.onboarding),
      loaded: true,
      loadError: '',
      // to = this （success after will ， not ）
      keepaliveDraft: null,
      travelTimesDraft: null,
    })
  } catch (error) {
    if (request === centerRequest) patch({ loaded: true, loadError: errorMessage(error) })
  }
}

let recordsRequest = 0

/** to （ + ）。 */
export async function loadRecords(): Promise<void> {
  const request = ++recordsRequest
  const query = store.recordsQuery
  patch({ recordsLoading: true })
  try {
    const data = await bridge().getCheckinRecords?.({
      limit: query.limit,
      offset: query.offset,
      outcome: query.outcome || undefined,
      kind: query.kind || undefined,
      provider: query.provider || undefined,
      keyword: query.keyword || undefined,
      start: query.start,
    })
    if (request !== recordsRequest) return
    const total = Math.max(0, Number(data?.total) || 0)
    const lastOffset = Math.max(0, (Math.ceil(total / query.limit) - 1) * query.limit)
    if (query.offset > lastOffset) {
      patch({ recordsQuery: { ...query, offset: lastOffset } })
      await loadRecords()
      return
    }
    patch({
      records: Array.isArray(data?.records) ? data.records : [],
      recordsTotal: total,
      recordsLoaded: true,
      recordsError: '',
    })
  } catch (error) {
    if (request === recordsRequest) patch({ recordsLoaded: true, recordsError: errorMessage(error) })
  } finally {
    if (request === recordsRequest) patch({ recordsLoading: false })
  }
}

/* ─── ───────────────────────────────────── */

/**
 * 1 。
 *
 * `run` ： `saving`、 1 、success after 。
 * use 「 」， not 1 try/catch/finally —— old iframe 
 * 40 is this ， and `finally` will in saving。
 */
async function run<T>(
  action: () => Promise<T>,
  options: { success?: string; reload?: boolean; failure?: string } = {},
): Promise<T | undefined> {
  patch({ saving: true })
  try {
    const result = await action()
    if (options.reload !== false) await loadCenter()
    if (options.success) toast(options.success)
    return result
  } catch (error) {
    toast(`${options.failure ?? '操作失败'}：${errorMessage(error)}`, 'err')
    return undefined
  } finally {
    patch({ saving: false })
  }
}

/** to （`id` as empty = can ） */
export async function runAllCheckin(accountId?: string): Promise<void> {
  if (store.runningAll) return
  patch({ runningAll: true })
  try {
    const result = await bridge().checkinAllAccounts?.(accountId)
    reportRunResult(result)
  } catch (error) {
    toast(`签到失败：${errorMessage(error)}`, 'err')
  } finally {
    patch({ runningAll: false })
    await loadCenter()
  }
}

/** （`mode` = ） */
export async function signSingle(accountId: string, mode: string): Promise<void> {
  const key = `${accountId}:${mode}`
  if (store.signing.has(key)) return
  const next = new Set(store.signing)
  next.add(key)
  patch({ signing: next })
  try {
    const result = mode === 'checkin'
      ? await bridge().checkinAllAccounts?.(accountId)
      : await bridge().runCheckinActivity?.(accountId, mode)
    reportRunResult(result, true)
  } catch (error) {
    toast(`签到失败：${errorMessage(error)}`, 'err')
  } finally {
    const after = new Set(store.signing)
    after.delete(key)
    patch({ signing: after })
    await loadCenter()
  }
}

function rowMessage(row: CheckinResultRow): string {
  return resultMessage(row) || t('未完成，请查看签到记录')
}

/** as ，HTTP successonly。 */
function reportRunResult(result: CheckinRunResult | null | undefined, single = false): void {
  if (!result) return
  const rows = Array.isArray(result.results) ? result.results
    : result.claim || result.activity || result.error ? [result] : []
  if (single && rows.length === 1) {
    const row = rows[0]
    const completed = rowCompleted(row)
    const message = completed ? row.claim?.msg?.trim() || result.reason || '签到完成' : rowMessage(row)
    toast(completed ? `✅ ${message}` : message, completed ? 'ok' : rowPending(row) ? 'warn' : 'err')
    return
  }
  const succeeded = Number(result.succeeded) || 0
  const active = Number(result.active) || 0
  const total = Number(result.total) || rows.length
  const failedRows = rows.filter(row => !rowCompleted(row) && !rowPending(row))
  const failed = result.failedCount ?? result.failed?.length ?? failedRows.length
  const activeText = active > 0 ? `，日活保活 ${active} 个` : ''
  if (failed > 0) {
    const failures = result.failed?.length ? result.failed : failedRows.map(rowMessage)
    const names = failures.length ? `（${failures.slice(0, 5).join('、')}）` : ''
    toast(`签到完成：${succeeded}/${total} 成功${activeText}，${failed} 个失败${names}`, 'err')
    return
  }
  if (single) {
    const completed = succeeded > 0 || active > 0
    toast(completed ? `✅ ${result.reason || '签到完成'}` : t('未完成，请查看签到记录'), completed ? 'ok' : 'warn')
    return
  }
  toast(`✅ 签到完成：${succeeded}/${total} 个账号成功领取${activeText}`, succeeded > 0 || active > 0 ? 'ok' : 'warn')
}

/** when to total */
export async function toggleMaster(enabled: boolean): Promise<void> {
  await run(
    async () => {
      const data = await bridge().saveAutoCheckin?.({ enabled })
      if (data) patch({ center: withAuto(store.center, data) })
    },
    { success: enabled ? '✅ 定时签到已开启' : '定时签到已关闭', reload: false, failure: '保存失败' },
  )
  void loadCenter()
}

/** 1 when （`closeEditor: false` = up ， not not ） */
export async function saveScheduleEntry(
  payload: Record<string, unknown>,
  options: { closeEditor?: boolean; silent?: boolean } = {},
): Promise<boolean> {
  const saved = await run(
    async () => {
      const data = await bridge().saveCheckinSchedule?.(payload)
      if (!data) throw new Error(t('后端未返回定时签到数据'))
      patch({
        center: withAuto(store.center, data),
        ...(options.closeEditor === false ? {} : { editingId: null }),
      })
      return true
    },
    { success: options.silent ? undefined : '✅ 已保存', reload: false, failure: '保存失败' },
  )
  if (saved) void loadCenter()
  return saved === true
}

/** delete 1 when */
export async function removeScheduleEntry(id: string): Promise<void> {
  await run(
    async () => {
      await bridge().deleteCheckinSchedule?.(id)
    },
    { success: '已删除', failure: '删除失败' },
  )
}

/** 1 when */
export async function runScheduleEntry(id: string): Promise<void> {
  if (store.runningScheduleId) return
  patch({ runningScheduleId: id })
  try {
    const result = await bridge().runCheckinSchedule?.(id)
    reportRunResult(result)
  } catch (error) {
    toast(`执行失败：${errorMessage(error)}`, 'err')
  } finally {
    patch({ runningScheduleId: null })
    await loadCenter()
  }
}

/** */
export async function saveKeepaliveModels(models: string[], apiKeyId?: string | null): Promise<void> {
  await run(
    async () => {
      const data = await bridge().saveCheckinKeepalive?.(models, apiKeyId)
      if (data && store.center) {
        patch({ center: { ...store.center, keepalive: data }, keepaliveDraft: null })
      }
    },
    { success: '✅ 模型链已保存', reload: false, failure: '保存失败' },
  )
}

/** （ / when ） */
export async function saveTravel(body: Record<string, unknown>): Promise<void> {
  await run(
    async () => {
      const data = await bridge().saveTravelSchedule?.(body)
      if (data && store.center) {
        patch({
          center: { ...store.center, travel: data },
          ...(body.times ? { travelTimesDraft: null } : {}),
        })
      }
    },
    { success: '✅ 已保存', reload: false, failure: '保存失败' },
  )
}

/** ： */
export async function runTravelNow(): Promise<void> {
  await run(
    async () => {
      const result = await bridge().runTravelScheduleNow?.()
      if (!result) throw new Error(t('后端未返回巡检结果'))
      if (result.outcome === 'skipped') {
        toast(t('本轮已跳过：{reason}', { reason: result.message ?? '' }))
        return
      }
      const failed = Number(result.summary?.failed) || 0
      toast(t('猫猫旅行：{n} 个动作，+{credit} 积分，{failed} 个失败', {
        n: result.summary?.actions?.length ?? 0,
        credit: Number(result.summary?.credit) || 0,
        failed,
      }), failed > 0 ? 'err' : 'ok')
    },
    { failure: '执行失败' },
  )
}

/** empty to （`all` as = ，otherwise by current） */
export async function clearRecords(all: boolean): Promise<void> {
  const query = store.recordsQuery
  await run(
    async () => {
      await bridge().clearCheckinRecords?.(all
        ? { all: '1' }
        : {
          outcome: query.outcome || undefined,
          kind: query.kind || undefined,
          provider: query.provider || undefined,
          keyword: query.keyword || undefined,
        })
    },
    { success: '签到记录已清空', reload: false, failure: '清空失败' },
  )
  await loadRecords()
}

/* ─── （WorkBuddy ）───────────── */

/** 1 （ no has to empty） */
export function growthOf(id: string): GrowthCache {
  return store.growth.get(id) ?? {
    status: 'idle',
    energy: null,
    streak: null,
    tasks: [],
    travel: null,
    school: null,
    running: false,
    runningSchool: false,
    results: [],
    error: '',
  }
}

function patchGrowth(id: string, next: Partial<GrowthCache>): void {
  const map = new Map(store.growth)
  map.set(id, { ...growthOf(id), ...next })
  patch({ growth: map })
}

/** 1 1 （ ， and old 1 ） */
function normalizeGrowthTask(raw: unknown): GrowthTask | null {
  const row = (raw ?? {}) as Record<string, unknown>
  const code = typeof row.code === 'string' ? row.code : ''
  if (!code) return null
  return {
    code,
    title: typeof row.title === 'string' && row.title ? row.title : code,
    description: typeof row.description === 'string' ? row.description : '',
    status: typeof row.status === 'string' ? row.status : '',
    current: Number(row.current) || 0,
    target: Number(row.target) || 0,
    locked: row.locked === true,
    supported: row.supported === true,
    unforgeable: row.unforgeable === true,
    kind: typeof row.kind === 'string' ? row.kind : '',
    source: typeof row.source === 'string' ? row.source : '',
  }
}

/** 1 （， 1 up ） */
export async function queryGrowth(id: string): Promise<void> {
  const current = growthOf(id)
  if (current.status === 'loading' || current.running) return
  patchGrowth(id, { status: 'loading', error: '' })
  try {
    const data = await bridge().getGrowthStatus?.(id)
    const tasksBlock = data?.tasks
    patchGrowth(id, {
      status: 'loaded',
      checkedAt: Date.now(),
      energy: data?.energy === null || data?.energy === undefined ? null : Number(data.energy),
      streak: data?.streak === null || data?.streak === undefined ? null : Number(data.streak),
      tasks: (Array.isArray(tasksBlock?.tasks) ? tasksBlock.tasks : [])
        .map(normalizeGrowthTask)
        .filter((task): task is GrowthTask => task !== null),
      travel: data?.travel ?? null,
      school: data?.school ?? null,
      // failure in up （ its has can ， not this ）
      error: typeof tasksBlock?.error === 'string' ? tasksBlock.error : '',
    })
  } catch (error) {
    patchGrowth(id, { status: 'error', error: errorMessage(error) })
  }
}

/**
 * （`full` = + ；`claim` = already ）。
 *
 * ── Why await ────────────────────────────
 * 1 request will API 60 seconds when ： after 1s+ 
 * （ `api::growth` module header）。 this in ， 1 
 * to —— use can to ， not is 1 「 in 」。
 */
export async function runGrowth(id: string, mode: 'full' | 'claim'): Promise<void> {
  const current = growthOf(id)
  if (current.running) return
  const targets = current.tasks.filter(task => growthTaskRunnable(task, mode))
  if (targets.length === 0) {
    toast('没有需要执行的任务')
    return
  }
  patchGrowth(id, { running: true, results: [], error: '' })
  let failed = 0
  const results: GrowthCache['results'] = []
  for (const task of targets) {
    try {
      const row = await bridge().runGrowthTask?.(id, task.code, mode)
      results.push({
        code: task.code,
        outcome: row?.outcome ?? 'done',
        message: row?.message ?? '',
      })
      if (row?.outcome === 'failed') failed += 1
    } catch (error) {
      failed += 1
      results.push({ code: task.code, outcome: 'failed', message: errorMessage(error) })
    }
    patchGrowth(id, { results: [...results] })
  }
  patchGrowth(id, { running: false })
  toast(failed > 0 ? `执行完成，${failed} 项失败` : '✅ 执行完成', failed > 0 ? 'err' : 'ok')
  await queryGrowth(id)
}

/** + */
export async function runSchool(id: string, lottery: boolean): Promise<void> {
  const current = growthOf(id)
  if (current.runningSchool) return
  patchGrowth(id, { runningSchool: true })
  try {
    const data = await bridge().runGrowthSchool?.(id, lottery)
    const message = typeof data?.message === 'string' && data.message
      ? data.message
      : (lottery ? '✅ 开学季任务已执行并抽奖' : '✅ 开学季任务已执行')
    toast(message)
  } catch (error) {
    toast(`执行失败：${errorMessage(error)}`, 'err')
  } finally {
    patchGrowth(id, { runningSchool: false })
    await queryGrowth(id)
  }
}

/** ： 1 （） */
export async function runTravelOnce(id: string): Promise<void> {
  try {
    const data = await bridge().runGrowthTravel?.(id)
    toast(typeof data?.message === 'string' && data.message ? data.message : '✅ 旅行巡检完成')
  } catch (error) {
    toast(`执行失败：${errorMessage(error)}`, 'err')
  } finally {
    await queryGrowth(id)
  }
}

/* ─── ─────────────────────────────── */

/**
 * 1 new `AutoState` and 。
 *
 * return is 「 when 」， 1 `checkin-center`
 * 1 ， and not will in in 1 down old 。
 */
function withAuto(center: CheckinCenter | null, auto: AutoState): CheckinCenter | null {
  if (!center) return center
  return { ...center, auto }
}
