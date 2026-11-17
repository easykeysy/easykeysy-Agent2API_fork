import * as React from 'react'
import { Alert, Button, Dialog, DialogBody, DialogContent, DialogFooter, DialogHeader, DialogTitle, Field, FieldLabel, Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@ui'
import { t } from '../i18n'
import { TextField } from './agent-fields'
import { errorMessage, savePath } from './agent-state'
import { dshTargetLabel } from './agent-dsh'
import type { AgentClient } from './agent-types'

export function PathDialog({client,onClose}: {client:AgentClient;onClose:()=>void}) {
  const [path,setPath] = React.useState(client.pathOverride || client.configTargets?.[0]?.path || client.configPath)
  const selectedTarget = client.configTargets?.find(target => target.path === path)
  const [saving,setSaving] = React.useState(false)
  const [error,setError] = React.useState('')
  const save = async (path:string) => {
    setSaving(true);setError('')
    try { await savePath(client,path);onClose() }
    catch(error) {setError(errorMessage(error));setSaving(false)}
  }
  return <Dialog open onOpenChange={(open,details)=>{if(!open){if(saving)details.cancel();else onClose()}}}>
    <DialogContent><DialogHeader><DialogTitle>{t('Specify configuration path')} · {client.name}</DialogTitle></DialogHeader>
      <DialogBody><Alert>{t('The path is on the gateway host, not the browser\'s device. Only the detection target changes; client files are not modified.')}</Alert>
        {client.id === 'dsh' && Boolean(client.configTargets?.length) && <Field>
          <FieldLabel>{t('DSH configuration target')}</FieldLabel>
          <Select value={path} disabled={saving} onValueChange={value => { if (typeof value === 'string') setPath(value) }}>
            <SelectTrigger><SelectValue>{selectedTarget ? dshTargetLabel(selectedTarget.profile, selectedTarget.label) : t('Custom path')}</SelectValue></SelectTrigger>
            <SelectContent>{client.configTargets?.map(target => <SelectItem key={target.path} value={target.path}>{dshTargetLabel(target.profile, target.label)}{target.version ? ` · ${target.version}` : ''}</SelectItem>)}</SelectContent>
          </Select>
          <p className='text-xs text-subtle'>{t('Newer web and desktop apps store model settings separately. Legacy model settings are shared; MCP is always written only to the selected profile.')}</p>
        </Field>}
        <TextField label={t('Full configuration file path')} fieldKey='' value={path} onChange={setPath} disabled={saving} />
        <p className='text-xs text-subtle'>{t('The client must use the same configuration file. This page does not change its launch arguments.')}</p>
        {error&&<Alert variant='destructive'>{error}</Alert>}
      </DialogBody>
      <DialogFooter><Button variant='ghost' disabled={saving} onClick={()=>void save('')}>{t('Restore automatic detection')}</Button><span className='mr-auto'/>
        <Button variant='outline' disabled={saving} onClick={onClose}>{t('Cancel')}</Button><Button disabled={saving} onClick={()=>void save(path)}>{t('Save path')}</Button></DialogFooter>
    </DialogContent>
  </Dialog>
}
