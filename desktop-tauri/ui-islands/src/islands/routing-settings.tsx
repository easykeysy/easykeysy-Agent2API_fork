import * as React from 'react'
import {
  Badge, Button, Checkbox, Dialog, DialogBody, DialogContent, DialogFooter, DialogHeader,
  DialogSection, DialogTitle, Empty, Field, FieldContent, FieldDescription, FieldLabel,
  Input, Label, SegmentedControl, Switch, confirmDialog,
} from '@ui'
import { t } from '../i18n'
import { cloneProfile, independentProfile, loadWorkspace, saveProfile } from './routing-state'
import { bridge, errorMessage, host, notify, type RouteSettings, type RoutingWorkspace } from './routing-types'
import { RouteIcon, RouteStateBadge } from './routing-ui'
import { displayNameOf } from './accounts-domain'
import { groupSortSummary } from './accounts-group-settings'

export function RoutingSettingsDialog({ workspace, onClose, protectionOnly = false }: { workspace: RoutingWorkspace; onClose(): void; protectionOnly?: boolean }) {
  const [draft, setDraft] = React.useState<RouteSettings>(() => ({ ...workspace.settings }))
  const [busy, setBusy] = React.useState(false)
  const [error, setError] = React.useState('')
  const update = <K extends keyof RouteSettings>(key: K, value: RouteSettings[K]) => setDraft(current => ({ ...current, [key]: value }))
  const save = async () => {
    if (!Number.isInteger(draft.routingBreakerThreshold) || draft.routingBreakerThreshold < 1 || draft.routingBreakerThreshold > 20
      || !Number.isInteger(draft.routingBreakerTimeoutSeconds) || draft.routingBreakerTimeoutSeconds < 10 || draft.routingBreakerTimeoutSeconds > 600
      || !Number.isInteger(draft.routingBreakerSuccessThreshold) || draft.routingBreakerSuccessThreshold < 1 || draft.routingBreakerSuccessThreshold > 10) {
      setError(t('Check the circuit breaker parameters: failure threshold 1–20, duration 10–600 seconds, recovery threshold 1–10.'))
      return
    }
    setBusy(true)
    try {
      const { routingStrategy, routingSticky, ...protection } = draft
      await bridge().setRoutingConfig(protectionOnly ? protection : draft)
      await loadWorkspace()
      notify(t('Global routing settings saved'))
      onClose()
    } catch (error) { setError(errorMessage(error)) }
    finally { setBusy(false) }
  }
  return <Dialog open onOpenChange={open => { if (!open && !busy) onClose() }}>
    <DialogContent className='w-[min(680px,calc(100vw-48px))]'>
      <DialogHeader><RouteIcon name='settings' /><DialogTitle>{protectionOnly ? t('Global protection settings') : t('Global routing settings')}</DialogTitle></DialogHeader>
      <DialogBody>
        <p className='text-xs text-muted-foreground'>{t('Protection settings are shared by all models. Independent model configurations can override participation, order, strategy, and stickiness.')}</p>
        {!protectionOnly ? <DialogSection>
          <Field><FieldLabel>{t('Routing strategy')}</FieldLabel><FieldContent>
            <SegmentedControl aria-label={t('Routing strategy')} value={draft.routingStrategy} disabled={busy} options={[
              { value: 'order', label: t('Strict order') }, { value: 'rotate', label: t('Load balancing') }, { value: 'provider', label: t('Per-provider rotation') },
            ]} onValueChange={value => update('routingStrategy', value)} />
            <FieldDescription>{draft.routingStrategy === 'order' ? t('Prefer the first account in the queue and fall back when it is unavailable.') : draft.routingStrategy === 'provider' ? t('Keep providers in priority order and rotate accounts within each provider.') : t('Rotate available accounts per request. With session stickiness enabled, the same session prefers its previous account.')}</FieldDescription>
          </FieldContent></Field>
          <Field><FieldLabel>{t('Session stickiness')}</FieldLabel><FieldContent>
            <SegmentedControl aria-label={t('Session stickiness')} value={draft.routingSticky} disabled={busy} options={[
              { value: 'auto', label: t('Auto') }, { value: 'session', label: t('In-session') },
              { value: 'turn', label: t('Per turn') }, { value: 'off', label: t('Close') },
            ]} onValueChange={value => update('routingSticky', value)} />
            <FieldDescription>{draft.routingSticky === 'auto' ? t('Released after five minutes of inactivity. Client session identifiers take priority; a content hash is used as a fallback.') : draft.routingSticky === 'session' ? t('Bindings last for up to six hours of inactivity and can be released at any time under Session bindings.') : t('Route each turn independently without retaining account bindings across requests.')}</FieldDescription>
          </FieldContent></Field>
        </DialogSection> : null}
        <DialogSection>
          <h3 className='text-sm font-semibold'>{t('Self-healing guards')}</h3>
          {([
            ['routingFailoverBackoff', t('Failure backoff'), t('Accounts that fail to forward rest temporarily, avoiding repeated requests to the same failing account.')],
            ['routingSink', t('Rate-limit demotion'), t('Previously rate-limited accounts move to the back temporarily and return after thirty minutes or a successful forward.')],
            ['routingBreakerEnabled', t('Circuit breaker'), t('Temporarily step aside after consecutive failures reach the threshold and participate again after recovery.')],
          ] as const).map(([key, label, hint]) => <Field key={key} orientation='horizontal'>
            <FieldContent><FieldLabel htmlFor={`routing-${key}`}>{label}</FieldLabel><FieldDescription>{hint}</FieldDescription></FieldContent>
            <Switch id={`routing-${key}`} checked={draft[key]} disabled={busy} onCheckedChange={value => update(key, value)} />
          </Field>)}
          {draft.routingBreakerEnabled ? <div className='grid grid-cols-3 gap-3'>
            {([
              ['routingBreakerThreshold', t('Failure threshold'), 1, 20],
              ['routingBreakerTimeoutSeconds', t('Breaker duration (seconds)'), 10, 600],
              ['routingBreakerSuccessThreshold', t('Recovery threshold'), 1, 10],
            ] as const).map(([key, label, min, max]) => <Field key={key}><FieldLabel>{label}</FieldLabel><Input type='number' min={min} max={max} step={1}
              disabled={busy} value={draft[key]} onChange={event => update(key, Number(event.currentTarget.value))} /></Field>)}
          </div> : null}
        </DialogSection>
        {error ? <p role='alert' className='text-destructive text-xs'>{error}</p> : null}
      </DialogBody>
      <DialogFooter><Button variant='outline' disabled={busy} onClick={onClose}>{t('Cancel')}</Button><Button disabled={busy} onClick={() => void save()}>{busy ? t('Saving…') : t('Save')}</Button></DialogFooter>
    </DialogContent>
  </Dialog>
}

export function ProviderRoutingDialog({ provider, workspace, onSort, onClose }: { provider: string; workspace: RoutingWorkspace; onSort(): void; onClose(): void }) {
  const label = workspace.providers.find(item => item.id === provider)?.label || provider
  const [draft, setDraft] = React.useState(() => workspace.model && !workspace.customized
    ? independentProfile(workspace.profile) : structuredClone(workspace.profile))
  const [busy, setBusy] = React.useState(false)
  const enabled = !draft.disabledProviders.includes(provider)
  const automatic = draft.providerScope === 'all'
  const accounts = workspace.accounts.filter(account => account.provider === provider)
  const statuses = new Map([...workspace.queue.ordered, ...workspace.queue.blocked].map(entry => [entry.id, entry]))
  const toggleAccount = (id: string, on: boolean) => setDraft(current => ({
    ...current, excludedAccounts: on ? current.excludedAccounts.filter(item => item !== id) : [...current.excludedAccounts.filter(item => item !== id), id],
  }))
  const save = async () => { setBusy(true); if (await saveProfile(draft)) onClose(); setBusy(false) }
  const remove = async () => {
    if (!await confirmDialog(t('Remove only from this route, retaining model and account settings. Other routes are unchanged.'), {
      title: t('Remove from this route'), confirmText: t('removed'),
    })) return
    const next = structuredClone(draft)
    if (!next.providers.includes(provider)) next.providers.push(provider)
    next.disabledProviders = [...new Set([...next.disabledProviders, provider])]
    setBusy(true)
    if (await saveProfile(next)) onClose()
    setBusy(false)
  }
  return <Dialog open onOpenChange={open => { if (!open && !busy) onClose() }}>
    <DialogContent className='w-[min(640px,calc(100vw-48px))]'>
      <DialogHeader><RouteIcon name='plugins' /><DialogTitle>{t('{name} · Provider routing settings', { name: label })}</DialogTitle></DialogHeader>
      <DialogBody>
        <Badge variant='brand' shape='tag' className='self-start'>{workspace.model || t('Global Configuration')}</Badge>
        <DialogSection>
          <div className='flex items-center justify-between gap-3'><h3 className='text-sm font-semibold'>{t('Consumption order')}</h3>
            <Button variant='outline' size='sm' disabled={busy} onClick={onSort}>{t('Edit global consumption rules')}</Button></div>
          <p className='text-xs text-muted-foreground'>{workspace.model && workspace.customized
            ? t('This model uses an independent order, unaffected by global automatic consumption sorting. Deleting the independent route makes it follow Accounts again.')
            : t('Shared with Accounts: {rules}. Rules such as credit expiry directly affect gateway routing.', { rules: groupSortSummary(workspace.sorting.rules[provider] || []) || t('Manual order only') })}</p>
        </DialogSection>
        {workspace.providers.find(item => item.id === provider)?.available === false ? <p className='text-xs text-warning'>{t('This provider currently has no available model binding. Its configuration is retained and will return automatically when the binding is restored in Model management.')}</p> : null}
        <Field orientation='horizontal'><FieldContent><FieldLabel>{t('Included in this route')}</FieldLabel><FieldDescription>{t('This only controls route participation, not whether the account is enabled.')}</FieldDescription></FieldContent>
          <Button variant='outline' disabled={busy} onClick={() => setDraft(current => ({ ...current,
            providers: [...new Set([...current.providers, provider])],
            disabledProviders: enabled ? [...new Set([...current.disabledProviders, provider])] : current.disabledProviders.filter(id => id !== provider),
          }))}>{enabled ? t('Remove from this route') : t('Add to route')}</Button></Field>
        <DialogSection><h3 className='text-sm font-semibold'>{t('Accounts participating in routing')}</h3>
          {!accounts.length ? <Empty>{t('This provider has no accounts. Add one on the Accounts page first.')}</Empty> : accounts.map(account => <div key={account.id} className='flex items-center gap-3 border-b border-hairline py-2 last:border-0'>
            <Checkbox checked={!draft.excludedAccounts.includes(account.id)} disabled={busy} aria-label={displayNameOf(account)} onCheckedChange={on => toggleAccount(account.id, on === true)} />
            <span className='min-w-0 flex-1 text-xs'>{displayNameOf(account)}</span><RouteStateBadge state={statuses.get(account.id)?.state || (account.enabled === false ? 'disabled' : 'usable')} />
          </div>)}
          <p className='text-xs text-muted-foreground'>{t('Participation here affects only this view. Account enablement, concurrency, and limiters belong to the account and are shared by all models.')}</p>
        </DialogSection>
        <div className='flex flex-wrap gap-2'><Button variant='outline' onClick={() => { onClose(); host().wbApp?.showPage('gateway') }}>{t('Model management')}</Button>
          <Button variant='outline' onClick={() => { onClose(); host().wbApp?.showPage('settings'); host().wbSettingsPanel?.showCategory('gateway') }}>{t('System prompt')}</Button>
          <Button variant='outline' onClick={() => { onClose(); host().wbApp?.showPage('settings'); host().wbSettingsPanel?.showCategory('security') }}>{t('Fingerprint sanitizing')}</Button></div>
      </DialogBody>
      <DialogFooter><Button variant='ghost' className='mr-auto text-destructive' disabled={busy || (automatic && !enabled)} onClick={() => void remove()}>{t('Remove from this route')}</Button><Button variant='outline' disabled={busy} onClick={onClose}>{t('Cancel')}</Button><Button disabled={busy} onClick={() => void save()}>{busy ? t('Saving…') : t('Save')}</Button></DialogFooter>
    </DialogContent>
  </Dialog>
}

export function AddRouteProviderDialog({ workspace, onClose }: { workspace: RoutingWorkspace; onClose(): void }) {
  const [picked, setPicked] = React.useState<string[]>([])
  const automatic = workspace.profile.providerScope === 'all' && (!workspace.model || workspace.customized)
  const options = workspace.providers.filter(provider => !workspace.profile.providers.includes(provider.id)
    || automatic && workspace.profile.disabledProviders.includes(provider.id))
  const [busy, setBusy] = React.useState(false)
  const save = async () => {
    const profile = cloneProfile()
    for (const id of picked) {
      if (!profile.providers.includes(id)) profile.providers.push(id)
      if (automatic) profile.disabledProviders = profile.disabledProviders.filter(provider => provider !== id)
      profile.accountOrder[id] = workspace.accounts.filter(account => account.provider === id)
        .sort((left, right) => Number(left.priority) - Number(right.priority)).map(account => account.id)
    }
    setBusy(true)
    if (await saveProfile(profile)) onClose()
    setBusy(false)
  }
  return <Dialog open onOpenChange={open => { if (!open && !busy) onClose() }}><DialogContent>
    <DialogHeader><DialogTitle>{automatic ? t('Restore routing providers') : t('Add routing provider')}</DialogTitle></DialogHeader><DialogBody>
      <p className='text-xs text-muted-foreground'>{automatic ? t('New available providers participate automatically. Use this dialog only to restore explicitly disabled providers.')
        : workspace.model ? t('Only providers with an enabled binding for this model are listed. This changes routing, not model bindings.') : t('Add to the fixed participation list. Future providers must still be added manually.')}</p>
      {options.length ? options.map(provider => <Label key={provider.id} className='flex items-center gap-3 rounded-md border border-border p-3'>
        <Checkbox checked={picked.includes(provider.id)} disabled={busy} onCheckedChange={on => setPicked(current => on ? [...current, provider.id] : current.filter(id => id !== provider.id))} />
        <span className='flex-1'>{provider.label}</span><Badge shape='tag' variant='secondary'>{t('{n} accounts', { n: provider.accountCount })}</Badge>
      </Label>) : <Empty>{automatic ? t('No providers need restoring') : t('All available providers are already in this route')}</Empty>}
    </DialogBody><DialogFooter><Button variant='outline' disabled={busy} onClick={onClose}>{t('Cancel')}</Button><Button disabled={busy || !picked.length} onClick={() => void save()}>{automatic ? t('Restore') : t('Add')}</Button></DialogFooter>
  </DialogContent></Dialog>
}
