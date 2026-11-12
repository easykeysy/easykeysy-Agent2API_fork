/**
 * can before ： 、、token and 。
 *
 * and and after `core::capability::KEYS` 1 （ and in 
 * that ）；「」—— 、 can 、
 * from this ， 1 and 。
 *
 * ── （ can can is null）────────────────────────────────────
 * can ：true = supports / false = unsupported / null = ****（ up 
 * no has this ）。 can ： = / null = 。 and need 
 * 「」 and 「unsupported」 —— in 1 after ， use will 「 not 」
 * 「unsupported」。
 *
 * ── ────────────────────────────────────────────────
 * `ManageModel.capOverrides` use （ for has ；
 * 1 is use ， no has 「 up 」， that as empty）。
 * use to 1 、 use 「 / 」。
 */

import { t } from '../i18n'

/** can （ = and can ， and after 1 ） */
export const CAPABILITY_KEYS = [
  'maxInputTokens',
  'maxOutputTokens',
  'supportsToolCall',
  'supportsImages',
  'supportsVideo',
  'supportsReasoning',
] as const

export type CapabilityKey = (typeof CAPABILITY_KEYS)[number]

/** 1 can ： → （null / = ； 1 not ） */
export type Capabilities = Partial<Record<CapabilityKey, number | boolean | null>>

/** token （ by ） */
export const TOKEN_KEYS: readonly CapabilityKey[] = ['maxInputTokens', 'maxOutputTokens']
/** can （ by ） */
export const BOOLEAN_KEYS: readonly CapabilityKey[] = [
  'supportsToolCall', 'supportsImages', 'supportsVideo', 'supportsReasoning',
]

/** and tooltip use */
export const CAPABILITY_LABELS: Record<CapabilityKey, string> = {
  maxInputTokens: t('上下文窗口'),
  maxOutputTokens: t('最大输出 Token'),
  supportsToolCall: t('工具调用'),
  supportsImages: t('图片识别'),
  supportsVideo: t('视频识别'),
  supportsReasoning: t('支持思考'),
}

/** can up （ has ） */
export const CAPABILITY_SHORT: Partial<Record<CapabilityKey, string>> = {
  supportsToolCall: t('工具'),
  supportsImages: t('图片'),
  supportsVideo: t('视频'),
  supportsReasoning: t('思考'),
}

/** token limit（ and after `core::capability::MAX_TOKEN_VALUE` ） */
export const MAX_TOKEN_VALUE = 100_000_000

/** this is not is token */
export function isTokenKey(key: CapabilityKey): boolean {
  return TOKEN_KEYS.includes(key)
}

/**
 * 1 （ and after `capability::normalize_value` 1 ）：
 * token 1~1 （）， bool； its to `null`
 * （= ， use by 「 this 1 no has 」）。
 */
export function normalizeCapability(key: CapabilityKey, value: unknown): number | boolean | null {
  if (isTokenKey(key)) {
    const number = typeof value === 'number' ? value : Number(value)
    if (!Number.isFinite(number) || number < 1 || number > MAX_TOKEN_VALUE) return null
    return Math.trunc(number)
  }
  return typeof value === 'boolean' ? value : null
}

/** any for （ after response / ） 1 1 can （ and ） */
export function normalizeCapabilities(raw: unknown): Capabilities {
  const result: Capabilities = {}
  if (!raw || typeof raw !== 'object') return result
  const source = raw as Record<string, unknown>
  for (const key of CAPABILITY_KEYS) {
    const value = normalizeCapability(key, source[key])
    if (value !== null) result[key] = value
  }
  return result
}

/** 1 （ → ； by CAPABILITY_KEYS） */
export function normalizeOverrides(raw: unknown): CapabilityKey[] {
  if (!Array.isArray(raw)) return []
  return CAPABILITY_KEYS.filter(key => raw.some(item => item === key))
}

/**
 * 1 （ `capabilities` for ） can 。
 *
 * not is `ManageModel`： not import `models-custom-source`
 *（ that import ， to use will ）。 in this 
 * —— after to 「、null = 」， to ， 1 after 
 * 「 has / no 」， not 。
 */
export function capabilitiesOf(model: { capabilities?: unknown } | null | undefined): Capabilities {
  return normalizeCapabilities(model?.capabilities)
}

/**
 * token ：`200000` → `200K`、`1000000` → `1M`。
 * 1 、 `.0`（`196608` → `196.6K`）； tooltip to 。
 */
export function formatTokens(value: unknown): string {
  const number = Number(value)
  if (!Number.isFinite(number) || number <= 0) return '—'
  const trim = (scaled: number): string => scaled.toFixed(1).replace(/\.0$/, '')
  if (number >= 1_000_000) return `${trim(number / 1_000_000)}M`
  if (number >= 1000) return `${trim(number / 1000)}K`
  return String(Math.round(number))
}

/** （tooltip to ，`196608` → `196,608`） */
export function exactTokens(value: unknown): string {
  const number = Number(value)
  if (!Number.isFinite(number) || number <= 0) return t('未声明')
  return Math.round(number).toLocaleString('en-US')
}

/** ：'on' supports / 'off' unsupported / 'unset' */
export type CapabilityState = 'on' | 'off' | 'unset'

export function capabilityState(value: unknown): CapabilityState {
  if (value === true) return 'on'
  if (value === false) return 'off'
  return 'unset'
}

/**
 * this ** can **（ optional； = ）。
 *
 * ── and `models-reasoning.ts` that 「 etc 」 not is 1 ──────────
 * that 1 is ** use to ** ， use 8 
 * （`off` / `none` / `minimal` / `low` / `medium` / `high` / `xhigh` / `max`），
 * use can ； this 1 is **supports** ， this directory
 * （e.g. ZCode GLM-5.3 `low` / `high` / `max` ， and not ），
 * **** —— is 「 this up 」， use not up 。
 *
 * （ no has this ） when return `null`： use ** not ** this 1 ， not is 1 
 * empty —— 「 up no 」 and 「 1 no has 」 for use is 。
 */
export function reasoningLevelsOf(
  model: {
    reasoningLevels?: unknown; reasoningDefaultLevel?: unknown
    reasoningFixedLevel?: unknown; reasoningSource?: unknown
  } | null | undefined,
): { levels: string[]; defaultLevel: string; fixedLevel: string; source: string } | null {
  const raw = model?.reasoningLevels
  if (!Array.isArray(raw)) return null
  const levels = raw.filter(
    (level): level is string => typeof level === 'string' && level.trim() !== '',
  )
  if (!levels.length) return null
  const preset = model?.reasoningDefaultLevel
  const fixed = model?.reasoningFixedLevel
  const source = model?.reasoningSource
  return {
    levels, defaultLevel: typeof preset === 'string' ? preset.trim() : '',
    fixedLevel: typeof fixed === 'string' ? fixed.trim() : '',
    source: typeof source === 'string' ? source : '',
  }
}

/** tooltip （ and use ） */
export function capabilityTip(
  key: CapabilityKey,
  value: unknown,
  overridden: boolean,
  source: CapabilitySource | '' = '',
): string {
  const label = CAPABILITY_LABELS[key]
  const suffix = overridden ? t('（已被手动覆盖，弹窗里可恢复继承）') : ''
  const origin = capabilitySourceText(source)
  if (isTokenKey(key)) {
    if (value === null || value === undefined) return t('{label}：上游未声明{suffix}', { label, suffix })
    return t('{label}：{value}（对下游声明的精确值）{origin}{suffix}', { label, value: exactTokens(value), origin, suffix })
  }
  const state = capabilityState(value)
  if (state === 'on') return t('{label}：支持{origin}{suffix}', { label, origin, suffix })
  if (state === 'off') return t('{label}：不支持{origin}{suffix}', { label, origin, suffix })
  return t('{label}：上游未声明（下游会按不支持处理）{suffix}', { label, suffix })
}

/**
 * 1 can ****（ after in ， `core::model_meta`）。
 *
 * `upstream` up （ this ）
 * `modelsdev` up no 、 by models.dev （ ）
 * `user` use in 「 can 」 （）
 *
 * empty = no has （）。 can ， up must ——
 * 1 1 、 not ， use no this not this 。
 */
export type CapabilitySource = 'upstream' | 'modelsdev' | 'user'

/** （ after no / not → empty） */
export function capSourceOf(
  model: { capSources?: unknown } | null | undefined,
  key: CapabilityKey,
): CapabilitySource | '' {
  const raw = (model?.capSources as Record<string, unknown> | undefined)?.[key]
  return raw === 'upstream' || raw === 'modelsdev' || raw === 'user' ? raw : ''
}

/** （ in after ；`upstream` not to —— 「 up 」 by use by ） */
export function capabilitySourceText(source: CapabilitySource | ''): string {
  if (source === 'modelsdev') return t('（由 models.dev 补齐）')
  if (source === 'user') return t('（手动设定）')
  return ''
}

/**
 * when ： 1 to （「」 not ——
 * use in this to is need ）。
 *
 * ： empty / = `null`（ to ）；
 * ：'inherit' → null、'on' → true、'off' → false。
 */
export type CapabilityDraft = Record<CapabilityKey, string>

export function patchFromDraft(draft: CapabilityDraft): Record<CapabilityKey, number | boolean | null> {
  const patch = {} as Record<CapabilityKey, number | boolean | null>
  for (const key of CAPABILITY_KEYS) {
    const raw = draft[key]
    if (isTokenKey(key)) {
      patch[key] = normalizeCapability(key, raw.trim())
    } else {
      patch[key] = raw === 'on' ? true : raw === 'off' ? false : null
    }
  }
  return patch
}

/** current → （ to ， to 'on'/'off'/'inherit'） */
export function draftFromCapabilities(capabilities: Capabilities): CapabilityDraft {
  const draft = {} as CapabilityDraft
  for (const key of CAPABILITY_KEYS) {
    const value = capabilities[key]
    if (isTokenKey(key)) {
      draft[key] = value === null || value === undefined ? '' : String(value)
    } else {
      draft[key] = value === true ? 'on' : value === false ? 'off' : 'inherit'
    }
  }
  return draft
}
