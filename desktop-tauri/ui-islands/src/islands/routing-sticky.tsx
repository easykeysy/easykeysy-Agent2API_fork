import * as React from 'react'
import {
  Badge, Button, Dialog, DialogBody, DialogContent, DialogFooter, DialogHeader, DialogTitle,
  Empty, Input, confirmDialog,
} from '@ui'
import { t } from '../i18n'
import { bridge, errorMessage, notify, type StickyEntry } from './routing-types'
import { loadWorkspace } from './routing-state'
import { formatDuration, RouteIcon, stickyLabel, timeText } from './routing-ui'

export function RoutingStickyDialog({ model, onClose }: { model: string; onClose(): void }) {
  const [entries, setEntries] = React.useState<StickyEntry[]>([])
  const [mode, setMode] = React.useState('')
  const [now, setNow] = React.useState(Date.now())
  const [offset, setOffset] = React.useState(0)
  const [search, setSearch] = React.useState('')
  const [busy, setBusy] = React.useState(false)
  const [error, setError] = React.useState('')
  const load = React.useCallback(async () => {
    try {
      const result = await bridge().getRoutingSticky(model || undefined)
      setEntries(result.entries)
      setMode(result.sticky)
      setOffset(result.now - Date.now())
      setNow(result.now)
      setError('')
    } catch (error) { setError(errorMessage(error)) }
  }, [model])
  React.useEffect(() => { void load() }, [load])
  React.useEffect(() => {
    const timer = window.setInterval(() => { if (!document.hidden) setNow(Date.now() + offset) }, 1000)
    return () => window.clearInterval(timer)
  }, [offset])
  const active = entries.filter(entry => entry.expiresAt > now)
  const visible = active.filter(entry => `${entry.model} ${entry.accountName} ${entry.providerLabel} ${entry.source} ${entry.keyPreview}`.toLowerCase().includes(search.toLowerCase()))
  const clear = async (entry?: StickyEntry) => {
    if (busy) return
    if (!entry && !await confirmDialog(model
      ? t('Release all session bindings for this model. In-flight requests are not terminated; later requests are routed again.')
      : t('Release all session bindings for all models. In-flight requests are not terminated; later requests are routed again.'), {
      title: t('Clear session bindings'), confirmText: t('Release all'),
    })) return
    setBusy(true)
    try {
      const result = await bridge().clearRoutingSticky(entry ? { id: entry.id, model: entry.model } : model ? { model } : { all: true })
      setEntries(current => entry ? current.filter(item => item.id !== entry.id) : [])
      notify(t('Released {n} session bindings', { n: result.cleared }))
      await load()
      await loadWorkspace(true)
    } catch (error) { setError(errorMessage(error)) }
    finally { setBusy(false) }
  }
  return <Dialog open onOpenChange={open => { if (!open && !busy) onClose() }}><DialogContent className='w-[min(820px,calc(100vw-48px))]'>
    <DialogHeader><RouteIcon name='route' /><DialogTitle>{t('Session bindings')} · {model || t('All models')}</DialogTitle><Button variant='ghost' size='sm' disabled={busy} onClick={() => void load()}>{t('Refresh')}</Button></DialogHeader>
    <DialogBody>
      <div className='flex flex-wrap items-center gap-2'><Badge variant='brand' shape='tag'>{t('Stickiness: {mode}', { mode: stickyLabel(mode) })}</Badge><span className='text-xs text-muted-foreground'>{t('{n} active bindings', { n: active.length })}</span></div>
      <p className='text-xs text-muted-foreground'>{t('Releasing clears only the gateway\'s account binding, not chat history or the stickiness setting. Later successful requests can create new bindings. Restarting the gateway clears these process-local records.')}</p>
      {mode === 'off' || mode === 'turn' ? <p className='rounded-md bg-warning-soft p-3 text-xs text-warning'>{t('The current mode does not retain bindings across requests, so there are no active bindings.')}</p> : null}
      <Input placeholder={t('Search models, accounts, or session sources')} value={search} onChange={event => setSearch(event.currentTarget.value)} />
      <div className='routing-sticky-list'>{visible.map(entry => <div className='routing-sticky-row' key={entry.id}>
        <span className='routing-sticky-mark'><RouteIcon name='route' size={20} /></span><div className='routing-sticky-info'><b>{entry.accountName}</b><span>{entry.providerLabel} · {entry.model}</span><div><Badge variant='secondary' shape='tag'>{entry.source === 'content-hash' ? t('Content hash') : entry.source}</Badge><code>{entry.keyPreview}</code><span>{t('{n} hits', { n: entry.hits })}</span></div><span>{t('Last used: {time}', { time: timeText(entry.lastSeen) })}</span></div>
        <div className='routing-sticky-time'><b>{formatDuration(entry.expiresAt - now)}</b><span>{t('until expiry')}</span></div><Button variant='outline' size='sm' disabled={busy} onClick={() => void clear(entry)}>{t('Release')}</Button>
      </div>)}
        {!visible.length ? <Empty>{search ? t('No matching sessions') : t('No active session bindings in this view')}</Empty> : null}
      </div>
      {error ? <p role='alert' className='text-xs text-destructive'>{error}</p> : null}
    </DialogBody><DialogFooter><Button variant='destructive' className='mr-auto' disabled={busy || !active.length} onClick={() => void clear()}>{model ? t('Release this model\'s bindings') : t('Release all bindings')}</Button><Button disabled={busy} onClick={onClose}>{t('Done')}</Button></DialogFooter>
  </DialogContent></Dialog>
}
