/**
 * Agent2API · 「 / 」： （ + ）。
 *
 * old add-provider-forms.js buildProviderBlock / mountWebLogin /
 * mountSmsLogin / mountOauthLogin / addProviderManual / addProviderDesktop and 
 * add-account.js WorkBuddy that 1 （ + + ）。
 *
 * ── and old 1 （ ）──────────────────────────
 * old 「」 ，**all in DOM **（
 * ）， as in 、 by id in that 
 * up （sms-login.js in create() when addEventListener）。 up use 1 ：
 * （ has current in that can ），。
 * is 「 → → → 」 when deviceId in ， as and old 1 。
 *
 * 1 use `hidden` ， when not and 。
 *
 * ── ─────────────────────────────────────────
 * web-login.js： by 、 use and etc ， use Button / Spinner 。
 * sms-login.js：create() when by id click， after .value。
 * autoclaw-oauth.js：create() when by and click， by id 。
 * id ，request and by for 。
 */

import * as React from 'react'
import {
  Alert,
  AlertDescription,
  Button,
  Input,
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  InputGroupInput,
  SegmentedControl,
  Spinner,
  Switch,
  Textarea,
  Label, Field, FieldGroup, FieldLabel,
} from '@ui'
import { AddFoot, AddGroup } from './add-account-kit'

import {
  ADD_METHODS,
  MAX_NAME_LENGTH,
  WEB_LOGIN_HINT,
  addButtonTextOf,
  desktopImportAvailable,
  fieldIdOf,
  manualNoteOf,
  manualTitleOf,
  maxLengthOf,
  methodsOf,
  platform,
  type FieldSpec,
  type MethodId,
  type ProviderConfig,
} from './add-account-configs'
import {
  afterAdd,
  addedLabelOf,
  clearFields,
  closeAddModals,
  describeError,
  draftProps,
  loginPrefs,
  postAccount,
  readField,
  shared,
  toast,
  type AccountRecord,
  type Edition,
  type LoginMode,
  type OauthController,
  type WebLoginController,
  type WebLoginView,
} from './add-account-bridge'
import { readFileAsDataUrl } from '../../../ui-shared/file-read'
import { accountFilePayload, accountExportEdition, isAccountExport } from './add-account-file'
import { draftOfProxy, ProxyForm, readProxyDraft } from './accounts-proxy-form'
import { DIALOG_SEG_CLASS } from './dialog-segmented'
import { t } from '../i18n'

/** that 「file in 」 need path */
const RACCOON_AUTH_PATH = t(
  '登录态文件路径：~/.box-agent/config/auth.json（Windows：C:\\Users\\<你的用户名>\\.box-agent\\config\\auth.json）')

/** by ： 1 （ old addBusy is ，） */
let addBusy = false

async function runAdd(setBusy: (value: boolean) => void, task: () => Promise<void>): Promise<void> {
  if (addBusy) return
  addBusy = true
  setBusy(true)
  try {
    await task()
  } finally {
    addBusy = false
    setBusy(false)
  }
}

/** ： use to is can （ and old innerHTML ）， */
function Note({ html, text }: { html?: string; text?: string }): React.ReactElement {
  if (html) return <p dangerouslySetInnerHTML={{ __html: html }} />
  return <p>{text || ''}</p>
}

/* ─── ─────────────────────────────── */

/**
 * `jsonExpand` ： and and to request（`false` = down ， 
 * 「 not is JSON」 after / 1 error）。
 *
 * ：failure and for （`"abc"` / `5` / `[]`）。 is 「
 * 」 or 「 1 」， this in this 1 。
 */
function expandJsonField(field: FieldSpec, value: string, payload: Record<string, unknown>): boolean {
  let parsed: unknown
  try {
    parsed = JSON.parse(value)
  } catch (error) {
    toast(t('{name} 不是合法 JSON：{reason}', { name: field.label, reason: describeError(error) }), 'err')
    return false
  }
  if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
    // JSON ， not can （ will t() {} ）， as 
    toast(t('{name}要粘一个 JSON 对象（形如 {sample}）', {
      name: field.label,
      sample: '{"codearts_provider_credential":{…}}',
    }), 'err')
    return false
  }
  Object.assign(payload, parsed as Record<string, unknown>)
  return true
}

function ManualSection({
  config,
  region,
  visible,
  footerHost,
}: {
  config: ProviderConfig
  region: string
  visible: boolean
  footerHost: HTMLElement | null
}): React.ReactElement {
  const prefix = config.provider
  const [busy, setBusy] = React.useState(false)
  const [hint, setHint] = React.useState('')
  const [proxyDraft, setProxyDraft] = React.useState(() => draftOfProxy(undefined))
  const note = manualNoteOf(config)
  const fieldIds = config.fields.map(field => fieldIdOf(config, field))

  async function submit(): Promise<void> {
    const payload: Record<string, unknown> = { provider: config.provider }
    // （Qoder）： use 1 
    if (config.regionOptions?.length) payload.mode = region
    for (const field of config.fields) {
      const value = readField(fieldIdOf(config, field))
      if (!field.optional && !value) {
        toast(t('请填写 {label}', { label: field.label }), 'err')
        return
      }
      if (!value) continue
      if (field.jsonExpand) {
        if (!expandJsonField(field, value, payload)) return
        continue
      }
      payload[field.key] = value
    }
    // 1 ： in `account.normalize` use 
    // （antigravity need 1 Google token ）。
    if (config.loginProxy) {
      try {
        payload.proxy = readProxyDraft(proxyDraft)
      } catch (error) {
        toast(t('添加失败：{reason}', { reason: describeError(error) }), 'err')
        return
      }
    }
    await runAdd(setBusy, async () => {
      try {
        const data = (await postAccount(payload)) as { account?: AccountRecord } | null
        clearFields(fieldIds)
        setHint('')
        await afterAdd(addedLabelOf(data?.account), config.label)
      } catch (error) {
        toast(t('添加失败：{reason}', { reason: describeError(error) }), 'err')
      }
    })
  }

  return (
    <AddGroup
      label={manualTitleOf(config)}
      hidden={!visible}
      // that 「file」： old is document up ， this in up 
      onClick={event => {
        if ((event.target as HTMLElement).closest('[data-raccoon-hint]')) {
          event.preventDefault()
          setHint(RACCOON_AUTH_PATH)
        }
      }}
    >
      <Note {...note} />
      {config.fields.map(field => {
        const id = fieldIdOf(config, field)
        // in up （ not is <form>， required not ）， in submit
        const marker = field.optional ? '' : t('（必填）')
        return (
          <div key={id} className={`field-row${field.rows ? ' stack' : ''}`}>
            <Label htmlFor={id}>
              {field.label}
              {marker}
            </Label>
            {field.rows ? (
              <Textarea id={id} rows={field.rows} placeholder={field.placeholder} {...draftProps(id)} />
            ) : (
              <Input
                id={id}
                type='text'
                maxLength={maxLengthOf(field)}
                placeholder={field.placeholder}
                {...draftProps(id)}
              />
            )}
          </div>
        )
      })}
      {config.loginProxy ? (
        <div className='flex min-w-0 flex-col gap-2'>
          <Label>{t('登录出网代理')}</Label>
          <ProxyForm draft={proxyDraft} onChange={setProxyDraft} idPrefix={`${prefix}-manual-proxy`} />
          <p className='detail'>
            {t('添加时要用它出网校验凭证（例如 Antigravity 要打一次 Google 的 token 端点）。无头服务器上没有系统代理，这里不选就走直连。')}
          </p>
        </div>
      ) : null}
      {/* in by up ； this 1 to path and failure */}
      <AddFoot host={footerHost} active={visible}
        note={<span className='detail' id={`${prefix}-add-hint`}>{hint}</span>}>
        <Button id={`${prefix}-add-button`} onClick={() => { void submit() }} disabled={busy}>
          {busy ? t('添加中…') : addButtonTextOf(config)}
        </Button>
      </AddFoot>
    </AddGroup>
  )
}

/* ─── from ─────────────────── */

function DesktopSection({
  config,
  visible,
  footerHost,
}: {
  config: ProviderConfig
  visible: boolean
  footerHost: HTMLElement | null
}): React.ReactElement | null {
  const prefix = config.provider
  const [busy, setBusy] = React.useState(false)
  if (!desktopImportAvailable(config) || !config.desktopNote) return null

  async function submit(): Promise<void> {
    await runAdd(setBusy, async () => {
      try {
        const data = (await postAccount({ provider: config.provider, importDesktop: true })) as
          | { account?: AccountRecord }
          | null
        await afterAdd(data?.account?.name || '', config.label)
      } catch (error) {
        // not to client when after to 400 + Reason， can 
        toast(t('导入失败：{reason}', { reason: describeError(error) }), 'err')
      }
    })
  }

  return (
    <AddGroup label={t('从本机导入桌面端登录态')} hidden={!visible}>
      <p>{config.desktopNote}</p>
      <AddFoot host={footerHost} active={visible}>
        <Button
          id={`${prefix}-desktop-button`}
          title={config.desktopHint}
          onClick={() => { void submit() }}
          disabled={busy}
        >
          {busy ? t('导入中…') : t('从本机导入桌面端登录态')}
        </Button>
      </AddFoot>
    </AddGroup>
  )
}

/* ─── （ has ）────────────────── */

/**
 * 「」： 1 by + 1 optional， down by 。
 *
 * ── Why 1 （ not is and ManualSection）──────────────
 * ** no has **： use not need need token is ， 1 down has 。
 * and 「」 will this path「 has 」， that and 。
 * has `{ provider, guest: true, name? }` —— after `api::accounts` 
 * Chatglm by `guest` `guest/access` （ that ）。
 */
function GuestSection({
  config,
  visible,
  footerHost,
}: {
  config: ProviderConfig
  visible: boolean
  footerHost: HTMLElement | null
}): React.ReactElement | null {
  const guest = config.guestLogin
  const [busy, setBusy] = React.useState(false)
  if (!guest) return null
  const nameId = fieldIdOf(config, { key: 'name', label: '', placeholder: '' })

  async function submit(): Promise<void> {
    await runAdd(setBusy, async () => {
      // from （ and ManualSection 1 ：empty to after ）
      const input = document.getElementById(nameId) as HTMLInputElement | null
      const name = input?.value.trim() || ''
      try {
        const data = (await postAccount({
          provider: config.provider,
          guest: true,
          ...(name ? { name } : {}),
        })) as { account?: AccountRecord } | null
        await afterAdd(data?.account?.name || '', config.label)
      } catch (error) {
        toast(t('添加失败：{reason}', { reason: describeError(error) }), 'err')
      }
    })
  }

  return (
    <AddGroup label={t('访客模式')} hidden={!visible}>
      {guest.noteHtml ? <Note html={guest.noteHtml} /> : null}
      <div className='form-field'>
        <Label htmlFor={nameId}>{t('备注名')}</Label>
        <Input
          id={nameId}
          type='text'
          maxLength={MAX_NAME_LENGTH}
          placeholder={guest.namePlaceholder || t('可选')}
          {...draftProps(nameId)}
        />
      </div>
      <AddFoot host={footerHost} active={visible}>
        <Button id={`${config.provider}-guest-button`} onClick={() => { void submit() }} disabled={busy}>
          {busy ? guest.busyText || t('正在添加…') : guest.button}
        </Button>
      </AddFoot>
    </AddGroup>
  )
}

/* ─── file（ use ；WorkBuddy that is 1 payload， not need and ）─── */


function base64BodyOf(dataUrl: string): string {
  const comma = dataUrl.indexOf(',')
  return comma >= 0 ? dataUrl.slice(comma + 1) : dataUrl
}

/**
 * 「file」（ use ）： use client file， before 
 * （ → `authFileContent`； → `authFileB64`） to 。
 *
 * and `DesktopSection` ： that has 1 by 、 is path； this 
 * file、file can any / ， after is ** can ** 。
 * WorkBuddy not and ： file****request
 * （credentials ），payload and this not 。
 */
function FileImportSection({
  config,
  visible,
  footerHost,
}: {
  config: ProviderConfig
  visible: boolean
  footerHost: HTMLElement | null
}): React.ReactElement | null {
  const [busy, setBusy] = React.useState(false)
  // in file：KukuAI Cookies no has ， by not is 1 
  const [hint, setHint] = React.useState('')
  const fileInput = React.useRef<HTMLInputElement | null>(null)
  const spec = config.fileImport
  // （ and ManualSection name 1 ）
  const nameId = `${config.provider}-file-name`
  if (!spec) return null

  async function importFile(file: File): Promise<void> {
    const fileImport = config.fileImport
    if (!fileImport) return
    await runAdd(setBusy, async () => {
      try {
        const remark = readField(nameId)
        const payload: Record<string, unknown> = {
          provider: config.provider,
          // fileused forerror（ after will file up ）， not and 
          authFileName: file.name,
        }
        if (fileImport.binary) {
          const prefix = await file.slice(0, 64).text()
          if (prefix.trimStart().startsWith('{')) {
            const parsed: unknown = JSON.parse(await file.text())
            if (!isAccountExport(parsed)) {
              throw new Error(t('请选择客户端凭证文件，或本软件导出的单账号 JSON 文件'))
            }
            Object.assign(payload, accountFilePayload(parsed, config.provider, remark))
          } else {
            payload.authFileB64 = base64BodyOf(await readFileAsDataUrl(file, t('文件读取失败')))
          }
        } else {
          const text = await file.text()
          // JSON 「file」 this 1 ：failure / not is for in this 
          // ， not is after 1 error（ and WorkBuddy to ）
          let parsed: unknown
          try {
            parsed = JSON.parse(text)
          } catch {
            toast(t('选择的文件不是合法 JSON'), 'err')
            return
          }
          if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
            toast(t('不是有效的凭证文件：内容要是一个 JSON 对象'), 'err')
            return
          }
          if (isAccountExport(parsed)) {
            Object.assign(payload, accountFilePayload(parsed, config.provider, remark))
          } else {
            // file by ；file 1 ，。
            payload.authFileContent = text
          }
        }
        if (remark) payload.name = remark
        const data = (await postAccount(payload)) as { account?: AccountRecord } | null
        setHint('')
        await afterAdd(addedLabelOf(data?.account), config.label)
      } catch (error) {
        // file （ / not / invalid） after to 400 + file 
        // Reason， can 
        toast(t('导入失败：{reason}', { reason: describeError(error) }), 'err')
      }
    })
  }

  return (
    <AddGroup label={t('导入凭证文件')} hidden={!visible}>
      <Note text={spec.note} />
      <p className='detail'>{t('也支持本软件导出的单账号 JSON 文件，保留账号凭证与附加信息。')}</p>
      <Field orientation='horizontal' className='field-row'>
        <FieldLabel className='lb-inline' htmlFor={nameId}>{t('备注名')}</FieldLabel>
        <Input
          id={nameId}
          type='text'
          maxLength={MAX_NAME_LENGTH}
          placeholder={t('可选，留空则用文件里的昵称')}
          {...draftProps(nameId)}
        />
      </Field>
      <Input
        ref={fileInput}
        type='file'
        // empty = not （KukuAI Cookies no has ，none from by ）
        accept={spec.accept}
        hidden
        onChange={event => {
          const file = event.target.files?.[0]
          // empty value： 1 file after 1 need can onChange
          event.target.value = ''
          if (file) {
            setHint(file.name)
            void importFile(file)
          }
        }}
      />
      <AddFoot host={footerHost} active={visible} note={<span className='detail'>{hint}</span>}>
        <Button onClick={() => fileInput.current?.click()} disabled={busy}>
          {busy ? t('导入中…') : t('选择凭证文件导入')}
        </Button>
      </AddFoot>
    </AddGroup>
  )
}

/* ─── （web-login.js）─────────────────── */

function WebLoginSection({
  config,
  mode,
  onModeChange,
  region,
  visible,
  footerHost,
}: {
  config: ProviderConfig
  mode: string
  onModeChange: (value: string) => void
  region: string
  visible: boolean
  footerHost: HTMLElement | null
}): React.ReactElement | null {
  const web = config.webLogin
  const prefix = config.provider
  const modeRef = React.useRef(mode)
  const regionRef = React.useRef(region)
  const controllerRef = React.useRef<WebLoginController | null>(null)
  // ： has `config.loginProxy` this 1 （ up must
  // can that ）。 ref is as `start` in when ，
  // state will to empty。
  const [proxyDraft, setProxyDraft] = React.useState(() => draftOfProxy(undefined))
  const proxyRef = React.useRef(proxyDraft)
  proxyRef.current = proxyDraft
  const [proxyError, setProxyError] = React.useState('')
  modeRef.current = mode
  regionRef.current = region
  // no has 「 / 」： down mode after not use ，
  // 1 window.open new （ web_shim.rs startLogin）。 this 1 
  // ， and as 1 that 1 。
  const isWeb = platform() === 'web'
  const [loginView, setLoginView] = React.useState<WebLoginView>(() => ({
    label: web?.button || '',
    hint: isWeb ? (web?.webHint || WEB_LOGIN_HINT) : (web?.hint || web?.modes?.[0]?.hint || ''),
    disabled: false,
    waiting: false,
  }))

  React.useEffect(() => {
    const engine = shared().wbWebLogin
    if (!engine || !web) return
    const modes = web.modes
    // in when （ and old create 1 to ）： not 
    // to up when ， to this 1 when use and already is for 
    controllerRef.current = engine.create({
      provider: prefix,
      buttonId: `${prefix}-web-button`,
      cancelId: `${prefix}-web-cancel`,
      hintId: `${prefix}-web-hint`,
      busyText: web.busyText,
      onStateChange: setLoginView,
      texts: () => {
        const active = modes?.find(item => item.value === modeRef.current)
        return {
          button: web.button,
          hint: isWeb ? (web.webHint || WEB_LOGIN_HINT) : (active?.hint || web.hint || ''),
        }
      },
      // edition to ： edition → current（Qoder need ）
      // → 'cn'（ its no has ， after that not this ）
      start: () => {
        // in `loginProxy` up ； its not this ，
        // request and before 1 。failure（e.g.） down ——
        // empty， use will to 「 not up up 」。
        let options: { proxy?: unknown } | undefined
        if (config.loginProxy) {
          try {
            options = { proxy: readProxyDraft(proxyRef.current) }
          } catch (error) {
            setProxyError(describeError(error))
            throw error
          }
          setProxyError('')
        }
        return shared().workbuddyDesktop?.startLogin(
          web.edition || regionRef.current || 'cn',
          modeRef.current || 'embedded',
          prefix,
          undefined,
          options,
        )
      },
      onSuccess: async result => {
        closeAddModals()
        await shared().wbApp?.refresh?.()
        // （ KukuAI「 up 」）： already ，
        // need use can can need need 1 ， not is etc new 。
        const warning = (result as { payload?: { warning?: string } } | null)?.payload?.warning
        if (warning) toast(t('账号已添加，但请留意：{warning}', { warning }), 'err')
        else toast(t('✅ {label}账号已添加', { label: config.label }))
      },
    }) ?? null
    return () => { controllerRef.current?.dispose(); controllerRef.current = null }
  }, [])

  // （ and not ）
  React.useEffect(() => {
    controllerRef.current?.syncTexts()
  }, [mode])

  if (!web) return null
  return (
    <AddGroup label={t('网页登录')} hidden={!visible}>
      <Note html={web.noteHtml} />
      {web.modes?.length && !isWeb ? (
        <div className='flex flex-col gap-2.5 border-l-2 border-border pl-[11px]'>
          <span className='text-[12.5px] text-subtle'>{t('打开方式')}</span>
          <SegmentedControl
            aria-label={t('{label} 网页登录的打开方式', { label: config.label })}
            options={web.modes}
            value={mode}
            onValueChange={onModeChange}
          />
        </div>
      ) : null}
      {config.loginProxy ? (
        <div className='flex min-w-0 flex-col gap-2'>
          <Label>{t('登录出网代理')}</Label>
          <ProxyForm
            draft={proxyDraft}
            onChange={value => { setProxyDraft(value); setProxyError('') }}
            idPrefix={`${prefix}-login-proxy`}
            allowRotate={false}
          />
          <p className='detail'>
            {t('授权页、换证与取账号信息都走这条线路。无头服务器上没有系统代理，这里不选就走直连（多数网络里到不了 Google）。')}
          </p>
          {proxyError ? <Alert variant='destructive'><AlertDescription>{proxyError}</AlertDescription></Alert> : null}
        </div>
      ) : null}
      <AddFoot host={footerHost} active={visible}
        note={<span className='detail' id={`${prefix}-web-hint`}>{loginView.hint}</span>}>
        <Button id={`${prefix}-web-button`} disabled={loginView.disabled || (isWeb && (web.desktopOnly || config.provider === 'doubaowork'))} onClick={() => controllerRef.current?.start()}>
          {loginView.waiting && <Spinner />}{loginView.label}
        </Button>
        <Button
          id={`${prefix}-web-cancel`}
          variant='outline'
          hidden={!loginView.waiting}
          onClick={() => controllerRef.current?.cancel()}
        >
          {t('取消等待')}
        </Button>
      </AddFoot>
    </AddGroup>
  )
}

/* ─── （sms-login.js）─────────────── */

/**
 * （ ui/css/page-accounts-providers.css .sms-form 1 ）：
 *
 * * [ 11 | get ] ← by in 
 * * [ 6 ] [ optional，empty then use ]
 * [ and ] …
 *
 * id and old 1 （ create() when by id click、 after by id 
 * .value and by / use ）， is that —— this not can 。
 */
function SmsSection({
  config,
  visible,
  footerHost,
}: {
  config: ProviderConfig
  visible: boolean
  footerHost: HTMLElement | null
}): React.ReactElement | null {
  const sms = config.smsLogin
  const prefix = config.provider

  React.useEffect(() => {
    if (!sms) return
    // DOM by down JSX （id and old 1 ）， in 
    shared().wbSmsLogin?.create({
      provider: prefix,
      onSuccess: data => afterAdd(addedLabelOf((data as { account?: AccountRecord } | null)?.account), config.label),
    })
  }, [])

  if (!sms) return null
  return (
    <AddGroup label={t('手机验证码登录')} hidden={!visible}>
      <Note
        html={sms.noteHtml}
        text={t('用 AutoClaw 账号绑定的手机号登录：点击「获取验证码」，收到短信后填入下方并登录。验证码由本机直接提交给官方接口，界面不显示 token。')}
      />
      <div className='sms-form'>
        {/* 1 ： and 1 。 by in （ 
            InputGroup， is as 「 1 」 ）， not 1 */}
        <div className='sms-field'>
          <Label className='lb' htmlFor={`${prefix}-sms-phone`}>
            {t('手机号')}<i className='req'>*</i>
          </Label>
          <InputGroup>
            <InputGroupInput
              id={`${prefix}-sms-phone`}
              type='text'
              maxLength={11}
              placeholder={t('11 位大陆手机号')}
              {...draftProps(`${prefix}-sms-phone`)}
            />
            <InputGroupAddon align='inline-end'>
              {/* in create() when up this 1 click（ and use ）。
                  ghost + ： not ， not in 30px 1 */}
              <InputGroupButton id={`${prefix}-sms-send`} className='text-primary-fg'>
                {t('获取验证码')}
              </InputGroupButton>
            </InputGroupAddon>
          </InputGroup>
        </div>

        {/* 2 ： 118px（6 ）， and up is */}
        <div className='sms-field'>
          <Label className='lb' htmlFor={`${prefix}-sms-code`}>
            {t('验证码')}<i className='req'>*</i>
          </Label>
          <div className='sms-pair'>
            {/* ** not **： in success after will （codeNode.value = ''），
                 if ， down will this already use —— and old */}
            <Input
              id={`${prefix}-sms-code`}
              className='sms-code'
              type='text'
              maxLength={6}
              placeholder={t('6 位数字')}
            />
            <Label className='lb-inline' htmlFor={`${prefix}-sms-name`}>{t('备注名')}</Label>
            <Input
              id={`${prefix}-sms-name`}
              className='sms-name'
              type='text'
              placeholder={t('可选，留空则用脱敏手机号')}
              {...draftProps(`${prefix}-sms-name`)}
            />
          </div>
        </div>

        <AddFoot host={footerHost} active={visible}
          note={<span className='detail sms-hint' id={`${prefix}-sms-hint`} />}>
          <Button id={`${prefix}-sms-submit`}>{t('登录并添加')}</Button>
        </AddFoot>
      </div>
    </AddGroup>
  )
}

/* ─── AutoClaw OAuth （autoclaw-oauth.js）─── */

function OauthSection({
  config,
  mode,
  onModeChange,
  visible,
  footerHost,
}: {
  config: ProviderConfig
  mode: string
  onModeChange: (value: string) => void
  visible: boolean
  footerHost: HTMLElement | null
}): React.ReactElement | null {
  const oauth = config.oauthLogin
  const prefix = config.provider
  const modeRef = React.useRef(mode)
  const controllerRef = React.useRef<OauthController | null>(null)
  modeRef.current = mode
  // and WebLoginSection 1 ： no has 「 / 」，
  // 1 in new （ web_shim.rs startAutoclawOauthLogin）
  const isWeb = platform() === 'web'

  const hintOf = (): string =>
    isWeb
      ? (oauth?.webHint || WEB_LOGIN_HINT)
      : (oauth?.modes?.find(item => item.value === modeRef.current)?.hint || oauth?.hint || '')

  React.useEffect(() => {
    const engine = shared().wbAutoclawOauth
    if (!engine || !oauth) return
    controllerRef.current = engine.create({
      provider: prefix,
      // （ use can can in that 1 ）， will after 1 
      mode: () => modeRef.current || 'embedded',
      hint: hintOf,
      // by ： = etc ， etc = that 1 
      cancelId: `${prefix}-oauth-cancel`,
      // not ： this by in （ {ok:true}），
      // provider name will to 「AutoClaw already ：AutoClaw 」 this 
      onSuccess: () => afterAdd('', config.label),
    }) ?? null
  }, [])

  React.useEffect(() => {
    controllerRef.current?.syncTexts()
  }, [mode])

  if (!oauth) return null
  return (
    <AddGroup label={oauth.title || t('网页登录')} hidden={!visible}>
      <Note html={oauth.noteHtml} />
      {oauth.modes?.length && !isWeb ? (
        <div className='flex flex-col gap-2.5 border-l-2 border-border pl-[11px]'>
          <span className='text-[12.5px] text-subtle'>{t('打开方式')}</span>
          <SegmentedControl
            aria-label={t('{label} 网页登录的打开方式', { label: config.label })}
            options={oauth.modes}
            value={mode}
            onValueChange={onModeChange}
          />
        </div>
      ) : null}
      <AddFoot host={footerHost} active={visible}
        note={<span className='detail' id={`${prefix}-oauth-hint`}>{hintOf()}</span>}>
        {/* 1 ： up is 、， use down will use 。
            ** not in this **：autoclaw-oauth.js create() will to this 
            click（ old 1 ）， this 1 will start() —— 
            must in DOM （ add-account-kit AddFoot ） */}
        <Button id={`${prefix}-oauth-zai`}>{t('使用 Zai 账号登录')}</Button>
        <Button id={`${prefix}-oauth-google`} variant='outline'>{t('使用 Google 账号登录')}</Button>
        {/* by by autoclaw-oauth.js paintCancel （ is hidden ） */}
        <Button id={`${prefix}-oauth-cancel`} variant='outline' hidden>
          {t('取消')}
        </Button>
      </AddFoot>
    </AddGroup>
  )
}

/* ─── 1 （ / / ）─────────── */

export function ProviderBlock({
  config,
  active,
  footerHost,
}: {
  config: ProviderConfig
  active: boolean
  footerHost: HTMLElement | null
}): React.ReactElement {
  const initialRegion = config.regionOptions?.[0]?.value ?? ''
  const [region, setRegion] = React.useState(initialRegion)
  const [method, setMethod] = React.useState<MethodId>(() => methodsOf(config, initialRegion)[0])
  const [webMode, setWebMode] = React.useState(config.webLogin?.modes?.[0]?.value ?? '')
  const [oauthMode, setOauthMode] = React.useState(config.oauthLogin?.modes?.[0]?.value ?? '')

  // after ： that 1 can can is current in ， when to 1 
  //（otherwise up will 「all、 1 can in no has 」）
  const methods = methodsOf(config, region)
  const effective = methods.includes(method) ? method : methods[0]
  const methodOptions = React.useMemo(
    () => ADD_METHODS.filter(item => methods.includes(item.id)).map(item => ({
      value: item.id,
      label: item.label,
    })),
    [methods.join(',')],
  )

  return (
    <div className='add-provider-block' hidden={!active}>
      {config.regionOptions?.length ? (
        <AddGroup label={t('地区')}>
          <SegmentedControl
            aria-label={t('{label} 账号地区', { label: config.label })}
            className={DIALOG_SEG_CLASS}
            options={config.regionOptions}
            value={region}
            onValueChange={setRegion}
          />
        </AddGroup>
      ) : null}

      <AddGroup label={t('添加方式')}>
        <SegmentedControl
          aria-label={t('{label} 账号的添加方式', { label: config.label })}
          className={DIALOG_SEG_CLASS}
          options={methodOptions}
          value={effective}
          onValueChange={value => setMethod(value as MethodId)}
        />
      </AddGroup>

      {/* `visible` need 「 this 1 is not is current in 」 1 ： 
          hidden in ， by is portal to —— no has 
          。 this 1 ， will when in that 1 
          （ 1 ，）。

          Note not can 「 not in not to footerHost」： that will by from to 
          ， 1 ，sms-login / autoclaw-oauth in create() when click 。 */}
      <GuestSection config={config} visible={active && effective === 'guest'} footerHost={footerHost} />
      <OauthSection
        config={config}
        mode={oauthMode}
        onModeChange={setOauthMode}
        visible={active && effective === 'oauth'}
        footerHost={footerHost}
      />
      <SmsSection config={config} visible={active && effective === 'sms'} footerHost={footerHost} />
      <WebLoginSection
        config={config}
        mode={webMode}
        onModeChange={setWebMode}
        region={region}
        visible={active && effective === 'web'}
        footerHost={footerHost}
      />
      <ManualSection config={config} region={region} visible={active && effective === 'manual'} footerHost={footerHost} />
      <FileImportSection config={config} visible={active && effective === 'file'} footerHost={footerHost} />
      <DesktopSection config={config} visible={active && effective === 'desktop'} footerHost={footerHost} />
    </div>
  )
}

/* ─── WorkBuddy file ─────────────────── */

/** clientdirectory（「file in 」 when ； / use 1 directory） */
const WORKBUDDY_AUTH_PATH = t(
  '官方客户端凭证目录：Windows 是 C:\\Users\\<你的用户名>\\AppData\\Local\\CodeBuddyExtension\\Data\\Public\\auth，macOS 是 ~/Library/Application Support/CodeBuddyExtension/Data/Public/auth',
)

/** id（ by id ， add-account-bridge draft ） */
const WORKBUDDY_FILE_NAME_ID = 'workbuddy-file-name'

/**
 * from for ： for （`auth` / `account`， `.info` and wb2api
 * ）， not to （wb2api ）。 and after 
 * `store_crud::add_account` 1 ， before 1 is as 「file」
 * 1 ， not is after 400 「 accessToken」。
 */
function credentialFieldOf(
  record: Record<string, unknown>,
  keys: string[],
  nestedKey: string,
): string {
  const nested = record[nestedKey]
  const sources: Array<Record<string, unknown>> = []
  if (nested && typeof nested === 'object' && !Array.isArray(nested)) {
    sources.push(nested as Record<string, unknown>)
  }
  sources.push(record)
  for (const source of sources) {
    for (const key of keys) {
      const value = source[key]
      if (typeof value === 'string' && value.trim()) return value.trim()
    }
  }
  return ''
}

/** by file：`auth.domain`（ .info） or `domain`（） */
function editionFromFile(record: Record<string, unknown>): Edition | null {
  const domain = credentialFieldOf(record, ['domain'], 'auth')
  if (/workbuddy\.ai/i.test(domain)) return 'intl'
  if (/workbuddy\.cn|codebuddy\.cn/i.test(domain)) return 'cn'
  return null
}

/** WorkBuddy 「」 id。 not ADD_METHODS： not use ， this */
type WorkBuddyMethod = 'web' | 'file' | 'desktop'

/**
 * 「file」（WorkBuddyBlock use ）：file —— client
 * （`.info`） or wb2api JSON。「 from 」 is 
 * （ `WorkBuddyDesktopImportSection`）， not in 。
 * （ / ） by file domain ：file 
 * 「」， not use current。
 */
function WorkBuddyFileImportSection({
  edition,
  visible,
  footerHost,
}: {
  edition: Edition
  visible: boolean
  footerHost: HTMLElement | null
}): React.ReactElement {
  const [busy, setBusy] = React.useState(false)
  const [hint, setHint] = React.useState('')
  const fileInput = React.useRef<HTMLInputElement | null>(null)

  async function importParsed(file: File): Promise<void> {
    await runAdd(setBusy, async () => {
      try {
        let parsed: unknown
        try {
          parsed = JSON.parse(await file.text())
        } catch {
          toast(t('选择的文件不是合法 JSON'), 'err')
          return
        }
        if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
          toast(t('不是有效的凭证文件：内容要是一个 JSON 对象'), 'err')
          return
        }
        const record = parsed as Record<string, unknown>
        const accessToken = credentialFieldOf(record, ['accessToken', 'token', 'access_token'], 'auth')
        const uid = credentialFieldOf(record, ['uid'], 'account')
        if (!accessToken || !uid) {
          toast(t('不是有效的凭证文件：需要包含 accessToken 与 uid'), 'err')
          return
        }
        const fromFile = accountExportEdition(record) ?? editionFromFile(record)
        const landed = fromFile ?? edition
        if (fromFile && fromFile !== edition) {
          toast(
            t('文件是{edition}凭证，已按该版本导入', {
              edition: fromFile === 'intl' ? t('国际版') : t('国内版'),
            }),
            'warn',
          )
        }
        const remark = readField(WORKBUDDY_FILE_NAME_ID)
        const payload: Record<string, unknown> = isAccountExport(parsed)
          ? accountFilePayload(record, workbuddyProviderId(landed), remark)
          : { ...record, provider: workbuddyProviderId(landed) }
        if (remark) payload.name = remark
        const data = (await postAccount(payload)) as { account?: AccountRecord } | null
        setHint('')
        await afterAdd(addedLabelOf(data?.account), landed === 'intl' ? 'WorkBuddy AI' : 'WorkBuddy')
      } catch (error) {
        toast(t('导入失败：{reason}', { reason: describeError(error) }), 'err')
      }
    })
  }

  return (
    <AddGroup label={t('导入凭证文件')} hidden={!visible}>
      <Note text={t('选择官方客户端的登录态凭证文件（workbuddy-desktop.info / workbuddy-desktop-ai.info）或 wb2api 导出的账号 JSON（嵌套 / 扁平形均可）。')} />
      <p className='detail'>{t('也支持本软件导出的单账号 JSON 文件，保留账号凭证与附加信息。')}</p>
      <p className='detail'>
        <a
          href='#'
          onClick={event => {
            event.preventDefault()
            setHint(WORKBUDDY_AUTH_PATH)
          }}
        >
          {t('凭证文件在哪？')}
        </a>
      </p>
      <Field orientation='horizontal' className='field-row'>
        <FieldLabel className='lb-inline' htmlFor={WORKBUDDY_FILE_NAME_ID}>{t('备注名')}</FieldLabel>
        <Input
          id={WORKBUDDY_FILE_NAME_ID}
          type='text'
          maxLength={100}
          placeholder={t('可选，留空则用文件里的昵称')}
          {...draftProps(WORKBUDDY_FILE_NAME_ID)}
        />
      </Field>
      <Input
        ref={fileInput}
        type='file'
        accept='.info,.json,application/json'
        hidden
        onChange={event => {
          const file = event.target.files?.[0]
          // empty value： 1 file after 1 need can onChange
          event.target.value = ''
          if (file) void importParsed(file)
        }}
      />
      <AddFoot host={footerHost} active={visible} note={<span className='detail'>{hint}</span>}>
        <Button onClick={() => fileInput.current?.click()} disabled={busy}>
          {busy ? t('导入中…') : t('选择凭证文件导入')}
        </Button>
      </AddFoot>
    </AddGroup>
  )
}

/**
 * 「 from 」（WorkBuddyBlock use ）： after client
 * directory、 new file（`importDesktop` ，
 * `account_store::workbuddy_import`）。 and 「file」 ——
 * for etc DesktopSection ： 1 by ， not file。
 */
function WorkBuddyDesktopImportSection({
  edition,
  visible,
  footerHost,
}: {
  edition: Edition
  visible: boolean
  footerHost: HTMLElement | null
}): React.ReactElement {
  const [busy, setBusy] = React.useState(false)

  async function importFromDesktop(): Promise<void> {
    await runAdd(setBusy, async () => {
      try {
        const data = (await postAccount({
          provider: workbuddyProviderId(edition),
          importDesktop: true,
        })) as { account?: AccountRecord } | null
        await afterAdd(addedLabelOf(data?.account), edition === 'intl' ? 'WorkBuddy AI' : 'WorkBuddy')
      } catch (error) {
        // not to client when after to 400 + Reason， can 
        toast(t('导入失败：{reason}', { reason: describeError(error) }), 'err')
      }
    })
  }

  return (
    <AddGroup label={t('从本机导入桌面端登录态')} hidden={!visible}>
      <Note text={t('读本机官方客户端（Tencent WorkBuddy / WorkBuddy AI）当前登录的账号：自动定位其凭证目录里最新的登录态文件（需已在本机官方客户端登录）。导入后由网关自动续期，无需跟随 2 小时时效手动换令牌。')} />
      <AddFoot host={footerHost} active={visible}>
        <Button onClick={() => void importFromDesktop()} disabled={busy}>
          {busy ? t('导入中…') : t('从本机导入桌面端登录态')}
        </Button>
      </AddFoot>
    </AddGroup>
  )
}

/**
 * → provider id。
 *
 * WorkBuddy （2026-10） after is ** provider**（`workbuddy` /
 * `workbuddy-intl`， `providers::workbuddy::region`）。 up is 1 
 * 「」 1 ， this in this 1 —— by 
 * 「 this 1 」、 after by 1 ， 1 is 
 * 「 use 、」（ 401）。
 */
function workbuddyProviderId(edition: Edition): string {
  return edition === 'intl' ? 'workbuddy-intl' : 'workbuddy'
}

/* ─── WorkBuddy（ + [ / file / ]）───
 *
 * and its not ： has （ / ） and 1 
 * ， and ；「」 in 、
 * file and （ ProviderBlock ：hidden ，
 * not is ）。 old this DOM in index.html 、 in add-account.js；
 * to up after in this ，index.html that 47 delete。 */

export function WorkBuddyBlock({ active, footerHost }: { active: boolean; footerHost: HTMLElement | null }): React.ReactElement {
  // loginPrefs： old in DOM up ，
  // is up （resetAddStep and ）
  const [edition, setEdition] = React.useState<Edition>(loginPrefs.edition)
  const [loginMode, setLoginMode] = React.useState<LoginMode>(loginPrefs.loginMode)
  const [socialRestore, setSocialRestore] = React.useState(loginPrefs.socialRestore)
  // no has 「 / 」： 1 in new ，
  // mode down （ web_shim.rs startLogin）。 and 
  // —— after in 「」 down ， is by not 。
  const webPanel = platform() === 'web'
  const [loginView, setLoginView] = React.useState<WebLoginView>(() => ({
    label: t('打开网页登录'),
    hint: webPanel ? WEB_LOGIN_HINT : t('内嵌窗口打开；完成后自动加入列表，关窗即取消等待'),
    disabled: false,
    waiting: false,
  }))
  // （ and ProviderBlock 「」 1 ）： not loginPrefs
  // —— use method not ，default
  const [method, setMethod] = React.useState<WorkBuddyMethod>('web')
  const editionRef = React.useRef(edition)
  const modeRef = React.useRef(loginMode)
  const socialRef = React.useRef(socialRestore)
  const controllerRef = React.useRef<WebLoginController | null>(null)
  editionRef.current = edition
  modeRef.current = loginMode
  socialRef.current = socialRestore

  React.useEffect(() => {
    const engine = shared().wbWebLogin
    if (!engine) return
    controllerRef.current = engine.create({
      // provider ****（getter）： by 
      // `loginProvider === config.provider` 「 etc in is not is 」，
      // 1 id will to after by use 、 and 
      // （ is workbuddy-intl）。
      get provider() {
        return workbuddyProviderId(editionRef.current)
      },
      buttonId: 'web-login-button',
      cancelId: 'web-login-cancel',
      hintId: 'web-login-hint',
      busyText: t('等待网页登录…'),
      onStateChange: setLoginView,
      texts: () => {
        // has 1 ： new 。 （「」「
        // 」） in this not —— startLogin down mode not use 
        if (webPanel) return { button: t('打开网页登录'), hint: WEB_LOGIN_HINT }
        const external = modeRef.current === 'external'
        return {
          button: external ? t('在浏览器中打开登录页') : t('打开网页登录'),
          hint: external
            ? t('系统浏览器打开（复用已登录账号）；完成后自动加入列表')
            : t('内嵌窗口打开；完成后自动加入列表，关窗即取消等待'),
        }
      },
      start: () => shared().workbuddyDesktop?.startLogin(
        editionRef.current,
        modeRef.current,
        // current： after is （ after by provider id ，
        // not from `edition` ， api::session login_start）
        workbuddyProviderId(editionRef.current),
        socialRef.current,
      ),
      onSuccess: async () => {
        const editionLabel = editionRef.current === 'intl' ? t('国际版') : t('国内版')
        closeAddModals()
        await shared().wbApp?.refresh?.()
        toast(t('✅ 登录成功，{edition}账号已加入列表', { edition: editionLabel }))
      },
    }) ?? null
    return () => { controllerRef.current?.dispose(); controllerRef.current = null }
  }, [])

  // by and （ etc in not ， to applyState）
  React.useEffect(() => {
    controllerRef.current?.syncTexts()
  }, [edition, loginMode])

  /** need ：default、default（ use before ） */
  function pickEdition(next: Edition): void {
    const mode: LoginMode = next === 'intl' ? 'external' : 'embedded'
    setEdition(next)
    loginPrefs.edition = next
    setLoginMode(mode)
    loginPrefs.loginMode = mode
  }

  function pickLoginMode(next: string): void {
    const mode: LoginMode = next === 'external' ? 'external' : 'embedded'
    setLoginMode(mode)
    loginPrefs.loginMode = mode
  }

  /**
   * 「 Google / GitHub 」 available：** + **。
   * down in use ， no has can ；
   * no has this （ use / / / SSO）， will ignore this 。
   * not when ， to 1 「 not 」 。
   */
  const intl = edition === 'intl'
  const embedded = loginMode === 'embedded'
  const socialUsable = intl && embedded
  const socialTitle = socialUsable
    ? t('国际版登录页默认只显示邮箱登录，勾选后恢复 Google / GitHub / X 入口')
    : !intl
      ? t('国内版登录页没有 Google / GitHub 入口（它用微信 / 手机号 / 邮箱登录）')
      : t('只有「内嵌窗口」能恢复第三方入口：系统浏览器里我们无法改动登录页')

  // ： no has client，「」
  // （ and desktopImportAvailable platform 1 ）。current in 
  // when ，「all、 1 can in no has 」。
  const effectiveMethod: WorkBuddyMethod =
    method === 'desktop' && platform() === 'web' ? 'web' : method

  return (
    <div className='add-provider-block' hidden={!active}>
      <AddGroup label={t('账号版本')}>
        <p>
          {t('两版账号可同时保存，各自一份模型清单（国内版与国际版是两家提供商，可分别启用与映射）。')}
        </p>
        <SegmentedControl
          aria-label={t('账号版本')}
          className={DIALOG_SEG_CLASS}
          options={[
            { value: 'cn', label: t('国内版（WorkBuddy）') },
            { value: 'intl', label: t('国际版（WorkBuddy AI）') },
          ]}
          value={edition}
          onValueChange={value => pickEdition(value as Edition)}
        />
      </AddGroup>

      <AddGroup label={t('添加方式')}>
        <SegmentedControl
          aria-label={t('{label} 账号的添加方式', { label: 'WorkBuddy' })}
          className={DIALOG_SEG_CLASS}
          options={[
            { value: 'web', label: t('网页登录') },
            { value: 'file', label: t('导入凭证文件') },
            ...(platform() !== 'web'
              ? [{ value: 'desktop', label: t('导入桌面端登录态') }]
              : []),
          ]}
          value={effectiveMethod}
          onValueChange={value => setMethod(value as WorkBuddyMethod)}
        />
      </AddGroup>

      <AddGroup label={t('网页登录')} hidden={effectiveMethod !== 'web'}>
        <p>{t('完成登录后自动加入账号列表，昵称取自上游。')}</p>
        {/* no has 「 / 」（ 1 new ），
             —— that in 「」 down Google / GitHub */}
        {webPanel ? null : (
          <>
            <FieldGroup orientation='horizontal' className='field-row'>
              <span className='text-[12.5px] whitespace-nowrap text-subtle'>{t('打开方式')}</span>
              <SegmentedControl
                aria-label={t('网页登录的打开方式')}
                className={DIALOG_SEG_CLASS}
                options={[
                  { value: 'embedded', label: t('内嵌窗口') },
                  { value: 'external', label: t('系统默认浏览器') },
                ]}
                value={loginMode}
                onValueChange={pickLoginMode}
              />
              <Label className='flex items-center gap-2' title={socialTitle}>
                <Switch
                  checked={socialRestore}
                  disabled={!socialUsable}
                  onCheckedChange={next => {
                    // when to ， not need need 
                    setSocialRestore(next)
                    loginPrefs.socialRestore = next
                  }}
                />
                <span className='text-[12.5px] text-subtle'>{t('恢复 Google / GitHub 入口')}</span>
              </Label>
            </FieldGroup>
            <p>{t('系统浏览器可复用已有登录态（国际版推荐）；内嵌窗口更干净。')}</p>
          </>
        )}
        <AddFoot host={footerHost} active={active && effectiveMethod === 'web'}
          note={<span className='detail' id='web-login-hint'>{loginView.hint}</span>}>
          <Button id='web-login-button' disabled={loginView.disabled} onClick={() => controllerRef.current?.start()}>
            {loginView.waiting && <Spinner />}{loginView.label}
          </Button>
          <Button
            id='web-login-cancel'
            variant='outline'
            hidden={!loginView.waiting}
            onClick={() => controllerRef.current?.cancel()}
          >
            {t('取消等待')}
          </Button>
        </AddFoot>
      </AddGroup>

      <WorkBuddyFileImportSection edition={edition} visible={active && effectiveMethod === 'file'} footerHost={footerHost} />
      <WorkBuddyDesktopImportSection edition={edition} visible={active && effectiveMethod === 'desktop'} footerHost={footerHost} />
    </div>
  )
}
