/**
 * PWA when ：、update、、。
 *
 * ── Why need 1 ────────────────────────────────────────
 * this has ， in will 1 if：
 * · ****：Chrome / Edge has `beforeinstallprompt`， can ；
 * iOS Safari no has this ， can use 「 → to 」；
 * already in when not this 。
 * · **update**： new Service Worker after need etc （ sw.js），
 * this 「 has new in etc 」 and 。
 * · ****：`navigator.onLine` has no has ， not can ——
 * this 「 can 」 ， can by request。
 *
 * ── Why not is React context ────────────────────
 * `beforeinstallprompt` can can in React before （ when ），
 * for can 1 ， no has 。must in when up ，
 * to （ lib/usePwa.ts）。
 */

export type PwaState = {
  /** can （Chrome / Edge and ） */
  canInstall: boolean
  /** iOS Safari： no has ， can to 「 to 」 */
  showIosHint: boolean
  /** already in （standalone） */
  standalone: boolean
  /** has new Service Worker in etc */
  updateReady: boolean
  /** （， not can ） */
  offline: boolean
  /**
   * Service Worker 。
   * · `checking` —— no return， not this down ；
   * · `ready` —— already ，available；
   * · `unsupported` —— up down （ http） or failure。
   */
  swState: 'checking' | 'ready' | 'unsupported'
}

/** `beforeinstallprompt` （TS no has ， this ） */
type InstallPromptEvent = Event & {
  prompt: () => Promise<void>
  userChoice: Promise<{ outcome: 'accepted' | 'dismissed' }>
}

let deferredPrompt: InstallPromptEvent | null = null
let started = false

const listeners = new Set<() => void>()

const state: PwaState = {
  canInstall: false,
  showIosHint: false,
  standalone: false,
  updateReady: false,
  offline: false,
  swState: 'checking',
}

/** for ：useSyncExternalStore need 1 return 1 use */
let snapshot: PwaState = { ...state }

function emit(): void {
  snapshot = { ...state }
  listeners.forEach(listener => listener())
}

function set(patch: Partial<PwaState>): void {
  let changed = false
  for (const key of Object.keys(patch) as Array<keyof PwaState>) {
    if (state[key] !== patch[key]) {
      state[key] = patch[key] as never
      changed = true
    }
  }
  if (changed) emit()
}

/** is in （ / PWA ） */
function detectStandalone(): boolean {
  if (typeof window === 'undefined') return false
  // display-mode is ；iOS navigator.standalone
  const queries = ['(display-mode: standalone)', '(display-mode: fullscreen)', '(display-mode: minimal-ui)']
  for (const query of queries) {
    if (window.matchMedia(query).matches) return true
  }
  const legacy = (window.navigator as Navigator & { standalone?: boolean }).standalone
  return legacy === true
}

/** iOS / iPadOS up Safari（ iPadOS 13+ macOS ） */
function isIosSafari(): boolean {
  if (typeof navigator === 'undefined') return false
  const ua = navigator.userAgent
  const isIos = /iPad|iPhone|iPod/.test(ua) || (ua.includes('Macintosh') && 'ontouchend' in document)
  if (!isIos) return false
  // Chrome / Firefox / Edge in iOS up is WebKit ， not ——
  // has Safari has 「 to 」， its when Safari
  return /Safari/.test(ua) && !/CriOS|FxiOS|EdgiOS|OPiOS/.test(ua)
}

/**
 * 。 etc ： use return（ use 1 can ）。
 *
 * return， need is to 「 need 」 use ； use 
 * 1 ， not no 。
 */
export function initPwa(): () => void {
  if (typeof window === 'undefined' || started) return () => undefined
  started = true

  set({ standalone: detectStandalone() })

  // ── ──────────────────────────────────────────────
  const onBeforeInstall = (event: Event) => {
    // default ： need in set when 
    event.preventDefault()
    deferredPrompt = event as InstallPromptEvent
    set({ canInstall: true, showIosHint: false })
  }
  const onInstalled = () => {
    deferredPrompt = null
    set({ canInstall: false, showIosHint: false, standalone: true })
  }
  window.addEventListener('beforeinstallprompt', onBeforeInstall)
  window.addEventListener('appinstalled', onInstalled)

  // iOS no has beforeinstallprompt： use 「 + iOS Safari」
  if (!detectStandalone() && isIosSafari()) {
    set({ showIosHint: true })
  }

  // display-mode ： from to （ after ）
  const mediaQueries = [
    '(display-mode: standalone)',
    '(display-mode: fullscreen)',
    '(display-mode: minimal-ui)',
  ].map(query => window.matchMedia(query))
  const onDisplayMode = () => {
    const standalone = detectStandalone()
    set({ standalone, showIosHint: standalone ? false : state.showIosHint })
  }
  mediaQueries.forEach(mq => {
    if (typeof mq.addEventListener === 'function') mq.addEventListener('change', onDisplayMode)
    else mq.addListener(onDisplayMode)
  })

  // ── ──────────────────────────────────────────────
  const onOnline = () => set({ offline: false })
  const onOffline = () => set({ offline: true })
  set({ offline: !navigator.onLine })
  window.addEventListener('online', onOnline)
  window.addEventListener('offline', onOffline)

  // ── Service Worker update ───────────────────────────────────
  let swCleanup: (() => void) | null = null
  let disposed = false

  // in up down ： http://<IP> down not SW
  if ('serviceWorker' in navigator && window.isSecureContext) {
    void navigator.serviceWorker
      // updateViaCache: 'none' —— need sw.js，
      // not HTTP 。 file，default down can can 
      // old update for ， is 「 has new 」 not to 。
      .register('sw.js', { scope: '/', updateViaCache: 'none' })
      .then(registration => {
        if (disposed) return
        set({ swState: 'ready' })
        swCleanup = watchUpdates(registration)
      })
      .catch(() => {
        // failure not use （ is no has ）
        set({ swState: 'unsupported' })
      })
  } else {
    set({ swState: 'unsupported' })
  }

  return () => {
    disposed = true
    window.removeEventListener('beforeinstallprompt', onBeforeInstall)
    window.removeEventListener('appinstalled', onInstalled)
    mediaQueries.forEach(mq => {
      if (typeof mq.removeEventListener === 'function') mq.removeEventListener('change', onDisplayMode)
      else mq.removeListener(onDisplayMode)
    })
    window.removeEventListener('online', onOnline)
    window.removeEventListener('offline', onOffline)
    swCleanup?.()
  }
}

/**
 * SW update。
 *
 * ：
 * · `updatefound` —— new and ；
 * · when already has 1 `waiting` SW（ up when down no ）。
 * as 「 has new in etc 」， to 1 「update」。
 */
function watchUpdates(registration: ServiceWorkerRegistration): () => void {
  const markReady = () => set({ updateReady: true })

  if (registration.waiting && navigator.serviceWorker.controller) {
    markReady()
  }

  const onUpdateFound = () => {
    const installing = registration.installing
    if (!installing) return
    installing.addEventListener('statechange', () => {
      // `installed` and already has controller = has old in ， new in etc 
      if (installing.state === 'installed' && navigator.serviceWorker.controller) {
        markReady()
      }
    })
  }
  registration.addEventListener('updatefound', onUpdateFound)

  // new can when 1 update（ up before is when ）
  const onVisible = () => {
    if (document.visibilityState === 'visible') void registration.update().catch(() => undefined)
  }
  document.addEventListener('visibilitychange', onVisible)

  return () => {
    registration.removeEventListener('updatefound', onUpdateFound)
    document.removeEventListener('visibilitychange', onVisible)
  }
}

/** （ React useSyncExternalStore use ） */
export function subscribePwa(listener: () => void): () => void {
  listeners.add(listener)
  return () => {
    listeners.delete(listener)
  }
}

/** current（ 1 return 1 use ， useSyncExternalStore need ） */
export function getPwaSnapshot(): PwaState {
  return snapshot
}

/**
 * 。return use ； no has available when return `'unavailable'`
 * （ use ）。
 */
export async function promptInstall(): Promise<'accepted' | 'dismissed' | 'unavailable'> {
  if (!deferredPrompt) return 'unavailable'
  const prompt = deferredPrompt
  deferredPrompt = null
  set({ canInstall: false })
  try {
    await prompt.prompt()
    const choice = await prompt.userChoice
    return choice.outcome
  } catch {
    return 'dismissed'
  }
}

/** etc in new SW （ after reload） */
export async function applyUpdate(): Promise<void> {
  const registration = await navigator.serviceWorker.getRegistration()
  registration?.waiting?.postMessage({ type: 'SKIP_WAITING' })
  // etc controller new ， new after is old SW in 
  await new Promise<void>(resolve => {
    let done = false
    const finish = () => {
      if (done) return
      done = true
      navigator.serviceWorker.removeEventListener('controllerchange', finish)
      resolve()
    }
    navigator.serviceWorker.addEventListener('controllerchange', finish)
    // ：600ms no new （e.g. SW already ）
    window.setTimeout(finish, 600)
  })
  window.location.reload()
}
