/**
 * Agent2API · 「登录 / 添加账号」弹窗里各家的表单配置（数据层，无 JSX）。
 *
 * 逐字合并旧实现的三份来源：add-provider-forms.js 的 ADD_FORMS、以及
 * add-qoder.js / add-cline.js / add-accio.js / add-zcode.js 四份**纯常量**配置
 * （它们各自是独立文件、注册到 window 上，只被 add-provider-forms.js 读 ——
 * 迁到岛上后同一份数据在一个模块里更省事，因此不再挂 window，见最终报告）。
 *
 * 文案、字段名、上限、方法与裁剪规则都逐字保留：这些是能跑通上游的既有实现，
 * 迁的只是界面外壳。WorkBuddy 不在这张表里 —— 它的块结构特殊（账号版本 +
 * 打开方式 + 第三方入口开关），单独一个组件，见 add-provider-blocks.tsx。
 */

import { shared } from './add-account-bridge'
import { DOUBAOWORK } from './add-doubaowork-config'
import { DEEPSEEK_WEB } from './add-provider-deepseek'
import { MONKEYCODE, MONKEYCODE_INTL } from './add-provider-monkeycode'
import { TABBIT, TABBIT_INTL } from './add-provider-tabbit'
import { t } from '../i18n'

/* ─── 类型 ───────────────────────────────── */

/** 一个手填字段：key 直接对应请求体键名（inputKey 只改 DOM id 的中间段） */
export type FieldSpec = {
  key: string
  inputKey?: string
  label: string
  optional?: boolean
  rows?: number
  placeholder: string
  /**
   * 整段是一个 JSON（CodeArts 的凭据就是这么落到用户手上的），提交时**逐键并进
   * 请求体**、原始那串文本不留。为什么在前端展开而不是把整串交给后端：
   * `POST /api/accounts` 的各家分派读的是平铺字段（`Credential::from_payload`
   * 认「带外层包装」与「铺平」两种形状，但不认「一个字符串里装着 JSON」），
   * 让后端再多一种形状 = 给所有家共用的路径加一个只有一家走到的分支。
   */
  jsonExpand?: boolean
}

export type RegionOption = { value: string; label: string }
export type MethodMode = { value: string; label: string; hint?: string }

export type WebLoginConfig = {
  /** 说明里的行内标记（原样注入，与旧实现的 noteOf 同口径） */
  noteHtml?: string
  button: string
  hint?: string
  busyText?: string
  /**
   * 有这一级时多一个「打开方式」分段（没有的家只给内嵌窗口）。
   *
   * 只在桌面壳里渲染：网页端（`platform() === 'web'`）没有「内嵌窗口」与
   * 「系统浏览器」之分 —— 桥接层的 startLogin 收下 mode 后并不会用它，
   * 一律 `window.open` 开新标签页（见 web_shim.rs 的同名方法）。所以网页端
   * 整级收起，改由 `webHint` 说明唯一那条路。
   */
  modes?: MethodMode[]
  /** 网页端那一条提示（缺省用通用的「将在新标签页中打开」） */
  webHint?: string
  /** Cookie 捕获需要桌面窗口，网页面板不可发起。 */
  desktopOnly?: boolean
  /** 这一家的网页登录只在某个地区可用（当前没有家用它，结构上保留） */
  region?: string
  /** 写死的变体选择器（传给壳侧 start_login 的第一个参数） */
  edition?: string
}

export type SmsLoginConfig = { noteHtml?: string }

export type OauthLoginConfig = {
  title?: string
  noteHtml?: string
  hint?: string
  /** 同 WebLoginConfig.modes：网页端整级收起，用 webHint 替代 */
  modes?: MethodMode[]
  webHint?: string
}

/** 网页端唯一打开方式的提示（没有 webHint 的家用它） */
export const WEB_LOGIN_HINT = t('The authorization page opens in a new tab; the account is added to the list automatically when done.')

/**
 * 回调落在本机 / 网关端口那几家的网页端提示：跨机部署时官方页面跳回的地址
 * 属于浏览器所在电脑、回不到网关，需要用户把最终回调地址粘回来。
 */
export const WEB_LOGIN_PASTE_HINT = t('The authorization page opens in a new tab; the account is added to the list automatically when done. If the page jumps back to a local address and the account does not appear, paste the full callback URL from the address bar when prompted.')

export type ProviderConfig = {
  provider: string
  label: string
  /** 手填段的主按钮文案（缺省「添加 X 账号」） */
  addButton?: string
  manualTitle?: string
  manualNote?: string
  manualNoteHtml?: string
  fields: FieldSpec[]
  webLogin?: WebLoginConfig
  smsLogin?: SmsLoginConfig
  oauthLogin?: OauthLoginConfig
  /**
   * 「访客模式」——本项目的**独一份**：一键换发访客凭证，不需要任何输入。
   *
   * 只有智谱清言有这一条（后端 `guest/access` 空 body 恒可换发，见
   * `providers::chatglm` 的模块头）。它在界面上是一个**没有表单**的分段：
   * 一段说明 + 一个按钮，点下去就加号（备注名可选）。
   * 语义上它不是「登录」，而是「先拿到一个能用的身份」—— 代价是没有积分体系
   * （余额 / 签到对访客如实跳过），所以说明里必须写清这件事。
   */
  guestLogin?: { noteHtml: string; button: string; busyText?: string; namePlaceholder?: string }
  regionOptions?: RegionOption[]
  /** false = 这一家没有「从本机导入桌面端登录态」这一条来源 */
  desktop?: boolean
  desktopNote?: string
  desktopHint?: string
  /** 登录态是 Electron safeStorage 密文、解密要走 DPAPI（仅 Windows） */
  desktopWindowsOnly?: boolean
  /**
   * 「导入凭证文件」这一条来源的配置（有它才有那个分段）。
   *
   * 与 `desktop` 的语义差别（别混用，见后端 `api::accounts` 的文件导入契约）：
   * `desktop` 只是建一条「转发时**实时**读本机固定路径」的记录；这里是把用户
   * **手选**的文件解析后落库 —— 得到可独立转发的账号，文件可以来自别的机器 / 备份。
   */
  fileImport?: {
    /** 文件选择器的 accept（空串 = 不过滤：KukuAI 的 Cookies 文件没有扩展名） */
    accept: string
    /** 段落说明：选哪个文件、在哪、导入后的凭证落在哪（正文里写清，界面不再折叠） */
    note: string
    /** true = 二进制文件，前端读 base64 传 `authFileB64`；否则读文本传 `authFileContent` */
    binary?: boolean
  }
}

/* ─── 长度上限（与后端逐条对齐，前端先挡一次）────────────── */

/** 备注名长度上限（与后端 truncate_chars(name, 100) 一致） */
export const MAX_NAME_LENGTH = 100
/** uid / deviceId 长度上限（与后端 MAX_USER_UID_LENGTH / MAX_IDENTITY_LENGTH 一致） */
export const MAX_IDENTITY_LENGTH = 256
/** 单行控件的长度上限：备注名按 100，其余标识字段按 256 */
export const maxLengthOf = (field: FieldSpec): number =>
  (field.key === 'name' ? MAX_NAME_LENGTH : MAX_IDENTITY_LENGTH)

/* ─── 各家配置 ─────────────────────────────── */

const JSON_CREDENTIAL_FILE: NonNullable<ProviderConfig['fileImport']> = {
  accept: '.json,application/json',
  note: t('Select a JSON file containing this provider\'s credentials or an individual account JSON file exported by this application. Imported credentials are stored in the gateway and can be used independently.'),
}

const RACCOON: ProviderConfig = {
  // raccoon_accounts::add_raccoon_account：token / access_token、refreshToken / refresh_token、name
  provider: 'raccoon',
  label: 'Raccoon',
  addButton: t('Add Raccoon account'),
  // 网页登录（后端 providers::raccoon::oauth）：官方登录页 + 一次性授权码回调。
  // 桌面端仍优先用内嵌窗口；网页端 / Docker 远程面板会把授权页切到官方
  // redirect 分支自动回调，前提是浏览器以 127.0.0.1/localhost（http + 端口）
  // 访问面板 —— 上游白名单如此，不满足时网页端发起前就会弹指引拦下
  // （见 web_shim 的 raccoonCallbackOriginProblem）。
  webLogin: {
    // 桌面端在内嵌窗口里跑；网页端没有内嵌窗口，统一在新标签页里打开，
    // 所以这句不写死「内嵌窗口」（否则网页端看到的是一句不成立的话）
    noteHtml: t('Complete the sign-in on the official page that opens; the account is added to the list automatically on success.'),
    button: t('Open web login'),
    hint: t('Opens in the embedded window; added to the list when done; closing it cancels the wait'),
    busyText: t('Waiting for Raccoon sign-in…'),
    // 网页端走的是官方 redirect 分支（见上方注释），浏览器在哪台机器上都能
    // 直接回网关，没有粘贴回调那一步
    webHint: WEB_LOGIN_HINT,
  },
  manualTitle: t('Paste token / refreshToken'),
  manualNoteHtml: t('refreshToken is optional and enables automatic renewal on expiry; both can be obtained from the <a href="#" class="raccoon-hint-link" data-raccoon-hint>Raccoon client login-state file</a>.'),
  fields: [
    { key: 'name', label: t('Label'), optional: true, placeholder: t('Optional; defaults to the account name in the credential') },
    { key: 'token', label: 'token', rows: 3, placeholder: t('Paste access_token (a long JWT)') },
    { key: 'refreshToken', inputKey: 'refresh', label: 'refreshToken', rows: 2, optional: true, placeholder: t('Optional') },
  ],
  desktopNote: t('Reads the local client\'s current login state live each time (deleting this record does not affect the client\'s login state). After signing in again in the client, click "Refresh token" to sync.'),
  // 「导入凭证文件」：客户端登录态是明文 JSON，前端读文本传 authFileContent。
  // 与上面 desktop 那条并排保留是刻意的 —— 一个是「实时读本机固定路径」，
  // 一个是「把用户手选的文件解析后落库」（文件可来自别的机器 / 备份）。
  fileImport: {
    accept: '.json,application/json',
    note: t('Pick the Little Raccoon client\'s login-state file auth.json (Windows: C:\\\\Users\\<Your username>\\.box-agent\\config\\auth.json; macOS / Linux: ~/.box-agent/config/auth.json). After import, credentials land directly in the gateway and can forward independently, no longer depending on the local client.'),
  },
}

const CATPAW: ProviderConfig = {
  // catpaw_accounts::add_catpaw_account：token / accessToken / access_token / auth_token
  // （即 X-Passport-Token）、uid / userId / loginName、name；没有刷新机制
  provider: 'catpaw',
  label: 'CatPaw',
  // 网页登录：美团 passport 授权页 + loopback 回调；服务端同时轮询 poll-token，
  // 因此 Docker / 远程面板不依赖浏览器访问容器内的 loopback 端口。
  webLogin: {
    noteHtml: t('Sign in with your CatPaw account (Meituan Passport); the account is added to the list automatically on success.'),
    button: t('Open CatPaw web login'),
    busyText: t('Waiting for CatPaw sign-in…'),
    modes: [
      { value: 'embedded', label: t('Embedded window (recommended)'), hint: t('Opens in the embedded window; added to the list when done; closing it cancels the wait') },
      { value: 'external', label: t('System browser'), hint: t('Opens in the system browser (reuses the signed-in Meituan account); added to the list when done') },
    ],
  },
  manualNote: t('token is the login-state Cookie; CatPaw has no refresh mechanism, so sign in again in the client after it expires.'),
  fields: [
    { key: 'token', label: 'token', rows: 3, placeholder: t('CatPaw\'s X-Passport-Token (login-state Cookie)') },
    { key: 'uid', label: 'uid', placeholder: t('Required; CatPaw account identifier') },
    { key: 'name', label: t('Label'), optional: true, placeholder: t('Optional; defaults to the login name or uid') },
  ],
  desktopNote: t('Reads the local client\'s current login state live each time (deleting this record does not affect the client\'s login state). After signing in again in the client, just re-import.'),
  desktopHint: t('Reads ~/.meituan-catpaw/auth.json; requires being signed in to the CatPaw client'),
  fileImport: {
    accept: '.json,application/json',
    note: t('Select CatPaw The client\'s login-state file auth.json (Windows: C:\\Users\\<user>\\AppData\\Roaming\\workbuddy\\)\\Users\\<Your username>\\.meituan-catpaw\\auth.json; macOS / Linux: ~/.meituan-catpaw/auth.json). After import, credentials land directly in the gateway and can forward independently, no longer depending on the local client.'),
  },
}

/**
 * AutoClaw 两个地区各占一项（与 Cline 两池同一手法）：`autoclaw`（国内版，id 不变
 * —— 存量账号的落盘契约）与 `autoclaw-intl`（国际版）。两项相邻排列。
 *
 * 两地的差别：域名（后端 `autoclaw::region`，前端不体现）；桌面端导入两地都给
 * （auth.json 两地共用、没有地区标记，地区由用户在哪一项下点导入决定）；
 * **登录方式完全不同** —— 国内版只有手机验证码，国际版只有 Zai / Google OAuth。
 */
const AUTOCLAW: ProviderConfig = {
  provider: 'autoclaw',
  label: t('AutoClaw China edition'),
  fileImport: JSON_CREDENTIAL_FILE,
  manualNote: t('token supports the enc: prefix (decrypted automatically on Windows). Without a refreshToken, automatic renewal is not possible.'),
  // 国内版**唯一**的官方登录方式（它的登录页不渲染 OAuth 按钮，已核对构建产物）
  smsLogin: {
    noteHtml: t('Sign in with your bound phone number: click "Get code" and enter it. This is the only official sign-in method for the China edition.'),
  },
  fields: [
    { key: 'token', label: 'token', rows: 3, placeholder: t('Plaintext JWT, or the enc: encrypted value in auth.json (decrypted automatically)') },
    { key: 'refreshToken', label: 'refreshToken', rows: 2, optional: true, placeholder: t('Without it, automatic renewal is not possible') },
    { key: 'deviceId', label: 'deviceId', optional: true, placeholder: t('Optional; sent when renewing') },
    { key: 'name', label: t('Label'), optional: true, placeholder: t('Optional; defaults to userId') },
  ],
  desktopNote: t('Reads the local client\'s current login state live each time (deleting this record does not affect the client\'s login state).'),
  desktopHint: t('Reads and decrypts %APPDATA%/AutoClaw/auth.json; Windows only'),
  // 登录态是 Electron safeStorage 密文，解密要走 DPAPI（仅 Windows），
  // 因此 macOS 上整段收起（理由见 desktopImportAvailable）
  desktopWindowsOnly: true,
}

const AUTOCLAW_INTL: ProviderConfig = {
  // AutoClaw 国际版（autoglm-api.autoglm.ai）：与国内版同一套协议与签名指纹
  // （appId/appKey 两地逐字相同，已实测），只有站点不同。
  provider: 'autoclaw-intl',
  label: t('AutoClaw International edition'),
  fileImport: JSON_CREDENTIAL_FILE,
  manualNote: t('The International and China editions have separate account systems; enter the International edition\'s credentials. Without a refreshToken, automatic renewal is not possible.'),
  // OAuth 网页登录：国际版**唯一**的登录方式（登录页只渲染 Zai / Google 两个按钮）。
  // 与另外五家的差别：授权地址前有一次强制风控验证码（阿里云滑块），
  // 必须在浏览器里跑完才能拿地址，因此点按钮后会先在本弹窗里弹滑块
  // （见 ui/autoclaw-oauth.js 的文件头）。
  oauthLogin: {
    title: t('Web login (Zai / Google)'),
    noteHtml: t('After clicking the button, first pass a slider verification (the vendor\'s risk control), then the sign-in page opens and the account is added automatically once you sign in.<br>This is the only official sign-in method for the International edition; if you are already signed in to the client, "Import desktop login state" is faster.'),
    modes: [
      { value: 'embedded', label: t('Embedded window (recommended)'), hint: t('Opens in the embedded window; closing it cancels the wait (if a Google account is rejected, switch to the system browser)') },
      { value: 'external', label: t('System browser'), hint: t('Opens in the system browser (reuses the signed-in Zai / Google account); added to the list when done') },
    ],
    // 网页端由 web_shim 的 runLoginFlow 开新标签页；这条链靠服务端轮询
    // 落定结果（不是回调），所以不需要「粘贴回调地址」那句话
    webHint: WEB_LOGIN_HINT,
  },
  fields: [
    { key: 'token', label: 'token', rows: 3, placeholder: t('Plaintext JWT (the access token of an International edition account)') },
    { key: 'refreshToken', label: 'refreshToken', rows: 2, optional: true, placeholder: t('Without it, automatic renewal is not possible') },
    { key: 'deviceId', label: 'deviceId', optional: true, placeholder: t('Optional; sent when renewing') },
    { key: 'name', label: t('Label'), optional: true, placeholder: t('Optional; defaults to userId') },
  ],
  // 桌面端登录态导入两个地区都给：auth.json 没有地区标记、本机判断不了，
  // 用户在哪一项下点导入就得到哪一家的账号，猜错的后果是可见的上游 401。
  desktopNote: t('Reads the local client\'s current login state live each time (deleting this record does not affect the client\'s login state). Users who sign in to the client with Zai / Google should use this one.'),
  desktopHint: t('Reads and decrypts %APPDATA%/AutoClaw/auth.json; Windows only'),
  desktopWindowsOnly: true,
}

/**
 * Qoder 两个地区（中国版 / 国际版）：同一套设备授权协议，只有站点主机不同。
 *
 * 为什么不是一个配置带一个「地区」下拉（拆家，2026-10）：与 AutoClaw / Accio /
 * ZCode 同一思路 —— 地区是 **provider 身份**而不是账号属性。拆家前两地区账号
 * 混在一个「模型来源」下拉里、一次「获取模型」只能刷到队首账号所属地区的目录；
 * 按两家建模后各自有独立的账号、清单与启停（除 id / 标签 / 文案外同构）。
 */
function qoderForm(spec: { provider: string; label: string; siteNote: string }): ProviderConfig {
  const { provider, label, siteNote } = spec
  return {
    provider,
    label,
    desktop: false,
    // 地区写死在 provider 身份里（后端按 provider id 反查），不再走地区分段
    webLogin: {
      // 两站是同一套 PKCE 设备授权协议，只有站点主机不同；edition 由 provider
      // 身份决定（后端按 provider id 反查地区，请求里的 edition 只是回显字段）
      noteHtml: t('Open the official authorization page and complete the device-code authorization; you will be signing in to a {label} ({site}) account.', { label, site: siteNote }),
      button: t('Open Qoder web login'),
      busyText: t('Waiting for Qoder authorization…'),
      edition: provider === 'qoder' ? 'cn' : 'global',
      modes: [
        { value: 'embedded', label: t('Embedded window (recommended)'), hint: t('The embedded window uses a fresh environment, so multiple accounts do not affect each other; closing it cancels the wait') },
        { value: 'external', label: t('System browser'), hint: t('Opens in the system browser (reuses a signed-in account); added to the list when done') },
      ],
    },
    manualTitle: t('Use a personal access token (PAT)'),
    manualNote: t('Generate a PAT in the Qoder account settings → Integrations for {label} (do not use a Google / GitHub token). Accounts and credentials do not carry over between the two regions.', { label }),
    fields: [
      { key: 'pat', label: 'Qoder PAT', rows: 3, placeholder: t('Paste the Qoder personal access token (pt-…)') },
      { key: 'name', label: t('Label'), optional: true, placeholder: t('Optional; defaults to the account nickname or email') },
    ],
  }
}

const QODER = qoderForm({ provider: 'qoder', label: t('China edition'), siteNote: 'qoder.com.cn' })
const QODER_INTL = qoderForm({ provider: 'qoder-intl', label: t('International edition'), siteNote: 'qoder.com' })

/**
 * 千问办公两版（国内版 `qwenwork` / 国际版 `qwenwork-intl`）。
 *
 * ── 登录链路 2026-11-03 换过一次（先读插件 `providers/qoder/authorize.rs`）──
 * 旧实现照社区参考实现抄的是**设备码授权**，上游已经把这个入口关了：那个地址
 * 对任何请求都回 400。现在走**授权码 + 本机回调**（官方 CLI / IDE 那条路）：
 * 浏览器打开官方登录页，登录完成后跳回本插件在本机开的随机端口，账号随即自动
 * 加入列表。国内版可用；国际版上游只认它自己注册的回调地址，授权码回不到本机，
 * 所以界面上直接写明「暂时无法登录」，而不是给一个点了必然报错的按钮。
 *
 * **只有网页登录这一条**：千问办公没有个人访问令牌（PAT）接口，
 * 后端 `Site::supports_pat` 为假，登录方式也只申报了网页登录 —— 界面上给一个
 * 点了必然报错的入口比不给更糟，所以这里不配 `manualTitle` / `fields`
 * （Qoder 那一份才有 PAT 段）。
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
    // 站点写死在 provider 身份里（后端按 provider id 反查）
    webLogin: {
      noteHtml: loginable
        ? t('Sign in to your QwenWork {label} ({site}) account in the page that opens. The account joins the list as soon as you finish. The two editions have separate accounts and credentials.', { label, site: siteNote })
        : t('QwenWork {label} cannot be signed in right now: the upstream service only accepts its own official callback address, so the authorization code never comes back to this machine.', { label }),
      button: loginable ? t('Open QwenWork web login') : t('Sign-in unavailable'),
      busyText: t('Waiting for the QwenWork sign-in to finish…'),
      edition: provider === 'qwenwork' ? 'cn' : 'global',
      modes: [
        { value: 'embedded', label: t('Embedded window (recommended)'), hint: t('The embedded window uses a fresh environment, so multiple accounts do not affect each other; closing it cancels the wait') },
        { value: 'external', label: t('System browser'), hint: t('Opens in the system browser (reuses a signed-in account); added to the list when done') },
      ],
    },
    fields: [
      { key: 'name', label: t('Label'), optional: true, placeholder: t('Optional; if left blank the account name or email is used') },
    ],
  }
}

const QWENWORK = qwenworkForm({
  provider: 'qwenwork',
  label: t('China edition'),
  siteNote: 'qwenwork.cn',
  loginable: true,
})
const QWENWORK_INTL = qwenworkForm({
  provider: 'qwenwork-intl',
  label: t('International edition'),
  siteNote: 'qwenwork.ai',
  loginable: false,
})

/**
 * Cline 是两个提供商（Cline Free / Cline Pass）。
 *
 * 为什么不是一个配置带一个「额度池」下拉：上游模型带计费通道前缀
 * （`cline-free/…` / `cline-pass/…`），池是**账号的属性**，混在一起会让
 * 「哪个池能用」在界面上看不出来、同一模型的启停规则跨池互相影响。
 * 按两家建模后各占一个分组、各有一套账号与模型清单。除 id / 标签 / 文案外同构。
 */
function clineForm(spec: { provider: string; label: string; poolNote: string }): ProviderConfig {
  const { provider, label, poolNote } = spec
  return {
    provider,
    label,
    // 桌面登录态本身不属于任何池，两家都能导入（想要两个池都用就在两家各导入一次）
    desktop: true,
    webLogin: {
      // 设备授权登录（WorkOS RFC 8628）：没有回调、没有自定义协议，
      // 就是「打开授权页 → 用户确认 → 网关轮询拿到令牌」，因此两种打开方式都可行。
      noteHtml: t('Open the authorization page and confirm; the account is added to <b>{label}</b> automatically{note}', { label, note: poolNote }),
      button: t('Open Cline authorization page'),
      busyText: t('Waiting for Cline authorization confirmation…'),
      modes: [
        { value: 'embedded', label: t('Embedded window (recommended)'), hint: t('Opens in the embedded window; added to the list when done; closing it cancels the wait') },
        { value: 'external', label: t('System browser'), hint: t('Opens in the system browser (reuses the signed-in Cline account); added to the list when done') },
      ],
    },
    manualTitle: t('Enter credentials'),
    manualNote: t('refreshToken is optional and enables automatic renewal; both can be obtained from the client login-state file. The same Cline account works in both pools; here it is grouped under {label}.', { label }),
    fields: [
      { key: 'accessToken', label: 'accessToken', rows: 3, placeholder: t('Paste a token starting with workos:… (the prefix is added automatically if missing)') },
      { key: 'refreshToken', label: 'refreshToken', rows: 2, optional: true, placeholder: t('Optional; without it, automatic renewal is not possible') },
      { key: 'name', label: t('Label'), optional: true, placeholder: t('Optional; defaults to the account name, email, or token fingerprint') },
    ],
    desktopNote: t('Reads the local client\'s current login state live each time (deleting this record does not affect the client\'s login state). This one is grouped under {label}.', { label }),
    desktopHint: t('Reads ~/.cline/data/settings/providers.json; requires being signed in to the Cline client'),
    // 「导入凭证文件」：providers.json 是**设置**文件（明文 JSON），前端读文本传
    // authFileContent。归池与 desktop 那条同一口径：在哪一项下导入就归入哪个池。
    fileImport: {
      accept: '.json,application/json',
      note: t('Select Cline the client\'s login-state file providers.json (Windows: C:\\Users\\<user>\\AppData\\Roaming\\workbuddy\\)\\Users\\<Your username>\\.cline\\data\\settings\\providers.json; macOS / Linux: ~/.cline/data/settings/providers.json). After import, credentials land directly in the gateway and can forward independently, no longer depending on the local client; this one is filed under {label}.",', { label }),
    },
  }
}

/**
 * Accio 两个地区（国际版 / 国内版）：同一个网关、同一套接口，只有登录站点与
 * x-package-region 头不同。地区不是账号的字段而是身份，因此输出两份配置。
 */
function accioForm(spec: { provider: string; label: string; site: string; siteNote: string }): ProviderConfig {
  const { provider, label, site, siteNote } = spec
  return {
    provider,
    label,
    // OAuth 2.0 授权码 + PKCE：同机直接接收 loopback；Docker / 远程面板
    // 由网页 shim 提示粘贴最终回调地址（与 CodeArts 同一兜底形态）。
    webLogin: {
      noteHtml: t('Open the official sign-in page for Accio <b>{label}</b> (<code>{site}</code>) and sign in with your Accio account: on success the official page redirects back to this machine; a same-machine setup completes automatically, while Docker / remote panels just paste the callback URL as prompted (the authorization code is passed to the gateway only on this machine, and no plaintext token is shown in the UI).', { label, site }),
      button: t('Open the Accio {label} sign-in page', { label }),
      busyText: t('Waiting for Accio {label} sign-in…', { label }),
      modes: [
        { value: 'embedded', label: t('Embedded window (recommended)'), hint: t('The embedded window will open; the account is added to the list automatically after sign-in. Closing the window cancels the wait') },
        { value: 'external', label: t('System browser'), hint: t('The sign-in page will open in your system default browser (reusing the Accio account already signed in there); the account is added to the list automatically after sign-in, and closing the popup cancels the wait') },
      ],
      webHint: WEB_LOGIN_PASTE_HINT,
    },
    manualTitle: t('Enter credentials'),
    manualNoteHtml: t('accessToken is Accio\'s login credential (a long opaque token, not a JWT); refreshToken is optional and enables automatic renewal on expiry. Enter the credential of the <b>{label}</b> account — {siteNote} (the easiest way to get it: just use "Web login" above; no need to hunt for the token manually.)', { label, siteNote }),
    fields: [
      { key: 'accessToken', label: 'accessToken', rows: 3, placeholder: t('Paste the Accio accessToken') },
      { key: 'refreshToken', label: 'refreshToken', rows: 2, optional: true, placeholder: t('Optional; without it, automatic renewal is not possible') },
      { key: 'name', label: t('Label'), optional: true, placeholder: t('Optional; defaults to the account nickname or email') },
    ],
  }
}

/**
 * ZCode 两个地区（国内版 / 国际版）：同一个 zcode 平面（登录 / 领取都在
 * zcode.z.ai），只有推理站点不同，因此输出两份配置。
 */
function zcodeForm(spec: { provider: string; label: string; site: string; planNote: string }): ProviderConfig {
  const { provider, label, site, planNote } = spec
  return {
    provider,
    label,
    fileImport: JSON_CREDENTIAL_FILE,
    // 本家没有「桌面端实时登录态」可导入：凭证在它自己的加密存储里，
    // 没有 auth.json 那种稳定可读的形态（与 Accio 同一处境）
    desktop: false,
    webLogin: {
      // 授权地址由**服务端**给且不回本机：网关拿到一次性授权地址，用户在浏览器里
      // 授权后由 ZCode 服务端记录结果，网关在后台轮询取回凭证。因此两种打开方式
      // 都走得通（与 Accio 同构），但「页面最后提示无法打开 zcode:// 链接」是正常的。
      noteHtml: t('Open the official authorization page for ZCode <b>{label}</b> and sign in with your account. The authorization result is recorded by the ZCode server; the gateway fetches the credential in the background and adds the account to the list.', { label }),
      button: t('Open the ZCode {label} authorization page', { label }),
      busyText: t('Waiting for ZCode {label} authorization…', { label }),
      modes: [
        { value: 'embedded', label: t('Embedded window (recommended)'), hint: t('The embedded window will open; the account is added to the list automatically once authorization completes. Closing the window cancels the wait. The authorization page may end with a message that it cannot open the zcode:// link — that is normal; the result has already been recorded server-side') },
        { value: 'external', label: t('System browser'), hint: t('The authorization page will open in your system default browser (reusing the ZCode account already signed in there); the account is added to the list automatically once authorization completes. A final message that the zcode:// link cannot be opened is normal') },
      ],
    },
    manualTitle: t('Enter credentials'),
    manualNoteHtml: t('This provider has two credentials that **do not replace each other**; fill in the one(s) for the features you need, at least one: the <b>Coding Plan API Key</b> is used for forwarding inference (hits <code>{site}</code>) and for querying the window limits (the remaining percentage of each N-hour / weekly window); the <b>jwt</b> is used for claiming plans and querying balance (hits the billing gateway at <code>zcode.z.ai</code>; officially called a Coding Plan JWT, a three-segment dot-separated string). Either one can query balance, but they read different things: the jwt reads the balance buckets and the plan expiry, while the API Key reads the window limits; and claiming only accepts the jwt, forwarding only accepts the API Key. Please enter the credentials for the <b>{label}</b> account — {plan} (the easiest way: just use "Web login" above — it exchanges this API Key for you and gets both credentials at once.)', { site, label, plan: planNote }),
    fields: [
      {
        key: 'accessToken',
        label: t('Coding Plan API Key'),
        rows: 3,
        optional: true,
        // 这里不能填 OAuth 登录态：那个串拿去转发会被上游按「OAuth 令牌」那条路
        // 校验并回 401。官方客户端与「网页登录」这条链都是先换成编码套餐 API Key 再用。
        placeholder: t('Used for forwarding; two dot-separated segments like apiKey.secret (if left blank, this account cannot forward)'),
      },
      { key: 'jwt', label: 'Coding Plan JWT', inputKey: 'jwt', rows: 3, optional: true, placeholder: t('Used for claiming plans and querying balance; if left blank, this account cannot claim or check balance') },
      { key: 'userId', label: t('User ID'), optional: true, placeholder: t('Optional; used to generate the account id and display name') },
      { key: 'name', label: t('Label'), optional: true, placeholder: t('Optional; generated automatically if blank') },
    ],
  }
}

/**
 * CodeArts（华为云 AI 代码助手 / snap-access）。
 *
 * ── 网页登录（OAuth 授权码 + PKCE）────────────────────────
 * 授权地址由网关拼（`providers::codearts::oauth::authorize_url`，参数与官方扩展
 * 逐字对过向量），portal 完成后把浏览器送回**网关自己的端口**上的
 * `http://127.0.0.1:<port>/oauth/callback` —— 路径不是我们能挑的：portal 只认授权
 * 地址里给的 `port`。所以浏览器与网关在同一台机器时两次回调（先 secret+redirect、
 * 再 code）都能落地；不在同一台时浏览器跳的是它自己的 127.0.0.1，到不了网关。
 * 那种部署（网关跑在 NAS / 服务器上）的用法写在下面的提示里，两条通道都通：
 *   · Docker / 远程面板直接把地址栏中的最终回调地址粘贴回网页 shim —— 带 `code` 就直接落账；
 *   · 只带 `secret` 时网关会转去走 ticket 轮询通道，代价是**拿不到 refresh token**
 *     （约一小时后要重新登录一次），界面上这句提示是照实说的。
 *
 * ── 凭据为什么是一整块、且只有一个 JSON 框 ──────────────────
 * CodeArts 的凭据是一次 OAuth/PKCE 登录换来的**临时三元组**（AK/SK/STS，实测约
 * 一小时到期），续期还要出示**当初那次登录的 PKCE verifier 与 DPoP 私钥**，
 * 所以少搬一半到期就刷不回来。而用户手上这份数据的**原生形状就是一段 JSON**
 * （官方插件写 `codearts_provider_credential`、CLIProxyAPI 的 auth 文件整份包在
 * 里面）：拆成六个框会把「粘哪一格」变成六次出错机会，`oauth_context` 本身还是
 * 嵌套对象、框里塞不下。因此一个框整份粘，由 `jsonExpand` 解析后铺开。
 */
const CODEARTS: ProviderConfig = {
  provider: 'codearts',
  label: 'CodeArts',
  // 本家**没有**「读本机客户端登录态」这条后端路径（凭据只能靠网页登录或粘贴）。
  // 不显式关掉的话，桌面壳里会出现一个选了之后什么都没有的分段：chip 只看
  // `desktop !== false`，而下面的 desktopNote 缺席就返回空串。
  // 浏览器面板看不到这个洞（platform()==='web' 时整段收起），只有 App 里会露。
  desktop: false,
  webLogin: {
    noteHtml: t('Open the official Huawei Cloud CodeArts authorization page to sign in: once done, the official page brings the browser back to the <b>gateway\'s own</b> <code>/oauth/callback</code>, and the gateway exchanges the one-time authorization code for temporary credentials and adds the account to the list.<br>When the gateway runs on another machine, the web UI prompts you to paste the final callback URL from the address bar back into the panel.'),
    button: t('Open CodeArts authorization page'),
    busyText: t('Waiting for CodeArts sign-in…'),
    modes: [
      {
        value: 'embedded',
        label: t('Embedded window (recommended)'),
        hint: t('The embedded window will open; the account is added to the list automatically after sign-in. Closing the window cancels the wait'),
      },
      {
        value: 'external',
        label: t('System browser'),
        hint: t('The authorization page will open in your system default browser (reusing the Huawei Cloud account already signed in there); when the browser and the gateway are not on the same machine, paste the final callback URL back into the panel as described above'),
      },
    ],
    webHint: WEB_LOGIN_PASTE_HINT,
  },
  manualTitle: t('Paste login credentials'),
  manualNoteHtml: t('Paste the whole credential JSON written to disk by the official plugin / CLIProxyAPI (like <code>{"codearts_provider_credential":{…}}</code>; a flattened form also works).<br>Required: <code>access_key_id</code>, <code>secret_access_key</code>, <code>security_token</code>.<b>For automatic renewal you must paste <code>refresh_token</code> and <code>oauth_context</code> as well</b> — temporary credentials expire in about an hour, and without that half they cannot be renewed; you would have to sign in again.'),
  fields: [
    {
      key: 'credentialJson',
      label: t('Credential JSON'),
      rows: 8,
      // 解析后按字段铺开进请求体（后端 `Credential::from_payload` 嵌套/平铺都认）
      jsonExpand: true,
      placeholder: '{"codearts_provider_credential":{"access_key_id":"HSTA…","secret_access_key":"…",'
        + '"security_token":"…","expires_at":"2026-09-27T16:17:00.327Z","domain_id":"…","user_id":"…",'
        + '"user_name":"…","refresh_token":"eyJ…","oauth_context":{…}}}',
    },
    { key: 'name', label: t('Label'), optional: true, placeholder: t('Optional; defaults to the user_name in the credential') },
  ],
}

/**
 * Trae（国内账号，Agent / IDE 聊天通道按模型目录选择）。
 *
 * ── 为什么只有一张卡、没有「地区」下拉 ──────────────────────
 * 国内 Agent / IDE 共用 trae.cn 账号，历史 cn / solo 谱系继续保留。
 * 国际版仍需独立 provider id 与目录，两地令牌不能混用；不将地区做成账号字段。
 *
 * ── 为什么回调必须落在本机 ────────────────────────────────
 * 上游对回调地址是**逐字正则校验**的（`^http://127.0.0.1:<port>/authorize$`），
 * 且回调里**没有 state** —— 一轮登录只能靠端口认回来。所以浏览器必须与网关同机；
 * 容器 / 远程部署形态收不到回调，只能在有浏览器的机器上登录后走粘贴那条路。
 */
const TRAE: ProviderConfig = {
  provider: 'trae',
  label: 'Trae',
  // Trae 桌面端的登录态在它自己的加密存储里（与 Accio / ZCode 同一处境），
  // 没有 auth.json 那种稳定可读的形态 —— 给了入口只会稳定失败。
  desktop: false,
  webLogin: {
    noteHtml: t('Open the official authorization page for <b>Trae</b> (<code>trae.cn</code>) and sign in with your Trae account: once authorized, the official page redirects back to a temporary port on <b>this machine</b>; a same-machine deployment receives it automatically, while Docker / remote panels should copy the callback URL from the address bar as prompted.'),
    button: t('Open Trae authorization page'),
    busyText: t('Waiting for Trae authorization…'),
    modes: [
      {
        value: 'embedded',
        label: t('Embedded window (recommended)'),
        hint: t('The embedded window will open; the account is added to the list automatically once authorization completes. On a remote panel you can paste the callback URL. Closing the window cancels the wait. The link is valid for 5 minutes; if it times out or is not clicked, it is voided (you can start again)'),
      },
      {
        value: 'external',
        label: t('System browser'),
        hint: t('The authorization page will open in your system default browser (reusing the Trae account already signed in there); on the same machine the account is added automatically when done. For Docker / remote panels, paste the callback URL from the address bar as prompted.'),
      },
    ],
    // 回调只落在本机端口（上游逐字校验 127.0.0.1），浏览器与网关不同机时
    // 官方页面跳回的地址回不到容器 —— 网页端必须说明「可能要粘回来」
    webHint: WEB_LOGIN_PASTE_HINT,
  },
  manualTitle: t('Enter credentials'),
  manualNoteHtml: t('accessToken is Trae\'s <code>Cloud-IDE-JWT</code> (three dot-separated segments), and refreshToken enables automatic renewal on expiry (this provider <b>rotates refreshToken on every use</b>: once exchanged, the old one is void, so do not refresh the same account from two programs at once).<br>When pasting manually, fill in <b>machineId / deviceId</b> as well: upstream ties them to the device public key uploaded at sign-in to identify the device, and swapping in a new pair from nowhere triggers the <code>2xxxx</code> family of device-binding rejections.<br>Without these three, the proper route is "Web login" above; for container / remote deployments, paste the callback URL from the address bar as prompted.'),
  fields: [
    { key: 'accessToken', label: 'accessToken', rows: 3, placeholder: t('Cloud-IDE-JWT (three dot-separated segments)') },
    { key: 'refreshToken', label: 'refreshToken', rows: 2, optional: true, placeholder: t('Optional; required for automatic renewal on expiry') },
    { key: 'uid', label: t('User ID'), optional: true, placeholder: t('Optional; used for deduplication and the display name (when blank, the gateway asks upstream once)') },
    { key: 'machineId', label: 'machineId', optional: true, placeholder: t('Optional; UUID form, lives and dies with the credential') },
    { key: 'deviceId', label: 'deviceId', optional: true, placeholder: t('Optional; digits only (upstream accepts only 8–24 digits)') },
    { key: 'name', label: t('Label'), optional: true, placeholder: t('Optional; defaults to the nickname') },
  ],
}

/**
 * Loomy（讯飞）：短信登录 + 粘贴 session。
 *
 * ── 三处与别家不同（都有上游依据，别照抄别家）────────────────
 *   1. 短信链路是**另一套**（端点 `/api/session/login/loomy/sms/*`、中间态字段
 *      叫 `msgid` 而不是 `deviceId`、请求带 HMAC-SHA1 签名头）——
 *      由 `ui/sms-login.js` 的 SMS_PROFILES 与壳侧桥接按 provider 分派；
 *   2. 粘贴的是**登录 session**（它同时是模型网关的 token，不是 JWT）；
 *   3. **没有续期**：session 约 14 天，过期只能重新登录（所以不渲染
 *      「刷新 Token」那一类控件）。
 */
const LOOMY: ProviderConfig = {
  provider: 'loomy',
  label: 'Loomy',
  fileImport: JSON_CREDENTIAL_FILE,
  smsLogin: {
    noteHtml: t('Sign in with the phone number bound to your Loomy account: click "Get code" and enter it. This is the easiest way in for this provider.'),
  },
  manualNote: t('What you paste is the login session (which is also the model gateway token). Loomy has no renewal API; the session is valid for about 14 days, so just sign in again after it expires.'),
  fields: [
    { key: 'session', label: 'session', rows: 3, placeholder: t('Login session (a long string)') },
    { key: 'userId', label: 'userId', optional: true, placeholder: t('Optional; the userid on the iFlytek side') },
    { key: 'phone', label: t('Phone number'), optional: true, placeholder: t('Optional; for display') },
    { key: 'name', label: t('Label'), optional: true, placeholder: t('Optional; defaults to the masked phone number or userId') },
  ],
  // 登录态在客户端自己的加密存储里（没有 auth.json 那种稳定可读的文件形态），
  // 不提供「导入桌面端登录态」（与 Accio / ZCode 同一处境，理由见各自配置注释）。
  desktop: false,
}

/**
 * KukuAI（百度文库「库库 AI / GenFlowPro」）：一家一个 provider，没有地区之分。
 *
 * ── 凭证形态 ────────────────────────────────────────────────
 * 三个 Cookie 就能认证：`BDUSS` / `STOKEN` / `gfprotpl=genflowpro`（后一个
 * 恒附加，不用填）。粘贴支持三种形态（与后端 `kuku::credentials` 同口径）：
 *   1. 整段 Cookie 头（`BDUSS=…; STOKEN=…; …`）；
 *   2. Cookie 编辑器导出的 `[{name, value}, …]` JSON 数组；
 *   3. `{BDUSS, STOKEN}` JSON 对象。
 *
 * ── 导入桌面端 ──────────────────────────────────────────────
 * 客户端是 Electron 套壳，登录态在 `%APPDATA%\baidugenflowpro\Network\Cookies`
 * （SQLite **明文**，无需解密）。客户端运行时该文件被独占锁，导入失败时
 * 提示先关闭客户端（占用冲突的两段式处置与 CatPaw 导入同一交互）。
 *
 * ── 没有刷新机制 ────────────────────────────────────────────
 * BDUSS 是百度通行证登录态，上游没有刷新接口，过期后重新导入/重新粘贴即可
 * （与 CatPaw 同一处境，所以不渲染「刷新 Token」那一类控件）。
 */
const KUKU: ProviderConfig = {
  provider: 'kuku',
  label: 'KukuAI',
  // 主站登录（最终方案）：打开 kuku.baidu.com，用户像平时用网页版一样登录
  // （手机验证码 / 扫码都由官方页面自己处理风控 —— 不再在登录页上做任何
  // 自动化）。登录成功后壳侧检测到 Cookie 里出现 BDUSS，自动读取**完整会话**
  // （含 HttpOnly 的 BAIDUID_BFESS 等）交回网关，窗口自动关闭、账号落列表。
  webLogin: {
    noteHtml: t('Sign in to the KukuAI main site with your Baidu account on the login page that opens (SMS code or QR scan both work); the account is added to the list automatically on success. The gateway exchanges it with Baidu for a KukuAI business session token (requires the KukuAI client installed on this machine).'),
    button: t('Open web login'),
    busyText: t('Waiting for KukuAI sign-in…'),
    // 这条链靠壳侧读本机客户端的 Cookie 落定（网页端没有壳，见 web_shim.rs
    // 的登录状态机），因此文案只描述「登录页已打开」，不承诺完成方式
    webHint: WEB_LOGIN_HINT,
  },
  manualTitle: t('Paste Cookie'),
  manualNote: t('BDUSS is required; paste the **full Cookie** (a Cookie editor can export it in one click) — the **PTOKEN must be kept**, as the gateway uses it to exchange for a KukuAI business session token from Baidu; without it, model refresh reports "Not signed in". A full Cookie header, a JSON array exported by a Cookie editor, or a {BDUSS, STOKEN} object are all supported. KukuAI has no refresh mechanism, so once the login state expires, just paste it again.'),
  fields: [
    { key: 'cookie', label: 'Cookie', rows: 3, placeholder: t('BDUSS=…; STOKEN=… (or the JSON exported by a Cookie editor)') },
    { key: 'name', label: t('Label'), optional: true, placeholder: t('Optional; defaults to the account uk') },
  ],
  desktop: true,
  desktopHint: t('Read this machine\'s KukuAI client login state (%APPDATA%\\baidugenflowpro\\Network\\Cookies), must already be logged in to the KukuAI client'),
  desktopNote: t('Reads the local client\'s current login state (plaintext Cookie, no decryption needed). The Cookies file is exclusively locked while the client runs, so close the KukuAI client before importing.'),
  // 「导入凭证文件」：**二进制** SQLite（文件名就叫 Cookies、没有后缀），
  // 原生文件读 base64 传 authFileB64；软件导出的 JSON 走统一单账号回流。
  // 与 desktop 那条的关键差别：网关读的是用户选中文件的**副本**（先解成临时
  // 文件再解析），不碰客户端正在用的那份 —— 因此这里无需先关闭客户端，
  // 文案要把这一点说清，否则用户会以为与上面那条同一个坑。
  fileImport: {
    binary: true,
    // 文件名无扩展名、文件类型也无从按扩展名筛：空串 = 选择器不过滤（全都能选）
    accept: '',
    note: t('Select KukuAI the client\'s Cookies file (%APPDATA%\\baidugenflowpro\\Network\\Cookies): it is a **binary** SQLite database (no extension, not JSON). Import reads a copy of the file you selected and does not compete with a running client, so **the client does not need to be closed first**. After import, credentials land directly in the gateway and can forward independently.",'),
  },
}

/**
 * Command Code（`api.commandcode.ai`）：一家一个 provider，没有地区之分
 * （规格 §9：参考里只有这一个主机，没有 cn / intl 双域名、没有 region 头）。
 *
 * ── 凭证是一枚 `user_` 前缀的 API Key，粘贴式 ─────────────────
 * 来源是官方 CLI 的 `~/.commandcode/auth.json` 或网页 studio
 * （commandcode.ai/studio）。粘贴形态很杂（裸 key / 带成对引号 /
 * `Authorization: Bearer …` 整行 / key 后面还跟着别的字段），网关会自己扫描
 * `user_` 剥成裸 key —— 整行直接粘就行，不用手工裁剪。
 *
 * ── 无续期、无桌面端导入 ───────────────────────────────────
 * key 是静态的：上游没有 refresh 接口、也不会过期，失效只能重新获取（所以
 * 不渲染「刷新 Token」那一类控件）。本机 `~/.commandcode/auth.json` 是 CLI
 * 自己的凭证文件，不是可导入的「客户端登录态」，没有那条路径（与 Loomy /
 * MonkeyCode 同一处境）。
 *
 * ── 添加时的一次线上探活 ───────────────────────────────────
 * 网关在添加时会打一次**轻量探活**（`GET /alpha/billing/credits`）：只有拿到
 * 401/403 才判 key 无效，网络失败 / 5xx / 未知一律放行 —— 失败不阻断添加，
 * 只作提示（不让一次上游抖动把用户挡在门外）。
 */
const COMMANDCODE: ProviderConfig = {
  provider: 'commandcode',
  label: t('Command Code'),
  fileImport: JSON_CREDENTIAL_FILE,
  desktop: false,
  manualTitle: t('Paste API Key'),
  manualNote: t('Paste the Command Code API Key (starts with user_; get it from the official CLI\'s ~/.commandcode/auth.json or the web studio). Pasting the whole line / with quotes / with the Bearer prefix all work; the gateway strips it down to the bare key itself. The key does not expire and there is no renewal API, so if it stops working just get a new one. When adding, the gateway runs a lightweight liveness check; a failure does not affect adding.'),
  fields: [
    { key: 'apiKey', label: 'apiKey', rows: 3, placeholder: t('API Key starting with user_ (you can also paste the whole Authorization: Bearer … line)') },
    { key: 'name', label: t('Label'), optional: true, placeholder: t('Optional; defaults to the key suffix') },
  ],
}

/**
 * Antigravity（Google 的 AI IDE，本次接它提供的 Gemini 模型）。
 *
 * ── 网页登录（Google OAuth 授权码 + loopback）───────────────
 * 后端适配器本地拼授权地址（`accounts.google.com`，6 个 scope +
 * `access_token=offline` / `prompt=consent`），完成后浏览器回到**网关自己的**
 * loopback 端口 `/oauth-callback`（redirect_uri 就是网关端口拼出来的，见
 * `providers::antigravity::oauth`）。回调落在网关，因此内嵌窗口与系统浏览器
 * 两种方式都能用（与 CatPaw 同款）；远程 / Docker 场景下浏览器到不了容器内
 * 的 loopback 端口时，可把地址栏整条回调 URL 粘回登录弹窗。
 *
 * ── 粘贴 Google refresh token（仍保留的手工入口）──────────────
 * 已从别处（官方 IDE / 参考实现）拿到 refresh_token 的用户可以直接粘贴；
 * 令牌以 `1//` 开头，直接粘贴含 `refresh_token=` 的整行也行（网关会归一化，
 * `1//` 本体留着）。
 *
 * ── 有自动续期 ─────────────────────────────────────────────
 * access_token 约一小时过期（Google 侧的 expires_in），refresh_token 是长寿命
 * 的主凭证：网页登录与粘贴式都会在添加时打一次 Google token 端点做**真实校验**
 * 并顺手拿到 access token，之后到期由网关自动续期 —— 令牌丢了只能重新走
 * Google 授权。
 *
 * ── projectId 可以留空 ─────────────────────────────────────
 * `projectId`（`cloudaicompanionProject`）是聊天请求必需、但每个 Google 账号
 * 各不相同的值（不在 token 里）：留空时网关会 best-effort 自动发现
 * （`loadCodeAssist` → 无则 `onboardUser`），失败不阻断添加，刷模型清单时会
 * 再试。`email` 同样可选，只用于展示与身份识别（网页登录会自己取 userinfo）。
 *
 * ── 没有「导入桌面端登录态」────────────────────────────────
 * 登录态在 Antigravity IDE 自己的存储里，没有 `auth.json` 那种稳定可读的
 * 形态（与 Accio / ZCode / Loomy 同一处境）。
 */
const ANTIGRAVITY: ProviderConfig = {
  provider: 'antigravity',
  label: t('Antigravity'),
  desktop: false,
  // 网页登录：Google 账号授权页 + loopback 回调。回调落在网关自己的端口，
  // 所以两种打开方式都可用（会复用系统浏览器里已登录的 Google 账号）。
  webLogin: {
    noteHtml: t('Sign in with your Google account on the login page that opens (Antigravity\'s authorization page); the account is added to the list automatically once authorization completes.'),
    button: t('Open Antigravity web login'),
    busyText: t('Waiting for Google authorization…'),
    modes: [
      { value: 'embedded', label: t('Embedded window (recommended)'), hint: t('Opens in the embedded window; added to the list when done; closing it cancels the wait') },
      { value: 'external', label: t('System browser'), hint: t('Opens in the system browser (reuses the Google account already signed in); added to the list when done') },
    ],
  },
  manualTitle: t('Paste Google refresh token'),
  manualNoteHtml: t('Paste Antigravity\'s Google refresh token (starts with <code>1//</code>; pasting the whole line containing <code>refresh_token=</code> also works).<br>This is a manual entry besides web login (for example, exported from the official IDE you are already signed in to). The refresh token does not expire, and the gateway renews the access token automatically when it expires; if the token is lost, you can only go through Google authorization again, so keep it safe.'),
  fields: [
    { key: 'refreshToken', label: 'refreshToken', rows: 2, placeholder: t('Google refresh token starting with 1// (you can also paste the whole refresh_token=… line)') },
    { key: 'projectId', label: 'projectId', optional: true, placeholder: t('Optional (cloudaicompanionProject); if blank, the gateway discovers it automatically') },
    { key: 'email', label: 'email', optional: true, placeholder: t('Optional; the account email, used for display and to recognize the identity (the same account is recognized even after switching to a new token)') },
    { key: 'name', label: t('Label'), optional: true, placeholder: t('Optional; defaults to the email or token suffix') },
  ],
}

/**
 * OfficeAce（华为云果办 / OfficeClaw）：一家一个 provider，没有地区之分。
 *
 * ── 两条来源（都落在后端 `add_officeace_account`）──────────────
 * ① **浏览器登录（自助 OAuth）**：网关向云端要 state、拼授权地址，用户在浏览器里
 *    登华为云，网关**轮询云端**取授权码并换到网关凭据 —— **不需要装 OfficeAce
 *    桌面端**（云端回调页不往本机跳，所以壳侧也不用开回调监听）。
 * ② **粘贴凭据**：从别的机器（或桌面端 `models.json`）拿 `baseUrl` 与
 *    `modelAppKey`/`modelAppSecret` 贴进来。只给这三个也能转发（网关凭据不过期）；
 *    控制面临时凭据（`accessKeyId` 等）可选。
 *
 * ── 没有「导入桌面端登录态」这条后端路径 ──────────────────────
 * 与 CodeArts 同：凭据只能靠网页登录或粘贴，所以显式 `desktop: false`
 * （不写的话桌面壳里会多出一个选了之后什么都没有的分段）。
 */
const OFFICEACE: ProviderConfig = {
  provider: 'officeace',
  label: t('OfficeAce'),
  fileImport: JSON_CREDENTIAL_FILE,
  desktop: false,
  webLogin: {
    // 整段是一个 t() 键：扫描器只认字符串字面量（拼接串提不到键），所以不拆行
    noteHtml: t('Open the Huawei Cloud authorization page and sign in: once finished, the gateway automatically <b>polls the cloud</b> for the authorization code and adds the account to the list (no OfficeAce desktop app required, and no need to paste a callback URL back manually).'),
    button: t('Open the authorization page'),
    busyText: t('Waiting for OfficeAce sign-in…'),
    // 两种打开方式都可用：授权码是网关**轮询云端**取回的（云端回调页不往本机
    // 跳），壳侧不开回调监听，因此浏览器在哪台机器上都不影响判定。
    modes: [
      {
        value: 'embedded',
        label: t('Embedded window (recommended)'),
        hint: t('The embedded window will open; the account is added to the list automatically after sign-in. Closing the window cancels the wait'),
      },
      {
        value: 'external',
        label: t('System browser'),
        hint: t('The authorization page will open in your system default browser (reusing the Huawei Cloud account already signed in there); the gateway polls the cloud for the authorization code, so it does not matter which machine the browser runs on. The account is added to the list automatically once done'),
      },
    ],
    // 授权码由网关轮询云端取回，浏览器在哪都一样
    webHint: WEB_LOGIN_HINT,
  },
  manualTitle: t('Paste gateway credentials'),
  manualNoteHtml: t('Grab three items from another machine (or from the OfficeAce desktop app\'s <code>models.json</code>) and paste them here: the model gateway base URL (<code>baseUrl</code>, shaped like <code>https://modelgw-…/v2</code>) plus <code>modelAppKey</code> / <code>modelAppSecret</code>. These three are enough to forward requests (the gateway credential never expires); the control-plane temporary credentials (<code>accessKeyId</code> and so on) are optional — leaving them blank disables balance queries and auto-renewal but does not affect forwarding.'),
  fields: [
    // 字段名与 URL 示例不包 t()（各语言相同，与 CodeArts 的 JSON 示例同一处置）
    { key: 'baseUrl', label: t('Model gateway base URL'), placeholder: 'https://modelgw-…huaweicloud-agentarts.com/v2' },
    { key: 'modelAppKey', label: t('Gateway key'), placeholder: 'model_app_key' },
    { key: 'modelAppSecret', label: t('Gateway secret'), placeholder: 'model_app_secret' },
    { key: 'name', label: t('Label'), optional: true, placeholder: t('Optional; defaults to “OfficeAce”') },
  ],
}

/**
 * OrcaTerm（腾讯云遨驰终端桌面客户端）：一家一个 provider，没有地区之分。
 *
 * ── 凭证形态 ────────────────────────────────────────────────
 * 微信 OAuth 登录后的一枚 JWT（`oauth_access_token`，约 2 小时时效），
 * `Authorization: Bearer` 直达上游。粘贴支持（与后端 `orcaterm::credentials`
 * 同口径）：裸 token、`Authorization: Bearer …` 整行、`ot_session=…`（Web 版
 * Cookie 名），或整个 `data.bin` 的 JSON —— 网关会自己剥出 token。
 *
 * ── 三条添加路径 ────────────────────────────────────────────
 * 网页登录（官方 OAuth 授权页微信扫码，回调交回网关换令牌）/
 * 导入桌面端登录态 / 粘贴 accessToken。三条最终都归一成同一枚 JWT
 * （见后端 `providers::orcaterm::login` 与 `api::accounts` 的 OrcaTerm 分支）。
 *
 * ── 导入桌面端 ──────────────────────────────────────────────
 * 客户端是 Tauri 应用，登录态在 `%APPDATA%\com.orcaterm-desktop.app\data.bin`
 * （**明文 JSON**，无需解密）。导入即复制当前令牌进账号记录，之后由网关接管。
 *
 * ── 有真续期接口 ───────────────────────────────────────────
 * 上游 `OAuthRefreshToken` 传旧 token 即可换新（2026-10-16 实测），因此账号
 * 添加后由网关自动续期，用户不需要跟着 2 小时时效手动换令牌（与 KukuAI /
 * CatPaw「过期只能重登」不同）。令牌丢了（被顶号 / 上游拒认）才需要重新导入。
 */
const ORCATERM: ProviderConfig = {
  provider: 'orcaterm',
  label: 'OrcaTerm',
  fileImport: { ...JSON_CREDENTIAL_FILE, accept: '.json,.bin,application/json' },
  // 网页登录（后端 providers::orcaterm::login）：打开**官方 OAuth 授权页**，
  // 它直接跳到微信二维码；扫完上游把浏览器带回自定义协议
  // `orcaterm://oauth_callback?session_id=…`，壳侧截下这一跳交回网关换令牌。
  webLogin: {
    noteHtml: t('Scan the WeChat QR code on the <strong>official authorization page</strong> that opens (the QR code appears right away). The account is added to the list automatically after a successful scan.'),
    button: t('Open web login'),
    hint: t('Opens in an embedded window; the account is added automatically once you have scanned — closing the window cancels the wait'),
    busyText: t('Waiting for OrcaTerm sign-in…'),
    // 网页端：授权页把浏览器带到 orcaterm:// 自定义协议，网页端没有壳去接它；
    // 结果由网关侧轮询落定，所以只承诺「新标签页打开」
    webHint: WEB_LOGIN_HINT,
  },
  desktop: true,
  desktopHint: t('Read this machine\'s OrcaTerm client login state (%APPDATA%\\com.orcaterm-desktop.app\\data.bin), must already be logged in to the OrcaTerm client'),
  desktopNote: t('Reads the OAuth token the local client is currently signed in with (plain-text JSON, no decryption needed). After importing, the gateway renews it automatically — no need to follow the client\'s 2-hour expiry by hand.'),
  manualTitle: t('Paste accessToken'),
  manualNoteHtml: t('Paste OrcaTerm\'s OAuth accessToken (a JWT with three dot-separated parts; available from the desktop client\'s <code>data.bin</code> or the web version\'s <code>ot_session</code> cookie). Pasting a whole line containing <code>Bearer</code> / <code>ot_session=</code> is fine too — the gateway strips it down to the bare token. After adding, the gateway renews it automatically (the upstream has a renewal API), so you do not need to follow the 2-hour expiry by hand.'),
  fields: [
    { key: 'accessToken', label: 'accessToken', rows: 3, placeholder: t('JWT (three dot-separated parts; you can also paste a whole Bearer line or the data.bin content)') },
    { key: 'name', label: t('Label'), optional: true, placeholder: t('Optional; leave blank to use the account nickname or userId') },
  ],
}

/**
 * 智谱清言（`chatglm.cn`）。逆向来源：客户端 2.0.7 的 `app.asar` +
 * `agentmore` 线上包（见后端 `providers::chatglm` 的模块头）。
 *
 * ── 三条添加路径（本家在这一点上最特别）────────────────────────
 *   1. **访客模式**（`guestLogin`）：不需要手机号也不需要扫码，后台空 body
 *      打一次 `guest/access` 就换到一对令牌 + 一个新设备号。本家「零配置就能用」
 *      的入口 —— 代价是**没有积分体系**（余额 / 签到 / 活动领取对访客如实跳过），
 *      说明里必须写清；
 *   2. **粘贴令牌**：从浏览器开发者工具复制 `chatglm_token` 的 cookie 值
 *      （或整条 JWT）。适合已有登录账号的用户 —— 这类账号有积分，签到可用；
 *   3. **导入本机登录态**：读本机客户端 Cookies。与 KukuAI 的明文列不同，
 *      本家那份是 **DPAPI 加密**的（Chromium os_crypt），只有 Windows 能解，
 *      因此 `desktopWindowsOnly`。
 *
 * ── 有真续期接口 ────────────────────────────────────────────
 * `POST /chatglm/user-api/user/refresh`（refresh_token 换新令牌，实测可用）。
 * 访客令牌失效时网关会用**同一个设备号**重新换发（设备号是身份的一部分，
 * 换掉等于换一个新访客）。
 */
const CHATGLM: ProviderConfig = {
  provider: 'chatglm',
  label: 'Zhipu Qingyan',
  fileImport: JSON_CREDENTIAL_FILE,
  guestLogin: {
    noteHtml: t('No sign-in needed: click the button below and the gateway will obtain a <strong>guest credential</strong> from Zhipu Qingyan directly and add it to the account list (you can fill in a note name first). The guest credential can be used for chat right away; the free quota is issued by the upstream per device.<br><br><strong>Note</strong>: a guest account <strong>has no points system</strong> — its member points, daily sign-in points and activity claim endpoints all require sign-in, so those capabilities are skipped for guests (no readings are shown on the accounts page or in the check-in center). To get points, use "Fill in credentials" to paste a signed-in account token, or "Import desktop sign-in state".'),
    button: t('Add a guest account'),
    busyText: t('Requesting a guest credential…'),
    namePlaceholder: t('Optional; a note name for this guest account'),
  },
  desktop: true,
  desktopWindowsOnly: true,
  desktopHint: t('Read this machine\'s Zhipu Qingyan client login state (%APPDATA%\\chatglm\\Network\\Cookies), must already be logged in to the client'),
  desktopNote: t('Read the current sign-in state of the local client. The client\'s cookies are DPAPI-encrypted (Chromium os_crypt), so this is <strong>Windows-only</strong>; the file may be unreadable while the client is running — quit the client and retry (or use "Fill in credentials" to paste a token instead).'),
  manualTitle: t('Paste an access token'),
  manualNoteHtml: t('Paste a Zhipu Qingyan access token (a JWT with three dot-separated parts). How to get it: open <code>chatglm.cn</code> in a browser, sign in, then copy the value of the <code>chatglm_token</code> cookie from Application → Cookies in the developer tools (pasting the whole <code>chatglm_token=…;</code> line works too). It is verified against the upstream once after pasting, and the account is only saved if that passes; signed-in accounts get points and sign-in capability, and the gateway renews it automatically afterwards (the upstream has a renewal API).'),
  fields: [
    {
      key: 'accessToken',
      label: 'accessToken',
      rows: 3,
      placeholder: t('JWT (three dot-separated parts; a whole chatglm_token=… line works too)'),
    },
    { key: 'name', label: t('Label'), optional: true, placeholder: t('Optional; defaults to the account nickname') },
  ],
}

/**
 * ModelScope（魔搭）的两个站点：一家一个 provider，粘贴 `ms-` 令牌。
 *
 * ── 一枚令牌两用（这是本家最省事的地方）──────────────────────
 * 同一个 `ms-` 令牌既作 `Authorization: Bearer` 调推理，又作
 * `Cookie: m_session_id=<令牌>` 读账号身份 / 魔粒余额 / 每日签到（实测确认）。
 * 所以凭据只有一个字段，用户从魔搭的「访问令牌」页复制一次就够。
 *
 * ── 两个站点是两套独立账号 ─────────────────────────────────
 * 国内站（modelscope.cn）与国际站（modelscope.ai）账号体系分开：令牌打错站点
 * 会得到 401，所以两家各一张卡、各填各的令牌。
 *
 * ── 没有续期、没有桌面端导入 ───────────────────────────────
 * 令牌长期有效（上游没有 refresh 接口），失效只能重新获取；魔搭没有桌面客户端
 * （与 Loomy / MonkeyCode 同一处境），因此不渲染「导入桌面端登录态」。
 *
 * ── 添加时的一次探活（会拦下添加）──────────────────────────
 * 令牌不透明、解不出身份，所以网关添加前必须打一次上游换回账号身份 ——
 * 令牌无效 / 站点点错会在添加的那一刻就说清楚，而不是等第一次转发才失败。
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
    // 没有桌面客户端，也就没有可导入的登录态文件（理由见上方注释）
    desktop: false,
    manualTitle: t('Paste API token'),
    // 整段是一个 t() 键：扫描器只认字符串字面量（拼接串提不到键），所以不拆行
    manualNoteHtml: t('Paste a ModelScope API token (starts with <code>ms-</code>; create one on the "Access tokens" page of {site}).<br>{siteNote}<br>The same token serves <b>two purposes</b>: the gateway uses it both to call inference and to read the account identity, the credits balance and the daily check-in. The token is long-lived and there is no renewal API — if it stops working, just create a new one and paste it again.', { site, siteNote }),
    fields: [
      // 字段名不包 t()（各语言相同，与别家同一处置）
      { key: 'accessToken', label: 'accessToken', rows: 3, placeholder: t('API token starting with ms-') },
      { key: 'name', label: t('Label'), optional: true, placeholder: t('Optional; defaults to the upstream account nickname') },
    ],
  }
}

const MODELSCOPE = modelscopeForm({
  provider: 'modelscope',
  label: t('ModelScope (China)'),
  site: 'modelscope.cn',
  siteNote: t('This is for the China site (modelscope.cn) account: paste the China site\'s token. The international site (modelscope.ai) is another provider with its own account; tokens are not interchangeable between the two.'),
})

const MODELSCOPE_INTL = modelscopeForm({
  provider: 'modelscope-intl',
  label: t('ModelScope (International)'),
  site: 'modelscope.ai',
  siteNote: t('This is for the international site (modelscope.ai) account: paste the international site\'s token. The China site (modelscope.cn) is another provider with its own account; tokens are not interchangeable between the two.'),
})

/**
 * AtomCode（AtomGit / GitCode）：把 AtomGit 的免费 CodingPlan 接进本网关。
 *
 * ── 两条来源 ────────────────────────────────────────────────
 * ① **浏览器登录**（推荐）：网关向代理现取授权地址 → 用户在浏览器里登 AtomGit
 *    → 网关**轮询代理**判完成并取回令牌对（授权回调由代理自己在
 *    `acs.atomgit.com/callback` 终结，**不往本机跳**，所以壳侧不需要开回调监听）。
 * ② **粘贴令牌对**（后备）：`accessToken` + `refreshToken`。⚠️ `refreshToken`
 *    强烈建议填 —— 本家的令牌**不是 JWT**（实测 7 天到期，读不到 `exp`），
 *    没有它账号无法自动续期，到期只能重新登录
 *    （见 `providers/atomcode/credentials.rs` 的模块头）。
 *
 * `desktop: false`：与 CodeArts / OfficeAce 同 —— 没有「导入桌面端登录态」这条
 * 后端路径（AtomCode 把令牌写在 `auth.toml` 里，且需要额外的机器指纹，不做这条入口）。
 */
const ATOMCODE: ProviderConfig = {
  provider: 'atomcode',
  label: t('AtomCode'),
  fileImport: JSON_CREDENTIAL_FILE,
  desktop: false,
  webLogin: {
    // 整段是一个 t() 键：扫描器只认字符串字面量（拼接串提不到键），所以不拆行
    noteHtml: t('Open the AtomGit authorization page and sign in: once finished, the gateway automatically <b>polls the broker</b> for the token and adds the account (the callback is terminated by the broker and never returns to this machine, so no callback URL needs to be filled in).'),
    button: t('Open the authorization page'),
    busyText: t('Waiting for AtomCode sign-in…'),
  },
  manualTitle: t('Paste token pair'),
  // 整段是一个 t() 键：扫描器只认字符串字面量（拼接串提不到键），所以不拆行
  manualNoteHtml: t('Paste the two values from the AtomCode page on AtomGit: <code>accessToken</code> and <code>refreshToken</code>. <b>The refresh token is strongly recommended</b> — this provider\'s token is not a JWT (it expires in about 7 days and carries no readable expiry), so an account without it cannot renew itself and must be re-authorised once it expires.'),
  fields: [
    // 字段名与示例值不包 t()（各语言相同，与 CodeArts 的 JSON 示例同一处置）
    { key: 'accessToken', label: t('Access token'), placeholder: 'access_token' },
    { key: 'refreshToken', label: t('Refresh token'), optional: true, placeholder: 'refresh_token' },
    { key: 'userId', label: t('User ID'), optional: true, placeholder: t('Optional; used to build the account identifier') },
    { key: 'name', label: t('Label'), optional: true, placeholder: t('Optional; defaults to “AtomCode”') },
  ],
}

/**
 * LobsterAI（网易有道「有道龙虾」`lobsterai.youdao.com`）：一家一个 provider，
 * 没有地区 / 额度池之分。
 *
 * ── 只有一条添加路径：浏览器登录（授权码 + 本机 loopback 回调）──────
 * 网关按 state 登记一轮登录并拼出授权地址，用户在门户授权（手机号 / 微信）后
 * 门户 302 回**网关自己的端口**：`http://127.0.0.1:<网关端口>/auth/callback`，
 * 网关拿授权码换令牌后落账号（见后端 `providers::lobsterai::oauth` 与
 * `core::login::lobsterai`）。回调路径是裸 `/auth/callback`（不是 `-lobsterai`
 * 后缀）：那是参考实现已验证的形状，门户是否对路径有白名单没有实测支撑。
 *
 * ── 没有「填写凭证」这条路径（后端如实拒绝，不是界面漏做）──────────
 * 续期要**换码那一刻**由上游下发的三件载荷（`uuid` + 两个 `keyfrom`），粘贴
 * 给不出它们（见后端 `account_store::lobsterai_accounts` 的模块头）。所以后端
 * `api::accounts` 对 LobsterAI 的粘贴入口**始终回 400**，这里也就不提供手填段。
 * `fields` 留空是被 `ProviderConfig` 逼的（该键在类型里是必填），不是漏写：
 * 本家唯一入口就是上面那条「网页登录」。
 *
 * ── 没有「导入桌面端登录态」这条路径 ──────────────────────────
 * 客户端登录态在它自己的存储里，没有 `data.bin` 那种稳定可读的明文形态
 * （与 Accio / ZCode 同一处境），所以显式 `desktop: false`。
 */
const LOBSTERAI: ProviderConfig = {
  provider: 'lobsterai',
  label: t('LobsterAI'),
  desktop: false,
  // 授权页把浏览器送回**本机端口**（与 Accio 同一形态）：同机自动完成，
  // 跨机（Docker / 远程面板）到不了网关 —— 本家没有「粘贴最终回调地址」那条
  // 兜底，所以说明里只讲同机这一条前提，不承诺粘贴通道。
  webLogin: {
    // 整段是一个 t() 键：扫描器只认字符串字面量（拼接串提不到键），所以不拆行
    noteHtml: t('Open the official <b>LobsterAI</b> sign-in page (<code>lobsterai.youdao.com</code>) and sign in with your LobsterAI account: once finished, the official page returns to this machine and the account is added automatically — the callback lands on <b>the gateway\'s own port</b>, so the browser must be on the same machine as the gateway; the auth code is only handed to the gateway locally and no plaintext token is shown in the UI.'),
    button: t('Open the LobsterAI sign-in page'),
    busyText: t('Waiting for LobsterAI sign-in…'),
    modes: [
      { value: 'embedded', label: t('Embedded window (recommended)'), hint: t('The embedded window will open; the account is added to the list automatically after sign-in. Closing the window cancels the wait') },
      { value: 'external', label: t('System browser'), hint: t('The sign-in page will open in your system default browser (reusing the LobsterAI account already signed in there); the account is added to the list automatically after sign-in, and closing the popup cancels the wait') },
    ],
    // 本家没有粘贴兜底：跨机时网页端只能靠用户自己在浏览器地址栏确认
    // 页面跳回去了，所以提示里只承诺「新标签页打开」
    webHint: WEB_LOGIN_HINT,
  },
  fields: [],
}

/** 内置家的表单块，顺序与旧 ADD_FORMS 一致（只影响 DOM 里的块顺序，不影响界面） */
export const BUILTIN_CONFIGS: ProviderConfig[] = [
  RACCOON,
  CATPAW,
  AUTOCLAW,
  AUTOCLAW_INTL,
  // Qoder 两个地区相邻（拆家后是两家 provider，与 AutoClaw / Accio / ZCode
  // 同一理由）：中国版在前，与存量账号的归属一致
  QODER,
  QODER_INTL,
  // 千问办公两版紧随其后（同一套协议栈的另一家产品，两版相邻）：
  // 国内版在前，与后端注册表 PROVIDERS 的排列一致
  QWENWORK,
  QWENWORK_INTL,
  // Cline 顺序即界面上「提供商」分段的顺序：免费池在前（无门槛，更常用）
  clineForm({ provider: 'cline-free', label: 'Cline Free', poolNote: t(' (free quota pool; model names carry the cline-free/ prefix).') }),
  clineForm({ provider: 'cline-pass', label: 'Cline Pass', poolNote: t(' (subscription pool; model names carry the cline-pass/ prefix and the account needs the matching subscription).') }),
  // Accio 顺序：国际版在前（默认安装的版本）
  accioForm({ provider: 'accio', label: t('International edition'), site: 'www.accio.com', siteNote: t('The International and China editions are **two separate sets of accounts** (two sites of the same account system); credentials do not carry over.') }),
  accioForm({ provider: 'accio-cn', label: t('China edition'), site: 'www.accio-ai.com', siteNote: t('The China edition\'s sign-in site is www.accio-ai.com, a different site from the International edition\'s.') }),
  // ZCode 顺序：国内版在前（国内网络环境下更常被添加的那个，与后端注册表 PROVIDERS 的排列一致）
  zcodeForm({ provider: 'zcode', label: t('China edition'), site: 'open.bigmodel.cn', planNote: t('The China edition and **the International edition are two separate sets of accounts and plans**; credentials and claimed plans do not carry over between them.') }),
  zcodeForm({ provider: 'zcode-intl', label: t('International edition'), site: 'api.z.ai', planNote: t('The International edition\'s inference site is api.z.ai, a different site from the China edition\'s; plans are separate as well.') }),
  // CodeArts（华为云 AI 代码助手）：一家一个 provider，没有地区/额度池之分
  // （region 写死 cn-north-4，与 token 签发地必须一致）。
  CODEARTS,
  // Trae 当前登记国内账号；国际版接入时使用独立身份。
  TRAE,
  // Loomy（讯飞）：单一入口（手机验证码登录），排在末尾 —— 与后端注册表
  // PROVIDERS 的排列一致（新增的一家加在表尾）
  LOOMY,
  // KukuAI（百度文库库库 AI）：粘贴 Cookie / 导入本机登录态，排在末尾
  // （与后端注册表 PROVIDERS 的排列一致，2026-10 接入）
  KUKU,
  // MonkeyCode（长亭科技）的两个站点相邻（与 AutoClaw / Qoder 同一拆法）：
  // 国内版在前 —— 与后端注册表 PROVIDERS 的追加顺序一致（2026-10 接入）。
  MONKEYCODE,
  MONKEYCODE_INTL,
  // Command Code / Antigravity：单一入口、没有地区伴生，同样按后端注册表
  // PROVIDERS 的追加顺序排在表尾（后到居后，新增的家加在末尾）。
  COMMANDCODE,
  ANTIGRAVITY,
  // OfficeAce（华为云果办 / OfficeClaw）：浏览器登录（自助 OAuth）或粘贴网关
  //凭据，排在末尾（与后端注册表 PROVIDERS 的排列一致，2026-10 接入）
  OFFICEACE,
  // OrcaTerm（腾讯云遨驰终端桌面客户端）：粘贴 accessToken / 导入桌面端登录态，
  // 排在末尾（与后端注册表 PROVIDERS 的排列一致，2026-10 接入）
  ORCATERM,
// 智谱清言（chatglm.cn）：访客模式 / 粘贴令牌 / 导入本机登录态，排在末尾
  // （与后端注册表 PROVIDERS 的排列一致，2026-10 接入）
  CHATGLM,
  MODELSCOPE,
  MODELSCOPE_INTL,
  // AtomCode（AtomGit / GitCode）：浏览器登录 + 粘贴令牌对，排在末尾
  // （与后端注册表 PROVIDERS 的排列一致，后到居后）
  ATOMCODE,
  // LobsterAI（网易有道「有道龙虾」）：只有浏览器登录一条入口，排在末尾
  // （与后端注册表 PROVIDERS 的排列一致，后到居后）
  LOBSTERAI,
  DOUBAOWORK,
  DEEPSEEK_WEB,
  // Tabbit 浏览器（国内站 / 国际站）：两站账号不通用，成对相邻排列。
  TABBIT,
  TABBIT_INTL,
]

/** WorkBuddy 的块 id（结构特殊，单独一个组件） */
export const WORKBUDDY_PROVIDER = 'workbuddy'
/** 自定义提供商的块 id（后注册块的 provider 取值） */
export const CUSTOM_PROVIDER = 'custom'

/**
 * WorkBuddy 在**添加账号入口**上的展示名：中性品牌名，不带地区。
 *
 * ── 为什么不用注册表里的名字（拆家后是「WorkBuddy 国内版」）──────
 * 这个入口是**两版共用**的：第 1 步只有一张卡，卡片里的「账号版本」分段负责
 * 切国内版 / 国际版（同一个 `WorkBuddyBlock` 的两个选项，见 add-provider-pick
 * 的说明）。名字带「国内版」而用户当场就能切到国际版，读起来就是自相矛盾 ——
 * 卡片与弹窗标题都因此用品牌名。
 *
 * 两处**都**走这个常量而不是各写一份：卡片与标题是同一个入口的两个步骤，
 * 名字不一致时用户会以为点了别的东西。
 *
 * ── 别处仍用注册表名 ────────────────────────────────────────
 * 账号页徽章、模型页、Key 页用的是注册表名（带地区）：那些地方要能分清两家，
 * 而这里一家代表两版。别家不需要这一层 —— AutoClaw / Accio / ZCode 的两版
 * 是**两张卡**，各自的注册名就是各自的身份。
 */
export const WORKBUDDY_ENTRY_LABEL = 'WorkBuddy'

export function configOf(providerId: string): ProviderConfig | undefined {
  return BUILTIN_CONFIGS.find(item => item.provider === providerId)
}

/* ─── 添加方式（分段项）─────────────────────────
 *
 * 文案要短（并排一行，太长会把弹窗挤到换行），详细说明留在各段自己的标题与正文里。
 * 露出哪些项由 methodsOf 按各家配置裁剪。
 *
 * `oauth` 单独一项而不并进「网页登录」：对用户来说两者都是「跳去官方页面登录」，
 * 但交互不同 —— 网页登录点一下就开窗口（等待态由 web-login.js 统一管），而
 * AutoClaw 国际版要先在本弹窗里弹一个阿里云滑块让用户拖完才能拿到授权地址。
 * 并进去会让那个引擎多出「有些家要先跑一段验证码」的分支，且两者的发起时序与
 * 按钮布局都不一样（这一项是两个变体各一个按钮）。
 *
 * `file` 紧跟 `manual` 而不并进 `desktop`：两者都是「导入本机登录态」，但
 * `desktop` 建的是「转发时实时读本机固定路径」的记录（只有本机能用），`file`
 * 是把用户手选的凭证文件解析后落库（可独立转发，文件可来自任意机器 / 备份）。
 * 失败处置与文案也各不相同（后者要选文件、要预检 JSON），因此各占一项。 */
export const ADD_METHODS = [
  { id: 'guest', label: t('Guest mode (no sign-in)') },
  { id: 'oauth', label: t('Web login (Zai / Google)') },
  { id: 'sms', label: t('SMS code login') },
  { id: 'web', label: t('Web login') },
  { id: 'manual', label: t('Enter credentials') },
  { id: 'file', label: t('Import credential file') },
  { id: 'desktop', label: t('Import desktop login state') },
] as const

export type MethodId = (typeof ADD_METHODS)[number]['id']

/**
 * 壳的编译目标平台（'macos' / 'windows' / 'linux' / 'web'），由桥接脚本注入。
 *
 * 浏览器直开（没有壳）时拿不到它，退回空串 —— 此时按「不裁剪」处理：
 * 浏览器直连网关本来就用不了这些壳侧功能，多显示一个选项不会误导谁，
 * 而误裁掉一个**本来可用**的功能会让 Windows 用户莫名其妙少一项。
 */
export const platform = (): string => shared().workbuddyDesktop?.platform || ''

/**
 * 桌面端导入在这一家、这个平台是否可用。
 *
 * `desktopWindowsOnly` 的家（AutoClaw / 智谱清言）要按平台裁掉：它们读本机
 * 客户端的登录态，而那份数据是 Electron / Chromium 的 **DPAPI 密文**
 * （AutoClaw 的 `%APPDATA%/AutoClaw/auth.json`、智谱清言客户端 Cookies 的
 * os_crypt 列），要先过 `CryptUnprotectData` 才能解出密钥 —— DPAPI 只有
 * Windows 有。
 *
 * ── 判据是「不是 Windows」而不是「是 macOS」────────────────────
 * 原来这里写的是 `platform() === 'macos'`，那是双平台时代的写法；第三端
 * （Linux 桌面壳）进来之后必须反过来判 —— 否则 Linux 会**照常列出**这个入口，
 * 用户点下去只能拿到「解密失败」，比不显示更糟。所以判据统一成
 * 「除 Windows 外一律收起」，将来再多一个桌面平台也自动正确。
 *
 * 其余几家（小浣熊 `~/.box-agent`、CatPaw `~/.meituan-catpaw`、Cline
 * `~/.cline`）读的都是 `HOME` 下的明文 JSON，macOS / Linux 上照样能导入，
 * 因此不受这一条影响（见各 ProviderConfig 的 desktopNote）。
 */
export function desktopImportAvailable(config: ProviderConfig): boolean {
  if (config.desktop === false) return false
  // 网页端（headless 托管面板注入 platform='web'）：没有本机桌面客户端可读
  const current = platform()
  if (current === 'web') return false
  if (config.desktopWindowsOnly && current !== 'windows') return false
  return true
}

/** 网页登录那一段只在某个地区露出时的判据（当前没有家用，结构上保留） */
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

/** 手填表单标题：默认「填写凭证添加」，小浣熊沿用原文案 */
export const manualTitleOf = (config: ProviderConfig): string => config.manualTitle || t('Enter credentials to add')
/** 说明中的行内标记优先，否则按纯文本渲染（React 的文本节点天然转义） */
export const manualNoteOf = (config: ProviderConfig): { html?: string; text?: string } =>
  (config.manualNoteHtml ? { html: config.manualNoteHtml } : { text: config.manualNote || '' })

/** 手填段主按钮文案 */
export const addButtonTextOf = (config: ProviderConfig): string =>
  config.addButton || t('Add {name} account', { name: config.label })

/** 字段的 DOM id（inputKey 保留小浣熊既有的 refresh-input id，请求体键仍为 refreshToken） */
export const fieldIdOf = (config: ProviderConfig, field: FieldSpec): string =>
  `${config.provider}-${field.inputKey || field.key}-input`
