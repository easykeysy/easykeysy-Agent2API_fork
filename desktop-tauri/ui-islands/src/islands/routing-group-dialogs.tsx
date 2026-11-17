import * as React from 'react'
import { Button, Dialog, DialogBody, DialogContent, DialogFooter, DialogHeader, DialogTitle, Empty } from '@ui'
import { t } from '../i18n'
import { displayNameOf, providerOf } from './accounts-domain'
import type { AccountRecord } from './accounts-shared'
import { RouteIcon } from './routing-ui'
import type { RouteGroup, RouteMember, RouteModelChoice } from './routing-types'
import { createRouteMembers, routeGroupFieldsError, routeModelKey, RoutingGroupForm, ROUTING_MEMBER_LIMIT, type RoutingGroupFields } from './routing-group-form'
import { RoutingModelPicker } from './routing-model-picker'

export function RoutingGroupDialog({ group, reserved, onSave, onClose }: {
  group: RouteGroup; reserved: string[]; onSave(group: RouteGroup): void; onClose(): void
}) {
  const [fields, setFields] = React.useState<RoutingGroupFields>({ name: group.name, model: group.model, description: group.description })
  const [error, setError] = React.useState('')
  const save = () => {
    const message = routeGroupFieldsError(fields, reserved, group.model)
    if (message) { setError(message); return }
    onSave({ ...group, name: fields.name.trim(), model: fields.model.trim().toLowerCase(), description: fields.description.trim() })
    onClose()
  }
  return <Dialog open onOpenChange={open => { if (!open) onClose() }}><DialogContent className='w-[min(600px,calc(100vw-40px))]'>
    <DialogHeader><RouteIcon name='route' /><DialogTitle>{t('Edit route group')}</DialogTitle></DialogHeader>
    <DialogBody>
      <p className='routing-config-note'>{t('Combine models from different providers into one entry. Clients request its public model ID.')}</p>
      <RoutingGroupForm value={fields} onChange={setFields} />
      {error ? <p className='routing-config-error' role='alert'>{error}</p> : null}
    </DialogBody><DialogFooter><Button variant='outline' onClick={onClose}>{t('Cancel')}</Button><Button onClick={save}>{t('Apply to draft')}</Button></DialogFooter>
  </DialogContent></Dialog>
}

export function RoutingMemberDialog({ choices, members, accounts, lastUsed, onAdd, onClose }: {
  choices: RouteModelChoice[]; members: RouteMember[]; accounts: AccountRecord[]; lastUsed?: Record<string, number>; onAdd(members: RouteMember[]): void; onClose(): void
}) {
  const [selected, setSelected] = React.useState<string[]>([])
  const options = choices.filter(choice => !members.some(member => member.provider === choice.provider && member.model.toLowerCase() === choice.model.toLowerCase()))
  const byKey = new Map(options.map(choice => [routeModelKey(choice), choice]))
  const picked = selected.flatMap(key => { const choice = byKey.get(key); return choice ? [choice] : [] })
  const limit = Math.max(0, ROUTING_MEMBER_LIMIT - members.length)
  const add = () => {
    if (!picked.length || picked.length > limit) return
    onAdd(createRouteMembers(picked, accounts))
    onClose()
  }
  return <Dialog open onOpenChange={open => { if (!open) onClose() }}><DialogContent className='routing-create-dialog'>
    <DialogHeader><DialogTitle>{t('Add participating models')}</DialogTitle></DialogHeader><DialogBody>
      <p className='routing-config-note'>{t('Select multiple models across providers. Different models from the same provider can also be included.')}</p>
      <RoutingModelPicker choices={options} accounts={accounts} lastUsed={lastUsed} selected={selected} onChange={setSelected} limit={limit} />
      <p className='routing-config-note'>{t('Only chat models from the actual catalog are listed. Model mappings remain unchanged.')}</p>
    </DialogBody><DialogFooter><Button variant='outline' onClick={onClose}>{t('Cancel')}</Button><Button disabled={!picked.length || picked.length > limit} onClick={add}>{t('Add')} · {picked.length}</Button></DialogFooter>
  </DialogContent></Dialog>
}

export function RoutingMemberAccounts({ member, accounts, onSave, onClose }: {
  member: RouteMember; accounts: AccountRecord[]; onSave(ids: string[]): void; onClose(): void
}) {
  const [selected, setSelected] = React.useState(member.accounts)
  const available = accounts.filter(account => providerOf(account) === member.provider)
  const order = [...new Set([...selected, ...available.map(account => account.id)])]
  const move = (id: string, direction: number) => setSelected(current => {
    const next = [...current], index = next.indexOf(id), to = index + direction
    if (index >= 0 && to >= 0 && to < next.length) [next[index], next[to]] = [next[to], next[index]]
    return next
  })
  return <Dialog open onOpenChange={open => { if (!open) onClose() }}><DialogContent className='w-[min(680px,calc(100vw-40px))]'>
    <DialogHeader><DialogTitle>{t('Participating accounts')} · {member.model}</DialogTitle></DialogHeader><DialogBody>
      <p className='text-sm text-muted-foreground'>{t('Only affects this member, not account enablement or other models. New accounts must be added manually.')}</p>
      <div className='routing-model-picker'>{order.map(id => {
        const account = available.find(item => item.id === id), on = selected.includes(id), index = selected.indexOf(id)
        return <div className='routing-group-picker-row' key={id}>
          <div><b>{account ? displayNameOf(account) : t('Account removed')}</b><small>{account?.enabled === false ? t('Account disabled') : on ? t('Included in this route') : t('Not in this route')}</small></div>
          <Button variant='ghost' size='icon' disabled={!on || index === 0} aria-label={t('Raise priority')} onClick={() => move(id, -1)}><RouteIcon name='arrowUp' /></Button>
          <Button variant='ghost' size='icon' disabled={!on || index === selected.length - 1} aria-label={t('Lower priority')} onClick={() => move(id, 1)}><RouteIcon name='arrowDown' /></Button>
          <Button variant={on ? 'ghost' : 'outline'} size='sm' disabled={!account && !on} onClick={() => setSelected(current => on ? current.filter(value => value !== id) : [...current, id])}>{on ? t('Remove from this route') : t('Add to route')}</Button>
        </div>
      })}{!order.length ? <Empty>{t('This provider has no accounts. Add one on the Accounts page first.')}</Empty> : null}</div>
    </DialogBody><DialogFooter><Button variant='outline' onClick={onClose}>{t('Cancel')}</Button><Button onClick={() => { onSave(selected); onClose() }}>{t('Apply to draft')}</Button></DialogFooter>
  </DialogContent></Dialog>
}
