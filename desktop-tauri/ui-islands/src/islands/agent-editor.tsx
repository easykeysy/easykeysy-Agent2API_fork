import * as React from 'react'
import { Alert, Button, Empty, StatusDot, cn } from '@ui'
import { t } from '../i18n'
import { confirm, errorMessage, getAgentStore, removeConfiguration, subscribeAgentStore, toast } from './agent-state'
import { kindLabel, protocolLabel } from './agent-model'
import { ProfileDialog } from './agent-profile-dialog'
import { ApplyDialog } from './agent-preview'
import { AgentActions, AgentIcon } from './agent-actions'
import type { AgentClient, AgentOperation, AgentProfile } from './agent-types'

export function ProfileEditor({ client }: { client: AgentClient }) {
  const store = React.useSyncExternalStore(subscribeAgentStore, getAgentStore, getAgentStore)
  const [editor, setEditor] = React.useState<AgentProfile | null>(null)
  const [applying, setApplying] = React.useState<{ profile: AgentProfile; operation: AgentOperation } | null>(null)
  const disabled = Boolean(store.busy) || store.loading || Boolean(client.storeError)
  const profiles = client.configurations
  const remove = async (profile: AgentProfile) => {
    if (profile.liveId) { setApplying({ profile, operation: 'config-delete' }); return }
    if (!await confirm(t('Delete configuration?'), t('This configuration is not in use in the client. Deleting it will not change the current configuration.'), t('Delete'))) return
    try { await removeConfiguration(client, profile); toast(t('Configuration deleted')) }
    catch (error) { toast(errorMessage(error), 'err') }
  }
  return <>
    <div className='agent-configurations' aria-label={t('Configurations')}>
      {client.readError && <Alert variant='destructive'>{client.readError}</Alert>}
      {client.storeError && <Alert variant='destructive'>{client.storeError}</Alert>}
      {!client.readError && !client.storeError && !profiles.length && <Empty className='agent-configuration-empty'>
        <span className='text-sm font-medium'>{t('No profiles yet')}</span>
        <p>{client.writable ? t('Use Add configuration at the top right to add a gateway or another service') : t('No recognized client configuration found')}</p>
      </Empty>}
      {profiles.map(profile => {
        const active = profile.status === 'active' || profile.status === 'enabled'
        const status = profile.status === 'active' ? t('In use') : profile.status === 'enabled' ? t('Enabled')
          : profile.status === 'disabled' ? t('Disabled') : ''
        const name = profile.name || profile.id
        const duplicateName = profiles.some(other => other !== profile && (other.name || other.id) === name)
        const models = profile.models.length ? profile.models.map(model => model.id) : profile.model ? [profile.model] : []
        const actionLabel = profile.status === 'disabled' ? t('enabled') : client.supportsDefault ? t('Use this configuration') : t('Add to client')
        return <div key={`${profile.liveId || 'saved'}:${profile.id}`} className={cn('agent-configuration-row', active && 'is-current')}>
          <span className='agent-configuration-mark' aria-hidden='true'>{profile.kind === 'gateway' ? 'A2' : name.slice(0, 2).toUpperCase()}</span>
          <div className='agent-configuration-main'>
            <div className='agent-configuration-title'><span title={name}>{name}</span>
              {duplicateName && <code>{profile.sourceId || profile.id}</code>}</div>
            <div className='agent-configuration-meta'><span className='agent-configuration-url' title={profile.baseUrl}>{profile.baseUrl || t('Client default endpoint')}</span>
              <span>{kindLabel(profile.kind)}</span><span>{protocolLabel(profile.protocol)}</span></div>
            {models.length > 0 && <p className='agent-configuration-models' title={models.join(' · ')}>{models.slice(0, 3).join(' · ')}{models.length > 3 ? ` · +${models.length - 3}` : ''}</p>}
            {profile.note && <p className='agent-configuration-note'>{profile.note}</p>}
            {profile.validationError && <p className='agent-configuration-error'>{t('Complete this configuration before using it')} · {profile.validationError}</p>}
          </div>
          <div className='agent-configuration-actions'>
            {status && <span className={cn('agent-configuration-status', active && 'is-current')} title={active ? t('Status comes from configuration files; existing sessions may need to reload') : undefined}>
              <StatusDot variant={active ? 'success' : 'muted'} />{status}</span>}
            {!active && profile.editable && <Button variant='outline' size='sm' disabled={disabled || Boolean(profile.validationError)}
              onClick={() => setApplying({ profile, operation: 'config-use' })}>{actionLabel}</Button>}
            {profile.editable && <Button variant='ghost' size='icon-sm' disabled={disabled} title={t('Edit')} aria-label={t('Edit: {name}', { name })}
              onClick={() => setEditor(profile)}><AgentIcon name='edit' /></Button>}
            {profile.deletable && <AgentActions label={t('Configuration actions')} disabled={disabled || Boolean(client.readError)} actions={[
              ...(client.supportsToggle && profile.status === 'enabled' ? [{ label: t('Disabled'), onClick: () => setApplying({ profile, operation: 'config-disable' }) }] : []),
              { label: t('Delete'), destructive: true, onClick: () => void remove(profile) },
            ]} />}
          </div>
        </div>
      })}
    </div>
    {editor && <ProfileDialog key={`${client.id}:${editor.id}`} client={client} profile={editor} creating={false} onClose={() => setEditor(null)} />}
    {applying && <ApplyDialog client={client} profile={applying.profile} operation={applying.operation} onClose={() => setApplying(null)} />}
  </>
}
