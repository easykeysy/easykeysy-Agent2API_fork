import * as React from 'react'
import {
  Alert, Button, Checkbox, Dialog, DialogBody, DialogContent, DialogFooter, DialogHeader,
  DialogTitle, Empty, Field, FieldDescription, FieldLabel, Select, SelectContent, SelectItem,
  SelectTrigger, SelectValue, SegmentedControl, Spinner, Input, Textarea,
} from '@ui'
import { t } from '../i18n'
import { toast } from './assistant-state'
import {
  fetchApiKeys, fetchInstallTargets, installMcp, installSkill, uninstallMcp, uninstallSkill, SERVICE_NAME,
  type ApiKeyOption, type InstallOutcome, type InstallTarget, type ToolService,
} from './tools-data'

type Mode = 'skill' | 'mcp'

function targetLabel(target: InstallTarget): string {
  if (!target.detected) return t('Not detected')
  return target.installed ? t('Installed') : t('Detected')
}

function KeyPicker({ keys, value, onChange }: { keys: ApiKeyOption[]; value: string; onChange: (value: string) => void }) {
  const selected = keys.find(key => key.id === value)
  const label = selected ? `${selected.name}（${selected.masked}）` : keys.length ? t('Select an API Key') : t('No available gateway Key')
  return <Field>
    <FieldLabel>{t('API Key for invocation')}</FieldLabel>
    <Select value={value} onValueChange={next => { if (typeof next === 'string') onChange(next) }}>
      <SelectTrigger className='w-full' disabled={!keys.length}><SelectValue>{label}</SelectValue></SelectTrigger>
      <SelectContent>{keys.map(key => <SelectItem key={key.id} value={key.id}>{key.name}（{key.masked}）</SelectItem>)}</SelectContent>
    </Select>
    <FieldDescription>{t('The plaintext Key is only written into the local configuration files of the selected clients.')}</FieldDescription>
  </Field>
}

function TargetList({ targets, selected, onToggle }: { targets: InstallTarget[]; selected: string[]; onToggle: (key: string) => void }) {
  if (!targets.length) return <Empty>{t('No install targets available')}</Empty>
  return <div className='flex max-h-[320px] flex-col gap-1 overflow-auto rounded-md border border-border p-2'>
    {targets.map(target => <label key={target.key} className='flex cursor-pointer items-center gap-2 rounded-sm px-2 py-1.5 text-sm hover:bg-nav-hover'>
      <Checkbox checked={selected.includes(target.key)} disabled={target.supported === false} onCheckedChange={() => onToggle(target.key)} />
      <span className='min-w-0 flex-1'>
        <span className='font-medium'>{target.label}</span>
        <span className='ml-2 text-xs text-subtle'>{targetLabel(target)}</span>
        <span className='block truncate text-[11px] text-subtle' title={target.path}>{target.path}</span>
        {target.note && <span className='block text-xs text-warning'>{t(target.note)}</span>}
      </span>
    </label>)}
  </div>
}

function outcomesMessage(results: InstallOutcome[]): { ok: number; failed: InstallOutcome[] } {
  return { ok: results.filter(item => item.ok).length, failed: results.filter(item => !item.ok) }
}

export function ToolInstallDialog({ service, mode, onClose, onDone }: {
  service: ToolService; mode: Mode; onClose: () => void; onDone: () => void
}) {
  const [targets, setTargets] = React.useState<InstallTarget[]>([])
  const [keys, setKeys] = React.useState<ApiKeyOption[]>([])
  const [selected, setSelected] = React.useState<string[]>([])
  const [keyId, setKeyId] = React.useState('')
  const [variant, setVariant] = React.useState(service.recommended_variant === 'python' ? 'python' : 'node')
  const imageService = service.id === 'image-generation'
  const [mcpMode, setMcpMode] = React.useState<'bridge' | 'http'>(service.id === 'vision' || imageService ? 'bridge' : 'http')
  const [outputDirectory, setOutputDirectory] = React.useState('')
  const [referenceDirectories, setReferenceDirectories] = React.useState('')
  const [loading, setLoading] = React.useState(true)
  const [busy, setBusy] = React.useState(false)
  const [error, setError] = React.useState('')
  const [results, setResults] = React.useState<InstallOutcome[] | null>(null)

  React.useEffect(() => {
    let alive = true
    setLoading(true); setError('')
    Promise.all([fetchInstallTargets(service.id, mode), fetchApiKeys()])
      .then(([targetsResult, keysResult]) => {
        if (!alive) return
        setTargets(targetsResult.targets || [])
        setKeys(keysResult.keys || [])
        setSelected((targetsResult.targets || []).filter(target => target.detected && target.supported !== false).map(target => target.key))
        setKeyId(keysResult.keys?.[0]?.id || '')
      })
      .catch(failure => { if (alive) setError(failure instanceof Error ? failure.message : String(failure)) })
      .finally(() => { if (alive) setLoading(false) })
    return () => { alive = false }
  }, [service.id, mode])

  const toggle = (key: string) => setSelected(current => current.includes(key) ? current.filter(item => item !== key) : [...current, key])
  const availableTargets = targets.filter(target => target.supported !== false)
  const allSelected = availableTargets.length > 0 && availableTargets.every(target => selected.includes(target.key))
  const toggleAll = () => setSelected(allSelected ? [] : availableTargets.map(target => target.key))

  const submit = async () => {
    if (!selected.length) { setError(t('Select at least one target')); return }
    if (!keyId) { setError(t('Select an API Key first')); return }
    if (imageService && mcpMode === 'bridge' && !outputDirectory.trim()) { setError(t('Select an image output directory')); return }
    setBusy(true); setError(''); setResults(null)
    try {
      const response = mode === 'skill'
        ? await installSkill({ variant, api_key_id: keyId, clients: selected })
        : await installMcp({ service_id: service.id, mode: mcpMode, api_key_id: keyId, clients: selected,
          ...(imageService && mcpMode === 'bridge' ? { output_directory: outputDirectory.trim(),
            reference_directories: referenceDirectories.split('\n').map(value => value.trim()).filter(Boolean) } : {}) })
      const { ok, failed } = outcomesMessage(response.results || [])
      setResults(response.results || [])
      if (ok > 0) {
        toast(mode === 'skill' ? t('Installed to {count} clients', { count: ok }) : t('Wrote configuration for {count} clients', { count: ok }), 'ok')
        onDone()
        if (!failed.length) onClose()
      }
      if (failed.length) setError(failed.map(item => `${item.label}：${item.error || t('Write failed')}`).join('\n'))
    } catch (failure) {
      setError(failure instanceof Error ? failure.message : String(failure))
    } finally {
      setBusy(false)
    }
  }

  const serviceLabel = SERVICE_NAME[service.id] || service.id
  const title = mode === 'skill'
    ? t('Install {name}', { name: serviceLabel })
    : t('Connect {name} to clients', { name: serviceLabel })

  return <Dialog open onOpenChange={(open, details) => { if (!open) { if (busy) details.cancel(); else onClose() } }}>
    <DialogContent className='max-w-2xl'>
      <DialogHeader><DialogTitle>{title}</DialogTitle></DialogHeader>
      <DialogBody className='flex flex-col gap-3'>
        <p className='text-xs text-subtle'>{mode === 'skill'
          ? t('Select install targets, runtime and the API Key for invocation. The Skill package and key are written only into the selected clients\' local directories.')
          : t('Select the clients to write the MCP configuration into. Only this service\'s entry is changed; other MCP servers stay untouched.')}</p>
        {loading ? <div className='flex items-center gap-2 py-6 text-sm text-subtle'><Spinner />{t('Loading install targets…')}</div> : <>
          <div className='flex items-center gap-2'>
            <span className='text-sm font-medium'>{t('Install targets')}</span>
            <span className='text-xs text-subtle'>{t('Selected {count} / {total}', { count: selected.length, total: targets.length })}</span>
            <Button variant='ghost' size='sm' className='ml-auto' onClick={toggleAll}>{allSelected ? t('Deselect all') : t('Select all')}</Button>
          </div>
          <TargetList targets={targets} selected={selected} onToggle={toggle} />
          {mode === 'mcp' && selected.includes('dsh') && <Alert>{t('DSH uses the configuration target selected on the Agent page. After writing, check that the tools appear in DSH and restart the corresponding client if needed.')}</Alert>}
          {mode === 'skill' && <Field>
            <FieldLabel>{t('Runtime')}</FieldLabel>
            <SegmentedControl value={variant} onValueChange={value => { if (typeof value === 'string') setVariant(value) }} aria-label={t('Runtime')} options={[
              { value: 'node', label: 'Node.js' }, { value: 'python', label: 'Python' },
            ]} />
            <FieldDescription>{t('Node.js needs no extra dependencies and is recommended; Python requires python on this machine.')}</FieldDescription>
          </Field>}
          {mode === 'mcp' && (service.id === 'vision' || imageService) && <Field>
            <FieldLabel>{t('Connection mode')}</FieldLabel>
            <SegmentedControl value={mcpMode} onValueChange={value => { if (value === 'bridge' || value === 'http') setMcpMode(value) }} aria-label={t('Connection mode')} options={[
              { value: 'bridge', label: t('Local bridge') }, { value: 'http', label: t('Remote HTTP') },
            ]} />
            <FieldDescription>{imageService
              ? t('The local bridge requires Node.js 18 or newer. It reads references from authorized directories and saves generated images. HTTP mode only returns download URLs.')
              : mcpMode === 'bridge'
                ? t('The local bridge runs a Node script on this machine and supports local image paths; recommended on desktop.')
                : t('Remote HTTP only accepts image URLs or data URIs; no local Node.js is required.')}</FieldDescription>
          </Field>}
          {imageService && mcpMode === 'bridge' ? <>
            <Field>
              <FieldLabel>{t('Image output directory')}</FieldLabel>
              <div className='flex gap-2'><Input value={outputDirectory} onChange={event => setOutputDirectory(event.currentTarget.value)} placeholder={t('Select an existing directory; images are saved only there or in its subdirectories')} />
                <Button variant='outline' onClick={() => {
                  const bridge = (window as unknown as { workbuddyDesktop?: { pickFolder?: (initial?: string) => Promise<{ path?: string }> } }).workbuddyDesktop
                  void bridge?.pickFolder?.(outputDirectory).then(result => { if (result.path) setOutputDirectory(result.path) })
                    .catch(failure => setError(failure instanceof Error ? failure.message : String(failure)))
                }}>{t('Choose directory')}</Button></div>
            </Field>
            <Field>
              <FieldLabel>{t('Allowed reference directories')}</FieldLabel>
              <Textarea value={referenceDirectories} rows={3} onChange={event => setReferenceDirectories(event.currentTarget.value)} />
              <FieldDescription>{t('Optional: one existing absolute directory per line. The output directory is readable by default; files elsewhere are not read or uploaded.')}</FieldDescription>
            </Field>
            <Alert>{t('Only new files are saved; different existing content is never overwritten. Directory permissions are written to the client configuration and must be confirmed again when updating it.')}</Alert>
          </> : null}
          <KeyPicker keys={keys} value={keyId} onChange={setKeyId} />
          {imageService ? <Alert>{t('Client calls time out after 120 seconds. The service controls the generation deadline; query unfinished jobs by their ID.')}</Alert>
            : service.timeout_seconds ? <Alert>{t('The service\'s tool execution timeout ({seconds}s) will be written into the client configuration.', { seconds: service.timeout_seconds })}</Alert> : null}
          {results && <Alert variant='success'>{t('Completed {count} targets', { count: results.filter(item => item.ok).length })}</Alert>}
          {error && <Alert variant='destructive' className='whitespace-pre-wrap'>{error}</Alert>}
        </>}
      </DialogBody>
      <DialogFooter>
        <Button variant='outline' disabled={busy} onClick={onClose}>{t('Cancel')}</Button>
        <Button disabled={busy || loading || !selected.length || !keyId} onClick={() => void submit()}>
          {busy ? <Spinner /> : null}{mode === 'skill' ? t('Install') : t('Write configuration')}
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
}

export function ToolUninstallDialog({ service, mode, onClose, onDone }: {
  service: ToolService; mode: Mode; onClose: () => void; onDone: () => void
}) {
  const [targets, setTargets] = React.useState<InstallTarget[]>([])
  const [selected, setSelected] = React.useState<string[]>([])
  const [loading, setLoading] = React.useState(true)
  const [busy, setBusy] = React.useState(false)
  const [error, setError] = React.useState('')

  React.useEffect(() => {
    let alive = true
    fetchInstallTargets(service.id, mode)
      .then(result => {
        if (!alive) return
        const installed = (result.targets || []).filter(target => target.installed)
        setTargets(installed)
        setSelected(installed.map(target => target.key))
      })
      .catch(failure => { if (alive) setError(failure instanceof Error ? failure.message : String(failure)) })
      .finally(() => { if (alive) setLoading(false) })
    return () => { alive = false }
  }, [service.id, mode])

  const toggle = (key: string) => setSelected(current => current.includes(key) ? current.filter(item => item !== key) : [...current, key])

  const submit = async () => {
    if (!selected.length) { setError(t('Select at least one target')); return }
    setBusy(true); setError('')
    try {
      const response = mode === 'skill'
        ? await uninstallSkill({ service_id: service.id, clients: selected })
        : await uninstallMcp({ service_id: service.id, clients: selected })
      const { ok, failed } = outcomesMessage(response.results || [])
      if (ok > 0) {
        toast(t('Processed {count} targets', { count: ok }), 'ok')
        onDone()
        if (!failed.length) onClose()
      }
      if (failed.length) setError(failed.map(item => `${item.label}：${item.error || t('Uninstall failed')}`).join('\n'))
    } catch (failure) {
      setError(failure instanceof Error ? failure.message : String(failure))
    } finally {
      setBusy(false)
    }
  }

  return <Dialog open onOpenChange={(open, details) => { if (!open) { if (busy) details.cancel(); else onClose() } }}>
    <DialogContent className='max-w-xl'>
      <DialogHeader><DialogTitle>{t('Remove {name} configuration', { name: SERVICE_NAME[service.id] || service.id })}</DialogTitle></DialogHeader>
      <DialogBody className='flex flex-col gap-3'>
        <p className='text-xs text-subtle'>{t('Select the clients to remove it from; unselected clients are left unchanged.')}</p>
        {loading ? <div className='flex items-center gap-2 py-6 text-sm text-subtle'><Spinner />{t('Loading install targets…')}</div>
          : targets.length ? <TargetList targets={targets} selected={selected} onToggle={toggle} /> : <Empty>{t('当前没有已安装/已配置的目标')}</Empty>}
        {error && <Alert variant='destructive' className='whitespace-pre-wrap'>{error}</Alert>}
      </DialogBody>
      <DialogFooter>
        <Button variant='outline' disabled={busy} onClick={onClose}>{t('Cancel')}</Button>
        <Button variant='destructive' disabled={busy || !selected.length} onClick={() => void submit()}>{t('Uninstall')}</Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
}
