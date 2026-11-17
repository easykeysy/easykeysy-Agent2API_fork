import * as React from 'react'
import { mountOnPage } from '../island-lazy'
import { Alert, Button, Card, Collapsible, CollapsibleContent, CollapsibleTrigger, Empty, InputGroup, InputGroupAddon, InputGroupInput, NavItem, NavList, SegmentedControl, Sidebar, StatusDot, cn } from '@ui'
import { t } from '../i18n'
import { counts, getAgentStore, loadPanel, selectClient, selectedClient, setFilter, setQuery, subscribeAgentStore, visibleClients } from './agent-state'
import { blankProfile, detectionLabel } from './agent-model'
import { ProfileEditor } from './agent-editor'
import { PathDialog } from './agent-path-dialog'
import { ApplyDialog } from './agent-preview'
import { ProfileDialog } from './agent-profile-dialog'
import { AgentActions, AgentIcon } from './agent-actions'
import { dshTargetLabel } from './agent-dsh'
import type { AgentClient, AgentOperation, AgentProfile } from './agent-types'
import { AgentResourcesPanel } from '../../../ui-shared/agent-resources-panel'
import type { ResourceKind } from '../../../ui-shared/agent-resources'
import { callJson, toast } from './assistant-state'

function ClientAvatar({client,size = 'md'}:{client:AgentClient;size?:'md'|'lg'}) {
  const [broken,setBroken] = React.useState(false)
  const icon = Boolean(client.icon)&&!broken
  return <span className={cn('agent-avatar',size==='lg'&&'is-lg',icon&&'has-icon',icon&&client.iconFull&&'is-full')}
    style={{'--av-h':String(client.hue)} as React.CSSProperties} aria-hidden='true'>
    {icon?<img src={client.icon} alt='' loading='lazy' onError={()=>setBroken(true)}/>:client.letter}
  </span>
}

export function AgentPage() {
  const store = React.useSyncExternalStore(subscribeAgentStore,getAgentStore,getAgentStore)
  const client = selectedClient()
  const list = visibleClients(store)
  const stats = counts(store)
  const detected = list.filter(client=>client.detected)
  const missing = list.filter(client=>!client.detected)
  return <Card render={<section/>} className='panel agent-panel'>
    <div className='toolbar agent-toolbar'>
      <InputGroup className='agent-search'><InputGroupInput type='search' placeholder={t('Search clients or config files…')} aria-label={t('Search clients')} value={store.query} autoComplete='off' onChange={event=>setQuery(event.currentTarget.value)}/><InputGroupAddon aria-hidden='true'><AgentIcon name='search' /></InputGroupAddon></InputGroup>
      <SegmentedControl value={store.filter} onValueChange={setFilter} aria-label={t('Filter by status')} options={[
        {value:'all',label:`${t('All')} ${stats.all}`},{value:'via',label:`${t('Connected to this gateway')} ${stats.via}`},{value:'direct',label:`${t('Not connected to this gateway')} ${stats.direct}`},{value:'missing',label:`${t('Not detected')} ${stats.missing}`},
      ]}/>
    </div>
    {store.error&&<Alert variant='destructive' className='mx-4 mt-3'>{store.error}</Alert>}
    <div className='agent-body'>
      <Sidebar className='agent-rail p-0 gap-0' aria-label={t('Clients')}><NavList className='agent-rail-scroll'>
        {detected.length>0&&<><div className='agent-rail-label'>{t('Detected')}（{detected.length}）</div>{detected.map(client=><ClientRow key={client.id} client={client} active={client.id===store.selected}/>)}</>}
        {missing.length>0&&<><div className='agent-rail-label'>{t('Not detected')}（{missing.length}）</div>{missing.map(client=><ClientRow key={client.id} client={client} active={client.id===store.selected}/>)}</>}
        {!list.length&&<Empty>{store.loading?t('Detecting clients…'):t('No matching clients')}</Empty>}
      </NavList></Sidebar>
      <div className='agent-detail'>{client?<ClientDetail key={client.id} client={client}/>:<Empty>{t('Pick a client to start')}
        <Button variant='outline' size='sm' disabled={store.loading||Boolean(store.busy)} onClick={()=>void loadPanel()}>{store.loading?t('Checking…'):t('Recheck')}</Button></Empty>}</div>
    </div>
  </Card>
}

function ClientRow({client,active}:{client:AgentClient;active:boolean}) {
  const live = client.configurations.find(profile => profile.status === 'active' || profile.status === 'enabled')
  return <NavItem active={active} icon={<ClientAvatar client={client}/>} trailing={<StatusDot variant={client.viaGateway?'success':'muted'}/>}
    className={cn('agent-client',active&&'is-active',!client.detected&&'is-missing')} onClick={()=>selectClient(client.id)}>
    <span className='agent-client-main'><span className='agent-client-name'>{client.name}<span className='agent-client-ver'>{client.version}</span></span>
      <span className='agent-client-sub'>{live?.name||detectionLabel(client)}</span></span>
  </NavItem>
}

function ClientDetail({client}:{client:AgentClient}) {
  const store = React.useSyncExternalStore(subscribeAgentStore,getAgentStore,getAgentStore)
  const [pathOpen,setPathOpen] = React.useState(false)
  const [restore,setRestore] = React.useState<AgentOperation|null>(null)
  const [tab, setTab] = React.useState<string>('config')
  const [creating, setCreating] = React.useState<AgentProfile | null>(null)
  const [detailsOpen, setDetailsOpen] = React.useState(false)
  const dshTarget = client.configTargets?.find(target => target.path === (client.pathOverride || client.configPath)) || client.configTargets?.[0]
  const busy = Boolean(store.busy)||store.loading
  const addReason = client.readError || client.storeError || (!client.writable ? client.supportNote : '')
  const add = () => {
    setTab('config')
    setCreating({ ...blankProfile(client, store.gateway), name: client.supportsGateway ? 'Agent2API' : '', gatewayKeyId: store.gatewayKeys[0]?.id || '' })
  }
  return <>
    <div className='agent-dhead'><ClientAvatar client={client} size='lg'/><div className='agent-dtitle'><h3>{client.name}</h3>
      <div className='agent-meta'><span>{client.vendor}</span>{client.version ? <span>{client.version}</span> : client.id === 'dsh' ? <span>{t('Version not identified')}</span> : null}
        {!client.writable && <span>{detectionLabel(client)}</span>}
        {client.id === 'dsh' && dshTarget && <span>{dshTargetLabel(dshTarget.profile, dshTarget.label)}</span>}</div></div>
      <div className='agent-dhead-actions'><Button size='sm' disabled={busy || Boolean(addReason)} title={addReason || undefined} onClick={add}><AgentIcon name='plus' />{t('Add configuration')}</Button>
        <AgentActions label={t('Client options')} disabled={busy} actions={[
          { label: t('Recheck'), onClick: () => void loadPanel() },
          { label: client.id === 'dsh' ? t('Select configuration target') : t('Specify configuration path'), onClick: () => setPathOpen(true) },
          ...(client.applied.canUndo ? [{ label: t('Undo last application'), onClick: () => setRestore('restore') }] : []),
          ...(client.applied.canDisconnect ? [{ label: t('Disconnect from this gateway'), onClick: () => setRestore('disconnect') }] : []),
        ]} /></div></div>
    <SegmentedControl className='mt-3' value={tab} onValueChange={setTab} aria-label={t('Client management category')} options={[
      { value: 'config', label: t('Model settings') }, { value: 'prompt', label: t('Prompt') }, { value: 'skills', label: 'Skill' }, { value: 'mcp', label: 'MCP' },
    ]} />
    {tab !== 'config' ? <>
      <p className='agent-machine-note'>{t('Target: gateway host')}{store.hostName?` · ${store.hostName}`:''}</p>
      <AgentResourcesPanel key={`${tab}:${client.pathOverride}:${client.configPath}`} clientId={client.id} kind={tab as ResourceKind} capabilities={client.resources}
        request={callJson} t={t} notify={(text, bad) => toast(text, bad ? 'err' : 'ok')} />
    </> : <>
      {!client.detected && <p className='agent-detection-hint'>{t('No client program or configuration file found')}{client.writable ? ` · ${t('You can add configurations now and use them after installation')}` : ''}</p>}
      {!client.writable && !client.readError && <Alert variant='warning' className='mt-4'>{client.supportNote}</Alert>}
      <ProfileEditor client={client}/>
      {client.applied.error && <Alert variant='warning' className='mt-4'>{client.applied.error}</Alert>}
      {client.applied.pendingRecovery && <Alert variant='warning' className='mt-4'>{t('The last apply operation did not finish. Review the restore preview first.')}
        <Button variant='outline' size='sm' disabled={busy} onClick={() => setRestore('restore')}>{t('Undo last application')}</Button></Alert>}
      <Collapsible className='agent-client-details' open={detailsOpen} onOpenChange={setDetailsOpen}>
        <CollapsibleTrigger render={<Button variant='ghost' size='sm' />}><span className={detailsOpen ? 'rotate-180' : undefined}><AgentIcon name='chevron' /></span>{t('Configuration location and details')}</CollapsibleTrigger>
        <CollapsibleContent>
          <p className='agent-machine-note'>{t('Target: gateway host')}{store.hostName?` · ${store.hostName}`:''}</p>
          <code className='agent-resolved-path'>{client.resolvedPath}</code>
          {client.evidence.length > 0 && <ul className='agent-evidence'>{client.evidence.map((evidence, index) => <li key={index}>{evidence}</li>)}</ul>}
          <p className='agent-details-note'>{client.supportNote}</p>
          <p className='agent-details-note'>{t('Status comes from configuration files; existing sessions may need to reload')}</p>
          {client.applied.modifiedExternally && <p className='agent-details-note text-warning'>{t('The client file has external changes. Automatic restore will refuse to overwrite them.')}</p>}
        </CollapsibleContent>
      </Collapsible>
    </>}
    {pathOpen&&<PathDialog client={client} onClose={()=>setPathOpen(false)}/>}
    {creating&&<ProfileDialog client={client} profile={creating} creating onClose={()=>setCreating(null)}/>}
    {restore&&<ApplyDialog client={client} operation={restore} onClose={()=>setRestore(null)}/>}
  </>
}

export default AgentPage

const PAGE_SELECTOR = '.page[data-page="agent"]'
let mounted = false
function mount(): void {
  if (mounted) return
  const section = document.querySelector<HTMLElement>(PAGE_SELECTOR)
  if (!section) return
  mounted = true; section.replaceChildren()
  void import('react-dom/client').then(({createRoot})=>createRoot(section).render(<AgentPage/>))
}
mountOnPage('agent', PAGE_SELECTOR, mount)
declare global { interface Window { wbAgentPanel?:{load:(silent?:boolean)=>Promise<void>} } }
window.wbAgentPanel = {load:loadPanel}
