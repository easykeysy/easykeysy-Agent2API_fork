/**
 * （） and current。
 *
 * ── ──────────────────────────────────────────────
 * · **PWA / **： is ，， as current origin。
 * this when not this use —— will request to 。
 * · ** App**：， and not 。 use need ， can can has 。
 *
 * is `isPackaged()`：Capacitor in will window `Capacitor`，
 * no has 。 not use UA —— `/m` and is 1 。
 *
 * ── Why 1 ────────────────────────────
 * will is by ， will 「 to B A 」——
 * is after 401、 A ，。 by id 。
 */

export type ServerEntry = {
  /** id（create when ， / delete， not ） */
  id: string
  /** use to this ，empty then */
  name: string
  /** 1 after ， `http://192.168.1.23:3065`（ not ） */
  base: string
}

const SERVERS_KEY = 'agent2api.servers'
const ACTIVE_KEY = 'agent2api.activeServer'

/** is in （Capacitor ） */
export function isPackaged(): boolean {
  if (typeof window === 'undefined') return false
  const cap = (window as Window & { Capacitor?: { isNativePlatform?: () => boolean } }).Capacitor
  return Boolean(cap?.isNativePlatform?.())
}

/**
 * current：`android` / `ios`， is `web`。
 *
 * ── Why need to this 1 ──────────────────────────────────────
 * 「 use update」 not ：
 * · Android can down APK and （ApkUpdaterPlugin）；
 * · iOS not App can —— can to App Store / TestFlight，
 * that not this 「update」 this down by 。
 * use `isPackaged()` will iOS APK ， down will 
 * 「current not is APK 」。
 */
export type NativePlatform = 'android' | 'ios' | 'web'

export function nativePlatform(): NativePlatform {
  if (typeof window === 'undefined') return 'web'
  const cap = (window as Window & { Capacitor?: { getPlatform?: () => string } }).Capacitor
  const platform = cap?.getPlatform?.()
  if (platform === 'android') return 'android'
  if (platform === 'ios') return 'ios'
  return 'web'
}

/**
 * 1 use 。
 *
 * use will `192.168.1.23`、`192.168.1.23:3065`、`http://192.168.1.23:3065/`、
 * `192.168.1.23:3065/m`、`https://gw.example.com` etc 。 1 
 * `scheme://host[:port]`：
 * · http / https， （ftp:// ） as ；
 * · no `http://`（ is http）；
 * · and path（ use can can `/m` or `/v1`）；
 * · http no when default 3065；https no when not ， to 
 * by 443 —— `:443` will /CDN 1 。
 * returnempty， by use 。
 */
export function normalizeBase(raw: string): string {
  let text = raw.trim()
  if (!text) return ''
  // is 「 use 」 1 ，must； http / https
  const schemeMatch = /^([a-zA-Z][a-zA-Z0-9+.-]*):\/\//.exec(text)
  const scheme = schemeMatch ? schemeMatch[1].toLowerCase() : ''
  if (scheme && scheme !== 'http' && scheme !== 'https') return ''
  // use from when will /m、/v1、#hash ， 1 
  text = text.replace(/^[a-zA-Z][a-zA-Z0-9+.-]*:\/\//, '')
  text = text.split(/[/?#]/)[0]
  if (!text) return ''
  // host can is IPv4 / / IPv6； need 
  const match = /^(\[[0-9a-fA-F:]+\]|[^:]+)(?::(\d+))?$/.exec(text)
  if (!match) return ''
  const host = match[1].toLowerCase()
  const port = match[2]
  // IPv4 / / IPv6；localhost this 
  if (!/^[\w.-]+$/.test(host) && !/^\[[0-9a-fA-F:]+\]$/.test(host)) return ''
  if (port && (Number(port) < 1 || Number(port) > 65535)) return ''
  if (port) return `${scheme || 'http'}://${host}:${port}`
  // no ：http default，https 443（ not ）
  return scheme === 'https' ? `https://${host}` : `http://${host}:3065`
}

/**
 * from to use host:port。
 *
 * `http://`： is default，；`https://`
 * must —— use need can 1 this is not is 。
 */
export function baseLabel(base: string): string {
  return base.replace(/^http:\/\//, '')
}

function readServers(): ServerEntry[] {
  try {
    const raw = localStorage.getItem(SERVERS_KEY)
    if (!raw) return []
    const parsed = JSON.parse(raw) as unknown
    if (!Array.isArray(parsed)) return []
    return parsed.filter(
      (item): item is ServerEntry =>
        Boolean(item) && typeof item === 'object' && typeof (item as ServerEntry).base === 'string',
    )
  } catch {
    return []
  }
}

function writeServers(list: ServerEntry[]): void {
  try {
    localStorage.setItem(SERVERS_KEY, JSON.stringify(list))
  } catch {
    /* ： will valid， down need */
  }
}

export function listServers(): ServerEntry[] {
  return readServers()
}

/** current；not configuredreturn null（APK is this ） */
export function activeServer(): ServerEntry | null {
  if (!isPackaged()) {
    // / PWA： is current origin， not 
    return { id: 'origin', name: '', base: location.origin }
  }
  const list = readServers()
  let id = ''
  try {
    id = localStorage.getItem(ACTIVE_KEY) || ''
  } catch {
    /* ignore */
  }
  return list.find(item => item.id === id) || list[0] || null
}

/** request use ； no has available when returnempty（ use config） */
export function apiBase(): string {
  return activeServer()?.base || ''
}

/** current id（ by ） */
export function activeServerId(): string {
  return activeServer()?.id || ''
}

/** in request and when id and ， will 。 */
export function serverIdentity(): string {
  const server = activeServer()
  return server ? `${server.id}:${server.base}` : ''
}

/**
 * add or update 1 。return（ already in then use that ，
 * not ）。
 */
export function saveServer(input: { id?: string; name?: string; base: string }): ServerEntry {
  const base = normalizeBase(input.base)
  if (!base) throw new Error('地址不合法')
  const list = readServers()
  const existing = input.id
    ? list.find(item => item.id === input.id)
    : list.find(item => item.base === base)
  if (existing) {
    if (existing.base !== base) {
      clearServerAuth(existing.id)
      storeServerKey(existing.id, '')
    }
    existing.base = base
    existing.name = (input.name || '').trim()
    writeServers(list)
    return existing
  }
  const entry: ServerEntry = {
    id: `srv_${Date.now().toString(36)}_${Math.random().toString(36).slice(2, 7)}`,
    name: (input.name || '').trim(),
    base,
  }
  writeServers([...list, entry])
  return entry
}

export function removeServer(id: string): void {
  const current = activeServerId()
  const list = readServers().filter(item => item.id !== id)
  writeServers(list)
  clearServerAuth(id)
  storeServerKey(id, '')
  if (current === id) {
    try {
      localStorage.setItem(ACTIVE_KEY, list[0]?.id || '')
    } catch {
      /* ignore */
    }
  }
}

export function setActiveServer(id: string): void {
  try {
    localStorage.setItem(ACTIVE_KEY, id)
  } catch {
    /* ignore */
  }
}

/* ── by ───────────────────────────────────── */

const KEY_STORAGE = (id: string) => `agent2api.webKey.${id}`
const PANEL_AUTH = (id: string) => `agent2api.panelAuth.${id}`

export type PanelAuth = {
  access?: string
  refresh?: string
  /**
   * access to when （ms；0 / = not ）。
   *
   * ** not is **：「 this not this in down request before 」（ client.ts 
   * `ensureFreshSession`）。server by will valid， this will 
   * 「401 → 」 path。 and in 1 ： will when 1 。
   */
  expiresAt?: number
}

export function readServerKey(id: string): string {
  try {
    return localStorage.getItem(KEY_STORAGE(id)) || ''
  } catch {
    return ''
  }
}

export function storeServerKey(id: string, value: string): void {
  try {
    if (value) localStorage.setItem(KEY_STORAGE(id), value)
    else localStorage.removeItem(KEY_STORAGE(id))
  } catch {
    /* ： in */
  }
}

export function readServerAuth(id: string): PanelAuth | null {
  try {
    const raw = localStorage.getItem(PANEL_AUTH(id))
    return raw ? (JSON.parse(raw) as PanelAuth) : null
  } catch {
    return null
  }
}

/**
 * 1 for 。`expiresIn` is server down remainingseconds（ `issue_response`），
 * for when down —— for seconds in 。
 */
export function storeServerAuth(
  id: string,
  access: string,
  refresh: string,
  expiresIn?: number,
): void {
  const seconds = Number(expiresIn)
  const expiresAt = Number.isFinite(seconds) && seconds > 0 ? Date.now() + seconds * 1000 : 0
  try {
    localStorage.setItem(PANEL_AUTH(id), JSON.stringify({ access, refresh, expiresAt }))
  } catch {
    /* up */
  }
}

export function clearServerAuth(id: string): void {
  try {
    localStorage.removeItem(PANEL_AUTH(id))
  } catch {
    /* none */
  }
}
