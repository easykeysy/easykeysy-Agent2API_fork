type Translate = (key: string, params?: Record<string, string | number>) => string

type CredentialAccount = {
  provider?: string
  hasAccessToken?: boolean
  accessTokenExpiresAt?: number | null
  hasRefreshToken?: boolean
  refreshTokenExpiresAt?: number | null
  refreshExpiresAt?: number | null
  refreshTokenLongLived?: boolean
}

export type CredentialExpiryRow = {
  label: 'Access Token' | 'Refresh Token'
  expiresAt: number | null
  longLived: boolean
}

/**
 * has ** to ** （）。
 *
 * ── Why not can by 「 has accessToken and has refreshToken」 ────────
 * OAuth when has this ， that is ** 1 **：access token
 * to after by refresh token ， use 「 can use 」 this 1 。 by 
 * in will ， 1 。
 *
 * ── ────────────────────────────────────────────────────
 * has this ** when up not to when **， and must after ：
 * · codearts —— STS when （`accessTokenExpiresAt`） and new 
 * （`refreshTokenExpiresAt`， from refresh JWT ） is when ，
 * will for AK/SK/STS。
 * 1 （ has that 1 ）， has refreshToken。
 */
const DUAL_CREDENTIAL_PROVIDERS: readonly string[] = ['codearts']

export function credentialExpiryMillis(value: unknown): number | null {
  if (typeof value !== 'number' && typeof value !== 'string') return null
  if (typeof value === 'string' && !value.trim()) return null
  const number = Number(value)
  const millis = Number.isFinite(number) ? number < 1e11 ? number * 1000 : number : Date.parse(String(value))
  return Number.isFinite(millis) && millis > 0 ? millis : null
}

export function credentialExpiryRowsOf(account: CredentialAccount, accessExpiry: unknown): CredentialExpiryRow[] {
  // by ： no （ DUAL_CREDENTIAL_PROVIDERS）
  if (!DUAL_CREDENTIAL_PROVIDERS.includes(String(account.provider ?? ''))) return []
  if (account.hasAccessToken !== true || account.hasRefreshToken !== true) return []
  const refreshExpiry = credentialExpiryMillis(account.refreshTokenExpiresAt ?? account.refreshExpiresAt)
  // must has ： to when （ or not to ） when ， is 1 ，
  // that —— otherwise will 「Access Token 317 after / Refresh Token 317 after 」
  // this 1 。
  if (refreshExpiry === null && account.refreshTokenLongLived !== true) return []
  const accessMillis = credentialExpiryMillis(account.accessTokenExpiresAt) ?? credentialExpiryMillis(accessExpiry)
  if (refreshExpiry !== null && accessMillis !== null && refreshExpiry === accessMillis) return []
  return [
    { label: 'Access Token', expiresAt: accessMillis, longLived: false },
    { label: 'Refresh Token', expiresAt: refreshExpiry, longLived: refreshExpiry === null && account.refreshTokenLongLived === true },
  ]
}

export function credentialExpiryText(row: CredentialExpiryRow, t: Translate, now = Date.now()): string {
  if (row.longLived) return t('长期有效')
  if (row.expiresAt === null) return '—'
  const left = row.expiresAt - now
  if (left <= 0) return t('已过期')
  if (left < 3_600_000) return t('{n} 分钟后', { n: Math.max(1, Math.round(left / 60_000)) })
  if (left < 48 * 3_600_000) return t('{n} 小时后', { n: Number((left / 3_600_000).toFixed(1)) })
  return t('{n} 天后', { n: Math.floor(left / 86_400_000) })
}
