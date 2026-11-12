/*
 * Agent2API Service Worker。
 *
 * ── Why in `ui/` 、 not is `ui/m/` ──────────────────────
 * SW default scope limit is in directory：`ui/m/sw.js` can 
 * `/m/*`， is `/m.html` —— in scope ， `/`
 * will （`The path of the provided scope is not under the max
 * scope`）， `/m/` not to 。 in `ui/` directory no has this 
 * ：`/sw.js` can scope to `/`。
 *
 * （vite.config.ts swBuildStamp） `mobile/public/sw.js`
 * to this and when ， this file is —— file。
 *
 * ── ， 1 ─────────────────────────────
 * scope is `/`， down fetch `/m` and `/m/*`， its request
 * return（ by ）。 and `/api/*` not 。
 *
 * ── ， by ────────────────────────────────────
 * · ****（`/m`、`/m.html`）：network-first + 。
 * not can stale-while-revalidate —— JS，
 * to old in after update will 「 new JS + old HTML」 1 。
 * · ****（mobile.js / mobile.css / manifest / ）：stale-while-revalidate。
 * seconds， after new to down 。
 * · ****（`/api/*`、`/v1/*`）：** not **。must when ，
 * 1 will use to as is 。
 *
 * ── ────────────────────────────────────────────────────
 * SW update is file。 sw.js not when not will SW，
 * old will 1 —— when update。
 * new SW after not skipWaiting， is etc —— otherwise use in 
 * will 1 new 。 in set to 「update」 by 。
 */

const CACHE = 'agent2api-mobile-__BUILD__'

/** file（ for SW in directory， `ui/` ） */
const SHELL = [
  './m.html',
  './m/mobile.js',
  './m/mobile.css',
  './m/manifest.webmanifest',
  './i18n.js',
  './aliyun-captcha.js',
  './assets/app-icon-192.png',
  './assets/app-icon-512.png',
  './assets/app-icon-maskable-512.png',
]

/** this request not （ need not need ） */
function isMobileAsset(pathname) {
  if (pathname === '/m' || pathname === '/m.html') return true
  return pathname.startsWith('/m/') || pathname === '/i18n.js' || pathname === '/aliyun-captcha.js' || pathname.startsWith('/i18n/')
}

/** is not is （`/m` or `/m.html`， query / hash path） */
function isEntryNavigation(request, url) {
  if (request.mode !== 'navigate') return false
  return url.pathname === '/m' || url.pathname === '/m.html'
}

/**
 * 1 file。
 *
 * not use `cache.add` is as will ** to **： when `/m.html` will 302 to 
 * `/login`，`cache.add` will that —— after is 。
 * this 1 ， to response（ to down in when ）。
 */
async function precache(cache, item) {
  try {
    const response = await fetch(item, { cache: 'no-store' })
    if (response.ok && !response.redirected) await cache.put(item, response)
  } catch {
    /* filefailure not its （e.g.） */
  }
}

self.addEventListener('install', event => {
  event.waitUntil(
    caches
      .open(CACHE)
      // not is addAll： 1 file not this failure
      .then(cache => Promise.all(SHELL.map(item => precache(cache, item))))
      .then(() => {
        // not in this skipWaiting： etc when （ message ）
      }),
  )
})

self.addEventListener('activate', event => {
  event.waitUntil(
    caches
      .keys()
      .then(keys => Promise.all(keys.filter(key => key !== CACHE).map(key => caches.delete(key))))
      .then(() => self.clients.claim()),
  )
})

/**
 * 。
 *
 * `SKIP_WAITING`： use 「update」。 new SW ， after 
 * reload to new 。
 */
self.addEventListener('message', event => {
  const data = event.data
  if (data && data.type === 'SKIP_WAITING') {
    void self.skipWaiting()
  }
})

self.addEventListener('fetch', event => {
  const request = event.request
  if (request.method !== 'GET') return

  const url = new URL(request.url)
  if (url.origin !== self.location.origin) return
  // and 1 not 
  if (!isMobileAsset(url.pathname)) return

  // ：network-first，
  if (isEntryNavigation(request, url)) {
    event.respondWith(
      fetch(request)
        .then(response => {
          // 「 in 」 response。 will when will 302 to /login，
          // fetch to after to is 1 200 —— 
          // ， down to is （ not ）。
          if (response.ok && !response.redirected) {
            const copy = response.clone()
            void caches
              .open(CACHE)
              .then(cache => cache.put('./m.html', copy))
              .catch(() => undefined)
          }
          return response
        })
        .catch(async () => {
          const cached = await caches.match('./m.html')
          if (cached) return cached
          // no has （）： to 1 can ，error
          return new Response(
            '<!doctype html><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">' +
              '<body style="font:15px/1.7 -apple-system,system-ui,sans-serif;padding:40px 24px;color:#333">' +
              '<h1 style="font-size:18px;margin:0 0 8px">离线</h1>' +
              '<p style="margin:0;color:#666">连不上网关。确认手机与网关在同一局域网，然后刷新。</p>' +
              '</body>',
            { headers: { 'Content-Type': 'text/html; charset=utf-8' } },
          )
        }),
    )
    return
  }

  const isShell = SHELL.some(item => url.pathname.endsWith(item.slice(1)))
  if (isShell) {
    // ：stale-while-revalidate —— seconds， after new to down 
    event.respondWith(
      caches.open(CACHE).then(async cache => {
        const cached = await cache.match(request)
        const network = fetch(request)
          .then(response => {
            if (response.ok) void cache.put(request, response.clone())
            return response
          })
          .catch(() => cached)
        return cached || network
      }),
    )
    return
  }

  // its ：network-first， when 
  event.respondWith(
    caches.open(CACHE).then(async cache => {
      try {
        const response = await fetch(request)
        if (response.ok && !response.redirected) {
          const copy = response.clone()
          event.waitUntil(cache.put(request, copy).catch(() => undefined))
        }
        return response
      } catch {
        const cached = await cache.match(request)
        if (cached) return cached
        throw new Error('offline')
      }
    }),
  )
})
