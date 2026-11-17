import * as React from 'react'
import {
  Alert, Badge, Button, Collapsible, CollapsibleContent, CollapsibleTrigger,
  Field, FieldDescription, FieldLabel, Input,
  InputGroup, InputGroupAddon, InputGroupButton, InputGroupInput, Label,
  RadioGroup, RadioGroupItem, Select, SelectContent, SelectItem, SelectTrigger,
  SelectValue, Switch,
} from '@ui'
import { t } from '../i18n'
import { shared } from './accounts-shared'

export type ProviderSettings = {
  id: string
  name?: string
  protocol?: string
  baseUrl?: string
  clientEmulation?: string
}

export function AccountSettingsIcon({ name, size = 16 }: { name: string; size?: number }) {
  return <span className='acct-dlg-icon' aria-hidden='true'
    dangerouslySetInnerHTML={{ __html: shared().wbIcons?.icon?.(name, size) || '' }} />
}

/**
 * 一组设置：分组名在方框外，框内的行以 1px 细线分隔（macOS 系统设置的分组列表）。
 *
 * 标题不塞进框里，是为了让「标题 → 框」的层级留在 DOM 上 —— 样式
 * （`acct-dlg-group-label` + `acct-dlg-group`）与读屏顺序都跟着这个结构走。
 * `action` 是分组名右侧的附加件（计数徽章、清除按钮），没有就不渲染。
 */
export function SettingGroup({ title, action, children }: {
  title: React.ReactNode
  action?: React.ReactNode
  children: React.ReactNode
}) {
  return <section className='acct-dlg-group-wrap'>
    <div className='acct-dlg-group-label'>
      <span>{title}</span>
      {action ? <span className='acct-dlg-label-action'>{action}</span> : null}
    </div>
    <div className='acct-dlg-group'>{children}</div>
  </section>
}

export function BalanceTokenField({ configured, value, onChange, onClear, clearBusy }: {
  configured: boolean
  value: string
  onChange: (next: string) => void
  onClear: () => void
  clearBusy: boolean
}) {
  return <SettingGroup title={t('Balance query credential')}
    action={<Badge shape='tag' variant={configured ? 'success' : 'secondary'}>{configured ? t('Configured') : t('Not configured')}</Badge>}>
    <Field className='acct-dlg-field'>
      <div className='acct-dlg-field-head'><FieldLabel htmlFor='account-balance-token-input'>{t('Balance query credential')}</FieldLabel>
        {configured ? <Button variant='ghost' size='xs' disabled={clearBusy} onClick={onClear}
          className='acct-dlg-clear'>{t('Clear')}</Button> : null}
      </div>
      <Input id='account-balance-token-input' type='password' autoComplete='new-password'
        placeholder={configured ? t('Configured; leave blank to keep') : t('Usually left blank')}
        value={value} onChange={event => onChange(event.currentTarget.value)} />
      <FieldDescription>{t('Points queries reuse the login credential used for forwarding, so this is usually left blank. It only has a value if an older version filled it in or credentials were imported from the old proxy')}</FieldDescription>
    </Field>
  </SettingGroup>
}

export function CustomCredentialFields({ hasApiKey, noAuth, value, onValue, onToggleNoAuth, onClear, clearBusy }: {
  hasApiKey: boolean
  noAuth: boolean
  value: string
  onValue: (next: string) => void
  onToggleNoAuth: (next: boolean) => void
  onClear: () => void
  clearBusy: boolean
}) {
  const [revealed, setRevealed] = React.useState(false)
  const pendingKey = Boolean(value.trim())
  return <SettingGroup title={t('Account credentials')}
    action={<Badge shape='tag' variant={noAuth || pendingKey ? 'secondary' : hasApiKey ? 'success' : 'warning'}>
      {noAuth ? t('No authentication') : pendingKey ? t('New key pending save') : hasApiKey ? t('Configured') : t('No credential')}
    </Badge>}>
    <Field className='acct-dlg-field'>
      <FieldLabel>{t('Authentication method')}</FieldLabel>
      <RadioGroup value={noAuth ? 'none' : 'key'} disabled={clearBusy} aria-label={t('Authentication method')}
        className='acct-dlg-choice-grid' onValueChange={next => onToggleNoAuth(next === 'none')}>
        <Label className='acct-dlg-choice' data-selected={!noAuth || undefined}>
          <RadioGroupItem value='key' /><span><strong>API Key</strong><small>{t('Use a key to access the upstream service')}</small></span>
        </Label>
        <Label className='acct-dlg-choice' data-selected={noAuth || undefined}>
          <RadioGroupItem value='none' /><span><strong>{t('No authentication')}</strong><small>{t('Local or credential-free upstream services')}</small></span>
        </Label>
      </RadioGroup>
    </Field>
    <Field className='acct-dlg-field'>
      <div className='acct-dlg-field-head'>
        <FieldLabel htmlFor='account-api-key-input'>API Key</FieldLabel>
        {hasApiKey && !noAuth ? <Button variant='ghost' size='xs' disabled={clearBusy} onClick={onClear}
          className='acct-dlg-clear'>{t('Clear saved key')}</Button> : null}
      </div>
      <InputGroup data-disabled={noAuth || clearBusy}>
        <InputGroupInput id='account-api-key-input' type={revealed && !noAuth ? 'text' : 'password'}
          autoComplete='new-password' spellCheck={false} disabled={noAuth || clearBusy}
          placeholder={hasApiKey ? t('Leave blank to keep the current key, or enter a replacement') : 'sk-…'}
          value={value} onChange={event => onValue(event.currentTarget.value)} />
        <InputGroupAddon align='inline-end'>
          <InputGroupButton size='icon-xs' disabled={noAuth || clearBusy} aria-pressed={revealed}
            aria-label={revealed ? t('Hide the newly entered key') : t('Show the newly entered key')}
            onClick={() => setRevealed(current => !current)}>
            <AccountSettingsIcon name={revealed ? 'eyeOff' : 'eye'} />
          </InputGroupButton>
        </InputGroupAddon>
      </InputGroup>
      <FieldDescription>{t('The existing key is never displayed. Replacing it does not change the account ID.')}</FieldDescription>
    </Field>
    <div className={`acct-dlg-inline-note${noAuth || !hasApiKey && !pendingKey ? ' warning' : ''}`}>
      <AccountSettingsIcon name={noAuth || !hasApiKey ? 'info' : 'shield'} size={14} />
      <span>{noAuth
        ? t('Saving removes the key and omits authentication headers when forwarding requests or fetching models. Make sure the upstream service does not require credentials.')
        : hasApiKey || pendingKey
          ? t('Leave blank to keep the key. A replacement takes effect after clicking Save changes.')
          : t('This account has no credentials and will be skipped when forwarding. Enter a key or select No authentication.')}</span>
    </div>
  </SettingGroup>
}

/** 与后端 custom_providers::MAX_NAME_CHARS 一致。 */
const MAX_PROVIDER_NAME_CHARS = 64

export function ProviderSection({ provider, name, protocol, baseUrl, accountBaseUrl, count, emulation,
  onName, onProtocol, onBaseUrl, onEmulation, onRemove, onModels, onClearAccountBaseUrl, disabled }: {
  provider: ProviderSettings
  name: string
  protocol: string
  baseUrl: string
  /** 当前账号自己存的地址覆盖项（空串 = 跟随提供商）；转发时它优先于上面的 Base URL */
  accountBaseUrl: string
  count: number
  emulation: string
  onName: (next: string) => void
  onProtocol: (next: string) => void
  onBaseUrl: (next: string) => void
  onEmulation: (next: string) => void
  onRemove: () => void
  onModels: () => void
  onClearAccountBaseUrl: () => void
  disabled: boolean
}) {
  const [advanced, setAdvanced] = React.useState(emulation === 'opencode')
  const protocols = shared().wbProviders?.PROTOCOL_OPTIONS || [
    { value: 'chat_completions', label: 'OpenAI - Chat Completions' },
    { value: 'responses', label: 'OpenAI - Responses' },
    { value: 'anthropic', label: 'Anthropic - Messages' },
  ]
  const current = protocols.find(option => option.value === protocol)?.label || protocol
  const suffix = protocol === 'anthropic' ? '/v1/messages' : protocol === 'responses' ? '/responses' : '/chat/completions'
  // 请求端点按**这个账号实际会用的地址**算：账号上的覆盖项优先于提供商的 Base URL。
  // 早先版本新建提供商时会把地址抄进首个账号，改提供商地址不生效就是它挡的
  // （issue #455）—— 预览必须如实反映这一点，否则界面显示的和实际发出的又是两个值。
  const override = accountBaseUrl.trim()
  const effectiveBase = override || baseUrl.trim()
  const endpoint = effectiveBase ? `${effectiveBase.replace(/\/+$/, '')}${suffix}` : ''
  return <>
    <Alert variant='warning' className='acct-dlg-scope-notice' role='note'>
      <AccountSettingsIcon name='accounts' />
      <div><strong>{t('Editing shared provider settings')}</strong>
        <p>{t('Shared by {count} accounts under {name}. Changing the protocol or address may interrupt requests in progress.', { name: provider.name || provider.id, count })}</p>
      </div>
    </Alert>
    <SettingGroup title={t('Connection parameters')}>
      <div className='acct-dlg-field-grid'>
        <Field className='acct-dlg-field'>
          <FieldLabel htmlFor='account-provider-name'>{t('Provider name')}</FieldLabel>
          <Input id='account-provider-name' maxLength={MAX_PROVIDER_NAME_CHARS} disabled={disabled}
            value={name} onChange={event => onName(event.currentTarget.value)} />
        </Field>
        <Field className='acct-dlg-field'>
          <FieldLabel htmlFor='account-provider-protocol'>{t('Protocol')}</FieldLabel>
          <Select value={protocol} disabled={disabled} onValueChange={value => onProtocol(String(value))}>
            <SelectTrigger id='account-provider-protocol'><SelectValue>{current}</SelectValue></SelectTrigger>
            <SelectContent>{protocols.map(option => <SelectItem key={option.value} value={option.value}>{option.label}</SelectItem>)}</SelectContent>
          </Select>
        </Field>
        <Field className='acct-dlg-field acct-dlg-field-full'>
          <div className='acct-dlg-field-head'>
            <FieldLabel htmlFor='account-provider-baseurl'>Base URL</FieldLabel>
            {override ? <Button variant='ghost' size='xs' disabled={disabled}
              className='acct-dlg-clear' onClick={onClearAccountBaseUrl}>{t('Follow provider')}</Button> : null}
          </div>
          <Input id='account-provider-baseurl' className='mono' disabled={disabled} spellCheck={false}
            placeholder={t('For OpenAI-compatible use /v1; for Anthropic use the root URL')}
            value={baseUrl} onChange={event => onBaseUrl(event.currentTarget.value)} />
          <FieldDescription>{t('Enter the API base URL without appending {path}.', { path: suffix })}</FieldDescription>
          {override ? <div className='acct-dlg-inline-note warning'>
            <AccountSettingsIcon name='info' size={14} />
            <span>{t('This account stores its own address ({url}), which takes precedence when forwarding. Changing the shared provider address above does not override it.', { url: override })}</span>
          </div> : null}
        </Field>
      </div>
      {endpoint ? <div className='acct-dlg-endpoint'><span>{t('Request endpoint')}</span><code>{endpoint}</code></div> : null}
      <div className='acct-dlg-model-link'><span>{t('Manage model lists and mappings in Model management.')}</span>
        <Button variant='link' size='xs' disabled={disabled} onClick={onModels}>{t('Model management')}<AccountSettingsIcon name='caretRight' size={12} /></Button>
      </div>
    </SettingGroup>
    <div className='acct-dlg-group-label'>{t('Advanced compatibility')}</div>
    <Collapsible open={advanced} onOpenChange={setAdvanced} className='acct-dlg-advanced'>
      <CollapsibleTrigger disabled={disabled} className='acct-dlg-advanced-trigger'>
        <AccountSettingsIcon name='sliders' /><span>{t('Advanced compatibility')}</span>
        <small>{emulation === 'opencode' ? t('Enabled') : t('Off by default')}</small>
        <AccountSettingsIcon name='caretRight' size={13} />
      </CollapsibleTrigger>
      <CollapsibleContent className='acct-dlg-advanced-content'>
        <Field className='acct-dlg-toggle-field'>
          <div><FieldLabel htmlFor='account-provider-emulation'>{t('Emulate the official OpenCode client')}</FieldLabel>
            <FieldDescription>{t('Only for upstream services such as OpenCode Zen. Leave this off for other providers.')}</FieldDescription>
          </div>
          <Switch id='account-provider-emulation' checked={emulation === 'opencode'} disabled={disabled}
            onCheckedChange={next => onEmulation(next ? 'opencode' : '')} />
        </Field>
        <p className='detail acct-dlg-advanced-note'>{t('When enabled, request bodies are rewritten to add session headers and placeholder tools. Without a key, the anonymous credential public is used.')}</p>
      </CollapsibleContent>
    </Collapsible>
    <div className='acct-dlg-danger'>
      <div><h3>{t('Delete provider')}</h3><p>{t('Also deletes all {count} accounts under this provider. This cannot be undone.', { count })}</p></div>
      <Button variant='outline' size='sm' disabled={disabled} onClick={onRemove} className='acct-dlg-delete'>{t('Delete provider')}</Button>
    </div>
  </>
}
