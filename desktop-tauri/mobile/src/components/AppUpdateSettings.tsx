import { useEffect, useState } from 'react'
import { Button, Input, Switch } from '@ui'
import { getUpdateChannels, getUpdateCheckPlan, saveUpdateChannels, saveUpdateCheckPlan, type UpdateChannels, type UpdateCheckPlan } from '../api/updates'
import { serverIdentity } from '../lib/servers'
import { store } from '../state/store'
import { t } from '../i18n'
import { Alert } from '@ui'

export function AppUpdateSettings({ onChanged }: { onChanged: (channelsChanged: boolean) => void }) {
  const [channels, setChannels] = useState<UpdateChannels | null>(null)
  const [plan, setPlan] = useState<UpdateCheckPlan | null>(null)
  const [draft, setDraft] = useState('')
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState('')
  const identity = serverIdentity()

  useEffect(() => {
    const controller = new AbortController()
    void getUpdateChannels(controller.signal).then(value => {
      if (!controller.signal.aborted) setChannels(value)
    }).catch(cause => { if (!controller.signal.aborted) setError(String(cause)) })
    void getUpdateCheckPlan(controller.signal).then(value => {
      if (!controller.signal.aborted) { setPlan(value); setDraft(String(value.interval)) }
    }).catch(cause => { if (!controller.signal.aborted) setError(String(cause)) })
    return () => controller.abort()
  }, [])

  async function saveChannel(patch: Partial<Pick<UpdateChannels, 'stable' | 'beta'>>) {
    if (busy || !channels) return
    const next = { ...channels, ...patch }
    if (!next.stable && !next.beta) {
      setError(t('Follow at least one channel: stable or beta.'))
      return
    }
    setBusy(true)
    setError('')
    try {
      const result = await saveUpdateChannels(patch)
      if (serverIdentity() !== identity) return
      setChannels(result)
      store.toast(result.saved === false ? t('Channel switched, but writing to disk failed (the old setting will be restored after a restart)') : t('Update channels saved'), result.saved === false ? 'err' : 'ok')
      onChanged(true)
    } catch (cause) {
      if (serverIdentity() === identity) setError(cause instanceof Error ? cause.message : String(cause))
    } finally { setBusy(false) }
  }

  async function savePlan(patch: Partial<Pick<UpdateCheckPlan, 'enabled' | 'interval'>>) {
    if (busy) return
    setBusy(true)
    setError('')
    try {
      const result = await saveUpdateCheckPlan(patch)
      if (serverIdentity() !== identity) return
      setPlan(result)
      setDraft(String(result.interval))
      store.toast(result.saved === false ? t('Settings applied, but could not be saved to disk (will revert after restart)') : t('✅ Automatic update checks saved'), result.saved === false ? 'err' : 'ok')
      onChanged(false)
    } catch (cause) {
      if (serverIdentity() === identity) setError(cause instanceof Error ? cause.message : String(cause))
    } finally { setBusy(false) }
  }

  function saveInterval() {
    const interval = Number(draft)
    if (!Number.isInteger(interval) || interval < 1 || interval > 1440) {
      setError(t('The check interval must be {min}–{max} minutes', { min: 1, max: 1440 }))
      return
    }
    void savePlan({ interval })
  }

  return <div className='flex flex-col gap-2 px-3 py-3' style={{ borderTop: '1px solid var(--m-line)' }}>
    <div className='m-t1'>{t('Update settings')}</div>
    <p className='m-t2'>{t('These settings belong to the connected gateway, not this phone.')}</p>
    <div className='flex items-center justify-between gap-2'>
      <span className='m-t1'>{t('Automatic update checks')}</span>
      <Switch checked={plan?.enabled === true} disabled={busy || !plan} aria-label={t('Automatic update checks')}
        onCheckedChange={value => void savePlan({ enabled: value === true })} />
    </div>
    <div className='flex items-center gap-2'>
      <Input type='number' min={1} max={1440} step={1} className='w-[110px] text-base' value={draft} disabled={busy || !plan?.enabled}
        onChange={event => setDraft(event.currentTarget.value)} aria-label={t('Check interval (minutes)')} />
      <span className='m-t2'>{t('minutes ({min}–{max})', { min: 1, max: 1440 })}</span>
      <Button size='sm' variant='outline' disabled={busy || !plan?.enabled || Number(draft) === plan.interval} onClick={saveInterval}>{t('Save')}</Button>
    </div>
    <div className='flex items-center justify-between gap-2'>
      <span className='m-t1'>{t('Follow stable updates')}</span>
      <Switch checked={channels?.stable === true} disabled={busy || !channels} aria-label={t('Follow stable updates')}
        onCheckedChange={value => void saveChannel({ stable: value === true })} />
    </div>
    <p className='m-t2'>{t('Enabled by default. Even on a beta build, you can receive stable releases with a higher version.')}</p>
    <div className='flex items-center justify-between gap-2'>
      <span className='m-t1'>{t('Follow beta updates')}</span>
      <Switch checked={channels?.beta === true} disabled={busy || !channels} aria-label={t('Follow beta updates')}
        onCheckedChange={value => void saveChannel({ beta: value === true })} />
    </div>
    <p className='m-t2'>{t('Beta builds may be unstable. With both channels enabled, the highest version is selected without downgrading. Your channel choices remain unchanged after installing either a stable or beta release.')}</p>
    <p className='m-t2'>{t('Plugins follow these update channels, automatic checks and interval by default. You can configure them separately in Plugin settings.')}</p>
    {error ? <Alert variant='destructive' className='m-t2' role='alert' style={{ color: 'var(--m-danger)' }}>{error}</Alert> : null}
  </div>
}
