import { useCallback, useEffect, useState } from 'react'
import { Alert, Badge, Button, Card, Dialog, DialogBody, DialogContent, DialogFooter, DialogHeader, DialogTitle, Empty, Input, Label, Textarea, confirmDialog } from '@ui'
import type { Translate } from './assistant-read-types'
import { resourceBase, type ResourceCapabilities, type ResourceKind, type ResourceRequest, type ResourcePreview, type PromptResource, type SkillsResource, type McpResource } from './agent-resources'

type Props = { clientId: string; kind: ResourceKind; capabilities?: ResourceCapabilities; request: ResourceRequest; t: Translate; notify: (text: string, bad?: boolean) => void }
type PendingChange = { payload: Record<string, unknown>; preview: ResourcePreview; name: string; deleting: boolean }

export function AgentResourcesPanel({ clientId, kind, capabilities, request, t, notify }: Props) {
  const [data, setData] = useState<PromptResource | SkillsResource | McpResource | null>(null)
  const [error, setError] = useState('')
  const [dialogError, setDialogError] = useState('')
  const [busy, setBusy] = useState(false)
  const [draft, setDraft] = useState('')
  const [selected, setSelected] = useState('')
  const [creating, setCreating] = useState(false)
  const [name, setName] = useState('')
  const [revision, setRevision] = useState('absent')
  const [editorOpen, setEditorOpen] = useState(false)
  const [pending, setPending] = useState<PendingChange | null>(null)
  const base = resourceBase(clientId)
  const supported = capabilities?.[kind] === true
  const load = useCallback(async () => {
    if (!supported) return
    setBusy(true); setError('')
    try {
      const result = await request<PromptResource | SkillsResource | McpResource>('GET', `${base}?kind=${kind}`)
      setData(result); setSelected(''); setName('')
      if (kind === 'prompt') { setDraft((result as PromptResource).content); setRevision((result as PromptResource).revision) }
      else { setDraft(''); setRevision(kind === 'mcp' ? (result as McpResource).revision : 'absent') }
    } catch (failure) { setError(failure instanceof Error ? failure.message : String(failure)) }
    finally { setBusy(false) }
  }, [base, kind, request, supported])
  useEffect(() => {
    setData(null); setEditorOpen(false); setPending(null); setDialogError('')
    void load()
  }, [load])
  const prompt = kind === 'prompt' && data && 'content' in data ? data : null
  const skills = kind === 'skills' && data && 'items' in data ? data : null
  const mcp = kind === 'mcp' && data && 'servers' in data ? data : null
  const skill = skills?.items.find(item => item.id === selected)

  const closeDialog = () => { setEditorOpen(false); setPending(null); setDialogError('') }
  const pick = async (value?: string) => {
    if (busy) return
    const adding = value === undefined
    setSelected(value ?? ''); setCreating(adding); setError(''); setDialogError(''); setPending(null)
    if (kind === 'skills') {
      if (adding) {
        setName(''); setRevision('absent'); setDraft('---\nname: my-skill\ndescription: Describe when to use this skill.\n---\n\n')
      } else {
        setBusy(true)
        try {
          const detail = await request<PromptResource>('GET', `${base}?kind=skill&id=${encodeURIComponent(value)}`)
          setDraft(detail.content); setRevision(detail.revision)
        } catch (failure) {
          setError(failure instanceof Error ? failure.message : String(failure))
          return
        } finally { setBusy(false) }
      }
    } else {
      const server = mcp?.servers.find(item => item.name === value)
      setName(server?.name || '')
      setRevision(mcp?.revision || 'absent')
      const template = clientId === 'opencode' ? { type: 'local', command: [''] } : clientId === 'dsh' ? { transport: 'stdio', command: '', args: [], toolCallTimeoutMs: 60000, failOnStartupError: false } : { command: '', args: [] }
      setDraft(JSON.stringify(server?.config || template, null, 2))
    }
    setEditorOpen(true)
  }

  const prepare = async (payload: Record<string, unknown>, resourceName: string, deleting = false) => {
    if (busy || !data) return
    setBusy(true); setError(''); setDialogError('')
    try {
      const preview = await request<ResourcePreview>('POST', `${base}/preview`, payload)
      if (!preview.hasChanges) { notify(t('The content already matches; no write is needed')); return }
      if (kind !== 'prompt') { setPending({ payload, preview, name: resourceName, deleting }); return }
      const confirmed = await confirmDialog(<div className='flex flex-col gap-3'>
        <p>{t('All operations act on the gateway host.')}</p>
        {preview.warnings.map((warning, index) => <p key={index}>{warning}</p>)}
        {preview.files.map(file => <div key={file.path}><code className='break-all'>{file.path}</code><pre className='max-h-72 overflow-auto whitespace-pre-wrap text-xs'>{file.diff.filter(line => line.kind !== 'same').map(line => `${line.kind === 'added' ? '+' : '−'} ${line.text}`).join('\n')}</pre></div>)}
      </div>, { title: t('Confirm writing client resources'), confirmText: t('Confirm'), destructive: false })
      if (!confirmed) return
      await request('POST', `${base}/apply`, { ...payload, token: preview.token })
      notify(t('Client resources saved and verified'))
      await load()
    } catch (failure) {
      const message = failure instanceof Error ? failure.message : String(failure)
      if (editorOpen) setDialogError(message)
      else setError(message)
    } finally { setBusy(false) }
  }

  const save = () => {
    const payload: Record<string, unknown> = { kind, revision, delete: false }
    if (kind === 'prompt') payload.content = draft
    if (kind === 'skills') { payload.id = creating ? `0:${name.trim()}` : selected; payload.content = draft }
    if (kind === 'mcp') {
      payload.name = name.trim()
      try { payload.config = JSON.parse(draft) }
      catch (failure) { setDialogError(failure instanceof Error ? failure.message : String(failure)); return }
    }
    void prepare(payload, kind === 'skills' ? skill?.name || name.trim() : name.trim())
  }

  const apply = async () => {
    if (busy || !pending) return
    setBusy(true); setDialogError('')
    try {
      await request('POST', `${base}/apply`, { ...pending.payload, token: pending.preview.token })
      notify(t('Client resources saved and verified'))
      closeDialog()
      await load()
    } catch (failure) { setDialogError(failure instanceof Error ? failure.message : String(failure)) }
    finally { setBusy(false) }
  }

  if (!supported) return <Alert className='mt-3'>{capabilities?.note || t('Resource management is not yet supported for this client')}</Alert>
  const editorTitle = kind === 'skills'
    ? creating ? t('New Skill') : t('Edit Skill')
    : creating ? t('Add MCP') : t('Edit MCP')
  return <div className='mt-4 flex min-w-0 flex-col gap-3'>
    <div className='flex items-center justify-between gap-3'>
      <p className='min-w-0 text-xs text-muted-foreground'>{capabilities?.note}</p>
      <div className='flex shrink-0 items-center gap-2'>
        {skills || mcp ? <Button size='sm' disabled={busy} onClick={() => void pick()}>{kind === 'skills' ? t('New Skill') : t('Add MCP')}</Button> : null}
        <Button size='sm' variant='outline' disabled={busy} onClick={() => void load()}>{t('Refresh')}</Button>
      </div>
    </div>
    {error ? <Alert variant='destructive'>{error}</Alert> : null}
    {prompt ? <><code className='break-all text-xs'>{prompt.path}</code><Textarea rows={16} className='font-mono' value={draft} disabled={busy} onChange={event => setDraft(event.currentTarget.value)} /><Button disabled={busy} onClick={save}>{t('预览并保存')}</Button></> : null}
    {skills ? <>
      <code className='whitespace-pre-wrap break-all text-xs'>{skills.roots.join('\n')}</code>
      {!skills.items.length ? <Empty>{t('This client has no Skills yet')}</Empty> : <ul className='m-0 flex list-none flex-col gap-2 p-0'>
        {skills.items.map(item => <Card key={item.id} render={<li />} className='flex items-start gap-3 p-3'>
          <div className='min-w-0 flex-1'>
            <div className='flex items-center gap-2'><span className='truncate text-sm font-medium'>{item.name}</span>{item.linked ? <Badge variant='outline'>{t('Read-only')}</Badge> : null}</div>
            {item.description ? <p className='mt-1 text-xs text-muted-foreground'>{item.description}</p> : null}
            <code className='mt-1 block break-all text-xs text-muted-foreground'>{item.path}</code>
          </div>
          <div className='flex shrink-0 items-center gap-1.5'>
            <Button size='sm' variant='outline' disabled={busy} onClick={() => void pick(item.id)}>{item.linked ? t('View') : t('Edit')}<Edit')}</Button>
            <Button size='sm' variant='destructive' disabled={busy || item.linked} onClick={() => void prepare({ kind, id: item.id, revision: item.revision, delete: true }, item.name, true)}>{t('Delete')}</Button>
          </div>
        </Card>)}
      </ul>}
    </> : null}
    {mcp ? <>
      <code className='break-all text-xs'>{mcp.path}</code>
      {clientId === 'dsh' && <p className='text-xs text-muted-foreground'>{t('MCP entries with dynamic expressions, isolated scopes, or global overrides are read-only. Edit them in DSH.')}</p>}
      {!mcp.servers.length ? <Empty>{t('This client has no MCP servers yet')}</Empty> : <ul className='m-0 flex list-none flex-col gap-2 p-0'>
        {mcp.servers.map(item => <Card key={item.name} render={<li />} className='flex items-start gap-3 p-3'>
          <div className='min-w-0 flex-1'><p className='truncate text-sm font-medium'>{item.name}</p>{item.endpoint ? <code className='mt-1 block break-all text-xs text-muted-foreground'>{item.endpoint}</code> : null}</div>
          <div className='flex shrink-0 items-center gap-1.5'>
            <Button size='sm' variant='outline' disabled={busy || item.editable === false} onClick={() => void pick(item.name)}>{t('Edit')}</Button>
            <Button size='sm' variant='destructive' disabled={busy || item.deletable === false} onClick={() => void prepare({ kind, name: item.name, revision: mcp.revision, delete: true }, item.name, true)}>{t('Delete')}</Button>
          </div>
        </Card>)}
      </ul>}
    </> : null}
    <Dialog open={editorOpen || Boolean(pending)} onOpenChange={open => { if (!open && !busy) closeDialog() }}>
      <DialogContent className='w-[min(760px,calc(100vw-48px))]' showCloseButton={!busy}>
        <DialogHeader><DialogTitle>{pending ? pending.deleting ? t('Confirm deleting client resources') : t('Confirm writing client resources') : editorTitle}</DialogTitle></DialogHeader>
        <DialogBody>
          {pending ? <>
            <p className='break-all font-medium'>{pending.name}</p>
            <p className='text-xs text-muted-foreground'>{t('All operations act on the gateway host.')}</p>
            {pending.deleting && kind === 'skills' ? <p className='text-xs text-muted-foreground'>{t('Deleting removes only SKILL.md. Supporting files remain in the original folder; no scripts are executed.')}</p> : null}
            {pending.preview.warnings.map((warning, index) => <Alert key={index} variant='warning'>{warning}</Alert>)}
            {pending.preview.files.map(file => <div key={file.path}><code className='break-all text-xs'>{file.path}</code><pre className='max-h-72 overflow-auto whitespace-pre-wrap text-xs'>{file.diff.filter(line => line.kind !== 'same').map(line => `${line.kind === 'added' ? '+' : '−'} ${line.text}`).join('\n')}</pre></div>)}
          </> : kind === 'skills' ? <>
            {creating ? <><Label htmlFor='agent-skill-name'>{t('Skill folder name')}</Label><Input id='agent-skill-name' placeholder='my-skill' value={name} disabled={busy} onChange={event => setName(event.currentTarget.value)} /></> : <code className='break-all text-xs'>{skill?.path}</code>}
            {skill?.description ? <p className='text-xs text-muted-foreground'>{skill.description}</p> : null}
            <Textarea rows={16} className='font-mono' value={draft} disabled={busy || skill?.linked} onChange={event => setDraft(event.currentTarget.value)} />
            <p className='text-xs text-muted-foreground'>{t('Deleting removes only SKILL.md. Supporting files remain in the original folder; no scripts are executed.')}</p>
          </> : <>
            <Label htmlFor='agent-mcp-name'>{t('MCP server name')}</Label><Input id='agent-mcp-name' value={name} disabled={!creating || busy} onChange={event => setName(event.currentTarget.value)} />
            <Label htmlFor='agent-mcp-config'>{t('Native client MCP configuration (JSON)')}</Label><Textarea id='agent-mcp-config' rows={14} className='font-mono' value={draft} disabled={busy} onChange={event => setDraft(event.currentTarget.value)} />
            <p className='text-xs text-muted-foreground'>{t('Only the selected MCP entry is changed; other settings and comments are preserved. Saving does not start the server.')}</p>
          </>}
          {dialogError ? <Alert variant='destructive'>{dialogError}</Alert> : null}
        </DialogBody>
        <DialogFooter>
          {pending && editorOpen ? <Button variant='outline' className='mr-auto' disabled={busy} onClick={() => { setPending(null); setDialogError('') }}>{t('Back to editing')}</Button> : null}
          <Button variant='outline' disabled={busy} onClick={closeDialog}>{t('Cancel')}</Button>
          {pending ? <Button variant={pending.deleting ? 'destructive' : 'default'} disabled={busy} onClick={() => void apply()}>{t('Confirm')}</Button>
            : <Button disabled={busy || skill?.linked || (creating && !name.trim())} onClick={save}>{t('Preview and save')}</Button>}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
}
