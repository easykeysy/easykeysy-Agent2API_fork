import * as React from 'react'
import {
  Alert, Button, Card, CardContent, CardHeader, CardTitle, Dialog, DialogBody, DialogContent,
  DialogFooter, DialogHeader, DialogTitle, Empty, Field, FieldDescription, FieldLabel, Input,
  MultiSelect, Select, SelectContent, SelectItem, SelectTrigger, SelectValue, Spinner, Switch,
  confirmDialog,
} from '@ui'
import { t } from '../i18n'
import { SearchableSelect } from '../../../ui-shared/searchable-select'
import { callJson } from './assistant-state'
import { openExternal } from './update-shared'
import {
  PROVIDER_KEY_PAGES, clearProviderKey, clearVision, fetchProviders, fetchSelection, fetchVision,
  formatTime, saveProvider, saveSelection, saveVision, testProvider, testVision,
  type ProviderRow, type ProviderSelection, type ProviderTestResult, type VisionConfig as VisionConfigRow,
  type VisionKeyOption,
} from './tools-data'

/* ─── 网络搜索：上游提供商 ─────────────────────────────────── */

/** 稳定错误码 → 界面文案（代码是后端口径，文案是中文键，便于扫描进词典） */
const TEST_FAILURE_TEXT: Record<string, string> = {
  invalid_provider_key: t('The API key or account login is invalid'),
  rate_limited: t('The upstream rate limit was hit'),
  timed_out: t('The upstream request timed out'),
  no_results: t('The upstream returned no results'),
  provider_unavailable: t('The provider is temporarily unavailable or misconfigured'),
  configuration: t('The search source configuration is incomplete'),
  permission_denied: t('This gateway Key is not allowed to use the selected account-based search sources'),
  invalid_input: t('Invalid request parameters'),
}

function testFailureText(result: ProviderTestResult): string {
  const text = result.code ? TEST_FAILURE_TEXT[result.code] : ''
  return result.error ? t(result.error) : text || t('Connection test failed')
}

export function WebSearchConfig() {
  const [providers, setProviders] = React.useState<ProviderRow[]>([])
  const [selection, setSelection] = React.useState<ProviderSelection>({ search_providers: [], fetch_provider: 'tavily' })
  const [loading, setLoading] = React.useState(true)
  const [saving, setSaving] = React.useState(false)
  const [error, setError] = React.useState('')
  const [notice, setNotice] = React.useState('')
  const [testing, setTesting] = React.useState('')
  const [editing, setEditing] = React.useState<ProviderRow | null>(null)

  /**
   * `silent` = 只刷新数据、不切换到「正在加载」骨架。
   *
   * 开关提供商、清密钥这类「就地更新」必须走 silent：切骨架会把整块配置
   * 卸载重挂，页面高度瞬间归零，浏览器就把滚动位置拉回顶部 —— 用户点一下
   * 开关，页面自己跳回最上面。
   */
  const load = React.useCallback(async (silent = false) => {
    if (!silent) setLoading(true)
    setError('')
    try {
      const [providersResult, selectionResult] = await Promise.all([fetchProviders(), fetchSelection()])
      setProviders(providersResult || [])
      setSelection(selectionResult)
    } catch (failure) {
      setError(t(failure instanceof Error ? failure.message : String(failure)))
    } finally {
      if (!silent) setLoading(false)
    }
  }, [])

  React.useEffect(() => { void load() }, [load])

  const options = providers.map(provider => ({ value: provider.provider, label: provider.display_name }))
  const fetchOptions = providers.filter(provider => provider.supports_fetch !== false)
    .map(provider => ({ value: provider.provider, label: provider.display_name }))
  const selected = new Set(selection.search_providers)
  // 选择范围和可用状态分开：密钥或账号配置失效时，明确显示会跳过哪些来源。
  const readySelected = providers.filter(provider => selected.has(provider.provider) && provider.enabled && provider.configured)
  const missing = providers.filter(provider => selected.has(provider.provider) && !(provider.enabled && provider.configured))
  const aggregateReady = readySelected.length > 0
  const readyNames = readySelected.map(provider => provider.display_name).join(', ')
  const missingNames = missing.map(provider => provider.display_name).join(', ')

  const saveProviders = async () => {
    if (!selection.search_providers.length) { setError(t('Select at least one provider for batch web search')); return }
    setSaving(true); setError(''); setNotice('')
    try {
      const next = await saveSelection(selection)
      setSelection(next)
      setNotice(t('Capability provider settings saved'))
    } catch (failure) {
      setError(t(failure instanceof Error ? failure.message : String(failure)))
    } finally {
      setSaving(false)
    }
  }

  const toggle = async (provider: ProviderRow, enabled: boolean) => {
    setError('')
    try {
      await saveProvider(provider.provider, { enabled })
      await load(true)
    } catch (failure) {
      setError(t(failure instanceof Error ? failure.message : String(failure)))
    }
  }

  const test = async (provider: ProviderRow) => {
    setTesting(provider.provider); setError(''); setNotice('')
    try {
      const result = await testProvider(provider.provider)
      if (result.success) {
        setNotice(t('Connected: {count} results in {duration}ms', { count: result.result_count ?? 0, duration: result.duration_ms ?? 0 }))
      } else {
        setError(testFailureText(result))
      }
    } catch (failure) {
      setError(t(failure instanceof Error ? failure.message : String(failure)))
    } finally {
      setTesting('')
    }
  }

  const clearKey = async (provider: ProviderRow) => {
    setError(''); setNotice('')
    try {
      await clearProviderKey(provider.provider)
      setNotice(t('Key cleared'))
      await load(true)
    } catch (failure) {
      setError(t(failure instanceof Error ? failure.message : String(failure)))
    }
  }

  if (loading) return <div className='flex items-center gap-2 p-4 text-sm text-subtle'><Spinner />{t('Loading tool service configuration…')}</div>

  return <Card>
    <CardHeader><CardTitle>{t('Upstream provider configuration')}</CardTitle></CardHeader>
    <CardContent className='flex flex-col gap-4'>
      <p className='text-xs text-subtle'>{t('Configure the upstream providers used for search and fetching. Callers no longer pass a provider; the saved settings are always used at runtime.')}</p>
      <Alert>{t('Upstream provider settings are shared by all access protocols; changes take effect for all of them.')}</Alert>
      {error && <Alert variant='destructive' className='whitespace-pre-wrap'>{error}</Alert>}
      {notice && <Alert variant='success'>{notice}</Alert>}

      <div className='grid grid-cols-1 gap-4 md:grid-cols-2'>
        <Field>
          <FieldLabel>{t('Batch web search providers')}</FieldLabel>
          <MultiSelect value={selection.search_providers} onValueChange={value => setSelection(current => ({ ...current, search_providers: value }))}
            options={options} placeholder={t('Select a provider')} aria-label={t('Batch web search providers')} />
          <FieldDescription>{t('Multiple selection. Each search calls the selected providers in parallel and merges duplicate sources.')}</FieldDescription>
        </Field>
        <Field>
          <FieldLabel>{t('Batch page fetch provider')}</FieldLabel>
          <SearchableSelect value={selection.fetch_provider} options={fetchOptions} className='w-full'
            aria-label={t('Batch page fetch provider')} searchPlaceholder={t('Search providers')}
            onValueChange={fetch_provider => setSelection(current => ({ ...current, fetch_provider }))} />
          <FieldDescription>{t('Single selection only; defaults to Tavily.')}</FieldDescription>
        </Field>
      </div>
      <div><Button disabled={saving} onClick={() => void saveProviders()}>{saving ? <Spinner /> : null}{t('Save capability settings')}</Button></div>

      {aggregateReady
        ? <Alert variant='success'>{missing.length
          ? t('Batch web search available: using {ready}; unconfigured {missing} will be skipped automatically.', { ready: readyNames, missing: missingNames })
          : t('Batch web search is available: all selected sources are enabled and configured.')}</Alert>
        : <Alert variant='warning'>{t('Batch web search is unavailable: enable a source and configure its API key or bind an available account.')}</Alert>}
      <p className='text-xs text-subtle'>{t('Account-based sources support search only, not page fetching. Requests use the bound account’s quota and outbound network settings.')}</p>

      <div className='grid grid-cols-1 gap-3 md:grid-cols-2 xl:grid-cols-4'>
        {providers.map(provider => <div key={provider.provider} className='flex flex-col gap-2 rounded-md border border-border p-3'>
          <div className='flex items-center gap-2'>
            <span className='font-semibold'>{provider.display_name}</span>
            <Switch className='ml-auto' checked={provider.enabled} disabled={provider.available === false && !provider.enabled} onCheckedChange={checked => void toggle(provider, checked === true)} aria-label={t('enabled')} />
          </div>
          <div className='flex flex-wrap gap-1.5 text-[11px]'>
            <span className={provider.configured ? 'text-success' : 'text-subtle'}>{provider.configured ? t('Configured') : t('Not configured')}</span>
            <span className='text-subtle'>·</span>
            <span className={provider.enabled ? 'text-primary' : 'text-subtle'}>{provider.enabled ? t('Enabled') : t('Disabled')}</span>
          </div>
          <div className='text-[11.5px] text-subtle'>
            {provider.auth_type === 'account' ? <>
              <div>{t('Existing account')} · {t('Search only')}</div>
              <div>{t('Search accounts')}: <span className='text-foreground'>{provider.account_name || t('No account selected')}</span></div>
              {provider.max_query_chars ? <div>{t('Up to {count} characters per query', { count: provider.max_query_chars })}</div> : null}
              {provider.unavailable_reason ? <div className='mt-1 text-warning'>{t(provider.unavailable_reason)}</div> : null}
            </> : <div>{t('Key hint')}: <span className='text-foreground'>{provider.key_hint || '-'}</span></div>}
            <div>{t('Updated')}: {formatTime(provider.updated_at)}</div>
          </div>
          <div className='mt-auto flex flex-wrap gap-1.5'>
            <Button size='sm' variant='outline' onClick={() => setEditing(provider)}>{t('Edit')}</Button>
            <Button size='sm' variant='outline' disabled={!provider.configured || testing === provider.provider} onClick={() => void test(provider)}>
              {testing === provider.provider ? <Spinner /> : null}{t('Test connection')}
            </Button>
            {provider.auth_type !== 'account' ? <Button size='sm' variant='ghost' disabled={!provider.configured} onClick={() => {
              void confirmDialog(t('Clear the API Key of {name}? The provider will be unavailable until a new key is saved.', { name: provider.display_name }), { destructive: true })
                .then(ok => { if (ok) void clearKey(provider) })
            }}>{t('Clear key')}</Button> : null}
          </div>
        </div>)}
        {!providers.length && <Empty>{t('No providers to configure')}</Empty>}
      </div>
    </CardContent>
    {editing && <ProviderDialog provider={editing} onClose={() => setEditing(null)} onSaved={() => { setEditing(null); void load(true) }} />}
  </Card>
}

function ProviderDialog({ provider, onClose, onSaved }: { provider: ProviderRow; onClose: () => void; onSaved: () => void }) {
  const [apiKey, setApiKey] = React.useState('')
  const [accountId, setAccountId] = React.useState(provider.account_id || '')
  const isAccount = provider.auth_type === 'account'
  const accounts = (provider.accounts || []).filter(account => account.enabled)
  const accountLabel = accounts.find(account => account.id === accountId)?.name
    || (accountId ? t('The bound account is unavailable; select another account') : t('No account selected'))
  const [enabled, setEnabled] = React.useState(provider.enabled)
  const [busy, setBusy] = React.useState(false)
  const [error, setError] = React.useState('')

  const save = async () => {
    setBusy(true); setError('')
    try {
      await saveProvider(provider.provider, isAccount
        ? { account_id: accountId || null, enabled }
        : { api_key: apiKey.trim() || undefined, enabled })
      onSaved()
    } catch (failure) {
      setError(t(failure instanceof Error ? failure.message : String(failure)))
    } finally {
      setBusy(false)
    }
  }

  return <Dialog open onOpenChange={(open, details) => { if (!open) { if (busy) details.cancel(); else onClose() } }}>
    <DialogContent className='max-w-lg'>
      <DialogHeader><DialogTitle>{t('Edit provider {name}', { name: provider.display_name })}</DialogTitle></DialogHeader>
      <DialogBody className='flex flex-col gap-3'>
        {isAccount ? <Field>
          <FieldLabel>{t('Search accounts')}</FieldLabel>
          <SearchableSelect value={accountId || '__none__'} className='w-full' displayValue={accountLabel}
            aria-label={t('Search accounts')} searchPlaceholder={t('Search accounts…')}
            onValueChange={value => setAccountId(value === '__none__' ? '' : value)} options={[
              { value: '__none__', label: t('No account selected') },
              ...(accountId && !accounts.some(account => account.id === accountId)
                ? [{ value: accountId, label: accountLabel, disabled: true }] : []),
              ...accounts.map(account => ({ value: account.id, label: account.name })),
            ]} />
          <FieldDescription>{t('Uses an existing login from the Accounts page; no additional search API key is required. Disabled or deleted accounts are not replaced automatically.')}</FieldDescription>
          {!accounts.length ? <Alert variant='warning'>{t('No enabled account is available to bind. Add or enable one on the Accounts page first.')}</Alert> : null}
          {provider.available === false && provider.unavailable_reason ? <Alert variant='warning'>{t(provider.unavailable_reason)}</Alert> : null}
        </Field> : <Field>
          <FieldLabel>{t('API Key')}</FieldLabel>
          <Input type='password' autoComplete='new-password' value={apiKey} placeholder={t('Enter a new key; leave empty to keep the current one')} onChange={event => setApiKey(event.currentTarget.value)} />
          <FieldDescription>{t('An empty API Key will not overwrite the saved key.')}</FieldDescription>
        </Field>}
        {PROVIDER_KEY_PAGES[provider.provider] ? <div>
          <Button variant='link' size='sm' className='h-auto px-0' onClick={() => void openExternal(PROVIDER_KEY_PAGES[provider.provider])}>
            {t('Get API Key')}
          </Button>
        </div> : null}
        <div className='flex items-center gap-2'>
          <span className='text-sm'>{t('enabled')}</span>
          <Switch checked={enabled} disabled={isAccount && provider.available === false && !enabled} onCheckedChange={checked => setEnabled(checked === true)} aria-label={t('enabled')} />
        </div>
        {error && <Alert variant='destructive'>{error}</Alert>}
      </DialogBody>
      <DialogFooter><Button variant='outline' disabled={busy} onClick={onClose}>{t('Cancel')}</Button><Button disabled={busy} onClick={() => void save()}>{busy ? <Spinner /> : null}{t('Save')}</Button></DialogFooter>
    </DialogContent>
  </Dialog>
}

/* ─── 视觉分析 ─────────────────────────────────────────────── */

type ManageModel = { id: string; provider: string; enabled: boolean; kind?: string }
type ManageProvider = { provider: string; providerLabel?: string; count?: number }

const REASONING_LABELS: Record<string, string> = {
  off: 'off', none: 'none', minimal: 'minimal', low: 'low', medium: 'medium', high: 'high', xhigh: 'xhigh', max: 'max',
}

export function VisionConfigPanel() {
  const [config, setConfig] = React.useState<VisionConfigRow | null>(null)
  const [keys, setKeys] = React.useState<VisionKeyOption[]>([])
  const [providers, setProviders] = React.useState<ManageProvider[]>([])
  const [models, setModels] = React.useState<ManageModel[]>([])
  const [reasoningLevels, setReasoningLevels] = React.useState<string[]>(Object.keys(REASONING_LABELS))
  const [provider, setProvider] = React.useState('')
  const [model, setModel] = React.useState('')
  const [reasoning, setReasoning] = React.useState('')
  const [keyId, setKeyId] = React.useState('')
  const [loading, setLoading] = React.useState(true)
  const [saving, setSaving] = React.useState(false)
  const [testing, setTesting] = React.useState(false)
  const [error, setError] = React.useState('')
  const [notice, setNotice] = React.useState('')

  // 与 WebSearchConfig 的 load 同一理由：保存后只刷新数据，不切骨架
  const load = React.useCallback(async (silent = false) => {
    if (!silent) setLoading(true)
    setError('')
    try {
      const [visionResult, manage] = await Promise.all([
        fetchVision(),
        callJson<{ providers?: ManageProvider[]; models?: ManageModel[]; reasoningLevels?: string[] }>('GET', '/api/models/manage'),
      ])
      setConfig(visionResult.config)
      setKeys(visionResult.keys || [])
      setProviders((manage.providers || []).filter(item => (item.count ?? 0) > 0))
      setModels((manage.models || []).filter(item => item.enabled && item.kind !== 'image'))
      if (manage.reasoningLevels?.length) setReasoningLevels(manage.reasoningLevels)
      const stored = visionResult.config
      setProvider(stored?.provider || '')
      setModel(stored?.upstream_model || '')
      setReasoning(stored?.reasoning || '')
      setKeyId(stored?.api_key_id && visionResult.keys.some(key => key.id === stored.api_key_id) ? stored.api_key_id : (visionResult.keys[0]?.id || ''))
    } catch (failure) {
      setError(t(failure instanceof Error ? failure.message : String(failure)))
    } finally {
      if (!silent) setLoading(false)
    }
  }, [])

  React.useEffect(() => { void load() }, [load])

  const providerOptions = providers.map(item => ({ value: item.provider, label: item.providerLabel || item.provider }))
  const modelOptions = models.filter(item => item.provider === provider).map(item => ({ value: item.id, label: item.id }))
  const keyOptions = keys.map(key => ({ value: key.id, label: `${key.name}（${key.key_prefix}）` }))
  const savedKeyMissing = config?.api_key_id && !keys.some(key => key.id === config.api_key_id)

  const save = async () => {
    if (!provider || !model) { setError(t('Select a provider and an upstream model')); return }
    if (!keyId) { setError(t('Select the internal API Key')); return }
    setSaving(true); setError(''); setNotice('')
    try {
      const saved = await saveVision({ provider, upstream_model: model, reasoning_effort: reasoning, api_key_id: keyId })
      setConfig(saved)
      setNotice(t('Vision service configuration saved'))
      await load(true)
    } catch (failure) {
      setError(t(failure instanceof Error ? failure.message : String(failure)))
    } finally {
      setSaving(false)
    }
  }

  const test = async () => {
    setTesting(true); setError(''); setNotice('')
    try {
      const result = await testVision()
      if (result.success) setNotice(t('Vision pipeline connected in {duration}ms', { duration: result.duration_ms ?? 0 }))
      else setError(result.error || t('Vision pipeline test failed'))
    } catch (failure) {
      setError(t(failure instanceof Error ? failure.message : String(failure)))
    } finally {
      setTesting(false)
    }
  }

  const clear = async () => {
    setError(''); setNotice('')
    try {
      await clearVision()
      setConfig(null); setProvider(''); setModel(''); setReasoning('')
      setNotice(t('Vision service configuration cleared'))
    } catch (failure) {
      setError(t(failure instanceof Error ? failure.message : String(failure)))
    }
  }

  if (loading) return <div className='flex items-center gap-2 p-4 text-sm text-subtle'><Spinner />{t('Loading tool service configuration…')}</div>

  return <Card>
    <CardHeader>
      <CardTitle>{t('Vision model configuration')}</CardTitle>
      <div className='ml-auto flex gap-2'>
        <Button variant='outline' disabled={!config?.configured || testing} onClick={() => void test()}>{testing ? <Spinner /> : null}{t('Test connection')}</Button>
        {config?.configured ? <Button variant='ghost' disabled={saving} onClick={() => void clear()}>{t('Clear configuration')}</Button> : null}
      </div>
    </CardHeader>
    <CardContent className='flex flex-col gap-4'>
      <p className='text-xs text-subtle'>{t('Select the upstream provider and model for images. Calls go through this gateway with the internal Key, reusing protocol translation, retries and failover.')}</p>
      {!config?.configured
        ? <Alert variant='warning'>{t('No vision model is configured yet; callers receive a "not configured" error until it is.')}</Alert>
        : config.key_available === false
          ? <Alert variant='destructive'>{t('The configured API Key is no longer valid (deleted or disabled); select a new one and save.')}</Alert>
          : <Alert variant='success'>{t('Current model: {provider} / {model}', { provider: config.provider_label || config.provider, model: config.upstream_model })}</Alert>}
      {error && <Alert variant='destructive' className='whitespace-pre-wrap'>{error}</Alert>}
      {notice && <Alert variant='success'>{notice}</Alert>}
      {!providers.length && <Alert variant='warning'>{t('No providers available yet. Add accounts on the Accounts page and make sure there are enabled models on the Models page.')}</Alert>}

      <div className='grid grid-cols-1 gap-4 md:grid-cols-2'>
        <Field>
          <FieldLabel>{t('Upstream provider')}</FieldLabel>
          <SearchableSelect value={provider} options={providerOptions} className='w-full'
            aria-label={t('Upstream provider')} searchPlaceholder={t('Search providers')}
            displayValue={providerOptions.find(option => option.value === provider)?.label || t('Select a provider')}
            onValueChange={value => { setProvider(value); setModel('') }} />
          <FieldDescription>{t('Only providers that are enabled and have at least one available model are shown.')}</FieldDescription>
        </Field>
        <Field>
          <FieldLabel>{t('Upstream model')}</FieldLabel>
          <SearchableSelect value={model} options={modelOptions} onValueChange={setModel} disabled={!provider}
            className='w-full' aria-label={t('Upstream model')} searchPlaceholder={t('Search models')}
            displayValue={model || (provider ? t('Select a model') : t('Select a provider first'))} />
          <FieldDescription>{t('Only enabled models under this provider are shown.')}</FieldDescription>
        </Field>
        <Field>
          <FieldLabel>{t('Reasoning level')}</FieldLabel>
          <Select value={reasoning} onValueChange={value => { if (typeof value === 'string') setReasoning(value) }}>
            <SelectTrigger className='w-full'><SelectValue>{reasoning || t('Not specified')}</SelectValue></SelectTrigger>
            <SelectContent>
              <SelectItem value=''>{t('Not specified')}</SelectItem>
              {reasoningLevels.map(level => <SelectItem key={level} value={level}>{REASONING_LABELS[level] || level}</SelectItem>)}
            </SelectContent>
          </Select>
          <FieldDescription>{t('Optional. Leave empty to omit it and let the mapping\'s reasoning override decide.')}</FieldDescription>
        </Field>
        <Field>
          <FieldLabel>{t('Internal API Key')}</FieldLabel>
          <SearchableSelect value={keyId} onValueChange={setKeyId} className='w-full'
            aria-label={t('Internal API Key')} placeholder={t('Select an API Key')}
            displayValue={keyOptions.find(option => option.value === keyId)?.label || t('Select an API Key')}
            options={[
              ...(savedKeyMissing ? [{ value: config?.api_key_id || '',
                label: `${config?.api_key_name || config?.api_key_id}（${t('Invalid')}）`, disabled: true }] : []),
              ...keyOptions,
            ]} />
          <FieldDescription>{t('The gateway Key used for internal requests to this gateway; quota, rate limits and request logs are recorded on that Key.')}</FieldDescription>
        </Field>
      </div>
      <div><Button disabled={saving} onClick={() => void save()}>{saving ? <Spinner /> : null}{t('Save')}</Button></div>
    </CardContent>
  </Card>
}
