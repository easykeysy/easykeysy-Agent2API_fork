import { t } from '../i18n'
import { shared, type PoolItem } from './accounts-shared'

type ProxyPoolData = {
  items: PoolItem[]
}

/** already （`null` = no ）； down use empty 1 */
let poolCache: PoolItem[] | null = null
/** in （ and use and 1 ） */
let poolInflight: Promise<ProxyPoolData> | null = null
/** ：`acceptProxyPool` 1 +1（used for「 in to old response new 」） */
let writeSeq = 0
/** failure Reason（success after empty）—— 「failure」 */
let lastPoolError: string | null = null

/** ： no has ，「failure → → 」 will */
let retryAt = 0
let bridgeWarned = false

let lastSignature: string | null = null

function signatureOf(items: PoolItem[]): string {
  return items
    .map(item => [
      item.id,
      item.name || '',
      item.enabled === false ? 0 : 1,
      item.protocol || '',
      item.host || '',
      item.resolved?.protocol || '',
      item.resolved?.host || '',
      item.resolved?.port ?? '',
      item.port ?? '',
      item.resolveError ?? '',
      item.usedBy?.length ?? 0,
      item.usedByServices?.join(',') ?? '',
    ].join(':'))
    .join('|')
}

/** and up etc ？ */
function unchanged(items: PoolItem[]): boolean {
  return lastSignature !== null && signatureOf(items) === lastSignature
}

function remember(items: PoolItem[]): void {
  lastSignature = signatureOf(items)
}

const listeners = new Set<() => void>()

export function subscribeProxyPool(listener: () => void): () => void {
  listeners.add(listener)
  return () => { listeners.delete(listener) }
}

function emit(): void {
  listeners.forEach(listener => listener())
}

/** already （`null` = no ） */
export function proxyPoolSnapshot(): PoolItem[] | null {
  return poolCache
}

/** failure Reason（`null` = no failure） */
export function poolError(): string | null {
  return lastPoolError
}

/** （ down 1 use will new ） */
export function invalidateProxyPoolCache(): void {
  poolCache = null
  // 1 ： after can can Reason「 in 」， that 
  // to must 1 ，otherwise will in 「 in 」 up 
  lastSignature = null
}

export function acceptProxyPool(items: PoolItem[] | null | undefined): void {
  if (!Array.isArray(items)) return
  poolCache = items
  writeSeq += 1
  lastPoolError = null
  retryAt = 0
  if (unchanged(items)) return
  remember(items)
  emit()
}

/**
 * 1 （； and use and 1 ）。
 *
 * when can can resolve `undefined`（ not is 
 * reject）， that by failure —— not will 「 1 no has 」。
 */
export async function proxyPoolOptions(options: { force?: boolean } = {}): Promise<PoolItem[]> {
  if (!options.force && poolCache) return poolCache
  // request that 1 ： when if already ， up this response old 
  const seqAtStart = writeSeq
  if (!poolInflight) {
    poolInflight = (async (): Promise<ProxyPoolData> => {
      try {
        const data = await shared().workbuddyDesktop?.getProxyPool?.()
        if (!data || typeof data !== 'object' || !Array.isArray(data.items)) {
          throw new Error(t('代理池响应异常（{type}）', { type: typeof data }))
        }
        // items is ， this —— down not 1 
        return { items: data.items }
      } finally {
        poolInflight = null
      }
    })()
  }
  const data = await poolInflight
  const items = data.items
  // in can can has already accept update （ 1 ）：
  // that this to old response new ， not can 
  const fresh = writeSeq === seqAtStart || poolCache === null
  lastPoolError = null
  if (!fresh && poolCache) return poolCache
  poolCache = items
  // to ：「 new 」、， will after 
  // down （ is ， no has not to new ）
  if (!unchanged(items)) {
    remember(items)
    emit()
  }
  return poolCache
}

/**
 * ： no to （ 30 seconds）。
 *
 * successpath in `proxyPoolOptions` （ to emit）； this failure ——
 * failure not ， Reason， can 「failure（ in ）」。
 */
export function ensureProxyPoolOptions(): void {
  if (poolCache || Date.now() < retryAt) return
  if (typeof shared().workbuddyDesktop?.getProxyPool !== 'function') {
    // no up （failure etc ）： will not ， and 1 
    if (!bridgeWarned) {
      bridgeWarned = true
      console.warn('[accounts] 桥未提供 getProxyPool：代理列的「已保存的代理」选项不可用')
    }
    retryAt = Infinity
    return
  }
  retryAt = Date.now() + 30_000
  void proxyPoolOptions()
    .then(() => { retryAt = 0 })
    .catch(error => {
      lastPoolError = error instanceof Error ? error.message : String(error)
      emit()
    })
}

/**
 * 1 （ when ， accounts-data refreshProxyPool）。
 *
 * and `ensureProxyPoolOptions` ： this ** 1 **request（）， use 
 * 「 in use 」 or 「 this no 」 。
 *
 * ** not **： will in request in that 1 「 in …」，
 * request no when （ ） not will —— that 1 in 「 in 」
 * up 。 old ， to new 、；failure when old in ，empty has use 。
 */
export async function refreshProxyPool(): Promise<void> {
  retryAt = 0
  try {
    await proxyPoolOptions({ force: true })
  } catch (error) {
    lastPoolError = error instanceof Error ? error.message : String(error)
    emit()
  }
}
