import { useEffect, useMemo, useState } from 'react'
import {
  Button, Card, Dialog, DialogBody, DialogContent, DialogFooter, DialogHeader, DialogTitle,
  Input, MultiSelect, Select, SelectContent, SelectItem, SelectTrigger, SelectValue, Switch, confirmDialog,
} from '@ui'
import { PageIntro, IconButton, Badge, Empty, ErrorNote, SkeletonList } from '../components/ui'
import { PullIndicator, usePullToRefresh } from '../components/PullToRefresh'
import { apiCall } from '../api/client'
import { store } from '../state/store'
import { usePolling } from '../lib/usePolling'
import { t } from '../i18n'
import { SearchableSelect } from '../../../ui-shared/searchable-select'
import { ImageGenerationSection } from './ToolsImageGeneration'

/**
 * MCP & Skills（手机端）。
 *
 * ── 手机能做什么、不能做什么 ────────────────────────────────
 * 服务开关、上游提供商密钥、提供商选择与视觉模型配置全部是**网关侧**的数据，
 * 手机上改完立刻生效；这部分与桌面端同一套接口。
 * 「本机接入」（把 Skill / MCP 配置写进客户端目录）作用在**网关所在机器**，
 * 手机上不做 —— 与「Agent 配置」页同一口径：手机上只读/远程操作，改本机文件回桌面端。
 */

type ToolProtocol = 'skill' | 'mcp' | 'api'
type ToolService = {
  id: 'web-search' | 'vision' | 'image-generation'; protocol: ToolProtocol; key: string; enabled: boolean
  capability_count: number; endpoint: string; version: string; timeout_seconds: number
}
type ProviderRow = {
  provider: string; display_name: string; configured: boolean; key_hint: string | null
  enabled: boolean; updated_at: number | null
  auth_type?: 'api_key' | 'account'; supports_fetch?: boolean; available?: boolean
  account_id?: string; account_name?: string; max_query_chars?: number | null; unavailable_reason?: string | null
  accounts?: Array<{ id: string; name: string; enabled: boolean }>
}
type Selection = { search_providers: string[]; fetch_provider: string }
type VisionConfig = {
  provider: string; provider_label: string; upstream_model: string; downstream_model: string
  reasoning: string; api_key_id: string; api_key_name: string; configured: boolean; key_available: boolean
}
type VisionKey = { id: string; name: string; key_prefix: string }
type ManageModel = { id: string; provider: string; enabled: boolean; kind?: string }
type ManageProvider = { provider: string; providerLabel?: string; count?: number }

const SERVICE_NAME: Record<string, string> = { 'web-search': t('Web Search'), vision: t('Vision Analysis'), 'image-generation': t('Image generation') }
const PROTOCOL_LABEL: Record<ToolProtocol, string> = { skill: 'Skill', mcp: 'MCP', api: 'API' }
const REASONING = ['off', 'none', 'minimal', 'low', 'medium', 'high', 'xhigh', 'max']

function message(failure: unknown): string {
  return t(failure instanceof Error ? failure.message : String(failure))
}

export function ToolsPage() {
  const [services, setServices] = useState<ToolService[]>([])
  const [providers, setProviders] = useState<ProviderRow[]>([])
  const [selection, setSelection] = useState<Selection>({ search_providers: [], fetch_provider: 'tavily' })
  const [vision, setVision] = useState<{ config: VisionConfig | null; keys: VisionKey[] }>({ config: null, keys: [] })
  const [manage, setManage] = useState<{ models: ManageModel[]; providers: ManageProvider[] }>({ models: [], providers: [] })
  const [error, setError] = useState('')
  const [loading, setLoading] = useState(true)
  const [busy, setBusy] = useState('')
  const [editing, setEditing] = useState<ProviderRow | null>(null)
  const [visionOpen, setVisionOpen] = useState(false)
  const [selectionOpen, setSelectionOpen] = useState(false)

  const load = async () => {
    try {
      const [servicesData, providersData, selectionData, visionData, manageData] = await Promise.all([
        apiCall<ToolService[]>('/api/tools/servers'),
        apiCall<ProviderRow[]>('/api/tools/providers'),
        apiCall<Selection>('/api/tools/provider-selection'),
        apiCall<{ config: VisionConfig | null; keys: VisionKey[] }>('/api/tools/vision/config'),
        apiCall<{ models: ManageModel[]; providers: ManageProvider[]; reasoningLevels?: string[] }>('/api/models/manage'),
      ])
      setServices((servicesData || []).filter(service => service.protocol !== 'api'))
      setProviders((providersData || []).map(provider => ({ ...provider, display_name: t(provider.display_name) })))
      setSelection(selectionData)
      setVision(visionData)
      setManage({ models: (manageData.models || []).filter(model => model.enabled && model.kind !== 'image'), providers: manageData.providers || [] })
      setError('')
    } catch (failure) {
      setError(message(failure))
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => { void load() }, [])
  usePolling(load, 120_000)
  const pull = usePullToRefresh({ onRefresh: load })

  const toggle = async (service: ToolService, enabled: boolean) => {
    setBusy(service.key)
    try {
      await apiCall(`/api/tools/servers/${service.id}`, { method: 'PATCH', body: { protocol: service.protocol, enabled } })
      setServices(current => current.map(item => item.key === service.key ? { ...item, enabled } : item))
      store.toast(t('Service status updated'))
    } catch (failure) {
      store.toast(message(failure), 'err')
    } finally {
      setBusy('')
    }
  }

  const toggleProvider = async (provider: ProviderRow, enabled: boolean) => {
    try {
      await apiCall(`/api/tools/providers/${provider.provider}`, { method: 'PUT', body: { enabled } })
      store.toast(t('Saved'))
      await load()
    } catch (failure) {
      store.toast(message(failure), 'err')
    }
  }

  const testProvider = async (provider: ProviderRow) => {
    setBusy(provider.provider)
    try {
      const result = await apiCall<{ success: boolean; result_count?: number; duration_ms?: number; error?: string; code?: string }>(
        `/api/tools/providers/${provider.provider}/test`, { method: 'POST' })
      if (result.success) store.toast(t('Connected: {count} results in {duration}ms', { count: result.result_count ?? 0, duration: result.duration_ms ?? 0 }))
      else store.toast(result.error ? t(result.error) : t('Connection test failed'), 'err')
    } catch (failure) {
      store.toast(message(failure), 'err')
    } finally {
      setBusy('')
    }
  }

  const clearProvider = async (provider: ProviderRow) => {
    try {
      await apiCall(`/api/tools/providers/${provider.provider}/key`, { method: 'DELETE' })
      store.toast(t('Key cleared'))
      await load()
    } catch (failure) {
      store.toast(message(failure), 'err')
    }
  }

  const providerOptions = providers.map(provider => ({ value: provider.provider, label: provider.display_name }))
  const fetchOptions = providers.filter(provider => provider.supports_fetch !== false)
    .map(provider => ({ value: provider.provider, label: provider.display_name }))
  const selectedProviders = providers.filter(provider => selection.search_providers.includes(provider.provider))
  const readyProviders = selectedProviders.filter(provider => provider.enabled && provider.configured)
  const missingProviders = selectedProviders.filter(provider => !(provider.enabled && provider.configured))
  const searchReady = readyProviders.length > 0
  const readyNames = readyProviders.map(provider => provider.display_name).join(', ')
  const missingNames = missingProviders.map(provider => provider.display_name).join(', ')

  return <div className='m-page' ref={pull.bindRef}>
    <PullIndicator pull={pull.pull} refreshing={pull.refreshing} />
    <PageIntro subtitle={t('Search, vision and image generation use gateway settings. Install MCP into local clients from the desktop app.')}
      actions={<IconButton name='refresh' label={t('Refresh')} onClick={() => void load()} />} />
    {error ? <ErrorNote text={error} /> : null}
    {loading && !services.length ? <SkeletonList rows={4} height={90} /> : null}

    {!loading ? <>
      <div className='m-section'>
        <Card className='m-card m-card-pad'>
          <div style={{ fontSize: 12.5, lineHeight: 1.6, color: 'var(--m-text-2)' }}>
            {t('Connect clients to search, vision and other capabilities through MCP or Skills.')}
          </div>
        </Card>
      </div>

      <div className='m-section'>
        <div className='m-section-head'><h2>{t('Services and protocols')}</h2></div>
        <Card className='m-card'>
          {services.map((service, index) => <div key={service.key} style={{ display: 'flex', alignItems: 'center', gap: 10, padding: '12px 14px', borderTop: index ? '1px solid var(--m-line)' : undefined }}>
            <div style={{ minWidth: 0, flex: 1 }}>
              <div style={{ fontSize: 13, fontWeight: 600 }}>
                {SERVICE_NAME[service.id] || service.id}
                <span style={{ marginLeft: 8, fontSize: 11, color: 'var(--m-text-3)' }}>{PROTOCOL_LABEL[service.protocol]}</span>
              </div>
              <div style={{ marginTop: 2, fontSize: 11.5, color: 'var(--m-text-3)' }}>
                <code>{service.endpoint}</code> · v{service.version} · {t('Timeout {seconds}s', { seconds: service.timeout_seconds })}
              </div>
            </div>
            <Switch checked={service.enabled} disabled={busy === service.key}
              onCheckedChange={checked => void toggle(service, checked === true)} aria-label={t('Enable / disable this service')} />
          </div>)}
          {!services.length ? <Empty text={t('No tool services available')} /> : null}
        </Card>
      </div>

      <div className='m-section'>
        <div className='m-section-head'>
          <h2>{t('Upstream provider')}</h2>
          <Button size='sm' variant='link' type='button' className='m-link px-0' onClick={() => setSelectionOpen(true)}>{t('Provider selection')}</Button>
        </div>
        <Card className='m-card m-card-pad'>
          <div style={{ fontSize: 12, color: searchReady ? 'var(--m-ok)' : 'var(--m-warn)', marginBottom: 8 }}>
            {searchReady
              ? (missingProviders.length
                ? t('Batch web search available: using {ready}; unconfigured {missing} will be skipped automatically.', { ready: readyNames, missing: missingNames })
                : t('Batch web search is available: all selected sources are enabled and configured.'))
              : t('Batch web search is unavailable: enable a source and configure its API key or bind an available account.')}
          </div>
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: 6, marginBottom: 8 }}>
            {selection.search_providers.map(item => <Badge key={item} tone='brand'>{item}</Badge>)}
            <Badge tone='mute'>{t('Fetch: {name}', { name: selection.fetch_provider })}</Badge>
          </div>
        </Card>
        {providers.map(provider => <Card key={provider.provider} className='m-card m-card-pad' style={{ marginTop: 8 }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
            <span style={{ fontSize: 13, fontWeight: 600 }}>{provider.display_name}</span>
            <Badge tone={provider.configured ? 'ok' : 'mute'}>{provider.configured ? t('Configured') : t('Not configured')}</Badge>
            <span style={{ marginLeft: 'auto' }} />
            <Switch checked={provider.enabled} disabled={provider.available === false && !provider.enabled} onCheckedChange={checked => void toggleProvider(provider, checked === true)} aria-label={t('enabled')} />
          </div>
          <div style={{ marginTop: 6, fontSize: 11.5, color: 'var(--m-text-3)' }}>
            {provider.auth_type === 'account' ? <>
              <div>{t('Existing account')} · {t('Search only')}</div>
              <div>{t('Search accounts')}: {provider.account_name || t('No account selected')}</div>
              {provider.max_query_chars ? <div>{t('Up to {count} characters per query', { count: provider.max_query_chars })}</div> : null}
              {provider.unavailable_reason ? <div style={{ color: 'var(--m-warn)' }}>{t(provider.unavailable_reason)}</div> : null}
            </> : <>{t('Key hint')}: {provider.key_hint || '-'}</>}
          </div>
          <div style={{ display: 'flex', gap: 6, marginTop: 8, flexWrap: 'wrap' }}>
            <Button size='sm' variant='outline' onClick={() => setEditing(provider)}>{t('Edit')}</Button>
            <Button size='sm' variant='outline' disabled={!provider.configured || busy === provider.provider} onClick={() => void testProvider(provider)}>{t('Test connection')}</Button>
            {provider.auth_type !== 'account' ? <Button size='sm' variant='ghost' disabled={!provider.configured} onClick={() => void confirmDialog(t('Clear the API Key of {name}? The provider will be unavailable until a new key is saved.', { name: provider.display_name }), { destructive: true }).then(ok => { if (ok) void clearProvider(provider) })}>{t('Clear key')}</Button> : null}
          </div>
        </Card>)}
      </div>

      <ImageGenerationSection onChanged={() => { void load() }} />

      <div className='m-section'>
        <div className='m-section-head'>
          <h2>{t('Vision Analysis')}</h2>
          <Button size='sm' variant='link' type='button' className='m-link px-0' onClick={() => setVisionOpen(true)}>{t('Profile')}</Button>
        </div>
        <Card className='m-card m-card-pad'>
          {!vision.config?.configured
            ? <div style={{ fontSize: 12.5, color: 'var(--m-warn)' }}>{t('No vision model is configured yet; callers receive a "not configured" error until it is.')}</div>
            : vision.config.key_available === false
              ? <div style={{ fontSize: 12.5, color: 'var(--m-danger)' }}>{t('The configured API Key is no longer valid (deleted or disabled); select a new one and save.')}</div>
              : <div style={{ fontSize: 12.5, color: 'var(--m-text-2)' }}>{t('Current model: {provider} / {model}', { provider: vision.config.provider_label || vision.config.provider, model: vision.config.upstream_model })}</div>}
          <div style={{ marginTop: 8, display: 'flex', gap: 8 }}>
            <Button size='sm' variant='outline' disabled={!vision.config?.configured} onClick={async () => {
              try {
                const result = await apiCall<{ success: boolean; duration_ms?: number; error?: string }>('/api/tools/vision/config/test', { method: 'POST' })
                if (result.success) store.toast(t('Vision pipeline connected in {duration}ms', { duration: result.duration_ms ?? 0 }))
                else store.toast(result.error || t('Vision pipeline test failed'), 'err')
              } catch (failure) { store.toast(message(failure), 'err') }
            }}>{t('Test connection')}</Button>
          </div>
        </Card>
      </div>
    </> : null}

    {editing ? <ProviderDialog provider={editing} onClose={() => setEditing(null)} onSaved={() => { setEditing(null); void load() }} /> : null}
    {selectionOpen ? <SelectionDialog
      providers={providerOptions}
      fetchProviders={fetchOptions}
      value={selection}
      onClose={() => setSelectionOpen(false)}
      onSaved={next => { setSelection(next); setSelectionOpen(false); void load() }}
    /> : null}
    {visionOpen ? <VisionDialog
      config={vision.config}
      keys={vision.keys}
      providers={(manage.providers || []).filter(item => (item.count ?? 0) > 0)}
      models={manage.models}
      onClose={() => setVisionOpen(false)}
      onSaved={() => { setVisionOpen(false); void load() }}
    /> : null}
  </div>
}

function ProviderDialog({ provider, onClose, onSaved }: { provider: ProviderRow; onClose: () => void; onSaved: () => void }) {
  const [apiKey, setApiKey] = useState('')
  const [accountId, setAccountId] = useState(provider.account_id || '')
  const isAccount = provider.auth_type === 'account'
  const accounts = (provider.accounts || []).filter(account => account.enabled)
  const accountLabel = accounts.find(account => account.id === accountId)?.name
    || (accountId ? t('The bound account is unavailable; select another account') : t('No account selected'))
  const [enabled, setEnabled] = useState(provider.enabled)
  const [busy, setBusy] = useState(false)
  const save = async () => {
    setBusy(true)
    try {
      await apiCall(`/api/tools/providers/${provider.provider}`, { method: 'PUT', body: isAccount
        ? { account_id: accountId || null, enabled }
        : { api_key: apiKey.trim() || undefined, enabled } })
      store.toast(t('Saved'))
      onSaved()
    } catch (failure) { store.toast(message(failure), 'err') }
    finally { setBusy(false) }
  }
  return <Dialog open onOpenChange={open => { if (!open && !busy) onClose() }}>
    <DialogContent>
      <DialogHeader><DialogTitle>{t('Edit {name}', { name: provider.display_name })}</DialogTitle></DialogHeader>
      <DialogBody className='flex flex-col gap-3'>
        {isAccount ? <div>
          <div style={{ fontSize: 12.5, marginBottom: 6 }}>{t('Search accounts')}</div>
          <SearchableSelect value={accountId || '__none__'} className='w-full' displayValue={accountLabel}
            aria-label={t('Search accounts')} searchPlaceholder={t('Search accounts…')}
            onValueChange={value => setAccountId(value === '__none__' ? '' : value)} options={[
              { value: '__none__', label: t('No account selected') },
              ...(accountId && !accounts.some(account => account.id === accountId)
                ? [{ value: accountId, label: accountLabel, disabled: true }] : []),
              ...accounts.map(account => ({ value: account.id, label: account.name })),
            ]} />
          <p style={{ fontSize: 11.5, color: 'var(--m-text-3)' }}>{t('Uses an existing login from the Accounts page; no additional search API key is required. Disabled or deleted accounts are not replaced automatically.')}</p>
          {!accounts.length ? <ErrorNote text={t('No enabled account is available to bind. Add or enable one on the Accounts page first.')} /> : null}
          {provider.available === false && provider.unavailable_reason ? <ErrorNote text={t(provider.unavailable_reason)} /> : null}
        </div> : <label style={{ fontSize: 12.5 }}>{t('API Key')}
          <Input type='password' autoComplete='new-password' value={apiKey} placeholder={t('Enter a new key; leave empty to keep the current one')} onChange={event => setApiKey(event.currentTarget.value)} />
        </label>}
        <div style={{ display: 'flex', alignItems: 'center', gap: 8, fontSize: 12.5 }}>
          {t('enabled')}<Switch checked={enabled} disabled={isAccount && provider.available === false && !enabled} onCheckedChange={checked => setEnabled(checked === true)} aria-label={t('enabled')} />
        </div>
      </DialogBody>
      <DialogFooter>
        <Button variant='outline' disabled={busy} onClick={onClose}>{t('Cancel')}</Button>
        <Button disabled={busy} onClick={() => void save()}>{t('Save')}</Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
}

function SelectionDialog({ providers, fetchProviders, value, onClose, onSaved }: {
  providers: Array<{ value: string; label: string }>; fetchProviders: Array<{ value: string; label: string }>; value: Selection
  onClose: () => void; onSaved: (value: Selection) => void
}) {
  const [draft, setDraft] = useState(value)
  const [busy, setBusy] = useState(false)
  const save = async () => {
    if (!draft.search_providers.length) { store.toast(t('Select at least one search provider'), 'err'); return }
    setBusy(true)
    try {
      const next = await apiCall<Selection>('/api/tools/provider-selection', { method: 'PUT', body: draft })
      onSaved(next)
    } catch (failure) { store.toast(message(failure), 'err') }
    finally { setBusy(false) }
  }
  const fetchLabel = fetchProviders.find(item => item.value === draft.fetch_provider)?.label || draft.fetch_provider
  return <Dialog open onOpenChange={open => { if (!open && !busy) onClose() }}>
    <DialogContent>
      <DialogHeader><DialogTitle>{t('Provider selection')}</DialogTitle></DialogHeader>
      <DialogBody className='flex flex-col gap-3'>
        <div>
          <div style={{ fontSize: 12.5, marginBottom: 6 }}>{t('Batch web search providers')}</div>
          <MultiSelect value={draft.search_providers} onValueChange={items => setDraft(current => ({ ...current, search_providers: items }))}
            options={providers} placeholder={t('Select a provider')} aria-label={t('Batch web search providers')} />
        </div>
        <div>
          <div style={{ fontSize: 12.5, marginBottom: 6 }}>{t('Batch page fetch provider')}</div>
          <SearchableSelect value={draft.fetch_provider} options={fetchProviders} className='w-full'
            aria-label={t('Batch page fetch provider')} searchPlaceholder={t('Search providers')} displayValue={fetchLabel}
            onValueChange={fetch_provider => setDraft(current => ({ ...current, fetch_provider }))} />
          <p style={{ fontSize: 11.5, color: 'var(--m-text-3)' }}>{t('Account-based sources support search only, not page fetching. Requests use the bound account’s quota and outbound network settings.')}</p>
        </div>
      </DialogBody>
      <DialogFooter>
        <Button variant='outline' disabled={busy} onClick={onClose}>{t('Cancel')}</Button>
        <Button disabled={busy} onClick={() => void save()}>{t('Save')}</Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
}

function VisionDialog({ config, keys, providers, models, onClose, onSaved }: {
  config: VisionConfig | null; keys: VisionKey[]
  providers: ManageProvider[]; models: ManageModel[]
  onClose: () => void; onSaved: () => void
}) {
  const [provider, setProvider] = useState(config?.provider || '')
  const [model, setModel] = useState(config?.upstream_model || '')
  const [reasoning, setReasoning] = useState(config?.reasoning || '')
  const [keyId, setKeyId] = useState(config?.api_key_id || keys[0]?.id || '')
  const [busy, setBusy] = useState(false)

  const providerOptions = useMemo(() => providers.map(item => ({ value: item.provider, label: item.providerLabel || item.provider })), [providers])
  const modelOptions = useMemo(() => models.filter(item => item.provider === provider).map(item => ({ value: item.id, label: item.id })), [models, provider])
  const keyOptions = keys.map(key => ({ value: key.id, label: `${key.name}（${key.key_prefix}）` }))
  const savedKeyMissing = Boolean(config?.api_key_id) && !keys.some(key => key.id === config?.api_key_id)
  const keyLabel = keyOptions.find(item => item.value === keyId)?.label
    || (savedKeyMissing ? `${config?.api_key_name || config?.api_key_id}（${t('Invalid')}）` : t('Select an API Key'))

  const save = async () => {
    if (!provider || !model) { store.toast(t('Select a provider and an upstream model'), 'err'); return }
    if (!keyId) { store.toast(t('Select the internal API Key'), 'err'); return }
    setBusy(true)
    try {
      await apiCall('/api/tools/vision/config', { method: 'PUT', body: { provider, upstream_model: model, reasoning_effort: reasoning, api_key_id: keyId } })
      store.toast(t('Vision service configuration saved'))
      onSaved()
    } catch (failure) { store.toast(message(failure), 'err') }
    finally { setBusy(false) }
  }

  return <Dialog open onOpenChange={open => { if (!open && !busy) onClose() }}>
    <DialogContent>
      <DialogHeader><DialogTitle>{t('Vision model configuration')}</DialogTitle></DialogHeader>
      <DialogBody className='flex flex-col gap-3'>
        <div>
          <div style={{ fontSize: 12.5, marginBottom: 6 }}>{t('Upstream provider')}</div>
          <SearchableSelect value={provider} options={providerOptions} className='w-full'
            aria-label={t('Upstream provider')} searchPlaceholder={t('Search providers')}
            displayValue={providerOptions.find(item => item.value === provider)?.label || t('Select a provider')}
            onValueChange={value => { setProvider(value); setModel('') }} />
        </div>
        <div>
          <div style={{ fontSize: 12.5, marginBottom: 6 }}>{t('Upstream model')}</div>
          <SearchableSelect value={model} options={modelOptions} onValueChange={setModel} disabled={!provider}
            className='w-full' aria-label={t('Upstream model')} searchPlaceholder={t('Search models')}
            displayValue={model || (provider ? t('Select a model') : t('Select a provider first'))} />
        </div>
        <div>
          <div style={{ fontSize: 12.5, marginBottom: 6 }}>{t('Reasoning level')}</div>
          <Select value={reasoning} onValueChange={next => { if (typeof next === 'string') setReasoning(next) }}>
            <SelectTrigger className='w-full'><SelectValue>{reasoning || t('Not specified')}</SelectValue></SelectTrigger>
            <SelectContent><SelectItem value=''>{t('Not specified')}</SelectItem>{REASONING.map(level => <SelectItem key={level} value={level}>{level}</SelectItem>)}</SelectContent>
          </Select>
        </div>
        <div>
          <div style={{ fontSize: 12.5, marginBottom: 6 }}>{t('Internal API Key')}</div>
          <SearchableSelect value={keyId} options={keyOptions} onValueChange={setKeyId} className='w-full'
            aria-label={t('Internal API Key')} displayValue={keyLabel} />
        </div>
        {!providers.length ? <div style={{ fontSize: 11.5, color: 'var(--m-warn)' }}>{t('No providers available yet. Add accounts on desktop and make sure there are enabled models.')}</div> : null}
      </DialogBody>
      <DialogFooter>
        <Button variant='outline' disabled={busy} onClick={onClose}>{t('Cancel')}</Button>
        <Button disabled={busy} onClick={() => void save()}>{t('Save')}</Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
}
