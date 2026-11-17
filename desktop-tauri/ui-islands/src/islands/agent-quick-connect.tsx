import * as React from 'react'
import { Alert, Badge, Button, Dialog, DialogBody, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, Empty } from '@ui'
import { SearchableSelect } from '../../../ui-shared/searchable-select'
import { t } from '../i18n'
import { refreshCatalogModels, useAgentCatalog, type CatalogModel } from './agent-catalog'
import { SingleModelFields } from './agent-fields'
import { ModelsFields } from './agent-model-fields'
import { protocolLabel, validateDraft } from './agent-model'
import { fillQuickProfile, quickProfile } from './agent-quick-model'
import { applyProfile, errorMessage, getAgentStore, requestPreview, subscribeAgentStore } from './agent-state'
import { WritePreview } from './agent-preview'
import type { AgentClient, AgentPreview, AgentProfile } from './agent-types'

export function QuickConnectDialog({ client, onClose }: { client: AgentClient; onClose: () => void }) {
  const store = React.useSyncExternalStore(subscribeAgentStore, getAgentStore, getAgentStore)
  const [draft, setDraft] = React.useState(() => ({ ...quickProfile(client, store.gateway), gatewayKeyId: store.gatewayKeys[0]?.id || '' }))
  const { catalog, loading, error: catalogError, retry } = useAgentCatalog(client.id === 'zed' ? undefined : draft.gatewayKeyId)
  const [error, setError] = React.useState('')
  const [preview, setPreview] = React.useState<AgentPreview | null>(null)
  const [busy, setBusy] = React.useState(false)
  const initialized = React.useRef(false)
  const previousCatalog = React.useRef<CatalogModel[]>([])
  const selectedKey = store.gatewayKeys.find(key => key.id === draft.gatewayKeyId)
  React.useEffect(() => {
    if (loading || catalogError) return
    const previous = previousCatalog.current
    previousCatalog.current = catalog
    if (!initialized.current) {
      initialized.current = true
      setDraft(profile => fillQuickProfile(client, profile, catalog, store.defaultModel))
    } else if (client.supportsModels && previous !== catalog) {
      setDraft(profile => ({ ...profile, models: refreshCatalogModels(client, profile.models, previous, catalog) }))
    }
  }, [loading, catalogError, catalog, client, store.defaultModel])
  const update = (patch: Partial<AgentProfile>) => { setDraft(current => ({ ...current, ...patch })); setPreview(null); setError('') }
  const ids = [...(client.supportsModels ? draft.models.map(model => model.id) : [draft.model]), ...Object.values(draft.claudeModels)].filter(Boolean)
  const unavailable = !loading && !catalogError ? ids.filter(id => !catalog.some(model => model.id.toLowerCase() === id.toLowerCase())) : []
  const connect = async () => {
    if (busy || loading || catalogError) return
    if (draft.authMode === 'gateway' && !selectedKey) { setError(t('Create and enable a key on the Gateway Key page first.')); return }
    const missing = validateDraft(client, draft)
    if (missing) { setError(missing); return }
    if (unavailable.length) { setError(t('Some selected models are unavailable or not allowed by this key. Remove them or choose another key.')); return }
    setBusy(true); setError('')
    try {
      if (!preview) setPreview(await requestPreview(client, draft, 'quick-connect'))
      else { await applyProfile(client, draft, 'quick-connect', preview); onClose() }
    } catch (error) { setError(errorMessage(error)); setPreview(null) }
    finally { setBusy(false) }
  }
  return <Dialog open onOpenChange={(open, details) => { if (!open && busy) details.cancel(); else if (!open) onClose() }}>
    <DialogContent className='agent-config-dialog agent-quick-dialog' showCloseButton={!busy}>
      {/* 说明挂到标题下当副标题：原来它是正文的第一段，与下面三块信息抢同一层级 */}
      <DialogHeader className='agent-quick-head'><DialogTitle>{t('Quick connect')} · {client.name}</DialogTitle>
        <DialogDescription>{t('Choose models to connect. File changes are shown before confirmation, and a backup is made before writing.')}</DialogDescription>
      </DialogHeader>
      <DialogBody className='agent-quick-body'>
        {preview ? <WritePreview preview={preview} /> : <>
          {/* 接进哪里：一行一个字段（端点 / 写入文件 / 网关 Key），左标签右值 */}
          <section className='agent-quick-info'>
            <div className='agent-quick-info-row'>
              <span className='agent-quick-info-key'>{t('Endpoint')}</span>
              <code className='agent-quick-info-value'>{draft.baseUrl}</code>
              <Badge variant='outline' shape='tag'>{protocolLabel(draft.protocol)}</Badge>
            </div>
            <div className='agent-quick-info-row'>
              <span className='agent-quick-info-key'>{t('Configuration file')}</span>
              <code className='agent-quick-info-value is-muted'>{client.resolvedPath}</code>
            </div>
            {client.id !== 'zed' && <div className='agent-quick-info-row'>
              <span className='agent-quick-info-key'>{t('Gateway key')}</span>
              <SearchableSelect value={draft.gatewayKeyId} disabled={busy || !store.gatewayKeys.length}
                onValueChange={gatewayKeyId => update({ gatewayKeyId })}
                className='agent-quick-info-select' aria-label={t('Gateway key')}
                displayValue={selectedKey ? selectedKey.name || selectedKey.id : t('Select a gateway key')}
                options={store.gatewayKeys.map(key => ({ value: key.id, label: key.name || key.id }))} />
              <span className='agent-quick-info-hint'>{t('Switching keys re-checks the selected models against the new key\'s permissions.')}</span>
            </div>}
          </section>
          {loading ? <Empty>{t('Loading gateway models and filling the configuration…')}</Empty> : catalogError ? <Alert variant='warning'>
            <span>{catalogError}</span><Button variant='outline' size='sm' disabled={busy} onClick={retry}>{t('Retry')}</Button>
          </Alert> : <>
            {unavailable.length > 0 && <Alert variant='warning'>{t('Some selected models are unavailable or not allowed by this key. Remove them or choose another key.')}<code className='block break-all'>{unavailable.join(', ')}</code></Alert>}
            {client.supportsModels ? <ModelsFields client={client} draft={draft} update={update} catalog={catalog} disabled={busy} quick layout='split' />
              : <SingleModelFields client={client} draft={draft} update={update} catalog={catalog} disabled={busy} />}
            {!catalog.length && <Alert variant='warning'>{t('No chat models are available for this key. Enable a model or choose another key.')}</Alert>}
          </>}
          {client.id === 'zed' && <Alert variant='warning'>{t('Enter the API key in Zed provider settings. This page does not modify system credentials.')}</Alert>}
        </>}
        {error && <Alert variant='destructive'>{error}</Alert>}
      </DialogBody>
      <DialogFooter>
        {preview && <Button variant='ghost' className='mr-auto' disabled={busy} onClick={() => { setPreview(null); setError('') }}>{t('Back to editing')}</Button>}
        <Button variant='outline' disabled={busy} onClick={onClose}>{t('Cancel')}</Button>
        <Button disabled={busy || loading || Boolean(catalogError)} onClick={() => void connect()}>{busy ? t('Processing…') : preview ? t('Confirm connection') : t('Connect client')}</Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
}
