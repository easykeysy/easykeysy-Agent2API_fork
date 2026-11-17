import * as React from 'react'
import {
  Badge,
  Button,
  Card,
  CardContent,
  CardHeader,
  Dialog,
  DialogBody,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Input,
  Label,
  Switch,
} from '@ui'
import { t } from '../i18n'
import { TIPS } from './settings-model'
import {
  cancelLanRegister,
  resolveLanConfirm,
  submitLanRegister,
  toggleLan,
  toggleLanPanel,
  type LanConfirm,
  type LanRegister,
  type SettingsSnapshot,
} from './settings-state'

export function NetworkAccessPanel({ snap }: { snap: SettingsSnapshot }) {
  if (snap.panelLogin) return null
  const { app } = snap
  const ready = app.status === 'ready'
  const disabled = !ready || snap.busy === 'lan'
  const base = app.lanIp && app.port ? `http://${app.lanIp}:${app.port}` : null

  return (
    <Card render={<section />} className='panel'>
      <CardHeader className='panel-head'>
        <h2>{t('Network access')}</h2>
        <span className='tip-q' data-tip={TIPS.networkAccess} />
        <Badge variant={!ready ? (app.status === 'unavailable' ? 'destructive' : 'outline') : app.lanAccess ? 'success' : 'outline'}>
          {!ready ? (app.status === 'unavailable' ? t('Unavailable') : t('Checking…')) : app.lanAccess ? t('Enabled') : t('This device only')}
        </Badge>
      </CardHeader>
      <CardContent className='panel-body'>
        <div className='settings-switches'>
          <Label className='switch'>
            <Switch id='settings-lan-access' checked={app.lanAccess} disabled={disabled} onCheckedChange={toggleLan} />
            <span>{t('Allow other devices to access the gateway')}</span>
          </Label>
          <Label className='switch'>
            <Switch id='settings-lan-panel' checked={app.lanPanel} disabled={disabled || !app.lanAccess} onCheckedChange={toggleLanPanel} />
            <span>{t('Also expose the web admin panel')}</span>
          </Label>
        </div>
        <div className='settings-state'>
          {!ready
            ? app.status === 'unavailable' ? t('Could not load network access settings. Please try again later.') : t('Checking…')
            : app.lanAccess
              ? t('Enabled: listening on 0.0.0.0 to allow access from other devices.')
              : t('Off: the gateway listens only on 127.0.0.1 and is reachable only from this machine.')}
        </div>
        {ready && app.lanAccess ? (
          <div className='mt-3 space-y-2 text-sm'>
            {base ? (
              <>
                <div className='flex flex-wrap items-baseline gap-x-2 gap-y-1'>
                  <span className='text-muted-foreground'>{t('This device\'s API address')}</span>
                  <code className='break-all'>{base}/v1</code>
                </div>
                {app.lanPanel ? (
                  <div className='flex flex-wrap items-baseline gap-x-2 gap-y-1'>
                    <span className='text-muted-foreground'>{t('Web management address')}</span>
                    <code className='break-all'>{base}</code>
                  </div>
                ) : null}
                <div className='hint'>{t('Use an address reachable from the connecting device. The local address shown here does not imply public internet reachability.')}</div>
              </>
            ) : (
              <div className='hint'>{t('The local access address is not available yet. Use this machine\'s IP and gateway port that the other device can reach.')}</div>
            )}
          </div>
        ) : null}
        <div className='hint retention-note'>
          {t('Enabling this listens on all IPv4 interfaces (0.0.0.0), not just the LAN, but does not automatically expose the gateway to the internet. Actual access depends on your network and firewall settings. Changes require an app restart.')}
        </div>
      </CardContent>
    </Card>
  )
}

const NETWORK_CONFIRM_COPY: Record<
  NonNullable<LanConfirm>['mode'],
  { title: string; body: (panel: boolean) => React.ReactNode; label: string }
> = {
  enable: {
    title: t('Enable network access'),
    body: () => (
      <>
        {t('The gateway will listen on all IPv4 interfaces (0.0.0.0). Access is no longer limited to this machine or to the LAN. Public internet access depends on your network and firewall settings.')}<br />
        {t('A panel administrator must be registered before enabling this; existing administrators skip this step. Management endpoints require an admin session or gateway key. API calls require an enabled gateway key; a default key is created automatically if none is available.')}<br />
        <strong>{t('The app will restart after saving for this to take effect.')}</strong>{t(' Continue?')}
      </>
    ),
    label: t('Continue'),
  },
  disable: {
    title: t('Disable network access'),
    body: () => (
      <>
        {t('The gateway will listen only on 127.0.0.1, preventing direct connections from other devices. The web management panel will also be disabled. Existing gateway keys and accounts are not affected.')}<br />
        <strong>{t('The app will restart after saving for this to take effect.')}</strong>{t(' Continue?')}
      </>
    ),
    label: t('Close & Restart'),
  },
  panel: {
    title: t('Change Web Admin Panel'),
    body: panel => (
      <>
        {panel
          ? t('Devices that can reach the gateway will be able to open the web management panel in a browser and sign in with an administrator account. Its access scope is the same as the gateway\'s and is not limited to the LAN.')
          : t('The web management panel will no longer be served. You can still manage the gateway in the local desktop app, and API forwarding is not affected.')}<br />
        <strong>{t('The app will restart after saving for this to take effect.')}</strong>{t(' Continue?')}
      </>
    ),
    label: t('Save & Restart'),
  },
}

export function NetworkAccessDialog({ confirm, register }: { confirm: LanConfirm; register: LanRegister }) {
  const cancelRef = React.useRef<HTMLButtonElement | null>(null)
  const nameRef = React.useRef<HTMLInputElement | null>(null)
  const passwordRef = React.useRef<HTMLInputElement | null>(null)
  const open = confirm !== null || register !== null
  const close = () => {
    if (register !== null) cancelLanRegister()
    else resolveLanConfirm(false)
  }
  const submitRegister = () => {
    submitLanRegister(nameRef.current?.value ?? '', passwordRef.current?.value ?? '')
  }

  return (
    <Dialog open={open} onOpenChange={next => { if (!next) close() }}>
      <DialogContent className='w-[min(480px,calc(100vw-48px))]' initialFocus={cancelRef}>
        {register !== null ? (
          <>
            <DialogHeader>
              <DialogTitle>{t('Register Admin Account')}</DialogTitle>
            </DialogHeader>
            <DialogBody>
              <div>{t('Set a panel administrator username and password before allowing access from other devices. After registration, network access will be enabled and the app will restart.')}</div>
              <form onSubmit={event => { event.preventDefault(); if (!register.busy) submitRegister() }}>
                <Label htmlFor='lan-admin-name' className='mt-3.5 mb-[5px] block text-[12.5px]'>{t('Accounts')}</Label>
                <Input ref={nameRef} id='lan-admin-name' autoComplete='username' placeholder={t('Admin account')} disabled={register.busy} />
                <Label htmlFor='lan-admin-password' className='mt-3.5 mb-[5px] block text-[12.5px]'>{t('Password')}</Label>
                <Input
                  ref={passwordRef}
                  id='lan-admin-password'
                  type='password'
                  autoComplete='new-password'
                  placeholder={t('At least 8 characters')}
                  disabled={register.busy}
                />
              </form>
              <div className='mt-3 min-h-5 text-[13px] whitespace-pre-wrap text-destructive'>{register.error}</div>
            </DialogBody>
            <DialogFooter>
              <div className='mr-auto' />
              <Button ref={cancelRef} variant='outline' onClick={cancelLanRegister} disabled={register.busy}>
                {t('Cancel')}
              </Button>
              <Button onClick={submitRegister} disabled={register.busy}>
                {register.busy ? t('Registering…') : t('Register & Enable')}
              </Button>
            </DialogFooter>
          </>
        ) : (
          <>
            <DialogHeader>
              <DialogTitle>{confirm ? NETWORK_CONFIRM_COPY[confirm.mode].title : ''}</DialogTitle>
            </DialogHeader>
            <DialogBody>
              <div>{confirm ? NETWORK_CONFIRM_COPY[confirm.mode].body(confirm.panel) : null}</div>
            </DialogBody>
            <DialogFooter>
              <div className='mr-auto' />
              <Button ref={cancelRef} variant='outline' onClick={() => resolveLanConfirm(false)}>{t('Cancel')}</Button>
              <Button onClick={() => resolveLanConfirm(true)}>
                {confirm ? NETWORK_CONFIRM_COPY[confirm.mode].label : t('Continue')}
              </Button>
            </DialogFooter>
          </>
        )}
      </DialogContent>
    </Dialog>
  )
}
