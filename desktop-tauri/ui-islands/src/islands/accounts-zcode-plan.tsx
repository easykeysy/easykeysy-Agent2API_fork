import * as React from 'react'
import { Button, Field, FieldLabel, Select, SelectContent, SelectItem, SelectTrigger, SelectValue, StatusDot, cn } from '@ui'
import { t } from '../i18n'
import { errorMessage, shared, type AccountsBridge } from './accounts-shared'
import { zcodePlanLabel, ZCODE_PLAN_CODING, ZCODE_PLAN_START } from './accounts-domain'

type PoolStats = NonNullable<Awaited<ReturnType<NonNullable<AccountsBridge['zcodeCaptchaStats']>>>>
type MintStatus = { state: string; error: string }

export function poolStatus(pool: PoolStats | null, mint: MintStatus | null): string {
  if (!pool) return t('Loading…')
  const error = pool.error || pool.configError
  if (error) return t('Failed to load: {error}', { error })
  if (pool.required === false) return t('Model requests currently require no human verification.')
  if (mint?.state === 'verifying') return t('Verifying…')
  if (mint?.state === 'error') return t('Verification failed: {error}', { error: mint.error })
  if (mint?.state === 'paused') return t('Verification paused. Retry manually when needed.')
  const counts = `${Number(pool.ready) || 0} / ${Number(pool.target) || 3}`
  if (!Number(pool.startPlanAccounts)) return t('No enabled Start Plan accounts; no verification tokens are generated.')
  if (Number(pool.ready) > 0) return t('Captcha tokens: {counts}', { counts })
  return pool.needsTokens === true
    ? t('A Start Plan request needs verification. Start verification manually, then retry the request.')
    : t('No recent Start Plan requests; no verification tokens are generated.')
}

export function ZcodeCaptchaStatus({ accountId, disabled = false, onBusy }: {
  accountId: string; disabled?: boolean; onBusy?: (busy: boolean) => void
}) {
  const [pool, setPool] = React.useState<PoolStats | null>(null)
  const [mint, setMint] = React.useState<MintStatus | null>(null)
  const [phase, setPhase] = React.useState('')
  const controller = React.useRef<AbortController | null>(null)
  const mounted = React.useRef(true)
  const refresh = React.useCallback(async () => {
    const api = shared().workbuddyDesktop
    try {
      const value = api?.zcodeCaptchaStats ? await api.zcodeCaptchaStats() : { error: t('CAPTCHA token pool interface unavailable') }
      if (mounted.current) {
        setPool(value || null)
        if (shared().wbZcodeCaptchaPool?.status) setMint(shared().wbZcodeCaptchaPool?.status?.() || null)
      }
    } catch (error) {
      if (mounted.current) setPool({ error: errorMessage(error) })
    }
  }, [])
  React.useEffect(() => {
    mounted.current = true
    void refresh()
    const timer = window.setInterval(() => void refresh(), 15000)
    const onStatus = () => {
      setMint(shared().wbZcodeCaptchaPool?.status?.() || null)
      void refresh()
    }
    window.addEventListener('zcode-captcha-status', onStatus)
    return () => {
      mounted.current = false
      controller.current?.abort()
      onBusy?.(false)
      window.clearInterval(timer)
      window.removeEventListener('zcode-captcha-status', onStatus)
    }
  }, [refresh, onBusy])

  async function verify() {
    const api = shared().wbZcodeCaptchaPool
    if (controller.current || !api?.verify) return
    const current = new AbortController()
    controller.current = current
    setPhase('preparing')
    onBusy?.(true)
    try {
      await api.verify(accountId, current.signal, value => { if (mounted.current) setPhase(value) })
    } catch (error) {
      if (mounted.current && !current.signal.aborted && (error as Error)?.name !== 'CaptchaCancelledError') {
        setMint({ state: 'error', error: errorMessage(error) })
      }
    } finally {
      if (controller.current === current) controller.current = null
      if (mounted.current) { setPhase(''); onBusy?.(false); void refresh() }
    }
  }

  const busy = Boolean(phase)
  const failed = Boolean(pool?.error || pool?.configError || mint?.state === 'error')
  /**
   * 这一块压成**一行**：一枚状态点 + 一句话 + 必要时的按钮。
   *
   * 「后台只读取状态，不会自动弹出验证码；验证结果一次性使用，短时间内过期」
   * 那条规则挂在 title 上，需要时悬停可查 —— 它是通道的说明，不是当前状态，
   * 不该跟状态抢同一行。
   */
  return (
    <div className='flex flex-wrap items-center gap-2'
      title={t('The background only reads status and never opens verification automatically. Each token is single-use and expires shortly.')}>
      <StatusDot variant={failed ? 'destructive' : pool?.required === false ? 'success' : 'warning'} />
      <span className={cn('text-[12.5px]', failed && 'text-destructive')}>{poolStatus(pool, mint)}</span>
      <div className='ml-auto flex flex-wrap items-center gap-2'>
        {pool?.required !== false && Number(pool?.startPlanAccounts) > 0 ? (
          <Button variant='outline' size='xs' disabled={disabled || busy || Boolean(pool?.error || pool?.configError) || !shared().wbZcodeCaptchaPool?.verify}
            onClick={() => void verify()}>{busy ? t('Verifying…') : t('Start verification')}</Button>
        ) : null}
        {busy ? <Button variant='outline' size='xs' disabled={phase === 'submitting'}
          onClick={() => controller.current?.abort()}>{t('Cancel verification')}</Button> : null}
        <Button variant='ghost' size='xs' disabled={busy || disabled} onClick={() => void refresh()}>{t('Refresh')}</Button>
      </div>
    </div>
  )
}

export function PlanChannelField({ plan, hasJwt, onChange }: {
  plan: string; hasJwt: boolean; onChange: (next: string) => void
}) {
  return (
    <Field orientation='horizontal' className='field-row mt-2.5'>
      <FieldLabel htmlFor='account-plan-channel'>{t('Plan in use')}</FieldLabel>
      <Select value={plan} onValueChange={value => onChange(String(value))}>
        <SelectTrigger id='account-plan-channel' className='min-w-[220px]'><SelectValue>{zcodePlanLabel(plan)}</SelectValue></SelectTrigger>
        <SelectContent>
          <SelectItem value={ZCODE_PLAN_CODING}>{t('Coding Plan — uses the open-platform endpoints and consumes open-platform quota (subscription plan / resource packs / prepaid balance)')}</SelectItem>
          <SelectItem value={ZCODE_PLAN_START}>{t('Plan pool (Start Plan) — uses the zcode.z.ai endpoints and the ZCode plan-pool quota (new-user trial quota / event grants)')}</SelectItem>
        </SelectContent>
      </Select>
      {plan === ZCODE_PLAN_START && !hasJwt ? <span className='detail text-destructive'>{t('This account has no plan login state (JWT); the plan-pool channel will fail — sign in to this account again')}</span> : null}
      {plan === ZCODE_PLAN_START ? <p className='detail'>{t('When human verification is required, start it manually in this account’s Plan details.')}</p> : null}
    </Field>
  )
}
