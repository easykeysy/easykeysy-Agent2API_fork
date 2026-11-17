import { t } from '../i18n'
import { errorMessage, shared, toast } from './accounts-shared'
import { getStore, patch } from './accounts-store'

let refreshSubscribed = false

export function installListRefresh(): void {
  if (refreshSubscribed) return
  const app = shared().wbApp
  if (!app?.getRefreshStatus || !app.subscribeRefresh) return
  refreshSubscribed = true
  const getStatus = app.getRefreshStatus
  const sync = () => {
    const status = getStatus()
    const store = getStore()
    if (store.listBusy === status.pending && store.listError === status.error) return
    patch({ listBusy: status.pending, listError: status.error })
  }
  app.subscribeRefresh(sync)
  sync()
}

export async function refreshList(options: { feedback?: boolean } = {}): Promise<boolean> {
  installListRefresh()
  try {
    const refresh = shared().wbApp?.refresh
    if (!refresh) throw new Error(t('Account refresh is unavailable. Reopen the page'))
    const applied = await refresh()
    if (applied) void shared().wbProviders?.refreshCustom?.()
    if (options.feedback) {
      if (applied) toast(t('Account list refreshed'))
      else toast(t('Failed to refresh accounts: {message}', { message: getStore().listError || t('Request failed') }), 'err')
    }
    return applied
  } catch (error) {
    const message = errorMessage(error)
    patch({ listBusy: false, listError: message })
    if (options.feedback) toast(t('Failed to refresh accounts: {message}', { message }), 'err')
    return false
  }
}
