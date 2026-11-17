import * as React from 'react'
import { Alert, Button, Collapsible, CollapsibleContent, CollapsibleTrigger, Dialog, DialogBody, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, Field, FieldDescription, FieldLabel, Switch } from '@ui'
import { t } from '../i18n'
import { applyProfile, confirm, errorMessage, getAgentStore, requestPreview, saveConfiguration, toast } from './agent-state'
import { cloneProfile, profileSignature, validateDraft } from './agent-model'
import { ConnectionFields, SingleModelFields, TextField } from './agent-fields'
import { ModelsFields } from './agent-model-fields'
import { refreshCatalogModels, useAgentCatalog, type CatalogModel } from './agent-catalog'
import { WritePreview } from './agent-preview'
import { fillQuickProfile } from './agent-quick-model'
import { AgentIcon } from './agent-actions'
import type { AgentClient, AgentPreview, AgentProfile } from './agent-types'

export function ProfileDialog({ client, profile, creating, onClose }: { client: AgentClient; profile: AgentProfile; creating: boolean; onClose: () => void }) {
  const native = Boolean(profile.liveId)
  const [draft, setDraft] = React.useState(() => cloneProfile(profile))
  const [error, setError] = React.useState('')
  const [saving, setSaving] = React.useState(false)
  const [preview, setPreview] = React.useState<AgentPreview | null>(null)
  const [previewing, setPreviewing] = React.useState(false)
  const { catalog, loading: catalogLoading, error: catalogError, retry: retryCatalog } = useAgentCatalog(
    draft.kind === 'gateway' && draft.authMode === 'gateway' ? draft.gatewayKeyId : undefined)
  const initial = React.useRef(profileSignature(profile))
  const nativeRevision = React.useRef(client.nativeRevision)
  const revision = React.useRef(client.revision)
  const configPath = React.useRef(client.configPath)
  const previewClient = { ...client, nativeRevision: nativeRevision.current, revision: revision.current, configPath: configPath.current }
  const previewGeneration = React.useRef(0)
  const dirty = profileSignature(draft) !== initial.current
  const operation = 'config-edit' as const
  const single = ['codex', 'claude-code'].includes(client.id) && !draft.baseUrl && draft.authMode === 'preserve' && !draft.models.length
  const validationClient = { ...client, supportsModels: client.supportsModels && !single,
    ...(native && !profile.enabled && !profile.models.length && !profile.model ? { supportsDefault: false } : {}) }
  const previousCatalog = React.useRef<CatalogModel[]>([])
  const initialized = React.useRef(!creating)
  const [confirming, setConfirming] = React.useState(false)
  const [detailsOpen, setDetailsOpen] = React.useState(false)
  React.useEffect(() => {
    if (catalogLoading || catalogError) return
    const previous = previousCatalog.current
    previousCatalog.current = catalog
    if (draft.kind !== 'gateway') return
    if (!initialized.current) {
      initialized.current = true
      setDraft(profile => profile.models.length || profile.model ? profile : fillQuickProfile(client, profile, catalog, getAgentStore().defaultModel))
      return
    }
    if (!previous.length || !client.supportsModels) return
    previewGeneration.current += 1
    setPreview(null); setConfirming(false); setPreviewing(false)
    setDraft(profile => ({ ...profile, models: refreshCatalogModels(client, profile.models, previous, catalog) }))
  }, [catalogLoading, catalogError, catalog, client.id, client.supportsModels, draft.kind])
  React.useEffect(() => () => { previewGeneration.current += 1 }, [])
  const update = (values: Partial<AgentProfile>) => {
    previewGeneration.current += 1
    setDraft(current => ({ ...current, ...values })); setPreview(null); setConfirming(false); setPreviewing(false); setError('')
  }
  const close = async () => {
    if (saving) return
    if (dirty && !await confirm(t('Discard unsaved changes?'), t('Closing discards these changes without saving the configuration or changing the client.'), t('Discard changes'))) return
    onClose()
  }
  const save = async () => {
    if (saving) return
    const message = validateDraft(validationClient,draft)
    if (message) { setError(message); return }
    if (native) {
      await showPreview()
      return
    }
    setSaving(true); setError('')
    try {
      await saveConfiguration(previewClient, draft, creating)
      onClose(); toast(t('Configuration saved. Select it in the list to use it.'))
    } catch (error) { setError(errorMessage(error)); setSaving(false) }
  }
  const showPreview = async () => {
    const message = validateDraft(validationClient,draft)
    if (message) { setError(message); return }
    const generation = ++previewGeneration.current
    setPreviewing(true); setError('')
    try { const result = await requestPreview(previewClient,draft,operation); if (generation === previewGeneration.current) { setPreview(result); setConfirming(true) } }
    catch (error) { if (generation === previewGeneration.current) setError(errorMessage(error)) }
    finally { if (generation === previewGeneration.current) setPreviewing(false) }
  }
  const apply = async () => {
    if (!preview || saving) return
    setSaving(true); setError('')
    try { await applyProfile(previewClient,draft,operation,preview); onClose() }
    catch (error) { setError(errorMessage(error)); setPreview(null); setConfirming(false) }
    finally { setSaving(false) }
  }
  return <Dialog open onOpenChange={(open,details) => { if (!open) { details.cancel(); void close() } }}>
    <DialogContent className='agent-config-dialog' showCloseButton={!saving}>
      <DialogHeader className='agent-config-head'><DialogTitle>{creating ? t('Add configuration') : t('Edit configuration')} · {client.name}</DialogTitle>
        <DialogDescription>{native ? t('Saving also updates the client. Review the changes before writing; a backup is created automatically.') : t('Once saved, switch to it from the configuration list. The current configuration remains unchanged.')}</DialogDescription>
      </DialogHeader>
      <DialogBody>
        {!confirming && <>
        <section className='agent-profile-connection flex min-w-0 flex-col gap-4'>
          <div className='grid min-w-0 grid-cols-1 gap-4 sm:grid-cols-2'>
            <TextField label={t('Configuration name')} fieldKey='' value={draft.name} onChange={name => update({name})} disabled={saving || previewing} />
            <TextField label={t('Note')} fieldKey='' value={draft.note} onChange={note => update({note})} disabled={saving || previewing} />
          </div>
          <ConnectionFields client={client} draft={draft} update={update} disabled={saving || previewing} />
          {draft.kind === 'gateway' && catalogLoading && <p className='text-xs text-subtle'>{t('Loading gateway models and filling the configuration…')}</p>}
          {catalogError && draft.kind === 'gateway' && <Alert variant='warning'>{t('Failed to read the model catalog. You can enter the actual model ID manually.')} {catalogError}
            <Button variant='outline' size='sm' onClick={retryCatalog} disabled={saving || previewing}>{t('Retry')}</Button></Alert>}
        </section>
        {validationClient.supportsModels ? <ModelsFields client={validationClient} draft={draft} update={update} catalog={catalog} disabled={saving || previewing || draft.kind === 'gateway' && catalogLoading} native={native} />
          : <section className='flex min-w-0 flex-col gap-4 border-t border-hairline pt-4'><h3 className='text-sm font-semibold'>{t('Model settings')}</h3>
            <SingleModelFields client={validationClient} draft={draft} update={update} catalog={catalog} disabled={saving || previewing} /></section>}
        {client.supportsToggle && native && <Field className='rounded-md border border-border bg-surface-inset p-3'>
          <div className='flex items-center justify-between gap-3'><FieldLabel>{t('Enable this configuration')}</FieldLabel>
            <Switch checked={draft.enabled} disabled={saving || previewing} onCheckedChange={enabled => update({enabled})} aria-label={t('Enable this configuration')} /></div>
          <FieldDescription>{t('Confirming the write also updates the client entry\'s enabled state. Canceling leaves it unchanged.')}</FieldDescription>
        </Field>}
        </>}
        {preview && <>
          <Alert>{t('Confirm to update the configuration files for {name}. The original files will be backed up automatically.', { name: client.name })}</Alert>
          {preview.warnings.map((warning, index) => <p key={index} className='text-xs leading-relaxed text-subtle'>{warning}</p>)}
          <Collapsible open={detailsOpen} onOpenChange={setDetailsOpen}>
            <CollapsibleTrigger render={<Button variant='ghost' size='sm' />}><AgentIcon name='chevron' />{t('View file changes')}</CollapsibleTrigger>
            <CollapsibleContent><WritePreview preview={preview} showWarnings={false} /></CollapsibleContent>
          </Collapsible>
        </>}
        {error && <Alert variant='destructive'>{error}</Alert>}
      </DialogBody>
      <DialogFooter className='agent-config-footer'>
        {confirming && <Button variant='ghost' disabled={saving || previewing} onClick={() => { setConfirming(false); setPreview(null) }}>{t('Back to editing')}</Button>}<span className='mr-auto' />
        <Button variant='outline' disabled={saving || previewing} onClick={() => void close()}>{t('Cancel')}</Button>
        <Button disabled={saving || previewing || (!creating && !dirty) || (confirming && !preview)} onClick={() => void (confirming ? apply() : save())}>{saving ? t('Saving…') : previewing ? t('Processing…') : confirming ? t('Confirm save') : t('Save')}</Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
}
