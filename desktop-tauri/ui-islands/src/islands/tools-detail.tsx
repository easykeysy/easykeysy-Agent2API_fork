import * as React from 'react'
import {
  Alert, Badge, Button, Card, CardContent, CardHeader, CardTitle, Field, FieldLabel, Input,
  SegmentedControl, Spinner, Switch,
} from '@ui'
import { t } from '../i18n'
import { toast } from './assistant-state'
import { WebSearchConfig, VisionConfigPanel } from './tools-config'
import { ImageGenerationConfigPanel } from './tools-image-config'
import {
  PROTOCOL_DESC, PROTOCOL_LABELS, SERVICE_DESC, SERVICE_NAME, fetchInstallInfo, patchServer,
  type InstallInfo, type ToolProtocol, type ToolService,
} from './tools-data'

const CAPABILITY_DESCRIPTIONS: Record<string, { title: string; desc: string; params: Array<{ name: string; type: string; desc: string; required: boolean }> }[]> = {
  'web-search': [
    {
      title: t('Batch web search'),
      desc: t('Search 1-10 queries in parallel with the providers selected on the server, then merge duplicate sources.'),
      params: [{ name: 'queries', type: 'string[]', desc: t('Queries to search, each executed in parallel (1-10)'), required: true }],
    },
    {
      title: t('Batch page fetching'),
      desc: t('Fetch 1-10 URLs using the single fetch provider selected on the server.'),
      params: [{ name: 'urls', type: 'string[]', desc: t('Page URLs to fetch and extract (1-10)'), required: true }],
    },
  ],
  'image-generation': [
    { title: t('Generate images'), desc: t('Generate images from a description using the configured model. Defaults to one image and consumes account credits.'), params: [
      { name: 'prompt', type: 'string', desc: t('Complete image requirements'), required: true },
      { name: 'request_id', type: 'string', desc: t('Unique submission ID; reuse it when retrying after a disconnect'), required: true },
    ] },
    { title: t('Edit images'), desc: t('Edit reference images; available only when a supported editing model is configured.'), params: [
      { name: 'images', type: 'string[]', desc: t('Reference URLs or generated image IDs; the local bridge also accepts file paths'), required: true },
    ] },
    { title: t('Get generation result'), desc: t('Query the original job and retrieve its images without generating or charging again.'), params: [
      { name: 'job_id / request_id', type: 'string', desc: t('Provide a job ID or the original submission ID'), required: true },
    ] },
  ],
  vision: [
    {
      title: t('Image analysis'),
      desc: t('Send 1-10 images and a prompt to the vision model configured on the server and return the model reply text.'),
      params: [
        { name: 'images', type: 'string[]', desc: t('Images to analyze (1-10), each an http(s) URL or an image data URI'), required: true },
        { name: 'prompt', type: 'string', desc: t('Prompt describing what the model should look for in the images'), required: true },
      ],
    },
  ],
}

function copyText(text: string) {
  if (navigator.clipboard?.writeText) {
    return navigator.clipboard.writeText(text).then(() => true).catch(() => false)
  }
  return Promise.resolve(false)
}

function McpConnectCard({ info, protocol }: { info: InstallInfo; protocol: ToolProtocol }) {
  if (protocol !== 'mcp') return null
  const snippet = JSON.stringify({
    mcpServers: {
      [info.mcp.server_name]: {
        type: 'http',
        url: info.mcp.url,
        headers: { Authorization: 'Bearer <Agent2API API Key>' },
      },
    },
  }, null, 2)
  return <Card>
    <CardHeader><CardTitle>{t('MCP connection details')}</CardTitle></CardHeader>
    <CardContent className='flex flex-col gap-3'>
      <div>
        <div className='text-sm font-medium'>{t('Service URL')}</div>
        <div className='mt-1 flex items-center gap-2'>
          <code className='min-w-0 flex-1 break-all rounded-sm bg-surface-2 px-2 py-1 text-xs'>{info.mcp.url}</code>
          <Button size='sm' variant='outline' onClick={() => void copyText(info.mcp.url).then(ok => toast(ok ? t('Copied') : t('Copy failed; select and copy manually'), ok ? 'ok' : 'err'))}>{t('Copy')}</Button>
        </div>
        <p className='mt-1 text-[11.5px] text-subtle'>{t('Fill in this URL in any client that supports remote MCP, and use a gateway Key as the Bearer token.')}</p>
      </div>
      <div>
        <div className='text-sm font-medium'>{t('Exposed tools')}</div>
        <div className='mt-1 flex flex-wrap gap-1.5'>{info.mcp.tools.map(name => <Badge key={name}>{name}</Badge>)}</div>
      </div>
      <div>
        <div className='flex items-center gap-2'>
          <div className='text-sm font-medium'>{t('Client configuration example')}</div>
          <Button size='sm' variant='outline' className='ml-auto' onClick={() => void copyText(snippet).then(ok => toast(ok ? t('Copied') : t('Copy failed; select and copy manually'), ok ? 'ok' : 'err'))}>{t('Copy')}</Button>
        </div>
        <p className='mt-1 text-[11.5px] text-subtle'>{t('Merge the snippet below into the client\'s mcpServers and replace the Key with your own.')}</p>
        <pre className='mt-2 max-h-64 overflow-auto rounded-md bg-surface-2 p-3 text-[11.5px] leading-relaxed'>{snippet}</pre>
      </div>
    </CardContent>
  </Card>
}

export function ToolDetail({ service, onBack, onChanged }: { service: ToolService; onBack: () => void; onChanged: () => void }) {
  const [info, setInfo] = React.useState<InstallInfo | null>(null)
  const [loading, setLoading] = React.useState(true)
  const [error, setError] = React.useState('')
  const [enabled, setEnabled] = React.useState(service.enabled)
  const [toggling, setToggling] = React.useState(false)
  const [timeoutDraft, setTimeoutDraft] = React.useState<number>(service.timeout_seconds)
  const [savedTimeout, setSavedTimeout] = React.useState<number>(service.timeout_seconds)
  const [savingTimeout, setSavingTimeout] = React.useState(false)
  const [tab, setTab] = React.useState<'config' | 'capabilities'>('config')

  React.useEffect(() => {
    let alive = true
    setLoading(true); setError('')
    fetchInstallInfo(service.id)
      .then(result => { if (alive) setInfo(result) })
      .catch(failure => { if (alive) setError(failure instanceof Error ? failure.message : String(failure)) })
      .finally(() => { if (alive) setLoading(false) })
    return () => { alive = false }
  }, [service.id, service.capability_count])

  React.useEffect(() => { setTimeoutDraft(service.timeout_seconds); setSavedTimeout(service.timeout_seconds) }, [service.timeout_seconds])
  React.useEffect(() => { setEnabled(service.enabled) }, [service.enabled])

  const toggle = async (enabled: boolean) => {
    setToggling(true); setError('')
    try {
      await patchServer(service.id, { protocol: service.protocol, enabled })
      setEnabled(enabled)
      toast(t('Service status updated'), 'ok')
      onChanged()
    } catch (failure) {
      setError(failure instanceof Error ? failure.message : String(failure))
    } finally {
      setToggling(false)
    }
  }

  const saveTimeout = async () => {
    if (!timeoutDraft) return
    setSavingTimeout(true); setError('')
    try {
      await patchServer(service.id, { protocol: service.protocol, timeout_seconds: timeoutDraft })
      setSavedTimeout(timeoutDraft)
      toast(t('Tool execution timeout saved'), 'ok')
      onChanged()
    } catch (failure) {
      setError(failure instanceof Error ? failure.message : String(failure))
    } finally {
      setSavingTimeout(false)
    }
  }

  const capabilities = (CAPABILITY_DESCRIPTIONS[service.id] || [])
    .filter(capability => service.id !== 'image-generation' || capability.title !== t('Edit images') || info?.mcp.tools.includes('edit_images'))

  return <div className='flex flex-col gap-4'>
    <Button variant='ghost' size='sm' className='self-start' onClick={onBack}>← {t('Back to list')}</Button>

    <Card>
      <CardHeader>
        <CardTitle>{SERVICE_NAME[service.id] || service.id}</CardTitle>
        <div className='ml-2 flex flex-wrap items-center gap-2'>
          <Badge>{PROTOCOL_LABELS[service.protocol]}</Badge>
          <Badge variant={enabled ? 'success' : 'outline'}>{enabled ? t('Enabled') : t('Disabled')}</Badge>
          <span className='text-xs text-subtle'>{t('Capabilities')}: {service.capability_count}</span>
          <span className='text-xs text-subtle'>v{service.version}</span>
        </div>
        <div className='ml-auto flex items-center gap-2'>
          <span className='text-xs text-subtle'>{t('Enable / disable this service')}</span>
          <Switch checked={enabled} disabled={toggling} onCheckedChange={checked => void toggle(checked === true)} aria-label={t('Enable / disable this service')} />
        </div>
      </CardHeader>
      <CardContent className='flex flex-col gap-3'>
        <p className='text-xs text-subtle'>{SERVICE_DESC[service.id] || ''}</p>
        <p className='text-xs text-subtle'>{PROTOCOL_DESC[service.protocol]}</p>
        <div className='flex flex-wrap items-end gap-2'>
          <Field className='w-[180px]'>
            <FieldLabel>{t('Tool execution timeout')}</FieldLabel>
            <Input type='number' min={5} max={600} value={String(timeoutDraft)} onChange={event => setTimeoutDraft(Number(event.currentTarget.value) || 0)} />
          </Field>
          <span className='pb-2 text-xs text-subtle'>{t('seconds')}</span>
          <Button size='sm' variant='outline' disabled={loading || savingTimeout || !timeoutDraft || timeoutDraft === savedTimeout} onClick={() => void saveTimeout()}>
            {savingTimeout ? <Spinner /> : null}{t('Save')}
          </Button>
        </div>
        <p className='text-[11.5px] text-subtle'>{service.id === 'image-generation'
          ? t('This limits the background generation task. Each MCP call waits up to 20 seconds, then returns a job ID if unfinished. Queries never generate again.')
          : t('Server-enforced timeout for a single tool execution (shared by the Skill / MCP protocols of the same service). Installing MCP writes this value into the client configuration; after changing it, re-run "Update" in the client and reopen the session.')}</p>
        {!enabled && <Alert variant='warning'>{t('This protocol is disabled; connected clients will receive a service-disabled error.')}</Alert>}
        {error && <Alert variant='destructive' className='whitespace-pre-wrap'>{error}</Alert>}
      </CardContent>
    </Card>

    {loading ? <div className='flex items-center gap-2 text-sm text-subtle'><Spinner />{t('Loading tool service configuration…')}</div>
      : service.protocol === 'mcp' && info ? <McpConnectCard info={info} protocol={service.protocol} />
        : service.protocol === 'skill' ? <Alert>{t('This protocol is installed into local client directories from the Agent2API panel.')}</Alert> : null}

    <div className='flex items-center gap-2'>
      <SegmentedControl value={tab} onValueChange={value => { if (value === 'config' || value === 'capabilities') setTab(value) }} options={[
        { value: 'config', label: t('Service configuration') }, { value: 'capabilities', label: t('Capabilities') },
      ]} aria-label={t('Service configuration')} />
    </div>

    {tab === 'config'
      ? (service.id === 'image-generation' ? <ImageGenerationConfigPanel onChanged={onChanged} />
        : service.id === 'vision' ? <VisionConfigPanel /> : <WebSearchConfig />)
      : <Card>
        <CardHeader><CardTitle>{t('Capabilities')}</CardTitle><span className='ml-auto text-xs text-subtle'>{t('{count} capabilities', { count: service.capability_count })}</span></CardHeader>
        <CardContent className='grid grid-cols-1 gap-3 md:grid-cols-2'>
          {capabilities.map(capability => <div key={capability.title} className='rounded-md border border-border p-3'>
            <div className='text-sm font-semibold'>{capability.title}</div>
            <p className='mt-1 text-xs text-subtle'>{capability.desc}</p>
            <div className='mt-2 flex flex-col gap-1.5'>
              {capability.params.map(param => <div key={param.name} className='flex flex-wrap items-baseline gap-2 text-[11.5px]'>
                <code className='rounded-sm bg-surface-2 px-1.5 py-0.5'>{param.name}</code>
                <span className='text-subtle'>{param.type}</span>
                <span className={param.required ? 'text-destructive' : 'text-subtle'}>{param.required ? t('Required') : t('Optional')}</span>
                <span className='w-full text-subtle'>{param.desc}</span>
              </div>)}
            </div>
          </div>)}
        </CardContent>
      </Card>}
  </div>
}
