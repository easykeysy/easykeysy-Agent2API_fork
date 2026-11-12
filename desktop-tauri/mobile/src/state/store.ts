import * as React from 'react'

/**
 * use store： will 、、 by 、。
 *
 * ── Why not use ─────────────────────────────────────────
 * has 1 「current + 1 will + if 」， 
 * has will and 。 Redux/Zustand will 「 1 」 1 ，
 * this need —— 。
 *
 * ── new ────────────────────────────────────────────────
 * `refresh()` `/api/session` and ；（、 to ）success after by 
 * use `refresh()` 1 ， not 「 after 」—— up and 
 * ， in can when new 。
 */

import { fetchSession, type SessionPayload, type Account } from '../api/gateway'
// parameter（valid / / can ）： will 1 down ，
// to 1 （ providerFacts module header）
import { adoptProviderFacts } from '../lib/providerFacts'
import { normalizeRoute, routeFromHash, type Route } from '../lib/routes'
export type { Route } from '../lib/routes'
import {
  activeServer,
  isPackaged,
  listServers,
  removeServer,
  saveServer,
  serverIdentity,
  setActiveServer,
  type ServerEntry,
} from '../lib/servers'

export type ThemeMode = 'system' | 'light' | 'dark'

type Listener = () => void

type AppState = {
  route: Route
  /** （ 1 when empty） */
  stack: Route[]
  session: SessionPayload | null
  sessionError: string
  loading: boolean
  theme: ThemeMode
  toast: { text: string; tone: 'ok' | 'err' | 'warn' } | null
  /**
   * need need use when ， for ：
   * · `login` —— will ，
   * · `key` —— no Key， Key 
   */
  authLost: 'login' | 'key' | null
  /** already config （APK down can ；PWA down as empty） */
  servers: ServerEntry[]
  /** current（PWA down is ，base = current origin） */
  server: ServerEntry | null
  /** （APK） need need config and */
  packaged: boolean
  /** or when ，used for up 1 。 */
  serverRevision: number
}

const THEME_KEY = 'workbuddy-desktop-theme'

function readTheme(): ThemeMode {
  try {
    const value = localStorage.getItem(THEME_KEY)
    if (value === 'light' || value === 'dark' || value === 'system') return value
  } catch {
    /* */
  }
  return 'system'
}

/**
 * from hash by （`#accounts` / `#requests/abc`）。
 *
 * in store when use ， not is etc React after setState —— after will 
 * to ， up is 1 can 。
 */
function readInitialRoute(): Route {
  return routeFromHash(location.hash)
}

/** use to <html data-theme>： and 1 ，tokens can */
function applyTheme(mode: ThemeMode): void {
  document.documentElement.dataset.theme = mode
  const systemDark = window.matchMedia('(prefers-color-scheme: dark)').matches
  const effective = mode === 'dark' || (mode === 'system' && systemDark) ? 'dark' : 'light'
  document.documentElement.style.colorScheme = effective
  // / PWA 。 has media theme-color（
  // m.html）， this 1 media after 1 —— otherwise will 
  // 「 is 」 that 。
  const metas = document.querySelectorAll('meta[name="theme-color"]')
  metas.forEach((meta, index) => {
    meta.setAttribute('content', effective === 'dark' ? '#0a0a0a' : '#f3f3f6')
    if (index === 0) meta.removeAttribute('media')
  })
}

class Store {
  private state: AppState = {
    route: readInitialRoute(),
    stack: [],
    session: null,
    sessionError: '',
    loading: false,
    theme: readTheme(),
    toast: null,
    authLost: null,
    servers: listServers(),
    server: activeServer(),
    packaged: isPackaged(),
    serverRevision: 0,
  }

  private listeners = new Set<Listener>()
  private toastTimer: number | null = null
  private inflight: AbortController | null = null
  /**
   * 。
   *
   * has （`push`） will add； 1 replaceState，
   * 「return」 in Tab 。 this ：
   * · `back()` can not can to `history.back()`（otherwise will use use ）；
   * · popstate when need 1 。
   */
  private historyDepth = 0
  /**
   * 、 popstate `history.back()` 。
   *
   * 1 `history.back()` will 1 popstate； that when use already 
   * ， 1 will 。 this 。
   */
  private selfBackPending = 0

  get(): AppState {
    return this.state
  }

  subscribe(listener: Listener): () => void {
    this.listeners.add(listener)
    return () => {
      this.listeners.delete(listener)
    }
  }

  private emit(): void {
    this.listeners.forEach(listener => listener())
  }

  private patch(next: Partial<AppState>): void {
    this.state = { ...this.state, ...next }
    this.emit()
  }

  /* ── by ── */

  /** 1 ：empty */
  go(screen: string): void {
    const route = normalizeRoute(screen)
    if (this.state.route.screen === route.screen && this.state.stack.length === 0) return
    this.patch({ route, stack: [] })
    // Tab not is 「 down 」： before 1 and （ that 
    // not —— will 「return」 1 ， use not to ）
    this.historyDepth = 0
    this.syncHash(true)
    // will is （、、）。
    // Tab when will ， not will this —— not 1 ，
    // from 「」「」 to is up 1 when 。
    // no when （APK ） not ： that will 1 「none」。
    if (!this.state.packaged || this.state.server) void this.refresh(true)
  }

  /** ： + 1 （return can up 1 ） */
  push(screen: string, params?: Record<string, string>): void {
    this.patch({
      stack: [...this.state.stack, this.state.route],
      route: normalizeRoute(screen, params),
    })
    this.historyDepth += 1
    this.syncHash(false)
  }

  /**
   * return up 1 。
   *
   * ****（response）， 1 。 
   * `history.back()` will 1 popstate， use `selfBackPending` ，
   * otherwise will 。
   */
  back(): void {
    const stack = [...this.state.stack]
    if (stack.length > 0) {
      const prev = stack.pop()
      this.patch({ stack, route: prev || { screen: 'home' } })
      if (this.historyDepth > 0) {
        this.historyDepth -= 1
        this.selfBackPending += 1
        window.history.back()
      } else {
        this.syncHash(true)
      }
      return
    }
    // empty has （e.g. push after restoreFromHash
    // ）： 1 ， use in 1 「 by no 」 up 
    if (this.historyDepth > 0) {
      this.historyDepth -= 1
      this.selfBackPending += 1
      window.history.back()
      return
    }
    if (this.state.route.screen !== 'home') {
      this.patch({ route: { screen: 'home' }, stack: [] })
      this.syncHash(true)
    }
  }

  /**
   * return 1 。
   *
   * return `true` 「 this return 」， use is use 。
   * ：（ to ）→ → → 。
   *
   * `selfBackPending > 0` when up 1 return in etc popstate （
   * is ）。 this when ** by 1 not can ** —— that will ；
   * 「 already 」， etc 1 。
   */
  handleHardwareBack(): boolean {
    if (this.state.authLost) return false
    if (this.selfBackPending > 0) return true
    if (this.state.stack.length > 0 || this.historyDepth > 0) {
      this.back()
      return true
    }
    if (this.state.route.screen !== 'home') {
      this.patch({ route: { screen: 'home' }, stack: [] })
      this.syncHash(true)
      return true
    }
    return false
  }

  /**
   * before / after （popstate）。
   *
   * ：
   * · `history.back()` —— already ， this ；
   * · use by return / return —— 1 ；
   * · empty —— in this （PWA to up 1 ）。
   * not ， by ； down will use in use 。
   */
  handlePopState(): void {
    if (this.selfBackPending > 0) {
      this.selfBackPending -= 1
      return
    }
    if (this.historyDepth > 0) this.historyDepth -= 1
    const stack = [...this.state.stack]
    if (stack.length === 0) return
    const prev = stack.pop()
    this.patch({ stack, route: prev || { screen: 'home' } })
  }

  /**
   * and by 。
   *
   * `replace = true` used for「 not add」 （ 1 、return when already in up 
   * that 1 ）；`false` used for —— that when need 1 ，return will 
   * up 1 not is use 。
   */
  private syncHash(replace: boolean): void {
    const { route } = this.state
    const param = route.screen === 'checkin' ? route.params?.tab : route.params?.id
    const id = param ? `${route.screen}/${param}` : route.screen
    try {
      if (replace) history.replaceState(null, '', `#${id}`)
      else history.pushState(null, '', `#${id}`)
    } catch {
      /* down ignore */
    }
  }

  /**
   * when from （popstate when ， before / after available）。
   *
   * ** already 1 / **： hash not in when ，
   * not is 1 empty。
   */
  restoreFromHash(): void {
    const route = routeFromHash(location.hash)
    if (JSON.stringify(route) !== JSON.stringify(this.state.route)) this.patch({ route, stack: [] })
    this.syncHash(true)
  }

  /* ── will ── */

  async refresh(force = false): Promise<void> {
    if (this.state.packaged && !this.state.server) return
    if (this.state.loading && !force) return
    this.inflight?.abort()
    const controller = new AbortController()
    const identity = serverIdentity()
    this.inflight = controller
    this.patch({ loading: true })
    try {
      const session = await fetchSession(controller.signal)
      if (controller.signal.aborted || serverIdentity() !== identity) return
      adoptProviderFacts(session.accountFeatures)
      this.patch({ session, sessionError: '', loading: false, authLost: null })
    } catch (error) {
      if (controller.signal.aborted || serverIdentity() !== identity) return
      const message = error instanceof Error ? error.message : String(error)
      this.patch({ sessionError: message, loading: false })
    }
  }

  /** （ will that is ） */
  accounts(): Account[] {
    return this.state.session?.accounts?.accounts || []
  }

  /* ── （only APK） ── */

  /** new from localStorage 1 （ after use ） */
  private reloadServers(): void {
    this.patch({ servers: listServers(), server: activeServer() })
  }

  private resetServerData(): void {
    this.inflight?.abort()
    this.inflight = null
    adoptProviderFacts(null)
    this.patch({
      session: null, sessionError: '', authLost: null, loading: false, toast: null,
      serverRevision: this.state.serverRevision + 1,
    })
    void this.refresh(true)
  }

  /** add 1 and ；return id */
  addServer(input: { name?: string; base: string }): string {
    const previous = serverIdentity()
    const entry = saveServer(input)
    setActiveServer(entry.id)
    this.reloadServers()
    if (serverIdentity() !== previous) this.resetServerData()
    return entry.id
  }

  /** already has （ / ） */
  updateServer(id: string, input: { name?: string; base: string }): void {
    const previous = serverIdentity()
    saveServer({ id, ...input })
    this.reloadServers()
    if (serverIdentity() !== previous) this.resetServerData()
  }

  deleteServer(id: string): void {
    const previous = serverIdentity()
    removeServer(id)
    this.reloadServers()
    if (serverIdentity() !== previous) this.resetServerData()
  }

  switchServer(id: string): void {
    if (this.state.server?.id === id || !this.state.servers.some(server => server.id === id)) return
    setActiveServer(id)
    this.reloadServers()
    this.patch({ stack: [], route: { screen: 'home' } })
    this.historyDepth = 0
    this.selfBackPending = 0
    this.syncHash(true)
    this.resetServerData()
  }

  /* ── ── */

  setTheme(mode: ThemeMode): void {
    try {
      localStorage.setItem(THEME_KEY, mode)
    } catch {
      /* ： in */
    }
    applyTheme(mode)
    this.patch({ theme: mode })
  }

  /** when ，system need 1 */
  watchSystemTheme(): void {
    const media = window.matchMedia('(prefers-color-scheme: dark)')
    const onChange = () => {
      if (this.state.theme === 'system') applyTheme('system')
    }
    if (typeof media.addEventListener === 'function') media.addEventListener('change', onChange)
    else media.addListener(onChange)
  }

  /* ── ── */

  toast(text: string, tone: 'ok' | 'err' | 'warn' = 'ok'): void {
    if (this.toastTimer !== null) window.clearTimeout(this.toastTimer)
    this.patch({ toast: { text, tone } })
    this.toastTimer = window.setTimeout(() => this.patch({ toast: null }), 2000)
  }

  /* ── ── */

  setAuthLost(mode: 'login' | 'key' | null): void {
    this.patch({ authLost: mode })
  }
}

export const store = new Store()

/** store（ use ； not as selector ） */
export function useStore(): AppState {
  return React.useSyncExternalStore(
    listener => store.subscribe(listener),
    () => store.get(),
  )
}

/** in when use 1 */
export function bootstrapTheme(): void {
  applyTheme(store.get().theme)
  store.watchSystemTheme()
}
