import * as React from 'react'
import { createRoot } from 'react-dom/client'
import { Badge, Button, Dialog, DialogBody, DialogContent, DialogFooter, DialogHeader, DialogTitle, Spinner } from '@ui'
import { t } from '../i18n'
import { claimedPlanIdsToday, zcodePlanStateLabel } from './accounts-domain'
import { refreshUsageAfterCheckin, usageEntryOf } from './accounts-data'
import { findAccount, getStore, subscribe } from './accounts-store'
import { errorMessage, shared } from './accounts-shared'
import { ZcodeCaptchaStatus } from './accounts-zcode-plan'
import { planClaimResultApplies, planClaimState, planQueryErrors, type ClaimResult, type Entitlement, type OwnedPlan, type PreviewResult } from '../../../ui-shared/zcode-plans'

/** directory and by and ；。 */

/* ─── ───────────────────────────────────── */

/** `wbZcodeClaim`（ui/zcode-claim.js） */
type ClaimBridge = {
  preview?: (accountId: string) => Promise<unknown>
  claim?: (accountId: string, planId: string, signal?: AbortSignal, onPhase?: (phase: string) => void) => Promise<unknown>
}

/** 「 this 」 ：billing has */
const SOURCE_BILLING = 'zcode.z.ai/billing'

/** → （ `zcodePlanStateLabel`， use 1 ） */
const STATE_VARIANT: Record<string, 'success' | 'warning' | 'outline' | 'ghost'> = {
  active: 'success',
  pending: 'warning',
  expired: 'ghost',
  unknown: 'outline',
}

/* ─── ─────────────────────────────────── */

/** → to 1 （ and after `period_label` 1 ） */
function periodLabel(period: unknown): string {
  const value = String(period || '').trim().toLowerCase()
  if (value === 'daily') return t('每日')
  if (value === 'weekly') return t('每周')
  if (value === 'monthly') return t('每月')
  return ''
}

/** → （ / ， or k / M）： in units.js 1 ， not */
function compactNumber(value: unknown): string {
  const number = Number(value)
  if (!Number.isFinite(number) || number <= 0) return ''
  const api = (window as unknown as { wbUnits?: { formatTokens?: (value: unknown) => string } }).wbUnits
  return api?.formatTokens ? api.formatTokens(number) : String(number)
}

/** unix seconds → when （ after to is seconds；`wbApp.formatTime` ms， this ） */
function stampText(seconds: unknown): string {
  const value = Number(seconds)
  if (!Number.isFinite(value) || value <= 0) return ''
  const text = shared().wbApp?.formatTime?.(value * 1000)
  return text || new Date(value * 1000).toLocaleString()
}

/** → 1 can （`GLM-5.3-Flash 1 token（daily）`） */
function entitlementText(item: Entitlement): string {
  const quota = compactNumber(item.grantUnits)
  const unit = String(item.unitType || '').trim()
  const amount = quota ? [quota, unit].filter(Boolean).join(' ') : ''
  const period = periodLabel(item.period)
  const body = [String(item.showName || '').trim(), amount].filter(Boolean).join(' ')
  if (!body) return ''
  return period ? t('{body}（{period}）', { body, period }) : body
}

/** 1 （ no has can when returnempty， use that 1 ） */
function entitlementLine(plan: { entitlements?: Entitlement[] }): string {
  return (plan.entitlements || []).map(entitlementText).filter(Boolean).join(t('、'))
}

/** 1 when （` → to `； has 1 when that 1 ） */
function windowText(plan: { startsAt?: number | null; endsAt?: number | null }): string {
  const start = stampText(plan.startsAt)
  const end = stampText(plan.endsAt)
  if (start && end) return `${start} → ${end}`
  if (end) return t('到期 {time}', { time: end })
  if (start) return t('开始 {time}', { time: start })
  return ''
}

/**
 * when ：** / ** use for （client 1 is 
 * 「 23:00」）， to when 。
 */
function effectiveText(seconds: number): string {
  const date = new Date(seconds * 1000)
  if (Number.isNaN(date.getTime())) return ''
  const time = `${String(date.getHours()).padStart(2, '0')}:${String(date.getMinutes()).padStart(2, '0')}`
  // （ when / when by 「 1 」， and 1 ）
  const dayIndex = (value: Date) => Math.floor(
    Date.UTC(value.getFullYear(), value.getMonth(), value.getDate()) / 86_400_000,
  )
  const diff = dayIndex(date) - dayIndex(new Date())
  if (diff === 0) return t('今天 {time}', { time })
  if (diff === 1) return t('明天 {time}', { time })
  return stampText(seconds)
}

/**
 * down when ：** in before 、 to in after **（` 23:00 · to 1012 09:00`）。
 *
 * ── Why not use `startsAt`（ when ）────────────────────
 * that is ** / when **。 Start Plan 「 { when }」 and 
 * 「expiration time { to }」（ `CodingPlanStatusMeta`）， from not 
 * `starts_at` —— when ， is 「 that 1 」 
 * when （ 1 client is 「 23:00」）。
 * `pendingUntil` by after by （ 1 no to ** when **）；
 * has and `endsAt` when `startsAt` 1 。
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
    if (text) parts.push(t('待生效 {time}', { time: text }))
  }
  const end = stampText(plan.endsAt)
  if (end) parts.push(t('到期 {time}', { time: end }))
  if (parts.length === 0) {
    const start = stampText(plan.startsAt)
    if (start) parts.push(t('开始 {time}', { time: start }))
  }
  return parts.join(' · ')
}

/* ─── ───────────────────────────────────── */

type OpenOptions = { id: string; name?: string }

type PreviewState =
  | { status: 'loading' }
  | { status: 'ready'; result: PreviewResult }
  | { status: 'error'; message: string }

function PlansModal({ options, onClose }: { options: OpenOptions; onClose: () => void }) {
  const id = options.id
  // ：（`claimPlans`） and can can （ after 
  // 、）， when ， not 「 already 」 will in that 1 。
  const [, setTick] = React.useState(0)
  React.useEffect(() => subscribe(() => setTick(value => value + 1)), [])
  // 「」 ：success after not etc new up 
  const [claimedNow, setClaimedNow] = React.useState<string[]>([])
  const [preview, setPreview] = React.useState<PreviewState>({ status: 'loading' })
  const [receiptError, setReceiptError] = React.useState('')
  const [claiming, setClaiming] = React.useState('')
  const [claimPhase, setClaimPhase] = React.useState('')
  const [claimResults, setClaimResults] = React.useState<Record<string, ClaimResult>>({})
  const [verifyingChannel, setVerifyingChannel] = React.useState(false)
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
  const activityPlans = preview.status === 'ready' ? preview.result.plans || [] : []
  const queryErrors = preview.status === 'ready' ? planQueryErrors(preview.result) : []
  const balancePlans = data && Array.isArray(data.plans) ? data.plans as OwnedPlan[] : []
  const ownedPlans = [...balancePlans]
  for (const plan of activityPlans) {
    const owned = plan.ownedPlan
    if (owned && !ownedPlans.some(item => owned.userPlanId
      ? item.userPlanId === owned.userPlanId : item.planId === owned.planId)) ownedPlans.push(owned)
  }
  // 、 up no has that id。**Why need has this 1 **：
  // to that 1 can can not will in up （ need new 、 up can can 
  // after this to up ）， that 「 to not to 」 will 。
  // this 1 —— has can 「 to 、 when 」。
  const unconfirmed = claimed.filter(planId => planId
    && !ownedPlans.some(plan => String(plan.planId || '') === planId))
  const inflight = getStore().usageInflight.has(id)
  const bridge = shared().wbZcodeClaim as ClaimBridge | undefined

  /** 1 can （ when 、failure、success after 1 ） */
  const loadPreview = React.useCallback(async () => {
    if (!bridge?.preview) {
      setPreview({ status: 'error', message: t('当前环境不支持领取（桥接方法缺失）') })
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

  // （ can is **** 1 up ， ui/zcode-claim.js）；
  // when ： no has 1 —— otherwise「 」 this 1 1 
  // is empty ， is this 。
  React.useEffect(() => {
    void loadPreview()
    // 「 no has 」（`usageEntryOf` for null return undefined none：
    // not in when not ）
    const fresh = findAccount(id)
    if (fresh && usageEntryOf(fresh) === undefined) void refreshUsageAfterCheckin(id)
    // in when 1 （ = ， will ）
  }, [id, loadPreview])

  /** 1 ： and request in ui/zcode-claim.js， this and new */
  async function claim(planId: string): Promise<void> {
    if (!bridge?.claim || claimController.current || verifyingChannel) return
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
      // 「 already 」 is ： up 、 and 1 （ up is ）
      if (result?.ok || result?.failure === 'already_claimed') {
        setReceiptError(result.marketingError || '')
        const settled = result.planId || planId
        setClaimResults(results => ({ ...results, [settled]: {
          ...result, claimedAt: result.claimedAt || Date.now(),
        } }))
        if (settled) setClaimedNow(list => (list.includes(settled) ? list : [...list, settled]))
        // new （ not await：toast is ， that not can ）
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
      <DialogContent showCloseButton={claimPhase !== 'submitting'} className='w-[min(560px,calc(100vw-48px))]'>
        <DialogHeader>
          <DialogTitle>{t('ZCode 套餐明细 — {name}', { name })}</DialogTitle>
        </DialogHeader>
        <DialogBody className='space-y-4'>
          <section>
            <div className='text-[12px] font-semibold text-muted-foreground'>{t('套餐池人机验证')}</div>
            <ZcodeCaptchaStatus accountId={id} disabled={Boolean(claiming)} onBusy={setVerifyingChannel} />
          </section>
          <section>
            <div className='mb-1.5 flex items-center gap-2 text-[12px] font-semibold text-muted-foreground'>
              {t('本期活动')}
              <Badge variant='outline' shape='tag'>{t('官方限时活动')}</Badge>
              <Button variant='outline' size='xs' className='ml-auto'
                disabled={preview.status === 'loading' || Boolean(claiming) || verifyingChannel}
                onClick={() => void loadPreview()}>{t('刷新')}</Button>
            </div>
            {preview.status === 'loading' ? (
              <div className='flex items-center gap-2 py-2 text-[12.5px] text-muted-foreground'>
                <Spinner className='size-3' />{t('正在查询本期活动…')}
              </div>
            ) : preview.status === 'error' ? (
              <div className='py-1 text-[12.5px] text-destructive'>
                {t('探测失败：{message}', { message: preview.message })}
                <Button variant='outline' size='xs' className='ml-2'
                  onClick={() => void loadPreview()}>{t('重试')}</Button>
              </div>
            ) : activityPlans.length === 0 ? (
              <p className='py-1 text-[12.5px] text-muted-foreground'>
                {queryErrors.length
                  ? t('部分来源查询失败，暂不能确认本期活动或领取状态')
                  : t('当前未发现本期活动，不代表账号不符合资格')}
              </p>
            ) : (
              <ul className='m-0 list-none p-0'>
                {activityPlans.map(plan => {
                  const planId = String(plan.planId || '')
                  const savedResult = claimResults[planId]
                  const result = planClaimResultApplies(plan, savedResult) ? savedResult : undefined
                  const locallySettled = result?.ok || result?.failure === 'already_claimed'
                  const state = planClaimState(plan, locallySettled ? [planId] : plan.claimState ? [] : stored)
                  const taken = state === 'claimed'
                  const rejected = !taken && (result?.failure === 'ineligible' || state === 'ineligible')
                  const statusLabel = taken ? t('已领取') : rejected ? t('当前不符合资格')
                    : state === 'unknown' ? t('状态待确认') : t('可领取')
                  const entitlements = entitlementLine(plan)
                  const owned = plan.ownedPlan
                  const window_ = windowText(plan)
                  const expiresAt = owned?.endsAt || plan.claimEndsAt || result?.endsAt
                  const message = result && !result.ok && result.failure !== 'already_claimed'
                    ? result.message || result.failureLabel : plan.claimMessage
                  return (
                    <li key={planId || plan.name}
                      className='flex items-start justify-between gap-3 border-b border-hairline py-2 last:border-b-0'>
                      <div className='min-w-0'>
                        <div className='flex items-center gap-2 text-[13px] font-medium'>
                          <span className='truncate'>{plan.name || planId || t('套餐')}</span>
                          <Badge variant={taken ? 'success' : rejected ? 'warning' : 'outline'} shape='tag'>{statusLabel}</Badge>
                          {owned?.state ? <Badge variant={STATE_VARIANT[owned.state] || 'outline'} shape='tag'>
                            {zcodePlanStateLabel(owned.state)}
                          </Badge> : null}
                        </div>
                        {owned?.pendingUntil ? <div className='text-[12px] text-muted-foreground'>
                          {t('待生效 {time}', { time: effectiveText(owned.pendingUntil) })}
                        </div> : null}
                        {window_ ? <div className='text-[12px] text-muted-foreground'>
                          {t('活动时间：{time}', { time: window_ })}
                        </div> : null}
                        {expiresAt ? <div className='text-[12px] text-muted-foreground'>
                          {t('账号套餐到期：{time}', { time: stampText(expiresAt) })}
                        </div> : null}
                        {message ? <div className='text-[12px] text-muted-foreground'>{message}</div> : null}
                        {entitlements ? <div className='text-[12px] text-muted-foreground'>{entitlements}</div> : null}
                        {plan.description ? <div className='text-[12px] text-muted-foreground'>{plan.description}</div> : null}
                      </div>
                      <Button variant='outline' size='xs' className='flex-none'
                        disabled={claiming !== '' || verifyingChannel || taken || !planId}
                        onClick={() => void claim(planId)}>
                        {claiming === planId ? t('领取中…') : taken ? t('已领取')
                          : state === 'unknown' || rejected ? t('尝试领取') : t('领取')}
                      </Button>
                    </li>
                  )
                })}
              </ul>
            )}
            {preview.status === 'ready' && queryErrors.length > 0 ? (
              <p className='mt-1.5 text-[11.5px] text-muted-foreground'>
                {t('部分来源查询失败，已保留其他结果：{message}', {
                  message: queryErrors.join('；'),
                })}
              </p>
            ) : null}
            {receiptError ? <p className='mt-1.5 text-[11.5px] text-muted-foreground'>
              {t('套餐已领取，但营销回执未完成，不会重复领取：{message}', { message: receiptError })}
            </p> : null}
            <p className='mt-1.5 text-[11.5px] text-muted-foreground'>
              {t('活动目录合并账号预览、营销活动与匿名探测。已领取仅依据领取记录或账号名下套餐；状态待确认时可尝试领取，资格以上游返回为准。')}
            </p>
            {preview.status === 'ready' && preview.result.activationError ? (
              <p className='text-[11.5px] text-muted-foreground'>
                {t('（激活事件补报失败，可能影响领取资格：{message}）', { message: preview.result.activationError })}
              </p>
            ) : null}
          </section>

          <section>
            <div className='mb-1.5 flex items-center gap-2 text-[12px] font-semibold text-muted-foreground'>
              {t('我的套餐')}
              {/*
                available in ** to ** when 。 no has when `availableView` is 
                「none」， in down that will 「 this no 」——
                 is 「 this no has 」（ can can in that up ，
                 after balance.rs `available` that ）。 that to down 
                。

                「available」 this before in billing up ： `availableView`
                 is 1 ****（「1 token」）。（`walletsFrom`） to is 
                ****（「 5 when 99%」）， before 「available」 will 
                「available 5 when 99%」 this —— can 。
              */}
              {Array.isArray(data?.wallets) && data.wallets.length > 0 && data.availableView ? (
                <Badge variant='outline' shape='tag'>
                  {String(data.walletsFrom || '') === 'monitor'
                    ? String(data.availableView)
                    : t('可用 {value}', { value: String(data.availableView) })}
                </Badge>
              ) : null}
              <div className='ml-auto'>
                <Button variant='outline' size='xs' disabled={inflight}
                  title={t('重新读取该账号的余额与套餐（走余额查询那条链）')}
                  onClick={() => void refreshUsageAfterCheckin(id)}>
                  {inflight ? t('查询中…') : t('查询余额')}
                </Button>
              </div>
            </div>
            {ownedPlans.length > 0 ? (
              <ul className='m-0 list-none p-0'>
                {ownedPlans.map(plan => {
                  const planId = String(plan.planId || '')
                  const state = String(plan.state || 'unknown')
                  const entitlements = entitlementLine(plan)
                  // when `planTimeText`（ + to ），** not use ** `windowText`：
                  // after before is `startsAt` = when ， not 
                  const window_ = planTimeText(plan)
                  const claimedAt = planId && account?.claimPlans
                    ? Number((account.claimPlans as Record<string, unknown>)[planId]) || 0
                    : 0
                  return (
                    <li key={plan.userPlanId || planId || plan.name}
                      className='flex items-start justify-between gap-3 border-b border-hairline py-2 last:border-b-0'>
                      <div className='min-w-0'>
                        <div className='flex items-center gap-2 text-[13px] font-medium'>
                          <span className='truncate'>{plan.name || planId || t('套餐')}</span>
                          <Badge variant={STATE_VARIANT[state] || 'outline'} shape='tag'>
                            {zcodePlanStateLabel(state)}
                          </Badge>
                        </div>
                        {window_ ? <div className='text-[12px] text-muted-foreground'>{window_}</div> : null}
                        {entitlements ? <div className='text-[12px] text-muted-foreground'>{entitlements}</div> : null}
                      </div>
                      {claimedAt > 0 ? (
                        <span className='flex-none text-[11.5px] text-muted-foreground'>
                          {t('本地领取于 {time}', { time: stampText(claimedAt / 1000) })}
                        </span>
                      ) : null}
                    </li>
                  )
                })}
              </ul>
            ) : (
              <p className='py-1 text-[12.5px] text-muted-foreground'>
                {entry === undefined
                  ? t('还没有余额读数（已自动查询一次，稍候；也可以点右上角「查询余额」重读）')
                  : data && data.source && data.source !== SOURCE_BILLING
                    ? t('这个账号的余额走窗口限额通道（只配了编码套餐 API Key）：只有每 N 小时的剩余比例，读不到套餐清单')
                    : t('读数里没有套餐清单（免费账号、或套餐的上游状态还没刷新；可点右上角「查询余额」重读）')}
              </p>
            )}
            {unconfirmed.length > 0 ? (
              <p className='mt-1.5 text-[11.5px] text-muted-foreground'>
                {t('本地台账里今天还领过这些，但上面的读数里还没有：{plans}（读数刷新可能有延迟）', {
                  plans: unconfirmed.map(planId => {
                    const at = Number(ledger[planId]) || 0
                    const time = at > 0 ? stampText(at / 1000) : ''
                    return time ? t('{plan}（{time} 领取）', { plan: planId, time }) : planId
                  }).join(t('、')),
                })}
              </p>
            ) : null}
          </section>
        </DialogBody>
        <DialogFooter>
          <span className='min-w-0 text-[11.5px] text-muted-foreground'>
            {claimed.length > 0
              ? t('今天（北京时间）已领取 {n} 份：{plans}', { n: claimed.length, plans: claimed.join(t('、')) })
              : t('本期活动按套餐分别展示，领取资格以上游返回为准')}
          </span>
          <div className='mr-auto' />
          {claiming ? <Button variant='outline' disabled={claimPhase === 'submitting'}
            onClick={() => claimController.current?.abort()}>{t('取消验证')}</Button> : null}
          <Button variant='outline' disabled={claimPhase === 'submitting'} onClick={close}>{t('关闭')}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

/* ─── ─────────────────────────────── */

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
     * ZCode 。`id` is id（）；`name` in not to when 
     * —— 「」 and by from 
     * （ has `claimedPlanIdsToday` 1 ）， use not 。
     */
    wbZcodePlans?: { open(options: OpenOptions): void }
  }
}

window.wbZcodePlans = { open: openModal }
