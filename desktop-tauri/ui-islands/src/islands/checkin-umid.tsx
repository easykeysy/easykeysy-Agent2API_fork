import * as React from 'react'
import { Button, Spinner } from '@ui'
import { t } from '../i18n'
import { bridgeApi, errorMessage, loadCenter, toast, type UmidState } from './checkin-store'

export function CheckinUmid(): React.ReactElement | null {
  const [state, setState] = React.useState<UmidState | null>(null)
  const [busy, setBusy] = React.useState(false)
  const [queryError, setQueryError] = React.useState('')
  const [installError, setInstallError] = React.useState('')
  const active = React.useRef(false)
  const installing = React.useRef(false)
  const request = React.useRef(0)

  const load = React.useCallback(async (): Promise<UmidState | null> => {
    const id = ++request.current
    try {
      const getStatus = bridgeApi().getQoderUmid
      if (!getStatus) throw new Error(t('The UMID component management API is unavailable. Refresh the page or update the gateway.'))
      const next = await getStatus()
      if (!next || typeof next.available !== 'boolean') {
        throw new Error(t('The backend did not return the UMID component status'))
      }
      if (active.current && id === request.current) {
        setState(next)
        setQueryError('')
      }
      return next
    } catch (error) {
      if (active.current && id === request.current) setQueryError(errorMessage(error))
      return null
    }
  }, [])

  React.useEffect(() => {
    active.current = true
    void load()
    return () => {
      active.current = false
      request.current += 1
    }
  }, [load])

  React.useEffect(() => {
    const timer = window.setInterval(() => { void load() }, busy || state?.installing ? 3000 : 60_000)
    return () => window.clearInterval(timer)
  }, [busy, state?.installing, load])

  const install = async (): Promise<void> => {
    if (installing.current || state?.installing) return
    installing.current = true
    setBusy(true)
    setInstallError('')
    try {
      const installComponent = bridgeApi().installQoderUmid
      if (!installComponent) throw new Error(t('The UMID component management API is unavailable. Refresh the page or update the gateway.'))
      const result = await installComponent()
      if (!result) throw new Error(t('The backend did not return a UMID component installation result'))
      const next = await load()
      if (!active.current) return
      if (next?.available) {
        toast(t('The UMID component is installed. You can check in again.'))
        await loadCenter()
      } else if (next && !next.installing) {
        throw new Error(t('The UMID component is still not detected after installation. Check again.'))
      }
    } catch (error) {
      if (active.current) {
        const message = t('UMID component installation failed: {error}', { error: errorMessage(error) })
        setInstallError(message)
        toast(message, 'err')
      }
    } finally {
      installing.current = false
      if (active.current) {
        setBusy(false)
        void load()
      }
    }
  }

  if (state?.available) return null
  const bridge = bridgeApi()
  const unavailable = !bridge.getQoderUmid || (state?.installSupported && !bridge.installQoderUmid)
  const error = installError || queryError
  const pending = busy || state?.installing === true

  return (
    <div className='flex flex-wrap items-center gap-2' aria-live='polite'>
      <span>{state
        ? t('International check-in requires the UMID component to generate a device risk-control identity (not detected on this machine).')
        : error ? t('Gateway device component') : t('Loading the UMID component status…')}</span>
      {state?.installSupported ? (
        <Button variant='outline' size='sm' disabled={pending || !bridge.installQoderUmid}
          onClick={() => void install()}>
          {pending ? <><Spinner className='size-3.5' />{t('Installing…')}</> : t('Install component')}
        </Button>
      ) : state ? <span>{t('Install the Qoder client or qodercli to enable it (no need to restart the gateway).')}</span> : null}
      <Button variant='ghost' size='sm' onClick={() => { setInstallError(''); void load() }}>
        {t('Check again')}
      </Button>
      {installError ? <span className='text-destructive'>{installError}</span>
        : queryError ? <span className='text-destructive'>{t('Query failed: {error}', { error: queryError })}</span> : null}
      {unavailable ? <span>
        {t('You can call this API with administrator credentials:')} <code>POST /api/qoder-umid/install</code>
      </span> : null}
    </div>
  )
}
