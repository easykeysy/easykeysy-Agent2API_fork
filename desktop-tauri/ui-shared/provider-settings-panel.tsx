import * as React from 'react'
import { Alert, Button, Card, SegmentedControl } from '@ui'
import { ProviderSettingsCard } from './provider-settings-card'
import {
  matchesProviderSettings, providerSettingsKey, providerSettingsLabel,
  type ProviderSettingsRequest, type ProviderSettingsRow, type ProviderSettingsTranslate, type ProviderSettingValue,
} from './provider-settings'

export function ProviderSettingsPanel({ request, t, locale, active = true, provider = '', navigation = 0, onPlugins }: {
  request: ProviderSettingsRequest
  t: ProviderSettingsTranslate
  locale: string
  active?: boolean
  provider?: string
  navigation?: number
  onPlugins: () => void
}) {
  const [rows, setRows] = React.useState<ProviderSettingsRow[] | null>(null)
  const [error, setError] = React.useState('')
  const [loading, setLoading] = React.useState(false)
  const [filter, setFilter] = React.useState(provider)
  const alive = React.useRef(true)
  const generation = React.useRef(0)
  const pending = React.useRef<Promise<void> | null>(null)
  const dirty = React.useRef(new Set<string>())
  const latest = React.useRef({ request, t, active })
  const endpoint = React.useRef('/api/provider-settings')
  latest.current = { request, t, active }
  const [dirtyCount, setDirtyCount] = React.useState(0)

  React.useEffect(() => { setFilter(provider) }, [provider, navigation])
  React.useEffect(() => {
    alive.current = true
    return () => { alive.current = false; generation.current++ }
  }, [])

  const load = React.useCallback((): Promise<void> => {
    if (pending.current) return pending.current
    if (!alive.current) return Promise.resolve()
    const current = ++generation.current
    const t = latest.current.t
    setLoading(true)
    const task = (async () => {
      try {
        let result: { settings: ProviderSettingsRow[] }
        try {
          result = await latest.current.request('GET', endpoint.current)
        } catch (cause) {
          // 新手机端可连接尚未升级的网关；只对明确的 404 回退，不重试保存请求。
          const status = cause && typeof cause === 'object' && 'status' in cause ? cause.status : undefined
          if (endpoint.current !== '/api/provider-settings' || status !== 404) throw cause
          result = await latest.current.request('GET', '/api/plugins/provider-settings')
          endpoint.current = '/api/plugins/provider-settings'
        }
        if (!Array.isArray(result?.settings)) throw new Error(t('The provider settings response is invalid. Refresh and try again.'))
        if (!alive.current || current !== generation.current) return
        setRows(previous => {
          const next = [...result.settings]
          for (const row of previous || []) {
            if (dirty.current.has(providerSettingsKey(row)) && !next.some(item => providerSettingsKey(item) === providerSettingsKey(row))) {
              next.push({ ...row, error: t('The provider is unavailable. Unsaved edits are kept; restore the plugin to continue editing.') })
            }
          }
          return next
        })
        setError('')
      } catch (cause) {
        if (alive.current && current === generation.current) setError(cause instanceof Error ? cause.message : String(cause))
      } finally {
        if (alive.current && current === generation.current) setLoading(false)
      }
    })()
    pending.current = task
    void task.finally(() => { if (pending.current === task) pending.current = null })
    return task
  }, [])

  React.useEffect(() => {
    if (!active) return
    const refresh = () => { if (latest.current.active && !document.hidden) void load() }
    refresh()
    const timer = window.setInterval(refresh, 30_000)
    window.addEventListener('wb:settings-load', refresh)
    document.addEventListener('visibilitychange', refresh)
    return () => {
      window.clearInterval(timer)
      window.removeEventListener('wb:settings-load', refresh)
      document.removeEventListener('visibilitychange', refresh)
    }
  }, [active, load])

  React.useEffect(() => {
    if (!dirtyCount) return
    const warn = (event: BeforeUnloadEvent) => { event.preventDefault(); event.returnValue = '' }
    window.addEventListener('beforeunload', warn)
    return () => window.removeEventListener('beforeunload', warn)
  }, [dirtyCount])

  function markDirty(key: string, changed: boolean) {
    if (changed) dirty.current.add(key)
    else dirty.current.delete(key)
    setDirtyCount(dirty.current.size)
  }

  async function saveValues(row: ProviderSettingsRow, values: Record<string, ProviderSettingValue>): Promise<ProviderSettingsRow> {
    const result = await latest.current.request<ProviderSettingsRow>('PATCH', endpoint.current, {
      provider: row.provider, revision: row.revision, values,
    })
    if (alive.current) {
      generation.current++
      pending.current = null
      setLoading(false)
      setRows(previous => previous?.map(item => providerSettingsKey(item) === providerSettingsKey(row)
        ? { ...result, label: item.label, group: result.group || item.group } : item) ?? null)
    }
    return result
  }

  const selected = filter ? rows?.find(row => matchesProviderSettings(row, filter)) : undefined
  const selectedKey = selected ? providerSettingsKey(selected) : filter
  const options = [{ value: '', label: t('All') }, ...(rows || []).map(row => ({
    value: providerSettingsKey(row), label: providerSettingsLabel(row, locale, t),
  }))]
  if (filter && !selected) options.push({ value: filter, label: t('Selected provider') })
  const empty = rows !== null && (!rows.length || Boolean(filter && !selected))

  return <section className='flex min-w-0 flex-col gap-4' aria-label={t('Provider settings')}>
    <div className='flex flex-wrap items-start justify-between gap-3'>
      <div className='min-w-0'>
        <h2 className='text-base font-semibold'>{t('Provider settings')}</h2>
        <p className='mt-1.5 max-w-[72ch] text-xs leading-relaxed text-muted-foreground'>
          {t('Manage provider-specific options. Changes apply to all accounts for that provider on the current gateway.')}
        </p>
      </div>
      <Button type='button' size='sm' variant='outline' disabled={loading} onClick={() => void load()}>
        {loading ? t('Refreshing…') : t('Refresh')}
      </Button>
    </div>
    <div className='min-w-0 max-w-full overflow-x-auto pb-1'>
      <SegmentedControl value={selectedKey} options={options} onValueChange={setFilter} aria-label={t('Filter settings by provider')}
        className='w-max [&_[data-slot=segmented-item]]:h-8' />
    </div>
    {error ? <Alert variant='destructive'>{error}</Alert> : null}
    {rows === null && !error ? <Card className='px-4 py-8 text-center text-sm text-muted-foreground' role='status'>
      {t('Loading provider settings…')}
    </Card> : null}
    {empty && !error ? <Card className='flex flex-col items-start gap-3 p-5'>
      <p className='text-sm font-medium'>{filter ? t('This provider has no editable settings') : t('No editable provider settings')}</p>
      <p className='text-xs leading-relaxed text-muted-foreground'>
        {t('Only providers with an installed, enabled plugin that declares settings appear here. Check installation and activation on the Plugins page.')}
      </p>
      <div className='flex flex-wrap gap-2'>
        {filter ? <Button type='button' size='sm' variant='outline' onClick={() => setFilter('')}>{t('View all')}</Button> : null}
        <Button type='button' size='sm' variant='outline' onClick={onPlugins}>{t('Manage plugins')}</Button>
      </div>
    </Card> : null}
    {(rows || []).map(row => {
      const key = providerSettingsKey(row)
      return <div key={key} hidden={Boolean(filter) && key !== selectedKey}>
        <ProviderSettingsCard row={row} t={t} locale={locale} saveValues={saveValues} refresh={load}
          onDirtyChange={changed => markDirty(key, changed)} />
      </div>
    })}
    {rows?.length ? <p className='text-xs leading-relaxed text-muted-foreground'>
      {t('Options and defaults come from plugins. Plugin updates do not overwrite your saved changes. Switching providers keeps unsaved edits.')}
    </p> : null}
  </section>
}
