import * as React from 'react'
import { Button, Collapsible, CollapsibleContent, CollapsibleTrigger, Field, FieldLabel, FieldDescription, Input, Select, SelectTrigger, SelectValue, SelectContent, SelectItem } from '@ui'
import { t } from '../i18n'
import type { AgentClient, AgentProfile } from './agent-types'
import type { CatalogModel } from './agent-catalog'
import { protocolLabel, syncClaudeTiers } from './agent-model'
import { getAgentStore } from './agent-state'
import { SearchableModelSelect } from './agent-model-picker'
import { AgentIcon } from './agent-actions'
import { SearchableSelect } from '../../../ui-shared/searchable-select'

export function AgentSelect({ value, options, onChange, disabled, label }: {
  value: string; options: Array<{ value: string; label: string }>; onChange: (value: string) => void; disabled?: boolean; label: string
}) {
  const selected = options.find(option => option.value === value)?.label || value || t('Select an option')
  return <Select value={value} disabled={disabled} onValueChange={next => { if (typeof next === 'string') onChange(next) }}>
    <SelectTrigger aria-label={label} title={selected} className='w-full'><SelectValue>{selected}</SelectValue></SelectTrigger>
    <SelectContent>{options.map(option => <SelectItem key={option.value} value={option.value}>{option.label}</SelectItem>)}</SelectContent>
  </Select>
}

export function TextField({ label, fieldKey, value, onChange, hint, type = 'text', disabled, placeholder }: {
  label: string; fieldKey: string; value: string; onChange: (value: string) => void; hint?: string; type?: string; disabled?: boolean; placeholder?: string
}) {
  const id = React.useId()
  return <Field><FieldLabel htmlFor={id} title={fieldKey || undefined}>{label}</FieldLabel>
    <Input id={id} type={type} autoComplete={type === 'password' ? 'new-password' : 'off'} spellCheck={false} value={value}
      onChange={event => onChange(event.currentTarget.value)} disabled={disabled} placeholder={placeholder} />
    {hint && <FieldDescription>{hint}</FieldDescription>}
  </Field>
}

/**
 * 网关模型下拉：列目录里的模型；当前值不在目录里时补一项，避免导入或手填的值被悄悄清掉。
 *
 * 「模型 ID 让用户敲」这件事只在读不到目录时才成立 —— 能选就不该让人手输。
 */
export function GatewayModelSelect({ label, value, catalog, onChange, disabled, allowEmpty, emptyLabel }: {
  label: string; value: string; catalog: CatalogModel[]; onChange: (value: string) => void
  disabled?: boolean; allowEmpty?: boolean; emptyLabel?: string
}) {
  const options = catalog.map(model => ({ value: model.id, label: model.label }))
  if (value && !options.some(option => option.value === value)) options.unshift({ value, label: t('{id} (not in gateway catalog)', { id: value }) })
  if (allowEmpty) options.unshift({ value: '', label: emptyLabel || t('Not set') })
  return <SearchableModelSelect label={label} value={value} options={options} disabled={disabled} onChange={onChange} />
}

/**
 * Claude Code 的四档模型映射（Opus / Sonnet / Haiku / 子 Agent）。
 *
 * Claude Code 自己按档位挑模型（主模型、后台小任务、子代理各走一档），所以接网关时
 * 不是「选一个模型」而是「每档指一个网关模型」—— 四档挤成一列全宽输入框既占地方又
 * 要求用户背模型 ID，这里按两列排、每档给下拉，另配「一键设置」把主模型铺到四档。
 */
export function ClaudeModelFields({ draft, update, catalog, disabled, note }: {
  draft: AgentProfile; update: (patch: Partial<AgentProfile>) => void; catalog: CatalogModel[]; disabled: boolean; note?: string
}) {
  const rows: Array<{ key: 'opus' | 'sonnet' | 'haiku' | 'subagent'; label: React.ReactNode; labelText: string; fieldKey: string }> = [
    { key: 'opus', label: 'Opus', labelText: 'Opus', fieldKey: 'ANTHROPIC_DEFAULT_OPUS_MODEL' },
    { key: 'sonnet', label: 'Sonnet', labelText: 'Sonnet', fieldKey: 'ANTHROPIC_DEFAULT_SONNET_MODEL' },
    { key: 'haiku', label: 'Haiku', labelText: 'Haiku', fieldKey: 'ANTHROPIC_DEFAULT_HAIKU_MODEL' },
    { key: 'subagent', label: t('Subagent'), labelText: t('Subagent'), fieldKey: 'CLAUDE_CODE_SUBAGENT_MODEL' },
  ]
  const pick = draft.kind === 'gateway' && catalog.length > 0
  const fill = () => update({ claudeModels: { opus: draft.model, sonnet: draft.model, haiku: draft.model, subagent: draft.model } })
  const setTier = (key: string, value: string) => update({ claudeModels: { ...draft.claudeModels, [key]: value } })
  return <section className='agent-claude-models flex min-w-0 flex-col gap-3 rounded-md border border-border bg-surface-inset p-3'>
    <div className='flex flex-wrap items-center gap-2'>
      <h4 className='text-xs font-semibold'>{t('Model mapping')}</h4><span className='mr-auto' />
      <Button variant='outline' size='xs' disabled={disabled || !draft.model} onClick={fill}>{t('Fill all')}</Button>
    </div>
    <div className='grid min-w-0 grid-cols-1 gap-3 sm:grid-cols-2'>
      {rows.map(row => <Field key={row.key}>
        <FieldLabel>{row.label}<code className='ml-auto text-[10px] text-subtle'>{row.fieldKey}</code></FieldLabel>
        {pick ? <GatewayModelSelect label={row.labelText} value={draft.claudeModels[row.key]} catalog={catalog} disabled={disabled} allowEmpty
          emptyLabel={t('Not set')} onChange={value => setTier(row.key, value)} />
          : <Input value={draft.claudeModels[row.key]} autoComplete='off' spellCheck={false} disabled={disabled} placeholder={t('Leave blank to skip')} onChange={event => setTier(row.key, event.currentTarget.value)} />}
      </Field>)}
    </div>
    <p className='text-xs leading-relaxed text-subtle'>{note || t('Claude Code requests models per tier (Opus / Sonnet / Haiku / subagent). Point each tier at a model from the gateway catalog; leave a tier blank to leave that field untouched.')}</p>
    {pick && rows.some(row => !draft.claudeModels[row.key]) && <p className='text-xs leading-relaxed text-warning'>{t('A blank tier makes Claude Code request the gateway with its built-in model name; that tier fails when the gateway has no model by that name. Use "Fill all" instead.')}</p>}
  </section>
}

export function ConnectionFields({ client, draft, update, disabled }: {
  client: AgentClient; draft: AgentProfile; update: (patch: Partial<AgentProfile>) => void; disabled: boolean
}) {
  const store = getAgentStore()
  const [advanced, setAdvanced] = React.useState(false)
  const authOptions = client.id === 'zed' ? [{ value: 'preserve', label: t('Manage authentication in client') }]
    : [ ...(client.supportsGateway && draft.kind === 'gateway' ? [{ value: 'gateway', label: t('Use gateway key') }] : []),
      { value: 'key', label: t('API Key') }, ...(client.supportsEnvKey ? [{ value: 'env', label: t('Environment variable') }] : []),
      { value: 'none', label: t('No authentication') }, ...(draft.sourceId ? [{ value: 'preserve', label: t('Keep existing authentication') }] : []) ]
  const switchSource = (kind: string) => {
    const baseUrl = kind === 'gateway' ? draft.protocol === 'anthropic' ? store.gateway : `${store.gateway}/v1` : draft.baseUrl
    update({ kind, baseUrl, authMode: client.id === 'zed' ? 'preserve' : kind === 'gateway' ? 'gateway' : draft.authMode === 'gateway' ? 'key' : draft.authMode })
  }
  return <div className='agent-connection-fields grid min-w-0 grid-cols-1 gap-4 sm:grid-cols-2'>
    <Field className='sm:col-span-2'><FieldLabel>{t('Connection type')}</FieldLabel><AgentSelect label={t('Connection type')} value={draft.kind} disabled={disabled} onChange={switchSource}
      options={[...(client.supportsGateway ? [{ value: 'gateway', label: t('This gateway') }] : []), { value: 'official', label: t('Official direct') }, { value: 'relay', label: t('Third-party relay') }, { value: 'local', label: t('Local service') }]} /></Field>
    <div className='sm:col-span-2'><TextField label={t('API base URL')} fieldKey={urlField(client.id)} value={draft.baseUrl} onChange={baseUrl => update({ baseUrl })} disabled={disabled}
      hint={client.id === 'workbuddy' ? t('The complete /chat/completions path is written automatically.') : draft.protocol === 'gemini' ? t('Enter the service root URL. Do not append /v1beta or a generateContent path.') : draft.protocol === 'anthropic' ? t('Do not append /v1/messages; the client adds the request path.') : t('Usually ends with /v1. Do not append a specific request path.')} /></div>
    <Field><FieldLabel>{t('Authentication method')}</FieldLabel><AgentSelect label={t('Authentication method')} value={draft.authMode} options={authOptions} onChange={authMode => update({ authMode })} disabled={disabled} /></Field>
    {draft.authMode === 'gateway' && store.requiresKey && <Field><FieldLabel>{t('Gateway key')}</FieldLabel>
      <SearchableSelect aria-label={t('Gateway key')} value={draft.gatewayKeyId} disabled={disabled}
        className='w-full' onValueChange={gatewayKeyId => update({ gatewayKeyId })}
        options={[{ value: '', label: t('First enabled key') }, ...store.gatewayKeys.map(key => ({ value: key.id, label: key.name }))]} />
      <FieldDescription>{t('Applying uses a currently enabled real key; previews hide the secret.')}</FieldDescription></Field>}
    {draft.authMode === 'gateway' && !store.requiresKey && <p className='text-xs text-subtle'>{t('API key authentication is not enabled on this gateway. A placeholder is used only when a client requires a non-empty key.')}</p>}
    {draft.authMode === 'key' && <TextField label={t('API Key')} fieldKey={client.id === 'codex' ? 'experimental_bearer_token' : client.id === 'claude-code' ? 'env.ANTHROPIC_API_KEY' : 'apiKey'} type='password' value={draft.apiKey} onChange={apiKey => update({ apiKey })} disabled={disabled}
      placeholder={draft.hasApiKey ? t('Leave blank to keep the saved key') : t('Enter a key')} hint={t('Keys are stored encrypted in Agent2API and written in the format required by the client when used.')} />}
    {draft.authMode === 'env' && <TextField label={t('Environment variable name')} fieldKey='env_key' value={draft.envKey} onChange={envKey => update({ envKey })} disabled={disabled}
      hint={t('Only the variable reference is saved. Set its value in the environment used to launch the client; the system environment is not modified.')} />}
    {client.id === 'zed' && <p className='text-xs leading-relaxed text-subtle'>{t('Save the API key in Zed\'s provider settings. This page does not modify the system credential store.')}</p>}
    <Collapsible open={advanced} onOpenChange={setAdvanced} className='sm:col-span-2'>
      <CollapsibleTrigger render={<Button variant='ghost' size='sm' />}><AgentIcon name='chevron' />{t('Advanced options')}<span className='text-xs text-subtle'>{protocolLabel(draft.protocol)}</span></CollapsibleTrigger>
      <CollapsibleContent className='pt-3'>
        <Field><FieldLabel>{t('Client protocol')}</FieldLabel><AgentSelect label={t('Client protocol')} value={draft.protocol} disabled={disabled || client.protocols.length === 1 && client.protocols.includes(draft.protocol)}
          onChange={protocol => update({ protocol, ...(draft.kind === 'gateway' ? { baseUrl: protocol === 'anthropic' ? store.gateway : `${store.gateway}/v1` } : {}) })}
          options={client.protocols.map(value => ({ value, label: protocolLabel(value) }))} />
          <FieldDescription>{t('Only protocols actually supported by the client can be selected here. The upstream provider protocol is unchanged.')}</FieldDescription></Field>
      </CollapsibleContent>
    </Collapsible>
  </div>
}

function urlField(id: string): string {
  return { 'claude-code': 'env.ANTHROPIC_BASE_URL', codex: 'model_providers.<id>.base_url', 'gemini-cli': 'GOOGLE_GEMINI_BASE_URL', opencode: 'provider.<id>.options.baseURL', zcode: 'config.api.baseUrl', workbuddy: 'models[].url', cline: 'providers.<id>.settings.baseUrl', 'kimi-code': 'providers.<id>.base_url', crush: 'providers.<id>.base_url', dsh: 'llm-pi-ai.providers.<id>.baseURL', zed: 'language_models.openai_compatible.<id>.api_url' }[id] || 'baseUrl'
}

export function SingleModelFields({ client, draft, update, catalog, disabled }: {
  client: AgentClient; draft: AgentProfile; update: (patch: Partial<AgentProfile>) => void; catalog: CatalogModel[]; disabled: boolean
}) {
  const selected = catalog.find(model => model.id === draft.model)
  const allowed = client.id === 'claude-code' ? ['low','medium','high','xhigh','max','auto'] : ['none','minimal','low','medium','high','xhigh']
  const levels = selected?.reasoningLevels.filter(level => allowed.includes(level)) || []
  const pick = draft.kind === 'gateway' && catalog.length > 0
  const modelKey = client.id === 'claude-code' ? 'model / ANTHROPIC_MODEL' : client.id === 'gemini-cli' ? 'model.name' : 'model'
  // 改主模型时把仍跟随的档一起带过去；Claude Code 的四档通常就等于主模型。
  const changeModel = (model: string) => update(client.id === 'claude-code'
    ? { model, effort: '', claudeModels: syncClaudeTiers(draft.claudeModels, draft.model, model) }
    : { model, effort: '' })
  return <>
    <div className='grid min-w-0 grid-cols-1 gap-4 sm:grid-cols-2'>
      <Field><FieldLabel>{pick ? t('Select a model from the gateway catalog') : t('Model ID')}<code className='ml-auto text-[10px] text-subtle'>{modelKey}</code></FieldLabel>
        {pick ? <GatewayModelSelect label={t('Select a model from the gateway catalog')} value={draft.model} catalog={catalog} disabled={disabled} onChange={changeModel} />
          : <Input value={draft.model} autoComplete='off' spellCheck={false} disabled={disabled} onChange={event => changeModel(event.currentTarget.value)} />}
        <FieldDescription>{t('Use the actual model ID, not its display name.')}</FieldDescription>
      </Field>
      {client.supportsEffort && (levels.length > 0 || draft.effort) && <Field><FieldLabel>{t('Reasoning level')}<code className='ml-auto text-[10px]'>{client.id === 'codex' ? 'model_reasoning_effort' : 'CLAUDE_CODE_EFFORT_LEVEL'}</code></FieldLabel>
        <AgentSelect label={t('Reasoning level')} value={draft.effort} disabled={disabled} onChange={effort => update({ effort })}
          options={[{ value: '', label: t('Model default') }, ...[...new Set([...levels, ...(draft.effort ? [draft.effort] : [])])].map(value => ({ value, label: value }))]} /></Field>}
    </div>
    {client.id === 'claude-code' && <ClaudeModelFields draft={draft} update={update} catalog={catalog} disabled={disabled} />}
    {client.supportsEffort && levels.length === 0 && !draft.effort && <p className='text-xs text-subtle'>{t('The model does not declare reasoning levels. Client defaults are preserved; guessed options are not shown.')}</p>}
  </>
}
