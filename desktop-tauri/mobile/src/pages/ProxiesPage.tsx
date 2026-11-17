import { useState } from 'react'
import {
  Button, Card, Dialog, DialogBody, DialogContent, DialogFooter, DialogHeader,
  DialogTitle, Input, SegmentedControl, Switch,
} from '@ui'
import { PageIntro, IconButton, Empty, ErrorNote, SkeletonList, copyWithToast } from '../components/ui'
import { GlyphLogo } from '../components/ProviderLogo'
import { Icon } from '../components/Icon'
import { PullIndicator, usePullToRefresh } from '../components/PullToRefresh'
import { store } from '../state/store'
import { fetchProxyPool, removeProxyItem, setProxyEnabled, syncClashProxies, testProxy, type PoolItem, type PoolPayload } from '../api/gateway'
import { formatRelative } from '../lib/format'
import { usePolling } from '../lib/usePolling'
import { t } from '../i18n'
import { proxyServiceLabels } from '../../../ui-shared/proxy-usage'
import { shouldShowProxyRetirement } from '../../../ui-shared/proxy-retirement'
import { ProxyRetirementDialog } from '../../../ui-shared/proxy-retirement-dialog'
import { ProxyEditorSheet, ProxyTestSettingsSheet } from './proxy-editor-sheet'
import { serverIdentity } from '../lib/servers'

export function ProxiesPage() {
  const [data, setData] = useState<PoolPayload | null>(null)
  const [error, setError] = useState('')
  const [busy, setBusy] = useState('')
  const [testing, setTesting] = useState('')
  const [keyword, setKeyword] = useState('')
  const [status, setStatus] = useState('all')
  const [editor, setEditor] = useState<{ item: PoolItem | null } | null>(null)
  const [settingsOpen, setSettingsOpen] = useState(false)
  const [deleting, setDeleting] = useState<PoolItem | null>(null)
  const identity = serverIdentity()
  const [noticeOpen, setNoticeOpen] = useState(() => shouldShowProxyRetirement(identity))

  const load = async () => {
    try {
      const next = await fetchProxyPool()
      if (serverIdentity() !== identity) return
      setData(next)
      setError('')
    } catch (failure) {
      if (serverIdentity() === identity) setError(message(failure))
    }
  }
  usePolling(load, 120_000)
  const pull = usePullToRefresh({ onRefresh: load })
  const items = data?.items || []
  const query = keyword.trim().toLowerCase()
  const shown = items.filter(item => (status === 'all' || stateOf(item) === status)
    && (!query || [item.name, addressOf(item), item.host, item.port, item.protocol].join(' ').toLowerCase().includes(query)))
  const statuses = [
    { value: 'all', label: t('All'), count: items.length },
    { value: 'ok', label: t('usable'), count: items.filter(item => stateOf(item) === 'ok').length },
    { value: 'fail', label: t('Failed'), count: items.filter(item => stateOf(item) === 'fail').length },
    { value: 'none', label: t('Not checked'), count: items.filter(item => stateOf(item) === 'none').length },
    { value: 'off', label: t('Disabled'), count: items.filter(item => stateOf(item) === 'off').length },
  ]

  async function syncClash(): Promise<void> {
    if (busy) return
    setBusy('sync-clash')
    try {
      const next = await syncClashProxies()
      if (serverIdentity() !== identity) return
      if (!Array.isArray(next.items)) throw new Error(t('Unexpected proxy list response'))
      setData(next)
      store.toast(next.syncError ? t('Could not sync: {reason}', { reason: next.syncError })
        : next.changes ? t('✅ Synced Clash Verge endpoints ({n} changes)', { n: next.changes })
          : t('Already up to date — no changes to sync'), next.syncError ? 'err' : 'ok')
    } catch (failure) { if (serverIdentity() === identity) store.toast(message(failure), 'err') }
    finally { if (serverIdentity() === identity) setBusy('') }
  }

  async function toggle(item: PoolItem, enabled: boolean): Promise<void> {
    if (busy) return
    setBusy(item.id)
    try {
      const next = await setProxyEnabled(item.id, enabled)
      if (serverIdentity() !== identity) return
      setData(next)
      store.toast(enabled ? t('Enabled "{name}"', { name: item.name }) : t('Disabled "{name}"', { name: item.name }))
    } catch (failure) { if (serverIdentity() === identity) store.toast(message(failure), 'err') }
    finally { if (serverIdentity() === identity) setBusy('') }
  }

  async function test(item: PoolItem): Promise<void> {
    if (busy) return
    setBusy(item.id)
    setTesting(item.id)
    try {
      const result = await testProxy(item.id)
      if (serverIdentity() !== identity) return
      if (result.items) setData(previous => ({ ...previous, items: result.items }))
      store.toast(result.success ? t('Proxy available') : result.error || t('Connection failed'), result.success ? 'ok' : 'err')
      if (result.saveError) store.toast(t('Could not save the test result: {reason}', { reason: result.saveError }), 'err')
    } catch (failure) { if (serverIdentity() === identity) store.toast(message(failure), 'err') }
    finally { if (serverIdentity() === identity) { setBusy(''); setTesting('') } }
  }

  async function remove(): Promise<void> {
    if (!deleting || busy) return
    setBusy(deleting.id)
    try {
      const next = await removeProxyItem(deleting.id)
      if (serverIdentity() !== identity) return
      setData(next)
      setDeleting(null)
      store.toast(t('Deleted "{name}"', { name: deleting.name }))
    } catch (failure) { if (serverIdentity() === identity) store.toast(message(failure), 'err') }
    finally { if (serverIdentity() === identity) setBusy('') }
  }

  return <div className='m-page' ref={pull.bindRef}>
    <PullIndicator pull={pull.pull} refreshing={pull.refreshing} />
    <PageIntro subtitle={data ? t('{n} enabled', { n: items.filter(item => item.enabled).length }) : t('Loading…')}
      actions={<><IconButton name='refresh' label={t('Refresh')} onClick={() => void load()} />
        <IconButton name='plus' label={t('New proxy')} primary onClick={() => setEditor({ item: null })} /></>} />
    <div className='m-section mt-0'>
      <Card className='m-card m-card-pad'>
        <Button variant='outline' size='touch-sm' className='w-full' disabled={Boolean(busy)} onClick={() => void syncClash()}>{busy === 'sync-clash' ? t('Syncing…') : t('Sync Clash Verge')}</Button>
        <div className='mt-3 flex items-center gap-2'>
          <Input type='search' className='min-w-0 flex-1' value={keyword} aria-label={t('Search proxies')} placeholder={t('Search name / address / port')} onChange={event => setKeyword(event.currentTarget.value)} />
          <Button variant='outline' size='icon-touch' aria-label={t('Test settings')} onClick={() => setSettingsOpen(true)}><Icon name='settings' className='size-4' /></Button>
        </div>
        <SegmentedControl className='mt-3 w-full flex-wrap' options={statuses} value={status} onValueChange={setStatus} aria-label={t('Filter by status')} />
      </Card>
    </div>
    {error ? <ErrorNote text={error} /> : null}
    {!data ? <SkeletonList rows={3} height={120} /> : null}
    {data && !items.length ? <div className='m-section'><Card className='m-card m-card-pad'>
      <Empty text={t('Add a manual HTTP / SOCKS5 address or sync Clash Verge endpoints for use by accounts and service routes.')} />
      <Button className='w-full' onClick={() => setEditor({ item: null })}>{t('Add your first proxy')}</Button>
    </Card></div> : null}
    {data && items.length > 0 && !shown.length ? <div className='m-section'><Card className='m-card m-card-pad'>
      <Empty text={t('No endpoints match the current filters')} /><Button variant='outline' className='w-full' onClick={() => { setKeyword(''); setStatus('all') }}>{t('清空筛选')}</Button>
    </Card></div> : null}
    {shown.map(item => <div key={item.id} className='m-section mt-2.5'>
      <Card className='m-card m-card-pad'>
        <div className='flex items-center gap-3'><GlyphLogo name='globe' /><div className='min-w-0 flex-1'>
          <strong className='text-sm'>{item.name || t('Untitled')}</strong>
          {item.nonDirect ? <p className='mt-1 text-[10px] text-subtle'>{t('Chained')} · {t('Connect through the system proxy first, then through this proxy')}</p> : null}
          {item.source === 'clash' ? <p className='mt-1 text-[10px] text-subtle'>Clash Verge · {t('Read-only: edit it in Clash Verge')}</p> : null}
          <div className='mt-1 flex items-center gap-1.5'>
            <span className='break-all font-mono text-[11px] text-subtle'>{addressOf(item)}</span>
            <Button variant='ghost' size='icon-xs' aria-label={t('Copy address')} onClick={() => void copyWithToast(addressOf(item))}><Icon name='copy' className='size-3' /></Button>
          </div>
        </div><Switch checked={item.enabled} disabled={Boolean(busy) || item.source === 'clash' && item.sourceEnabled === false} aria-label={t('Enable "{name}"', { name: item.name })} onCheckedChange={next => void toggle(item, next)} /></div>
        {item.source === 'clash' && item.sourceEnabled === false ? <p className='mt-2 text-[11px] text-subtle'>{t('This listener is disabled in Clash Verge, so enabling it here has no effect — turn it on in Clash Verge')}</p> : null}
        {item.resolveError ? <p className='mt-2 break-all text-[11px] text-destructive'>{item.resolveError}</p> : null}
        <div className='mt-3 border-t border-border pt-3 text-xs'>
          <div className={item.lastTest?.at ? item.lastTest.success ? 'text-success' : 'text-destructive' : 'text-subtle'}>
            {testing === item.id ? t('Testing…') : item.lastTest?.at ? item.lastTest.success ? t('Connected') : t('Connection failed') : t('Not checked')}
            {item.lastTest?.durationMs !== undefined ? ` · ${item.lastTest.durationMs} ms` : ''}
          </div>
          {item.lastTest?.at ? <p className='mt-1 break-all text-[11px] text-subtle'>
            {item.lastTest.success ? item.lastTest.ip : item.lastTest.error} · {formatRelative(item.lastTest.at)}
          </p> : null}
          <div className='mt-2 flex flex-wrap gap-x-3 gap-y-1 text-[11px] text-subtle'>
            {item.usedBy?.length ? <span>{t('{n} accounts using', { n: item.usedBy.length })}</span> : null}
            {proxyServiceLabels(item, t).map(service => <span key={service}>{service}</span>)}
            {!item.usedBy?.length && !item.usedByServices?.length ? t('Not in use') : null}
          </div>
        </div>
        <div className='mt-3 flex justify-end gap-2'>
          <Button size='touch-sm' variant='outline' disabled={Boolean(busy)} onClick={() => void test(item)}>{t('Test')}</Button>
          {item.source !== 'clash' ? <>
            <Button size='touch-sm' variant='outline' disabled={Boolean(busy)} onClick={() => setEditor({ item })}>{t('Edit')}</Button>
            <Button size='touch-sm' variant='ghost' className='text-destructive' disabled={Boolean(busy)} onClick={() => setDeleting(item)}>{t('Delete')}</Button>
          </> : null}
        </div>
      </Card>
    </div>)}
    {editor ? <ProxyEditorSheet item={editor.item} onClose={() => setEditor(null)} onSaved={setData} /> : null}
    {settingsOpen ? <ProxyTestSettingsSheet onClose={() => setSettingsOpen(false)} /> : null}
    <ProxyRetirementDialog open={noticeOpen} onClose={() => setNoticeOpen(false)} t={t} scope={identity} />
    <Dialog open={Boolean(deleting)} onOpenChange={open => { if (!open && !busy) setDeleting(null) }}>
      <DialogContent className='max-w-[440px]'><DialogHeader><DialogTitle>{t('Delete proxy')}</DialogTitle></DialogHeader>
        <DialogBody>
          <p className='text-sm'>{t('Delete "{name}"?', { name: deleting?.name || '' })}</p>
          {deleting?.usedBy?.length ? <p className='text-xs text-subtle'>{t('{n} accounts use this proxy. After deletion, reconfigure their proxies; they will not connect directly automatically.', { n: deleting.usedBy.length })}</p> : null}
          {deleting && proxyServiceLabels(deleting, t).length ? <p className='text-xs text-destructive'>{t('This proxy is used for software / plugin updates. Deleting it makes those routes unavailable; they will not automatically fall back to direct connections.')}</p> : null}
        </DialogBody><DialogFooter><Button variant='outline' disabled={Boolean(busy)} onClick={() => setDeleting(null)}>{t('Cancel')}</Button>
          <Button variant='destructive' disabled={Boolean(busy)} onClick={() => void remove()}>{t('Delete')}</Button></DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
}

function addressOf(item: PoolItem): string {
  const proxy = item.resolved || item
  if (!proxy.host || !proxy.port) return t('Incomplete address')
  const host = proxy.host.includes(':') && !proxy.host.startsWith('[') ? `[${proxy.host}]` : proxy.host
  return `${proxy.protocol || 'http'}://${host}:${proxy.port}`
}
function stateOf(item: PoolItem): string { return !item.enabled ? 'off' : !item.lastTest?.at ? 'none' : item.lastTest.success ? 'ok' : 'fail' }
function message(failure: unknown): string { return failure instanceof Error ? failure.message : String(failure) }
