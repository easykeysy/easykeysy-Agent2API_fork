import * as React from 'react'
import { Button, Input, Label, Field, FieldGroup, FieldLabel, SegmentedControl, Spinner, Switch } from '@ui'
import { AddFoot, AddGroup } from './add-account-kit'
import { WEB_LOGIN_HINT, platform } from './add-account-configs'
import { afterAdd, addedLabelOf, closeAddModals, describeError, draftProps, loginPrefs, postAccount, readField, shared, startLogin, toast, type AccountRecord, type Edition, type LoginMode, type WebLoginController, type WebLoginView } from './add-account-bridge'
import { accountFilePayload, accountExportEdition, isAccountExport } from './add-account-file'
import { Note, runAdd } from './add-provider-common'
import { useAddAccountProxy, useAccountAddBusy } from './add-account-proxy'
import { DIALOG_SEG_CLASS } from './dialog-segmented'
import { t } from '../i18n'

/* ─── WorkBuddy 导入凭证文件 ─────────────────── */

/** 官方客户端凭证目录（点「凭证文件在哪」时显示；国内版 / 国际版共用同一目录） */
const WORKBUDDY_AUTH_PATH = t(
  'Official client credential directory: Windows C:\\Users\\<user>\\AppData\\Roaming\\zhipu-qingyan\\\\Users\\<Your username>\\AppData\\Local\\CodeBuddyExtension\\Data\\Public\\auth, macOS Yes ~/Library/Application Support/CodeBuddyExtension/Data/Public/auth',
)

/** 备注名输入框的 id（草稿按 id 存，见 add-account-bridge 的 draft 机制） */
const WORKBUDDY_FILE_NAME_ID = 'workbuddy-file-name'

/**
 * 从凭证对象取字段：先读嵌套对象（`auth` / `account`，官方 `.info` 与 wb2api
 * 的嵌套形），读不到再读顶层键（wb2api 的手写扁平形）。键名清单与后端
 * `store_crud::add_account` 的取值口径一致，前端先挡一道是为了把「粘错文件」
 * 变成一句人话，而不是后端 400 的「缺少 accessToken」。
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

/** 按文件内容推断归属：`auth.domain`（官方 .info）或顶层 `domain`（扁平形） */
function editionFromFile(record: Record<string, unknown>): Edition | null {
  const domain = credentialFieldOf(record, ['domain'], 'auth')
  if (/workbuddy\.ai/i.test(domain)) return 'intl'
  if (/workbuddy\.cn|codebuddy\.cn/i.test(domain)) return 'cn'
  return null
}

/** WorkBuddy 块「添加方式」的选项 id。不进 ADD_METHODS：本块不走通用分派，选项固定这三项 */
type WorkBuddyMethod = 'web' | 'file' | 'desktop'

/**
 * 「导入凭证文件」段（WorkBuddyBlock 专用）：选文件导入 —— 官方客户端登录态
 * （`.info`）或 wb2api 的账号 JSON。「从本机导入桌面端登录态」是独立的添加
 * 方式（见 `WorkBuddyDesktopImportSection`），不放在本段里。
 * 归属（国内版 / 国际版）优先按文件内容的 domain 推断：文件自己的事实比
 * 「账号版本」分段更权威，推不出才用分段当前值。
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
  const [busy, setBusy] = useAccountAddBusy()
  const proxy = useAddAccountProxy()
  const [hint, setHint] = React.useState('')
  const fileInput = React.useRef<HTMLInputElement | null>(null)

  async function importParsed(file: File): Promise<void> {
    await runAdd(setBusy, async () => {
      try {
        let parsed: unknown
        try {
          parsed = JSON.parse(await file.text())
        } catch {
          toast(t('The selected file is not valid JSON'), 'err')
          return
        }
        if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
          toast(t('Not a valid credential file: the content must be a JSON object'), 'err')
          return
        }
        const record = parsed as Record<string, unknown>
        const accessToken = credentialFieldOf(record, ['accessToken', 'token', 'access_token'], 'auth')
        const uid = credentialFieldOf(record, ['uid'], 'account')
        if (!accessToken || !uid) {
          toast(t('Not a valid credential file: accessToken and uid are required'), 'err')
          return
        }
        const fromFile = accountExportEdition(record) ?? editionFromFile(record)
        const landed = fromFile ?? edition
        if (fromFile && fromFile !== edition) {
          toast(
            t('The file contains {edition} credentials and has been imported as that edition', {
              edition: fromFile === 'intl' ? t('International edition') : t('China edition'),
            }),
            'warn',
          )
        }
        const remark = readField(WORKBUDDY_FILE_NAME_ID)
        const payload: Record<string, unknown> = isAccountExport(parsed)
          ? accountFilePayload(record, workbuddyProviderId(landed), remark)
          : { ...record, provider: workbuddyProviderId(landed) }
        if (remark) payload.name = remark
        Object.assign(payload, isAccountExport(payload) ? proxy.importFields() : { proxy: proxy.read() })
        const data = (await postAccount(payload)) as { account?: AccountRecord } | null
        setHint('')
        await afterAdd(addedLabelOf(data?.account), landed === 'intl' ? 'WorkBuddy AI' : 'WorkBuddy')
      } catch (error) {
        toast(t('Import failed: {reason}', { reason: describeError(error) }), 'err')
      }
    })
  }

  return (
    <AddGroup label={t('Import credential file')} hidden={!visible}>
      <Note text={t('Select the official client\'s session credential file (workbuddy-desktop.info / workbuddy-desktop-ai.info) or an account JSON exported by wb2api (nested or flat).')} />
      <p className='detail'>{t('Individual account JSON files exported by this application are also supported, preserving credentials and additional account data.')}</p>
      <p className='detail'>
        <a
          href='#'
          onClick={event => {
            event.preventDefault()
            setHint(WORKBUDDY_AUTH_PATH)
          }}
        >
          {t('Where is the credential file?')}
        </a>
      </p>
      <Field orientation='horizontal' className='field-row'>
        <FieldLabel className='lb-inline' htmlFor={WORKBUDDY_FILE_NAME_ID}>{t('Label')}</FieldLabel>
        <Input
          id={WORKBUDDY_FILE_NAME_ID}
          type='text'
          maxLength={100}
          placeholder={t('Optional; leave empty to use the nickname from the file')}
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
          // 清空 value：同一份文件改内容后再选一次也要能触发 onChange
          event.target.value = ''
          if (file) void importParsed(file)
        }}
      />
      <AddFoot host={footerHost} active={visible} note={<span className='detail'>{hint}</span>}>
        <Button onClick={() => fileInput.current?.click()} disabled={busy}>
          {busy ? t('Importing…') : t('Import from credential file')}
        </Button>
      </AddFoot>
    </AddGroup>
  )
}

/**
 * 「从本机导入桌面端登录态」段（WorkBuddyBlock 专用）：后端扫描官方客户端凭证
 * 目录、取最新的登录态文件落库（`importDesktop` 链路，见
 * `account_store::workbuddy_import`）。与「导入凭证文件」分成两个添加方式 ——
 * 对齐小浣熊等家 DesktopSection 的形态：一颗按钮，不选文件。
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
  const [busy, setBusy] = useAccountAddBusy()
  const proxy = useAddAccountProxy()

  async function importFromDesktop(): Promise<void> {
    await runAdd(setBusy, async () => {
      try {
        const data = (await postAccount({
          provider: workbuddyProviderId(edition),
          importDesktop: true,
          proxy: proxy.read(),
        })) as { account?: AccountRecord } | null
        await afterAdd(addedLabelOf(data?.account), edition === 'intl' ? 'WorkBuddy AI' : 'WorkBuddy')
      } catch (error) {
        // 读不到官方客户端登录态时后端给 400 + 明确原因，原样透出即可
        toast(t('Import failed: {reason}', { reason: describeError(error) }), 'err')
      }
    })
  }

  return (
    <AddGroup label={t('Import desktop login state from this machine')} hidden={!visible}>
      <Note text={t('Reads the account currently logged in to the official client (Tencent WorkBuddy / WorkBuddy AI) on this machine: the newest session file in its credential directory is located automatically (the official client must be logged in on this machine). The gateway renews the token after import — no need to swap tokens manually with the 2-hour expiry.')} />
      <AddFoot host={footerHost} active={visible}>
        <Button onClick={() => void importFromDesktop()} disabled={busy}>
          {busy ? t('Importing…') : t('Import desktop login state from this machine')}
        </Button>
      </AddFoot>
    </AddGroup>
  )
}

/**
 * 账号版本 → provider id。
 *
 * WorkBuddy 拆家（2026-10）后两个地区是**两家 provider**（`workbuddy` /
 * `workbuddy-intl`，见 `providers::workbuddy::region`）。界面上仍然是同一个
 * 「添加账号」块里的一个分段控件，因此这个映射只在这里写一份 —— 壳侧按它记录
 * 「这次登录属于哪一家」、后端按它决定账号落进哪一组，写错任一处的症状都是
 * 「用国际版登录、账号进了国内版组」（转发稳定 401）。
 */
function workbuddyProviderId(edition: Edition): string {
  return edition === 'intl' ? 'workbuddy-intl' : 'workbuddy'
}

/* ─── WorkBuddy（账号版本 + 添加方式[网页登录 / 导入凭证文件 / 导入桌面端登录态]）───
 *
 * 结构与其余各家不同：它有两处静态分段（账号版本 / 打开方式）与一个第三方入口
 * 开关，且网页登录的文案随打开方式变；「添加方式」分段在网页登录、导入凭证
 * 文件与导入桌面端登录态之间切换（显隐同 ProviderBlock 的手法：hidden 属性，
 * 不是行内样式）。旧实现里这段 DOM 写在 index.html 里、交互在 add-account.js；
 * 迁到岛上后整块在这里，index.html 的那 47 行骨架随之删除。 */

export function WorkBuddyBlock({ active, footerHost, initialEdition }: {
  active: boolean
  footerHost: HTMLElement | null
  initialEdition?: Edition
}): React.ReactElement {
  const proxy = useAddAccountProxy()
  // 三处偏好的初值来自模块级 loginPrefs：旧实现里它们住在静态 DOM 上，
  // 关掉弹窗再打开仍是上次的选择（resetAddStep 只复位提供商与步骤）
  const [edition, setEdition] = React.useState<Edition>(initialEdition ?? loginPrefs.edition)
  const [loginMode, setLoginMode] = React.useState<LoginMode>(initialEdition
    ? initialEdition === 'intl' ? 'external' : 'embedded' : loginPrefs.loginMode)
  const [socialRestore, setSocialRestore] = React.useState(loginPrefs.socialRestore)
  // 网页端没有「内嵌窗口 / 系统浏览器」之分：桥接层一律在新标签页里打开，
  // mode 收下即弃（见 web_shim.rs 的 startLogin）。整行打开方式与第三方入口
  // 开关都收起 —— 后者只在「内嵌窗口」下生效，留着必然是个灰按不动的假选项。
  const webPanel = platform() === 'web'
  const [loginView, setLoginView] = React.useState<WebLoginView>(() => ({
    label: t('Open web login'),
    hint: webPanel ? WEB_LOGIN_HINT : t('Opens in the embedded window; added to the list when done; closing it cancels the wait'),
    disabled: false,
    waiting: false,
  }))
  // 添加方式（与各家 ProviderBlock 的「添加方式」分段同一形态）：不进 loginPrefs
  // 持久化 —— 通用块的 method 也不持久化，弹窗重开回默认项
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
      // provider 随分段控件**动态解析**（getter）：引擎按
      // `loginProvider === config.provider` 判断「等待中的是不是本家」，
      // 写死一个 id 会让切到国际版之后按钮禁用态、取消与提示全部失联
      // （壳侧记的是 workbuddy-intl）。
      get provider() {
        return workbuddyProviderId(editionRef.current)
      },
      buttonId: 'web-login-button',
      cancelId: 'web-login-cancel',
      hintId: 'web-login-hint',
      busyText: t('Waiting for web login…'),
      onStateChange: view => { setLoginView(view); proxy.setBusy(view.disabled) },
      texts: () => {
        // 网页端只有一条路：新标签页。模式相关的文案（「内嵌窗口」「关窗即
        // 取消」）在这里都不成立 —— 桥接层的 startLogin 收下 mode 却不使用它
        if (webPanel) return { button: t('Open web login'), hint: WEB_LOGIN_HINT }
        const external = modeRef.current === 'external'
        return {
          button: external ? t('Open the sign-in page in your browser') : t('Open web login'),
          hint: external
            ? t('Opens in the system browser (reuses a signed-in account); added to the list when done')
            : t('Opens in the embedded window; added to the list when done; closing it cancels the wait'),
        }
      },
      start: () => startLogin(
        workbuddyProviderId(editionRef.current),
        editionRef.current,
        modeRef.current,
        socialRef.current,
        { proxy: proxy.read() },
      ),
      onSuccess: async result => {
        const editionLabel = editionRef.current === 'intl' ? t('International edition') : t('China edition')
        closeAddModals()
        await shared().wbApp?.refresh?.()
        toast(t('✅ Signed in; {edition} account added to the list', { edition: editionLabel }))
        const warning = (result as { warning?: string } | null)?.warning
        if (warning) toast(t('Account added, but note: {warning}', { warning }), 'warn')
      },
    }) ?? null
    return () => { controllerRef.current?.dispose(); controllerRef.current = null }
  }, [])

  // 打开方式影响登录按钮与提示的文案（等待中不覆盖，交给引擎的 applyState）
  React.useEffect(() => {
    controllerRef.current?.syncTexts()
  }, [edition, loginMode])

  /** 切版本要联动打开方式：国际版默认系统浏览器、国内版默认内嵌窗口（沿用改造前的联动） */
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

  // 第三方入口只在国际版的内嵌窗口里有效；切换模式时保留之前的开关偏好。
  const socialUsable = edition === 'intl' && loginMode === 'embedded'

  // 分段选项随平台变化：网页托管面板没有本机官方客户端，「导入桌面端登录态」
  // 整项收起（与 desktopImportAvailable 的 platform 判断同一口径）。当前选中项
  // 被收起时退回网页登录，避免「所有段都藏着、一个可见的选中项也没有」。
  const effectiveMethod: WorkBuddyMethod =
    method === 'desktop' && platform() === 'web' ? 'web' : method

  return (
    <div className='add-provider-block' hidden={!active}>
      <AddGroup label={t('Account edition')}>
        <p>
          {t('Both editions\' accounts can be saved at once, each with its own model catalog (the China and International editions are two providers that can be enabled and mapped separately).')}
        </p>
        <SegmentedControl
          aria-label={t('Account edition')}
          className={DIALOG_SEG_CLASS}
          options={[
            { value: 'cn', label: t('China edition (WorkBuddy)') },
            { value: 'intl', label: t('International edition (WorkBuddy AI)') },
          ]}
          value={edition}
          onValueChange={value => pickEdition(value as Edition)}
        />
      </AddGroup>

      <AddGroup label={t('Add method')}>
        <SegmentedControl
          aria-label={t('How to add a {label} account', { label: 'WorkBuddy' })}
          className={DIALOG_SEG_CLASS}
          options={[
            { value: 'web', label: t('Web login') },
            { value: 'file', label: t('Import credential file') },
            ...(platform() !== 'web'
              ? [{ value: 'desktop', label: t('Import desktop login state') }]
              : []),
          ]}
          value={effectiveMethod}
          onValueChange={value => setMethod(value as WorkBuddyMethod)}
        />
      </AddGroup>

      <AddGroup label={t('Web login')} hidden={effectiveMethod !== 'web'}>
        <p>{t('Added to the account list automatically after sign-in; the nickname comes from upstream.')}</p>
        {/* 网页端没有「内嵌窗口 / 系统浏览器」之分（桥接层一律开新标签页），
            整行收起 —— 连带那个只在「内嵌窗口」下生效的 Google / GitHub 开关 */}
        {webPanel ? null : (
          <>
            <FieldGroup orientation='horizontal' className='add-login-setting-row'>
              <span className='text-[12.5px] whitespace-nowrap text-subtle'>{t('Open with')}</span>
              <SegmentedControl
                aria-label={t('How to open web login')}
                className={DIALOG_SEG_CLASS}
                options={[
                  { value: 'embedded', label: t('Embedded window') },
                  { value: 'external', label: t('System default browser') },
                ]}
                value={loginMode}
                onValueChange={pickLoginMode}
              />
            </FieldGroup>
            {socialUsable ? (
              <Label className='add-login-setting-row' title={t('The International edition\'s sign-in page shows only email sign-in by default; checking this restores the Google / GitHub / X options')}>
                <span className='text-[12.5px] text-subtle'>{t('Restore Google / GitHub options')}</span>
                <Switch
                  checked={socialRestore}
                  onCheckedChange={next => {
                    setSocialRestore(next)
                    loginPrefs.socialRestore = next
                  }}
                />
              </Label>
            ) : null}
            <p>{t('The system browser can reuse an existing login state (recommended for the International edition); the embedded window is cleaner.')}</p>
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
            {t('Cancel sign-in')}
          </Button>
        </AddFoot>
      </AddGroup>

      <WorkBuddyFileImportSection edition={edition} visible={active && effectiveMethod === 'file'} footerHost={footerHost} />
      <WorkBuddyDesktopImportSection edition={edition} visible={active && effectiveMethod === 'desktop'} footerHost={footerHost} />
    </div>
  )
}
