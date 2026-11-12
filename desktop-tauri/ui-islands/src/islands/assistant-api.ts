/**
 * AI API ：、、JSON and error 1 。
 *
 * ── Whyfile ────────────────────────────────────────
 * for 、config、 will 、 down ， and （） need 
 * request。 in `assistant-tools` ， that `assistant-state`
 * （ use `loadModelNames`）—— import will 。 
 * 「 1 request」 to this ：`assistant-state` and ，
 * to is to 。
 *
 * ── Why this request not ─────────────────────────────────
 * （`window.workbuddyDesktop.xxx`） is 「 1 」 ： no has ，
 * and has 60 secondstotal when （ `src/gateway.rs` REQUEST_TIMEOUT_MS）—— for 
 * seconds， will 。 this fetch to ****：
 * · web （ by ）： = origin，，cookie ；
 * · ： Tauri ， can need 
 * （`getBackendStatus`）， `http://127.0.0.1:{port}`。
 * in by `/api/*`（none CORS）， not need need use 
 * CORS —— that is `/v1/*`， and this none。
 *
 * ── request ：**default `same-origin`， not need `include`** ──
 *
 * by CORS is `Access-Control-Allow-Origin: *`（
 * `http::attach_cors`）， by Fetch ，** request to `*` will 
 * as CORS failure** —— is 1 `TypeError: Failed to fetch`， and server
 * not to （request its to 、）。 is 
 * （ `http://tauri.localhost` → `http://127.0.0.1:{port}`），
 * use `include` will 1 「setfailure：Failed to fetch」。
 *
 * ── Why not can is `omit` ───────────────────────────────────
 * `omit` is 「 not 」，**** not —— web （ by 、
 * need need will ） will this 。 `same-origin` is that ：
 * · web → cookie， will not ；
 * · → not cookie， is can `*` CORS check
 * —— by request， down 。
 *
 * ── and Key ───────────────────────────────────
 * use `directCredentials` return will ； use use 
 * `x-api-key`， will not can Key， not Key or limit。
 */

import { t } from '../i18n'

const FALLBACK_PORT = 3065

/** （ for 1 ） */
let cachedBase: string | null = null

type BackendStatus = { port?: number }
type DesktopBridge = {
  platform?: string
  getBackendStatus?: () => Promise<BackendStatus | null | undefined>
  waitForBackend?: () => Promise<BackendStatus | null | undefined>
  /** current； refresh=true when ， by 。 */
  directCredentials?: (refresh?: boolean) => Promise<unknown>
}

function bridge(): DesktopBridge | undefined {
  return (window as unknown as { workbuddyDesktop?: DesktopBridge }).workbuddyDesktop
}

/** current this use request（module header） */
export async function gatewayBase(force = false): Promise<string> {
  if (cachedBase && !force) return cachedBase
  const api = bridge()
  if (api?.platform === 'web') {
    const origin = typeof window === 'undefined' ? '' : window.location?.origin
    if (origin && origin !== 'null') {
      cachedBase = origin
      return cachedBase
    }
  }
  let port = 0
  try {
    const status = await api?.getBackendStatus?.()
    const value = Number(status?.port)
    if (Number.isFinite(value) && value > 0) port = value
  } catch { /* no ： down defaultpath */ }
  if (port > 0) {
    cachedBase = `http://127.0.0.1:${port}`
    return cachedBase
  }
  // no up （ in ）： use default，** not ** —— down 1 use will 
  // new 1 ， 1 。 1 ， will 「
  // that seconds」 after 1 
  return `http://127.0.0.1:${FALLBACK_PORT}`
}

/** request cookie；request by current will 。 */
const CREDENTIALS: RequestCredentials = 'same-origin'

/** return → as empty （，/） */
function asHeaderTable(value: unknown): Record<string, string> {
  const table: Record<string, string> = {}
  if (!value || typeof value !== 'object') return table
  for (const [name, raw] of Object.entries(value as Record<string, unknown>)) {
    if (typeof raw === 'string' && raw) table[name] = raw
  }
  return table
}

/** not request： will before ， can can already in 1 。 */
async function directAuthHeaders(refresh = false): Promise<Record<string, string>> {
  return asHeaderTable(await bridge()?.directCredentials?.(refresh))
}

/**
 * error → can 。
 *
 * failure is `{success:false, error:""}`，** no to by ** is 
 * axum 404：`{error:{message:"Not found: ..."}}` —— that for 
 * `Error` message will to 1 `[object Object]`， this 1 。
 */
function describeError(value: unknown, status: number): string {
  if (typeof value === 'string' && value.trim()) return value.trim()
  if (value && typeof value === 'object') {
    const message = (value as { message?: unknown }).message
    if (typeof message === 'string' && message.trim()) return message.trim()
    try { return JSON.stringify(value) } catch { /* use etc ： down */ }
  }
  return `HTTP ${status}`
}

export class AssistantRequestError extends Error {
  constructor(message: string, readonly status: number, readonly code: string) {
    super(message)
    this.name = 'AssistantRequestError'
  }
}

export function requestError(parsed: unknown, status: number, method: string, path: string): AssistantRequestError {
  const envelope = parsed && typeof parsed === 'object'
    ? parsed as { error?: unknown; code?: unknown } : null
  const raw = envelope?.error ?? parsed
  const detail = raw && typeof raw === 'object' ? raw as { code?: unknown } : null
  const code = typeof detail?.code === 'string' ? detail.code
    : typeof envelope?.code === 'string' ? envelope.code : ''
  const message = describeError(raw, status)
  // has by ， and will error can can is 404。
  if (status === 404 && message === `Not found: ${method} ${path}`) {
    return new AssistantRequestError(t('这一版网关还没有 AI 助手接口（HTTP 404）：后端还是旧的，重新编译 / 重启客户端后再试。{message}', { message }), status, 'api_not_found')
  }
  return new AssistantRequestError(message, status, code)
}

/** fetch failure Error。
 * for not up has 1 `TypeError: Failed to fetch`， not up down 
 * —— 「 is 」， use for 1 can for not for 。 */
function wrapFetchError(error: unknown, base: string): Error {
  const message = error instanceof Error ? error.message : String(error)
  return new Error(t('连接网关失败（{base}）：{message}', { base, message }))
}

/**
 * 1 request（ up CREDENTIALS 1 ）。
 *
 * `body` to for by JSON （ is JSON ），`undefined` 
 * not —— and use headers/body ， and 、Content-Type
 * and has this 1 。
 *
 * will when new get， when ； 1 ， Key error not 。
 */
export async function requestDirect(
  path: string,
  options: { method: string; body?: unknown; signal?: AbortSignal; apiKey?: string },
  retried = false,
): Promise<Response> {
  const api = bridge()
  if (api?.platform !== 'web' && api?.waitForBackend) await api.waitForBackend()
  const base = await gatewayBase()
  const headers: Record<string, string> = { ...(await directAuthHeaders(retried)) }
  if (options.apiKey !== undefined) {
    headers['x-api-key'] = options.apiKey
    headers.Authorization = `Bearer ${options.apiKey}`
  }
  const payload = options.body === undefined ? undefined : JSON.stringify(options.body)
  if (payload !== undefined) headers['Content-Type'] = 'application/json'
  let response: Response
  try {
    response = await fetch(base + path, {
      method: options.method,
      headers,
      body: payload,
      signal: options.signal,
      credentials: CREDENTIALS,
    })
  } catch (error) {
    // use （abort） not is ， —— use by signal.aborted 
    if (options.signal?.aborted) throw error
    throw wrapFetchError(error, base)
  }
  if (response.status === 401 && !retried) {
    type AuthFailure = { error?: { type?: string; code?: string } }
    let rejected: AuthFailure | null = null
    try { rejected = await response.clone().json() as AuthFailure } catch { /* JSON not will */ }
    const kind = rejected?.error?.type || rejected?.error?.code
    const desktopRecovery = api?.platform !== 'web' && (kind === 'panel_login_required'
      || kind === 'invalid_api_key' && options.apiKey === undefined)
    const webRetry = api?.platform === 'web' && kind === 'panel_login_required' && options.apiKey === undefined
      && (options.method === 'GET' || !path.startsWith('/api/assistant/images/'))
    if (desktopRecovery || webRetry) {
      await response.text().catch(() => '')
      return requestDirect(path, options, true)
    }
  }
  return response
}

export async function callJson<T>(method: string, path: string, body?: unknown, signal?: AbortSignal, apiKey?: string): Promise<T> {
  const response = await requestDirect(path, { method, body, signal, apiKey })
  const text = await response.text()
  let parsed: unknown = null
  try { parsed = text ? JSON.parse(text) : null } catch { parsed = null }
  const envelope = parsed as { success?: boolean; data?: unknown; error?: unknown } | null
  if (!response.ok || envelope?.success === false) {
    throw requestError(parsed, response.status, method, path)
  }
  const payload = envelope && typeof envelope === 'object' && 'data' in envelope ? envelope.data : parsed
  return payload as T
}
