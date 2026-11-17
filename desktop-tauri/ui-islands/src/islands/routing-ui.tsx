import type * as React from 'react'
import { Badge, Button, cn } from '@ui'
import { t } from '../i18n'
import { host } from './routing-types'

export function RouteIcon({ name, size = 16 }: { name: string; size?: number }) {
  const html = host().wbIcons?.icon(name, size)
  if (html) return <span className='ico inline-flex shrink-0 items-center justify-center leading-none [&>svg]:block'
    style={{ width: size, height: size }} aria-hidden='true' dangerouslySetInnerHTML={{ __html: html }} />
  const paths: Record<string, string> = {
    plus: 'M12 5v14M5 12h14',
    copy: 'M9 5V3h12v14h-3M3 7h12v14H3z',
    lock: 'M5 10h14v11H5zM8 10V6a4 4 0 0 1 8 0v4',
    more: 'M4 12h.01M12 12h.01M20 12h.01',
    search: 'M21 21l-5-5M18 10a8 8 0 1 1-16 0 8 8 0 0 1 16 0',
    check: 'M5 12l4 4L19 6',
    chevronLeft: 'm15 5-7 7 7 7',
    chevronRight: 'm9 5 7 7-7 7',
  }
  const path = paths[name]
  return path ? <svg width={size} height={size} viewBox='0 0 24 24' fill='none' stroke='currentColor' strokeWidth={name === 'more' ? 3.5 : 1.8} strokeLinecap='round' strokeLinejoin='round' aria-hidden='true'><path d={path} /></svg> : null
}
export function IconButton({ name, title, className, ...props }: {
  name: string; title: string
} & React.ComponentProps<typeof Button>) {
  return <Button variant='ghost' size='icon' className={cn('routing-icon-button', className)}
    title={title} aria-label={title} {...props}><RouteIcon name={name} size={14} /></Button>
}
export function RouteStateBadge({ state, children }: { state: string; children?: React.ReactNode }) {
  const label = state === 'usable' ? t('usable') : state === 'cooldown' ? t('Cooling down')
    : state === 'concurrency' ? t('Concurrency limit reached') : state === 'disabled' ? t('Disabled')
      : state === 'no-credentials' ? t('No credentials') : state === 'limiter' ? t('Skipped by limiter')
        : state === 'excluded' ? t('Not participating') : state === 'tried' ? t('Already attempted') : state === 'enabled' ? t('Enabled') : state === 'missing' ? t('Account removed') : t('Not ready')
  return <Badge shape='tag' variant={state === 'usable' ? 'success'
    : state === 'no-credentials' ? 'destructive'
      : state === 'cooldown' || state === 'concurrency' || state === 'limiter' ? 'warning' : 'secondary'}>
    {children || label}
  </Badge>
}
export function ProviderRouteBadge({ rank, effectiveRank, unavailable, disabled, states }: {
  rank?: number
  effectiveRank?: number
  unavailable: boolean
  disabled: boolean
  states: string[]
}) {
  if (unavailable || disabled) return <Badge shape='tag' variant='secondary'>{unavailable ? t('Model binding unavailable') : t('Not participating')}</Badge>
  if (rank !== undefined) return <Badge shape='tag' variant={effectiveRank === undefined ? 'secondary' : 'brand'} title={effectiveRank === undefined
    ? t('Priority {rank} among enabled providers; no candidate accounts are currently available.', { rank })
    : t('Priority {rank} among enabled providers; current available-candidate priority {effective}.', { rank, effective: effectiveRank })}>{t('Priority {n}', { n: rank })}</Badge>
  if (!states.length) return <Badge shape='tag' variant='secondary'>{t('No accounts participating in routing')}</Badge>
  const state = states[0]
  if (!states.every(value => value === state)) return <Badge shape='tag' variant='secondary'>{t('No available accounts')}</Badge>
  return <RouteStateBadge state={state}>{state === 'disabled' ? t('All accounts disabled') : undefined}</RouteStateBadge>
}
export function formatDuration(ms?: number | null): string {
  if (ms === undefined || ms === null) return '—'
  if (ms < 1000) return `${Math.round(ms)} ms`
  if (ms < 60_000) return `${(ms / 1000).toFixed(1)} s`
  return `${Math.floor(ms / 60_000)}:${String(Math.floor(ms / 1000) % 60).padStart(2, '0')}`
}
export function timeText(ts: number): string { return new Date(ts).toLocaleString() }
export function strategyLabel(strategy?: string): string {
  return strategy === 'rotate' || strategy === 'usage' ? t('Load balancing')
    : strategy === 'provider' ? t('Per-provider rotation') : t('Strict order')
}
export function stickyLabel(sticky?: string): string {
  return sticky === 'session' ? t('In-session') : sticky === 'off' ? t('Close')
    : sticky === 'turn' ? t('Per turn') : t('Auto')
}
