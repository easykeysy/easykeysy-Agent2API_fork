import * as React from 'react'
import {
  Button, Collapsible, CollapsibleContent, CollapsibleTrigger, Dialog, DialogBody, DialogContent,
  DialogDescription, DialogFooter, DialogHeader, DialogTitle, Spinner,
} from '@ui'
import { t } from '../i18n'
import { bridge, errorMessage, type ReplayRow, type RouteGroup, type RoutingWorkspace } from './routing-types'
import { traceProvider } from './routing-trace-model'
import { execScopeLabel } from './routing-exec-model'
import { RouteIcon, stickyLabel, strategyLabel, timeText } from './routing-ui'

type CurrentConfig = { workspace: RoutingWorkspace; group?: RouteGroup; missing: boolean }
const routeStrategy = (strategy?: string, group = false) => strategy ? group && strategy === 'provider' ? t('Rotate within members') : strategyLabel(strategy) : t('Unrecorded')
const providerScope = (scope?: string) => scope === 'all' ? t('All providers') : scope === 'selected' ? t('Selected providers') : t('Unrecorded')

function SnapshotMembers({ row }: { row: ReplayRow }) {
  const members = row.route?.configSnapshot?.members
  if (!members?.length) return null
  return <section className='routing-config-detail-section'>
    <h3>{t('Member models at request time')}</h3>
    <div className='routing-snapshot-members'>{members.map(member => <div key={member.id} className='routing-snapshot-member'>
      <code>{member.model}</code><span>{traceProvider(member.provider)} · {member.included ? t('Included in this route') : t('Not in this route')} · {t('{n} accounts', { n: member.accountCount })}</span>
    </div>)}</div>
  </section>
}

function RoutingSnapshotDialog({ row, onClose, onConfig }: { row: ReplayRow; onClose(): void; onConfig(model: string): void }) {
  const [current, setCurrent] = React.useState<CurrentConfig | null>(null)
  const [error, setError] = React.useState('')
  const [revision, reload] = React.useReducer(value => value + 1, 0)
  const [loading, setLoading] = React.useState(true)
  const snapshot = row.route?.configSnapshot
  const scope = snapshot?.scope ?? row.route?.scope
  const requestModel = row.clientModel || row.model || ''
  const model = scope ?? requestModel
  const groupModel = scope || requestModel
  const key = `${row.id}:${row.ts}`
  React.useEffect(() => {
    let alive = true
    setCurrent(null); setError(''); setLoading(true)
    void Promise.all([bridge().getRoutingWorkspace(model || undefined), bridge().getRoutingGroups()]).then(([workspace, groups]) => {
      if (!alive) return
      if (!workspace?.settings || !workspace?.profile || !Array.isArray(groups?.groups)) throw new Error(t('The routing API returned incomplete data'))
      const group = groups.groups.find(entry => snapshot?.groupId ? entry.group.id === snapshot.groupId
        : Boolean(model && entry.group.model.toLowerCase() === groupModel.toLowerCase()))?.group
      setCurrent({ workspace, group, missing: Boolean(snapshot?.groupId && !group) })
    }).catch(failure => { if (alive) setError(errorMessage(failure)) }).finally(() => { if (alive) setLoading(false) })
    return () => { alive = false }
  }, [key, model, groupModel, snapshot?.groupId, revision])

  const scopeText = !current ? '' : current.missing ? t('Route group deleted') : current.group
    ? `${t('Custom route groups')} · ${current.group.name || current.group.model}`
    : current.workspace.customized ? `${t('Independent model configuration')} · ${current.workspace.model || model}` : t('Inherits global configuration')
  const available = current && !current.missing
  const currentText = (text?: string) => loading ? t('Loading…') : available ? text || '—' : '—'
  const rows = [
    { label: t('Configuration scope'), before: execScopeLabel(row), after: loading ? t('Loading…') : scopeText || '—' },
    { label: t('Routing strategy'), before: routeStrategy(snapshot?.strategy || row.route?.strategy, Boolean(snapshot?.groupId)),
      after: currentText(routeStrategy(current?.group?.strategy || current?.workspace.settings.routingStrategy, Boolean(current?.group))) },
    { label: t('Session bindings'), before: snapshot?.sticky || row.route?.sticky ? stickyLabel(snapshot?.sticky || row.route?.sticky) : t('Unrecorded'),
      after: currentText(stickyLabel(current?.group?.sticky || current?.workspace.settings.routingSticky)) },
    { label: t('Provider scope'), before: providerScope(snapshot?.providerScope), after: currentText(current?.group ? t('Selected providers') : providerScope(current?.workspace.profile.providerScope)) },
  ]
  const session = row.route?.sessionId || row.route?.stickyId
  const editScope = current?.group?.model ?? (current?.workspace.customized ? current.workspace.model || model : '')
  return <Dialog open onOpenChange={open => { if (!open) onClose() }}>
    <DialogContent className='routing-config-detail-dialog'>
      <DialogHeader><DialogTitle>{t('Configuration then and now')}</DialogTitle></DialogHeader>
      <DialogBody>
        <DialogDescription className='routing-exec-note'>{t('History is read-only. Configuration changes do not alter this request snapshot.')}</DialogDescription>
        <div className='routing-config-compare' role='table' aria-label={t('Configuration then and now')}>
          <div className='routing-config-compare-row is-heading' role='row'><span role='columnheader'>{t('Setting')}</span><span role='columnheader'>{t('At request time')}</span><span role='columnheader'>{t('Current configuration')}</span></div>
          {rows.map(item => <div key={item.label} className='routing-config-compare-row' role='row' data-same={item.before === item.after ? '' : undefined}>
            <span role='rowheader'>{item.label}</span><span role='cell'>{item.before}</span><span role='cell'>{item.after}</span>
          </div>)}
        </div>
        {loading ? <div className='routing-config-detail-loading'><Spinner />{t('Loading current configuration')}</div> : null}
        {error ? <p className='routing-config-detail-error' role='alert'>{error}<Button variant='link' size='xs' onClick={reload}>{t('Retry')}</Button></p> : null}
        {current?.missing ? <p className='routing-exec-note'>{t('The route group used by this request has been deleted; its saved snapshot is still available')}</p> : null}
        <section className='routing-config-detail-section'>
          <h3>{t('Configuration at request time')}</h3>
          <dl className='routing-config-detail-facts'>
            <dt>{t('Captured at')}</dt><dd>{snapshot?.capturedAt != null ? timeText(snapshot.capturedAt) : t('Unrecorded')}</dd>
            <dt>{t('Candidate pool')}</dt><dd>{snapshot?.providerCount != null ? `${t('{n} providers', { n: snapshot.providerCount })} · ` : ''}{snapshot?.accountCount != null ? t('{n} accounts', { n: snapshot.accountCount }) : t('Unrecorded')}</dd>
            {session ? <><dt>{t('Sessions')}</dt><dd className='routing-exec-mono'>{session}</dd></> : null}
            {row.route?.sessionSource ? <><dt>{t('Session source')}</dt><dd>{row.route.sessionSource}</dd></> : null}
            {row.route?.stickyHit != null ? <><dt>{t('Binding hits')}</dt><dd>{row.route.stickyHit ? t('Yes') : t('No')}</dd></> : null}
          </dl>
        </section>
        <SnapshotMembers row={row} />
      </DialogBody>
      <DialogFooter>
        <Button variant='ghost' size='sm' disabled={!available} onClick={() => { onClose(); onConfig(editScope) }}>{t('View current configuration')}</Button>
        <Button variant='outline' size='sm' onClick={onClose}>{t('Close')}</Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
}

export function RoutingExecSnapshot({ row, onConfig }: { row: ReplayRow; onConfig(model: string): void }) {
  const [open, setOpen] = React.useState(false)
  React.useEffect(() => { setOpen(false) }, [row.id, row.ts])
  const snapshot = row.route?.configSnapshot
  const strategy = snapshot?.strategy || row.route?.strategy
  const sticky = snapshot?.sticky || row.route?.sticky
  return <section className='routing-exec-section'>
    <div className='routing-exec-section-head'><h3>{t('Configuration at request time')}</h3><Button variant='link' size='xs' onClick={() => setOpen(true)}>{t('View configuration details')}</Button></div>
    <dl className='routing-exec-definitions'>
      <div><dt>{t('Configuration scope')}</dt><dd>{execScopeLabel(row)}</dd></div>
      <div><dt>{t('Routing strategy')}</dt><dd>{routeStrategy(strategy, Boolean(snapshot?.groupId))}</dd></div>
      <div><dt>{t('Session bindings')}</dt><dd>{sticky ? stickyLabel(sticky) : t('Unrecorded')}</dd></div>
      {row.upstreamModel ? <div className='is-wide'><dt>{t('Upstream model')}</dt><dd className='routing-exec-mono'>{row.upstreamModel}</dd></div> : null}
    </dl>
    {snapshot?.members?.length ? <Collapsible className='routing-exec-member-toggle'>
      <CollapsibleTrigger className='routing-detail-toggle'><RouteIcon name='chevronDown' />{t('Member models')} · {snapshot.members.length}</CollapsibleTrigger>
      <CollapsibleContent><SnapshotMembers row={row} /></CollapsibleContent>
    </Collapsible> : null}
    {open ? <RoutingSnapshotDialog row={row} onClose={() => setOpen(false)} onConfig={onConfig} /> : null}
  </section>
}
