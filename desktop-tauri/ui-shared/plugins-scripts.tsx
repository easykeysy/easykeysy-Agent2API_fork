import * as React from 'react'
import {
  Alert, Badge, Button, Card, Checkbox, Dialog, DialogBody, DialogContent, DialogDescription,
  DialogFooter, DialogHeader, DialogTitle, Empty, EmptyDescription, EmptyTitle, Input, Label, SegmentedControl, Switch,
} from '@ui'
import type { ScriptOperation, ScriptPackage, ScriptPluginsState, ScriptRecommendation } from './plugins'
import { matchesPluginStatus, matchesScriptPublisher, scriptPublisher, type PluginStatusFilter, type ScriptPublisherFilter } from './plugins-filters'
import { ScriptOperationCard, ScriptOperationProgress, scriptOperationMatches } from './plugins-script-progress'
import { PluginGetButton, PluginSectionHeading, PluginSettingsButton, PluginStoreCard, type PluginTranslate } from './plugins-store'
import { ScriptPluginSettings } from './plugins-script-settings'
import { ScriptPluginInstallNotice } from './plugins-script-install-notice'
import { cachedPluginIconPath } from './provider-icons'
import { getPluginStartup, pluginFailed, pluginPending, refreshPluginStartup, startupForPack, subscribePluginStartup } from './plugin-startup'
import { requestPluginDiagnostics } from './plugin-startup-view'
import { isRecommendationInstalled, matchesRecommendedPlugin, matchesScriptPlugin, recommendationKey, recommendedDescription, reinstallRequest, scriptOfficialBadge, scriptPluginDescription, scriptPluginName, scriptRecommendations, scriptSourceLabel, scriptUpdate, updateLabel } from './plugins-script-display'
import { ScriptCatalogStatus, ScriptRepository, scriptUnavailableReason, type PluginOpenExternal } from './plugins-script-catalog'
import { ScriptPaginationFooter, useScriptPagination } from './plugins-script-pagination'

type Install = { name: string; version: string; id?: string }
type GalleryItem = { kind: 'installed'; value: ScriptPackage } | { kind: 'recommended'; value: ScriptRecommendation }

export function ScriptPluginsPanel({ state, request, refresh, t, disabled = false, status = 'all',
  publisher: controlledPublisher, onPublisherChange, onClearFilters,
  query = '', galleryVisible = true, showInstallAction = true, installRequest = 0, onBusyChange,
  resolveIcon = path => path, onOpenExternal }: {
  state: ScriptPluginsState
  request: (path: string, body: Record<string, unknown>) => Promise<unknown>
  refresh: () => Promise<void>
  t: PluginTranslate
  disabled?: boolean
  status?: PluginStatusFilter
  publisher?: ScriptPublisherFilter
  onPublisherChange?: (publisher: ScriptPublisherFilter) => void
  onClearFilters?: () => void
  query?: string
  galleryVisible?: boolean
  showInstallAction?: boolean
  installRequest?: number
  onBusyChange?: (busy: boolean) => void
  resolveIcon?: (path: string) => string
  onOpenExternal?: PluginOpenExternal
}) {
  React.useSyncExternalStore(subscribePluginStartup, getPluginStartup)
  const [localPublisher, setLocalPublisher] = React.useState<ScriptPublisherFilter>('all')
  const publisher = controlledPublisher ?? localPublisher
  const changePublisher = (value: ScriptPublisherFilter) => {
    setLocalPublisher(value)
    onPublisherChange?.(value)
  }
  const [install, setInstall] = React.useState<Install | null>(null)
  const [installNotice, setInstallNotice] = React.useState<Install | null>(null)
  const [remove, setRemove] = React.useState<ScriptPackage | null>(null)
  const [settingsId, setSettingsId] = React.useState<string | null>(null)
  const [trusted, setTrusted] = React.useState(false)
  const [busy, setBusy] = React.useState(false)
  const [error, setError] = React.useState('')
  const [operationTarget, setOperationTarget] = React.useState<ScriptOperation['target']>()
  const pending = React.useRef(false)
  const lastInstallRequest = React.useRef(0)
  const prefix = React.useId()
  const locked = disabled || busy || state.operation.active
  const recommended = scriptRecommendations(state)
  const github = /^(github:|https:\/\/github\.com\/)/i.test(install?.name.trim() || '')
  const magpie = install?.name.trim().startsWith('magpie:') === true
  const settingsPack = state.packages.find(pack => pack.id === settingsId)
  const operation: ScriptOperation = state.operation.target ? state.operation : { ...state.operation, target: operationTarget }
  const matchingPackages = state.packages.filter(pack => matchesPluginStatus({ installed: true, updatable: scriptUpdate(state, pack.id) !== undefined }, status)
    && matchesScriptPlugin(pack, recommended, query, t))
  const matchingRecommendations = recommended.filter(item => !isRecommendationInstalled(item, state.packages)
    && matchesPluginStatus({ installed: false, updatable: false }, status) && matchesRecommendedPlugin(item, query, t))
  const publisherItems = [...matchingPackages, ...matchingRecommendations]
  const publisherCount = (value: ScriptPublisherFilter) => publisherItems.filter(item => matchesScriptPublisher(item, value)).length
  const packages = state.packages.filter(pack => scriptOperationMatches(operation, pack)
    || (galleryVisible && matchingPackages.includes(pack) && matchesScriptPublisher(pack, publisher)))
  const recommendations = recommended.filter(item => !isRecommendationInstalled(item, state.packages)
    && (scriptOperationMatches(operation, item)
      || (galleryVisible && matchingRecommendations.includes(item) && matchesScriptPublisher(item, publisher))))
  const gallery: GalleryItem[] = [
    ...packages.map(value => ({ kind: 'installed' as const, value })),
    ...recommendations.map(value => ({ kind: 'recommended' as const, value })),
  ]
  const publisherRank = { official: 0, recommended: 1, 'third-party': 2 }
  gallery.sort((left, right) => publisherRank[scriptPublisher(left.value)] - publisherRank[scriptPublisher(right.value)]
    || (right.value.stars ?? -1) - (left.value.stars ?? -1))
  const count = gallery.length
  const page = useScriptPagination(count, JSON.stringify([publisher, status, query.trim().toLocaleLowerCase(), galleryVisible]), galleryVisible)
  const visibleGallery = gallery.filter((entry, index) => index < page.visible || scriptOperationMatches(operation, entry.value))
  const catalogPending = Boolean(state.catalog && (state.catalog.refreshing
    || (!state.catalog.marketCheckedAt && !state.catalog.marketError)
    || (!state.catalog.githubCheckedAt && !state.catalog.githubError)))
  const catalogFailed = Boolean(state.catalog?.marketError || state.catalog?.githubError)
  const unmatchedOperation = operation.active && !gallery.some(entry => scriptOperationMatches(operation, entry.value))
  const iconUrl = (value?: string | null) => {
    const path = cachedPluginIconPath(value)
    return path ? resolveIcon(path) : undefined
  }

  React.useEffect(() => { onBusyChange?.(busy) }, [busy, onBusyChange])
  React.useEffect(() => () => { onBusyChange?.(false) }, [onBusyChange])
  React.useEffect(() => {
    if (!state.operation.active) setOperationTarget(undefined)
  }, [state.operation.active, state.operation.updatedAt])
  React.useEffect(() => {
    if (installRequest <= lastInstallRequest.current) return
    lastInstallRequest.current = installRequest
    setTrusted(false); setError(''); setInstallNotice(null); setInstall({ name: '', version: '' })
  }, [installRequest])
  React.useEffect(() => {
    if (settingsId && !settingsPack) setSettingsId(null)
  }, [settingsId, settingsPack])

  async function act(path: string, body: Record<string, unknown>): Promise<boolean> {
    if (pending.current || locked) return false
    pending.current = true
    setBusy(true); setError('')
    const id = typeof body.id === 'string' ? body.id : undefined
    const pack = state.packages.find(pack => pack.id === id)
    setOperationTarget({ id, name: pack?.name, source: typeof body.name === 'string' ? body.name.trim() : undefined })
    try {
      await request(path, body)
      refreshPluginStartup()
      await refresh()
      return true
    } catch (failure) {
      setOperationTarget(undefined)
      setError(failure instanceof Error ? failure.message : String(failure))
      return false
    } finally { pending.current = false; setBusy(false) }
  }

  function openInstall(value: Install): void {
    setSettingsId(null); setTrusted(false); setError(''); setInstallNotice(null); setInstall(value)
  }

  async function confirmInstall(): Promise<void> {
    if (!installNotice || !trusted || locked || pending.current) return
    const source = installNotice.name.trim()
    const accepted = await act('/api/plugins/scripts/install', {
      id: installNotice.id, name: source, source, version: installNotice.version.trim(), trusted: true,
      // 旧网关仍读取这一字段；新版不再提供单独的签到授权开关。
      checkinAllowed: true,
    })
    if (accepted) { setInstallNotice(null); setInstall(null) }
  }

  const starting = state.packages.some(pack => pluginPending(startupForPack(pack.id, pack.startup)))
  const failure = error || state.error || (!starting && (state.hostError || state.runtime.error || state.operation.error))

  return <>
    {failure ? <Alert variant='destructive' className='whitespace-pre-wrap break-words'>{failure}</Alert> : null}
    {galleryVisible || count > 0 || unmatchedOperation ? <section aria-label={t('Magpie plugins')}>
      <PluginSectionHeading t={t} title={t('Magpie plugins')} count={count + Number(unmatchedOperation)}
        filters={galleryVisible ? <SegmentedControl<ScriptPublisherFilter> value={publisher} onValueChange={changePublisher}
          aria-label={`${t('Magpie plugins')} · ${t('Source')}`} className='max-w-full flex-wrap' options={[
            { value: 'all', label: t('All'), count: publisherCount('all') },
            { value: 'official', label: t('Magpie official'), count: publisherCount('official') },
            { value: 'recommended', label: t('Magpie recommended'), count: publisherCount('recommended') },
            { value: 'third-party', label: t('Third-party'), count: publisherCount('third-party') },
          ]} /> : null}
        hint={t('Official and community Magpie plugins share one runtime downloaded on demand')}
        actions={galleryVisible && showInstallAction ? <Button variant='ghost' size='sm' disabled={locked} onClick={() => openInstall({ name: '', version: '' })}>
          {t('Install from npm / GitHub / Magpie')}
        </Button> : null} />
      {galleryVisible && state.catalog ? <ScriptCatalogStatus catalog={state.catalog} request={request} refresh={refresh} disabled={disabled} t={t} /> : null}
      <div className='grid grid-cols-1 gap-3.5 min-[900px]:grid-cols-2'>
        {unmatchedOperation ? <ScriptOperationCard operation={operation} t={t} /> : null}
        {visibleGallery.map(entry => {
          const progress = scriptOperationMatches(operation, entry.value) ? <ScriptOperationProgress operation={operation} t={t} /> : null
          if (entry.kind === 'recommended') {
            const item = entry.value
            const unavailable = item.installable === false ? scriptUnavailableReason(item, t) : ''
            return <PluginStoreCard key={`recommended:${recommendationKey(item)}`} t={t} name={item.label}
              description={recommendedDescription(item, t)} version={item.version}
              highlightedSource={scriptOfficialBadge(item)} sourceLabel={scriptSourceLabel(item, t)}
              icon={iconUrl(item.icon)} status='missing' providerCount={item.providers.length || undefined}
              metadata={<ScriptRepository item={item} onOpenExternal={onOpenExternal} t={t} />}
              feedback={<>{unavailable ? <p className='m-0 break-words text-xs text-muted-foreground'>{unavailable}</p> : null}{progress}</>}
              actions={unavailable ? <Badge variant='secondary' shape='tag'>{t('Unavailable')}</Badge>
                : <PluginGetButton t={t} disabled={locked} onClick={() => openInstall({ name: item.source || item.name, version: item.selector ?? item.version })} />} />
          }
          const pack = entry.value
          const startup = startupForPack(pack.id, pack.startup)
          const name = scriptPluginName(pack, recommended, t)
          const update = scriptUpdate(state, pack.id)
          return <PluginStoreCard key={`installed:${pack.id}`} t={t} name={name} description={scriptPluginDescription(pack, recommended, t)}
            version={pack.version} icon={iconUrl(pack.icon)} highlightedSource={scriptOfficialBadge(pack)} sourceLabel={scriptSourceLabel(pack, t)}
            badges={update ? <Badge variant='warning' shape='tag'>{updateLabel(update, t)}</Badge> : undefined}
            metadata={<ScriptRepository item={pack} onOpenExternal={onOpenExternal} t={t} />}
            status={!pack.enabled ? 'disabled' : pack.running && !pack.error ? 'running' : 'unavailable'} startup={startup}
            footerAction={<Button variant='link' size='xs' onClick={() => requestPluginDiagnostics(pack.id)}>{t('Load diagnostics')}</Button>}
            providerCount={pack.providers.length} enabledCount={pack.providers.filter(provider => provider.enabled).length}
            actions={<>
              {update ? <Button variant='outline' size='sm' disabled={locked}
                onClick={() => openInstall(reinstallRequest(pack))}>{t('Update')}</Button> : null}
              <PluginSettingsButton name={name} t={t} onClick={() => { setError(''); setSettingsId(pack.id) }} />
              <Switch checked={pack.enabled} disabled={locked} aria-label={`${t('enabled')} ${name}`}
                onCheckedChange={enabled => { void act('/api/plugins/scripts/change', { id: pack.id, action: 'enable', enabled }) }} />
            </>}
            feedback={<>
              {(startup ? pluginFailed(startup) && startup.error : pack.error) ? <p className='m-0 break-words text-xs text-destructive'>{startup?.error || pack.error}</p> : null}
              {progress}
            </>} />
        })}
      </div>
      {galleryVisible ? <ScriptPaginationFooter page={page} total={count} t={t} /> : null}
      {galleryVisible && count === 0 && !unmatchedOperation ? <Card><Empty>
        <EmptyTitle>{catalogPending ? t('Loading plugin catalog…') : catalogFailed ? t('The plugin catalog is not fully loaded')
          : !recommended.length && !state.packages.length ? t('No plugins available to install') : t('No plugins match these filters')}</EmptyTitle>
        <EmptyDescription>{catalogPending ? t('Plugins appear as their information loads. You can leave this page while loading continues.')
          : catalogFailed ? t('See the reason above and refresh again later.') : t('Try different filters, or set both Source and Status back to All.')}</EmptyDescription>
        {publisher !== 'all' || status !== 'all' || query.trim() ? <Button variant='outline' size='sm' className='mt-2'
          onClick={() => { changePublisher('all'); onClearFilters?.() }}>{t('Clear filters')}</Button> : null}
      </Empty></Card> : null}
    </section> : null}
    {settingsPack ? <ScriptPluginSettings key={settingsPack.id} pack={settingsPack} icon={iconUrl(settingsPack.icon)} recommended={recommended} t={t}
      disabled={locked} error={error} request={request} refresh={refresh} onOpenExternal={onOpenExternal} onClose={() => setSettingsId(null)}
      onEnabled={enabled => { void act('/api/plugins/scripts/change', { id: settingsPack.id, action: 'enable', enabled }) }}
      onProvider={(provider, enabled) => { void act('/api/plugins/scripts/change', { id: settingsPack.id, action: 'provider', provider, enabled }) }}
      onReinstall={() => openInstall(reinstallRequest(settingsPack))}
      onRemove={() => { setError(''); setSettingsId(null); setRemove(settingsPack) }} /> : null}
    <Dialog open={install !== null} onOpenChange={(open, details) => {
      if (!open) { if (busy || installNotice) details.cancel(); else setInstall(null) }
    }}>
      <DialogContent className='text-foreground'>
        <DialogHeader><DialogTitle>{t('Install script plugin')}</DialogTitle></DialogHeader>
        <DialogBody className='gap-4'>
          <DialogDescription className='text-xs leading-relaxed text-subtle'>{t('Only install npm packages, GitHub repositories or Magpie community packages you trust. Integrity verification is not a security audit.')}</DialogDescription>
          <div className='space-y-2'><Label htmlFor={`${prefix}-package`}>{t('npm package, GitHub URL or magpie:package')}</Label>
            <Input id={`${prefix}-package`} value={install?.name || ''} disabled={busy} autoComplete='off' placeholder='@scope/plugin / https://github.com/owner/repo'
              onChange={event => { setInstall(value => value ? { ...value, name: event.target.value } : value); setTrusted(false) }} /></div>
          <div className='space-y-2'><Label htmlFor={`${prefix}-version`}>{github ? t('Branch, tag or commit (leave blank for the default branch)') : t('Plugin version (leave blank for latest)')}</Label>
            <Input id={`${prefix}-version`} value={install?.version || ''} disabled={busy} placeholder={github ? t('Default branch') : 'latest'} autoComplete='off'
              onChange={event => { setInstall(value => value ? { ...value, version: event.target.value } : value); setTrusted(false) }} /></div>
          {github ? <p className='m-0 text-xs leading-relaxed text-muted-foreground'>{t('Downloads a pinned commit directly from GitHub. The repository must include a runnable provider entry point. Runtime dependencies are installed, but build and installation scripts are not run.')}</p> : null}
          {magpie ? <p className='m-0 text-xs leading-relaxed text-muted-foreground'>{t('Downloads from Magpie\'s official community distribution with SHA-512 verification. Updates and reinstalls keep this source.')}</p> : null}
          <p className='m-0 text-xs leading-relaxed text-muted-foreground'>{t('You can also enter file: followed by the absolute path to a plugin directory or tgz file on the gateway computer. Local plugins are copied for installation without changing the source directory.')}</p>
          <p className='m-0 text-xs leading-relaxed text-muted-foreground'>{t('Packages missing from npm are looked up in Magpie\'s community distribution. Integrity checks are never skipped, and install scripts are not run.')}</p>
          <div className='space-y-2 text-xs leading-relaxed text-muted-foreground'>
            <Badge variant='outline'>Bun {state.runtime.version} · {state.runtime.installed ? t('Downloaded') : t('Download on demand')}</Badge>
            <p className='m-0'>{t('The runtime is not bundled with the installer. It is downloaded and verified when you first install a script plugin.')}</p>
            <p className='m-0'>{t('Supports OpenCode v1 / Magpie provider plugins and their runtime dependencies. UI plugins, standalone tools and Pi extensions are not loaded.')}</p>
            <p className='m-0'>{t('Supports automatic authorization, device codes, manual authorization codes and plugin-defined credential forms. Local imports read from the gateway machine.')}</p>
          </div>
          <Alert variant='warning'>{t('Scripts are not sandboxed and can access files and the network on the gateway computer. Do not install plugins from unknown sources.')}</Alert>
          <Label className='flex items-start gap-2 text-xs leading-relaxed'><Checkbox checked={trusted} disabled={busy} onCheckedChange={value => setTrusted(value === true)} />
            {t('I trust this plugin source and allow it to run on the gateway computer.')}</Label>
          {error ? <p role='alert' className='m-0 text-xs text-destructive'>{error}</p> : null}
        </DialogBody>
        <DialogFooter><Button variant='outline' disabled={busy} onClick={() => setInstall(null)}>{t('Cancel')}</Button>
          <Button disabled={locked || !trusted || !install?.name.trim()} onClick={() => {
            if (install && trusted && !locked && !pending.current) {
              setError('')
              setInstallNotice({ ...install, name: install.name.trim(), version: install.version.trim() })
            }
          }}>{t('Continue installing')}</Button></DialogFooter>
        <ScriptPluginInstallNotice open={installNotice !== null} source={installNotice?.name || ''}
          version={installNotice?.version || ''} disabled={locked || !trusted} busy={busy} error={error} t={t}
          onCancel={() => { setInstallNotice(null); setError('') }} onConfirm={() => void confirmInstall()} />
      </DialogContent>
    </Dialog>
    <Dialog open={remove !== null} onOpenChange={open => { if (!open && !busy) setRemove(null) }}>
      <DialogContent className='text-foreground'><DialogHeader><DialogTitle>{t('Uninstall plugin pack')}</DialogTitle></DialogHeader>
        <DialogBody><p className='m-0 text-sm leading-relaxed'>{t('Uninstall “{name}”? Its providers will stop forwarding. Account data is retained and becomes available again after reinstalling.', { name: remove?.name || '' })}</p>
          {error ? <p role='alert' className='m-0 text-xs text-destructive'>{error}</p> : null}
        </DialogBody>
        <DialogFooter><Button variant='outline' disabled={busy} onClick={() => setRemove(null)}>{t('Cancel')}</Button>
          <Button variant='destructive' disabled={locked} onClick={async () => {
            if (remove && await act('/api/plugins/scripts/change', { id: remove.id, action: 'remove' })) setRemove(null)
          }}>{t('Uninstall')}</Button></DialogFooter>
      </DialogContent>
    </Dialog>
  </>
}
