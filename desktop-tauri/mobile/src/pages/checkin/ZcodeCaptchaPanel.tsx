import { useCallback, useEffect, useRef, useState } from 'react'
import { Button } from '@ui'
import { captchaPoolStats, verifyPlanChannel, type CaptchaPhase, type CaptchaPoolStats } from '../../api/zcode'
import { serverIdentity } from '../../lib/servers'
import { t } from '../../i18n'

export function ZcodeCaptchaPanel({ accountId, disabled, onBusy }: {
  accountId: string; disabled: boolean; onBusy?: (busy: boolean) => void
}) {
  const [stats, setStats] = useState<CaptchaPoolStats | null>(null)
  const [phase, setPhase] = useState<CaptchaPhase | ''>('')
  const [error, setError] = useState('')
  const controller = useRef<AbortController | null>(null)
  const mounted = useRef(true)
  const identity = serverIdentity()
  const refresh = useCallback(async (clearError = false) => {
    try {
      const value = await captchaPoolStats()
      if (mounted.current && serverIdentity() === identity) { setStats(value); if (clearError) setError('') }
    } catch (cause) {
      if (mounted.current && serverIdentity() === identity) setError(cause instanceof Error ? cause.message : String(cause))
    }
  }, [identity])
  useEffect(() => {
    mounted.current = true
    void refresh()
    return () => { mounted.current = false; controller.current?.abort(); onBusy?.(false) }
  }, [accountId, refresh, onBusy])

  async function verify() {
    if (controller.current || disabled) return
    const current = new AbortController()
    controller.current = current
    setError('')
    setPhase('preparing')
    onBusy?.(true)
    try {
      await verifyPlanChannel(accountId, current.signal, value => { if (mounted.current) setPhase(value) })
    } catch (cause) {
      if (mounted.current && !current.signal.aborted && (cause as Error)?.name !== 'CaptchaCancelledError') {
        setError(cause instanceof Error ? cause.message : String(cause))
      }
    } finally {
      if (controller.current === current) controller.current = null
      if (mounted.current && serverIdentity() === identity) { setPhase(''); onBusy?.(false); void refresh() }
    }
  }

  const failure = error || stats?.error || stats?.configError
  const text = stats?.required === false ? t('Model requests currently require no human verification.')
    : Number(stats?.ready) > 0 ? t('Captcha tokens: {counts}', { counts: `${stats?.ready} / ${stats?.target || 3}` })
      : stats?.needsTokens ? t('A Start Plan request needs verification. Start verification manually, then retry the request.')
        : t('No recent Start Plan requests; no verification tokens are generated.')
  return (
    <section className='m-ck-account'>
      <div className='m-ck-title'>{t('Start Plan human verification')}</div>
      <p className={failure ? 'm-ck-error' : 'm-ck-note'}>{failure || (stats ? text : t('Loading…'))}</p>
      <p className='m-ck-note'>{t('The background only reads status and never opens verification automatically. Each token is single-use and expires shortly.')}</p>
      <div className='m-ck-actions'>
        {stats?.required !== false && Number(stats?.startPlanAccounts) > 0 ? (
          <Button size='sm' variant='outline' disabled={disabled || Boolean(phase) || Boolean(stats?.error || stats?.configError)} onClick={() => void verify()}>
            {phase ? t('Verifying…') : t('Start verification')}
          </Button>
        ) : null}
        {phase ? <Button size='sm' variant='outline' disabled={phase === 'submitting'} onClick={() => controller.current?.abort()}>{t('Cancel verification')}</Button> : null}
        <Button size='sm' variant='ghost' disabled={disabled || Boolean(phase)} onClick={() => void refresh(true)}>{t('Refresh')}</Button>
      </div>
    </section>
  )
}
