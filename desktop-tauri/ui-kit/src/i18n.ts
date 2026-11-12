/**
 * i18n （ in ，）。
 *
 * Why ui-kit need 1 、 not use （ui-islands/src/i18n.ts） ：
 * typecheck（directory node_modules）， to import will in 1 ——
 * has ，。
 *
 * when and t() is 1 window.wbI18n（ui/i18n.js ，head ）；
 * window not in or when when ， in ， not 。
 */

type TParams = Record<string, string | number>

/** ；params {name} */
export function t(key: string, params?: TParams): string {
  if (typeof window === 'undefined') return key
  // 、 not declare global： already window.wbI18n， when 1 
  // TS program when will TS2717（must）
  const api = (window as unknown as { wbI18n?: { t(key: string, params?: TParams): string } }).wbI18n
  return api ? (api.t(key, params) ?? key) : key
}
