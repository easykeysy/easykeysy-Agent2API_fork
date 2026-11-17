import { t } from '../i18n'
import { providerOf } from './accounts-domain'
import type { AccountRecord } from './accounts-shared'
import type { FlowPath, FlowSelection } from './routing-flow-types'
import type { ReplayRow, RoutingWorkspace } from './routing-types'
import { isTraceRunning, isTraceSuccess, traceAttempts } from './routing-trace-model'

/**
 * 客户端显示名。面板内置的模型调用在库里存的是内部用途标识（`assistant` /
 * `model_test` / `checkin_keepalive` 等），它们不是产品名，直接显示出来会让人
 * 以为是陌生的第三方客户端。这里统一翻成产品里的叫法，与请求日志同一口径
 * （见 ui-shared/model-api-key.ts 的 requestClientLabel）。
 */
export const clientLabel = (client: string): string => {
  switch (client) {
    case '': return t('Unidentified client')
    case 'other': return t('Other clients')
    case 'assistant': return t('Model chat')
    case 'assistant_images': return t('Assistant image tools')
    case 'assistant_title': return t('Assistant title summary')
    case 'model_test': return t('Model test')
    case 'checkin_keepalive': return t('Daily activity keepalive')
    default: return client
  }
}
export const clientIcon = (client: string) => ({ 'Claude Code': 'claude-code.svg', Codex: 'codex.png', ZCode: 'zcode.png' } as Record<string, string>)[client]
export const sessionId = (row: ReplayRow) => row.route?.sessionId || row.route?.stickyId || ''
export const requestModelId = (row: ReplayRow) => row.clientModel || row.model || ''
export const selectedRequest = (selection: FlowSelection) => selection?.kind === 'request' ? selection.row : selection?.kind === 'session' ? selection.session.latest : null
export function pathForRequest(row: ReplayRow): FlowPath {
  return { id: JSON.stringify([row.client || '', requestModelId(row), row.provider || '', row.accountId || '', row.accountId ? '' : row.accountName || '']),
    client: row.client || '', clientLabel: row.clientLabel, model: requestModelId(row), provider: row.provider || '', accountId: row.accountId || '', accountName: row.accountName || '',
    requests: 1, running: Number(isTraceRunning(row)), successful: Number(isTraceSuccess(row)), failed: Number(!isTraceRunning(row) && !isTraceSuccess(row)),
    sessions: Number(Boolean(sessionId(row))), stickyHits: Number(Boolean(row.route?.stickyHit)), switched: Number((row.attempts || 1) > 1), lastSeen: row.ts }
}
export function matchesSelection(path: FlowPath, selection: FlowSelection): boolean {
  if (!selection) return true
  if (selection.kind === 'path') return path.id === selection.id
  if (selection.kind === 'edge') return path.model === selection.model && (selection.client == null || path.client === selection.client)
    && (selection.accountId == null || path.accountId === selection.accountId) && (selection.provider == null || path.provider === selection.provider)
  if (selection.kind === 'account') return path.accountId === selection.id
  if (selection.kind === 'client') return path.client === selection.id
  if (selection.kind === 'model') return path.model.toLowerCase() === selection.id.toLowerCase()
  if (selection.kind === 'provider') return path.provider === selection.id
  const row = selectedRequest(selection)
  return Boolean(row && path.id === pathForRequest(row).id)
}
export const sumPaths = (paths: FlowPath[], key: 'requests' | 'running' | 'sessions' | 'stickyHits') => paths.reduce((sum, path) => sum + path[key], 0)
export function accountParticipation(workspace: RoutingWorkspace, account: AccountRecord): boolean {
  const profile = workspace.profile, provider = providerOf(account)
  return (profile.providerScope === 'all' || profile.providers.includes(provider))
    && !profile.disabledProviders.includes(provider) && !profile.excludedAccounts.includes(account.id)
}
export function providerFlowReason(workspace: RoutingWorkspace, provider: string): string {
  const profile = workspace.profile
  if (profile.disabledProviders.includes(provider) || profile.providerScope !== 'all' && !profile.providers.includes(provider)) return t('Excluded by current configuration')
  if (workspace.providers.find(item => item.id === provider)?.available === false) return t('Model binding unavailable')
  const accounts = workspace.accounts.filter(account => providerOf(account) === provider)
  if (!accounts.length) return t('No accounts participating in routing')
  const participants = accounts.filter(account => accountParticipation(workspace, account))
  if (!participants.length) return t('No accounts participating in routing')
  return participants.every(account => account.enabled === false) ? t('All accounts disabled') : ''
}
export function flowProviderGroups(workspace: RoutingWorkspace, paths: FlowPath[], selection: FlowSelection, focused: boolean) {
  const request = selectedRequest(selection)
  const accountProviders = new Map(workspace.accounts.map(account => [account.id, providerOf(account)]))
  const attempted = request ? traceAttempts(request).map(attempt => attempt.provider || accountProviders.get(attempt.accountId || '') || '') : []
  const providers = [...new Set([...workspace.profile.providers, ...workspace.providers.map(provider => provider.id), ...paths.map(path => path.provider), ...attempted])].filter(Boolean)
  const selected = new Set(selection ? paths.filter(path => matchesSelection(path, selection)).map(path => path.provider).filter(Boolean) : [])
  if (request) attempted.filter(Boolean).forEach(provider => selected.add(provider))
  if (selection?.kind === 'provider') selected.add(selection.id)
  if (selection?.kind === 'account') {
    const provider = accountProviders.get(selection.id)
    if (provider) selected.add(provider)
  }
  if (selection?.kind === 'edge' && selection.provider) selected.add(selection.provider)
  const running = new Map<string, number>()
  for (const account of workspace.accounts) {
    const provider = providerOf(account)
    running.set(provider, (running.get(provider) || 0) + (workspace.metrics.accountRunning?.[account.id] || 0))
  }
  const pathRunning = new Map<string, number>()
  for (const path of paths) pathRunning.set(path.provider, (pathRunning.get(path.provider) || 0) + path.running)
  for (const [provider, count] of pathRunning) running.set(provider, Math.max(running.get(provider) || 0, count))
  const reasons = new Map(providers.map(provider => [provider, providerFlowReason(workspace, provider)]))
  const recent = new Set(paths.filter(path => path.requests > 0).map(path => path.provider))
  const priority = (provider: string) => selected.has(provider) ? 0 : running.get(provider) ? 1 : reasons.get(provider) ? 4 : recent.has(provider) ? 2 : 3
  const visible = providers.filter(provider => !focused || !selection || selected.has(provider)).sort((left, right) => priority(left) - priority(right))
  return { primary: visible.filter(provider => priority(provider) < 4), inactive: visible.filter(provider => priority(provider) === 4), selected, running, reasons }
}
export function accountFlowState(workspace: RoutingWorkspace, account: AccountRecord, now: number) {
  const fact = workspace.accountStates?.[account.id]
  const entry = [...workspace.queue.ordered, ...workspace.queue.blocked].find(entry => entry.id === account.id)
  if (!workspace.model && fact?.limitedModels) return { state: 'partial', label: t('{n} models rate-limited', { n: fact.limitedModels }), reason: t('Select a request model to see its rate-limit state'), resetAt: 0 }
  if ((fact?.resetAt || 0) > now) return { state: 'cooldown', label: t('Rate-limited for this model'), reason: fact?.model || '', resetAt: fact!.resetAt }
  if (account.enabled === false) return { state: 'disabled', label: t('Disabled'), reason: '', resetAt: 0 }
  if (account.hasCredentials === false) return { state: 'no-credentials', label: t('No credentials'), reason: '', resetAt: 0 }
  if (!accountParticipation(workspace, account)) return { state: 'excluded', label: t('Excluded by current configuration'), reason: '', resetAt: 0 }
  const state = entry?.state || 'unready'
  const label = state === 'usable' ? workspace.metrics.accountRunning?.[account.id] ? t('Current handling account') : t('Available, on standby') : state === 'concurrency' ? t('Concurrency limit reached') : state === 'limiter' ? t('Skipped by limiter') : state === 'cooldown' ? t('Cooling down') : t('Not ready')
  return { state, label, reason: entry?.reason || '', resetAt: entry?.resetAt || 0 }
}
