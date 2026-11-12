import type { CapacitorConfig } from '@capacitor/cli'

/**
 * Capacitor config（Android APK + iOS IPA use 1 Web ）。
 *
 * ── Why androidScheme is https， cleartext ──────────
 * this not ， 1 ：
 * · `androidScheme: 'https'` ** use ** origin（`https://localhost`）。
 * must is up down ，otherwise `crypto.subtle` unavailable —— 
 * need use SHA-256。 http will JS ， and no need 。
 * · `cleartext: true` + manifest `usesCleartextTraffic` ** can not can 
 * http:// **。 is http（ no has ）， not this 
 * allrequest will Android 9+ 。
 *
 * is https、request is http —— this is mixed content，WebView default。
 * need `allowMixedContent: true`。 「 not used for」， this 
 * use is 「 http 」， no has ； and is 
 * ， not in 。
 *
 * ── iOS this Why not can androidScheme ──────────────────────
 * `iosScheme` not is `https` / `http` —— that is WKWebView ，
 * Capacitor in `InstanceDescriptor.normalize()` will as and ****
 * default `capacitor`（ @capacitor/ios CAPInstanceDescriptor.swift:169）。
 * iOS origin is `capacitor://localhost`。
 *
 * is not 「 up down 」，`crypto.subtle` in iOS up unavailable —— 
 * will `src/api/auth.ts` JS SHA-256 （ that path
 * is as http ， is 1 ， 1 ）。
 *
 * ── iOS http ──────────────────────────────
 * `ios/App/App/Info.plist` `NSAppTransportSecurity`（ATS）， not in 
 * this 。 is IP + http，iOS 17 ATS default IP ，
 * that when `NSAllowsArbitraryLoads` and `NSAllowsLocalNetworking`。
 *
 * ── Why not use server.url to ────────────────────────────
 * that App 1 ： need WebView，
 * is that ， and when not 。 this need is 「 in 
 * + can + 」， + request。
 */
const config: CapacitorConfig = {
  appId: 'com.agent2api.mobile',
  appName: 'Agent2API',
  // directory（npm run build:native ，Android / iOS use ）
  webDir: 'android-www',
  android: {
    // is http， not this request 
    allowMixedContent: true,
  },
  ios: {
    // by env(safe-area-inset-*) ：WKWebView not 
    // 1 ，otherwise / will ， 1 empty。
    contentInset: 'never',
    // and Android 1 ： to ， down 
    backgroundColor: '#f3f3f6',
  },
  server: {
    // origin use https://localhost： up down ，crypto.subtle available
    androidScheme: 'https',
    // http 
    cleartext: true,
  },
  plugins: {
    Keyboard: {
      // iOS default native will WebView ， fixed / flex
      // will 1 down ；body body ， visualViewport 
      // （Shell.tsx useKeyboardOpen）。
      resize: 'body',
      // Android when not will resize， need need 
      resizeOnFullScreen: true,
    },
  },
}

export default config
