import * as React from 'react'
import { Badge, Button, Collapsible, CollapsibleContent, CollapsibleTrigger, Empty, InputGroup, InputGroupAddon, InputGroupInput, Label, Spinner, confirmDialog } from '@ui'
import { t } from '../i18n'
import { displayNameOf, providerOf } from './accounts-domain'
import { RoutingDragOverlay, reorderRoute, useRoutingDrag, type RoutingDrop } from './routing-drag'
import { RoutingModelFilter, useRoutingModelFilter } from './routing-model-filter'
import { RoutingSelect } from './routing-select'
import { RoutingActions } from './routing-canvas-controls'
import { RoutingCanvasLayout, RoutingDraftFooter, RoutingRefreshError } from './routing-canvas-layout'
import { useRoutingCanvasActivity } from './routing-canvas-activity'
import { RoutingProviderCard } from './routing-provider-card'
import { routingAccountSections, routingCollapsedSections, useRoutingExpansion } from './routing-account-sections'
import { RouteIcon, stickyLabel, strategyLabel } from './routing-ui'
import { draftDirty, type RoutingEditorState } from './routing-editor-state'
import { RoutingLegacyAccountsDialog, type useRoutingAccounts } from './routing-accounts'
import { host, type RouteProfile, type RoutingWorkspace } from './routing-types'

export type RoutingConfigProps = {
  editor: RoutingEditorState; scope: string; now: number; accountControl: ReturnType<typeof useRoutingAccounts>
  onAccount(id: string): void; onSort(provider: string): void; onProtection(workspace: RoutingWorkspace): void
  refreshToken: number; onWorkspace(workspace: RoutingWorkspace): void
}

/** 账号启停即时保存；独立路由的参与名单、策略与顺序保留草稿。 */
export function RoutingConfigPanel({ editor, scope, now, accountControl, refreshToken, onWorkspace, onAccount, onSort, onProtection }: RoutingConfigProps) {
  const { setOpened, sectionOpen, setSectionOpen, search, setSearch, query, isOpen } = useRoutingExpansion()
  const [legacyDialog, setLegacyDialog] = React.useState<RoutingWorkspace['legacyGlobalParticipation']>(null)
  const draft = editor.draft
  const globalAccounts = !scope
  const legacy = draft?.workspace.legacyGlobalParticipation
  const globalBlocked = globalAccounts && (!draft?.workspace.globalAccountControl || Boolean(legacy))
  const modelFilter = useRoutingModelFilter(draft?.workspace, scope)
  const live = useRoutingCanvasActivity(Boolean(draft), modelFilter.model, {
    workspaceModel: scope, paused: editor.busy || editor.loading, refreshToken, onWorkspace,
  })
  const { activity } = live
  const disabled = editor.busy || editor.loading
  const mutate = React.useCallback((fn: (profile: RouteProfile, workspace: RoutingWorkspace) => void) => {
    editor.update(current => { const profile = structuredClone(current.profile); fn(profile, current.workspace); return { ...current, profile } })
  }, [editor])
  const drop = React.useCallback(({ kind, provider, id, target, after }: RoutingDrop) => mutate((profile, workspace) => {
    if (kind === 'provider' && profile.providerScope === 'all') {
      for (const item of workspace.providers) if (!profile.providers.includes(item.id)) profile.providers.push(item.id)
    }
    if (kind === 'account') {
      profile.accountOrder[provider] = [...new Set([...(profile.accountOrder[provider] || []), ...workspace.accounts.filter(account => providerOf(account) === provider).map(account => account.id)])]
    }
    reorderRoute(kind === 'provider' ? profile.providers : profile.accountOrder[provider] || [], id, target, after)
  }), [mutate])
  const drag = useRoutingDrag(!disabled && !globalBlocked && !modelFilter.loading && draft?.mode !== 'inherit' && !search.trim(), `${scope}:${modelFilter.model}:${search}`, drop)
  if (!draft) return <div className='routing-config-loading'>
    <Empty>{editor.loading ? <Spinner /> : editor.error || t('Select a scope to configure routing')}</Empty>
  </div>
  const workspace = live.workspace && live.workspace.updatedAt >= draft.workspace.updatedAt ? live.workspace : draft.workspace
  const inherited = draft.mode === 'inherit'
  const profile = inherited ? draft.workspace.inheritedProfile || draft.profile : draft.profile
  const locked = disabled || inherited || globalBlocked
  const dirty = draftDirty(draft)
  const label = (id: string) => workspace.providers.find(item => item.id === id)?.label || id
  const allProviders = [...new Set([...profile.providers, ...workspace.providers.map(provider => provider.id)])]
  const enabledProviders = new Set(workspace.accounts.filter(account => account.enabled !== false).map(providerOf))
  const included = (id: string) => globalAccounts ? enabledProviders.has(id)
    : (profile.providerScope === 'all' || profile.providers.includes(id)) && !profile.disabledProviders.includes(id)
  const participating = allProviders.filter(included)
  const excluded = allProviders.filter(id => !included(id))
  const participants = workspace.accounts.filter(account => globalAccounts ? account.enabled !== false
    : included(providerOf(account)) && !profile.excludedAccounts.includes(account.id))
  const accountProviders = new Set(workspace.accounts.map(providerOf))
  const displayedProviders = allProviders.filter(id => accountProviders.has(id) && (!modelFilter.providers || modelFilter.providers.has(id)))
  const displayedParticipating = displayedProviders.filter(included)
  const displayedAccounts = workspace.accounts.filter(account => displayedProviders.includes(providerOf(account))
    && (!query || label(providerOf(account)).toLowerCase().includes(query) || displayNameOf(account).toLowerCase().includes(query)))
  const sections = routingAccountSections(displayedProviders, workspace.accounts, id => id, included)
  const orderingPeers = (id: string) => (Object.values(sections).find(ids => ids.includes(id)) || []).filter(included)
  const matches = (id: string) => !query || label(id).toLowerCase().includes(query)
    || workspace.accounts.some(account => providerOf(account) === id && displayNameOf(account).toLowerCase().includes(query))
  const visible = sections.active.filter(matches)
  const expandable = [...sections.active, ...routingCollapsedSections.flatMap(({ key }) => sectionOpen[key] ? sections[key] : [])].filter(matches)
  const allOpen = expandable.length > 0 && expandable.every(isOpen)
  const ensure = (next: RouteProfile, id: string) => {
    if (!next.providers.includes(id)) next.providers.push(id)
    next.accountOrder[id] = [...new Set([...(next.accountOrder[id] || []), ...workspace.accounts.filter(account => providerOf(account) === id).map(account => account.id)])]
  }
  const enableGroup = (id: string, on: boolean) => {
    void accountControl.setEnabled(workspace.accounts.filter(account => providerOf(account) === id).map(account => account.id), on, true)
  }
  const join = (id: string, on: boolean) => {
    if (globalAccounts) { enableGroup(id, on); return }
    mutate(next => {
      ensure(next, id)
      next.disabledProviders = on ? next.disabledProviders.filter(value => value !== id) : [...new Set([...next.disabledProviders, id])]
    })
  }
  const moveProvider = (id: string, direction: number) => mutate(next => {
    participating.forEach(provider => ensure(next, provider))
    const peers = orderingPeers(id), target = peers[peers.indexOf(id) + direction]
    if (target) reorderRoute(next.providers, id, target, direction > 0)
  })
  const moveAccount = (provider: string, id: string, direction: number) => mutate(next => {
    ensure(next, provider)
    const order = next.accountOrder[provider].filter(key => globalAccounts
      ? (workspace.accounts.find(account => account.id === key)?.enabled !== false) === (workspace.accounts.find(account => account.id === id)?.enabled !== false)
      : profile.excludedAccounts.includes(key) === profile.excludedAccounts.includes(id))
    const target = order[order.indexOf(id) + direction]
    if (target) reorderRoute(next.accountOrder[provider], id, target, direction > 0)
  })
  const removeAll = async () => {
    if (!await confirmDialog(t('Remove only from this route. Accounts and other routes stay unchanged. After saving, this entry has no available route.'), {
      title: t('Remove all from this route'), confirmText: t('Remove all'),
    })) return
    mutate(next => { next.providerScope = 'selected'; next.providers = allProviders; next.disabledProviders = [...allProviders] })
  }
  const save = async () => {
    if (!globalAccounts && !participants.length && !await confirmDialog(t('No accounts participate in this entry. After saving, requests fail explicitly, without global fallback or restoring the list automatically.'), {
      title: t('Save empty route'), confirmText: t('Save anyway'), destructive: true,
    })) return
    await editor.save()
  }
  const removeIndependent = async () => {
    if (!await confirmDialog(t('Deleting it makes this model follow the global configuration again. Saved independent settings and any unsaved changes will be removed.'), {
      title: t('Delete independent config'), confirmText: t('Delete'), destructive: true,
    })) return
    await editor.remove()
  }
  const renderProvider = (id: string) => <RoutingProviderCard key={id} id={id} profile={profile} workspace={workspace} scope={scope} now={now}
    index={participating.indexOf(id)} first={orderingPeers(id)[0] === id} last={orderingPeers(id).at(-1) === id}
    included={included(id)} open={isOpen(id)} locked={locked} busy={editor.busy} query={query} drag={drag} activity={activity}
    onOpen={open => setOpened(current => ({ ...current, [id]: open }))} onJoin={on => join(id, on)} onMove={direction => moveProvider(id, direction)}
    onMoveAccount={(key, direction) => moveAccount(id, key, direction)} onToggleAccount={key => {
      if (globalAccounts) {
        const account = workspace.accounts.find(account => account.id === key)
        if (account) void accountControl.setEnabled([key], account.enabled === false)
        return
      }
      mutate(next => { next.excludedAccounts = next.excludedAccounts.includes(key) ? next.excludedAccounts.filter(value => value !== key) : [...next.excludedAccounts, key] })
    }} onAccount={onAccount} onSort={() => onSort(id)} onEnableGroup={globalAccounts ? on => enableGroup(id, on) : undefined} />
  const groups = <section className='routing-groups-column'>
    <div className='routing-column-head'><RouteIcon name='accounts' size={14} /><b>{t('账号组')}</b><span>{globalAccounts
      ? t('{n} accounts enabled', { n: displayedAccounts.filter(account => account.enabled !== false).length })
      : t('{n} participating', { n: displayedParticipating.length })}</span>
      <div className='right'><Button variant='ghost' size='sm' disabled={!expandable.length || Boolean(query)} onClick={() => setOpened(current => ({ ...current, ...Object.fromEntries(expandable.map(id => [id, !allOpen])) }))}>
        {allOpen ? t('Collapse all') : t('Expand all')}</Button>
        <RoutingActions label={globalAccounts ? t('Account enable/disable actions') : t('Route participation actions')} actions={globalAccounts ? [
          { label: t('Enable accounts in this list'), disabled: locked || !displayedAccounts.some(account => account.enabled === false),
            onClick: () => void accountControl.setEnabled(displayedAccounts.map(account => account.id), true, true) },
          { label: t('Disable accounts in this list'), disabled: locked || !displayedAccounts.some(account => account.enabled !== false),
            onClick: () => void accountControl.setEnabled(displayedAccounts.map(account => account.id), false, true) },
        ] : [{ label: t('Remove all'), disabled: locked || !participating.length, onClick: () => void removeAll() }]} />
      </div>
    </div>
    <InputGroup className='routing-config-search'><InputGroupInput value={search} placeholder={t('Search providers or accounts')} aria-label={t('Search providers or accounts')}
      onChange={event => setSearch(event.currentTarget.value)} /><InputGroupAddon><RouteIcon name='search' size={14} /></InputGroupAddon></InputGroup>
    <p className='routing-groups-hint'>{t('Drag a handle or use Alt + arrow keys to reorder. More actions are in the menu on the right.')}</p>
    <RoutingRefreshError error={live.error} onRetry={live.refresh} />
    <div className='routing-config-listcol' data-route-scroll aria-busy={modelFilter.loading}>
      {modelFilter.loading ? <Empty><Spinner />{t('Loading…')}</Empty> : modelFilter.error ? <Empty>
        <p className='routing-config-error' role='alert'>{modelFilter.error}</p><Button variant='outline' size='sm' onClick={modelFilter.retry}>{t('Retry')}</Button>
      </Empty> : <>
        <div className='routing-config-participants' data-route-list>{visible.map(renderProvider)}</div>
        {!globalAccounts && !participating.length && displayedProviders.length ? <Empty>{t('This entry has no participating providers. Requests will fail explicitly after saving.')}</Empty> : null}
        {routingCollapsedSections.map(({ key, label }) => {
          const ids = sections[key].filter(matches)
          return ids.length ? <Collapsible key={key} open={Boolean(sectionOpen[key]) || Boolean(query)}
            onOpenChange={open => setSectionOpen(current => ({ ...current, [key]: open }))} className='routing-config-excluded'>
            <CollapsibleTrigger className='routing-excluded-trigger'><RouteIcon name='chevronDown' size={13} />{label} · {ids.length}</CollapsibleTrigger>
            <CollapsibleContent data-route-list>{ids.map(renderProvider)}</CollapsibleContent>
          </Collapsible> : null
        })}
        {query && displayedProviders.length && !displayedProviders.some(matches) ? <Empty>{t('No matching providers or accounts')}</Empty> : null}
        {!displayedProviders.length ? <Empty><span>{modelFilter.model ? t('No accounts are associated with this model. Add an account or check its model bindings.') : t('No accounts yet. Add one on the Accounts page; only providers with accounts appear here.')}</span>
          <Button variant='link' onClick={() => host().wbApp?.showPage('accounts')}>{t('Add on the Accounts page')}</Button></Empty> : null}
      </>}
    </div>
  </section>
  return <>
    <RoutingCanvasLayout title={scope ? t('Model routes') : t('Global defaults')} groups={groups} globalAccounts={globalAccounts}
      note={scope ? t('Only affects subsequent requests to this route') : t('Account status stays in sync with Accounts and applies immediately to all routes. Save policy and order changes separately.')}>
      <div className='routing-config-card' data-route-source data-route-running={activity.running > 0 ? '' : undefined}>
        <div className='routing-config-card-scroll'>
          <header className='routing-config-head'><div><span className='routing-config-eyebrow'>{inherited ? t('Inherit global') : scope ? t('Independent configuration') : t('Default rules')}</span><h2>{scope || t('Global defaults')}</h2></div>
            <RoutingActions label={t('Route configuration actions')} actions={[
              { label: t('Global protection settings'), icon: 'shield', disabled: editor.busy, onClick: () => onProtection(workspace) },
              ...(scope && workspace.customized ? [{ label: t('Delete independent config'), destructive: true, disabled, onClick: () => void removeIndependent() }] : []),
            ]} />
          </header>
          {globalAccounts && !workspace.globalAccountControl ? <div className='routing-inherited-note' role='alert'>{t('This gateway does not yet support global account controls. Update the gateway and reload.')}</div> : null}
          {globalAccounts && legacy ? <div className='routing-inherited-note'><p>{t('Legacy global participation settings were found. Confirm their conversion before managing account status.')}</p>
            <Button variant='outline' disabled={disabled} onClick={() => setLegacyDialog(legacy)}>{t('Convert legacy settings')}</Button></div> : null}
          {inherited ? <div className='routing-inherited-note'><p>{t('Models without independent configurations use the global route.')}</p><Button variant='outline' disabled={disabled} onClick={() => void editor.create(scope)}>{t('Create independent configuration')}</Button></div> : null}
          <div className='routing-config-rules'>
            <div className='routing-rule-field'><Label>{t('Routing strategy')}</Label><RoutingSelect label={t('Routing strategy')} value={profile.strategy || '__inherit'} disabled={locked}
              options={[...(scope ? [['__inherit', t('Inherit global: {value}', { value: strategyLabel(workspace.globalSettings?.routingStrategy) })] as [string, string]] : []),
                ['order', t('Strict order')], ['rotate', t('Load balancing')], ['provider', t('Per-provider rotation')]]}
              onChange={value => mutate(next => { next.strategy = value === '__inherit' ? undefined : value })} /></div>
            <div className='routing-rule-field'><Label>{t('Session bindings')}</Label><RoutingSelect label={t('Session bindings')} value={profile.sticky || '__inherit'} disabled={locked}
              options={[...(scope ? [['__inherit', t('Inherit global: {value}', { value: stickyLabel(workspace.globalSettings?.routingSticky) })] as [string, string]] : []),
                ['auto', t('Auto')], ['session', t('In-session')], ['turn', t('Per turn')], ['off', t('Close')]]}
              onChange={value => mutate(next => { next.sticky = value === '__inherit' ? undefined : value })} /></div>
            {!globalAccounts ? <div className='routing-rule-field routing-rule-wide'><Label>{t('Participation')}</Label><RoutingSelect label={t('Participation')} value={profile.providerScope || 'all'} disabled={locked}
              options={[['all', t('All providers automatically')], ['selected', t('Only added providers')]]} onChange={value => mutate(next => {
                next.providerScope = value === 'all' ? 'all' : 'selected'; next.providers = allProviders; next.disabledProviders = excluded
              })} /></div> : null}
          </div>
          {!globalAccounts ? <p className='routing-config-note'>{profile.providerScope === 'all' ? t('New providers join automatically. Removed providers are not automatically re-added.') : t('Only added providers participate. New providers must be added manually.')}</p> : null}
          {!scope ? <RoutingModelFilter value={modelFilter.model} models={workspace.models} lastUsed={workspace.modelLastUsed} disabled={disabled} onChange={modelFilter.setModel} /> : null}
          <div className='routing-protection-summary'><RouteIcon name='shield' size={14} /><span>{t('全局保护设置')}</span>
            <Badge variant='ghost' shape='tag'>{[workspace.settings.routingFailoverBackoff && t('Failure backoff'), workspace.settings.routingSink && t('Rate-limit demotion'), workspace.settings.routingBreakerEnabled && t('Circuit breaker')].filter(Boolean).join(' · ') || t('Close')}</Badge>
            <Button variant='link' size='sm' disabled={editor.busy} onClick={() => onProtection(workspace)}>{t('Settings')}</Button>
          </div>
        </div>
        <RoutingDraftFooter dirty={dirty} inherited={inherited} busy={editor.busy} disabled={disabled || globalBlocked}
          error={[editor.error, globalAccounts && accountControl.error].filter(Boolean).join('; ')} onDiscard={editor.discard} onSave={() => void save()} />
      </div>
    </RoutingCanvasLayout>
    <RoutingDragOverlay state={drag.state} />
    {legacyDialog ? <RoutingLegacyAccountsDialog legacy={legacyDialog} busy={accountControl.busy} error={accountControl.error}
      onClose={() => setLegacyDialog(null)} onConfirm={() => { void accountControl.convert(legacyDialog.revision).then(done => { if (done) setLegacyDialog(null) }) }} /> : null}
  </>
}
