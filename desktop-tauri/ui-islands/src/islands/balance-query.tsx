/**
 * Agent2API · ****（， and after 
 * `providers::custom::balance` `Scheme` 1 1 for ）。
 *
 * ** not is **（ not to window ）： is （set、
 * add） use and —— `.tsx` after is as has 1 JSX，
 * `index.tsx` glob will import ， that is none use empty。
 *
 * ── Why need this 1 ────────────────────────────────────────
 * is 「 this 1 is 」 （ in create when ），
 * use ** not **； need is 1 ： this need need use 、
 * 。 that has before （ / / is ），
 * after 「 」 this 1 （`custom_accounts::BALANCE_QUERY_FIELDS`）。
 *
 * ── not use ──────────────────────────────────────────────
 * use ** API Key**（sub2api / DeepSeek / OpenCode Go / 
 * ）—— `fields` is empty， 1 （ use 「DeepSeek not use 
 * 」 ）；add and 「set」 this ， not 。
 *
 * ── use not （）────────────────
 * · add（`add-custom-provider.tsx`）： is **** 
 * （ in draft ， when by DOM id ）， that use 
 * `balanceFieldId` id、；file `BalanceQuerySection`
 * 「set」 use （ that is React ）。
 * · ：**empty = need **、**empty = not **（ by ，
 * null to after ）—— and apiKey / that 1 1 。
 */

import * as React from 'react'
import { Button, Input, Field, FieldLabel } from '@ui'

import { t } from '../i18n'

/** id（ and after `custom_providers::BALANCE_SCHEME_*` 1 ，） */
export type BalanceScheme =
  | 'sub2api'
  | 'new_api'
  | 'deepseek'
  | 'opencode'
  | 'sensenova'
  | 'zhipu'
  | 'zhipu_zai'

/** （ and after `custom_accounts::BALANCE_QUERY_FIELDS` 1 ） */
export type BalanceFieldKey = 'userId' | 'accessToken' | 'username' | 'password' | 'token'

/** `balanceQuery`（ after ； has has this ） */
export type BalanceQueryPublic = {
  scheme?: string
  userId?: string
  username?: string
  /** ** has in **， not （ and apiKey 1 ） */
  hasAccessToken?: boolean
  hasPassword?: boolean
  hasToken?: boolean
}

export type BalanceField = {
  key: BalanceFieldKey
  label: string
  /** ：、empty = not 、 by （ null） */
  secret?: boolean
  placeholder: string
  hint?: string
}

export type BalanceSchemeSpec = {
  /** （ and after `Scheme::label` ） */
  label: string
  /** 「」（ that ， and OmniProxy 1 ） */
  endpoint: string
  /** （ in ； need 、） */
  note: string
  /** this need use （empty = use API Key） */
  fields: BalanceField[]
}

/**
 * 。 OmniProxy （`balance_*` 1 ），
 * by （"set" → 「set → set」）；
 * （zhipu_zai） is ，OmniProxy no has this 1 。
 */
export const BALANCE_SCHEME_SPECS: Record<BalanceScheme, BalanceSchemeSpec> = {
  sub2api: {
    label: 'Sub2API',
    endpoint: '{base}/v1/usage',
    note: t('用这条账号的 API Key 查询，不需要额外配置。'),
    fields: [],
  },
  new_api: {
    label: 'NewAPI',
    endpoint: '{base}/api/user/self + /api/status',
    note: t('用 NewAPI 站点的个人访问令牌查询账户余额（不是这条账号的上游 API Key）。令牌在站点的「个人设置」里生成，只有旧版 / 定制站点才需要额外填用户 ID。'),
    fields: [
      {
        key: 'accessToken',
        label: t('NewAPI 个人访问令牌'),
        secret: true,
        placeholder: t('在 NewAPI 站点「个人设置」里生成的访问令牌'),
        hint: t('必填；留空 = 不修改，清除用右边的按钮'),
      },
      {
        key: 'userId',
        label: t('NewAPI 用户 ID'),
        placeholder: t('旧版 / 定制站点才需要，新版可留空'),
        hint: t('可留空：新版请求头只认令牌；旧版会把用户 ID 放进 New-Api-User 头'),
      },
    ],
  },
  deepseek: {
    label: 'DeepSeek',
    endpoint: '{base}/user/balance',
    note: t('用这条账号的 API Key 查询，不需要额外配置。'),
    fields: [],
  },
  opencode: {
    label: 'OpenCode Go',
    endpoint: '{base}/v1/usage',
    note: t('用这条账号的 API Key 查询订阅用量，不需要额外配置；读数展开成 5 小时 / 每周 / 每月三个窗口。'),
    fields: [],
  },
  sensenova: {
    label: '商汤日日新',
    endpoint: 'platform.sensenova.cn 控制台',
    note: t('用商汤日日新控制台的账号密码查询（与推理用的 API Key 不是同一套）。网关会自动换取登录票并缓存，过期后自动重登，不需要手动维护。'),
    fields: [
      {
        key: 'username',
        label: t('账号'),
        placeholder: t('商汤日日新控制台的登录账号'),
        hint: t('必填'),
      },
      {
        key: 'password',
        label: t('密码'),
        secret: true,
        placeholder: t('商汤日日新控制台的登录密码'),
        hint: t('留空 = 不修改，清除用右边的按钮'),
      },
    ],
  },
  zhipu: {
    label: '智谱',
    endpoint: 'bigmodel.cn 控制台',
    note: t('用 bigmodel.cn 控制台的访问令牌查询账户余额（人民币元）。令牌是登录后浏览器 Cookie `bigmodel_token_production` 的值：开发者工具 → 应用（Application）→ Cookie 里复制。'),
    fields: [
      {
        key: 'token',
        label: t('访问令牌'),
        secret: true,
        placeholder: t('bigmodel_token_production 的值'),
        hint: t('必填；留空 = 不修改，清除用右边的按钮'),
      },
    ],
  },
  // （z.ai）： 1 ， not —— 
  // 「API Key SIGN 」（ after `zhipu_sign_jwt`）， Key / not 。
  // is need that API Key， this 1 ** no has **（deepseek ）。
  zhipu_zai: {
    label: '智谱（z.ai 国际）',
    endpoint: 'api.z.ai 控制台',
    note: t('用这条账号的 API Key 生成签名后查询账户余额（美元），不需要额外配置。要求 API Key 是 z.ai 的「id.secret」形态（控制台生成的 Key 就是）。'),
    fields: [],
  },
}

/** → （unknown / to null = not this 1 ） */
export function balanceSpecOf(scheme: string | undefined | null): BalanceSchemeSpec | null {
  if (!scheme) return null
  return (BALANCE_SCHEME_SPECS as Record<string, BalanceSchemeSpec | undefined>)[scheme] ?? null
}

/** is already config（ `has*` ） */
export function balanceFieldConfigured(
  query: BalanceQueryPublic | null | undefined,
  field: BalanceField,
): boolean {
  if (!field.secret) return false
  if (field.key === 'accessToken') return query?.hasAccessToken === true
  if (field.key === 'password') return query?.hasPassword === true
  if (field.key === 'token') return query?.hasToken === true
  return false
}

/**
 * this is ****（ will ； not after 「not configured」，
 * in not is failure）。
 *
 * and after for （`balance.rs` ）：
 * · sub2api / deepseek / opencode / zhipu_zai → has API Key（ that is ， not in ）；
 * · new_api → ；sensenova → + ；zhipu → 。
 * （ after ） not this 1 ： after 。
 */
export function balanceQueryConfigured(query: BalanceQueryPublic | null | undefined): boolean {
  const scheme = String(query?.scheme || '')
  if (scheme === 'new_api') return query?.hasAccessToken === true
  if (scheme === 'sensenova') {
    return String(query?.username || '').trim() !== '' && query?.hasPassword === true
  }
  if (scheme === 'zhipu') return query?.hasToken === true
  // its API Key： this not （ up 「not configured」 by to ）
  return true
}

/* ─── and 「」（set use ；add DOM ）── */

/**
 * → 。
 *
 * ** 1 empty**： has `has*` 、 no has （ after to ，
 * and ）—— is empty ，empty = not 。
 * （`userId` / `username`）， use 。
 */
export function balanceQueryDraftOf(
  query: BalanceQueryPublic | null | undefined,
): Record<string, string> {
  return {
    userId: String(query?.userId || ''),
    username: String(query?.username || ''),
    accessToken: '',
    password: '',
    token: '',
  }
}

/**
 * → `PATCH /api/accounts` `balanceQuery` for 。
 * `null` = this 1 no has can （ not this ）。
 *
 * and after `normalize_balance_query_patch` 1 ：
 * · ：**empty = not **（「」 by ， null）；
 * · ：（ from ， no is → after none；
 * use empty = this 1 ， is ）。
 */
export function readBalanceQueryChanges(
  scheme: string | undefined | null,
  values: Record<string, string>,
): Record<string, string> | null {
  const spec = balanceSpecOf(scheme)
  if (!spec || !spec.fields.length) return null
  const patch: Record<string, string> = {}
  for (const field of spec.fields) {
    const next = String(values[field.key] ?? '').trim()
    if (field.secret) {
      if (next) patch[field.key] = next
      continue
    }
    patch[field.key] = next
  }
  return Object.keys(patch).length ? patch : null
}

/* ─── ：set 「set」 / add use ──── */

/**
 * 1 （）。
 *
 * `configured` for has （ `has*`）： already config when 
 * 「 already config，empty then not 」 and to 「」 by —— and CatPaw 、
 * API Key 。
 */
export function BalanceFieldRow({
  field, value, configured, onValue, onClear, clearBusy, idPrefix,
}: {
  field: BalanceField
  value: string
  configured: boolean
  onValue: (next: string) => void
  /** already config （ null to after ）； not to this */
  onClear?: () => void
  clearBusy?: boolean
  /** DOM id before （ use id not can ） */
  idPrefix: string
}): React.ReactElement {
  const id = `${idPrefix}-${field.key}`
  return (
    <Field orientation='horizontal' className='field-row mt-2.5'>
      <FieldLabel htmlFor={id}>{field.label}</FieldLabel>
      <Input id={id} className='min-w-[220px]' type={field.secret ? 'password' : 'text'}
        autoComplete={field.secret ? 'new-password' : 'off'}
        placeholder={configured ? t('已配置，留空则不修改') : field.placeholder}
        title={field.hint} value={value}
        onChange={event => onValue(event.currentTarget.value)} />
      {configured && onClear ? (
        <Button variant='outline' size='sm' disabled={clearBusy === true} onClick={onClear}
          title={t('清除已配置的 {label}', { label: field.label })}>{t('清除')}</Button>
      ) : null}
    </Field>
  )
}

/**
 * 1 （ + ）。`values` ** in ** 
 * （empty = not ）；`configured` is 「 already config」。
 */
export function BalanceQuerySection({
  spec, values, configured, onValue, onClear, clearBusy, idPrefix,
}: {
  spec: BalanceSchemeSpec
  values: Record<string, string>
  configured: Record<string, boolean>
  onValue: (key: BalanceFieldKey, next: string) => void
  onClear?: (key: BalanceFieldKey) => void
  clearBusy?: boolean
  idPrefix: string
}): React.ReactElement {
  return (
    <>
      <p className='detail mt-2.5'>
        {t('余额查询方案：{label}', { label: spec.label })}
        <span className='ml-1 text-subtle'>{spec.endpoint}</span>
        {' — '}{spec.note}
      </p>
      {spec.fields.map(field => (
        <BalanceFieldRow key={field.key} field={field} idPrefix={idPrefix}
          value={values[field.key] || ''} configured={configured[field.key] === true}
          onValue={next => onValue(field.key, next)}
          onClear={onClear ? () => onClear(field.key) : undefined}
          clearBusy={clearBusy} />
      ))}
    </>
  )
}
