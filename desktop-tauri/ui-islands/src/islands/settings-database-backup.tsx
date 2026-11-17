import * as React from 'react'
import {
  Alert, Badge, Button, Card, CardContent, CardHeader,
  Dialog, DialogBody, DialogContent, DialogFooter, DialogHeader, DialogTitle,
  Input, Label, Switch,
} from '@ui'
import { NOTES, TIPS } from './settings-model'
import {
  cancelDbExport, confirmDbExport, confirmDbRestore, exportDatabase,
  resolveDbBackupConfirm, toggleDbIncludeLogs, useSettings,
} from './settings-state'
import { t } from '../i18n'

export function DatabaseBackupPanel(): React.ReactElement {
  const snap = useSettings()
  const busy = snap.busy === 'dbExport' || snap.busy === 'dbImport' || snap.dbRestore?.active === true
  return (
    <Card render={<section />} className='panel'>
      <CardHeader className='panel-head'>
        <h2>{t('Full-database backup / restore')}</h2>
        <span className='tip-q' data-tip={TIPS.backup} />
        <Badge variant='outline'>{t('Password encrypted · Recovery keys included')}</Badge>
        <div className='head-actions'>
          <Button id='btn-settings-db-export' variant='outline' disabled={busy} onClick={confirmDbExport}>
            {snap.busy === 'dbExport' ? t('Backing up…') : t('Back up database')}
          </Button>
          <Button id='btn-settings-db-restore' variant='destructive' disabled={busy} onClick={confirmDbRestore}>
            {snap.busy === 'dbImport' ? t('Restoring…') : t('Restore database')}
          </Button>
        </div>
      </CardHeader>
      <CardContent className='panel-body'>
        {snap.dbWarning ? (
          <Alert variant='destructive' className='danger-zone' id='settings-db-warning'>
            <strong>{t('⚠️ Key warning')}</strong><br />{snap.dbWarning}
          </Alert>
        ) : null}
        <div className='retention-row'>
          <Label htmlFor='settings-db-include-logs'>{t('Include logs and debug payloads')}</Label>
          <Switch id='settings-db-include-logs' checked={snap.dbIncludeLogs} disabled={snap.busy !== null || busy} onCheckedChange={toggleDbIncludeLogs} />
          <div className='hint'>
            {t('Unchecked, only business data is backed up (accounts, configuration, balance records) and the file shrinks considerably; event logs, request stats, and debug payloads are left out.')}
          </div>
        </div>
        <p className='hint'>
          {t('One .a2ab file restores data on another machine without separately copying keys for accounts, gateway keys, GitHub tokens, or cloud sync credentials. Plugin programs and files outside the database are not included.')}
        </p>
        <Alert variant='destructive' className='danger-zone'>{NOTES.backupDanger}</Alert>
      </CardContent>
    </Card>
  )
}

export function DatabaseBackupDialogs(): React.ReactElement {
  const snap = useSettings()
  return (
    <>
      {snap.dbExportConfirm ? <BackupPasswordDialog mode='export' /> : null}
      {snap.dbBackupConfirm ? <BackupPasswordDialog mode='restore' /> : null}
    </>
  )
}

function BackupPasswordDialog({ mode }: { mode: 'export' | 'restore' }): React.ReactElement {
  const passwordRef = React.useRef<HTMLInputElement>(null)
  const repeatRef = React.useRef<HTMLInputElement>(null)
  const cancelRef = React.useRef<HTMLButtonElement>(null)
  const [error, setError] = React.useState('')
  const exporting = mode === 'export'
  const clear = () => {
    if (passwordRef.current) passwordRef.current.value = ''
    if (repeatRef.current) repeatRef.current.value = ''
  }
  const close = () => {
    clear()
    if (exporting) cancelDbExport()
    else resolveDbBackupConfirm(false)
  }
  const submit = () => {
    const password = passwordRef.current?.value ?? ''
    if (new TextEncoder().encode(password).length > 1024) {
      setError(t('The backup password must not exceed 1024 bytes'))
      return
    }
    if (exporting && Array.from(password).length < 12) {
      setError(t('Use at least 12 characters; a long, unique password is recommended'))
      return
    }
    if (exporting && password !== repeatRef.current?.value) {
      setError(t('The backup passwords do not match'))
      return
    }
    clear()
    if (exporting) void exportDatabase(password)
    else resolveDbBackupConfirm(true, password)
  }
  return (
    <Dialog open onOpenChange={next => { if (!next) close() }}>
      <DialogContent className='w-[min(480px,calc(100vw-48px))]' initialFocus={cancelRef}>
        <DialogHeader>
          <DialogTitle>{exporting ? t('Set backup password') : t('Restore full backup')}</DialogTitle>
        </DialogHeader>
        <DialogBody>
          {exporting ? (
            <Alert>{t('The database and recovery keys are encrypted together. The password is not stored in the application or backup. Keep it safe: without it, restoration is impossible.')}</Alert>
          ) : (
            <Alert variant='destructive'>
              {t('All local database data will be replaced. A safety backup is saved first, and you must sign in again afterward. Existing local keys will not be overwritten.')}
            </Alert>
          )}
          <form id={`db-${mode}-password-form`} onSubmit={event => { event.preventDefault(); submit() }}>
            <Label htmlFor={`db-${mode}-password`} className='mt-3 mb-1 block'>{t('Backup password')}</Label>
            <Input id={`db-${mode}-password`} ref={passwordRef} type='password' autoComplete={exporting ? 'new-password' : 'off'} maxLength={1024} placeholder={exporting ? t('At least 12 characters') : t('Required for encrypted backups; leave blank for legacy SQL')} onChange={() => setError('')} />
            {exporting ? (
              <>
                <Label htmlFor='db-export-repeat' className='mt-3 mb-1 block'>{t('Confirm backup password')}</Label>
                <Input id='db-export-repeat' ref={repeatRef} type='password' autoComplete='new-password' maxLength={1024} onChange={() => setError('')} />
              </>
            ) : (
              <p className='hint mt-2'>{t('Supports encrypted .a2ab backups and legacy .sql files. Legacy SQL does not include recovery keys, so another machine still needs the original keys.')}</p>
            )}
            {error ? <p role='alert' className='mt-2 text-destructive'>{error}</p> : null}
          </form>
        </DialogBody>
        <DialogFooter>
          <Button ref={cancelRef} variant='outline' onClick={close}>{t('Cancel')}</Button>
          <Button variant={exporting ? 'default' : 'destructive'} type='submit' form={`db-${mode}-password-form`}>
            {exporting ? t('Choose location and encrypt backup') : t('Choose a backup file and restore')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
