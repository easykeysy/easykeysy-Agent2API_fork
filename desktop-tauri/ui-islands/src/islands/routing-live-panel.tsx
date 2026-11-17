import * as React from 'react'
import { Badge, Button, Spinner, StatusDot } from '@ui'
import { t } from '../i18n'
import { formatDuration, RouteIcon, stickyLabel, strategyLabel } from './routing-ui'
import { RoutingStateText } from './routing-state-text'
import type { RouteEntry, RoutePreview } from './routing-types'

/** 服务端只读预演；紧凑摘要与完整候选共用同一结果。 */
export function RoutingLivePanel({ workspace, model, loading, error, dirty, globalScope, modelsAvailable, onOpenScope, onRefresh, preview = false, compact = false }: {
  workspace: Pick<RoutePreview, 'customized' | 'queue'> | null
  model: string
  preview?: boolean
  compact?: boolean
  loading: boolean
  error: string
  dirty: boolean
  globalScope: boolean
  modelsAvailable: boolean
  onOpenScope(model: string): void
  onRefresh(): void
}) {
  const [now, setNow] = React.useState(() => Date.now())
  React.useEffect(() => {
    const timer = window.setInterval(() => setNow(Date.now()), 1000)
    return () => window.clearInterval(timer)
  }, [])

  const customized = Boolean(workspace?.customized)
  const queue = workspace?.queue
  const available = queue?.ordered || []
  const blocked = queue?.blocked || []
  const next = available[0]
  const badge = preview ? { dot: 'warning' as const, text: t('Draft preview') } : dirty
    ? { dot: 'warning' as const, text: t('Using saved configuration') }
    : customized ? { dot: 'muted' as const, text: t('Independent configuration') }
      : { dot: 'success' as const, text: t('Inherit global') }

  const side = (entry: RouteEntry) => {
    const label = entry.providerLabel || entry.provider
    const reset = entry.resetAt && entry.resetAt > now
      ? t('Recovers in {time}', { time: formatDuration(entry.resetAt - now) })
      : ''
    return <div className='routing-live-side'><span>{label}</span>{entry.model ? <code>{entry.model}</code> : null}{reset ? <span className='routing-live-reset'>{reset}</span> : null}</div>
  }

  if (compact) return <div className='routing-preview-summary' aria-label={t('Draft preview')}>
    {globalScope && customized && workspace ? <div className='routing-live-banner'>
      <b>{t('{model} has its own configuration', { model })}</b><span>{t('Changes on the left apply to the global default and do not affect it.')}</span>
      <Button variant='link' size='sm' onClick={() => onOpenScope(model)}>{t('Open this model\'s configuration')}</Button>
    </div> : null}
    {loading ? <div className='routing-live-loading'><Spinner />{t('Calculating…')}</div>
      : error ? <p className='routing-live-error' role='alert'>{error}<Button variant='link' size='sm' onClick={onRefresh}>{t('Retry')}</Button></p>
        : workspace ? <>
          {available.slice(0, 3).map((entry, index) => <div className={`routing-preview-row${index === 0 ? ' is-first' : ''}`} key={entry.candidateId || `${entry.memberId || ''}:${entry.id}`}>
            <span className='routing-live-rank'>{String(index + 1).padStart(2, '0')}</span>
            <div className='routing-preview-row-main'><b title={entry.name}>{entry.name}</b><span title={entry.reason}>{entry.providerLabel || entry.provider}{entry.model ? ` · ${entry.model}` : ''}</span></div>
            <Badge shape='tag' variant={index === 0 ? 'brand' : 'ghost'}>{index === 0 ? t('Preview primary choice') : t('Candidates')}</Badge>
          </div>)}
          {!available.length ? <p className='routing-live-empty'>{t('No accounts available')}</p> : null}
          <p className='routing-preview-counts'>{t('{available} available · {blocked} skipped', { available: available.length, blocked: blocked.length })}</p>
        </> : <p className='routing-live-empty'>{modelsAvailable ? t('Select a preview model to view the queue') : t('No models available yet. Add an account on the Accounts page first.')}</p>}
  </div>

  return <aside className='routing-live-panel' aria-label={t('Current queue')}>
    <div className='routing-live-head'>
      <b>{t('Current queue')}</b>
      <span className='routing-live-badge'><StatusDot variant={badge.dot} />{badge.text}</span>
      <span className='routing-live-model'>{model}</span>
      <Button variant='ghost' size='sm' className='routing-live-refresh' onClick={onRefresh} title={t('Refresh')} aria-label={t('Refresh')}>
        <RouteIcon name='refresh' size={13} />
      </Button>
    </div>

    {globalScope && customized && workspace ? <div className='routing-live-banner'>
      <b>{t('{model} has its own configuration', { model })}</b>
      <span>{t('Changes on the left apply to the global default and do not affect it.')}</span>
      <Button variant='outline' size='sm' onClick={() => onOpenScope(model)}>{t('Open this model\'s configuration')}</Button>
    </div> : null}

    {loading && !workspace ? <div className='routing-live-loading'><Spinner /></div> : null}
    {error ? <p className='routing-live-error' role='alert'>{error}</p> : null}
    {!loading && !error && !workspace ? <div className='routing-live-empty'>
      {modelsAvailable ? t('Select a preview model to view the queue') : t('No models available yet. Add an account on the Accounts page first.')}
    </div> : null}

    {workspace ? <>
      <div className='routing-live-summary'>
        {next ? <>
          <div className='routing-live-eyebrow'>{preview ? t('Preview primary choice') : t('Current available queue head')}</div>
          <div className='routing-live-name'>{next.model || next.name}</div>
          {next.model ? <p className='routing-live-reason'>{next.providerLabel || next.provider} · {next.name}</p> : null}
          <p className='routing-live-reason'>{next.reason}</p>
        </> : <>
          <div className='routing-live-eyebrow'>{t('Next request')}</div>
          <div className='routing-live-name is-empty'>{t('No accounts available')}</div>
          <p className='routing-live-reason'>{t('Resolve credentials, limits, or scope first.')}</p>
        </>}
        <div className='routing-live-counts'>
          <span>{t('{available} available · {blocked} skipped', { available: available.length, blocked: blocked.length })}</span>
          <span>{strategyLabel(queue?.strategy)} · {t('Affinity {mode}', { mode: stickyLabel(queue?.sticky) })}</span>
        </div>
      </div>

      {available.length ? <>
        <div className='routing-live-subhead'><span>{t('Available candidates')}</span><span>{available.length}</span></div>
        <div className='routing-live-list'>{available.map((entry, index) => <div className='routing-live-row' key={entry.candidateId || `${entry.memberId || ''}:${entry.id}`}>
          <span className='routing-live-rank'>{index + 1}</span>
          <div className='routing-live-main'>
            <div className='routing-live-title'><b>{entry.name}</b><RoutingStateText state={entry.state} /></div>
            <p className='routing-live-row-reason'>{entry.reason}</p>
          </div>
          {side(entry)}
        </div>)}</div>
      </> : null}

      {blocked.length ? <>
        <div className='routing-live-subhead'><span>{t('Skipped')}</span><span>{blocked.length}</span></div>
        <div className='routing-live-list'>{blocked.map(entry => <div className='routing-live-row is-blocked' key={entry.candidateId || `${entry.memberId || ''}:${entry.id}`}>
          <span className='routing-live-rank'>–</span>
          <div className='routing-live-main'>
            <div className='routing-live-title'><b>{entry.name}</b><RoutingStateText state={entry.state} /></div>
            <p className='routing-live-row-reason'>{entry.reason}</p>
          </div>
          {side(entry)}
        </div>)}</div>
      </> : null}

      <div className='routing-live-foot'>
        {dirty ? <p>{t('There are unsaved changes; the preview still uses the saved configuration.')}</p> : null}
        <p>{t('Read-only preview; it does not advance rotation. Only saving affects real requests.')}</p>
        <p>{t('Real requests also consider current session bindings, concurrency and quota.')}</p>
      </div>
    </> : null}
  </aside>
}
