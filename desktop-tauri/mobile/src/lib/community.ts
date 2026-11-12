/**
 * 。
 *
 * ── Why 1 ────────────────────────────────────────────
 * 1 in `ui-islands/src/islands/update-shared.ts` 
 * `QQ_GROUPS` / `COMMUNITY_LINKS`。 is （ Vite ），
 * not can import that file（ ）。** / when 1 **
 * —— this and that is 1 。
 *
 * ── Why to ， not to ────────────────────────────────
 * and 1 by （ that `QQ_GROUPS` ）： is ，
 * / 1 ； + by 1 can 。
 */

export type QqGroup = { no: string; nearlyFull: boolean }

export const QQ_GROUPS: readonly QqGroup[] = [
  { no: '1127656296', nearlyFull: true },
  { no: '1130894674', nearlyFull: true },
  { no: '1108276647', nearlyFull: false },
]

export type CommunityLink = { id: 'telegram'; url: string }

export const COMMUNITY_LINKS: readonly CommunityLink[] = [
  { id: 'telegram', url: 'https://t.me/+9gM8YPvgRL02NGRl' },
]

/**
 * use 1 。
 *
 * ── Why APK is 「」 not is window.open ─────────────
 * Capacitor WebView no supports，`window.open` will ignore（return
 * null， not ）。 will `BridgeWebViewClient` 
 * `shouldOverrideUrlLoading` → `bridge.launchIntent`： not is use 
 * origin when ， 1 `ACTION_VIEW` Intent to ， by default，
 * WebView in 。 this use `location.href`。
 *
 * PWA no has that ，`window.open` is for （ new ， not current）。
 */
export function openExternal(url: string): void {
  const target = String(url || '').trim()
  if (!/^https?:\/\//i.test(target)) return
  const packaged = Boolean(
    (window as Window & { Capacitor?: { isNativePlatform?: () => boolean } }).Capacitor?.isNativePlatform?.(),
  )
  if (packaged) {
    window.location.href = target
    return
  }
  window.open(target, '_blank', 'noopener')
}
