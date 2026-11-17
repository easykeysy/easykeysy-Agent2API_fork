import * as React from 'react'
import { Alert, Button, Spinner } from '@ui'
import type { ScriptCatalogInfo, ScriptPluginsState, ScriptRecommendation } from './plugins'
import type { PluginTranslate } from './plugins-store'

export type PluginOpenExternal = (url: string) => void | Promise<void>

export function ScriptRepository({ item, onOpenExternal, t }: {
  item: ScriptCatalogInfo
  onOpenExternal?: PluginOpenExternal
  t: PluginTranslate
}) {
  const repository = item.repository?.trim()
  if (!repository || !/^[a-z\d_-][a-z\d._-]*\/[a-z\d_-][a-z\d._-]*$/i.test(repository)) return null
  const url = `https://github.com/${repository}`
  const stars = typeof item.stars === 'number' && Number.isFinite(item.stars) && item.stars >= 0 ? item.stars : null
  const updated = item.starsAt ? t('Stars updated {time}', { time: new Date(item.starsAt).toLocaleString() }) : undefined
  return <div className='flex min-w-0 flex-wrap items-center gap-x-3 gap-y-1 text-[11.5px] text-muted-foreground'>
    <Button variant='link' size='xs' nativeButton={false} className='h-auto min-w-0 max-w-full shrink px-0 text-[11.5px]'
      render={<a href={url} target='_blank' rel='noopener noreferrer' />}
      title={url} aria-label={t('View GitHub repository: {repository}', { repository })}
      onClick={event => {
        if (!onOpenExternal || event.ctrlKey || event.metaKey || event.shiftKey || event.altKey) return
        event.preventDefault()
        void onOpenExternal(url)
      }}>
      <span className='shrink-0'>GitHub</span><span className='truncate'>{repository}</span>
    </Button>
    <span className='inline-flex shrink-0 items-center gap-1 tabular-nums' title={updated}
      aria-label={stars === null ? t('Star count is not available yet') : t('{count} stars', { count: stars.toLocaleString() })}>
      <svg viewBox='0 0 24 24' className='size-3.5' fill='none' stroke='currentColor' strokeWidth='1.6'
        strokeLinejoin='round' aria-hidden='true'><path d='m12 3 2.8 5.8 6.4.9-4.6 4.5 1.1 6.4L12 17.6l-5.7 3 1.1-6.4-4.6-4.5 6.4-.9Z' /></svg>
      {stars === null ? '—' : stars.toLocaleString()}
    </span>
  </div>
}

export function scriptUnavailableReason(item: ScriptRecommendation, t: PluginTranslate): string {
  switch (item.unavailableReason) {
    case 'This is not a provider plugin and cannot be installed here.': return t('This is not a provider plugin and cannot be installed here.')
    case 'This repository has no committed loadable entry point. Wait for the author to publish a build.': return t('This repository has no committed loadable entry point. Wait for the author to publish a build.')
    case 'The plugin has no valid version declared and cannot be installed yet.': return t('The plugin has no valid version declared and cannot be installed yet.')
    case 'The plugin does not have a downloadable release yet.': return t('The plugin does not have a downloadable release yet.')
    case 'Could not read the plugin release information. Refresh later.': return t('Could not read the plugin release information. Refresh later.')
    default: return item.unavailableReason || t('Installation is currently unavailable')
  }
}

export function ScriptCatalogStatus({ catalog, request, refresh, disabled, t }: {
  catalog: NonNullable<ScriptPluginsState['catalog']>
  request: (path: string, body: Record<string, unknown>) => Promise<unknown>
  refresh: () => Promise<void>
  disabled?: boolean
  t: PluginTranslate
}) {
  const [busy, setBusy] = React.useState(false)
  const [error, setError] = React.useState('')
  const pending = React.useRef(false)
  const refreshing = busy || catalog.refreshing
  const checkedAt = Math.max(catalog.marketCheckedAt, catalog.githubCheckedAt)
  const now = Date.now()
  const retryAt = [catalog.marketRetryAt, catalog.githubRetryAt].filter((value): value is number => Boolean(value && value > now))
  const problems = [
    catalog.marketError ? `${t('Magpie official catalog')}：${catalog.marketError}` : '',
    catalog.githubError ? `GitHub：${catalog.githubError}` : '',
    error,
  ].filter(Boolean)

  async function reload(): Promise<void> {
    if (pending.current || refreshing || disabled) return
    pending.current = true
    setBusy(true); setError('')
    try {
      await request('/api/plugins/scripts/catalog/refresh', {})
      await refresh()
    } catch (failure) {
      setError(failure instanceof Error ? failure.message : String(failure))
    } finally { pending.current = false; setBusy(false) }
  }

  return <div className='mb-3 space-y-2'>
    <div className='flex flex-wrap items-center justify-between gap-x-3 gap-y-2 text-[11.5px] text-muted-foreground'>
      <p className='m-0 min-w-0 flex-1 leading-relaxed'>{t('Official and recommended plugins come from Magpie\'s official catalog. Third-party plugins are discovered through GitHub\'s magpie-plugin topic and are not officially recommended.')}</p>
      <div className='flex flex-wrap items-center gap-2.5'>
        <span className='inline-flex items-center gap-1.5' role='status'>
          {refreshing ? <><Spinner className='size-3' />{catalog.total > 0
            ? t('Reading plugin information {done}/{total}', { done: catalog.completed, total: catalog.total })
            : t('Loading plugin catalog…')}</>
            : checkedAt ? t('Catalog updated {time}', { time: new Date(checkedAt).toLocaleString() })
              : t('Plugin catalog not loaded yet')}
        </span>
        <Button variant='ghost' size='xs' disabled={disabled || refreshing} onClick={() => void reload()}>{t('Refresh catalog')}</Button>
      </div>
    </div>
    {problems.length ? <Alert variant='warning' className='text-xs'>
      <p className='m-0'>{t('Some information could not be updated. Previously loaded plugins remain available to browse.')}</p>
      {problems.map((problem, index) => <p key={index} className='m-0 mt-1 whitespace-pre-wrap break-words'>{problem}</p>)}
      {retryAt.length ? <p className='m-0 mt-1'>{t('Affected sources will retry automatically after {time}', { time: new Date(Math.min(...retryAt)).toLocaleTimeString() })}</p> : null}
    </Alert> : null}
  </div>
}
