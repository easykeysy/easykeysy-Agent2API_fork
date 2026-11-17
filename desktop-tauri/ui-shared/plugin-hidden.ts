import type { HiddenAccountProvider } from './plugins'
import { pluginPhaseDetail, pluginPhaseLabel } from './plugin-startup'

type Translate = (key: string, params?: Record<string, string | number>) => string
export type HiddenFilter = 'all' | 'loading' | 'missing' | 'disabled' | 'broken'
export const hiddenFailed = (item: HiddenAccountProvider) => item.reason === 'load_failed' || item.reason === 'load_timeout'
export function hiddenInFilter(item: HiddenAccountProvider, filter: HiddenFilter): boolean {
  if (filter === 'all') return true
  if (filter === 'loading') return item.reason === 'loading' || item.reason === 'not_loaded'
  if (filter === 'missing') return item.reason === 'not_installed'
  if (filter === 'disabled') return item.reason === 'pack_disabled' || item.reason === 'provider_disabled'
  return hiddenFailed(item)
}
export function hiddenLabel(item: HiddenAccountProvider, t: Translate): string {
  switch (item.reason) {
    case 'not_installed': return t('Plugin not installed')
    case 'pack_disabled': return t('Plugin pack disabled')
    case 'provider_disabled': return t('Provider disabled')
    case 'load_failed': return t('Load failed')
    case 'load_timeout': return t('Initialization timed out')
    case 'loading': return item.startup ? pluginPhaseLabel(item.startup, t) : t('Loading')
    default: return pluginPhaseLabel(item.startup, t)
  }
}
export function hiddenDetail(item: HiddenAccountProvider, t: Translate, now?: number): string {
  switch (item.reason) {
    case 'not_installed': return t('Reinstall this plugin and they return automatically')
    case 'pack_disabled': return t('Enable the plugin pack and they return automatically')
    case 'provider_disabled': return t('Enable this provider and they return automatically')
    default: return pluginPhaseDetail(item.startup, t, now)
  }
}
export function hiddenAction(item: HiddenAccountProvider, t: Translate): string {
  if (item.reason === 'not_installed') return t('Install')
  if (item.reason === 'pack_disabled' || item.reason === 'provider_disabled') return t('Enable')
  return hiddenFailed(item) || item.startup?.phase === 'blocked' ? t('View reason') : t('View progress')
}
