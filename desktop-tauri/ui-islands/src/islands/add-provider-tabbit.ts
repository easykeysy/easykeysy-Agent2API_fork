import type { ProviderConfig } from './add-account-configs'
import { t } from '../i18n'

/**
 * Tabbit （ web.tabbit.com）。
 *
 * ── Why Web not is Agent ──────────────────────────
 * 「client」 is 1 Chromium ， is Cookie JWT；
 * up is for ， no has use 。 and / DeepSeek 1 。
 *
 * ── Why not 「」──────────────────────────
 * Google OAuth to redirect_uri，
 * not to ； in 。 「 already has 」。
 *
 * ── Why has ────────────────────────────────────────────
 * key not 、 not use 、 not （ has ），
 * by 「」 then `tabbit` / `tabbit-intl`。
 */
export const TABBIT: ProviderConfig = {
  provider: 'tabbit',
  label: t('Tabbit 国内版'),
  desktop: false,
  smsLogin: {
    noteHtml: t('用绑定的手机号登录：点「获取验证码」，收到短信后填入下方并点「登录并添加」。这是国内站官方唯一的登录方式。'),
  },
  manualTitle: t('粘贴网页登录凭证'),
  manualNote: t('在 web.tabbit.com 登录后，从开发者工具 Application → Cookies 复制 token 与 user_id，或直接粘贴该站点请求的 Copy as cURL 整段。登录态有效期约 7 天，上游没有续期接口，过期后重新导入即可。Cookie 与 JWT 至少填一项。'),
  fields: [
    {
      key: 'cookie',
      label: t('网页 Cookie'),
      rows: 3,
      optional: true,
      placeholder: t('token=…; user_id=…（也可粘贴 Copy as cURL 整段）'),
    },
    {
      key: 'token',
      label: t('登录 JWT'),
      rows: 3,
      optional: true,
      placeholder: t('只拿到 JWT 时填这里；Cookie 里有 token 可留空'),
    },
    {
      key: 'unlockPro',
      label: t('解锁 Premium'),
      optional: true,
      placeholder: t('默认开启；填 0 关闭（发送「已设为默认浏览器」标记）'),
    },
    {
      key: 'clientVersion',
      label: t('客户端版本'),
      optional: true,
      placeholder: t('留空使用 1.17.16(10117016)'),
    },
    {
      key: 'name',
      label: t('备注名'),
      optional: true,
      placeholder: t('可选，留空自动生成'),
    },
  ],
  fileImport: {
    accept: '.json,.txt,text/plain,application/json',
    note: t('选择包含 Tabbit Cookie / JWT 的 JSON 或文本文件；也可以直接粘贴到上方表单。'),
  },
}

/**
 * Tabbit （ web.tabbit.ai）。
 *
 * can use 「 already has 」：Google not to ，
 * in up 403 （）。 this no has 。
 */
export const TABBIT_INTL: ProviderConfig = {
  provider: 'tabbit-intl',
  label: t('Tabbit 国际版'),
  desktop: false,
  manualTitle: t('粘贴网页登录凭证'),
  manualNote: t('在 web.tabbit.ai 登录后，从开发者工具 Application → Cookies 复制 token 与 user_id，或直接粘贴该站点请求的 Copy as cURL 整段。国际站没有可复刻的网页授权回调，手机号登录也处在上游灰度里，所以只能导入已有登录态。登录态有效期约 7 天，过期后重新导入。Cookie 与 JWT 至少填一项。'),
  fields: [
    {
      key: 'cookie',
      label: t('网页 Cookie'),
      rows: 3,
      optional: true,
      placeholder: t('token=…; user_id=…（也可粘贴 Copy as cURL 整段）'),
    },
    {
      key: 'token',
      label: t('登录 JWT'),
      rows: 3,
      optional: true,
      placeholder: t('只拿到 JWT 时填这里；Cookie 里有 token 可留空'),
    },
    {
      key: 'unlockPro',
      label: t('解锁 Premium'),
      optional: true,
      placeholder: t('默认开启；填 0 关闭（发送「已设为默认浏览器」标记）'),
    },
    {
      key: 'clientVersion',
      label: t('客户端版本'),
      optional: true,
      placeholder: t('留空使用 1.17.16(10117016)'),
    },
    {
      key: 'name',
      label: t('备注名'),
      optional: true,
      placeholder: t('可选，留空自动生成'),
    },
  ],
  fileImport: {
    accept: '.json,.txt,text/plain,application/json',
    note: t('选择包含 Tabbit Cookie / JWT 的 JSON 或文本文件；也可以直接粘贴到上方表单。'),
  },
}
