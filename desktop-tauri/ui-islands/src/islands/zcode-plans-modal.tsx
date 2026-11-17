import * as React from 'react'
import { createRoot } from 'react-dom/client'
import {
  Badge, Button, Collapsible, CollapsibleContent, CollapsibleTrigger, Dialog, DialogBody,
  DialogContent, DialogFooter, DialogHeader, DialogTitle, Spinner, cn,
} from '@ui'
import { t } from '../i18n'
import { claimedPlanIdsToday, zcodePlanStateLabel } from './accounts-domain'
import { refreshUsageAfterCheckin, usageEntryOf } from './accounts-data'
import { findAccount, getStore, subscribe } from './accounts-store'
import { errorMessage, shared } from './accounts-shared'
import { ZcodeCaptchaStatus } from './accounts-zcode-plan'
import { planActivityEnded, planClaimResultApplies, planClaimState, planQueryErrors, unconfirmedPlanIds, type ClaimResult, type Entitlement, type OwnedPlan, type PreviewResult } from '../../../ui-shared/zcode-plans'
import { useDisplayClock } from '../../../ui-shared/use-display-clock'

/** 本期活动目录和账号领取状态由插件合并；余额明细仍来自账号余额查询。 */

/* ─── 桥接 ───────────────────────────────────── */

/** `wbZcodeClaim`（ui/zcode-claim.js）的公开面 */
type ClaimBridge = {
  preview?: (accountId: string) => Promise<unknown>
  claim?: (accountId: string, planId: string, signal?: AbortSignal, onPhase?: (phase: string) => void) => Promise<unknown>
}

/** 余额读数里「这条读数来自哪个通道」的取值：billing 通道才有套餐清单 */
const SOURCE_BILLING = 'zcode.z.ai/billing'

/** 三态 → 徽章的语义色（文案走域层的 `zcodePlanStateLabel`，三处共用一份译法） */
const STATE_VARIANT: Record<string, 'success' | 'warning' | 'outline' | 'ghost'> = {
  active: 'success',
  pending: 'warning',
  expired: 'ghost',
  unknown: 'outline',
}

/* ─── 小工具 ─────────────────────────────────── */

/** 权益的周期 → 给人看的一小段（与后端 `period_label` 同一套措辞） */
function periodLabel(period: unknown): string {
  const value = String(period || '').trim().toLowerCase()
  if (value === 'daily') return t('Daily')
  if (value === 'weekly') return t('Weekly')
  if (value === 'monthly') return t('Monthly')
  return ''
}

/** 数值 → 紧凑读数（万 / 亿，或英文的 k / M）：量级词在 units.js 一处，本弹窗不另写 */
function compactNumber(value: unknown): string {
  const number = Number(value)
  if (!Number.isFinite(number) || number <= 0) return ''
  const api = (window as unknown as { wbUnits?: { formatTokens?: (value: unknown) => string } }).wbUnits
  return api?.formatTokens ? api.formatTokens(number) : String(number)
}

/** unix 秒 → 本地时间串（后端给的是秒；`wbApp.formatTime` 收毫秒，这里自己转） */
function stampText(seconds: unknown): string {
  const value = Number(seconds)
  if (!Number.isFinite(value) || value <= 0) return ''
  const text = shared().wbApp?.formatTime?.(value * 1000)
  return text || new Date(value * 1000).toLocaleString()
}

/**
 * unix 秒 → 「剩 2 天」这类相对读数（已过期返回空串）。
 *
 * 相对读数排在精确时刻后面、但信息量更大 —— 用户想知道的是「还来得及吗」，
 * 不是「哪一秒到期」。
 */
function remainText(seconds: unknown): string {
  const value = Number(seconds)
  if (!Number.isFinite(value) || value <= 0) return ''
  const diff = value * 1000 - Date.now()
  if (diff <= 0) return ''
  const days = Math.floor(diff / 86_400_000)
  const hours = Math.floor((diff % 86_400_000) / 3_600_000)
  if (days >= 1) return t('{days} days left', { days })
  if (hours >= 1) return t('{hours} hours left', { hours })
  return t('{minutes} minutes left', { minutes: Math.max(1, Math.floor(diff / 60_000)) })
}

/** `到期 11-09 09:00（剩 2 天）` —— 没有相对读数时不硬塞一对空括号 */
function expireText(seconds: unknown): string {
  if (!seconds) return ''
  const remain = remainText(seconds)
  return remain
    ? t('Expires {time}({remain})', { time: stampText(seconds), remain })
    : t('Expires {time}', { time: stampText(seconds) })
}

/** 权益 → 一行可读串（`GLM-5.3-Flash 1亿 token（每日）`） */
function entitlementText(item: Entitlement): string {
  const quota = compactNumber(item.grantUnits)
  const unit = String(item.unitType || '').trim()
  const amount = quota ? [quota, unit].filter(Boolean).join(' ') : ''
  const period = periodLabel(item.period)
  const body = [String(item.showName || '').trim(), amount].filter(Boolean).join(' ')
  if (!body) return ''
  return period ? t('{body} ({period})', { body, period }) : body
}

/** 一份套餐的权益行（没有可读权益时返回空串，调用处据此省略那一行） */
function entitlementLine(plan: { entitlements?: Entitlement[] }): string {
  return (plan.entitlements || []).map(entitlementText).filter(Boolean).join(t(', '))
}

/** 一份套餐的时间窗（`开始 → 到期`；只有一个时只说那一个） */
function windowText(plan: { startsAt?: number | null; endsAt?: number | null }): string {
  const start = stampText(plan.startsAt)
  const end = stampText(plan.endsAt)
  if (start && end) return `${start} → ${end}`
  if (end) return t('Expires {time}', { time: end })
  if (start) return t('Starts {time}', { time: start })
  return ''
}

/**
 * 生效时间的展示串：**今天 / 明天**用相对日（官方客户端同一份数据就是显示
 * 「待生效 今天 23:00」），更远的日子给本地日期时间。
 */
function effectiveText(seconds: number): string {
  const date = new Date(seconds * 1000)
  if (Number.isNaN(date.getTime())) return ''
  const time = `${String(date.getHours()).padStart(2, '0')}:${String(date.getMinutes()).padStart(2, '0')}`
  // 本地日历日序号（跨时区 / 跨夏令时都按「本机的一天」算，与官方同一算法）
  const dayIndex = (value: Date) => Math.floor(
    Date.UTC(value.getFullYear(), value.getMonth(), value.getDate()) / 86_400_000,
  )
  const diff = dayIndex(date) - dayIndex(new Date())
  if (diff === 0) return t('today {time}', { time })
  if (diff === 1) return t('tomorrow {time}', { time })
  return stampText(seconds)
}

/**
 * 名下套餐的时间行：**待生效在前、到期在后**（`待生效 今天 23:00 · 到期 10月12日 09:00`）。
 *
 * ── 为什么不用 `startsAt`（套餐级开始时间）────────────────────
 * 那是**领取 / 购买时间**。官方 Start Plan 卡片只展示「待生效 {生效时间}」与
 * 「过期时间 {到期}」两样（见官方 `CodingPlanStatusMeta`），从不展示套餐级
 * `starts_at` —— 早先我们拿它当生效时间显示，于是「领取那一刻」被读成了生效
 * 时间（同一份套餐官方客户端写的是「待生效 今天 23:00」）。
 * `pendingUntil` 由后端按官方口径算好（最早一条还没到点的**权益生效时间**）；
 * 只有它和 `endsAt` 都缺时才退回 `startsAt` 兜一句。
 */
function planTimeText(plan: {
  startsAt?: number | null
  endsAt?: number | null
  pendingUntil?: number | null
}): string {
  const parts: string[] = []
  const pending = Number(plan.pendingUntil)
  if (Number.isFinite(pending) && pending > 0) {
    const text = effectiveText(pending)
    if (text) parts.push(t('Effective from {time}', { time: text }))
  }
  const end = stampText(plan.endsAt)
  if (end) parts.push(t('Expires {time}', { time: end }))
  if (parts.length === 0) {
    const start = stampText(plan.startsAt)
    if (start) parts.push(t('Starts {time}', { time: start }))
  }
  return parts.join(' · ')
}

/* ─── 主体 ───────────────────────────────────── */

type OpenOptions = { id: string; name?: string }

type PreviewState =
  | { status: 'loading' }
  | { status: 'ready'; result: PreviewResult }
  | { status: 'error'; message: string }

function PlansModal({ options, onClose }: { options: OpenOptions; onClose: () => void }) {
  const id = options.id
  const now = useDisplayClock()
  // 账号记录的订阅：领取台账（`claimPlans`）与余额读数都可能被别处改（后台自动
  // 查询、别处领取），弹窗开着时跟着重画，不然「今日已领」会停在打开那一刻。
  const [, setTick] = React.useState(0)
  React.useEffect(() => subscribe(() => setTick(value => value + 1)), [])
  // 本地「刚领过」的集合：领取成功后不等账号刷新回来就先把状态标上
  const [claimedNow, setClaimedNow] = React.useState<string[]>([])
  const [preview, setPreview] = React.useState<PreviewState>({ status: 'loading' })
  const [receiptError, setReceiptError] = React.useState('')
  const [claiming, setClaiming] = React.useState('')
  const [claimPhase, setClaimPhase] = React.useState('')
  const [claimResults, setClaimResults] = React.useState<Record<string, ClaimResult>>({})
  const [verifyingChannel, setVerifyingChannel] = React.useState(false)
  // 「数据来源与判定说明」默认收起：它解释的是我们怎么算的，不是当前状态
  const [notesOpen, setNotesOpen] = React.useState(false)
  const previewRequest = React.useRef(0)
  const claimController = React.useRef<AbortController | null>(null)
  const mounted = React.useRef(true)
  React.useEffect(() => {
    mounted.current = true
    return () => { mounted.current = false; claimController.current?.abort() }
  }, [])

  const account = findAccount(id)
  const name = String(account?.name || options.name || id)
  const stored = claimedPlanIdsToday(account)
  const claimed = [...new Set([...stored, ...claimedNow])]
  const ledger = account?.claimPlans && typeof account.claimPlans === 'object' && !Array.isArray(account.claimPlans)
    ? account.claimPlans as Record<string, unknown>
    : {}
  const entry = account ? usageEntryOf(account) : undefined
  const data = entry && typeof entry === 'object' && !Array.isArray(entry)
    ? entry as Record<string, unknown>
    : null
  const previewPlans = preview.status === 'ready' ? preview.result.plans || [] : []
  const activityPlans = previewPlans.filter(plan => !planActivityEnded(plan, now))
  const queryErrors = preview.status === 'ready' ? planQueryErrors(preview.result) : []
  const balancePlans = data && Array.isArray(data.plans) ? data.plans as OwnedPlan[] : []
  const ownedPlans = [...balancePlans]
  for (const plan of previewPlans) {
    const owned = plan.ownedPlan
    if (owned && !ownedPlans.some(item => owned.userPlanId
      ? item.userPlanId === owned.userPlanId : item.planId === owned.planId)) ownedPlans.push(owned)
  }
  const unconfirmed = unconfirmedPlanIds(claimed, previewPlans, ownedPlans, now)
  const inflight = getStore().usageInflight.has(id)
  const bridge = shared().wbZcodeClaim as ClaimBridge | undefined

  /** 探测一次可领取清单（打开时、失败重试、领取成功后各一次） */
  const loadPreview = React.useCallback(async () => {
    if (!bridge?.preview) {
      setPreview({ status: 'error', message: t('Claiming is not supported in this environment (bridge method missing)') })
      return
    }
    const request = ++previewRequest.current
    setPreview({ status: 'loading' })
    try {
      const result = await bridge.preview(id) as PreviewResult
      if (mounted.current && request === previewRequest.current) {
        setPreview({ status: 'ready', result: result || {} })
        void window.wbCheckin?.load?.()
      }
    } catch (error) {
      if (mounted.current && request === previewRequest.current) setPreview({ status: 'error', message: errorMessage(error) })
    }
  }, [bridge, id])

  // 打开即探测（可领取清单是**免费**的一次上游读，见 ui/zcode-claim.js）；
  // 同时：还没有余额读数就静默补查一次单账号 —— 否则「我的套餐」这一段第一次
  // 打开永远是空的，而它恰恰是这次改动的重点。
  React.useEffect(() => {
    void loadPreview()
    // 「还没有读数」才补查（`usageEntryOf` 对 null 账号返回 undefined 也无妨：
    // 账号不在列表里时本来也查不了）
    const fresh = findAccount(id)
    if (fresh && usageEntryOf(fresh) === undefined) void refreshUsageAfterCheckin(id)
    // 只在打开时跑一次（挂载 = 打开，重复打开会先卸载再挂）
  }, [id, loadPreview])

  /** 领一份：验证码与请求在 ui/zcode-claim.js，这里只收结果并刷新 */
  async function claim(planId: string): Promise<void> {
    if (!bridge?.claim || claimController.current || verifyingChannel) return
    const plan = activityPlans.find(item => String(item.planId || '') === planId)
    if (!plan || planActivityEnded(plan)) return
    const controller = new AbortController()
    claimController.current = controller
    setClaiming(planId)
    setClaimPhase('preparing')
    try {
      const result = await bridge.claim(id, planId, controller.signal, phase => {
        if (mounted.current) setClaimPhase(phase)
      }) as ClaimResult | null
      if (!mounted.current || controller.signal.aborted) return
      if (result) setClaimResults(results => ({ ...results, [result.planId || planId]: result }))
      // 「已领过」同样是既成事实：本地标上、并重探一次（上游此刻的清单才是事实）
      if (result?.ok || result?.failure === 'already_claimed') {
        setReceiptError(result.marketingError || '')
        const settled = result.planId || planId
        setClaimResults(results => ({ ...results, [settled]: {
          ...result, claimedAt: result.claimedAt || Date.now(),
        } }))
        if (settled) setClaimedNow(list => (list.includes(settled) ? list : [...list, settled]))
        // 余额静默刷新（不 await：toast 是单例，领取结果那条不能被顶掉）
        void refreshUsageAfterCheckin(id)
        void shared().wbApp?.refresh?.()
        void window.wbCheckin?.load?.()
        void loadPreview()
      }
    } finally {
      if (claimController.current === controller) claimController.current = null
      if (mounted.current) { setClaiming(''); setClaimPhase('') }
    }
  }

  function close() {
    if (claimPhase === 'submitting') return
    claimController.current?.abort()
    onClose()
  }

  return (
    <Dialog
      open
      modal={!claiming && !verifyingChannel}
      onOpenChange={(next, eventDetails) => {
        if (next) return
        if (claimPhase === 'submitting' || ((claiming || verifyingChannel) && ['outside-press', 'focus-out'].includes(eventDetails.reason))) {
          eventDetails.cancel()
          return
        }
        close()
      }}
    >
      <DialogContent showCloseButton={claimPhase !== 'submitting'} className='w-[min(520px,calc(100vw-48px))]'>
        <DialogHeader className='items-start'>
          <div className='min-w-0'>
            <DialogTitle>{t('ZCode Plan details')}</DialogTitle>
            <div className='mt-1 truncate text-[11.5px] text-muted-foreground'>{name}</div>
          </div>
        </DialogHeader>
        <DialogBody className='gap-5'>
          {/*
            本期活动排在最上：打开这个弹窗十有八九就是为了领它。
            通道状态（人机验证）降级成弹窗底部的一行 —— 它是低频信息，
            旧版却为它在最前面铺了三行正文。
          */}
          <section>
            <div className='mb-2 flex items-center gap-2'>
              <h3 className='m-0 text-[12.5px] font-semibold'>{t('Current events')}</h3>
              <Badge variant='outline' shape='tag'>{t('Official limited-time event')}</Badge>
              <Button variant='ghost' size='xs' className='ml-auto'
                disabled={preview.status === 'loading' || Boolean(claiming) || verifyingChannel}
                onClick={() => void loadPreview()}>{t('Refresh')}</Button>
            </div>
            {preview.status === 'loading' ? (
              <div className='flex items-center gap-2 py-2 text-[12.5px] text-muted-foreground'>
                <Spinner className='size-3' />{t('Checking current events…')}
              </div>
            ) : preview.status === 'error' ? (
              <div className='py-1 text-[12.5px] text-destructive'>
                {t('Probe failed: {message}', { message: preview.message })}
                <Button variant='outline' size='xs' className='ml-2'
                  onClick={() => void loadPreview()}>{t('Retry')}</Button>
              </div>
            ) : activityPlans.length === 0 ? (
              <p className='py-1 text-[12.5px] text-muted-foreground'>
                {queryErrors.length
                  ? t('Some sources could not be queried, so current events or claim status cannot be confirmed yet.')
                  : t('No current events were found. This does not mean the account is ineligible.')}
              </p>
            ) : (
              <div className='flex flex-col gap-2'>
                {activityPlans.map(plan => {
                  const planId = String(plan.planId || '')
                  const savedResult = claimResults[planId]
                  const result = planClaimResultApplies(plan, savedResult) ? savedResult : undefined
                  const locallySettled = result?.ok || result?.failure === 'already_claimed'
                  const state = planClaimState(plan, locallySettled ? [planId] : plan.claimState ? [] : stored)
                  const taken = state === 'claimed'
                  const rejected = !taken && (result?.failure === 'ineligible' || state === 'ineligible')
                  const statusLabel = taken ? t('Claimed') : rejected ? t('Currently ineligible')
                    : state === 'unknown' ? t('Status unconfirmed') : t('Claimable')
                  const entitlements = entitlementLine(plan)
                  const owned = plan.ownedPlan
                  const window_ = windowText(plan)
                  const expiresAt = owned?.endsAt || plan.claimEndsAt || result?.endsAt
                  const message = result && !result.ok && result.failure !== 'already_claimed'
                    ? result.message || result.failureLabel : plan.claimMessage
                  // 时间行：待生效 / 活动窗口 / 到期各一段，用一行的距离说完
                  const times = [
                    owned?.pendingUntil ? t('Effective from {time}', { time: effectiveText(owned.pendingUntil) }) : '',
                    window_ ? t('The event period runs until {time}', { time: window_ }) : '',
                    expireText(expiresAt),
                  ].filter(Boolean).join(' · ')
                  return (
                    <div key={planId || plan.name} className='rounded-md border border-border bg-surface px-3.5 py-3'>
                      <div className='flex items-center gap-3'>
                        <div className='min-w-0 flex-1'>
                          <div className='flex flex-wrap items-center gap-2'>
                            <span className='truncate text-[13px] font-medium'>{plan.name || planId || t('Plan')}</span>
                            <Badge variant={taken ? 'success' : rejected ? 'warning' : 'outline'} shape='tag'>{statusLabel}</Badge>
                            {owned?.state && owned.state !== 'unknown' ? (
                              <Badge variant={STATE_VARIANT[owned.state] || 'outline'} shape='tag'>
                                {zcodePlanStateLabel(owned.state)}
                              </Badge>
                            ) : null}
                          </div>
                          {/* 额度是用户来这一趟要看的东西，给它主读数的字号 */}
                          {entitlements
                            ? <div className='mt-1.5 text-[15.5px] font-semibold tabular-nums'>{entitlements}</div>
                            : null}
                          {times ? <div className='mt-0.5 text-[12px] text-muted-foreground'>{times}</div> : null}
                          {plan.description ? <div className='text-[11.5px] text-muted-foreground'>{plan.description}</div> : null}
                          {message ? <div className='text-[11.5px] text-muted-foreground'>{message}</div> : null}
                        </div>
                        {/* 已领取就不再给按钮：徽章已经说明了，再挂一颗禁用按钮只是重复 */}
                        {!taken ? (
                          <Button variant={state === 'unknown' || rejected ? 'outline' : 'default'} size='sm'
                            className={cn('shrink-0', claiming === planId && 'pointer-events-none')}
                            disabled={claiming !== '' || verifyingChannel || !planId}
                            onClick={() => void claim(planId)}>
                            {claiming === planId ? t('Claiming…')
                              : state === 'unknown' || rejected ? t('Try claiming') : t('Claim')}
                          </Button>
                        ) : null}
                      </div>
                    </div>
                  )
                })}
              </div>
            )}
          </section>

          <section>
            <div className='mb-2 flex items-center gap-2'>
              <h3 className='m-0 text-[12.5px] font-semibold'>{t('My plans')}</h3>
              {/*
                可用读数只在**真读到额度**时显示。没有桶时 `availableView` 是
                「无额度」，挂在下面那些套餐旁边会被读成「这份套餐没额度」——
                而实际情况是「这个读数里没有桶」（额度可能记在活动套餐那条通道上，
                见后端 balance.rs 的 `available` 那段）。那种情况交给下面逐份列出的
                套餐说话。

                「可用」这个前缀只在 billing 通道的读数上加：它的 `availableView`
                是一个**量**（「1亿 token」）。监控通道（`walletsFrom`）给的是
                **窗口表头**（「每 5 小时剩 99%」），前面再挂「可用」会读成
                「可用 5 小时剩 99%」这种别扭句子 —— 直接摆表头即可。
              */}
              {Array.isArray(data?.wallets) && data.wallets.length > 0 && data.availableView ? (
                <Badge variant='outline' shape='tag'>
                  {String(data.walletsFrom || '') === 'monitor'
                    ? String(data.availableView)
                    : t('Available {value}', { value: String(data.availableView) })}
                </Badge>
              ) : null}
              <Button variant='ghost' size='xs' className='ml-auto' disabled={inflight}
                title={t('Re-read this account\'s balance and plans (goes through the balance query chain)')}
                onClick={() => void refreshUsageAfterCheckin(id)}>
                {inflight ? t('Checking…') : t('Query balance')}
              </Button>
            </div>
            {ownedPlans.length > 0 ? (
              <div className='flex flex-col gap-2'>
                {ownedPlans.map(plan => {
                  const planId = String(plan.planId || '')
                  const state = String(plan.state || 'unknown')
                  const entitlements = entitlementLine(plan)
                  // 时间行走 `planTimeText`（待生效 + 到期），**不用** `windowText`：
                  // 后者的前半段是套餐级 `startsAt` = 领取时间，官方卡片不展示它
                  const window_ = planTimeText(plan)
                  const claimedAt = planId && account?.claimPlans
                    ? Number((account.claimPlans as Record<string, unknown>)[planId]) || 0
                    : 0
                  return (
                    <div key={plan.userPlanId || planId || plan.name}
                      className='rounded-md border border-border bg-surface px-3.5 py-3'>
                      <div className='flex flex-wrap items-center gap-2'>
                        <span className='truncate text-[13px] font-medium'>{plan.name || planId || t('Plan')}</span>
                        <Badge variant={STATE_VARIANT[state] || 'outline'} shape='tag'>
                          {zcodePlanStateLabel(state)}
                        </Badge>
                        {claimedAt > 0 ? (
                          <span className='ml-auto text-[11.5px] text-muted-foreground'>
                            {t('Claimed locally at {time}', { time: stampText(claimedAt / 1000) })}
                          </span>
                        ) : null}
                      </div>
                      {entitlements
                        ? <div className='mt-1.5 text-[15.5px] font-semibold tabular-nums'>{entitlements}</div>
                        : null}
                      {window_ ? <div className='mt-0.5 text-[12px] text-muted-foreground'>{window_}</div> : null}
                    </div>
                  )
                })}
              </div>
            ) : (
              <p className='m-0 py-1 text-[12.5px] text-muted-foreground'>
                {entry === undefined
                  ? t('No balance reading yet (one query has been started automatically, please wait; you can also click "Check balance" at the top right)')
                  : data && data.source && data.source !== SOURCE_BILLING
                    ? t('This account reads its balance through the window-limit channel (only a Coding Plan API Key is configured): it only has the remaining percentage of each N-hour window, not the plan list')
                    : t('The reading contains no plan list (free account, or the upstream plan state is not refreshed yet; click "Check balance" at the top right to re-read)')}
              </p>
            )}
          </section>

          {/* 通道状态压成一行（人机验证），规则与判定口径统一收进下面的折叠区 */}
          <div className='border-t border-hairline pt-3.5'>
            <ZcodeCaptchaStatus accountId={id} disabled={Boolean(claiming)} onBusy={setVerifyingChannel} />
          </div>

          <Collapsible open={notesOpen} onOpenChange={setNotesOpen}>
            <CollapsibleTrigger
              className='h-auto w-full justify-start gap-1.5 px-0 py-0 text-[11.5px] font-normal text-muted-foreground'
            >
              <svg width='12' height='12' viewBox='0 0 12 12' fill='none' stroke='currentColor'
                strokeWidth={1.7} strokeLinecap='round' strokeLinejoin='round'
                className={cn('shrink-0 transition-transform duration-150', notesOpen && 'rotate-90')}
                aria-hidden='true'>
                <path d='M4.5 2.5 8 6l-3.5 3.5' />
              </svg>
              {t('Data sources and judgment notes')}
            </CollapsibleTrigger>
            <CollapsibleContent>
              <div className='mt-2 flex flex-col gap-1.5 pl-[18px] text-[11.5px] leading-relaxed text-muted-foreground'>
                <p className='m-0'>
                  {t('The event catalog combines account previews, marketing events, and anonymous discovery. Claimed status requires a claim record or an account-owned plan. If the status is unconfirmed, you can try claiming; eligibility is determined by the upstream response.')}
                </p>
                <p className='m-0'>
                  {t('The background only reads status and never opens verification automatically. Each token is single-use and expires shortly.')}
                </p>
                {preview.status === 'ready' && queryErrors.length > 0 ? (
                  <p className='m-0'>
                    {t('Some sources could not be queried; other results were retained: {message}', { message: queryErrors.join('; ') })}
                  </p>
                ) : null}
                {receiptError ? (
                  <p className='m-0'>
                    {t('The plan has been claimed, but the marketing receipt is incomplete; it will not be claimed again: {message}', { message: receiptError })}
                  </p>
                ) : null}
                {preview.status === 'ready' && preview.result.activationError ? (
                  <p className='m-0'>
                    {t('Failed to backfill activation events; this may affect claim eligibility: {message}', { message: preview.result.activationError })}
                  </p>
                ) : null}
                {claimed.length > 0 ? (
                  <p className='m-0'>
                    {t('Claimed on this machine today (Beijing time): {plans}', { plans: claimed.join(t(', ')) })}
                  </p>
                ) : null}
                {unconfirmed.length > 0 ? (
                  <p className='m-0'>
                    {t('Also claimed today according to the local record, but not in the readings above yet: {plans} (the reading may take a while to refresh)', {
                      plans: unconfirmed.map(planId => {
                        const at = Number(ledger[planId]) || 0
                        const time = at > 0 ? stampText(at / 1000) : ''
                        return time ? t('{plan} (claimed {time})', { plan: planId, time }) : planId
                      }).join(t(', ')),
                    })}
                  </p>
                ) : null}
              </div>
            </CollapsibleContent>
          </Collapsible>
        </DialogBody>
        <DialogFooter>
          <span className='min-w-0 text-[11.5px] text-muted-foreground'>
            {claimed.length > 0
              ? t('Claimed today (Beijing time): {n} items', { n: claimed.length })
              : t('Current events are listed by plan. Eligibility is determined by the upstream response.')}
          </span>
          <div className='mr-auto' />
          {claiming ? <Button variant='outline' disabled={claimPhase === 'submitting'}
            onClick={() => claimController.current?.abort()}>{t('Cancel verification')}</Button> : null}
          <Button variant='outline' disabled={claimPhase === 'submitting'} onClick={close}>{t('Close')}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

/* ─── 命令式外壳 ─────────────────────────────── */

let root: ReturnType<typeof createRoot> | null = null
let host: HTMLElement | null = null

function unmountModal() {
  if (root) { root.unmount(); root = null }
  if (host) { host.remove(); host = null }
}

function openModal(options: OpenOptions) {
  if (!options?.id) return
  unmountModal() // 重复打开先拆掉上一份（与 confirm-dialog / models-fetch-modal 同一手法）
  host = document.createElement('div')
  document.body.append(host)
  root = createRoot(host)
  root.render(<PlansModal options={options} onClose={unmountModal} />)
}

declare global {
  interface Window {
    /**
     * ZCode 套餐明细弹窗。`id` 是账号 id（必填）；`name` 只在账号记录取不到时
     * 当显示名的兜底 —— 「今天领过哪几份」与余额读数都由弹窗自己从账号页状态里读
     * （判据只有 `claimedPlanIdsToday` 一处），调用方不必先算好再传。
     */
    wbZcodePlans?: { open(options: OpenOptions): void }
  }
}

window.wbZcodePlans = { open: openModal }
