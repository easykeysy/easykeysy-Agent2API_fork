import * as React from 'react'
import { Button, Dialog, DialogBody, DialogContent, DialogFooter, DialogHeader, DialogTitle, confirmDialog } from '@ui'
import { t } from '../i18n'
import { confirmBatchResult, refreshSavedAccounts } from './accounts-update'
import type { RoutingEditorState } from './routing-editor-state'
import { bridge, errorMessage, notify, type RoutingWorkspace } from './routing-types'

export function useRoutingAccounts(editor: RoutingEditorState, onStart: () => void, onFinished: () => void) {
  const [busy, setBusy] = React.useState(false)
  const [error, setError] = React.useState('')
  const pending = React.useRef(false)
  const sync = async (readWorkspace: boolean) => {
    if (readWorkspace) {
      try { editor.syncWorkspace(await bridge().getRoutingWorkspace()) }
      catch (failure) { setError(t('The account was saved, but the list failed to refresh: {error}', { error: errorMessage(failure) })) }
    }
    try { await refreshSavedAccounts() }
    catch (failure) { setError(t('The account was saved, but the list failed to refresh: {error}', { error: errorMessage(failure) })) }
  }
  const setEnabled = async (ids: string[], enabled: boolean, batch = false) => {
    const workspace = editor.draft?.workspace
    if (pending.current || editor.busy || editor.loading || editor.scope || !workspace) return
    if (!workspace.globalAccountControl || workspace.legacyGlobalParticipation) return
    const selected = new Set(ids)
    const targets = workspace.accounts.filter(account => selected.has(account.id) && (account.enabled !== false) !== enabled)
    if (!targets.length) return
    pending.current = true; onStart(); setBusy(true); setError('')
    try {
      if (batch && !enabled && !await confirmDialog(t('Disable the {n} selected accounts for all routes. This change is saved immediately and does not delete the accounts.', { n: targets.length }), {
        title: t('Disable accounts in bulk'), confirmText: t('Disable account'), destructive: true,
      })) return
      const payload = { action: enabled ? 'enable' : 'disable', ids: targets.map(account => account.id) }
      const result = confirmBatchResult(await bridge().batchAccounts(payload), payload)
      editor.syncAccounts(result.list!.accounts!)
      const failed = result.failed || [], skipped = result.skipped || []
      if (failed.length || skipped.length) {
        const details = [...failed.map(item => `${targets.find(account => account.id === item.id)?.name || item.id}: ${item.error}`), ...skipped.map(item => `${item.name || item.id}: ${item.reason}`)].join('; ')
        setError(t('Account status partially updated: {ok} succeeded, {fail} failed, {skip} skipped. {details}', {
          ok: result.ok?.length || 0, fail: failed.length, skip: skipped.length, details,
        }))
      } else notify(enabled ? t('✅ Enabled {n} accounts', { n: targets.length }) : t('✅ Disabled {n} accounts', { n: targets.length }))
      await sync(true)
    } catch (failure) { setError(errorMessage(failure)); void refreshSavedAccounts().catch(() => {}) }
    finally { pending.current = false; onFinished(); setBusy(false) }
  }
  const convert = async (revision: string): Promise<boolean> => {
    if (pending.current || editor.busy || editor.loading || editor.scope) return false
    pending.current = true; onStart(); setBusy(true); setError('')
    try {
      const workspace = await bridge().saveRoutingProfile({ migrateGlobalAccounts: revision })
      if (!workspace.globalAccountControl || workspace.legacyGlobalParticipation || !workspace.revision) {
        throw new Error(t('Conversion could not be confirmed. Reload the global configuration.'))
      }
      editor.syncAccounts(workspace.accounts)
      editor.syncWorkspace(workspace, true)
      notify(t('Converted to account enable/disable controls. Independent route participation lists are unchanged.'))
      await sync(false)
      return true
    } catch (failure) { setError(errorMessage(failure)); return false }
    finally { pending.current = false; onFinished(); setBusy(false) }
  }
  return { busy, error, setEnabled, convert }
}

export function RoutingLegacyAccountsDialog({ legacy, busy, error, onConfirm, onClose }: {
  legacy: NonNullable<RoutingWorkspace['legacyGlobalParticipation']>; busy: boolean; error: string
  onConfirm(): void; onClose(): void
}) {
  return <Dialog open onOpenChange={(open, details) => { if (!open) { if (busy) details.cancel(); else onClose() } }}>
    <DialogContent className='w-[min(620px,calc(100vw-40px))]'>
      <DialogHeader><DialogTitle>{t('Convert legacy global participation settings')}</DialogTitle></DialogHeader>
      <DialogBody>
        <p className='routing-config-note'>{t('These {n} accounts were excluded from the global route. Confirming disables them for all routes. Existing participation lists in independent routes are preserved.', { n: legacy.accounts.length })}</p>
        {legacy.accounts.length ? <div className='routing-model-picker'>{legacy.accounts.map(account => <div key={account.id} className='routing-group-picker-row'>
          <div><b>{account.name || account.id}</b><small>{account.providerLabel}</small></div>
        </div>)}</div> : null}
        <p className='routing-config-note'>{t('Conversion only affects existing accounts. New accounts follow their status on the Accounts page. No disabled account will be enabled.')}</p>
        {error ? <p className='routing-config-error' role='alert'>{error}</p> : null}
      </DialogBody>
      <DialogFooter><Button variant='outline' disabled={busy} onClick={onClose}>{t('Cancel')}</Button>
        <Button variant={legacy.accounts.length ? 'destructive' : 'default'} disabled={busy} onClick={onConfirm}>{busy ? t('Saving…') : t('Confirm conversion')}</Button></DialogFooter>
    </DialogContent>
  </Dialog>
}
