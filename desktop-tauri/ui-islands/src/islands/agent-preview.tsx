import * as React from 'react'
import { Alert, Button, Collapsible, CollapsibleContent, CollapsibleTrigger, Dialog, DialogBody, DialogContent, DialogFooter, DialogHeader, DialogTitle, Empty } from '@ui'
import { t } from '../i18n'
import { applyProfile, errorMessage, requestPreview, restoreClient } from './agent-state'
import { AgentIcon } from './agent-actions'
import type { AgentClient, AgentOperation, AgentPreview, AgentProfile } from './agent-types'

export function WritePreview({ preview, showWarnings = true }: { preview: AgentPreview; showWarnings?: boolean }) {
  return <div className='flex flex-col gap-3'>
    {showWarnings && preview.warnings.map((warning,index) => <Alert key={index} variant='warning'>{warning}</Alert>)}
    <p className='text-xs text-subtle'>{t('Keys are hidden. Green indicates additions and red indicates removals. All files to be written are listed here.')}</p>
    {preview.files.map(file => <div key={file.path} className='agent-preview-file'>
      <div className='flex flex-wrap items-center gap-2 pb-2 text-xs'><code className='break-all'>{file.path}</code>
        <span className='ml-auto text-subtle'>{file.created ? t('New file') : file.deleted ? t('Remove newly created file') : file.changed ? t('Changed') : t('No changes')}</span></div>
      <pre className='agent-code' aria-label={t('Configuration diff')}>
        {file.diff.map((line,index) => <span key={index} className={`agent-line ${line.kind === 'added' ? 'is-added' : line.kind === 'removed' ? 'is-removed' : ''}`}>
          <span className='agent-line-no'>{line.newLine ?? line.oldLine ?? ''}</span>
          <span className='agent-line-sign' aria-hidden='true'>{line.kind === 'added' ? '+' : line.kind === 'removed' ? '−' : ' '}</span>
          <span className='agent-line-text'>{line.text || ' '}</span>
        </span>)}
      </pre>
    </div>)}
  </div>
}

export function ApplyDialog({ client, profile, operation, onClose }: {
  client: AgentClient; profile?: AgentProfile; operation: AgentOperation; onClose: () => void
}) {
  const [preview, setPreview] = React.useState<AgentPreview | null>(null)
  const [error, setError] = React.useState('')
  const [loading, setLoading] = React.useState(true)
  const [saving, setSaving] = React.useState(false)
  const [generation, setGeneration] = React.useState(0)
  const nativeRevision = React.useRef(client.nativeRevision)
  const revision = React.useRef(client.revision)
  const configPath = React.useRef(client.configPath)
  const unified = operation.startsWith('config-')
  const deleting = operation === 'native-delete' || operation === 'config-delete'
  const [detailsOpen, setDetailsOpen] = React.useState(false)
  const previewClient = operation.startsWith('native-') || unified ? { ...client, nativeRevision: nativeRevision.current, revision: revision.current, configPath: configPath.current } : client
  const title = deleting ? t('Delete configuration') : operation === 'config-disable' ? t('Disable configuration') : operation === 'config-use' ? client.supportsDefault ? t('Use this configuration') : t('Enable configuration')
    : operation === 'disconnect' ? t('Disconnect from this gateway') : operation === 'restore' ? t('Undo last application') : operation === 'default' ? t('Apply and set as default') : t('Apply to client')
  React.useEffect(() => {
    let alive = true
    setLoading(true); setPreview(null); setError('')
    void requestPreview(previewClient, profile, operation).then(result => { if (alive) setPreview(result) })
      .catch(error => { if (alive) setError(errorMessage(error)) }).finally(() => { if (alive) setLoading(false) })
    return () => { alive = false }
  }, [client.id, profile, operation, generation])
  const apply = async () => {
    if (!preview || saving || loading) return
    setSaving(true); setError('')
    try {
      if (operation === 'restore' || operation === 'disconnect') await restoreClient(client, operation, preview)
      else if (profile) await applyProfile(previewClient, profile, operation, preview)
      else throw new Error(t('No configuration selected'))
      onClose()
    } catch (error) { setError(errorMessage(error)); setPreview(null); setSaving(false) }
  }
  return <Dialog open onOpenChange={(open,details) => { if (open) return; if (saving) details.cancel(); else onClose() }}>
    <DialogContent className='agent-config-dialog'>
      <DialogHeader><DialogTitle>{title} · {client.name}</DialogTitle></DialogHeader>
      <DialogBody>
        {profile && <p className='text-sm font-medium'>{profile.name}</p>}
        {loading ? <Empty>{t('Generating exact preview…')}</Empty> : preview ? unified ? <>
          <p className='text-sm leading-relaxed'>{deleting ? t('This configuration and its matching client entry will be deleted. Other configurations are unaffected.')
            : operation === 'config-disable' ? t('This configuration will be disabled in the client\'s available options. Other configurations are unaffected.')
              : client.supportsDefault ? t('Confirm to switch to this configuration. The previous configuration will remain in the list.') : t('Confirm to add this configuration to the client. Existing configurations will be kept.')}</p>
          {preview.warnings.map((warning, index) => <p key={index} className='text-xs leading-relaxed text-subtle'>{warning}</p>)}
          <Collapsible open={detailsOpen} onOpenChange={setDetailsOpen}>
            <CollapsibleTrigger render={<Button variant='ghost' size='sm' />}><AgentIcon name='chevron' />{t('View file changes')}</CollapsibleTrigger>
            <CollapsibleContent><WritePreview preview={preview} showWarnings={false} /></CollapsibleContent>
          </Collapsible>
        </> : <WritePreview preview={preview} /> : null}
        {error && <Alert variant='destructive'>{error}</Alert>}
      </DialogBody>
      <DialogFooter>
        <Button variant='ghost' size='sm' disabled={saving || loading} onClick={() => setGeneration(value => value+1)}>{t('Regenerate preview')}</Button>
        <span className='mr-auto' />
        <Button variant='outline' disabled={saving} onClick={onClose}>{t('Cancel')}</Button>
        <Button variant={deleting ? 'destructive' : 'default'} disabled={!preview || loading || saving || (!preview.hasChanges && !unified && !['restore','disconnect'].includes(operation))} onClick={() => void apply()}>{saving ? t('Processing…') : deleting ? t('Confirm deletion') : operation === 'config-disable' ? t('Confirm disable') : operation === 'config-use' ? t('Confirm') : t('Confirm application')}</Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
}
