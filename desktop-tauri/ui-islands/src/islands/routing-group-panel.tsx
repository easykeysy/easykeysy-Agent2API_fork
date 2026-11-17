import * as React from 'react'
import { Button, Collapsible, CollapsibleContent, CollapsibleTrigger, Dialog, DialogBody, DialogContent, DialogFooter, DialogHeader, DialogTitle, Empty, InputGroup, InputGroupAddon, InputGroupInput, Label, confirmDialog } from '@ui'
import { t } from '../i18n'
import type { AccountRecord, AccountSorting } from './accounts-shared'
import { displayNameOf } from './accounts-domain'
import { RoutingDragOverlay, reorderRoute, useRoutingDrag, type RoutingDrop } from './routing-drag'
import { RoutingGroupDialog, RoutingMemberAccounts, RoutingMemberDialog } from './routing-group-dialogs'
import { groupDirty, type RouteGroupEditor } from './routing-group-state'
import { RoutingSelect } from './routing-select'
import { RoutingActions } from './routing-canvas-controls'
import { RoutingCanvasLayout, RoutingDraftFooter, RoutingRefreshError } from './routing-canvas-layout'
import { useRoutingCanvasActivity } from './routing-canvas-activity'
import { RoutingMemberCard } from './routing-member-card'
import { routingAccountSections, routingCollapsedSections, useRoutingExpansion } from './routing-account-sections'
import { IconButton, RouteIcon } from './routing-ui'
import { notify, type RouteGroup, type RouteMember, type RoutingWorkspace } from './routing-types'

export function RoutingGroupPanel({ editor, accounts, lastUsed, reserved, baseUrl, sorting, now, refreshToken, paused, onWorkspace, onAccount, onSort }: {
  editor: RouteGroupEditor; accounts: AccountRecord[]; lastUsed?: Record<string, number>; reserved: string[]; baseUrl: string; sorting?: AccountSorting | null; now: number
  refreshToken: number; paused: boolean; onWorkspace(workspace: RoutingWorkspace): void
  onAccount(id: string): void; onSort(provider: string): void
}) {
  const [dialog, setDialog] = React.useState<'edit' | 'add' | 'example' | null>(null)
  const [accountMember, setAccountMember] = React.useState<string | null>(null)
  const { setOpened, sectionOpen, setSectionOpen, search, setSearch, query, isOpen } = useRoutingExpansion()
  const group = editor.draft?.group
  const live = useRoutingCanvasActivity(Boolean(group), editor.draft?.baseline?.model || group?.model || '', {
    groupId: group?.id, workspaceModel: '', paused: paused || editor.busy || editor.loading, refreshToken, onWorkspace,
  })
  const { activity } = live
  accounts = live.workspace?.accounts || accounts
  if (live.workspace && (!sorting || live.workspace.sorting.updatedAt >= sorting.updatedAt)) sorting = live.workspace.sorting
  const locked = editor.busy || editor.loading
  const update = (mutate: (group: RouteGroup) => void) => editor.update(mutate)
  const drop = ({ kind, provider, id, target, after }: RoutingDrop) => update(next => {
    if (kind === 'account') {
      const member = next.members.find(member => member.id === provider)
      if (member) reorderRoute(member.accounts, id, target, after)
      return
    }
    const ids = next.members.filter(member => member.included).map(member => member.id)
    reorderRoute(ids, id, target, after)
    next.members = [...ids.map(id => next.members.find(member => member.id === id)!), ...next.members.filter(member => !member.included)]
  })
  const drag = useRoutingDrag(!locked && !query, `${group?.id || ''}:${search}`, drop)
  if (!group || !editor.draft) return <Empty>{t('Select a route entry')}</Empty>
  const active = group.members.filter(member => member.included)
  const sections = routingAccountSections(group.members, accounts, member => member.provider, member => member.included)
  const orderingPeers = (id: string) => (Object.values(sections).find(members => members.some(member => member.id === id)) || []).filter(member => member.included)
  const dirty = groupDirty(editor.draft)
  const label = (provider: string) => editor.data?.catalog.find(choice => choice.provider === provider)?.providerLabel || provider
  const name = (member: RouteMember) => editor.data?.catalog.find(choice => choice.provider === member.provider && choice.model.toLowerCase() === member.model.toLowerCase())?.name || member.model
  const editMember = (id: string, mutate: (member: RouteMember) => void) => update(next => { const member = next.members.find(item => item.id === id); if (member) mutate(member) })
  const move = (id: string, direction: number) => {
    const peers = orderingPeers(id), target = peers[peers.findIndex(member => member.id === id) + direction]
    if (target) drop({ kind: 'member', provider: '', id, target: target.id, after: direction > 0 })
  }
  const moveAccount = (member: RouteMember, id: string, direction: number) => {
    const target = member.accounts[member.accounts.indexOf(id) + direction]
    if (target) editMember(member.id, next => reorderRoute(next.accounts, id, target, direction > 0))
  }
  const removeAll = async () => {
    if (await confirmDialog(t('Remove only from this route. Accounts and other routes stay unchanged. After saving, this entry has no available route.'), {
      title: t('Remove all from this route'), confirmText: t('Remove all'),
    })) update(next => { next.members.forEach(member => { member.included = false }) })
  }
  const copy = async (value: string) => {
    try { await navigator.clipboard.writeText(value); notify(t('Copied')) }
    catch { notify(t('Copy failed. Select and copy manually.'), 'error') }
  }
  const request = JSON.stringify({ model: group.model, messages: [{ role: 'user', content: 'Hello' }], stream: true }, null, 2)
  const snippet = `POST ${baseUrl.replace(/\/$/, '')}/v1/chat/completions\nAuthorization: Bearer <YOUR_GATEWAY_KEY>\nContent-Type: application/json\n\n${request}`
  const matches = (member: RouteMember) => !query || `${name(member)} ${label(member.provider)} ${member.model}`.toLowerCase().includes(query)
    || accounts.some(account => account.provider === member.provider && displayNameOf(account).toLowerCase().includes(query))
  const visible = sections.active.filter(matches)
  const expandable = [...sections.active, ...routingCollapsedSections.flatMap(({ key }) => sectionOpen[key] ? sections[key] : [])].filter(matches)
  const allOpen = expandable.length > 0 && expandable.every(member => isOpen(member.id))
  const renderMember = (member: RouteMember) => <RoutingMemberCard key={member.id} member={member} accounts={accounts} name={name(member)} label={label(member.provider)}
    index={active.indexOf(member)} first={orderingPeers(member.id)[0]?.id === member.id} last={orderingPeers(member.id).at(-1)?.id === member.id}
    open={isOpen(member.id)} locked={locked} query={query} drag={drag} activity={activity}
    onOpen={open => setOpened(current => ({ ...current, [member.id]: open }))} onJoin={on => editMember(member.id, next => { next.included = on })}
    onMove={direction => move(member.id, direction)} onAccounts={() => setAccountMember(member.id)} onMoveAccount={(id, direction) => moveAccount(member, id, direction)}
    onToggleAccount={id => editMember(member.id, next => { next.accounts = next.accounts.includes(id) ? next.accounts.filter(key => key !== id) : [...next.accounts, id] })}
    onAccount={onAccount} sorting={sorting} now={now} onSort={() => onSort(member.provider)} />
  const selectedAccountMember = group.members.find(member => member.id === accountMember)
  const members = <section className='routing-groups-column'>
    <div className='routing-column-head'><RouteIcon name='plugins' size={14} /><b>{t('成员模型')}</b><span>{t('{n} participating', { n: active.length })}</span>
      <div className='right'><Button variant='ghost' size='sm' disabled={!expandable.length || Boolean(query)} onClick={() => setOpened(current => ({ ...current, ...Object.fromEntries(expandable.map(member => [member.id, !allOpen])) }))}>
        {allOpen ? t('Collapse all') : t('Expand all')}</Button>
        <RoutingActions label={t('Route participation actions')} actions={[
          { label: t('Add model'), icon: 'plus', disabled: locked, onClick: () => setDialog('add') },
          { label: t('Remove all'), disabled: locked || !active.length, onClick: () => void removeAll() },
        ]} />
      </div>
    </div>
    <InputGroup className='routing-config-search'><InputGroupInput value={search} onChange={event => setSearch(event.currentTarget.value)}
      placeholder={t('Search providers or models')} aria-label={t('Search providers or models')} /><InputGroupAddon><RouteIcon name='search' size={14} /></InputGroupAddon></InputGroup>
    <p className='routing-groups-hint'>{t('Drag a handle or use Alt + arrow keys to reorder. More actions are in the menu on the right.')}</p>
    <RoutingRefreshError error={live.error} onRetry={live.refresh} />
    <div className='routing-config-listcol' data-route-scroll>
      <div className='routing-config-participants' data-route-list>{visible.map(renderMember)}</div>
      {!active.length ? <Empty><b>{t('No participating models yet')}</b><span>{t('An empty route can be saved, but requests will fail explicitly without falling back globally.')}</span></Empty> : null}
      {routingCollapsedSections.map(({ key, label }) => {
        const members = sections[key].filter(matches)
        return members.length ? <Collapsible key={key} className='routing-config-excluded' open={Boolean(sectionOpen[key]) || Boolean(query)}
          onOpenChange={open => setSectionOpen(current => ({ ...current, [key]: open }))}>
          <CollapsibleTrigger className='routing-excluded-trigger'><RouteIcon name='chevronDown' size={13} />{label} · {members.length}</CollapsibleTrigger>
          <CollapsibleContent data-route-list>{members.map(renderMember)}</CollapsibleContent>
        </Collapsible> : null
      })}
      {query && !group.members.some(matches) ? <Empty>{t('No matching candidates')}</Empty> : null}
      <Button variant='dashed' className='routing-add-member' disabled={locked} onClick={() => setDialog('add')}><RouteIcon name='plus' size={14} />{t('添加模型')}</Button>
    </div>
  </section>
  return <>
    <RoutingCanvasLayout title={t('Custom route groups')} groups={members} note={t('Only affects this member, not account enablement or other models. New accounts must be added manually.')}>
      <div className='routing-config-card' data-route-source data-route-running={activity.running > 0 ? '' : undefined}>
        <div className='routing-config-card-scroll'>
          <header className='routing-config-head'><div><span className='routing-config-eyebrow'>{t('Mixed-model group')}</span><h2>{group.name}</h2></div>
            <RoutingActions label={t('Route configuration actions')} actions={[
              { label: t('Edit route group'), icon: 'settings', disabled: locked, onClick: () => setDialog('edit') },
              { label: t('Request example'), disabled: locked, onClick: () => setDialog('example') },
              { label: t('Delete route group'), destructive: true, disabled: locked, onClick: () => void editor.remove() },
            ]} />
          </header>
          {group.description ? <p className='routing-config-note'>{group.description}</p> : null}
          <div className='routing-config-alias'><span>{t('Public model ID')}</span><code title={group.model}>{group.model}</code>
            <IconButton name='copy' title={t('Copy')} onClick={() => void copy(group.model)} /></div>
          <div className='routing-config-rules'>
            <div className='routing-rule-field'><Label>{t('Routing strategy')}</LRouting strategy')}</Label><RoutingSelect label={t('Routing strategy')} value={group.strategy} disabled={locked}
              options={[[ 'order', t('Strict order') ], ['rotate', t('Load balancing')], ['provider', t('Rotate within members')]]} onChange={value => update(next => { next.strategy = value })} /></div>
            <div className='routing-rule-field'><Label>{t('Session bindings')}</Label><RoutingSelect label={t('Session bindings')} value={group.sticky} disabled={locked}
              options={[[ 'auto', t('Auto') ], ['session', t('In-session')], ['turn', t('Per turn')], ['off', t('Close')]]} onChange={value => update(next => { next.sticky = value })} /></div>
          </div>
          <p className='routing-config-note'>{group.sticky === 'auto' || group.sticky === 'session' ? t('Prefer the same model and account within a session; fall back when unavailable.') : t('Route each turn independently without retaining account bindings across requests.')}</p>
          <Button variant='link' size='sm' className='routing-group-example' onClick={() => setDialog('example')}>{t('Request example')}</Button>
        </div>
        <RoutingDraftFooter dirty={dirty} busy={editor.busy} disabled={locked} error={editor.error} onDiscard={editor.discard} onSave={() => void editor.save()} />
      </div>
    </RoutingCanvasLayout>
    <RoutingDragOverlay state={drag.state} />
    {dialog === 'edit' ? <RoutingGroupDialog group={group} reserved={reserved} onSave={group => update(next => Object.assign(next, group))} onClose={() => setDialog(null)} /> : null}
    {dialog === 'add' ? <RoutingMemberDialog choices={(editor.data?.catalog || []).filter(choice => choice.model.toLowerCase() !== group.model.toLowerCase()
      && !Object.values(editor.drafts).some(draft => draft.group.model.toLowerCase() === choice.model.toLowerCase()))} members={group.members} accounts={accounts} lastUsed={lastUsed} onAdd={members => update(next => { next.members.push(...members) })} onClose={() => setDialog(null)} /> : null}
    {selectedAccountMember ? <RoutingMemberAccounts member={selectedAccountMember} accounts={accounts} onSave={ids => editMember(selectedAccountMember.id, member => { member.accounts = ids })} onClose={() => setAccountMember(null)} /> : null}
    {dialog === 'example' ? <Dialog open onOpenChange={open => { if (!open) setDialog(null) }}><DialogContent>
      <DialogHeader><DialogTitle>{t('Request example')}</DialogTitle></DialogHeader><DialogBody><p className='text-sm text-muted-foreground'>{t('After saving, use this model ID in your client with your own gateway key.')}</p><pre className='routing-group-code'>{snippet}</pre></DialogBody>
      <DialogFooter><Button variant='outline' onClick={() => setDialog(null)}>{t('Close')}</Button><Button onClick={() => void copy(snippet)}>{t('Copy')}</Button></DialogFooter>
    </DialogContent></Dialog> : null}
  </>
}
