import * as React from 'react'
import { Badge, Button, SegmentedControl, Sheet, SheetBody, SheetContent, SheetFooter, SheetHeader, SheetTitle, StatusDot } from '@ui'
import { t } from '../i18n'
import { shared, type HiddenAccountsSummary } from './accounts-shared'
import type { HiddenAccountProvider } from '../../../ui-shared/plugins'
import { hiddenAction, hiddenDetail, hiddenFailed, hiddenInFilter, hiddenLabel, type HiddenFilter } from '../../../ui-shared/plugin-hidden'
import { getPluginStartup, refreshPluginStartup } from '../../../ui-shared/plugin-startup'
import { requestPluginDiagnostics } from '../../../ui-shared/plugin-startup-view'

export function brokenCountOf(hidden: HiddenAccountsSummary): number {
  return hidden.providers.filter(hiddenFailed).length
}

export function HiddenAccountsEntry({ hidden, onOpen }: {
  hidden: HiddenAccountsSummary; onOpen(): void
}): React.ReactElement | null {
  if (hidden.total <= 0) return null
  return <Button id='btn-hidden-accounts' variant='outline' className='acct-hidden-entry'
    title={t('Accounts hidden because their provider plugin is not ready')} onClick={() => { refreshPluginStartup(); onOpen() }}>
    <svg viewBox='0 0 24 24' width='14' height='14' fill='none' stroke='currentColor' strokeWidth={1.7}
      strokeLinecap='round' strokeLinejoin='round' aria-hidden='true'>
      <path d='M2.5 12s3.4-6 9.5-6 9.5 6 9.5 6-3.4 6-9.5 6-9.5-6Z' /><path d='m4 20 16-16' />
    </svg>
    {t('Hidden accounts')}<Badge variant={brokenCountOf(hidden) ? 'warning' : 'outline'} shape='tag'>{hidden.providers.length}</Badge>
  </Button>
}

export function HiddenAccountsDrawer({ hidden, open, onOpenChange }: {
  hidden: HiddenAccountsSummary; open: boolean; onOpenChange(open: boolean): void
}): React.ReactElement {
  const [filter, setFilter] = React.useState<HiddenFilter>('all')
  React.useEffect(() => { if (open) refreshPluginStartup() }, [open])
  const state = getPluginStartup()
  const countOf = (value: HiddenFilter) => hidden.providers.filter(item => hiddenInFilter(item, value)).length
  const shown = hidden.providers.filter(item => hiddenInFilter(item, filter))
  const loading = hidden.providers.filter(item => item.reason === 'loading').length
  const goPlugins = (item?: HiddenAccountProvider) => {
    if (item?.packId && !['not_installed', 'pack_disabled', 'provider_disabled'].includes(item.reason)) requestPluginDiagnostics(item.packId)
    onOpenChange(false)
    shared().wbApp?.showPage?.('plugins')
  }
  return <Sheet open={open} onOpenChange={next => { if (!next) onOpenChange(false) }}>
    <SheetContent side='right' showHandle={false} className='w-[min(480px,94vw)]'>
      <SheetHeader><SheetTitle>{t('Hidden accounts')}</SheetTitle></SheetHeader>
      <SheetBody className='acct-hidden-body'>
        <p className='acct-hidden-lead'>{t('{providers} providers, {total} accounts. The data is still here — there is just no working plugin to run them right now.', {
          providers: hidden.providers.length, total: hidden.total,
        })}</p>
        {loading ? <p className='m-0 text-xs text-muted-foreground' role='status'>
          {t('Loading · Accounts from {n} providers are temporarily hidden and will return when ready.', { n: loading })}
        </p> : null}
        {state.error && !state.unavailable ? <p className='m-0 text-xs text-muted-foreground' role='status'>
          {t('Plugin status is temporarily unavailable. Showing the previous result; this does not mean the plugin failed to start.')}
          <Button variant='link' size='xs' onClick={refreshPluginStartup}>{t('Refresh status')}</Button>
        </p> : null}
        <SegmentedControl aria-label={t('Filter by reason')} value={filter} onValueChange={setFilter} options={[
          { value: 'all', label: t('All'), count: countOf('all') },
          { value: 'loading', label: t('Not ready yet'), count: countOf('loading') },
          { value: 'missing', label: t('Not installed'), count: countOf('missing') },
          { value: 'disabled', label: t('Disabled'), count: countOf('disabled') },
          { value: 'broken', label: t('Load failed'), count: countOf('broken') },
        ]} />
        <div className='acct-hidden-rows'>{shown.map(item => <div key={item.id} className='acct-hidden-row'>
          <span className='acct-hidden-avatar' aria-hidden='true'>{t(item.label).trim().slice(0, 1)}</span>
          <div className='acct-hidden-info'>
            <div className='acct-hidden-name'><span className='truncate'>{t(item.label)}</span>
              <Badge variant={hiddenFailed(item) ? 'warning' : 'outline'} shape='tag'>{hiddenLabel(item, t)}</Badge>
            </div>
            <p className='acct-hidden-detail'>{t('Keeps {n} accounts', { n: item.count })} · {hiddenDetail(item, t, state.value?.now)}</p>
          </div>
          <Button variant='outline' size='sm' onClick={() => goPlugins(item)}>{hiddenAction(item, t)}</Button>
        </div>)}{!shown.length ? <p className='acct-hidden-none'>{t('No accounts in this category')}</p> : null}</div>
        <p className='acct-hidden-foot'>{t('This affects display only: uninstalling or disabling a plugin never deletes accounts. Install, enable and load the plugin successfully and they come back automatically.')}</p>
      </SheetBody>
      <SheetFooter><Button variant='outline' onClick={() => onOpenChange(false)}>{t('Got it')}</Button>
        <Button onClick={() => goPlugins()}>{t('Open Plugins')}</Button></SheetFooter>
    </SheetContent>
  </Sheet>
}

export function HiddenAccountsEmptyNote({ hidden, onOpen }: { hidden: HiddenAccountsSummary; onOpen(): void }): React.ReactElement {
  return <div className='acct-hidden-empty' role='status'>
    <span><StatusDot variant={brokenCountOf(hidden) ? 'warning' : 'muted'} className='size-1.5' />{t('{n} accounts are not shown', { n: hidden.total })}</span>
    <Button variant='outline' size='sm' onClick={onOpen}>{t('View')}</Button>
  </div>
}
