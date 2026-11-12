/**
 * by （valid / / valid / can ）。
 *
 * ── ：， not is client ──────────────────────
 * 「valid in 」 by not ：WorkBuddy `expiresAt`，
 * `tokenExpiresAt`，OrcaTerm `tokenExpireAt`（ 1 s），KukuAI no has 
 * this 。 this ** is up **， in and 1 ——
 * that is from （file ）， 1 is 「valid
 * 1 —」。
 *
 * in is ** in when `AccountFeatures`**， by after in 
 * `/api/session` `accountFeatures` up 1 down 。 need need 「 down + 」，
 * new up ， not client。
 *
 * ── ────────────────────────────────────────────────────
 * old not （`accountFeatures` no has this 1 ） when `FALLBACK`：
 * 、 has that 。 can ， not need as 1 。
 */

/** 1 parameter（ and `AccountFeatures` ， use to ） */
export type ProviderFacts = {
  /** valid ****（ by 1 can ）；empty = no has this */
  expiry: readonly string[]
  /** */
  identifier: readonly string[]
  /** valid（ no has to when ） */
  longLived: boolean
  /** this has no has can */
  usage: boolean
}

/**
 * （`provider id → `）， by `/api/session` `accountFeatures` 。
 *
 * not is React state： in （、），
 * will new 1 —— 1 new Map 。
 */
let declared: ReadonlyMap<string, ProviderFacts> = new Map()
/** up 1 1 use for （ use ：`/api/session` 1 1 ） */
let declaredSource: unknown = null

/**
 * down after 。
 *
 * is `AccountFeatures` serde （snake_case）。 by 
 * `FALLBACK` —— this not to 「 in not in 」。
 */
export function adoptProviderFacts(raw: unknown): void {
  if (!raw || typeof raw !== 'object' || Array.isArray(raw)) {
    declared = new Map()
    return
  }
  const next = new Map<string, ProviderFacts>()
  for (const [id, value] of Object.entries(raw as Record<string, unknown>)) {
    if (!id || !value || typeof value !== 'object' || Array.isArray(value)) continue
    next.set(id, normalize(value as Record<string, unknown>))
  }
  declared = next
}

/** 1 。 */
function normalize(raw: Record<string, unknown>): ProviderFacts {
  return {
    expiry: keyList(raw.expiry),
    identifier: keyList(raw.identifier, FALLBACK.identifier),
    longLived: raw.long_lived === true,
    usage: raw.usage === true,
  }
}

/** ：， 1 ， its 。 */
function keyList(value: unknown, fallback: readonly string[] = []): readonly string[] {
  if (typeof value === 'string') return value ? [value] : []
  if (!Array.isArray(value)) return fallback
  return value.filter((item): item is string => typeof item === 'string' && item.length > 0)
}

const FALLBACK: ProviderFacts = {
  expiry: ['expiresAt', 'tokenExpiresAt'],
  identifier: ['userId'],
  longLived: false,
  usage: false,
}

/**
 * parameter。：** → use **。
 *
 * `session` parameteroptional： use has will when ， will by use 
 * need not need new 1 （`/api/session` new 1 new for ）。
 */
export function providerFacts(
  id: string | undefined,
  session?: { accountFeatures?: Record<string, unknown> } | null,
): ProviderFacts {
  syncDeclared(session)
  return (id && declared.get(id)) || FALLBACK
}

/** will new 1 （ use ； no has will when ） */
function syncDeclared(session?: { accountFeatures?: Record<string, unknown> } | null): void {
  if (!session) return
  const raw = session.accountFeatures
  if (raw === declaredSource) return
  declaredSource = raw
  adoptProviderFacts(raw)
}

/** current（ use ； `providerFacts`） */
export function declaredProviderFacts(): ReadonlyMap<string, ProviderFacts> {
  return declared
}

/**
 * this can not can ： balanceQuery.scheme，。
 *
 * （`isGuest`） 1 false： no has （ up 
 * 1 401）， will to failure。 and `accounts-domain` 
 * 、 and after `usage_query::supports_usage` —— must 1 ，
 * otherwise and after return will for not up 。
 */
export function supportsUsage(account: {
  provider?: string
  balanceQuery?: unknown
  isGuest?: boolean
}): boolean {
  if (account.isGuest === true) return false
  const id = String(account.provider || '')
  if (id.startsWith('custom-')) {
    const scheme = (account.balanceQuery as { scheme?: unknown } | undefined)?.scheme
    return Boolean(scheme)
  }
  return providerFacts(id).usage
}

/** 1 to ms（seconds and ms in ， accounts-domain ） */
function expiryMillis(value: unknown): number {
  const raw = Number(value) || 0
  if (raw <= 0) return 0
  return raw < 1e11 ? raw * 1000 : raw
}

/**
 * valid when （ms，0 = no has ）。
 *
 * by ** 1 can ** ： new and old 
 * not （ is `expiresAt` / `tokenExpiresAt`）， in 。
 * is 「 can 」 not is 「 has 」—— 1 `null` not this after has 。
 */
export function expiryOf(account: {
  provider?: string
  expiresAt?: number
  tokenExpiresAt?: number
  [key: string]: unknown
}, session?: { accountFeatures?: Record<string, unknown> } | null): number {
  const facts = providerFacts(account.provider, session)
  for (const key of facts.expiry) {
    const value = expiryMillis(account[key])
    if (value > 0) return value
  }
  return 0
}

/** this has no has 「valid」 this （ not that 1 ） */
export function hasExpiry(provider: string | undefined): boolean {
  const facts = providerFacts(provider)
  return facts.expiry.length > 0 || facts.longLived
}

/**
 * this is not is **valid**（ no has to when ）。
 *
 * has 「valid」 that 1 ， is 「valid」 not is 「—」：
 * 「—」「 no has to when 」，valid「 not will to 」。
 */
export function isLongLivedCredential(provider: string | undefined): boolean {
  return providerFacts(provider).longLived
}
