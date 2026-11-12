/**
 * and HTTP client。
 *
 * ── ， 1 ────────────────────────────────────
 * · **PWA / **： by ，`/api/*` is for path，server use 
 * HttpOnly cookie will —— and 1 。
 * · ** App**：， origin is ，request
 * 。request not will cookie，must request
 * （`x-panel-token` / `x-panel-refresh`），and when need server 
 * response（`x-panel-auth-mode: body`，server `tokens_in_body`）。
 *
 * path in 「」 and 「 from 」， in `request()`
 * 1 ， not 。
 *
 * ── by ──────────────────────────────────────────
 * lib/servers.ts： is by ， will after 401。
 *
 * ── 401 ────────────────────────────────────────────
 * · `error.type === 'panel_login_required'` → already will no ：
 * refresh，failure to ；
 * · its 401 → no Key： to 。
 */

import {
  activeServerId,
  apiBase,
  clearServerAuth,
  isPackaged,
  readServerAuth,
  readServerKey,
  serverIdentity,
  storeServerAuth,
  storeServerKey,
  type PanelAuth,
} from '../lib/servers'

/** API 1 error */
export class ApiError extends Error {
  status: number
  /** after `error.type`（`panel_login_required` ）， no has then as empty */
  type: string
  constructor(message: string, status: number, type = '') {
    super(message)
    this.name = 'ApiError'
    this.status = status
    this.type = type
  }
}

/** for path（PWA down return， to by ） */
export function url(path: string): string {
  if (!isPackaged()) return path
  const base = apiBase()
  if (!base) throw new ApiError('还没有配置服务器地址', 0)
  return `${base}${path}`
}

/** for path（PWA down return， to by ） */
export function resolveUrl(path: string): string {
  return url(path)
}

/**
 * current will （ to 「 not can apiCall」 use use ，e.g. SSE ）。
 *
 * and {@link apiCall} `buildHeaders` is 1 —— 1 ，
 * 「」 in not up will not 。
 */
export function authHeaders(extra?: Record<string, string>): Record<string, string> {
  return buildHeaders(extra)
}

/** （ for 401 when use ）。successreturn true */
export function refreshPanelAuth(): Promise<boolean> {
  return tryRefresh()
}

/**
 * to before （ to not `apiCall` use use ，e.g. SSE for and file up ）。
 *
 * `apiCall` already ； this fetch path no has ， need in request before 
 * 1 ，otherwise will in access that 1 1 401 —— that is need 
 * （ `ensureFreshSession` ）。 not 。
 */
export function refreshPanelAuthIfExpiring(): Promise<void> {
  return ensureFreshSession()
}

export function readApiKey(): string {
  return readServerKey(activeServerId())
}

export function storeApiKey(value: string): void {
  storeServerKey(activeServerId(), value)
}

export function readPanelAuth(): PanelAuth | null {
  return readServerAuth(activeServerId())
}

export function storePanelTokens(access: string, refresh: string, expiresIn?: number): void {
  storeServerAuth(activeServerId(), access, refresh, expiresIn)
}

export function clearPanelAuth(): void {
  clearServerAuth(activeServerId())
}

/** will is valid（used for， not ） */
export function hasStoredCredential(): boolean {
  const auth = readPanelAuth()
  return Boolean(readApiKey() || auth?.access)
}

/** 401 when by （ / Key）；client not */
let onAuthLost: ((mode: 'login' | 'key') => void) | null = null
export function setAuthLostHandler(handler: ((mode: 'login' | 'key') => void) | null): void {
  onAuthLost = handler
}

function buildHeaders(extra?: Record<string, string>): Record<string, string> {
  const headers: Record<string, string> = { Accept: 'application/json', ...extra }
  const apiKey = readApiKey()
  if (apiKey) headers['x-api-key'] = apiKey
  const auth = readPanelAuth()
  if (auth?.access) {
    headers['x-panel-token'] = auth.access
    // Authorization is ： in will 
    headers.Authorization = `Bearer ${auth.access}`
  }
  return headers
}

/** by ； after not can use up 1 in 。 */
const refreshInFlight = new Map<string, Promise<boolean>>()

function requireCurrentServer(identity: string): void {
  if (serverIdentity() !== identity) throw new DOMException('服务器已切换', 'AbortError')
}

/**
 * ， and 401 ：access that 1 will when to 401，
 * ， after to 1 （ ）refresh up ， can server
 * （`access::ROTATION_GRACE_MS`）。 this 1 when 1 
 * request， its etc 1 —— to to new all etc use 。
 * successreturn true
 */
async function tryRefresh(): Promise<boolean> {
  const identity = serverIdentity()
  const pending = refreshInFlight.get(identity)
  if (pending) return pending
  const request = doRefresh(identity).finally(() => refreshInFlight.delete(identity))
  refreshInFlight.set(identity, request)
  return request
}

async function doRefresh(identity: string): Promise<boolean> {
  const serverId = activeServerId()
  const auth = readServerAuth(serverId)
  const headers = buildHeaders({ 'Content-Type': 'application/json' })
  if (auth?.refresh) {
    headers['x-panel-refresh'] = auth.refresh
    // need response：APK down no has available cookie ， body
    headers['x-panel-auth-mode'] = 'body'
  }
  try {
    const response = await fetch(url('/api/panel/refresh'), {
      method: 'POST',
      headers,
      body: '{}',
    })
    if (!response.ok) return false
    const payload = (await response.json().catch(() => null)) as
      | { data?: { accessToken?: string; refreshToken?: string; expiresIn?: number } }
      | null
    if (serverIdentity() !== identity) return false
    const data = payload?.data
    if (data?.accessToken && data?.refreshToken) {
      storeServerAuth(serverId, data.accessToken, data.refreshToken, data.expiresIn)
    }
    return true
  } catch {
    return false
  }
}

/**
 * before （ms）： to before this 。
 *
 * minutes「 this 1 request in to 」 ， 2 when ——
 * not will as when or 1 request before （ that will 1 ）。
 */
const REFRESH_AHEAD_MS = 2 * 60 * 1000

/**
 * to before ： 「access → 401 → 」 before to before 。
 *
 * 401 when ，**** 1 401（server by 「refresh
 * 」 already verbose，request is 1 ）。 not 
 * when 1 will up ， this from that request。
 *
 * failure 1 ，request：`doRefresh` return false can can is will 
 * no （ this 401 → ）， can can is 。 has server，
 * to `apiCall` has 401 。
 */
async function ensureFreshSession(): Promise<void> {
  const auth = readPanelAuth()
  const expiresAt = Number(auth?.expiresAt)
  if (!Number.isFinite(expiresAt) || expiresAt <= 0) return
  if (Date.now() < expiresAt - REFRESH_AHEAD_MS) return
  await tryRefresh()
}

type RequestInitLite = {
  method?: 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE'
  body?: unknown
  /** not 401 （request use ） */
  quiet?: boolean
  signal?: AbortSignal
  apiKey?: string
}

/**
 * 1 API request and `{success, data}` 。
 *
 * return is `data` ；failure 1 {@link ApiError}。
 */
export function apiCall<T = unknown>(path: string, init: RequestInitLite = {}): Promise<T> {
  return requestApi<T>(path, init, serverIdentity(), true)
}

async function requestApi<T>(path: string, init: RequestInitLite, identity: string, retry: boolean): Promise<T> {
  requireCurrentServer(identity)
  if (init.signal?.aborted) throw new DOMException('请求已取消', 'AbortError')
  await ensureFreshSession()
  requireCurrentServer(identity)
  const method = init.method || 'GET'
  const headers = buildHeaders()
  if (init.apiKey !== undefined) {
    headers['x-api-key'] = init.apiKey
    headers.Authorization = `Bearer ${init.apiKey}`
  }
  const requestInit: RequestInit = { method, headers, signal: init.signal }
  if (init.body !== undefined || method !== 'GET') {
    headers['Content-Type'] = 'application/json'
    requestInit.body = JSON.stringify(init.body ?? {})
  }

  let response: Response
  try {
    response = await fetch(url(path), requestInit)
  } catch (error) {
    requireCurrentServer(identity)
    if (init.signal?.aborted || (error instanceof DOMException && error.name === 'AbortError')) throw error
    throw new ApiError(`无法连接网关：${error instanceof Error ? error.message : String(error)}`, 0)
  }

  const text = await response.text()
  requireCurrentServer(identity)
  let payload: unknown = null
  try {
    payload = text ? JSON.parse(text) : null
  } catch {
    throw new ApiError(text.trim() || `网关返回了空响应（HTTP ${response.status}）`, response.status)
  }

  if (response.status === 401 && !init.quiet) {
    const type = readErrorType(payload)
    if (type === 'panel_login_required') {
      const refreshed = retry && !path.startsWith('/api/assistant/images/') && await tryRefresh()
      requireCurrentServer(identity)
      if (refreshed) return requestApi<T>(path, init, identity, false)
      clearPanelAuth()
      onAuthLost?.('login')
    } else if (init.apiKey === undefined) {
      onAuthLost?.('key')
    }
  }

  const envelope = payload as { success?: boolean; data?: unknown; error?: unknown } | null
  const ok = response.status >= 200 && response.status < 300
  const failed = envelope?.success === false
  if (!ok || failed) {
    throw new ApiError(errorMessage(envelope?.error, response.status), response.status, readErrorType(payload))
  }
  return (envelope && 'data' in envelope ? envelope.data : envelope) as T
}

function readErrorType(payload: unknown): string {
  const error = (payload as { error?: unknown } | null)?.error
  if (error && typeof error === 'object' && 'type' in error) {
    const type = (error as { type?: unknown }).type
    return typeof type === 'string' ? type : ''
  }
  return ''
}

function errorMessage(error: unknown, status: number): string {
  if (typeof error === 'string' && error) return error
  if (error && typeof error === 'object') {
    const message = (error as { message?: unknown }).message
    if (typeof message === 'string' && message) return message
  }
  return `HTTP ${status}`
}

/** ： not can （`/health` ） */
export async function ping(): Promise<boolean> {
  try {
    const response = await fetch(url('/health'), { headers: { Accept: 'application/json' } })
    return response.ok
  } catch {
    return false
  }
}

/** and （config when use ， can is not is ） */
export async function probe(base: string): Promise<{ ok: boolean; version?: string }> {
  try {
    const response = await fetch(`${base}/health`, { headers: { Accept: 'application/json' } })
    if (!response.ok) return { ok: false }
    const payload = (await response.json().catch(() => null)) as { version?: string; data?: { version?: string } } | null
    return { ok: true, version: payload?.version || payload?.data?.version }
  } catch {
    return { ok: false }
  }
}

/** （「」 is 「create」） */
export type PanelStatus = { registered?: boolean; captchaEnabled?: boolean }
export async function panelStatus(): Promise<PanelStatus> {
  const response = await fetch(url('/api/panel/status'), { headers: { Accept: 'application/json' } })
  const payload = (await response.json().catch(() => null)) as { data?: PanelStatus } | null
  return payload?.data || {}
}
