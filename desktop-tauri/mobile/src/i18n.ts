/**
 * i18n 。
 *
 * and ui-islands/src/i18n.ts 1 ： by `ui/i18n.js` in head 
 * to `window.wbI18nDict`，、 not 。 in ，
 * 。 and use 1 ，（、 to 、…）
 * has ， not 1 ARB。
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
    wbI18n?: WbI18nApi
    wbI18nDict?: Record<string, string>
    __WB_I18N_DOMAINS?: string[]
  }
}

/** ；params {name} 。 / none when */
export function t(key: string, params?: TParams): string {
  const api = typeof window !== 'undefined' ? window.wbI18n : undefined
  if (api && typeof api.t === 'function') return api.t(key, params)
  // when when 1 ： in before ，
  // when down （i18n.js ） can 
  const dict = typeof window !== 'undefined' ? window.wbI18nDict : undefined
  const hit = dict?.[key]
  let text = typeof hit === 'string' && hit ? hit : key
  if (params) {
    for (const [name, value] of Object.entries(params)) {
      text = text.split(`{${name}}`).join(String(value))
    }
  }
  return text
}

/** current（`zh-Hans` / `en` / …） */
export function locale(): string {
  const api = typeof window !== 'undefined' ? window.wbI18n : undefined
  if (api && typeof api.locale === 'function') return api.locale()
  return 'zh-Hans'
}

export type LocaleOption = { code: string; label: string; nameKey: string }

/** can （， not ） */
export function localeOptions(): readonly LocaleOption[] {
  const api = typeof window !== 'undefined' ? window.wbI18n : undefined
  if (api && Array.isArray(api.LOCALES)) return api.LOCALES
  return [
    { code: 'zh-Hans', label: '简体中文', nameKey: '简体中文' },
    { code: 'zh-Hant', label: '繁體中文', nameKey: '繁体中文' },
    { code: 'en', label: 'English', nameKey: '英语' },
    { code: 'ja', label: '日本語', nameKey: '日语' },
    { code: 'ko', label: '한국어', nameKey: '韩语' },
    { code: 'pt-BR', label: 'Português (BR)', nameKey: '葡萄牙语（巴西）' },
  ]
}

/** = new （ and 1 ， ui/i18n.js ） */
export function setLocale(code: string): void {
  const api = typeof window !== 'undefined' ? window.wbI18n : undefined
  if (api && typeof api.setLocale === 'function') {
    api.setLocale(code)
    return
  }
  try {
    localStorage.setItem('workbuddy-desktop-locale', code)
  } catch {
    /* ： new after to */
  }
  window.location.reload()
}
