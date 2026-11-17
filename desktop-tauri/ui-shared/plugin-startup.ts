import type { HiddenAccountProvider, HiddenAccountsSummary } from './plugins'

export type PluginPhase = 'queued' | 'runtime' | 'starting' | 'loading' | 'registering' | 'ready' | 'failed' | 'timeout' | 'blocked' | 'disabled' | 'unknown'
export type PluginLoadStatus = {
  phase: PluginPhase
  attempt: number
  startedAt: number | null
  updatedAt: number
  finishedAt: number | null
  deadlineAt: number | null
  error: string | null
  previousError: string | null
  steps?: { phase: PluginPhase; durationMs: number }[]
}
export type PluginProviderState = {
  id: string; label: string; packId: string; packEnabled: boolean; enabled: boolean; available: boolean; startup: PluginLoadStatus
}
export type PluginStartupSnapshot = {
  bootId: string
  revision: string
  availabilityRevision: string
  pending: boolean
  packs: { id: string; name: string; script: boolean; startup: PluginLoadStatus }[]
  providers: Record<string, PluginProviderState>
  error: string | null
  scriptOperation?: { active: boolean; target?: { id?: string }; error?: string | null; updatedAt?: number }
  now: number
}
type Translate = (key: string, params?: Record<string, string | number>) => string

export function pluginPending(status?: PluginLoadStatus | null): boolean {
  return Boolean(status && ['queued', 'runtime', 'starting', 'loading', 'registering'].includes(status.phase))
}
export function pluginFailed(status?: PluginLoadStatus | null): boolean {
  return status?.phase === 'failed' || status?.phase === 'timeout'
}
export function pluginPhaseLabel(status: PluginLoadStatus | null | undefined, t: Translate): string {
  switch (status?.phase) {
    case 'queued': return t('Waiting to load')
    case 'runtime': return t('Preparing runtime')
    case 'starting': return t('Starting')
    case 'loading': return t('Loading')
    case 'registering': return t('Almost ready')
    case 'ready': return t('Ready')
    case 'failed': return t('Load failed')
    case 'timeout': return t('Initialization timed out')
    case 'blocked': return t('Waiting for runtime recovery')
    case 'disabled': return t('Disabled')
    default: return t('Status unconfirmed')
  }
}
export function pluginPhaseDetail(status: PluginLoadStatus | null | undefined, t: Translate, now = Date.now()): string {
  if (status?.error && pluginFailed(status)) return status.error
  switch (status?.phase) {
    case 'queued': return t('Waiting for earlier plugins to finish. This is not a load failure.')
    case 'runtime': return t('Checking or preparing the script runtime.')
    case 'starting': return t('Starting the plugin. Accounts will return automatically when it is ready.')
    case 'loading': return now - status.updatedAt > 10_000
      ? t('Startup is taking longer, but loading is still in progress. No failure has been reported.') : t('Initializing the plugin. No failure has been reported.')
    case 'registering': return t('Initialization is complete. Waiting for validation and registration.')
    case 'ready': return t('The plugin is ready. The account list will update automatically.')
    case 'timeout': return t('This initialization exceeded its time limit and was stopped. Check diagnostics before retrying.')
    case 'blocked': return t('The shared runtime is not ready. No load failure has been confirmed for this plugin.')
    case 'failed': return t('This load did not complete. Check diagnostics.')
    default: return t('No confirmed load result is available yet. This does not indicate a startup failure.')
  }
}

type View = { value: PluginStartupSnapshot | null; error: string; receivedAt: number; unavailable: boolean }
let view: View = { value: null, error: '', receivedAt: 0, unavailable: false }
const listeners = new Set<() => void>()
export const subscribePluginStartup = (listener: () => void) => { listeners.add(listener); return () => { listeners.delete(listener) } }
export const getPluginStartup = () => view
function publish(patch: Partial<View>) { view = { ...view, ...patch }; listeners.forEach(listener => listener()) }
let wake: (() => void) | undefined
export function refreshPluginStartup() { wake?.() }

export function startPluginStartupPolling(options: {
  read(signal: AbortSignal): Promise<PluginStartupSnapshot>
  refresh(): Promise<unknown>
  session(): PluginStartupSnapshot | null | undefined
  onChange?(): void
}) {
  let disposed = false
  let pending = false
  let again = false
  let timer: ReturnType<typeof setTimeout> | undefined
  let controller: AbortController | undefined
  let syncing: Promise<void> | null = null
  let syncAgain = false
  let lastAvailable = ''
  let failures = 0
  publish({ value: null, error: '', receivedAt: 0, unavailable: false })
  const sync = () => {
    if (syncing) { syncAgain = true; return }
    syncing = (async () => {
      do {
        syncAgain = false
        try { await options.refresh() } catch { /* 会话读取错误由账号页展示 */ }
      } while (!disposed && syncAgain)
    })().finally(() => { syncing = null })
  }
  const poll = async () => {
    if (disposed || document.hidden) return
    if (pending) { again = true; return }
    pending = true
    controller = new AbortController()
    const timeout = setTimeout(() => controller?.abort(), 8_000)
    try {
      const next = await options.read(controller.signal)
      if (disposed || controller.signal.aborted) return
      if (!next || typeof next.bootId !== 'string' || typeof next.revision !== 'string'
        || !next.providers || !Array.isArray(next.packs)) throw new Error('Invalid plugin startup status')
      failures = 0
      const changed = view.value?.bootId !== next.bootId || view.value?.revision !== next.revision
      publish({ value: next, error: next.error || '', receivedAt: Date.now(), unavailable: false })
      if (changed) options.onChange?.()
      const availability = `${next.bootId}:${next.availabilityRevision}`
      const session = options.session()
      if (lastAvailable && lastAvailable !== availability || !session
        || session.bootId !== next.bootId || session.availabilityRevision !== next.availabilityRevision) sync()
      lastAvailable = availability
    } catch (error) {
      if (disposed) return
      failures += 1
      const status = (error as { status?: number })?.status
      publish({ error: error instanceof Error ? error.message : String(error), unavailable: status === 404 })
    } finally {
      clearTimeout(timeout)
      pending = false
      controller = undefined
      if (!disposed) {
        clearTimeout(timer)
        const delay = again ? 0 : view.unavailable ? 60_000 : failures ? Math.min(30_000, 2_000 * failures)
          : view.value?.pending || view.value?.scriptOperation?.active ? 1500 : 20_000
        again = false
        timer = setTimeout(() => { void poll() }, delay)
      }
    }
  }
  const refresh = () => { clearTimeout(timer); void poll() }
  const visibility = () => { if (!document.hidden) refresh(); else clearTimeout(timer) }
  wake = refresh
  document.addEventListener('visibilitychange', visibility)
  refresh()
  return () => {
    disposed = true
    clearTimeout(timer)
    controller?.abort()
    if (wake === refresh) wake = undefined
    document.removeEventListener('visibilitychange', visibility)
  }
}

export function startupForPack(id: string, fallback?: PluginLoadStatus | null): PluginLoadStatus | null | undefined {
  const live = view.value?.packs.find(pack => pack.id === id)?.startup
  return fallback && (!live || fallback.updatedAt > live.updatedAt) ? fallback : live || fallback
}
export function hiddenWithStartup(hidden: HiddenAccountsSummary): HiddenAccountsSummary {
  const providers = hidden.providers.map((item): HiddenAccountProvider => {
    const live = view.value?.providers[item.id]
    if (!live || item.startup && item.startup.updatedAt > live.startup.updatedAt) return item
    if (!live.enabled) return { ...item, packId: live.packId, startup: live.startup,
      reason: live.packEnabled ? 'provider_disabled' : 'pack_disabled' }
    const startup = live.startup
    const reason = pluginFailed(startup) ? startup.phase === 'timeout' ? 'load_timeout' : 'load_failed'
      : pluginPending(startup) || live.available ? 'loading' : 'not_loaded'
    return { ...item, packId: live.packId, startup, reason }
  })
  return { ...hidden, providers }
}
