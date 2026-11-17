export type LogView = 'important' | 'all' | 'debug'
export type LogEntry = {
  id?: number
  ts?: number
  level?: string
  category?: string
  message?: string
  data?: Record<string, unknown> | null
}
export type LogGroup = {
  id: number
  firstTs: number
  lastTs: number
  count: number
  entry: LogEntry
  entries: LogEntry[]
}
export type LogPage = {
  entries?: LogEntry[]
  groups?: LogGroup[]
  total?: number
  matched?: number
  grouped?: number
  offset?: number
  limit?: number
  lastId?: number
  byLevel?: Record<string, number>
  categories?: Record<string, string>
  debugEnabled?: boolean
}
export type LogQuery = {
  view?: LogView
  aggregate?: boolean
  offset?: number
  limit?: number
  untilId?: number
  sinceId?: number
  level?: string
  category?: string
  keyword?: string
  start?: number
  end?: number
  startDate?: string
  endDate?: string
}

type Translate = (text: string, params?: Record<string, string | number>) => string

export function logQueryString(query: LogQuery): string {
  const params = new URLSearchParams()
  for (const [key, value] of Object.entries(query)) {
    if (value !== undefined && value !== '') params.set(key, typeof value === 'boolean' ? (value ? '1' : '0') : String(value))
  }
  return params.toString()
}

export function logGroups(page: LogPage | null | undefined): LogGroup[] {
  if (Array.isArray(page?.groups)) return page.groups
  return (page?.entries || []).map((entry, index) => ({
    id: entry.id ?? index, firstTs: entry.ts ?? 0, lastTs: entry.ts ?? 0,
    count: 1, entry, entries: [entry],
  }))
}

export function logText(entry: LogEntry, key: string): string {
  const value = entry.data?.[key]
  return typeof value === 'string' || typeof value === 'number' ? String(value) : ''
}

export function logLevel(level: string | undefined, t: Translate): string {
  switch (level) {
    case 'debug': return t('Debug')
    case 'warn': return t('Warning')
    case 'error': return t('Error')
    default: return t('Info')
  }
}

export function logCategory(category: string | undefined, t: Translate): string {
  switch (category) {
    case 'server': return t('Service')
    case 'auth': return t('Sign in')
    case 'account': return t('Accounts')
    case 'model': return t('Model')
    case 'upstream': return t('Upstream')
    case 'config': return t('Profile')
    case 'checkin': return t('Auto check-in')
    case 'maintenance': return t('Credential auto-maintenance')
    case 'update': return t('Software update check')
    case 'desensitize': return t('Redaction')
    case 'plugin': return t('Plugins')
    default: return category || '—'
  }
}

export function logSource(entry: LogEntry, t: Translate): string {
  return logText(entry, 'pluginName') || logText(entry, 'providerLabel') || logText(entry, 'provider')
    || logText(entry, 'component') || logCategory(entry.category, t)
}

export function logSummary(entry: LogEntry, t: Translate): string {
  if (logText(entry, 'action') === 'plugin.startup.summary') {
    const ready = logNumber(entry, 'readyCount') ?? 0
    const failed = logNumber(entry, 'failedCount') ?? 0
    const blocked = logNumber(entry, 'blockedCount') ?? 0
    if (!failed && !blocked) return t('Plugin startup complete: {n} ready', { n: ready })
    return t('Some plugins could not start')
  }
  if (logText(entry, 'action') === 'plugin.startup') {
    switch (logText(entry, 'phase')) {
      case 'ready': return t('Plugin ready')
      case 'failed': return t('Plugin failed to start')
      case 'timeout': return t('Plugin startup timed out')
      case 'blocked': return t('Plugin unavailable while the runtime recovers')
      default: return t('Plugin did not become ready')
    }
  }
  switch (logText(entry, 'action')) {
    case 'plugin.failure': return t('Plugin temporarily unavailable')
    case 'plugin.exit': return t('Plugin process exited')
    case 'plugin.recovered': return t('Plugin is running again')
    case 'plugin.shutdown': return t('Plugin did not stop normally')
  }
  const summary = logText(entry, 'summary') || (entry.message || '').replace(/^(?:\[[^\]]+\]\s*)+/, '')
  switch (summary) {
    case 'Billing API request': return t('Billing API request')
    case 'Upstream API request': return t('Upstream API request')
    case 'Outbound parameter normalization': return t('Outbound parameter normalization')
    case 'System message added': return t('System message added')
    case 'Balance query completed': return t('Balance query completed')
    case 'Balance query not configured': return t('Balance query not configured')
    case 'Balance query failed': return t('Balance query failed')
    default: return summary || '—'
  }
}

export function logNumber(entry: LogEntry, key: string): number | undefined {
  const value = entry.data?.[key]
  if (typeof value !== 'number' && (typeof value !== 'string' || !value.trim())) return undefined
  const number = Number(value)
  return Number.isFinite(number) && number >= 0 ? number : undefined
}

export function formatLogDuration(ms: number | undefined, t: Translate): string {
  if (ms === undefined) return ''
  if (ms < 1000) return t('{n} ms', { n: Math.round(ms) })
  if (ms < 60_000) return t('{n} s', { n: Number((ms / 1000).toFixed(1)) })
  const seconds = Math.round(ms / 1000)
  return t('{minutes} min {seconds} s', { minutes: Math.floor(seconds / 60), seconds: seconds % 60 })
}

export function logContext(entry: LogEntry, t: Translate): string {
  const parts = [logText(entry, 'accountName') || logText(entry, 'account'), logText(entry, 'model')]
  const from = logText(entry, 'from')
  const to = logText(entry, 'to')
  if (from || to) parts.push([from, to].filter(Boolean).join(' → '))
  const action = logText(entry, 'action')
  if (action === 'plugin.startup.summary') {
    const ready = logNumber(entry, 'readyCount') ?? 0
    const failed = logNumber(entry, 'failedCount') ?? 0
    const blocked = logNumber(entry, 'blockedCount') ?? 0
    if (failed || blocked) parts.push(t('{n} ready', { n: ready }))
    if (failed) parts.push(t('{n} failed to start', { n: failed }))
    if (blocked) parts.push(t('{n} waiting for recovery', { n: blocked }))
  }
  const duration = formatLogDuration(logNumber(entry, 'durationMs'), t)
  if (duration) parts.push(t('Took {duration}', { duration }))
  const status = logText(entry, 'status')
  if (status) parts.push(`HTTP ${status}`)
  if (action === 'plugin.startup' && logText(entry, 'phase') !== 'ready') {
    parts.push(t('Related features are unavailable; see details for the reason'))
  } else if (action === 'plugin.failure' || action === 'plugin.exit') {
    parts.push(t('Related features are unavailable; see details for the reason'))
  } else {
    const detail = logText(entry, 'detail')
    if (detail && entry.level !== 'debug' && !action.startsWith('plugin.')) parts.push(detail)
  }
  return parts.filter(Boolean).join(' · ')
}

export function logDetailText(entry: LogEntry): string {
  const detail = logText(entry, 'detail')
  const action = logText(entry, 'action')
  if (!['plugin.failure', 'plugin.exit', 'plugin.shutdown', 'plugin.recovered'].includes(action)) return detail
  const message = (entry.message || '').replace(/^(?:(?:\[[^\]]+\]|Plugin)\s*)+/, '')
  return [...new Set([message, detail].filter(Boolean))].join('\n\n')
}

export function logAdvice(entry: LogEntry, t: Translate): string {
  const action = logText(entry, 'action')
  if (action === 'plugin.startup.summary') {
    return (logNumber(entry, 'failedCount') || logNumber(entry, 'blockedCount'))
      ? t('Some plugins are unavailable. Open the Plugins page to view their loading diagnostics.') : ''
  }
  if (action === 'plugin.startup') {
    switch (logText(entry, 'phase')) {
      case 'ready': return ''
      case 'timeout': return t('Startup exceeded the time limit. Review the loading diagnostics on the Plugins page before reloading.')
      case 'blocked': return t('The shared runtime is not ready; this does not mean the plugin is damaged. View the loading diagnostics on the Plugins page.')
      default: return t('View the loading diagnostics on the Plugins page and check the cause before reloading.')
    }
  }
  if (['plugin.failure', 'plugin.exit', 'plugin.shutdown'].includes(action)) {
    return t('View the loading diagnostics on the Plugins page and check the cause before reloading.')
  }
  return ''
}

export function logPhase(phase: string, t: Translate): string {
  switch (phase) {
    case 'queued': return t('Waiting to load')
    case 'runtime': return t('Preparing runtime')
    case 'starting': return t('Starting process')
    case 'loading': return t('Loading plugin')
    case 'registering': return t('Registering providers')
    case 'ready': return t('Ready')
    case 'failed': return t('Launch failed')
    case 'timeout': return t('Startup timed out')
    case 'blocked': return t('Waiting for recovery')
    default: return t('Result unconfirmed')
  }
}

export function logSteps(entry: LogEntry, t: Translate): { label: string; duration: string }[] {
  const steps = entry.data?.steps
  if (!Array.isArray(steps)) return []
  return steps.flatMap(step => {
    if (!step || typeof step !== 'object' || typeof step.phase !== 'string'
      || typeof step.durationMs !== 'number' || !Number.isFinite(step.durationMs) || step.durationMs < 0) return []
    return [{ label: logPhase(step.phase, t), duration: formatLogDuration(step.durationMs, t) }]
  })
}

export function logPlugins(entry: LogEntry, t: Translate): { id: string; name: string; status: string; phase: string }[] {
  const plugins = entry.data?.plugins
  if (!Array.isArray(plugins)) return []
  return plugins.flatMap(plugin => {
    if (!plugin || typeof plugin !== 'object' || typeof plugin.pluginId !== 'string' || typeof plugin.phase !== 'string') return []
    return [{ id: plugin.pluginId, name: typeof plugin.pluginName === 'string' && plugin.pluginName ? plugin.pluginName : t('Plugin name not recorded'),
      status: logPhase(plugin.phase, t), phase: plugin.phase }]
  }).sort((left, right) => Number(left.phase === 'ready') - Number(right.phase === 'ready'))
}

export function logViewNote(view: LogView, t: Translate): string {
  switch (view) {
    case 'important': return t('Shows startup summaries and events that need attention. Individual successful startups are in All events.')
    case 'all': return t('Shows all retained events. Open details for technical information.')
    default: return t('Shows only debug records for troubleshooting.')
  }
}

export function formatLogTime(ts: number | undefined, clockOnly = false): string {
  if (!ts) return '—'
  const date = new Date(ts)
  const pad = (n: number) => String(n).padStart(2, '0')
  const clock = `${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}`
  if (clockOnly) return clock
  const day = `${pad(date.getMonth() + 1)}-${pad(date.getDate())}`
  return date.getFullYear() === new Date().getFullYear() ? `${day} ${clock}` : `${date.getFullYear()}-${day} ${clock}`
}

export function groupTime(group: LogGroup): string {
  if (group.firstTs === group.lastTs) return formatLogTime(group.lastTs)
  const sameDay = new Date(group.firstTs).toDateString() === new Date(group.lastTs).toDateString()
  return `${formatLogTime(group.firstTs)}–${formatLogTime(group.lastTs, sameDay)}`
}

export function logRawText(entries: LogEntry[]): string {
  return entries.map(entry => JSON.stringify(entry)).join('\n')
}
