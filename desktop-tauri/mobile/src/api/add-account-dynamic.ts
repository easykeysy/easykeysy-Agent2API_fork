/** 只使用现有 methods/start/wait/callback/cancel 契约；不猜提供商协议。 */
import { apiCall, ApiError } from './client'
import { serverIdentity } from '../lib/servers'
import { isSuspendedProvider } from '../../../ui-shared/provider-availability'
import { locale, t } from '../i18n'

export type DynamicLoginStart = {
  state?: string
  flow?: string
  provider?: string
  url?: string
  authUrl?: string
  instructions?: string
  manualCallback?: boolean
  callbackInput?: 'url' | 'code'
  localOnly?: boolean
  pollIntervalMs?: number
  browserMode?: 'choice' | 'external' | 'plugin' | 'none'
}

export type DynamicLoginWait = {
  pending?: boolean
  done?: boolean
  error?: string
  instructions?: string
  hint?: string
  session?: { accountUid?: string; accountId?: string; nickname?: string; provider?: string; proxySaved?: boolean; warning?: string } | null
}

export type LoginInput = { provider: string; methodIndex: number; inputs: Record<string, string>; proxy?: unknown }

function requireAvailableProvider(provider: string): void {
  if (isSuspendedProvider(provider)) {
    throw new Error(t('This provider is no longer in the runtime catalog. Return to the list and select again.'))
  }
}

export function readDynamicMethods(provider: string, signal?: AbortSignal): Promise<unknown> {
  requireAvailableProvider(provider)
  return apiCall(`/api/session/login/methods?provider=${encodeURIComponent(provider)}&locale=${encodeURIComponent(locale())}`, { signal })
}

export function startDynamicLogin(input: LoginInput): Promise<DynamicLoginStart> {
  requireAvailableProvider(input.provider)
  return apiCall('/api/session/login/start', { method: 'POST', body: input })
}

export function waitDynamicLogin(state: string): Promise<DynamicLoginWait> {
  return apiCall(`/api/session/login/wait?state=${encodeURIComponent(state)}`)
}

export async function cancelDynamicLogin(state: string): Promise<void> {
  const result = await apiCall<{ canceled?: boolean }>('/api/session/login/cancel', { method: 'POST', body: { state } })
  if (result?.canceled !== true) throw new Error(t('Sign-in cancellation was not confirmed. Refresh the status and retry.'))
}

export function submitDynamicCallback(state: string, callbackUrl: string): Promise<{
  accountId?: string; accepted?: boolean; continue?: boolean; nextUrl?: string; retryable?: boolean; hint?: string
}> {
  // URL / 裸 code 原文必须保留，不能 trim、拆参数或重建 state。
  return apiCall('/api/session/login/callback', { method: 'POST', body: { state, callbackUrl } })
}

export function dynamicLoginError(error: unknown): string {
  return error instanceof Error ? t(error.message) : t('Unknown error')
}

export async function addDynamicApiAccount(input: LoginInput & { name?: string }): Promise<string> {
  requireAvailableProvider(input.provider)
  const identity = serverIdentity()
  const result = await apiCall<{ account?: { id?: string; provider?: string }; proxySaved?: boolean; warning?: string }>(
    '/api/accounts', { method: 'POST', body: input },
  )
  if (serverIdentity() !== identity) throw new DOMException('The server has switched', 'AbortError')
  if (!result?.account?.id || result.account.provider !== input.provider) {
    throw new Error(t('The sign-in bridge did not return a completed account or session. Try again.'))
  }
  const warning = typeof result.warning === 'string' ? t(result.warning) : ''
  return saveDynamicAccountProxy(result.account.id, input.proxy, result.proxySaved === true, warning, identity)
}

export async function saveDynamicAccountProxy(
  accountId: string, proxy: unknown, saved: boolean, warning = '', identity = serverIdentity(),
): Promise<string> {
  if (saved || proxy === undefined) return warning
  if (serverIdentity() !== identity) throw new DOMException('The server has switched', 'AbortError')
  let proxyWarning = t('The older gateway did not confirm a combined proxy save. The compatibility endpoint was used; check the proxy selection in account settings.')
  try {
    await apiCall(`/api/accounts/${encodeURIComponent(accountId)}`, {
      method: 'PATCH', body: { proxy: proxy ?? { source: 'direct' } },
    })
  } catch (error) {
    if (serverIdentity() !== identity) throw error
    proxyWarning = t('Account added, but saving the proxy selection was not confirmed. Check it in account settings.')
  }
  return [warning, proxyWarning].filter(Boolean).join('\n')
}

export function methodsUnavailable(error: unknown): boolean {
  return error instanceof ApiError && error.status === 501
}
