import type { ProviderConfig } from './add-account-configs'
import { t } from '../i18n'

/**
 * Tabbit 浏览器（国内站 web.tabbit.com）。
 *
 * ── 为什么归 Web 段而不是 Agent 段 ──────────────────────────
 * 它的「客户端」是一个 Chromium 浏览器，登录态是网页 Cookie 里的 JWT；
 * 上游是网页对话产品，没有原生工具调用。与智谱清言 / DeepSeek 网页同一形态。
 *
 * ── 添加账号为什么不提供「网页登录」──────────────────────────
 * 国际站的 Google OAuth 只把授权结果投递给它自己登记的 redirect_uri，网关
 * 接不到；国内站的手机号登录在灰度里。所以两个站点都走「粘贴已有登录态」。
 *
 * ── 为什么有两家 ────────────────────────────────────────────
 * 两站签名 key 不同、账号不通用、登录方式也不同（国内站有手机号登录），
 * 按仓库的「两套账号就拆家」规则拆成 `tabbit` / `tabbit-intl`。
 */
export const TABBIT: ProviderConfig = {
  provider: 'tabbit',
  label: t('Tabbit (China)'),
  desktop: false,
  smsLogin: {
    noteHtml: t('Sign in with your bound phone number: tap "Get code", enter the SMS code below and click "Sign in and add". This is the only official sign-in method for the China site.'),
  },
  manualTitle: t('Paste web login credentials'),
  manualNote: t('After signing in at web.tabbit.com, copy token and user_id from DevTools → Application → Cookies, or paste the whole Copy as cURL of a request to that site. The login state lasts about 7 days and the upstream has no refresh endpoint; re-import it after expiry. Provide at least one of Cookie or JWT.'),
  fields: [
    {
      key: 'cookie',
      label: t('Web Cookie'),
      rows: 3,
      optional: true,
      placeholder: t('token=…; user_id=… (you can also paste the whole Copy as cURL)'),
    },
    {
      key: 'token',
      label: t('Login JWT'),
      rows: 3,
      optional: true,
      placeholder: t('Fill this only if you have just the JWT; leave empty if the Cookie contains token'),
    },
    {
      key: 'unlockPro',
      label: t('Unlock Premium'),
      optional: true,
      placeholder: t('Enabled by default; enter 0 to disable (sends the "set as default browser" marker)'),
    },
    {
      key: 'clientVersion',
      label: t('Client version'),
      optional: true,
      placeholder: t('Leave empty to use 1.17.16(10117016)'),
    },
    {
      key: 'name',
      label: t('Label'),
      optional: true,
      placeholder: t('Optional; generated automatically if blank'),
    },
  ],
  fileImport: {
    accept: '.json,.txt,text/plain,application/json',
    note: t('Choose a JSON or text file containing the Tabbit Cookie / JWT; you can also paste it into the form above.'),
  },
}

/**
 * Tabbit 浏览器（国际站 web.tabbit.ai）。
 *
 * 国际站只能用「已有登录态」导入：Google 授权回调回不到网关，手机号登录
 * 在上游灰度里被 403 拒绝（参考实现实测）。所以这里没有网页登录入口。
 */
export const TABBIT_INTL: ProviderConfig = {
  provider: 'tabbit-intl',
  label: t('Tabbit (International)'),
  desktop: false,
  manualTitle: t('Paste web login credentials'),
  manualNote: t('After signing in at web.tabbit.ai, copy token and user_id from DevTools → Application → Cookies, or paste the whole Copy as cURL of a request to that site. The international site has no reproducible web-auth callback, and phone login is still in upstream rollout, so only an existing login state can be imported. The login state lasts about 7 days; re-import it after expiry. Provide at least one of Cookie or JWT.'),
  fields: [
    {
      key: 'cookie',
      label: t('Web Cookie'),
      rows: 3,
      optional: true,
      placeholder: t('token=…; user_id=… (you can also paste the whole Copy as cURL)'),
    },
    {
      key: 'token',
      label: t('Login JWT'),
      rows: 3,
      optional: true,
      placeholder: t('Fill this only if you have just the JWT; leave empty if the Cookie contains token'),
    },
    {
      key: 'unlockPro',
      label: t('Unlock Premium'),
      optional: true,
      placeholder: t('Enabled by default; enter 0 to disable (sends the "set as default browser" marker)'),
    },
    {
      key: 'clientVersion',
      label: t('Client version'),
      optional: true,
      placeholder: t('Leave empty to use 1.17.16(10117016)'),
    },
    {
      key: 'name',
      label: t('Label'),
      optional: true,
      placeholder: t('Optional; generated automatically if blank'),
    },
  ],
  fileImport: {
    accept: '.json,.txt,text/plain,application/json',
    note: t('Choose a JSON or text file containing the Tabbit Cookie / JWT; you can also paste it into the form above.'),
  },
}
