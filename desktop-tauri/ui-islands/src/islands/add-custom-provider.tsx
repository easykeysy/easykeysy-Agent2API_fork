/**
 * Agent2API · 「」：（ new / already has ）。
 *
 * old ui/add-custom-provider.js。（custom- before ，） 
 * has ： new + （POST /api/custom-providers）、 already has 
 * （POST /api/accounts）—— and for ， and that 
 * 「 1 config + 1 」 for not up ， 1 。
 *
 * ── ────────────────────────────────────
 * ** not by **。 1 is 「 new 」 that is 1 already has 
 * ， already 「 to 」： before up down no has id → new ， after 
 * that id → already has and 。 need （ 1 in 
 * down ）。 is this 「 by up down to 」， no has 。
 *
 * ── by Why in ──────────────────────────────
 * in will and 1 、 not ， when to failure 1 
 * （toast seconds after no ）。 ：（ this ） and 
 * （CustomFootActions）， DOM id and mode —— is 
 * current， and old 1 。
 */

import * as React from 'react'
import {
  Button,
  Checkbox,
  DialogSection,
  Input,
  Label,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@ui'

import {
  clearFields,
  describeError,
  draftProps,
  readField,
  setDraftValue,
  shared,
  toast,
  type CustomProviderRecord,
} from './add-account-bridge'
import { balanceSpecOf, type BalanceFieldKey } from './balance-query'
import { DuplicateProviderDialog } from './add-provider-duplicate-dialog'
import { presetOfProvider, sameProviderEndpoint } from './add-provider-metadata'
import { t } from '../i18n'

/** limit（ and after custom_providers::MAX_NAME_CHARS 1 ， before 1 ） */
const MAX_NAME_CHARS = 64
/** limit（ and add-provider-forms MAX_NAME_LENGTH 1 ） */
const MAX_ACCOUNT_NAME_CHARS = 100

/** Base URL ：Anthropic ，OpenAI need /v1（ after by path） */
const BASE_HINT_OPENAI = t('OpenAI 兼容填到 /v1；Anthropic 填根地址')
const BASE_HINT_ANTHROPIC = t('Anthropic 协议填根地址，不要带 /v1')
const BASE_PLACEHOLDER_OPENAI = 'https://open.bigmodel.cn/api/paas/v4'
const BASE_PLACEHOLDER_ANTHROPIC = 'https://api.anthropic.com'

/** down ：must and after PROTOCOLS 1 （ in providers.js） */
const FALLBACK_PROTOCOLS = [
  { value: 'chat_completions', label: 'OpenAI - Chat Completions' },
  { value: 'responses', label: 'OpenAI - Responses' },
  { value: 'anthropic', label: 'Anthropic - Messages' },
]

const PROTOCOL_ID = 'custom-protocol-select'
const NAME_ID = 'custom-name-input'
const BASEURL_ID = 'custom-baseurl-input'
const APIKEY_ID = 'custom-apikey-input'
const NOAUTH_ID = 'custom-noauth-check'
const EXISTING_APIKEY_ID = 'custom-existing-apikey-input'
const EXISTING_NOAUTH_ID = 'custom-existing-noauth-check'
const EXISTING_NAME_ID = 'custom-existing-name-input'

export const CREATE_BUTTON_ID = 'custom-create-button'
export const EXISTING_BUTTON_ID = 'custom-existing-button'
export const REMOVE_BUTTON_ID = 'custom-existing-remove'

/** ： not use busy （ and addBusy 1 to ） */
let submitBusy = false

const protocolOptions = (): Array<{ value: string; label: string }> =>
  shared().wbProviders?.PROTOCOL_OPTIONS || FALLBACK_PROTOCOLS

const customList = (): CustomProviderRecord[] => shared().wbProviders?.customList?.() || []

/* ─── that 1 （ when ）─────────
 *
 * and `balance-query.tsx`； this 「 by 」 and 
 * 「 when 」。（ new / already has ） has 1 ，DOM id before 。
 */

const BALANCE_CREATE_PREFIX = 'custom-balance'
const BALANCE_EXISTING_PREFIX = 'custom-existing-balance'

/** DOM id（ and 、 use 1 ） */
const balanceFieldId = (prefix: string, key: BalanceFieldKey): string => `${prefix}-${key}-input`

/** （empty = this no has ） */
function presetSchemeOf(presetKey: string): string {
  if (!presetKey) return ''
  const preset = shared().wbPresetProviders?.presetOf?.(presetKey)
  return String(preset?.balance?.scheme || '')
}

/**
 * → request `balanceQuery`（**empty**）。
 *
 * and after `custom_accounts::normalize_balance_query_patch` 1 ： = not 、
 * empty = not 。add this 1 no has 「 already config」 can ，empty is 「 not 」， 
 * not can ； no has （Sub2API / DeepSeek / OpenCode Go / 
 * ）return null。
 */
function readBalanceQuery(scheme: string, prefix: string): Record<string, string> | null {
  const spec = balanceSpecOf(scheme)
  if (!spec || !spec.fields.length) return null
  const query: Record<string, string> = {}
  for (const field of spec.fields) {
    const value = readField(balanceFieldId(prefix, field.key))
    if (value) query[field.key] = value
  }
  return Object.keys(query).length ? query : null
}

/** empty（success after ， down is ） */
function clearBalanceFields(scheme: string, prefix: string): void {
  const spec = balanceSpecOf(scheme)
  if (!spec) return
  for (const field of spec.fields) setDraftValue(balanceFieldId(prefix, field.key), '')
}

/**
 * that 1 （** has **）。
 *
 * + ： and this its 1 （ in that 
 * ， by DOM id ，file）。 no has when ** not **， 1 
 * —— use need 「 this 1 」（DeepSeek that 「 not use 
 * 」 ， this 1 is ）。
 */
function BalanceFields({ scheme, prefix }: { scheme: string; prefix: string }): React.ReactElement | null {
  const spec = balanceSpecOf(scheme)
  if (!spec) return null
  return (
    <>
      <div className='add-field'>
        <Label>{t('余额查询')}</Label>
        <div className='col-start-2 row-start-1 flex h-[30px] items-center'>
          <span className='text-xs text-subtle'>{t('方案：{label}', { label: spec.label })}</span>
          <span className='ml-1.5 text-xs text-subtle'>{spec.endpoint}</span>
        </div>
        <span className='hint'>{spec.note}</span>
      </div>
      {spec.fields.map(field => (
        <div className='add-field' key={field.key}>
          <Label htmlFor={balanceFieldId(prefix, field.key)}>{field.label}</Label>
          <Input
            id={balanceFieldId(prefix, field.key)}
            type={field.secret ? 'password' : 'text'}
            autoComplete={field.secret ? 'new-password' : 'off'}
            placeholder={field.placeholder}
            {...draftProps(balanceFieldId(prefix, field.key))}
          />
          <span className='hint'>{field.hint}</span>
        </div>
      ))}
    </>
  )
}

/** by （ and runAdd 1 ； is ，） */
async function runSubmit(setBusy: (value: boolean) => void, task: () => Promise<void>): Promise<void> {
  if (submitBusy) return
  submitBusy = true
  setBusy(true)
  try {
    await task()
  } finally {
    submitBusy = false
    setBusy(false)
  }
}

/** success after 1 ：、 new directory and 、 */
async function afterCustomAdd(message: string): Promise<void> {
  // 1 （ + down ）： will in up 
  shared().wbAddAccountModal?.close?.()
  // directory： / wbProviders 
  await shared().wbProviders?.refreshCustom?.()
  await shared().wbApp?.refresh?.()
  toast(message)
}

/* ─── 「 this up none need 」 ─────────────── */

/**
 * by React has （ Checkbox not is input，`readField`
 * that DOM for invalid）， when 1 to —— in ** 1 **
 * （CustomFootActions）， and has 「DOM id + 」 this 1 
 * （ and down 1 ，file）。 '1' / empty， `readField`
 * for input id will to ， is not React 。
 */
function writeNoAuthDraft(id: string, value: boolean): void {
  setDraftValue(id, value ? '1' : '')
}

function readNoAuthDraft(id: string): boolean {
  return readField(id) === '1'
}

/** + （ use 1 ， in ） */
function NoAuthCheckbox({
  checked, note, onChange,
}: {
  checked: boolean
  note: string
  onChange: (next: boolean) => void
}): React.ReactElement {
  return (
    <div className='add-field'>
      <Label>{t('鉴权')}</Label>
      {/* Checkbox not is input（ role=checkbox by ），
           use 1 <label> ：button is can ， can */}
      <div className='col-start-2 row-start-1 flex h-[30px] items-center'>
        <Label className='inline-flex cursor-pointer items-center gap-2.5'>
          <Checkbox checked={checked} aria-label={t('该上游无需鉴权')}
            onCheckedChange={next => onChange(next === true)} />
          <span className='text-xs text-subtle'>{t('该上游无需鉴权（不发送鉴权头）')}</span>
        </Label>
      </div>
      <span className='hint'>{note}</span>
    </div>
  )
}

/* ─── ─────────────────────────────── */

export type CustomMode = 'create' | 'existing'

export function CustomProviderBlock({
  mode,
  providerHint,
  presetKey,
  showToken,
  version,
}: {
  mode: CustomMode
  /** 1 is 1 already has when id（empty = no has ） */
  providerHint: string
  /** 1 is when key（empty = not is from ） */
  presetKey: string
  /** 1 1 +1： in 「 1 」 when new （ old onShow ） */
  showToken: number
  /** directory new after */
  version: number
}): React.ReactElement {
  const options = protocolOptions()
  const [protocol, setProtocol] = React.useState(() => readField(PROTOCOL_ID) || options[0].value)
  /** to Base URL （ use 1 ， to by that ） */
  const [baseHintOverride, setBaseHintOverride] = React.useState('')
  /** to Base URL （ no has ：`https:// /…`） */
  const [basePlaceholderOverride, setBasePlaceholderOverride] = React.useState('')
  /** has 1 「 this up none need 」， from （ in ） */
  const [noAuth, setNoAuth] = React.useState(() => readNoAuthDraft(NOAUTH_ID))
  const [existingNoAuth, setExistingNoAuth] = React.useState(() => readNoAuthDraft(EXISTING_NOAUTH_ID))

  const list = React.useMemo(() => customList(), [version])
  const pickedProvider = list.find(item => item.id === providerHint)
  const current = pickedProvider?.id || ''
  const pickedName = pickedProvider?.name || t('该提供商')
  const selfHosted = mode === 'create' && Boolean(presetKey) && !shared().wbPresetProviders?.presetOf?.(presetKey)?.baseUrl

  React.useEffect(() => {
    if (mode !== 'create' || !presetKey) return
    const preset = shared().wbPresetProviders?.presetOf?.(presetKey)
    if (!preset) return
    const presetNoAuth = preset.account?.noAuth === true
    setDraftValue(NAME_ID, preset.baseUrl ? preset.name : '')
    setDraftValue(BASEURL_ID, preset.baseUrl || '')
    if (preset.protocol) {
      setProtocol(preset.protocol)
      setDraftValue(PROTOCOL_ID, preset.protocol)
    }
    setNoAuth(presetNoAuth)
    writeNoAuthDraft(NOAUTH_ID, presetNoAuth)
    setBaseHintOverride(preset.hint || '')
    setBasePlaceholderOverride(preset.placeholder || '')
    // showToken and ： 1 1 need new （ old onShow this 1 ）
  }, [mode, presetKey, providerHint, showToken])

  const existingPresetKey = pickedProvider?.presetKey || ''
  React.useEffect(() => {
    if (mode !== 'existing') return
    const presetNoAuth = presetOfProvider(pickedProvider)?.account?.noAuth === true
    setExistingNoAuth(presetNoAuth)
    writeNoAuthDraft(EXISTING_NOAUTH_ID, presetNoAuth)
  }, [mode, providerHint, existingPresetKey, showToken])

  const anthropic = protocol === 'anthropic'
  const baseHint = baseHintOverride || (anthropic ? BASE_HINT_ANTHROPIC : BASE_HINT_OPENAI)
  const basePlaceholder = basePlaceholderOverride || (anthropic ? BASE_PLACEHOLDER_ANTHROPIC : BASE_PLACEHOLDER_OPENAI)
  // ： new ****、 already has ** in that **（ in 
  // ， select up for `list` use ）。empty = this 1 not ，
  // that 1 not 。
  const createScheme = mode === 'create' ? presetSchemeOf(presetKey) : ''
  const existingScheme = mode === 'existing'
    ? String(list.find(item => item.id === current)?.balanceScheme || '')
    : ''
  /** 1 ：React + 1 （） */
  const toggleNoAuth = (id: string, setter: (next: boolean) => void) => (next: boolean) => {
    setter(next)
    writeNoAuthDraft(id, next)
  }

  return (
    <>
      <DialogSection hidden={mode !== 'create'}>
        <div className='add-panel-head'>
          <h3>{t('上游信息')}</h3>
          <span>{t('创建这个提供商，并同时建立它的第一个账号。')}</span>
        </div>
        <div className='add-form'>
          <div className='add-field'>
            <Label htmlFor={NAME_ID}>
              {selfHosted ? t('站点名称') : t('名称')}
              <i className='req' aria-hidden='true'>*</i>
            </Label>
            <Input
              id={NAME_ID}
              type='text'
              maxLength={MAX_NAME_CHARS}
              aria-required='true'
              placeholder={selfHosted ? t('如：公司中转站、个人备用站') : t('如：智谱 GLM')}
              {...draftProps(NAME_ID)}
            />
            <span className='hint'>{selfHosted
              ? t('填写具体站点名称；模板类型不会作为站点身份')
              : t('1~64 个字符，账号列表里按它分组显示')}</span>
          </div>
          <div className='add-field'>
            <Label htmlFor={PROTOCOL_ID}>{t('协议')}</Label>
            {/* down Base URL ， this */}
            <Select
              value={protocol}
              onValueChange={next => {
                const value = String(next)
                setProtocol(value)
                setDraftValue(PROTOCOL_ID, value)
                setBaseHintOverride('')
              }}
            >
              {/* .add-field then select（`> select`）， need */}
              <SelectTrigger id={PROTOCOL_ID} className='col-start-2 row-start-1 w-full'>
                <SelectValue>{options.find(item => item.value === protocol)?.label || protocol}</SelectValue>
              </SelectTrigger>
              <SelectContent>
                {options.map(item => (
                  <SelectItem key={item.value} value={item.value}>{item.label}</SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className='add-field'>
            <Label htmlFor={BASEURL_ID}>
              Base URL
              <i className='req' aria-hidden='true'>*</i>
            </Label>
            <Input
              id={BASEURL_ID}
              type='text'
              aria-required='true'
              placeholder={basePlaceholder}
              {...draftProps(BASEURL_ID)}
            />
            <span className='hint' id='custom-baseurl-hint'>{baseHint}</span>
          </div>
          <div className='add-field'>
            <Label htmlFor={APIKEY_ID}>API Key</Label>
            <Input
              id={APIKEY_ID}
              type='password'
              autoComplete='new-password'
              placeholder='sk-…'
              // 「none need 」 no has key can （ after by 1 ： 
              // can has 1 ， custom_accounts::add_custom_account）
              disabled={noAuth}
              {...draftProps(APIKEY_ID)}
            />
            <span className='hint'>
              {noAuth
                ? t('不需要 Key：转发与拉取模型都不发送鉴权头')
                : t('上游不要鉴权时留空，并勾选下面一项（留空又不勾会被当成未配置凭证）')}
            </span>
          </div>
          <NoAuthCheckbox
            checked={noAuth}
            note={noAuth
              ? t('已声明无需鉴权：账号会被正常选路，出网时不带任何鉴权头')
              : t('本地 Ollama、OpenCode Zen 免费档这类上游要勾上，否则账号不可用')}
            onChange={toggleNoAuth(NOAUTH_ID, setNoAuth)}
          />
          {/* ： has （ new when ） */}
          <BalanceFields scheme={createScheme} prefix={BALANCE_CREATE_PREFIX} />
        </div>
      </DialogSection>

      <DialogSection hidden={mode !== 'existing'}>
        <div className='add-panel-head'>
          <h3>{t('账号信息')}</h3>
          {/* is use ， in JSX to React （ not dangerouslySetInnerHTML）；
               in t()，empty is 1 */}
          <span>
            {t('添加到 ')}<b>{pickedName}</b>{t('　同一家可以放多把 key，按优先级轮换。')}
          </span>
        </div>
        <div className='add-form'>
          <div className='add-field'>
            <Label>{t('提供商')}</Label>
            <Input readOnly value={pickedProvider ? pickedName : t('该提供商已不存在（可能已被删除），请刷新后重试')} />
            <span className='hint'>{t('要添加到其他站点，请返回「已建 API」重新选择')}</span>
          </div>
          <div className='add-field'>
            <Label>Base URL</Label>
            <Input readOnly value={pickedProvider?.baseUrl || ''} />
          </div>
          <div className='add-field'>
            <Label htmlFor={EXISTING_APIKEY_ID}>API Key</Label>
            <Input
              id={EXISTING_APIKEY_ID}
              type='password'
              autoComplete='new-password'
              placeholder='sk-…'
              disabled={existingNoAuth}
              {...draftProps(EXISTING_APIKEY_ID)}
            />
            <span className='hint'>
              {existingNoAuth
                ? t('不需要 Key：转发与拉取模型都不发送鉴权头')
                : t('上游不要鉴权时留空，并勾选下面一项（留空又不勾会被当成未配置凭证）')}
            </span>
          </div>
          <NoAuthCheckbox
            checked={existingNoAuth}
            note={existingNoAuth
              ? t('已声明无需鉴权：账号会被正常选路，出网时不带任何鉴权头')
              : t('同一家可以混着放：有 Key 的账号与无鉴权账号各按各的规则走；无需鉴权的账号也能并存多条（各自可配不同的出网代理）')}
            onChange={toggleNoAuth(EXISTING_NOAUTH_ID, setExistingNoAuth)}
          />
          {/* ： this 1 （， not can ） */}
          <BalanceFields scheme={existingScheme} prefix={BALANCE_EXISTING_PREFIX} />
          <div className='add-field'>
            <Label htmlFor={EXISTING_NAME_ID}>{t('备注名')}</Label>
            <Input
              id={EXISTING_NAME_ID}
              type='text'
              maxLength={MAX_ACCOUNT_NAME_CHARS}
              placeholder={t('可选')}
              {...draftProps(EXISTING_NAME_ID)}
            />
            <span className='hint'>{t('留空则用提供商名称')}</span>
          </div>
        </div>
      </DialogSection>
    </>
  )
}

/* ─── up by ─────────────────── */

/** this 1 need use to ： and failure（ by that has ） */
type FootContext = {
  presetKey: string
  providerId: string
  setBusy: (value: boolean) => void
  setHint: (message: string) => void
  setDuplicates: (providers: CustomProviderRecord[]) => void
}

/** failure when toast and （toast seconds after no ） */
function showSubmitError(context: FootContext, error: unknown): void {
  const message = describeError(error)
  toast(t('添加失败：{reason}', { reason: message }), 'err')
  context.setHint(message)
}

/** new ：POST /api/custom-providers（ + 1 ） */
async function submitCreate(context: FootContext, allowDuplicate = false, target?: CustomProviderRecord): Promise<void> {
  const name = readField(NAME_ID)
  const protocol = readField(PROTOCOL_ID) || protocolOptions()[0].value
  const baseUrl = readField(BASEURL_ID)
  const apiKey = readField(APIKEY_ID)
  const noAuth = readNoAuthDraft(NOAUTH_ID)
  // in 1 （ not is <form>， required not ）
  if (!name) { toast(t('请填写名称'), 'err'); return }
  if (!baseUrl) { toast(t('请填写 Base URL'), 'err'); return }
  await runSubmit(context.setBusy, async () => {
    if (!allowDuplicate) {
      await shared().wbProviders?.refreshCustom?.()
      const duplicates = customList().filter(provider => sameProviderEndpoint(provider, protocol, baseUrl))
      if (duplicates.length) {
        context.setDuplicates(duplicates)
        return
      }
    }
    context.setDuplicates([])
    if (target && !customList().some(provider => provider.id === target.id && sameProviderEndpoint(provider, protocol, baseUrl))) {
      toast(t('该提供商已不存在或地址已变更，请重新选择'), 'err')
      return
    }
    const payload: Record<string, unknown> = target ? { provider: target.id } : { name, protocol, baseUrl }
    // up （ by this request， preset-providers.js quirks）
    const preset = context.presetKey
      ? shared().wbPresetProviders?.presetOf?.(context.presetKey)
      : null
    const scheme = presetSchemeOf(context.presetKey)
    if (!target) {
      if (preset) payload.presetKey = preset.key
      const quirks = preset?.quirks || {}
      if (quirks.urlSuffix) payload.urlSuffix = quirks.urlSuffix
      if (quirks.headers && Object.keys(quirks.headers).length) payload.headers = { ...quirks.headers }
      if (quirks.anthropicToolType) payload.anthropicToolType = quirks.anthropicToolType
      if (preset?.clientEmulation) payload.clientEmulation = preset.clientEmulation
      if (scheme) payload.balanceScheme = scheme
    }
    // to already has not can new to 1 up 。
    const balanceQuery = !target || target.balanceScheme === scheme
      ? readBalanceQuery(scheme, BALANCE_CREATE_PREFIX)
      : null
    if (balanceQuery) payload.balanceQuery = balanceQuery
    // （ after this 1 ）：none need not apiKey up 
    if (noAuth) payload.noAuth = true
    else if (apiKey) payload.apiKey = apiKey
    context.setHint('')
    try {
      const data = (await shared().wbProviders?.customRequest?.(
        'POST', target ? '/api/accounts' : '/api/custom-providers', payload,
      )) as { provider?: { name?: string }; account?: { name?: string }; created?: boolean } | null
      clearFields([NAME_ID, BASEURL_ID, APIKEY_ID])
      clearBalanceFields(scheme, BALANCE_CREATE_PREFIX)
      // and 1 （ down is default， not up 1 ）
      writeNoAuthDraft(NOAUTH_ID, false)
      const message = target
        ? addedMessage(data?.account?.name || target.name || '', data?.created)
        : t('✅ 已创建自定义提供商「{name}」并添加账号', { name: data?.provider?.name || name })
      await afterCustomAdd(message)
    } catch (error) {
      showSubmitError(context, error)
    }
  })
}

function addedMessage(label: string, created: boolean | undefined): string {
  const suffix = label ? `：${label}` : ''
  return created !== false
    ? t('✅ {label}账号已添加{name}', { label: '', name: suffix })
    : t('✅ 已更新已有账号{name}（同一提供商内，同一把 API Key 只保留一条）', { name: suffix })
}

/** already has ：POST /api/accounts（custom provider = custom-xxx ） */
async function submitExisting(context: FootContext): Promise<void> {
  const providerId = context.providerId
  if (!providerId || !customList().some(item => item.id === providerId)) {
    toast(t('该提供商已不存在（可能已被删除），请刷新后重试'), 'err')
    return
  }
  const apiKey = readField(EXISTING_APIKEY_ID)
  const noAuth = readNoAuthDraft(EXISTING_NOAUTH_ID)
  const name = readField(EXISTING_NAME_ID)
  // in this 1 up （）： need not need that by ，
  // when by （ 1 ， not will 「 up has 、 when 」）
  const scheme = String(customList().find(item => item.id === providerId)?.balanceScheme || '')
  await runSubmit(context.setBusy, async () => {
    const payload: Record<string, unknown> = { provider: providerId }
    // and new 1 ：none need not apiKey
    if (noAuth) payload.noAuth = true
    else if (apiKey) payload.apiKey = apiKey
    if (name) payload.name = name
    const balanceQuery = readBalanceQuery(scheme, BALANCE_EXISTING_PREFIX)
    if (balanceQuery) payload.balanceQuery = balanceQuery
    context.setHint('')
    try {
      const data = (await shared().wbProviders?.customRequest?.(
        'POST', '/api/accounts', payload,
      )) as { account?: { name?: string }; created?: boolean } | null
      clearFields([EXISTING_APIKEY_ID, EXISTING_NAME_ID])
      clearBalanceFields(scheme, BALANCE_EXISTING_PREFIX)
      writeNoAuthDraft(EXISTING_NOAUTH_ID, false)
      const label = data?.account?.name
        || customList().find(item => item.id === providerId)?.name
        || ''
      await afterCustomAdd(addedMessage(label, data?.created))
    } catch (error) {
      showSubmitError(context, error)
    }
  })
}

/**
 * delete「 already has 」 down in that 1 （）。
 *
 * in `wbCustomProvidersUi.remove` —— （ will 
 * ）、POST /api/custom-providers/remove、 new directory and in that ， and 
 * set 「delete」 use 1 ； this ： is 1 
 * （ up down in ID）、（ —— down ， no has can 
 * up down ）。
 */
async function removeExistingProvider(providerId: string): Promise<void> {
  if (!providerId) { toast(t('请先选择一个自定义提供商'), 'err'); return }
  const remover = shared().wbCustomProvidersUi?.remove
  if (typeof remover !== 'function') { toast(t('删除功能不可用（脚本未就绪）'), 'err'); return }
  await runSubmit(() => {}, async () => {
    const removed = await remover(providerId)
    if (removed) shared().wbAddAccountModal?.close?.()
  })
}

export function CustomFootActions({
  mode,
  presetKey,
  providerId,
}: {
  mode: CustomMode
  presetKey: string
  providerId: string
}): React.ReactElement {
  const [busy, setBusy] = React.useState(false)
  const [hint, setHint] = React.useState('')
  const [duplicates, setDuplicates] = React.useState<CustomProviderRecord[]>([])
  const context: FootContext = { presetKey, providerId, setBusy, setHint, setDuplicates }

  return (
    <>
      {duplicates.length ? (
        <DuplicateProviderDialog
          providers={duplicates}
          onClose={() => setDuplicates([])}
          onCreate={() => { void submitCreate(context, true) }}
          onExisting={provider => { void submitCreate(context, true, provider) }}
        />
      ) : null}
      <span className={`add-foot-hint${hint ? ' err' : ''}`} id='add-form-foot-hint'>{hint}</span>
      <span className='add-foot-actions' id='add-form-foot-actions'>
        {mode === 'existing' ? (
          <Button
            id={REMOVE_BUTTON_ID}
            variant='destructive'
            title={t('级联删除名下全部账号，不可恢复')}
            onClick={() => { void removeExistingProvider(providerId) }}
          >
            {t('删除此提供商')}
          </Button>
        ) : null}
        {/* by 、 by （ and old 1 ： and for in 1 ） */}
        <Button
          id={CREATE_BUTTON_ID}
          hidden={mode !== 'create'}
          disabled={busy}
          onClick={() => { void submitCreate(context) }}
        >
          {busy && mode === 'create' ? t('提交中…') : t('创建并添加账号')}
        </Button>
        <Button
          id={EXISTING_BUTTON_ID}
          hidden={mode !== 'existing'}
          disabled={busy}
          onClick={() => { void submitExisting(context) }}
        >
          {busy && mode === 'existing' ? t('提交中…') : t('添加账号')}
        </Button>
      </span>
    </>
  )
}
