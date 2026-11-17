import * as React from 'react'
import { mountOnPage } from '../island-lazy'
import { createRoot } from 'react-dom/client'
import { Badge, Button, Empty, SegmentedControl, Spinner, confirmDialog } from '@ui'
import { t } from '../i18n'
import { AccountSettingsDialog } from './accounts-settings-dialog'
import type { AccountSorting } from './accounts-shared'
import { getStore, subscribe } from './accounts-store'
import { adoptAccountSorting } from './accounts-sort'
import { GroupSettingsDialog } from './accounts-group-settings'
import { RoutingSettingsDialog } from './routing-settings'
import { RoutingStickyDialog } from './routing-sticky'
import { RoutingConfigPanel } from './routing-config-panel'
import { RoutingGroupPanel } from './routing-group-panel'
import { RoutingCreateDialog } from './routing-create-dialog'
import { useRouteGroups } from './routing-group-state'
import { useRoutingAccounts } from './routing-accounts'
import { RoutingRail, type SpecialKey } from './routing-rail'
import { CompactSpecialPanel, TitleSpecialPanel, VisionSpecialPanel } from './routing-special-panels'
import { RoutingSpecialHits } from './routing-special-hits'
import { compactApi, titleApi, useSpecialEditor, visionApi } from './routing-special-state'
import { RoutingConfigShell } from './routing-canvas-layout'
import { RoutingExecTimeline } from './routing-exec-timeline'
import { RoutingRequestList } from './routing-request-list'
import { useRoutingReplay } from './routing-replay-state'
import { draftDirty, useRoutingEditor } from './routing-editor-state'
import { RouteIcon, stickyLabel, strategyLabel } from './routing-ui'
import { isTraceRunning, requestKey } from './routing-trace-model'
import { bridge, errorMessage, host, type ReplayRow, type RoutingWorkspace } from './routing-types'

let requestTrace: ((row?: ReplayRow | null) => void) | undefined
let refreshWorkspace: (() => void) | undefined

export function RoutingPage() {
  const [mode, setMode] = React.useState<'config' | 'replay'>('config')
  // 选中的特殊路由（null = 走原来的账号路由配置视图）
  const [specialKey, setSpecialKey] = React.useState<SpecialKey | null>(null)
  const [selected, setSelected] = React.useState<ReplayRow | null>(null)
  const [refreshToken, setRefreshToken] = React.useState(0)
  const sortingState = React.useSyncExternalStore(subscribe, getStore)
  const [sortingSyncBusy, setSortingSyncBusy] = React.useState(false)
  const [sortingError, setSortingError] = React.useState('')
  const sortSequence = React.useRef(0)
  const replay = useRoutingReplay(true, mode === 'config' ? '' : undefined, mode === 'replay' || Boolean(specialKey))
  const refreshRouting = React.useCallback(() => { replay.invalidateWorkspace(); replay.refresh() }, [replay.invalidateWorkspace, replay.refresh])
  const editor = useRoutingEditor(refreshRouting)
  const groups = useRouteGroups(refreshRouting, () => { void editor.load('') })
  const accountControl = useRoutingAccounts(editor, replay.suspendWorkspace, replay.resumeWorkspace)
  const pageEditor = { ...editor, busy: editor.busy || accountControl.busy || sortingState.sortBusy || sortingSyncBusy }
  const busyRef = React.useRef(pageEditor.busy)
  busyRef.current = pageEditor.busy
  // 特殊请求路由：三块各自持有草稿，但同一时刻只显示一块，所以只让当前那块拉数据。
  const special = React.useMemo(() => ({
    compact: compactApi(),
    title: titleApi(),
    vision: visionApi,
  }), [])
  const compactEditor = useSpecialEditor(special.compact, mode === 'config' && specialKey === 'compact')
  const visionEditor = useSpecialEditor(special.vision, mode === 'config' && specialKey === 'vision')
  const titleEditor = useSpecialEditor(special.title, mode === 'config' && specialKey === 'title')
  const specialDirty = (compactEditor.dirty ? 1 : 0) + (visionEditor.dirty ? 1 : 0) + (titleEditor.dirty ? 1 : 0)
  const dirtyCount = editor.dirtyCount + groups.dirtyCount + specialDirty
  const [modal, setModal] = React.useState<
    | { kind: 'sticky' }
    | { kind: 'account' | 'sort'; id: string }
    | { kind: 'protection'; workspace: RoutingWorkspace }
    | { kind: 'create' }
    | null
  >(null)
  const workspace = editor.draft?.workspace || replay.workspace
  const usageWorkspace = replay.workspace && (!workspace || replay.workspace.updatedAt >= workspace.updatedAt) ? replay.workspace : workspace
  const modelLastUsed = usageWorkspace?.modelLastUsed
  const sortSnapshot = replay.workspace?.sorting && (!workspace?.sorting || replay.workspace.sorting.updatedAt > workspace.sorting.updatedAt)
    ? replay.workspace.sorting : workspace?.sorting
  const sorting = sortingState.sorting && (!sortSnapshot || sortingState.sorting.updatedAt > sortSnapshot.updatedAt) ? sortingState.sorting : sortSnapshot
  const profile = editor.draft?.mode === 'inherit' ? workspace?.inheritedProfile : editor.draft?.profile
  const strategy = groups.draft?.group.strategy || profile?.strategy || workspace?.globalSettings?.routingStrategy || workspace?.settings.routingStrategy
  const sticky = groups.draft?.group.sticky || profile?.sticky || workspace?.globalSettings?.routingSticky || workspace?.settings.routingSticky
  const scopeModel = groups.draft?.group.model || editor.scope

  React.useEffect(() => { void editor.load('') }, [editor.load])
  React.useEffect(() => { if (sortSnapshot) adoptAccountSorting(sortSnapshot) }, [sortSnapshot])
  React.useEffect(() => {
    if (!replay.workspace || accountControl.busy || editor.busy || sortingState.sortBusy || sortingSyncBusy) return
    editor.syncAccounts(replay.workspace.accounts, replay.workspace.updatedAt)
    editor.syncWorkspace(replay.workspace)
    editor.syncSorting(replay.workspace.sorting)
  }, [replay.workspace, accountControl.busy, editor.busy, sortingState.sortBusy, sortingSyncBusy, editor.syncAccounts, editor.syncWorkspace, editor.syncSorting])
  const reloadSorting = React.useCallback(async () => {
    const id = ++sortSequence.current
    busyRef.current = true
    replay.invalidateWorkspace()
    setSortingSyncBusy(true); setSortingError('')
    try {
      const next = await bridge().getRoutingWorkspace()
      if (id !== sortSequence.current) return
      editor.syncAccounts(next.accounts, next.updatedAt)
      editor.syncWorkspace(next)
      editor.syncSorting(next.sorting)
      adoptAccountSorting(next.sorting)
    } catch (failure) {
      if (id === sortSequence.current) setSortingError(t('Consumption rules were saved, but the routing list could not be refreshed: {error}', { error: errorMessage(failure) }))
    } finally {
      if (id === sortSequence.current) { setSortingSyncBusy(false); refreshRouting() }
    }
  }, [editor.syncAccounts, editor.syncWorkspace, editor.syncSorting, replay.invalidateWorkspace, refreshRouting])
  React.useEffect(() => {
    const changed = (event: Event) => {
      const next = (event as CustomEvent<AccountSorting>).detail
      if (!next?.rules || !next.accountOrder) return
      void reloadSorting()
    }
    window.addEventListener('wb:account-sorting', changed)
    return () => { sortSequence.current++; window.removeEventListener('wb:account-sorting', changed) }
  }, [reloadSorting])
  React.useEffect(() => {
    if (mode !== 'replay' || !selected || !isTraceRunning(selected)) return
    let alive = true
    let pending = false
    const timer = window.setInterval(async () => {
      if (pending || document.hidden || host().wbApp?.currentPage !== 'routing') return
      pending = true
      try {
        const next = await bridge().getRoutingTraceRequest({ id: selected.id, ts: selected.ts })
        if (alive) setSelected(current => current && requestKey(current) === requestKey(next) ? next : current)
      } catch { /* 下个周期再试 */ }
      finally { pending = false }
    }, 2500)
    return () => { alive = false; window.clearInterval(timer) }
  }, [mode, selected?.id, selected?.ts, selected && isTraceRunning(selected)])

  const openRequest = (row: ReplayRow) => { setSelected(row); setMode('replay') }
  const openScope = (model: string) => {
    if (busyRef.current || groups.busy) return
    // 从特殊路由切回账号路由：左栏的选中态必须跟着走，否则会同时亮两项。
    setSpecialKey(null)
    const group = Object.values(groups.drafts).find(draft => draft.group.model.toLowerCase() === model.toLowerCase())
    groups.select(group?.group.id || '')
    if (!group) void editor.load(model)
  }
  const openConfig = (model: string) => { setMode('config'); openScope(model) }
  const refreshReplay = async () => {
    replay.refresh()
    setRefreshToken(value => value + 1)
    if (!selected) return
    try {
      const next = await bridge().getRoutingTraceRequest({ id: selected.id, ts: selected.ts })
      setSelected(current => current && requestKey(current) === requestKey(next) ? next : current)
    } catch { /* 列表下次刷新会带回新状态 */ }
  }
  const close = () => { setModal(null); refreshRouting() }
  const reloadConfig = async () => {
    replay.refresh()
    if (groups.selected) { await groups.reload(); return }
    if (editor.draft && draftDirty(editor.draft)
      && !await confirmDialog(t('Reloading discards the unsaved draft for this scope. Drafts in other scopes are kept.'), { title: t('Reload configuration'), confirmText: t('Reload') })) return
    void editor.load(editor.scope, true)
  }
  React.useEffect(() => {
    requestTrace = row => { if (row) setSelected({ ...row }); setSpecialKey(null); setMode('replay') }
    refreshWorkspace = replay.refresh
    return () => { requestTrace = undefined; refreshWorkspace = undefined }
  }, [replay.refresh])

  const groupModels = React.useMemo(() => Object.values(groups.drafts).map(draft => draft.group.model), [groups.drafts])
  const reserved = [...(workspace?.models || []), ...(workspace?.configuredModels || []), ...Object.keys(editor.drafts),
    ...(groups.data?.catalog || []).map(choice => choice.model), ...groupModels]
  // 特殊请求路由的编辑器已在上面创建，这里不再重复声明。
  const rail = <RoutingRail editor={pageEditor} groups={groups} workspace={workspace} onModel={openScope} onGroup={id => { if (!busyRef.current) groups.select(id) }}
    special={specialKey} onSpecial={key => { if (!busyRef.current) setSpecialKey(key) }}
    onAdd={() => setModal({ kind: 'create' })} />
  const accountSettings = (id: string) => {
    if (busyRef.current) return
    setModal({ kind: 'account', id })
  }
  const groupSettings = (id: string) => {
    if (busyRef.current || groups.busy) return
    if (sorting) adoptAccountSorting(sorting)
    setModal({ kind: 'sort', id })
  }

  return <>
    <div className='routing-page-bar'>
      <SegmentedControl aria-label={t('Routing mode')} value={mode} onValueChange={setMode} options={[
        { value: 'config', label: <><RouteIcon name='route' />{t('Routing configuration')}{dirtyCount ? <Badge variant='warning' shape='tag'>{dirtyCount}</Badge> : null}</> },
        { value: 'replay', label: <><RouteIcon name='requests' />{t('Request replay')}</> },
      ]} />
      <span className='routing-page-policy'>{mode === 'config'
        ? specialKey ? t('Special request routing · rewrites the model only, account rules stay unchanged')
          : workspace ? `${groups.draft && strategy === 'provider' ? t('Rotate within members') : strategyLabel(strategy)} · ${t('Affinity {mode}', { mode: stickyLabel(sticky) })}` : ''
        : t('Read-only history; current settings are unchanged')}</span>
      <div className='routing-page-actions'>
        {mode === 'replay' ? <Button variant='ghost' size='sm' onClick={() => host().wbApp?.showPage('requests')}><RouteIcon name='requests' />{t('Request logs')}</Button> : null}
        <Button variant='ghost' size='sm' onClick={() => setModal({ kind: 'sticky' })}><RouteIcon name='lock' />{t('Session bindings')}</Button>
        {mode === 'replay'
          ? <Button variant='ghost' size='sm' disabled={replay.loading} onClick={() => void refreshReplay()}><RouteIcon name='refresh' />{t('Refresh')}</Button>
          : <Button variant='ghost' size='sm' disabled={editor.loading || pageEditor.busy || groups.loading || groups.busy} onClick={() => void reloadConfig()}><RouteIcon name='refresh' />{t('Reload')}</Button>}
      </div>
    </div>

    {replay.error && mode === 'replay' ? <div className='routing-page-error' role='alert'>
      <RouteIcon name='info' /><span>{replay.error}</span><Button variant='ghost' size='sm' onClick={replay.refresh}>{t('Retry')}</Button>
    </div> : null}

    {mode === 'config' ? <div className='routing-config-view'>
      {editor.error && !editor.draft ? <div className='routing-page-error' role='alert'>
        <RouteIcon name='info' /><span>{editor.error}</span><Button variant='ghost' size='sm' onClick={() => void editor.load(editor.scope, true)}>{t('Reload')}</Button>
      </div> : null}
      {sortingError ? <div className='routing-page-error' role='alert'><RouteIcon name='info' /><span>{sortingError}</span>
        <Button variant='ghost' size='sm' disabled={sortingSyncBusy} onClick={() => void reloadSorting()}>{t('Retry')}</Button></div> : null}
      {/* 特殊请求路由只有一份规则列表，用平底画布；账号路由那边保留点阵画布 */}
      <RoutingConfigShell plain={Boolean(specialKey)}>
        {rail}
        {specialKey === 'compact' ? <CompactSpecialPanel editor={compactEditor} /> :
          specialKey === 'vision' ? <VisionSpecialPanel editor={visionEditor} /> :
            specialKey === 'title' ? <TitleSpecialPanel editor={titleEditor} /> :
              groups.selected ? <RoutingGroupPanel key={groups.selected} editor={groups} accounts={replay.workspace?.accounts || workspace?.accounts || []}
                reserved={reserved} baseUrl={host().wbPortPanel?.gatewayBase() || `http://127.0.0.1:${workspace?.gateway.port || 3065}`}
                onAccount={accountSettings} sorting={sorting} now={replay.now} onSort={groupSettings} lastUsed={modelLastUsed}
                refreshToken={replay.refreshToken} paused={pageEditor.busy} onWorkspace={replay.acceptWorkspace} /> :
                <RoutingConfigPanel key={editor.scope} editor={pageEditor} scope={editor.scope} now={replay.now} accountControl={accountControl}
                  refreshToken={replay.refreshToken} onWorkspace={replay.acceptWorkspace}
                  onAccount={accountSettings} onSort={groupSettings}
                  onProtection={workspace => setModal({ kind: 'protection', workspace })} />}
        {/* 右栏：选中特殊路由时并排显示这条路由最近命中的请求（原型的「配置在左、记录在右」） */}
        {specialKey ? <RoutingSpecialHits kind={specialKey} refreshToken={refreshToken}
          onOpenReplay={() => setMode('replay')} /> : null}
      </RoutingConfigShell>
    </div> : <div className='routing-replay-layout'>
      <aside className='routing-request-rail'>
        <RoutingRequestList active query={replay.query} selectedKey={selected ? requestKey(selected) : ''} refreshToken={refreshToken}
          onOpenRequest={openRequest} onModel={replay.selectModel} onMinutes={replay.setMinutes} onScope={replay.setScope} onSearch={replay.setSearch} />
      </aside>
      <section className='routing-replay-main'>
        {selected ? <RoutingExecTimeline key={requestKey(selected)} row={selected} now={replay.now}
          onDetail={row => host().wbRequestDetail?.open(row.id, row)} onConfig={openConfig} />
          : <Empty>{replay.loading ? <Spinner /> : t('Select a request on the left to view its routing.')}</Empty>}
      </section>
    </div>}

    {modal?.kind === 'create' && workspace && groups.data ? <RoutingCreateDialog workspace={workspace} choices={groups.data.catalog}
      accounts={replay.workspace?.accounts || workspace.accounts} lastUsed={modelLastUsed} reserved={reserved} groupModels={groupModels}
      disabled={pageEditor.busy || editor.loading || groups.busy || groups.loading} modelError={editor.error} onClose={() => setModal(null)}
      onCreateModel={async model => { const created = await editor.create(model); if (created) groups.select(''); return created }}
      onCreateGroup={groups.create} /> : null}
    {modal?.kind === 'sticky' ? <RoutingStickyDialog model={mode === 'config' ? scopeModel : replay.model} onClose={close} /> : null}
    {modal?.kind === 'protection' ? <RoutingSettingsDialog protectionOnly workspace={modal.workspace} onClose={close} /> : null}
    {modal?.kind === 'sort' ? <GroupSettingsDialog key={modal.id} provider={modal.id} onClose={close}
      accounts={workspace?.accounts} sorting={sorting}
      routeContext={groups.selected || editor.scope && editor.draft?.mode !== 'inherit'
        ? <p className='routing-config-note'>{t('These are the account group\'s global consumption rules. The current independent route\'s account order stays unchanged.')}</p> : undefined} /> : null}
    {modal?.kind === 'account' ? <AccountSettingsDialog key={modal.id} id={modal.id} initialAccount={replay.workspace?.accounts.find(account => account.id === modal.id) || workspace?.accounts.find(account => account.id === modal.id)} onClose={close}
      routeContext={<p className='routing-config-note'>{t('Account settings apply across all routes without changing this route\'s participation list.')}</p>} /> : null}
  </>
}

const selector = '.page[data-page="routing"]'
function mount() {
  const section = document.querySelector<HTMLElement>(selector)
  if (!section || section.dataset.routingMounted === '1') return
  section.dataset.routingMounted = '1'
  createRoot(section).render(<RoutingPage />)
}
declare global { interface Window { wbRoutingPanel?: { load(): Promise<void>; openReplay(row?: ReplayRow | null): Promise<void> } } }
window.wbRoutingPanel = {
  async load() { refreshWorkspace?.() },
  async openReplay(row) { host().wbRequestDetail?.close?.(); host().wbApp?.showPage('routing'); requestTrace?.(row) },
}
mountOnPage('routing', selector, mount)
