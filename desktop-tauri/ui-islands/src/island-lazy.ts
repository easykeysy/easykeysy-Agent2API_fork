/**
 * by need 。
 *
 * ── Why need need ────────────────────────────────────────────
 * by `island-registry` 1 eager ， in 
 * `mount()`。 is ** **： 
 * 、、、set、 to …… use in its in 1 。，
 * CPU、 and request。
 *
 * this 「」 and 「 to DOM」： in 1 bundle 
 * （IIFE not ）， React root in 1 to when 
 * create。（`.page[data-page=...]`） by 1 ，
 * in can to ； need etc is `app.js::showPage` `wb:page` 。
 *
 * ── Why not use 「current section has active 」 ────────────
 * when `showPage` no ，default overview active。 if by this 
 * ， use up in when overview will 1 。 1 etc 
 * `wb:page` ， and 1 。
 *
 * ── （set 「」「update」）──────────────────
 * this by set in React after create， that 1 can can not in 。
 * `tryMount` not to when 1 MutationObserver， etc ；
 * to when up 1 ，。
 */

type PendingMount = {
  selector: string
  mount: () => void
  done: boolean
}

const pending = new Map<string, PendingMount[]>()
const observers = new Map<string, MutationObserver>()
let activePage: string | null = null

/**
 * 1 ：`page` 1 when `mount`。
 *
 * can use （ / ） not will —— 1 
 * selector 1 success mount。
 */
export function mountOnPage(page: string, selector: string, mount: () => void): void {
  const entry: PendingMount = { selector, mount, done: false }
  const list = pending.get(page)
  if (list) list.push(entry)
  else pending.set(page, [entry])

  // 「 in already after 」 path（ not will to ，
  // and when can can ）。
  if (activePage === page) mountReady(page)
}

function tryMount(entry: PendingMount): void {
  if (entry.done) return
  const node = document.querySelector<HTMLElement>(entry.selector)
  if (!node) return
  entry.done = true
  try {
    entry.mount()
  } catch (error) {
    console.error('[islands] 页面挂载失败:', entry.selector, error)
  }
}

/** 1 already 、 and already up ； no to 。 */
function mountReady(page: string): void {
  const list = pending.get(page)
  if (!list || list.length === 0) return

  for (const entry of list) tryMount(entry)
  const waiting = list.filter(entry => !entry.done)
  if (waiting.length === 0) {
    pending.delete(page)
    observers.get(page)?.disconnect()
    observers.delete(page)
    return
  }

  // 1 already has 1 in etc ， not create。
  if (observers.has(page)) return
  const observer = new MutationObserver(() => {
    for (const entry of waiting) tryMount(entry)
    if (waiting.every(entry => entry.done)) {
      observer.disconnect()
      observers.delete(page)
      pending.delete(page)
    }
  })
  observer.observe(document.body, { childList: true, subtree: true })
  observers.set(page, observer)
}

/** ： up 1 ，current already 。 */
function activate(page: string): void {
  if (page !== activePage) {
    for (const [key, observer] of observers) {
      if (key === page) continue
      observer.disconnect()
      observers.delete(key)
    }
  }
  activePage = page
  mountReady(page)
}

// `app.js::showPage` in when ； by index.tsx in 
// by localStorage before 1 ， its etc 1 to 。
window.addEventListener('wb:page', event => {
  const page = (event as CustomEvent<unknown>).detail
  if (typeof page === 'string' && page) activate(page)
})
