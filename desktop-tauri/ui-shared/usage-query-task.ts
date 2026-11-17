export type UsageQueryStatus = 'running' | 'success' | 'failed' | 'timeout' | 'cancelled' | 'superseded' | 'config_changed'

export type UsageQueryTask = {
  id: string
  requestId: string
  status: UsageQueryStatus
  error?: string | null
  startedAt: number
  deadlineAt: number
}

export type UsageQueryStart = { queries: UsageQueryTask[]; skipped?: number }

export const USAGE_QUERY_POLL_MS = 800

export function usageTaskText(task: UsageQueryTask | undefined, t: (key: string) => string): string {
  switch (task?.status) {
    case 'running': return t('Checking…')
    case 'cancelled': return t('Query cancelled')
    case 'config_changed': return t('Settings changed. Query again to update the balance.')
    case 'failed': return task.error ? t(task.error) : t('Query failed')
    case 'timeout': return t('Query timed out. You can try again.')
    default: return ''
  }
}

export function usageQueryDeadline<T>(promise: Promise<T>, milliseconds = 15_000): Promise<T> {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error('Timed out waiting for the balance query response; please query again')), milliseconds)
    promise.then(value => { clearTimeout(timer); resolve(value) }, error => { clearTimeout(timer); reject(error) })
  })
}
