/**
 * i18n ： by ui/i18n.js in head （window.wbI18n / window.wbI18nDict），
 * bundle not 。 in ，。
 *
 * （islands/ui.js） is 、 in head i18n.js after ， this 
 * window.wbI18n total is ； has when （ has to i18n.js before ） will to 
 * undefined —— that when ， in ， not 。
 *
 * t()： / / in ui/i18n.js up ， need need 
 * window.wbI18n（set 「」 down ）， not in this 1 。
 */

type TParams = Record<string, string | number>

type WbI18nApi = {
  t(key: string, params?: TParams): string
  locale(): string
  rawLocale(): string
  setLocale(value: string): void
  sweep(root?: ParentNode): void
  LOCALES: ReadonlyArray<{ code: string; label: string; nameKey: string }>
}

declare global {
  interface Window {
    /** and （ui/i18n.js ） */
    wbI18n?: WbI18nApi
    /** when ： by ui/i18n/<locale>/<domain>.js Object.assign up */
    wbI18nDict?: Record<string, string>
    /** ： by ui/i18n/manifest.js */
    __WB_I18N_DOMAINS?: string[]
  }
}

/** ；params {name} 。 / none when 。 */
export function t(key: string, params?: TParams): string {
  const api = typeof window !== 'undefined' ? window.wbI18n : undefined
  if (api && typeof api.t === 'function') return api.t(key, params)
  return key
}
