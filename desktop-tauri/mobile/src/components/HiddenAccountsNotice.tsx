import { useEffect, useId, useState, useSyncExternalStore } from 'react'
import { Badge, Button, Sheet, SheetBody, SheetContent, SheetFooter, SheetHeader, SheetTitle, SegmentedControl } from '@ui'
import type { HiddenAccountProvider, HiddenAccountsSummary } from '../../../ui-shared/plugins'
import { hiddenAction, hiddenDetail, hiddenFailed, hiddenInFilter, hiddenLabel, type HiddenFilter } from '../../../ui-shared/plugin-hidden'
import { getPluginStartup, hiddenWithStartup, refreshPluginStartup, subscribePluginStartup } from '../../../ui-shared/plugin-startup'
import { store } from '../state/store'
import { t } from '../i18n'

export function HiddenAccountsNotice({ hidden }: { hidden?: HiddenAccountsSummary }) {
  const [open, setOpen] = useState(false)
  const [filter, setFilter] = useState<HiddenFilter>('all')
  const state = useSyncExternalStore(subscribePluginStartup, getPluginStartup)
  const prefix = useId()
  useEffect(() => { if (open) refreshPluginStartup() }, [open])
  if (!hidden?.total) return null
  const providers = hiddenWithStartup(hidden).providers
  const broken = providers.filter(hiddenFailed).length
  const loading = providers.filter(item => item.reason === 'loading').length
  const countOf = (value: HiddenFilter) => providers.filter(item => hiddenInFilter(item, value)).length
  const shown = providers.filter(item => hiddenInFilter(item, filter))
  const goPlugins = (item?: HiddenAccountProvider) => {
    setOpen(false)
    const inspect = item?.packId && !['not_installed', 'pack_disabled', 'provider_disabled'].includes(item.reason)
    store.push('plugins', inspect ? { id: item.packId! } : undefined)
  }
  return <>
    <div className='m-section'>
      <Button variant='outline' size='sm' id={`${prefix}-entry`} onClick={() => setOpen(true)}>
        {t('Hidden accounts')} {providers.length}
        {broken ? <Badge variant='warning' shape='tag'>{t('Load failed')}</Badge> : null}
      </Button>
    </div>
    <Sheet open={open} onOpenChange={next => { if (!next) setOpen(false) }}>
      <SheetContent>
        <SheetHeader><SheetTitle>{t('Hidden accounts')}</SheetTitle></SheetHeader>
        <SheetBody className='gap-3'>
          <p className='m-0 text-xs leading-relaxed text-muted-foreground'>
            {t('{providers} providers, {total} accounts. The data is still here — there is just no working plugin to run them right now.', { providers: providers.length, total: hidden.total })}
          </p>
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
          {shown.map(item => <div key={item.id} className='flex items-center gap-2.5 border-t border-hairline pt-2.5'>
            <div className='min-w-0 flex-1'>
              <div className='flex items-center gap-1.5 text-[13px] font-medium'><span className='truncate'>{t(item.label)}</span>
                <Badge variant={hiddenFailed(item) ? 'warning' : 'secondary'} shape='tag'>{hiddenLabel(item, t)}</Badge></div>
              <p className='m-0 mt-0.5 text-xs text-muted-foreground'>
                {t('Keeps {n} accounts', { n: item.count })} · {hiddenDetail(item, t, state.value?.now)}
              </p>
            </div>
            <Button variant='outline' size='sm' onClick={() => goPlugins(item)}>{hiddenAction(item, t)}</Button>
          </div>)}
          {!shown.length ? <p className='m-0 text-center text-xs text-muted-foreground'>{t('No accounts in this category')}</p> : null}
          <p className='m-0 text-xs leading-relaxed text-muted-foreground'>{t('This affects display only: uninstalling or disabling a plugin never deletes accounts. Install, enable and load the plugin successfully and they come back automatically.')}</p>
        </SheetBody>
        <SheetFooter><Button variant='outline' onClick={() => setOpen(false)}>{t('Got it')}</Button>
          <Button onClick={() => goPlugins()}>{t('Open Plugins')}</Button></SheetFooter>
      </SheetContent>
    </Sheet>
  </>
}
