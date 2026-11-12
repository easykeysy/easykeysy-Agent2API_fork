/**
 * Agent2API · 「 / 」 config（，none JSX）。
 *
 * and old ：add-provider-forms.js ADD_FORMS、 and 
 * add-qoder.js / add-cline.js / add-accio.js / add-zcode.js ****config
 * （ is file、 to window up ， add-provider-forms.js ——
 * to up after 1 in 1 ， not window，）。
 *
 * 、、limit、 and then ： this is can up has ，
 * is 。WorkBuddy not in this —— （ +
 * + ）， 1 ， add-provider-blocks.tsx。
 */

import { shared } from './add-account-bridge'
import { DOUBAOWORK } from './add-doubaowork-config'
import { DEEPSEEK_WEB } from './add-provider-deepseek'
import { MONKEYCODE, MONKEYCODE_INTL } from './add-provider-monkeycode'
import { TABBIT, TABBIT_INTL } from './add-provider-tabbit'
import { t } from '../i18n'

/* ─── ───────────────────────────────── */

/** 1 ：key for request（inputKey DOM id in ） */
export type FieldSpec = {
  key: string
  inputKey?: string
  label: string
  optional?: boolean
  rows?: number
  placeholder: string
  /**
   * is 1 JSON（CodeArts is this to use up ）， when ** and 
   * request**、 that not 。Why in before not is to after ：
   * `POST /api/accounts` is （`Credential::from_payload`
   * 「」 and 「」， not 「 1 JSON」），
   * after 1 = to all use path 1 has 1 to 。
   */
  jsonExpand?: boolean
}

export type RegionOption = { value: string; label: string }
export type MethodMode = { value: string; label: string; hint?: string }

export type WebLoginConfig = {
  /** （， and old noteOf ） */
  noteHtml?: string
  button: string
  hint?: string
  busyText?: string
  /**
   * has this 1 when 1 「」（ no has to ）。
   *
   * in ：（`platform() === 'web'`） no has 「」 and 
   * 「」 —— startLogin down mode after and not will use ，
   * 1 `window.open` new （ web_shim.rs ）。
   * ， by `webHint` 1 that 。
   */
  modes?: MethodMode[]
  /** that 1 （ use use 「 will in new in 」） */
  webHint?: string
  /** Cookie need need ， not can 。 */
  desktopOnly?: boolean
  /** this 1 in available（current no has use ， up ） */
  region?: string
  /** （ to start_login 1 parameter） */
  edition?: string
}

export type SmsLoginConfig = { noteHtml?: string }

export type OauthLoginConfig = {
  title?: string
  noteHtml?: string
  hint?: string
  /** WebLoginConfig.modes：， use webHint */
  modes?: MethodMode[]
  webHint?: string
}

/** 1 （ no has webHint use ） */
export const WEB_LOGIN_HINT = t('将在新标签页中打开授权页；完成后自动加入账号列表。')

/**
 * in / that ： when 
 * in 、 not to ， need need use 。
 */
export const WEB_LOGIN_PASTE_HINT = t('将在新标签页中打开授权页；完成后自动加入账号列表。若页面跳回本机地址后账号没有出现，按提示把地址栏里的完整回调地址粘贴回来即可。')

export type ProviderConfig = {
  provider: string
  label: string
  /** by （「 X 」） */
  addButton?: string
  manualTitle?: string
  manualNote?: string
  manualNoteHtml?: string
  fields: FieldSpec[]
  webLogin?: WebLoginConfig
  smsLogin?: SmsLoginConfig
  oauthLogin?: OauthLoginConfig
  /**
   * 「」—— ** 1 **： 1 ， not need need 。
   *
   * has has this 1 （ after `guest/access` empty body can ，
   * `providers::chatglm` module header）。 in up is 1 ** no has ** ：
   * 1 + 1 by ， down （optional）。
   * up not is 「」， is 「 to 1 can use 」—— is no has 
   * （ / to for ），must this 。
   */
  guestLogin?: { noteHtml: string; button: string; busyText?: string; namePlaceholder?: string }
  regionOptions?: RegionOption[]
  /**
   * this 1 / need not need 「」。
   *
   * ── Why need need this （ not is ）──────────────────────────
   * has up in **must can ** need need —— has 
   * Antigravity（Google）。 its can to ， 1 will use as 
   * 「 not use not 」。 after ： `options.proxy`
   * to `login.start`， `POST /api/accounts` 
   * `proxy` （ by in up use ）。
   */
  loginProxy?: boolean
  /** false = this 1 no has 「 from 」 this 1 */
  desktop?: boolean
  desktopNote?: string
  desktopHint?: string
  /** is Electron safeStorage 、 need DPAPI（only Windows） */
  desktopWindowsOnly?: boolean
  /**
   * 「file」 this 1 config（ has has that ）。
   *
   * and `desktop` （ use ， after `api::accounts` file）：
   * `desktop` is 1 「 when ** when **path」 ； this is use 
   * **** file after —— to can ，file can / 。
   */
  fileImport?: {
    /** file accept（empty = not ：KukuAI Cookies file no has ） */
    accept: string
    /** ： file、 in 、 after in （， not ） */
    note: string
    /** true = file， before base64 `authFileB64`；otherwise `authFileContent` */
    binary?: boolean
  }
}

/* ─── limit（ and after for ， before 1 ）────────────── */

/** limit（ and after truncate_chars(name, 100) 1 ） */
export const MAX_NAME_LENGTH = 100
/** uid / deviceId limit（ and after MAX_USER_UID_LENGTH / MAX_IDENTITY_LENGTH 1 ） */
export const MAX_IDENTITY_LENGTH = 256
/** limit： by 100， its by 256 */
export const maxLengthOf = (field: FieldSpec): number =>
  (field.key === 'name' ? MAX_NAME_LENGTH : MAX_IDENTITY_LENGTH)

/* ─── config ─────────────────────────────── */

const JSON_CREDENTIAL_FILE: NonNullable<ProviderConfig['fileImport']> = {
  accept: '.json,application/json',
  note: t('选择包含该提供商凭证的 JSON 文件，或本软件导出的单账号 JSON 文件。导入后凭证保存在网关，可独立使用。'),
}

const RACCOON: ProviderConfig = {
  // raccoon_accounts::add_raccoon_account：token / access_token、refreshToken / refresh_token、name
  provider: 'raccoon',
  label: '小浣熊',
  addButton: t('添加小浣熊账号'),
  // （ after providers::raccoon::oauth）： + 1 。
  // use ； / Docker will to 
  // redirect ， before is 127.0.0.1/localhost（http + ）
  // —— up ， not when before will down 
  // （ web_shim raccoonCallbackOriginProblem）。
  webLogin: {
    // in ； no has ， 1 in new ，
    // this not 「」（otherwise to is 1 not ）
    noteHtml: t('在打开的官方登录页里完成登录，成功后自动加入账号列表。'),
    button: t('打开网页登录'),
    hint: t('内嵌窗口打开；完成后自动加入列表，关窗即取消等待'),
    busyText: t('等待小浣熊登录完成…'),
    // is redirect （ up ）， in up can 
    // ， no has that 1 
    webHint: WEB_LOGIN_HINT,
  },
  manualTitle: t('粘贴 token / refreshToken'),
  manualNoteHtml: t('refreshToken 可选，填了到期可自动续期；两者都能从<a href="#" class="raccoon-hint-link" data-raccoon-hint>小浣熊客户端登录态文件</a>里取到。'),
  fields: [
    { key: 'name', label: t('备注名'), optional: true, placeholder: t('可选，留空则用凭证里的账号名') },
    { key: 'token', label: 'token', rows: 3, placeholder: t('粘贴 access_token（一长串 JWT）') },
    { key: 'refreshToken', inputKey: 'refresh', label: 'refreshToken', rows: 2, optional: true, placeholder: t('可选') },
  ],
  desktopNote: t('读本机客户端当前登录态，每次实时读取（删掉这条记录不影响客户端登录态）。客户端重新登录后点「刷新 Token」同步。'),
  // 「file」：client is JSON， before authFileContent。
  // and up desktop that and is —— 1 is 「 when path」，
  // 1 is 「 use file after 」（file can / ）。
  fileImport: {
    accept: '.json,application/json',
    note: t('选择小浣熊客户端的登录态文件 auth.json（Windows：C:\\Users\\<你的用户名>\\.box-agent\\config\\auth.json；macOS / Linux：~/.box-agent/config/auth.json）。导入后凭证直接落在网关、可独立转发，不再依赖本机客户端。'),
  },
}

const CATPAW: ProviderConfig = {
  // catpaw_accounts::add_catpaw_account：token / accessToken / access_token / auth_token
  // （ X-Passport-Token）、uid / userId / loginName、name； no has new 
  provider: 'catpaw',
  label: 'CatPaw',
  // ： passport + loopback ；server when poll-token，
  // Docker / not loopback 。
  webLogin: {
    noteHtml: t('用 CatPaw 账号完成登录（美团 passport），成功后自动加入账号列表。'),
    button: t('打开 CatPaw 网页登录'),
    busyText: t('等待 CatPaw 登录完成…'),
    modes: [
      { value: 'embedded', label: t('内嵌窗口（推荐）'), hint: t('内嵌窗口打开；完成后自动加入列表，关窗即取消等待') },
      { value: 'external', label: t('系统浏览器'), hint: t('系统浏览器打开（复用已登录的美团账号）；完成后自动加入列表') },
    ],
  },
  manualNote: t('token 是登录态 Cookie；CatPaw 没有刷新机制，过期后需在客户端重新登录。'),
  fields: [
    { key: 'token', label: 'token', rows: 3, placeholder: t('CatPaw 的 X-Passport-Token（登录态 Cookie）') },
    { key: 'uid', label: 'uid', placeholder: t('必填，CatPaw 账号标识') },
    { key: 'name', label: t('备注名'), optional: true, placeholder: t('可选，留空则用登录名或 uid') },
  ],
  desktopNote: t('读本机客户端当前登录态，每次实时读取（删掉这条记录不影响客户端登录态）。客户端重新登录后重新导入即可。'),
  desktopHint: t('读取 ~/.meituan-catpaw/auth.json，需已在 CatPaw 客户端登录'),
  fileImport: {
    accept: '.json,application/json',
    note: t('选择 CatPaw 客户端的登录态文件 auth.json（Windows：C:\\Users\\<你的用户名>\\.meituan-catpaw\\auth.json；macOS / Linux：~/.meituan-catpaw/auth.json）。导入后凭证直接落在网关、可独立转发，不再依赖本机客户端。'),
  },
}

/**
 * AutoClaw 1 （ and Cline 1 ）：`autoclaw`（，id not 
 * —— ） and `autoclaw-intl`（）。。
 *
 * ：（ after `autoclaw::region`， before not ）； to 
 * （auth.json use 、 no has ， by use in 1 down ）；
 * ** not ** —— has ， has Zai / Google OAuth。
 */
const AUTOCLAW: ProviderConfig = {
  provider: 'autoclaw',
  label: t('AutoClaw 国内版'),
  fileImport: JSON_CREDENTIAL_FILE,
  manualNote: t('token 支持 enc: 前缀（Windows 上自动解密）。未填 refreshToken 无法自动续期。'),
  // ** 1 ** （ not OAuth by ， already for ）
  smsLogin: {
    noteHtml: t('用绑定的手机号登录：点「获取验证码」后填入即可。这是国内版官方唯一的登录方式。'),
  },
  fields: [
    { key: 'token', label: 'token', rows: 3, placeholder: t('明文 JWT 或 auth.json 里的 enc: 加密值（自动解密）') },
    { key: 'refreshToken', label: 'refreshToken', rows: 2, optional: true, placeholder: t('没有则无法自动续期') },
    { key: 'deviceId', label: 'deviceId', optional: true, placeholder: t('可选，续期时带上') },
    { key: 'name', label: t('备注名'), optional: true, placeholder: t('可选，留空则用 userId') },
  ],
  desktopNote: t('读本机客户端当前登录态，每次实时读取（删掉这条记录不影响客户端登录态）。'),
  desktopHint: t('读取 %APPDATA%/AutoClaw/auth.json 并解密，仅 Windows'),
  // is Electron safeStorage ， need DPAPI（only Windows），
  // macOS up （ by desktopImportAvailable）
  desktopWindowsOnly: true,
}

const AUTOCLAW_INTL: ProviderConfig = {
  // AutoClaw （autoglm-api.autoglm.ai）： and 1 and 
  // （appId/appKey ， already ）， has not 。
  provider: 'autoclaw-intl',
  label: t('AutoClaw 国际版'),
  fileImport: JSON_CREDENTIAL_FILE,
  manualNote: t('国际版与国内版账号体系独立，请填国际版的凭证。未填 refreshToken 无法自动续期。'),
  // OAuth ：** 1 ** （ Zai / Google by ）。
  // and ： before has 1 （），
  // must in can ， by after will in 
  // （ ui/autoclaw-oauth.js file）。
  oauthLogin: {
    title: t('网页登录（Zai / Google）'),
    noteHtml: t('点按钮后先过一次滑块验证（官方风控），随后打开登录页，登录完成即自动添加账号。<br>这是国际版官方唯一的登录方式；已在客户端登录过的，用「导入桌面端登录态」更快。'),
    modes: [
      { value: 'embedded', label: t('内嵌窗口（推荐）'), hint: t('内嵌窗口打开；关窗即取消等待（Google 账号被拒时改用系统浏览器）') },
      { value: 'external', label: t('系统浏览器'), hint: t('系统浏览器打开（复用已登录的 Zai / Google 账号）；完成后自动加入列表') },
    ],
    // by web_shim runLoginFlow new ； this server
    // （ not is ）， not need need 「」 that 
    webHint: WEB_LOGIN_HINT,
  },
  fields: [
    { key: 'token', label: 'token', rows: 3, placeholder: t('明文 JWT（国际版账号的 access token）') },
    { key: 'refreshToken', label: 'refreshToken', rows: 2, optional: true, placeholder: t('没有则无法自动续期') },
    { key: 'deviceId', label: 'deviceId', optional: true, placeholder: t('可选，续期时带上') },
    { key: 'name', label: t('备注名'), optional: true, placeholder: t('可选，留空则用 userId') },
  ],
  // to ：auth.json no has 、 not ，
  // use in 1 down to 1 ， after is can up 401。
  desktopNote: t('读本机客户端当前登录态，每次实时读取（删掉这条记录不影响客户端登录态）。用 Zai / Google 登录客户端的用户走这一条。'),
  desktopHint: t('读取 %APPDATA%/AutoClaw/auth.json 并解密，仅 Windows'),
  desktopWindowsOnly: true,
}

/**
 * Qoder （ in / ）： 1 ， has not 。
 *
 * Why not is 1 config 1 「」 down （，2026-10）： and AutoClaw / Accio /
 * ZCode 1 —— is **provider ** not is 。 before 
 * in 1 「」 down 、 1 「get」 can to directory；
 * by after has 、 and （ id / / ）。
 */
function qoderForm(spec: { provider: string; label: string; siteNote: string }): ProviderConfig {
  const { provider, label, siteNote } = spec
  return {
    provider,
    label,
    desktop: false,
    // in provider （ after by provider id ）， not 
    webLogin: {
      // is 1 PKCE ， has not ；edition by provider
      // （ after by provider id ，request edition is ）
      noteHtml: t('打开官方授权页完成设备码授权，登录的是{label}（{site}）的账号。', { label, site: siteNote }),
      button: t('打开 Qoder 网页登录'),
      busyText: t('等待 Qoder 授权完成…'),
      edition: provider === 'qoder' ? 'cn' : 'global',
      modes: [
        { value: 'embedded', label: t('内嵌窗口（推荐）'), hint: t('内嵌窗口用全新环境，多账号互不影响；关窗即取消等待') },
        { value: 'external', label: t('系统浏览器'), hint: t('系统浏览器打开（复用已登录账号）；完成后自动加入列表') },
      ],
    },
    manualTitle: t('使用个人访问令牌（PAT）'),
    manualNote: t('在{label}的 Qoder 账号设置 → Integrations 生成 PAT（别填 Google / GitHub 的令牌）。两个地区的账号与凭证不通用。', { label }),
    fields: [
      { key: 'pat', label: 'Qoder PAT', rows: 3, placeholder: t('粘贴 Qoder 个人访问令牌（pt-…）') },
      { key: 'name', label: t('备注名'), optional: true, placeholder: t('可选，留空使用账号昵称或邮箱') },
    ],
  }
}

const QODER = qoderForm({ provider: 'qoder', label: t('中国版'), siteNote: 'qoder.com.cn' })
const QODER_INTL = qoderForm({ provider: 'qoder-intl', label: t('国际版'), siteNote: 'qoder.com' })

/**
 * （ `qwenwork` / `qwenwork-intl`）。
 *
 * ── 2026-11-03 1 （ `providers/qoder/authorize.rs`）──
 * old is ****， up already this ： that 
 * for request 400。 in ** + **（ CLI / IDE that ）：
 * ， after in ，
 * 。available； up ， not to ，
 * up 「 when none」， not is to 1 by 。
 *
 * ** has this 1 **： no has （PAT），
 * after `Site::supports_pat` as ， —— up to 1 
 * not to ， this not `manualTitle` / `fields`
 * （Qoder that 1 has PAT ）。
 */
function qwenworkForm(spec: {
  provider: string
  label: string
  siteNote: string
  loginable: boolean
}): ProviderConfig {
  const { provider, label, siteNote, loginable } = spec
  return {
    provider,
    label,
    desktop: false,
    // in provider （ after by provider id ）
    webLogin: {
      noteHtml: loginable
        ? t('在打开的页面里登录千问办公{label}（{site}）的账号，登录完成后会自动加入列表。两版的账号与凭证不通用。', { label, site: siteNote })
        : t('千问办公{label}暂时无法登录：上游只接受它自己的官方回调地址，不论证码到本机。', { label }),
      button: loginable ? t('打开千问办公网页登录') : t('暂时无法登录'),
      busyText: t('等待千问办公登录完成…'),
      edition: provider === 'qwenwork' ? 'cn' : 'global',
      modes: [
        { value: 'embedded', label: t('内嵌窗口（推荐）'), hint: t('内嵌窗口用全新环境，多账号互不影响；关窗即取消等待') },
        { value: 'external', label: t('系统浏览器'), hint: t('系统浏览器打开（复用已登录账号）；完成后自动加入列表') },
      ],
    },
    fields: [
      { key: 'name', label: t('备注名'), optional: true, placeholder: t('可选，留空使用账号名或邮箱') },
    ],
  }
}

const QWENWORK = qwenworkForm({
  provider: 'qwenwork',
  label: t('国内版'),
  siteNote: 'qwenwork.cn',
  loginable: true,
})
const QWENWORK_INTL = qwenworkForm({
  provider: 'qwenwork-intl',
  label: t('国际版'),
  siteNote: 'qwenwork.ai',
  loginable: false,
})

/**
 * Cline is （Cline Free / Cline Pass）。
 *
 * Why not is 1 config 1 「」 down ： up before 
 * （`cline-free/…` / `cline-pass/…`）， is ** **， in 1 will 
 * 「 can use 」 in up not 、 1 then 。
 * by after 1 、 has 1 and 。 id / / 。
 */
function clineForm(spec: { provider: string; label: string; poolNote: string }): ProviderConfig {
  const { provider, label, poolNote } = spec
  return {
    provider,
    label,
    // not ， can （ need use in 1 ）
    desktop: true,
    webLogin: {
      // （WorkOS RFC 8628）： no has 、 no has ，
      // is 「 → use → to 」， can 。
      noteHtml: t('打开授权页完成确认，账号自动加入 <b>{label}</b>{note}', { label, note: poolNote }),
      button: t('打开 Cline 授权页'),
      busyText: t('等待 Cline 授权确认…'),
      modes: [
        { value: 'embedded', label: t('内嵌窗口（推荐）'), hint: t('内嵌窗口打开；完成后自动加入列表，关窗即取消等待') },
        { value: 'external', label: t('系统浏览器'), hint: t('系统浏览器打开（复用已登录的 Cline 账号）；完成后自动加入列表') },
      ],
    },
    manualTitle: t('填写凭证'),
    manualNote: t('refreshToken 可选，填了可自动续期；两者都能从客户端登录态文件取到。同一个 Cline 账号两个池都能用，这里归入 {label}。', { label }),
    fields: [
      { key: 'accessToken', label: 'accessToken', rows: 3, placeholder: t('粘贴 workos:… 开头的令牌（不带前缀也会自动补上）') },
      { key: 'refreshToken', label: 'refreshToken', rows: 2, optional: true, placeholder: t('可选，没有则无法自动续期') },
      { key: 'name', label: t('备注名'), optional: true, placeholder: t('可选，留空使用账号姓名、邮箱或令牌指纹') },
    ],
    desktopNote: t('读本机客户端当前登录态，每次实时读取（删掉这条记录不影响客户端登录态）。这一条归入 {label}。', { label }),
    desktopHint: t('读取 ~/.cline/data/settings/providers.json，需已在 Cline 客户端登录'),
    // 「file」：providers.json is **set**file（ JSON）， before 
    // authFileContent。 and desktop that 1 ： in 1 down 。
    fileImport: {
      accept: '.json,application/json',
      note: t('选择 Cline 客户端的登录态文件 providers.json（Windows：C:\\Users\\<你的用户名>\\.cline\\data\\settings\\providers.json；macOS / Linux：~/.cline/data/settings/providers.json）。导入后凭证直接落在网关、可独立转发，不再依赖本机客户端；这一条归入 {label}。', { label }),
    },
  }
}

/**
 * Accio （ / ）： 1 、 1 ， has and 
 * x-package-region not 。 not is is ，config。
 */
function accioForm(spec: { provider: string; label: string; site: string; siteNote: string }): ProviderConfig {
  const { provider, label, site, siteNote } = spec
  return {
    provider,
    label,
    // OAuth 2.0 + PKCE： loopback；Docker / 
    // by shim （ and CodeArts 1 ）。
    webLogin: {
      noteHtml: t('打开 Accio <b>{label}</b>的官方登录页（<code>{site}</code>）并用你的 Accio 账号登录：登录成功后官方页面会跳回本机；同机自动完成，Docker / 远程面板按提示粘贴回调地址即可（授权码只在本机传给网关，界面不显示明文 token）。', { label, site }),
      button: t('打开 Accio {label}登录页', { label }),
      busyText: t('等待 Accio {label}登录完成…', { label }),
      modes: [
        { value: 'embedded', label: t('内嵌窗口（推荐）'), hint: t('将打开内嵌窗口；登录完成后自动加入账号列表。关掉窗口即取消等待') },
        { value: 'external', label: t('系统浏览器'), hint: t('将用系统默认浏览器打开登录页（会复用浏览器里已登录的 Accio 账号）；完成登录后自动加入账号列表，关掉弹窗即取消等待') },
      ],
      webHint: WEB_LOGIN_PASTE_HINT,
    },
    manualTitle: t('填写凭证'),
    manualNoteHtml: t('accessToken 是 Accio 的登录凭证（一长串不透明 token，不是 JWT）；refreshToken 可选，填了之后到期能自动续期。请填写 <b>{label}</b>账号的凭证 —— {siteNote}（最容易拿到的办法：直接用上方的「网页登录」，不需要手工找 token。）', { label, siteNote }),
    fields: [
      { key: 'accessToken', label: 'accessToken', rows: 3, placeholder: t('粘贴 Accio 的 accessToken') },
      { key: 'refreshToken', label: 'refreshToken', rows: 2, optional: true, placeholder: t('可选，没有则无法自动续期') },
      { key: 'name', label: t('备注名'), optional: true, placeholder: t('可选，留空使用账号昵称或邮箱') },
    ],
  }
}

/**
 * ZCode （ / ）： 1 zcode （ / in 
 * zcode.z.ai）， has not ，config。
 */
function zcodeForm(spec: { provider: string; label: string; site: string; planNote: string }): ProviderConfig {
  const { provider, label, site, planNote } = spec
  return {
    provider,
    label,
    fileImport: JSON_CREDENTIAL_FILE,
    // no has 「 when 」 can ： in ，
    // no has auth.json that can （ and Accio 1 ）
    desktop: false,
    webLogin: {
      // by **server** to and not ： to 1 ， use in 
      // after by ZCode server， in after 。
      // （ and Accio ），「 after none zcode:// 」 is 。
      noteHtml: t('打开 ZCode <b>{label}</b>的官方授权页并用你的账号登录。授权结果由 ZCode 服务端记录，网关在后台自动取回凭证并加入账号列表。', { label }),
      button: t('打开 ZCode {label}授权页', { label }),
      busyText: t('等待 ZCode {label}授权完成…', { label }),
      modes: [
        { value: 'embedded', label: t('内嵌窗口（推荐）'), hint: t('将打开内嵌窗口；授权完成后自动加入账号列表。关掉窗口即取消等待。授权页最后可能提示无法打开 zcode:// 链接，这是正常的 —— 结果已由服务端记下') },
        { value: 'external', label: t('系统浏览器'), hint: t('将用系统默认浏览器打开授权页（会复用浏览器里已登录的 ZCode 账号）；授权完成后自动加入账号列表。页面最后可能提示无法打开 zcode:// 链接，属正常现象') },
      ],
    },
    manualTitle: t('填写凭证'),
    manualNoteHtml: t('本家有两个**互不替代**的凭证，按你要用的功能填，至少填一个：<b>编码套餐 API Key</b> 用于转发推理（打 <code>{site}</code>）与查询窗口限额（每 N 小时 / 每周的剩余比例）；<b>jwt</b> 用于领取套餐与查询余额（打 <code>zcode.z.ai</code> 的 billing 网关，官方叫 Coding Plan JWT，是一串三段点分的字符串）。余额两把都能查、但读到的东西不同：jwt 读到余额桶与套餐到期，API Key 读到窗口限额；而领取只认 jwt、转发只认 API Key。请填写 <b>{label}</b>账号的凭证 —— {plan}（最容易拿到的办法：直接用上方的「网页登录」—— 它会替你把这个 API Key 换好，两个凭证一起拿到。）', { site, label, plan: planNote }),
    fields: [
      {
        key: 'accessToken',
        label: t('编码套餐 API Key'),
        rows: 3,
        optional: true,
        // this not can OAuth ： that will up by 「OAuth 」 that 
        // and 401。client and 「」 this is API Key use 。
        placeholder: t('用于转发；形如 apiKey.secret 两段点分（不填则这个账号不能转发）'),
      },
      { key: 'jwt', label: 'Coding Plan JWT', inputKey: 'jwt', rows: 3, optional: true, placeholder: t('用于领取套餐与查询余额；不填则这个账号不能领取与查余额') },
      { key: 'userId', label: t('用户 ID'), optional: true, placeholder: t('可选；用于生成账号 id 与展示名') },
      { key: 'name', label: t('备注名'), optional: true, placeholder: t('可选，留空自动生成') },
    ],
  }
}

/**
 * CodeArts（ as AI / snap-access）。
 *
 * ── （OAuth + PKCE）────────────────────────
 * by （`providers::codearts::oauth::authorize_url`，parameter and 
 * for to ），portal after ** ** up 
 * `http://127.0.0.1:<port>/oauth/callback` —— path not is can ：portal 
 * to `port`。 and in 1 when （ secret+redirect、
 * code） can ； not in 1 when is 127.0.0.1， to not 。
 * that （ in NAS / up ） use in down ，：
 * · Docker / in shim —— `code` ；
 * · `secret` when will ticket ， is ** not to refresh token**
 * （ 1 when after need new 1 ）， up this is 。
 *
 * ── Why is 1 、 and has 1 JSON ──────────────────
 * CodeArts is 1 OAuth/PKCE ** when **（AK/SK/STS，
 * 1 when to ）， need ** that PKCE verifier and DPoP **，
 * 1 to not 。 use up this ** is 1 JSON**
 * （ `codearts_provider_credential`、CLIProxyAPI auth file in 
 * ）： will 「 1 」 will ，`oauth_context` is 
 * for 、 not down 。 1 ， by `jsonExpand` after 。
 */
const CODEARTS: ProviderConfig = {
  provider: 'codearts',
  label: 'CodeArts',
  // ** no has **「client」 this after path（ can or ）。
  // not ， will 1 after no has ：chip 
  // `desktop !== false`， down desktopNote returnempty。
  // not to this （platform()==='web' when ）， has App will 。
  desktop: false,
  webLogin: {
    noteHtml: t('打开华为云 CodeArts 的官方授权页登录：登录完成后官方页面会把浏览器带回<b>网关自己的</b> <code>/oauth/callback</code>，网关用一次性授权码换取临时凭据并加入账号列表。<br>网关跑在另一台机器上时，网页端会提示把地址栏中的最终回调地址直接粘贴回面板。'),
    button: t('打开 CodeArts 授权页'),
    busyText: t('等待 CodeArts 登录完成…'),
    modes: [
      {
        value: 'embedded',
        label: t('内嵌窗口（推荐）'),
        hint: t('将打开内嵌窗口；登录完成后自动加入账号列表。关掉窗口即取消等待'),
      },
      {
        value: 'external',
        label: t('系统浏览器'),
        hint: t('将用系统默认浏览器打开授权页（会复用浏览器里已登录的华为云账号）；浏览器与网关不在同一台机器时，按上方说明把最终回调地址粘贴回面板'),
      },
    ],
    webHint: WEB_LOGIN_PASTE_HINT,
  },
  manualTitle: t('粘贴登录凭据'),
  manualNoteHtml: t('整份粘贴官方插件 / CLIProxyAPI 落盘的凭据 JSON（形如 <code>{"codearts_provider_credential":{…}}</code>，铺平的也行）。<br>必填：<code>access_key_id</code>、<code>secret_access_key</code>、<code>security_token</code>。<b>要能自动续期，必须连 <code>refresh_token</code> 与 <code>oauth_context</code> 一起粘</b> —— 临时凭据约一小时到期，缺这半块就续不回来，只能重新登录。'),
  fields: [
    {
      key: 'credentialJson',
      label: t('凭据 JSON'),
      rows: 8,
      // after by request（ after `Credential::from_payload` /）
      jsonExpand: true,
      placeholder: '{"codearts_provider_credential":{"access_key_id":"HSTA…","secret_access_key":"…",'
        + '"security_token":"…","expires_at":"2026-09-27T16:17:00.327Z","domain_id":"…","user_id":"…",'
        + '"user_name":"…","refresh_token":"eyJ…","oauth_context":{…}}}',
    },
    { key: 'name', label: t('备注名'), optional: true, placeholder: t('可选，留空使用凭据里的 user_name') },
  ],
}

/**
 * Trae（，Agent / IDE by directory）。
 *
 * ── Why has 1 、 no has 「」 down ──────────────────────
 * Agent / IDE use trae.cn ， cn / solo 。
 * need provider id and directory， not can use ； not will 。
 *
 * ── Whymust in ────────────────────────────────
 * up for is ** then ** （`^http://127.0.0.1:<port>/authorize$`），
 * and ** no has state** —— 1 can 。must and ；
 * / not to ， can in has up after that 。
 */
const TRAE: ProviderConfig = {
  provider: 'trae',
  label: 'Trae',
  // Trae in （ and Accio / ZCode 1 ），
  // no has auth.json that can —— to will failure。
  desktop: false,
  webLogin: {
    noteHtml: t('打开 <b>Trae</b> 的官方授权页（<code>trae.cn</code>）并用你的 Trae 账号登录：授权完成后官方页面会跳回<b>本机</b>的一个临时端口；同机部署自动接收，Docker / 远程面板请按提示复制地址栏回调地址。'),
    button: t('打开 Trae 授权页'),
    busyText: t('等待 Trae 授权完成…'),
    modes: [
      {
        value: 'embedded',
        label: t('内嵌窗口（推荐）'),
        hint: t('将打开内嵌窗口；授权完成后自动加入账号列表。远程面板可粘贴回调地址。关掉窗口即取消等待。链接 5 分钟内有效，超时或未点就会作废（可重新发起）'),
      },
      {
        value: 'external',
        label: t('系统浏览器'),
        hint: t('将用系统默认浏览器打开授权页（会复用浏览器里已登录的 Trae 账号）；同机完成后自动加入，Docker / 远程面板按提示粘贴地址栏回调地址。'),
      },
    ],
    // in （ up 127.0.0.1）， and not when 
    // not to —— must「 can can need 」
    webHint: WEB_LOGIN_PASTE_HINT,
  },
  manualTitle: t('填写凭证'),
  manualNoteHtml: t('accessToken 是 Trae 的 <code>Cloud-IDE-JWT</code>（三段点分），refreshToken 用于到期自动续期（本家 <b>refreshToken 一次一换</b>：换发一次旧的即作废，所以两份程序别同时刷同一个账号）。<br>手工粘贴时请连 <b>machineId / deviceId</b> 一起填：上游把它们与登录时上传的设备公钥绑在一起判设备，凭空换一对会撞 <code>2xxxx</code> 那族设备绑定拒绝。<br>没有这三样时的正路是用上方的「网页登录」；容器 / 远程部署形态按提示粘贴地址栏回调地址即可。'),
  fields: [
    { key: 'accessToken', label: 'accessToken', rows: 3, placeholder: t('Cloud-IDE-JWT（三段点分）') },
    { key: 'refreshToken', label: 'refreshToken', rows: 2, optional: true, placeholder: t('可选；填了才能到期自动续期') },
    { key: 'uid', label: t('用户 ID'), optional: true, placeholder: t('可选；用于去重与展示名（留空时网关会问一次上游）') },
    { key: 'machineId', label: 'machineId', optional: true, placeholder: t('可选；UUID 形态，与凭据同生共死') },
    { key: 'deviceId', label: 'deviceId', optional: true, placeholder: t('可选；纯数字串（上游只收 8–24 位数字）') },
    { key: 'name', label: t('备注名'), optional: true, placeholder: t('可选，留空自动用昵称') },
  ],
}

/**
 * Loomy（）： + session。
 *
 * ── and not （ has up ，）────────────────
 * 1. is ** 1 **（ `/api/session/login/loomy/sms/*`、 in 
 * `msgid` not is `deviceId`、request HMAC-SHA1 ）——
 * by `ui/sms-login.js` SMS_PROFILES and by provider ；
 * 2. is ** session**（ when is token， not is JWT）；
 * 3. ** no has **：session 14 ， can new （ not 
 * 「 new Token」 that 1 ）。
 */
const LOOMY: ProviderConfig = {
  provider: 'loomy',
  label: 'Loomy',
  fileImport: JSON_CREDENTIAL_FILE,
  smsLogin: {
    noteHtml: t('用 Loomy 账号绑定的手机号登录：点「获取验证码」后填入即可。这是本家最省事的入口。'),
  },
  manualNote: t('粘贴的是登录 session（同时是模型网关的 token）。Loomy 没有续期接口，session 有效期约 14 天，过期后重新登录即可。'),
  fields: [
    { key: 'session', label: 'session', rows: 3, placeholder: t('登录 session（一长串）') },
    { key: 'userId', label: 'userId', optional: true, placeholder: t('可选，讯飞侧 userid') },
    { key: 'phone', label: t('手机号'), optional: true, placeholder: t('可选，展示用') },
    { key: 'name', label: t('备注名'), optional: true, placeholder: t('可选，留空则用脱敏手机号或 userId') },
  ],
  // in client （ no has auth.json that can file），
  // not 「」（ and Accio / ZCode 1 ， by config）。
  desktop: false,
}

/**
 * KukuAI（「 AI / GenFlowPro」）： 1 1 provider， no has 。
 *
 * ── ────────────────────────────────────────────────
 * Cookie can ：`BDUSS` / `STOKEN` / `gfprotpl=genflowpro`（ after 1 
 * ， not use ）。supports（ and after `kuku::credentials` ）：
 * 1. Cookie （`BDUSS=…; STOKEN=…; …`）；
 * 2. Cookie `[{name, value}, …]` JSON ；
 * 3. `{BDUSS, STOKEN}` JSON for 。
 *
 * ── ──────────────────────────────────────────────
 * client is Electron ， in `%APPDATA%\baidugenflowpro\Network\Cookies`
 * （SQLite ****，none need ）。client when this file ，failure when 
 * client（ use and CatPaw 1 ）。
 *
 * ── no has new ────────────────────────────────────────────
 * BDUSS is ， up no has new ， after new / new can 
 * （ and CatPaw 1 ， not 「 new Token」 that 1 ）。
 */
const KUKU: ProviderConfig = {
  provider: 'kuku',
  label: 'KukuAI',
  // （）： kuku.baidu.com， use when use 1 
  // （ / by —— not in up 
  // ）。success after to Cookie BDUSS，** will **
  // （ HttpOnly BAIDUID_BFESS etc ），、。
  webLogin: {
    noteHtml: t('在打开的登录页里用百度账号登录 KukuAI 主站（手机验证码 / 扫码均可），登录成功后自动加入账号列表。网关会向百度换发 KukuAI 业务会话令牌（依赖本机已安装库库AI 客户端）。'),
    button: t('打开网页登录'),
    busyText: t('等待 KukuAI 登录完成…'),
    // this client Cookie （ no has ， web_shim.rs
    // ），「 already 」， not 
    webHint: WEB_LOGIN_HINT,
  },
  manualTitle: t('粘贴 Cookie'),
  manualNote: t('BDUSS 必填；请粘贴**完整 Cookie**（Cookie 编辑器一键导出即可）—— 其中 **PTOKEN 必须保留**，网关要用它向百度换发 KukuAI 业务会话令牌，缺了它模型刷新会报「未登录」。支持整段 Cookie 头、Cookie 编辑器导出的 JSON 数组，或 {BDUSS, STOKEN} 对象。KukuAI 没有刷新机制，登录态过期后重新粘贴即可。'),
  fields: [
    { key: 'cookie', label: 'Cookie', rows: 3, placeholder: t('BDUSS=…; STOKEN=…（或 Cookie 编辑器导出的 JSON）') },
    { key: 'name', label: t('备注名'), optional: true, placeholder: t('可选，留空则用账号 uk') },
  ],
  desktop: true,
  desktopHint: t('读取本机 KukuAI 客户端登录态（%APPDATA%\\baidugenflowpro\\Network\\Cookies），需已在 KukuAI 客户端登录'),
  desktopNote: t('读本机客户端当前登录态（明文 Cookie，无需解密）。客户端运行时 Cookies 文件被独占占用，导入前请先关闭 KukuAI 客户端。'),
  // 「file」：**** SQLite（file Cookies、 no has after ），
  // file base64 authFileB64； JSON 1 。
  // and desktop that ： is use in file ****（ when 
  // file）， not client in use that —— this none need client，
  // need this 1 ，otherwise use will as and up that 1 。
  fileImport: {
    binary: true,
    // filenone、filenone from by ：empty = not （ can ）
    accept: '',
    note: t('选择 KukuAI 客户端的 Cookies 文件（%APPDATA%\\baidugenflowpro\\Network\\Cookies）：它是**二进制** SQLite 数据库（无扩展名，不是 JSON）。导入时读的是你选中文件的副本、不和运行中的客户端抢占用，因此**无需先关闭客户端**。导入后凭证直接落在网关、可独立转发。'),
  },
}

/**
 * Command Code（`api.commandcode.ai`）： 1 1 provider， no has 
 * （ §9： has this 1 ， no has cn / intl 、 no has region ）。
 *
 * ── is 1 `user_` before API Key， ─────────────────
 * is CLI `~/.commandcode/auth.json` or studio
 * （commandcode.ai/studio）。（ key / for /
 * `Authorization: Bearer …` / key after ）， will 
 * `user_` key —— ， not use 。
 *
 * ── none、none ───────────────────────────────────
 * key is ： up no has refresh 、 not will ， can new get（
 * not 「 new Token」 that 1 ）。 `~/.commandcode/auth.json` is CLI
 * file， not is can 「client」， no has that path（ and Loomy /
 * MonkeyCode 1 ）。
 *
 * ── when 1 up ───────────────────────────────────
 * in when will 1 ****（`GET /alpha/billing/credits`）： has to 
 * 401/403 key invalid，failure / 5xx / unknown 1 —— failure not ，
 * （ not 1 up use in ）。
 */
const COMMANDCODE: ProviderConfig = {
  provider: 'commandcode',
  label: t('Command Code'),
  fileImport: JSON_CREDENTIAL_FILE,
  desktop: false,
  manualTitle: t('粘贴 API Key'),
  manualNote: t('粘贴 Command Code 的 API Key（user_ 开头；从官方 CLI 的 ~/.commandcode/auth.json 或网页 studio 获取）。整行 / 带引号 / 带 Bearer 前缀地粘都行，网关会自己剥成裸 key。key 不过期、也没有续期接口，失效后重新获取即可；添加时网关会做一次轻量探活，失败不影响添加。'),
  fields: [
    { key: 'apiKey', label: 'apiKey', rows: 3, placeholder: t('user_ 开头的 API Key（也可直接粘贴 Authorization: Bearer … 整行）') },
    { key: 'name', label: t('备注名'), optional: true, placeholder: t('可选，留空则用 key 尾号') },
  ],
}

/**
 * Antigravity（Google AI IDE， Gemini ）。
 *
 * ── （Google OAuth + loopback）───────────────
 * after （`accounts.google.com`，6 scope +
 * `access_token=offline` / `prompt=consent`）， after to ** **
 * loopback `/oauth-callback`（redirect_uri is ，
 * `providers::antigravity::oauth`）。 in ， and 
 * can use （ and CatPaw ）； / Docker down to not 
 * loopback when ， can URL 。
 *
 * ── Google refresh token（ ）──────────────
 * already from （ IDE / ） to refresh_token use can ；
 * `1//` ， `refresh_token=` （ will 1 ，
 * `1//` ）。
 *
 * ── has ─────────────────────────────────────────────
 * access_token 1 when （Google expires_in），refresh_token is 
 * ： and will in when 1 Google token ****
 * and to access token， after to by —— can new 
 * Google 。
 *
 * ── projectId can empty ─────────────────────────────────────
 * `projectId`（`cloudaicompanionProject`） is requestrequired、 Google 
 * not （ not in token ）：empty when will best-effort 
 * （`loadCodeAssist` → none then `onboardUser`），failure not ， when will 
 * 。`email` optional，used for and （ will userinfo）。
 *
 * ── no has 「」────────────────────────────────
 * in Antigravity IDE ， no has `auth.json` that can 
 * （ and Accio / ZCode / Loomy 1 ）。
 */
const ANTIGRAVITY: ProviderConfig = {
  provider: 'antigravity',
  label: t('Antigravity'),
  desktop: false,
  // up is Google： has can 。none up no has ，
  // 「 / 」 can by use in this （ `loginProxy` ）。
  loginProxy: true,
  // ：Google + loopback 。 in ，
  // available（ will use already Google ）。
  webLogin: {
    noteHtml: t('在打开的登录页里登录 Google 账号（Antigravity 的授权页），授权完成后自动加入账号列表。'),
    button: t('打开 Antigravity 网页登录'),
    busyText: t('等待 Google 授权完成…'),
    modes: [
      { value: 'embedded', label: t('内嵌窗口（推荐）'), hint: t('内嵌窗口打开；完成后自动加入列表，关窗即取消等待') },
      { value: 'external', label: t('系统浏览器'), hint: t('系统浏览器打开（复用已登录的 Google 账号）；完成后自动加入列表') },
    ],
  },
  manualTitle: t('粘贴 Google refresh token'),
  manualNoteHtml: t('粘贴 Antigravity 的 Google refresh token（以 <code>1//</code> 开头；直接粘贴含 <code>refresh_token=</code> 的整行也行）。<br>这是网页登录之外的手工入口（例如从已登录的官方 IDE 里导出）。refresh token 不会过期，access token 到期由网关自动续期；令牌丢了只能重新走 Google 授权，请妥善保存。'),
  fields: [
    { key: 'refreshToken', label: 'refreshToken', rows: 2, placeholder: t('1// 开头的 Google refresh token（也可直接粘贴 refresh_token=… 整行）') },
    { key: 'projectId', label: 'projectId', optional: true, placeholder: t('可选（cloudaicompanionProject）：留空由网关自动发现') },
    { key: 'email', label: 'email', optional: true, placeholder: t('可选：账号邮箱，用于展示与身份识别（换新令牌也能认回同一账号）') },
    { key: 'name', label: t('备注名'), optional: true, placeholder: t('可选，留空则用邮箱或令牌尾号') },
  ],
}

/**
 * OfficeAce（ as / OfficeClaw）： 1 1 provider， no has 。
 *
 * ── （ in after `add_officeace_account`）──────────────
 * ① **（ OAuth）**： to need state、， use in 
 * as ，**** and to —— ** not need need OfficeAce
 * **（ not ， not use ）。
 * ② ****： from （ or `models.json`） `baseUrl` and 
 * `modelAppKey`/`modelAppSecret` 。 to this can （ not ）；
 * when （`accessKeyId` etc ）optional。
 *
 * ── no has 「」 this after path ──────────────────────
 * and CodeArts ： can or ， `desktop: false`
 * （ not will 1 after no has ）。
 */
const OFFICEACE: ProviderConfig = {
  provider: 'officeace',
  label: t('OfficeAce 果办'),
  fileImport: JSON_CREDENTIAL_FILE,
  desktop: false,
  webLogin: {
    // is 1 t() ：（ not to ）， not 
    noteHtml: t('打开华为云授权页登录：登录完成后网关会自动<b>轮询云端</b>取回授权码并加入账号列表（不需要装 OfficeAce 桌面端，也无需手工回填回调地址）。'),
    button: t('打开授权页'),
    busyText: t('等待 OfficeAce 登录完成…'),
    // available： is **** （ not 
    // ）， not ， in up not 。
    modes: [
      {
        value: 'embedded',
        label: t('内嵌窗口（推荐）'),
        hint: t('将打开内嵌窗口；登录完成后自动加入账号列表。关掉窗口即取消等待'),
      },
      {
        value: 'external',
        label: t('系统浏览器'),
        hint: t('将用系统默认浏览器打开授权页（会复用浏览器里已登录的华为云账号）；授权码由网关轮询云端取回，与浏览器跑在哪台机器无关，完成后自动加入账号列表'),
      },
    ],
    // by ， in 1 
    webHint: WEB_LOGIN_HINT,
  },
  manualTitle: t('粘贴网关凭据'),
  manualNoteHtml: t('从别的机器（或 OfficeAce 桌面端 <code>models.json</code>）取三项贴进来：模型网关基址（<code>baseUrl</code>，形如 <code>https://modelgw-…/v2</code>）与 <code>modelAppKey</code> / <code>modelAppSecret</code>。这三项就能转发（网关凭据不过期）；控制面临时凭据（<code>accessKeyId</code> 等）可选，留空则额度/自动续期不可用，但不影响转发。'),
  fields: [
    // and URL not t()（， and CodeArts JSON 1 ）
    { key: 'baseUrl', label: t('模型网关基址'), placeholder: 'https://modelgw-…huaweicloud-agentarts.com/v2' },
    { key: 'modelAppKey', label: t('网关 Key'), placeholder: 'model_app_key' },
    { key: 'modelAppSecret', label: t('网关 Secret'), placeholder: 'model_app_secret' },
    { key: 'name', label: t('备注名'), optional: true, placeholder: t('可选，留空用「OfficeAce 果办」') },
  ],
}

/**
 * OrcaTerm（client）： 1 1 provider， no has 。
 *
 * ── ────────────────────────────────────────────────
 * OAuth after 1 JWT（`oauth_access_token`， 2 when when ），
 * `Authorization: Bearer` up 。supports（ and after `orcaterm::credentials`
 * ）： token、`Authorization: Bearer …` 、`ot_session=…`（Web 
 * Cookie ）， or `data.bin` JSON —— will token。
 *
 * ── path ────────────────────────────────────────────
 * （ OAuth ，）/
 * / accessToken。 1 1 JWT
 * （ after `providers::orcaterm::login` and `api::accounts` OrcaTerm ）。
 *
 * ── ──────────────────────────────────────────────
 * client is Tauri use ， in `%APPDATA%\com.orcaterm-desktop.app\data.bin`
 * （** JSON**，none need ）。current， after by 。
 *
 * ── has ───────────────────────────────────────────
 * up `OAuthRefreshToken` old token can new （2026-10-16 ），
 * after by ， use not need need 2 when when （ and KukuAI /
 * CatPaw「 can 」 not ）。（ / up ） need need new 。
 */
const ORCATERM: ProviderConfig = {
  provider: 'orcaterm',
  label: 'OrcaTerm',
  fileImport: { ...JSON_CREDENTIAL_FILE, accept: '.json,.bin,application/json' },
  // （ after providers::orcaterm::login）：** OAuth **，
  // to ； up 
  // `orcaterm://oauth_callback?session_id=…`， down this 1 。
  webLogin: {
    noteHtml: t('在打开的<strong>官方授权页</strong>里用微信扫码登录（页面会直接显示二维码）。扫码成功后自动加入账号列表。'),
    button: t('打开网页登录'),
    hint: t('内嵌窗口打开；扫码完成后自动加入列表，关窗即取消等待'),
    busyText: t('等待 OrcaTerm 登录完成…'),
    // ： to orcaterm:// ， no has ；
    // by ，「 new 」
    webHint: WEB_LOGIN_HINT,
  },
  desktop: true,
  desktopHint: t('读取本机 OrcaTerm 客户端登录态（%APPDATA%\\com.orcaterm-desktop.app\\data.bin），需已在 OrcaTerm 客户端登录'),
  desktopNote: t('读本机客户端当前登录的 OAuth 令牌（明文 JSON，无需解密）。导入后由网关自动续期，无需跟随客户端的 2 小时时效手动换令牌。'),
  manualTitle: t('粘贴 accessToken'),
  manualNoteHtml: t('粘贴 OrcaTerm 的 OAuth accessToken（JWT，三段点分；从桌面客户端 <code>data.bin</code> 或 Web 版 Cookie <code>ot_session</code> 获取）。直接粘贴含 <code>Bearer</code> / <code>ot_session=</code> 的整行也行，网关会自己剥成裸 token。添加后由网关自动续期（上游有续期接口），无需跟随 2 小时时效手动更换。'),
  fields: [
    { key: 'accessToken', label: 'accessToken', rows: 3, placeholder: t('JWT（三段点分；也可直接粘贴 Bearer 整行或 data.bin 内容）') },
    { key: 'name', label: t('备注名'), optional: true, placeholder: t('可选，留空则用账号昵称或 userId') },
  ],
}

/**
 * （`chatglm.cn`）。 to ：client 2.0.7 `app.asar` +
 * `agentmore` up （ after `providers::chatglm` module header）。
 *
 * ── path（ in this 1 up ）────────────────────────
 * 1. ****（`guestLogin`）： not need need not need need ， after empty body
 * 1 `guest/access` to 1 for + 1 new 。「config can use 」
 * —— is ** no has **（ / to / for ），
 * must；
 * 2. ****： from `chatglm_token` cookie 
 * （ or JWT）。 already has use —— this has ， to available；
 * 3. ****：client Cookies。 and KukuAI not ，
 * that is **DPAPI ** （Chromium os_crypt）， has Windows can ，
 * `desktopWindowsOnly`。
 *
 * ── has ────────────────────────────────────────────
 * `POST /chatglm/user-api/user/refresh`（refresh_token new ，available）。
 * when will use ** 1 ** new （ is 1 ，
 * etc 1 new ）。
 */
const CHATGLM: ProviderConfig = {
  provider: 'chatglm',
  label: '智谱清言',
  fileImport: JSON_CREDENTIAL_FILE,
  guestLogin: {
    noteHtml: t('不需要登录：点下面的按钮，网关会直接在智谱清言换发一份<strong>访客凭证</strong>并加入账号列表（也可以先填个备注名）。访客凭证可以直接用来对话，免费额度由上游按设备发放。<br><br><strong>注意</strong>：访客账号<strong>没有积分体系</strong> —— 它与「会员积分」「每日登录积分」「活动领取」相关的接口一律需要登录，因此那些能力对访客会如实跳过（账号页的积分、签到中心都不显示读数）。想要积分就用「填写凭证」粘一个已登录账号的令牌，或「导入桌面端登录态」。'),
    button: t('添加访客账号'),
    busyText: t('正在换发访客凭证…'),
    namePlaceholder: t('可选，给这个访客账号起个备注名'),
  },
  desktop: true,
  desktopWindowsOnly: true,
  desktopHint: t('读取本机智谱清言客户端登录态（%APPDATA%\\chatglm\\Network\\Cookies），需已在客户端登录'),
  desktopNote: t('读本机客户端当前的登录态。客户端的 Cookies 是 DPAPI 加密的（Chromium os_crypt），因此<strong>仅 Windows 可用</strong>；客户端正在运行时可能读不到该文件，先退出客户端再试即可（也可改用「填写凭证」粘贴令牌）。'),
  manualTitle: t('粘贴访问令牌'),
  manualNoteHtml: t('粘贴智谱清言的访问令牌（JWT，三段点分）。获取方式：浏览器打开 <code>chatglm.cn</code> 并登录后，在开发者工具的 Application → Cookies 里复制 <code>chatglm_token</code> 的值（也可以直接粘 <code>chatglm_token=…;</code> 整行）。粘贴后会先向上游校验一次，校验通过才落账号；登录账号的积分、签到能力都能用，之后由网关自动续期（上游有续期接口）。'),
  fields: [
    {
      key: 'accessToken',
      label: 'accessToken',
      rows: 3,
      placeholder: t('JWT（三段点分；也可直接粘贴 chatglm_token=… 整行）'),
    },
    { key: 'name', label: t('备注名'), optional: true, placeholder: t('可选，留空则用账号昵称') },
  ],
}

/**
 * ModelScope（） ： 1 1 provider， `ms-` 。
 *
 * ── 1 use （ this is ）──────────────────────
 * 1 `ms-` `Authorization: Bearer` ，
 * `Cookie: m_session_id=<>` / / daily to （）。
 * has 1 ， use from 「」 1 。
 *
 * ── is ─────────────────────────────────
 * （modelscope.cn） and （modelscope.ai）：
 * will to 401， 1 、 。
 *
 * ── no has 、 no has ───────────────────────────────
 * valid（ up no has refresh ）， can new get； no has client
 * （ and Loomy / MonkeyCode 1 ）， not 「」。
 *
 * ── when 1 （ will down ）──────────────────────────
 * not 、 not ， before must 1 up ——
 * invalid / will in that 1 ， not is etc 1 failure。
 */
function modelscopeForm(spec: {
  provider: string
  label: string
  site: string
  siteNote: string
}): ProviderConfig {
  const { provider, label, site, siteNote } = spec
  return {
    provider,
    label,
    fileImport: JSON_CREDENTIAL_FILE,
    // no has client， no has can file（ by up ）
    desktop: false,
    manualTitle: t('粘贴 API 令牌'),
    // is 1 t() ：（ not to ）， not 
    manualNoteHtml: t('粘贴 ModelScope 的 API 令牌（<code>ms-</code> 开头，在 {site} 的「访问令牌」页获取）。<br>{siteNote}<br>这枚令牌<b>两用</b>：网关既用它调推理，也用它读账号身份、魔粒余额与每日签到。令牌长期有效、没有续期接口，失效后重新获取再粘一次即可。', { site, siteNote }),
    fields: [
      // not t()（， and 1 ）
      { key: 'accessToken', label: 'accessToken', rows: 3, placeholder: t('ms- 开头的 API 令牌') },
      { key: 'name', label: t('备注名'), optional: true, placeholder: t('可选，留空则用上游账号昵称') },
    ],
  }
}

const MODELSCOPE = modelscopeForm({
  provider: 'modelscope',
  label: t('ModelScope 国内版'),
  site: 'modelscope.cn',
  siteNote: t('这里填国内站（modelscope.cn）的账号：请粘贴国内站的令牌。国际站（modelscope.ai）是另一家 provider、另一套账号，两边的令牌不通用。'),
})

const MODELSCOPE_INTL = modelscopeForm({
  provider: 'modelscope-intl',
  label: t('ModelScope 国际版'),
  site: 'modelscope.ai',
  siteNote: t('这里填国际站（modelscope.ai）的账号：请粘贴国际站的令牌。国内站（modelscope.cn）是另一家 provider、另一套账号，两边的令牌不通用。'),
})

/**
 * AtomCode（AtomGit / GitCode）： AtomGit CodingPlan 。
 *
 * ── ────────────────────────────────────────────────
 * ① ****（）： to → use in AtomGit
 * → **** and for （ by in 
 * `acs.atomgit.com/callback` ，** not **， not need need ）。
 * ② ** for **（ after ）：`accessToken` + `refreshToken`。⚠️ `refreshToken`
 * —— ** not is JWT**（ 7 to ， not to `exp`），
 * no has none， to can new 
 * （ `providers/atomcode/credentials.rs` module header）。
 *
 * `desktop: false`： and CodeArts / OfficeAce —— no has 「」 this 
 * after path（AtomCode in `auth.toml` ， and need need ， not this ）。
 */
const ATOMCODE: ProviderConfig = {
  provider: 'atomcode',
  label: t('AtomCode'),
  fileImport: JSON_CREDENTIAL_FILE,
  desktop: false,
  webLogin: {
    // is 1 t() ：（ not to ）， not 
    noteHtml: t('打开 AtomGit 授权页登录：登录完成后网关会自动<b>轮询代理</b>取回令牌并加入账号列表（授权回调由代理终结，不回到本机，因此无需手工回填回调地址）。'),
    button: t('打开授权页'),
    busyText: t('等待 AtomCode 登录完成…'),
  },
  manualTitle: t('粘贴令牌对'),
  // is 1 t() ：（ not to ）， not 
  manualNoteHtml: t('在 AtomGit 的 AtomCode 页面登出后取得两项贴进来：<code>accessToken</code> 与 <code>refreshToken</code>。<b>续期令牌建议填写</b>——本家令牌不是 JWT（约 7 天到期，读不到过期时间），没有它账号无法自动续期，到期只能重新登录。'),
  fields: [
    // and not t()（， and CodeArts JSON 1 ）
    { key: 'accessToken', label: t('访问令牌'), placeholder: 'access_token' },
    { key: 'refreshToken', label: t('续期令牌'), optional: true, placeholder: 'refresh_token' },
    { key: 'userId', label: t('用户 ID'), optional: true, placeholder: t('可选，用于生成账号标识') },
    { key: 'name', label: t('备注名'), optional: true, placeholder: t('可选，留空用「AtomCode」') },
  ],
}

/**
 * LobsterAI（ has 「 has 」`lobsterai.youdao.com`）： 1 1 provider，
 * no has / 。
 *
 * ── has 1 path：（ + loopback ）──────
 * by state 1 and ， use in （ / ） after 
 * 302 ** **：`http://127.0.0.1:<>/auth/callback`，
 * after （ after `providers::lobsterai::oauth` and 
 * `core::login::lobsterai`）。path is `/auth/callback`（ not is `-lobsterai`
 * after ）： that is already ， is for path has no has 。
 *
 * ── no has 「」 this path（ after ， not is ）──────────
 * need ** that 1 ** by up down （`uuid` + `keyfrom`），
 * to not （ after `account_store::lobsterai_accounts` module header）。 after 
 * `api::accounts` for LobsterAI ** 400**， this not 。
 * `fields` empty is `ProviderConfig` （ this in is ）， not is ：
 * 1 is up that 「」。
 *
 * ── no has 「」 this path ──────────────────────────
 * client in ， no has `data.bin` that can 
 * （ and Accio / ZCode 1 ）， `desktop: false`。
 */
const LOBSTERAI: ProviderConfig = {
  provider: 'lobsterai',
  label: t('LobsterAI'),
  desktop: false,
  // ****（ and Accio 1 ）：，
  // （Docker / ） to not —— no has 「」 that 
  // ， this 1 before ， not 。
  webLogin: {
    // is 1 t() ：（ not to ）， not 
    noteHtml: t('打开 <b>LobsterAI（有道龙虾）</b>的官方登录页（<code>lobsterai.youdao.com</code>）并用你的 LobsterAI 账号登录：登录成功后官方页面会跳回本机并自动加入账号列表 —— 回调落在<b>网关自己的端口</b>上，所以浏览器要与网关在同一台机器上；授权码只在本机传给网关，界面不显示明文 token。'),
    button: t('打开 LobsterAI 登录页'),
    busyText: t('等待 LobsterAI 登录完成…'),
    modes: [
      { value: 'embedded', label: t('内嵌窗口（推荐）'), hint: t('将打开内嵌窗口；登录完成后自动加入账号列表。关掉窗口即取消等待') },
      { value: 'external', label: t('系统浏览器'), hint: t('将用系统默认浏览器打开登录页（会复用浏览器里已登录的 LobsterAI 账号）；完成登录后自动加入账号列表，关掉弹窗即取消等待') },
    ],
    // no has ： when can use in 
    // ，「 new 」
    webHint: WEB_LOGIN_HINT,
  },
  fields: [],
}

/** ， and old ADD_FORMS 1 （ DOM ， not ） */
export const BUILTIN_CONFIGS: ProviderConfig[] = [
  RACCOON,
  CATPAW,
  AUTOCLAW,
  AUTOCLAW_INTL,
  // Qoder （ after is provider， and AutoClaw / Accio / ZCode
  // 1 by ）： in in before ， and 1 
  QODER,
  QODER_INTL,
  // its after （ 1 1 ，）：
  // in before ， and after PROVIDERS 1 
  QWENWORK,
  QWENWORK_INTL,
  // Cline up 「」 ： in before （none， use ）
  clineForm({ provider: 'cline-free', label: 'Cline Free', poolNote: t('（免费额度池，模型名带 cline-free/ 前缀）。') }),
  clineForm({ provider: 'cline-pass', label: 'Cline Pass', poolNote: t('（订阅池，模型名带 cline-pass/ 前缀，需要账号有对应订阅）。') }),
  // Accio ： in before （default ）
  accioForm({ provider: 'accio', label: t('国际版'), site: 'www.accio.com', siteNote: t('国际版与国内版是**两套独立的账号**（同一账号体系的两个站点），凭证不通用。') }),
  accioForm({ provider: 'accio-cn', label: t('国内版'), site: 'www.accio-ai.com', siteNote: t('国内版的登录站点是 www.accio-ai.com，与国际版不是同一站。') }),
  // ZCode ： in before （ down that ， and after PROVIDERS 1 ）
  zcodeForm({ provider: 'zcode', label: t('国内版'), site: 'open.bigmodel.cn', planNote: t('国内版与**国际版是两套独立的账号与套餐**，凭证与领取的套餐都不通用。') }),
  zcodeForm({ provider: 'zcode-intl', label: t('国际版'), site: 'api.z.ai', planNote: t('国际版的推理站点是 api.z.ai，与国内版不是同一站；套餐也各自独立。') }),
  // CodeArts（ as AI ）： 1 1 provider， no has /
  // （region cn-north-4， and token must 1 ）。
  CODEARTS,
  // Trae current； when use 。
  TRAE,
  // Loomy（）： 1 （）， in —— and after 
  // PROVIDERS 1 （add 1 in ）
  LOOMY,
  // KukuAI（ AI）： Cookie / ， in 
  // （ and after PROVIDERS 1 ，2026-10 ）
  KUKU,
  // MonkeyCode（） （ and AutoClaw / Qoder 1 ）：
  // in before —— and after PROVIDERS 1 （2026-10 ）。
  MONKEYCODE,
  MONKEYCODE_INTL,
  // Command Code / Antigravity： 1 、 no has ， by after 
  // PROVIDERS in （ after to after ，add in ）。
  COMMANDCODE,
  ANTIGRAVITY,
  // OfficeAce（ as / OfficeClaw）：（ OAuth） or 
  //， in （ and after PROVIDERS 1 ，2026-10 ）
  OFFICEACE,
  // OrcaTerm（client）： accessToken / ，
  // in （ and after PROVIDERS 1 ，2026-10 ）
  ORCATERM,
// （chatglm.cn）： / / ， in 
  // （ and after PROVIDERS 1 ，2026-10 ）
  CHATGLM,
  MODELSCOPE,
  MODELSCOPE_INTL,
  // AtomCode（AtomGit / GitCode）： + for ， in 
  // （ and after PROVIDERS 1 ， after to after ）
  ATOMCODE,
  // LobsterAI（ has 「 has 」）： has 1 ， in 
  // （ and after PROVIDERS 1 ， after to after ）
  LOBSTERAI,
  DOUBAOWORK,
  DEEPSEEK_WEB,
  // Tabbit （ / ）： not use ， for 。
  TABBIT,
  TABBIT_INTL,
]

/** WorkBuddy id（， 1 ） */
export const WORKBUDDY_PROVIDER = 'workbuddy'
/** id（ after provider ） */
export const CUSTOM_PROVIDER = 'custom'

/**
 * WorkBuddy in **** up ： in ， not 。
 *
 * ── Why not use （ after is 「WorkBuddy 」）──────
 * this is ** use ** ： 1 has 1 ， 「」
 * / （ 1 `WorkBuddyBlock` ， add-provider-pick
 * ）。「」 use can to ， is ——
 * and use 。
 *
 * **** this not is 1 ： and is 1 ，
 * not 1 when use will as 。
 *
 * ── use ────────────────────────────────────────
 * 、、Key use is （）： that need can ，
 * this 1 。 not need need this 1 —— AutoClaw / Accio / ZCode 
 * is ****， is 。
 */
export const WORKBUDDY_ENTRY_LABEL = 'WorkBuddy'

export function configOf(providerId: string): ProviderConfig | undefined {
  return BUILTIN_CONFIGS.find(item => item.provider === providerId)
}

/* ─── （）─────────────────────────
 *
 * need （ and 1 ， will to ）， in and 。
 * by methodsOf by config。
 *
 * `oauth` 1 not and 「」： for use is 「」，
 * not —— 1 down （ etc by web-login.js 1 ），
 * AutoClaw need in 1 use can to 。
 * and will that 「 has need 1 」 ， and when and 
 * by not 1 （ this 1 is 1 by ）。
 *
 * `file` `manual` not and `desktop`： is 「」，
 * `desktop` is 「 when when path」 （ has can use ），`file`
 * is use file after （ can ，file can any / ）。
 * failure and not （ after need file、 need JSON）， 1 。 */
export const ADD_METHODS = [
  { id: 'guest', label: t('访客模式（免登录）') },
  { id: 'oauth', label: t('网页登录（Zai / Google）') },
  { id: 'sms', label: t('手机验证码登录') },
  { id: 'web', label: t('网页登录') },
  { id: 'manual', label: t('填写凭证') },
  { id: 'file', label: t('导入凭证文件') },
  { id: 'desktop', label: t('导入桌面端登录态') },
] as const

export type MethodId = (typeof ADD_METHODS)[number]['id']

/**
 * （'macos' / 'windows' / 'linux' / 'web'）， by 。
 *
 * （ no has ） when not to ，empty —— when by 「 not 」：
 * use not this can ， 1 not will ，
 * 1 **available** can will Windows use its 1 。
 */
export const platform = (): string => shared().workbuddyDesktop?.platform || ''

/**
 * in this 1 、 this is available。
 *
 * `desktopWindowsOnly` （AutoClaw / ） need by ：
 * client ， that is Electron / Chromium **DPAPI **
 * （AutoClaw `%APPDATA%/AutoClaw/auth.json`、client Cookies 
 * os_crypt ）， need `CryptUnprotectData` can —— DPAPI has 
 * Windows has 。
 *
 * ── is 「 not is Windows」 not is 「 is macOS」────────────────────
 * this is `platform() === 'macos'`， that is when ；
 * （Linux ） after must —— otherwise Linux will **** this ，
 * use down can to 「failure」， not 。 1 
 * 「 Windows 1 」， will 1 。
 *
 * its （ `~/.box-agent`、CatPaw `~/.meituan-catpaw`、Cline
 * `~/.cline`） is `HOME` down JSON，macOS / Linux up can ，
 * not this 1 （ ProviderConfig desktopNote）。
 */
export function desktopImportAvailable(config: ProviderConfig): boolean {
  if (config.desktop === false) return false
  // （headless platform='web'）： no has client can 
  const current = platform()
  if (current === 'web') return false
  if (config.desktopWindowsOnly && current !== 'windows') return false
  return true
}

/** that 1 in when （current no has use ， up ） */
export function methodsOf(config: ProviderConfig, region: string): MethodId[] {
  return ADD_METHODS.filter(method => {
    if (method.id === 'guest') return Boolean(config.guestLogin)
    if (method.id === 'oauth') return Boolean(config.oauthLogin)
    if (method.id === 'sms') return Boolean(config.smsLogin)
    if (method.id === 'web') return Boolean(config.webLogin) && (!config.webLogin?.region || region === config.webLogin.region)
    if (method.id === 'file') return Boolean(config.fileImport)
    if (method.id === 'desktop') return desktopImportAvailable(config)
    return true
  }).map(method => method.id)
}

/** ：default「」， use */
export const manualTitleOf = (config: ProviderConfig): string => config.manualTitle || t('填写凭证添加')
/** in ，otherwise by （React ） */
export const manualNoteOf = (config: ProviderConfig): { html?: string; text?: string } =>
  (config.manualNoteHtml ? { html: config.manualNoteHtml } : { text: config.manualNote || '' })

/** by */
export const addButtonTextOf = (config: ProviderConfig): string =>
  config.addButton || t('添加 {name} 账号', { name: config.label })

/** DOM id（inputKey has refresh-input id，request as refreshToken） */
export const fieldIdOf = (config: ProviderConfig, field: FieldSpec): string =>
  `${config.provider}-${field.inputKey || field.key}-input`
