import { t } from '../i18n'
import type { AccountRecord } from './accounts-shared'

export function supportsTokenRefresh(account: AccountRecord): boolean {
  return account.desktop !== true && (account.canRefresh === true || account.hasRefreshToken === true)
}

export function tokenRefreshSkipReason(account: AccountRecord): string {
  if (account.desktop === true) return t('Update this live desktop session in the original client')
  if (account.needsRelogin === true) return accountReloginText(account)
  if (!supportsTokenRefresh(account)) return t('This account does not support token refresh or has no refresh credentials')
  return ''
}

export function accountReloginText(account: AccountRecord): string {
  return account.needsReloginReason?.trim()
    || t('Sign-in authorization is no longer valid. Sign in again; automatic renewal cannot restore an expired or revoked refresh token.')
}
