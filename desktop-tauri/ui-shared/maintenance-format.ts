import { t } from '../ui-kit/src/i18n'
import type { MaintenanceItem, MaintenanceRun, MaintenanceRunStatus, MaintenanceTaskId } from './maintenance-types'

export const maintenanceLabel = (id: MaintenanceTaskId): string => id === 'credentialMaintenance' ? t('Credential maintenance') : t('Model catalog update')
export const sourceLabel = (source: string): string => source === 'manual' ? t('Manual') : source === 'passive' ? t('Client-triggered') : t('Automatic')
export const statusLabel = (status: MaintenanceRunStatus): string => ({
  running: t('Running…'), completed: t('Completed'), partial: t('Partially failed'), failed: t('Failed'),
  skipped: t('All skipped'), interrupted: t('Interrupted'),
})[status] ?? status
export const statusTone = (status: MaintenanceRunStatus): 'success' | 'warning' | 'destructive' | 'outline' =>
  status === 'completed' ? 'success' : status === 'partial' || status === 'interrupted' ? 'warning' : status === 'failed' ? 'destructive' : 'outline'
export const resultLabel = (status: MaintenanceItem['status']): string => status === 'refreshed' ? t('Success') : status === 'failed' ? t('Failed') : t('Skipped')
export const stamp = (value: number | null | undefined): string => value ? new Date(value).toLocaleString() : '—'
export const duration = (value: number | null | undefined): string => value == null ? '—' : t('{n} s', { n: Number((value / 1000).toFixed(1)) })
export const hasFailures = (run: MaintenanceRun): boolean => run.failed > 0 || run.metadata?.status === 'failed'

export function maintenanceMessage(message: string): string {
  const known: Record<string, string> = {
    'Credentials are not expired or close to expiry': t('Credentials are not expired or close to expiry'),
    'Sign-in authorization has expired. Sign in to this account again.': t('Sign-in authorization has expired. Sign in to this account again.'),
    'Unregistered provider': t('Unregistered provider'),
    'This provider does not support credential renewal': t('This provider does not support credential renewal'),
    'The account no longer exists; maintenance was not performed': t('The account no longer exists; maintenance was not performed'),
    'The provider is unavailable; its catalog was not updated': t('The provider is unavailable; its catalog was not updated'),
    'Using the cache; the next update is not due yet': t('Using the cache; the next update is not due yet'),
    'Using cached data, or no catalog source is currently available': t('Using cached data, or no catalog source is currently available'),
    'No account is available for fetching the catalog': t('No account is available for fetching the catalog'),
    'The selected account belongs to another provider, is disabled, or has no usable credentials': t('The selected account belongs to another provider, is disabled, or has no usable credentials'),
    'models.dev metadata is disabled in Settings': t('models.dev metadata is disabled in Settings'),
    'The task is already running. Please wait.': t('The task is already running. Please wait.'),
    'This task just ran. Please retry shortly.': t('This task just ran. Please retry shortly.'),
    'The run was interrupted. Some changes may have taken effect; check the account status first.': t('The run was interrupted. Some changes may have taken effect; check the account status first.'),
    'The previous run was interrupted and has no complete results. Check the account status.': t('The previous run was interrupted and has no complete results. Check the account status.'),
    'Maintenance ran, but the complete results could not be saved. Check account status before running it again.': t('Maintenance ran, but the complete results could not be saved. Check account status before running it again.'),
  }
  return Object.prototype.hasOwnProperty.call(known, message) ? known[message] : t(message)
}

export function itemMessage(item: MaintenanceItem, taskId: MaintenanceTaskId): string {
  if (item.fixed) return t('This provider uses a fixed model list')
  if (item.message) return maintenanceMessage(item.message)
  if (item.status === 'refreshed') return taskId === 'credentialMaintenance' ? t('Renewed and confirmed that credentials were updated') : t('Latest model catalog fetched')
  if (item.status === 'skipped') return t('No update needed; keeping the current state')
  return t('No failure reason was returned')
}

export function itemChange(item: MaintenanceItem, taskId: MaintenanceTaskId): string {
  if (item.status !== 'refreshed') return t('Unchanged')
  if (taskId === 'modelRefresh') return item.count == null ? '—' : t('{n} models', { n: item.count })
  return item.expiresBefore && item.expiresAfter && item.expiresAfter > item.expiresBefore ? t('Expiry extended') : t('Credentials updated')
}
