import * as React from 'react'
import { Alert, Button, Input, Select, SelectContent, SelectItem, SelectTrigger, SelectValue, Switch } from '../ui-kit/src'
import { useGithubClock } from './github-failure'
import {
  DEFAULT_GITHUB_MIRRORS, githubMirrors, githubSourceLabel,
  type GithubAccessPatch, type GithubAccessSettings, type GithubMirror, type GithubSourceObservation,
} from './github-access'

type Translate = (key: string, params?: Record<string, string | number>) => string

type Props = {
  load: () => Promise<GithubAccessSettings | null | undefined>
  persist: (patch: GithubAccessPatch) => Promise<GithubAccessSettings | null | undefined>
  notify: (message: string, failed: boolean) => void
  t: Translate
  mobile?: boolean
}

export function GithubAccessEditor({ load, persist, notify, t, mobile = false }: Props) {
  const [settings, setSettings] = React.useState<GithubAccessSettings | null>(null)
  const [draft, setDraft] = React.useState('')
  const [busy, setBusy] = React.useState(false)
  const [error, setError] = React.useState('')
  const saving = React.useRef(false)
  const revision = React.useRef(0)
  const modeOptions = [
    { value: 'auto', label: t('Automatically select the fastest available source') },
    { value: 'official', label: t('Official source only') },
    { value: 'accelerated', label: t('Selected accelerator first') },
  ] as const

  React.useEffect(() => {
    let alive = true
    async function refresh(initial: boolean): Promise<void> {
      if (saving.current || (!initial && document.visibilityState === 'hidden')) return
      const generation = revision.current
      try {
        const value = await load()
        if (!value) throw new Error(t('Request failed'))
        if (alive && !saving.current && revision.current === generation) setSettings(value)
      } catch (failure) {
        if (alive && initial) setError(failure instanceof Error ? failure.message : String(failure))
      }
    }
    void refresh(true)
    const timer = window.setInterval(() => void refresh(false), 5_000)
    return () => { alive = false; window.clearInterval(timer) }
  }, [load, t])

  async function save(patch: GithubAccessPatch): Promise<boolean> {
    if (saving.current) return false
    saving.current = true
    revision.current += 1
    setBusy(true)
    setError('')
    try {
      const value = await persist(patch)
      if (!value) throw new Error(t('Request failed'))
      setSettings(value)
      notify(value.saved === false
        ? t('Settings applied, but could not be saved to disk (will revert after restart)') : t('GitHub access settings saved'), value.saved === false)
      return true
    } catch (failure) {
      setError(failure instanceof Error ? failure.message : String(failure))
      return false
    } finally {
      saving.current = false
      setBusy(false)
    }
  }

  const mirrors = settings ? githubMirrors(settings) : []
  const modern = Array.isArray(settings?.mirrors)
  const disabled = busy || !settings || settings.mode === 'official'
  const enabled = mirrors.filter(mirror => mirror.enabled)
  const missingDefaults = DEFAULT_GITHUB_MIRRORS.filter(defaultMirror => !mirrors.some(mirror => mirror.url === defaultMirror.url))

  function updateMirror(url: string, patch: Partial<GithubMirror>): void {
    void save({ mirrors: mirrors.map(mirror => mirror.url === url ? { ...mirror, ...patch } : mirror) })
  }

  async function addMirror(): Promise<void> {
    const value = draft.trim()
    if (!value) return
    if (await save(modern ? { mirrors: [...mirrors, { url: value, enabled: true, supportsApi: true }] } : { mirrorUrl: value })) {
      setDraft('')
    }
  }

  return (
    <div className='flex flex-col gap-3 text-[13px] leading-relaxed'>
      <div className='font-semibold text-foreground'>{t('GitHub access mode')}</div>
      <Select value={settings?.mode || 'auto'} disabled={busy || !settings}
        onValueChange={value => { if (value) void save({ mode: value as GithubAccessSettings['mode'] }) }}>
        <SelectTrigger className='w-full' aria-label={t('GitHub access mode')}>
          <SelectValue>{modeOptions.find(option => option.value === settings?.mode)?.label || modeOptions[0].label}</SelectValue>
        </SelectTrigger>
        <SelectContent>
          {modeOptions.map(option => <SelectItem key={option.value} value={option.value}>{option.label}</SelectItem>)}
        </SelectContent>
      </Select>
      <p className='text-subtle'>
        {settings?.mode === 'official' ? t('Uses only the official GitHub source, through the selected outbound proxy.')
          : settings?.mode === 'accelerated' ? t('Try the selected accelerator first, then other enabled accelerators, and finally the official source.')
            : t('Check release queries, plugin indexes and downloads separately. Use the first valid response, briefly reuse successful routes and switch on failure.')}
      </p>
      {settings?.mode === 'accelerated' && enabled.length > 0 ? (
        <Select value={settings.mirrorUrl} disabled={busy}
          onValueChange={value => { if (value) void save({ mirrorUrl: value }) }}>
          <SelectTrigger className='w-full' aria-label={t('Preferred accelerator')}>
            <SelectValue>{githubSourceLabel(enabled.find(mirror => mirror.url === settings.mirrorUrl)?.url || enabled[0].url)}</SelectValue>
          </SelectTrigger>
          <SelectContent>
            {enabled.map(mirror => <SelectItem key={mirror.url} value={mirror.url}>{githubSourceLabel(mirror.url)}</SelectItem>)}
          </SelectContent>
        </Select>
      ) : null}
      <div className='rounded-md border border-border p-3'>
        <div className='font-medium text-foreground'>{t('Official source')}</div>
        <SourceStatus source='official' observations={settings?.observations || []} serverTime={settings?.serverTime} t={t} />
      </div>
      <div className='flex flex-col gap-2'>
        {mirrors.map(mirror => {
          const builtin = DEFAULT_GITHUB_MIRRORS.some(item => item.url === mirror.url)
          return (
            <div key={mirror.url} className='rounded-md border border-border p-3'>
              <div className='flex items-center justify-between gap-3'>
                <span className='min-w-0 break-all font-medium text-foreground'>{githubSourceLabel(mirror.url)}</span>
                {modern ? <Switch checked={mirror.enabled} disabled={disabled}
                  aria-label={t('Enable accelerator {source}', { source: githubSourceLabel(mirror.url) })}
                  onCheckedChange={enabled => updateMirror(mirror.url, { enabled })} /> : null}
              </div>
              <p className='mt-1 text-xs text-subtle'>
                {mirror.supportsApi ? t('Release queries, plugin indexes and downloads') : t('Plugin indexes and downloads only; release queries are not supported')}
              </p>
              {mirror.url === 'https://cdn.gh-proxy.org/' ? <p className='mt-1 text-xs text-subtle'>{t('An alternative route from the same service as gh-proxy.com.')}</p> : null}
              {modern && !builtin ? (
                <div className='mt-2 flex items-center justify-between gap-2'>
                  <label className='inline-flex items-center gap-2 text-xs text-subtle'>
                    <Switch checked={mirror.supportsApi} disabled={disabled || /https:\/\/(ghproxy\.net|ghfast\.top)\//.test(mirror.url)}
                      aria-label={t('Supports release queries')} onCheckedChange={supportsApi => updateMirror(mirror.url, { supportsApi })} />
                    {t('Supports release queries')}
                  </label>
                  <Button size='sm' variant='ghost' disabled={disabled}
                    onClick={() => void save({ mirrors: mirrors.filter(item => item.url !== mirror.url) })}>{t('Remove')}</Button>
                </div>
              ) : null}
              <SourceStatus source={mirror.url} observations={settings?.observations || []} serverTime={settings?.serverTime} t={t} />
            </div>
          )
        })}
      </div>
      {settings && settings.mode !== 'official' && !enabled.length ? <p className='text-subtle'>{t('No accelerators are enabled. Only the official source will be used.')}</p> : null}
      <div className='flex items-center gap-2'>
        <Input value={draft} disabled={disabled || (modern && mirrors.length >= 6)} className='min-w-0 flex-1'
          aria-label={t('Custom accelerator prefix')} placeholder='https://example.com/' autoComplete='off' spellCheck={false}
          onChange={event => setDraft(event.currentTarget.value)}
          onKeyDown={event => { if (event.key === 'Enter') { event.preventDefault(); void addMirror() } }} />
        <Button size='sm' variant='outline' disabled={disabled || !draft.trim() || (modern && mirrors.length >= 6)}
          onClick={() => void addMirror()}>{modern ? t('Add') : t('Save')}</Button>
      </div>
      {modern && missingDefaults.length > 0 ? <Button className='self-start' size='sm' variant='ghost'
        disabled={disabled || mirrors.length + missingDefaults.length > 6}
        onClick={() => void save({ mirrors: [...mirrors, ...missingDefaults] })}>{t('Add missing built-in sources')}</Button> : null}
      <p className='text-xs text-subtle'>{t('Keep up to 6 HTTPS accelerators. Release queries can be disabled for custom sources.')}</p>
      <p className='text-xs text-subtle'>{t('These are results from the latest real requests, not live speed tests. Expired results need a new check. Rate limits pause only the affected route; other routes remain available.')}</p>
      <p className='text-xs text-subtle'>{t('Software updates and plugins share this setting. GitHub tokens go only to the official API. Mirrored installers require an official checksum; mirrored plugins require a trusted signature.')}</p>
      {mobile ? <p className='text-xs text-subtle'>{t('These results are from the gateway. APK downloads select a route using the phone’s own network.')}</p> : null}
      {!modern && settings ? <p className='text-xs text-subtle'>{t('This gateway still uses legacy single-source settings. Upgrade it to manage multiple accelerators.')}</p> : null}
      {error ? <Alert variant='destructive' role='alert' className='text-xs'>{error}</Alert> : null}
    </div>
  )
}

function SourceStatus({ source, observations, serverTime, t }: { source: string; observations: GithubSourceObservation[]; serverTime?: number; t: Translate }) {
  const now = useGithubClock(serverTime)
  const items = observations.filter(item => item.source === source)
  if (!items.length) return <p className='mt-2 text-xs text-subtle'>{t('Not checked')}</p>
  const states: Record<string, string> = {
    ok: t('usable'), failed: t('Connection failed'), timeout: t('Connection timed out'), invalid: t('Invalid response content'),
    unsupported: t('Request type not supported'), rateLimited: t('Waiting for rate limit reset'), stale: t('Needs a new check'),
  }
  const resources = { api: t('Release query'), index: t('Plugin index'), download: t('File download') }
  return <div className='mt-2 flex flex-col gap-1 text-xs text-subtle'>
    {items.map(item => {
      const cooling = Boolean(item.retryAt && item.retryAt > now)
      const stale = !cooling && (item.state === 'stale' || item.state === 'rateLimited' || Boolean(item.validUntil && item.validUntil <= now))
      const state = cooling ? 'rateLimited' : stale ? 'stale' : item.state
      const previous = item.lastState || item.state
      return <div key={item.resource}>
        <span>{resources[item.resource]} · {states[state] || t('Connection failed')}</span>
        {cooling ? <>
          <span> · {t('Retry in {seconds} seconds', { seconds: Math.ceil((item.retryAt! - now) / 1000) })}</span>
          <span> · {item.retrySource === 'fallback' ? t('No reset time provided; briefly waiting as GitHub recommends')
            : item.retrySource === 'legacy' ? t('Using the previously recorded reset time') : t('Using the upstream reset time')}</span>
        </> : null}
        {stale && previous !== 'stale' ? <span> · {t('Previous result: {state}', { state: states[previous] || previous })}</span> : null}
        {state === 'ok' && item.elapsedMs != null ? <span> · {t('{ms} ms', { ms: item.elapsedMs })}</span> : null}
        {item.selected && state === 'ok' ? <span> · {t('Last used')}</span> : null}
        {item.checkedAt > 0 ? <span> · {t('Recorded at {time}', { time: new Date(item.checkedAt).toLocaleTimeString() })}</span> : null}
      </div>
    })}
  </div>
}
