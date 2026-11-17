/** 分组与勾选账号共用的批量操作弹窗。 */

import * as React from 'react'
import {
  Button, Dialog, DialogBody, DialogContent, DialogFooter, DialogHeader, DialogSection,
  DialogTitle, Input, Label, RadioGroup, RadioGroupItem, Select, SelectContent,
  SelectItem, SelectTrigger, SelectValue,
  Field, FieldLabel, FieldSet,
} from '@ui'
import { t } from '../i18n'
import {
  ACCOUNT_CONCURRENCY_MAX as CONC_MAX, errorMessage, esc, shared, toast,
  type AccountRecord, type BatchAccountsPayload, type BatchAccountsResult, type BatchLimiterMode, type LimiterGroup,
} from './accounts-shared'
import {
  displayNameOf, providerFeatures, providerOf, supportsProxyRotate, supportsUsage,
} from './accounts-domain'
import { allAccounts, findAccount, syncBalancesSnapshot } from './accounts-data'
import { confirmBatchResult, refreshSavedAccounts } from './accounts-update'
import { tokenRefreshSkipReason } from './accounts-authorization'
import { refreshAccountTokens, type BatchTokenRefreshProgress } from './accounts-batch-refresh'
import { limiterDraftOf, LimiterSection } from './accounts-dialog-limiter'
import { readUsageDraft, UsageSettingsSection, usageDraftOf, type UsageDraft } from './accounts-dialog-usage'
import { draftOfProxy, ProxyForm, readProxyDraft, type ProxyDraft } from './accounts-proxy-form'

const BATCH_ACTIONS = [
  { value: 'enable', label: () => t('enabled') },
  { value: 'disable', label: () => t('Disable') },
  { value: 'refresh', label: () => t('Refresh token') },
  { value: 'proxy', label: () => t('Change proxy') },
  { value: 'concurrency', label: () => t('Set concurrency limit') },
  { value: 'usageQuery', label: () => t('Query settings') },
  { value: 'limiters', label: () => t('Limiter') },
  { value: 'remove', label: () => t('Delete') },
] as const

const LIMITER_MODES: Array<{ value: BatchLimiterMode; label: () => string }> = [
  { value: 'append', label: () => t('Append rules (keep existing)') },
  { value: 'replace', label: () => t('Replace all rules') },
  { value: 'enable', label: () => t('Enable all rules') },
  { value: 'disable', label: () => t('Disable all rules') },
  { value: 'clear', label: () => t('Clear all rules') },
]

function labelOf(account: AccountRecord | null | undefined): string {
  return displayNameOf(account) || account?.id || ''
}

export function BatchDialog({ ids, action, onClose }: { ids: string[]; action: string; onClose: () => void }) {
  const [current, setCurrent] = React.useState(BATCH_ACTIONS.some(item => item.value === action) ? action : 'enable')
  const [proxyDraft, setProxyDraft] = React.useState<ProxyDraft>(() => draftOfProxy(null))
  const [concDraft, setConcDraft] = React.useState('')
  // 混选时没有共同的间隔；不预填，避免直接执行就覆盖整组配置。
  const [usageDraft, setUsageDraft] = React.useState<UsageDraft>(() => ({ ...usageDraftOf(null), intervalInput: '' }))
  const [limiterMode, setLimiterMode] = React.useState<BatchLimiterMode>('append')
  const [limiterDraft, setLimiterDraft] = React.useState<LimiterGroup[]>([])
  const [sourceId, setSourceId] = React.useState('')
  const [busy, setBusy] = React.useState(false)
  const [result, setResult] = React.useState<React.ReactNode>('')
  const [refreshProgress, setRefreshProgress] = React.useState<BatchTokenRefreshProgress | null>(null)
  const running = React.useRef(false)
  const accounts = ids.map(id => findAccount(id)).filter((account): account is AccountRecord => account !== null)
  const usageAccounts = accounts.filter(supportsUsage)
  const editingRules = current === 'limiters' && (limiterMode === 'append' || limiterMode === 'replace')
  const needsBalance = editingRules && limiterDraft.some(group => group.conditions.some(condition => condition.metric !== 'tokens' && condition.metric !== 'requests'))
  const targets = current === 'refresh'
    ? accounts.filter(account => !tokenRefreshSkipReason(account))
    : current === 'usageQuery' || needsBalance ? usageAccounts : accounts
  const skippedAccounts = accounts.filter(account => !targets.includes(account))
  const copyOptions = current === 'usageQuery' ? usageAccounts : accounts
  const source = copyOptions.find(account => account.id === sourceId)
  const concHardCap = accounts.reduce(
    (cap, account) => Math.max(cap, Number(providerFeatures(providerOf(account)).concurrencyDefault) || 0), 0,
  )
  const allowRotate = accounts.length > 0 && accounts.every(supportsProxyRotate)
  const destructive = current === 'remove' || (current === 'limiters' && (limiterMode === 'replace' || limiterMode === 'clear'))
  const verb = current === 'limiters'
    ? (limiterMode === 'append' ? t('Append limiter rules') : LIMITER_MODES.find(item => item.value === limiterMode)!.label())
    : BATCH_ACTIONS.find(item => item.value === current)!.label()

  function copySettings(): void {
    if (!source) return
    if (current === 'usageQuery') setUsageDraft(usageDraftOf(source))
    else setLimiterDraft(limiterDraftOf(source))
    setResult('')
    toast(t('Copied to the draft; applies to the selected accounts only after running'))
  }

  async function confirmOperation(): Promise<boolean> {
    const ask = shared().wbConfirm?.ask
    if (current === 'remove') {
      const ok = await Promise.resolve(ask?.({
        title: t('Delete selected accounts'),
        html: t('Delete the selected <strong>{count}</strong> accounts? This cannot be undone, and their login state will be removed as well.', { count: ids.length }),
        okText: t('Delete'), okClass: 'danger',
      }) ?? false)
      if (!ok) return false
    }
    if (current === 'disable' || current === 'remove') {
      const picked = new Set(ids)
      const survivors = allAccounts().filter(account => account.enabled !== false && !picked.has(account.id))
      if (!survivors.length) {
        const what = current === 'remove' ? t('Delete') : t('Disable')
        return Promise.resolve(ask?.({
          title: t('This will {what} all enabled accounts', { what }),
          html: t('This will <strong>{what}</strong> all enabled accounts, and forwarding will be unavailable. Continue?', { what }),
          okText: what, okClass: current === 'remove' ? 'danger' : 'primary',
        }) ?? false)
      }
    }
    if (current === 'limiters' && limiterMode === 'replace') {
      return Promise.resolve(ask?.({
        title: t('Replace selected accounts’ limiter rules'),
        html: t('Replace all limiter rules of <strong>{count}</strong> accounts with the <strong>{rules}</strong> draft rules? Existing rules will not be kept.', {
          rules: limiterDraft.length, count: targets.length,
        }) + (skippedAccounts.length ? `<p>${t('These {count} accounts do not support balance queries and will be skipped: {names}', {
          count: skippedAccounts.length, names: esc(skippedAccounts.map(labelOf).join(t(', '))),
        })}</p>` : ''),
        okText: t('Replace all rules'), okClass: 'danger',
      }) ?? false)
    }
    if (current === 'limiters' && limiterMode === 'clear') {
      return Promise.resolve(ask?.({
        title: t('Clear selected accounts’ limiter rules'),
        html: t('Clear all limiter rules from <strong>{count}</strong> accounts? Balance and Token limits will be removed; disabled accounts will not be enabled automatically.', { count: targets.length }),
        okText: t('Clear all rules'), okClass: 'danger',
      }) ?? false)
    }
    return true
  }

  async function run(): Promise<void> {
    if (running.current) return
    if (!ids.length) { toast(t('No accounts selected'), 'err'); return }
    if (!targets.length) { toast(t('No accounts eligible for this operation'), 'err'); return }
    const payload: BatchAccountsPayload = { action: current, ids }
    try {
      if (current === 'proxy') payload.proxy = readProxyDraft(proxyDraft)
      if (current === 'concurrency') {
        const raw = Number(concDraft.trim())
        if (!concDraft.trim() || !Number.isFinite(raw) || raw < 0) {
          throw new Error(t('Concurrency limit must be an integer from 0 to {max}', { max: CONC_MAX }))
        }
        payload.maxConcurrent = Math.min(CONC_MAX, Math.round(raw))
      }
      if (current === 'usageQuery') {
        const changes = readUsageDraft(usageDraft)
        if ('error' in changes) throw new Error(changes.error)
        payload.usageQuery = changes.value
      }
      if (current === 'limiters') {
        payload.limiterMode = limiterMode
        if (editingRules) {
          if (!limiterDraft.length) throw new Error(t('Add at least one limiter rule; use the clear operation to remove all rules'))
          if (limiterDraft.some(group => group.conditions.some(condition =>
            condition.sourceId !== '' && condition.sourceId !== 'local' && condition.sourceId !== 'account'))) {
            throw new Error(t('Plan instances cannot be copied between accounts. Use a general quota or local protection condition instead.'))
          }
          payload.limiterGroups = structuredClone(limiterDraft)
        }
      }
    } catch (error) {
      setResult(<span className='text-destructive'>{errorMessage(error)}</span>)
      toast(errorMessage(error), 'err')
      return
    }

    running.current = true
    setBusy(true)
    setResult('')
    setRefreshProgress(null)
    try {
      if (!(await confirmOperation())) return
      let data: BatchAccountsResult
      if (current === 'refresh') {
        data = await refreshAccountTokens(ids, setRefreshProgress)
      } else {
        const bridge = shared().workbuddyDesktop
        if (!bridge?.batchAccounts) throw new Error(t('Bulk operation service is unavailable; reopen the page'))
        data = confirmBatchResult(await bridge.batchAccounts(payload), payload)
      }
      const accepted = data.ok || []
      const changed = current === 'remove' ? (data.removed || []).length
        : current === 'refresh' ? accepted.length : accepted.filter(item => item.changes?.length).length
      const unchanged = current === 'remove' ? 0 : accepted.length - changed
      const failed = data.failed || []
      const skipped = data.skipped || []
      const summary = current === 'refresh'
        ? t('Token refresh complete: {ok} succeeded, {skip} skipped, {fail} failed', {
          ok: changed, skip: skipped.length, fail: failed.length,
        })
        : t('{verb} completed: {ok} changed, {unchanged} unchanged, {skip} skipped, {fail} failed', {
          verb, ok: changed, unchanged, skip: skipped.length, fail: failed.length,
        })
      toast(summary, failed.length ? 'err' : 'ok')
      if (failed.length || skipped.length) {
        setResult(<div role='status' className='space-y-2'>
          <p>{summary}</p>
          {skipped.length ? <p>{t('{count} skipped: ', { count: skipped.length })}
            {skipped.map(item => `${labelOf(findAccount(String(item.id))) || item.name || item.id}（${item.reason}）`).join(t('; '))}
          </p> : null}
          {failed.length ? <p className='text-destructive'>{t('{count} failed:', { count: failed.length })}
            {failed.map(item => `${labelOf(findAccount(String(item.id))) || item.id}（${item.error}）`).join(t('; '))}
          </p> : null}
        </div>)
      } else {
        onClose()
      }
      try {
        await refreshSavedAccounts()
        await syncBalancesSnapshot()
      } catch (error) {
        toast(t('Bulk changes were saved, but the list failed to refresh: {error}', { error: errorMessage(error) }), 'err')
      }
    } catch (error) {
      setResult(<span className='text-destructive'>{errorMessage(error)}</span>)
      toast(t('Batch operation failed: {error}', { error: errorMessage(error) }), 'err')
    } finally {
      running.current = false
      setBusy(false)
      setRefreshProgress(null)
    }
  }

  return (
    <Dialog open onOpenChange={next => { if (next || busy) return; onClose() }}>
      <DialogContent className='w-[min(900px,calc(100vw-48px))]' showCloseButton={!busy}>
        <DialogHeader><DialogTitle>{t('Batch operation · {count} accounts selected', { count: ids.length })}</DialogTitle></DialogHeader>
        <DialogBody>
          <FieldSet disabled={busy} className='m-0 flex min-w-0 flex-col gap-4 border-0 p-0'>
            <DialogSection>
              <h3>{t('Applies to the following accounts')}</h3>
              <p className='max-h-[84px] overflow-y-auto'>{targets.map(labelOf).join(t(', ')) || t('No accounts eligible for this operation')}</p>
              {skippedAccounts.length ? <p className='max-h-[84px] overflow-y-auto'>{current === 'refresh'
                ? <>{t('{count} skipped: ', { count: skippedAccounts.length })}
                  {skippedAccounts.map(account => `${labelOf(account)}（${tokenRefreshSkipReason(account)}）`).join(t('; '))}
                </>
                : t('These {count} accounts do not support balance queries and will be skipped: {names}', {
                  count: skippedAccounts.length, names: skippedAccounts.map(labelOf).join(t(', ')),
                })}</p> : null}
              {current === 'refresh'
                ? <p>{t('Refresh tokens for the selected accounts one at a time, even if they have not expired. Enabled status, proxies, and forwarding order are kept.')}</p>
                : current !== 'remove' ? <p>{t('Only changes the chosen settings; labels, credentials, forwarding order, and other settings are kept.')}</p> : null}
            </DialogSection>
            <DialogSection>
              <h3>{t('Actions')}</h3>
              <RadioGroup value={current} onValueChange={value => { setCurrent(value); setResult('') }}
                disabled={busy} className='flex-row flex-wrap items-center gap-5' aria-label={t('Batch action')}>
                {BATCH_ACTIONS.map(item => <Label key={item.value} className='inline-flex cursor-pointer items-center gap-2 font-normal'>
                  <RadioGroupItem value={item.value} />{item.label()}
                </Label>)}
              </RadioGroup>
              {current === 'proxy' ? <div className='mt-2.5'>
                <ProxyForm draft={proxyDraft} onChange={setProxyDraft} idPrefix='batch-proxy' allowRotate={allowRotate} />
              </div> : null}
              {current === 'concurrency' ? <div className='mt-2.5'>
                <Field orientation='horizontal' className='field-row'>
                  <FieldLabel htmlFor='batch-conc-input'>{t('Concurrency limit')}</FieldLabel>
                  <Input id='batch-conc-input' type='number' min={0} max={CONC_MAX} step={1}
                    className='max-w-[110px]' placeholder={t('Concurrency limit')} value={concDraft}
                    onChange={event => setConcDraft(event.currentTarget.value)} />
                  <span className='detail'>{t('0 = unlimited')}</span>
                </Field>
                <p className='detail mt-2.5'>{concHardCap > 0
                  ? t('Concurrency limit 0–{max}; for providers with a default (e.g. CodeArts), 0 means that default; accounts at the limit are skipped and requests go to other accounts', { max: CONC_MAX })
                  : t('Concurrency limit 0–{max}; accounts at the limit are skipped and requests go to other accounts', { max: CONC_MAX })}</p>
              </div> : null}
              {current === 'limiters' ? <div className='mt-2.5'>
                <Field orientation='horizontal' className='field-row'>
                  <FieldLabel htmlFor='batch-limiter-mode'>{t('Rule operation')}</FieldLabel>
                  <Select value={limiterMode} onValueChange={value => { setLimiterMode(value as BatchLimiterMode); setResult('') }}>
                    <SelectTrigger id='batch-limiter-mode' className='min-w-[220px]'>
                      <SelectValue>{LIMITER_MODES.find(item => item.value === limiterMode)!.label()}</SelectValue>
                    </SelectTrigger>
                    <SelectContent>{LIMITER_MODES.map(item => <SelectItem key={item.value} value={item.value}>{item.label()}</SelectItem>)}</SelectContent>
                  </Select>
                </Field>
                <p className='detail mt-2.5'>{limiterMode === 'append'
                  ? t('Keeps each account’s existing rules and appends the rules below; identical rules are not added twice.')
                  : limiterMode === 'replace'
                    ? t('Replaces every account’s entire rule list with the rules below; confirmation is required before running.')
                    : limiterMode === 'clear'
                      ? t('Removes every account’s limiter rules; confirmation is required before running.')
                      : t('Only changes existing rule switches; thresholds, actions, reset modes, and account enabled states are kept.')}</p>
              </div> : null}
              {current === 'usageQuery' || editingRules ? <div className='mt-2.5'>
                <Field orientation='horizontal' className='field-row'>
                  <FieldLabel htmlFor='batch-settings-source'>{t('Copy from account')}</FieldLabel>
                  <Select value={source?.id || ''} disabled={!copyOptions.length} onValueChange={value => setSourceId(String(value))}>
                    <SelectTrigger id='batch-settings-source' className='min-w-0 flex-1'>
                      <SelectValue>{source ? labelOf(source) : t('Choose a selected account as a template (optional)')}</SelectValue>
                    </SelectTrigger>
                    <SelectContent>{copyOptions.map(account => <SelectItem key={account.id} value={account.id}>{labelOf(account)}</SelectItem>)}</SelectContent>
                  </Select>
                  <Button variant='outline' size='sm' disabled={!source} onClick={copySettings}>{t('Copy current settings')}</Button>
                </Field>
                {current === 'usageQuery' ? <>
                  <UsageSettingsSection draft={usageDraft} onChange={setUsageDraft} idPrefix='batch-usage' />
                  <p className='detail mt-2.5'>{t('Sets the auto-query switch, interval, and retry count; balance query credentials are never copied or overwritten.')}</p>
                </> : <LimiterSection draft={limiterDraft} onChange={setLimiterDraft} allowBalance={usageAccounts.length > 0} />}
              </div> : null}
              {current === 'usageQuery' && !usageDraft.enabled ? <p className='detail mt-2.5'>
                {t('With auto-query off, balance limiters use the last reading; manual queries remain available.')}
              </p> : null}
              {current === 'limiters' ? <p className='detail mt-2.5'>
                {t('Token rules use each account’s existing window usage; saving does not reset usage. Disabling or clearing rules does not enable accounts.')}
              </p> : null}
            </DialogSection>
          </FieldSet>
          {refreshProgress ? <p role='status' className='detail'>
            {t('Refreshing tokens: {completed}/{total} processed, current account: {name}', refreshProgress)}
          </p> : null}
          <div className='detail min-h-[18px]'>{result}</div>
        </DialogBody>
        <DialogFooter>
          <div className='mr-auto' />
          <Button variant='outline' disabled={busy} onClick={onClose}>{t('Cancel')}</Button>
          <Button variant={destructive ? 'destructive' : 'default'} disabled={busy || !targets.length} onClick={() => void run()}>
            {busy ? t('Running…') : t('Run ({action})', { action: verb })}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
