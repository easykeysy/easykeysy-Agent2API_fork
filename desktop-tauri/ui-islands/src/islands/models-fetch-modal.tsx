import * as React from 'react'
import { createRoot } from 'react-dom/client'
import {
  Button,
  Dialog,
  DialogBody,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  InputGroup,
  InputGroupAddon,
  InputGroupInput,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@ui'
import { t } from '../i18n'
import { ModelsFetchError } from './models-fetch-error'
import { ModelsFetchTable, type RefreshItem, type Row } from './models-fetch-table'

/**
 * Agent2API · 「get」（ that by ）。
 *
 * ui/models-fetch-modal.js（innerHTML .modal-mask / .models-table that ）；
 * for and ** 1 **：
 * `window.wbModelsFetchModal.open({ providerId, custom, name, onDone, onRefreshed })`，
 * use （models-panel.js refreshModels） 1 not use 。
 *
 * ── ， ────────────────────────────────────
 * · ****： by use ，「 → → 」 —— id and 
 * for ， already in 「 already 」 and ** use **（ not ），default that ；
 * （models-custom-source.js addModels）。
 * · ****： 1 directory， up has has ， no has 「」 this 1 —— 
 * 「 new directory + 」（ old has 1 3.5 seconds toast）。
 *
 * ── ** not ** ─────────────────────────────────────
 * 1 is ** up **（ 1 request）， use can can is 1 when 、
 * or 1 down 「」 use ； etc 「 1 」 1 new ，「」 and 
 * 「 up 」 in up not 。 1 get ， by use that 「get」。
 *
 * ── ： 1 ， not ─────────────────────────────
 * that is ****； 1 has 1 「get / new get」—— request `providers` 
 * this 1 （ after up ， not 、 not ， api/models.rs `parse_providers`），
 * after this 1 ** and **， its 。 by ：directory new is 1 1 up ， use 
 * is to 1 、 or 1 —— as this 1 1 。
 *
 * is ** 1 **（ buildRows）：get and already has 1 ， = 
 * （= ； after and ， providers::PROVIDERS and implemented_kinds）。
 *
 * ── is 「 after ， that this 1 」──────────
 * use ， not ：`onDone` in success after 、`onRefreshed` in 
 * directory**** after （`result.refreshed > 0`；failure / when 1 no ， not ）。
 *
 * ── and old ****（ ， not ）────────────
 * 1. id （`fm-table-intl` / `fm-table-custom`）：（table-columns.js） is by 
 * this （ TABLES root），`col` / `th` `data-col` is ——
 * id will this and ； its DOM id 1 not 。
 * 2. from CSS to `<col>` up （ old CSS by `#fm-table-intl .f-provider` ， id 1 
 * ）； need `table-layout: fixed`（ old .models-table ），otherwise not 。
 * 3. 「」 1 localStorage ， by loadSavedSource。
 */

/* ─── ───────────────────────────────────── */

/** use to （ its not ） */
type Account = {
  id?: unknown; provider?: string; name?: string; nickname?: string; email?: string
  nameCustom?: boolean
  available?: boolean; enabled?: boolean
}

/** 1 （`wbProviders.all()`）；directory 1 （`customList()`） */
type ProviderMeta = { id?: string; label?: string }
type CustomProvider = { id?: string; models?: Array<{ id?: unknown }> }

type RefreshResponse = { results?: RefreshItem[]; refreshed?: unknown }

/** open() （ and old 1 ） */
type OpenOptions = {
  providerId: string
  /** by use ：true = */
  custom?: boolean
  /** （ use ）； providerId */
  name?: string
  /** ：success after ；：directory after （ ） */
  onDone?: () => void
  onRefreshed?: () => void
}

/** ： no / in / failure / has —— use not is ， is as 
 * 「 no 」 and 「failure」 in up （ not ）。 */
type Status = 'idle' | 'loading' | 'error' | 'ready'

/**
 * window up by its / its 。 use 「 + 」 not is declare global
 * Window up ： this is use ， if declare 1 ， and will 
 * not 1 TS2717 —— and when 。file declare 
 * wbModelsFetchModal（file）。
 */
type SharedWindow = {
  workbuddyDesktop?: {
    /** POST /api/models/refresh：`{accounts, providers}` → `{results, refreshed, …}` */
    refreshModels(payload: {
      accounts: Record<string, string>; providers: string[]
    }): Promise<RefreshResponse | null | undefined>
  }
  wbApp?: {
    toast?: (message: string, kind?: 'err' | 'ok') => void
    getState?: () => { accounts?: { accounts?: Account[] } } | null | undefined
    formatTime?: (value: unknown) => string
  }
  /** ：app.js ， old formatTime(...) */
  formatTime?: (value: unknown) => string
  wbProviders?: {
    all?: () => ProviderMeta[] // 内置家清单（同步读主状态，只有内置家）
    customList?: () => CustomProvider[] // 自定义家目录缓存（同步读）
    /** use API use （ that POST /api/custom-providers/fetch-models ） */
    customRequest?: (method: string, path: string, body?: unknown) => Promise<{ models?: unknown[] } | null | undefined>
  }
  wbModelsPanel?: {
    /** ** has ** （ new ） */
    builtinProviders?: () => string[]
    /** current when （ms，0 = unknown / from success） */
    providerRefreshedAt?: (providerId: string) => number
  }
  wbModelsCustom?: { addModels?: (id: string, modelIds: string[]) => Promise<unknown> } // 批量登记
  wbFilterMemory?: { save?: (key: string, patch: Record<string, string>) => void } // 只用它的 save
  wbAccountsModel?: {
    /** （emailAsName use ， accountLabel） */
    providerFeatures?: (provider?: string) => { emailAsName?: boolean } | undefined
    byPriorityOrder?: (a: Account, b: Account) => number // 账号页同一条优先级排序
  }
}

function shared(): SharedWindow {
  return window as unknown as SharedWindow
}

/* ─── ───────────────────────────────────── */

/**
 * directory new ** not ** （ and after `ProviderAdapter::refresh_uses_account` false 
 * ）：Cline none、 is ，「 use 」 for no has —— that 1 as empty
 * ， not is to 1 1 down 。
 */
const ACCOUNTLESS_REFRESH = new Set(['cline-free', 'cline-pass'])

/** 「」 ：`{providerId: accountId}`（ loadSavedSource ） */
const SOURCE_KEY = 'workbuddy-desktop-fetch-model-source'

/* ─── ──────────────────────────────── */

/** ：empty + not （ and after and its 1 ） */
function norm(value: unknown): string {
  return String(value ?? '').trim().toLowerCase()
}

/** / toast 1 ：toast seconds after no ， 1 */
function toast(message: string, kind?: 'err' | 'ok') {
  shared().wbApp?.toast?.(message, kind)
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}

/** when → when （app.js formatTime：0 and returnempty， use ）。 and 
 * wbApp.formatTime is ， to use —— can can app.js ，
 * when 1 in use after ， already 。 */
function formatTime(value: number): string {
  const fn = shared().formatTime ?? shared().wbApp?.formatTime
  return String(fn?.(value) ?? '')
}

/** （ and old 1 ， not ） */
function accounts(): Account[] {
  return shared().wbApp?.getState?.()?.accounts?.accounts || []
}

/** this current id（directory； has ） */
function managedIds(providerId: string): Set<string> {
  const list = shared().wbProviders?.customList?.() || []
  const models = list.find(item => item.id === providerId)?.models
  return new Set((Array.isArray(models) ? models : []).map(model => norm(model?.id)).filter(Boolean))
}

/** ： and 1 （displayNameOf）—— （nameCustom）
 * use ； old ： （Qoder / AutoClaw /
 * Accio） use ， its / / / id。 */
function accountLabel(account: Account): string {
  if (account?.nameCustom === true && account?.name) return account.name
  const email = String(account?.email || '').trim()
  if (shared().wbAccountsModel?.providerFeatures?.(account?.provider)?.emailAsName && email) return email
  const name = String(account?.nickname || account?.name || email || account?.id || '').trim()
  return name || t('未命名账号')
}

/** this availabledirectory （「」 down ）。 = `available`（ after ：enabled + has 
 * ）+ `enabled`； use 1 `byPriorityOrder`（ before and after `order_key` is 
 * ， 「 1 」 is 1 ）。， not 。 */
function sourceAccounts(providerId: string): Account[] {
  if (ACCOUNTLESS_REFRESH.has(providerId)) return []
  const order = shared().wbAccountsModel?.byPriorityOrder
  const list = accounts().filter(account => (account?.provider || 'workbuddy') === providerId
    && account?.available !== false && account?.enabled !== false)
  return order ? list.sort(order) : list
}

/**
 * new ：** has ∪ has enabled **。
 *
 * and 「」 down 1 （ sourceAccounts）： use in up not to （ no has enabled、
 * as empty） not this new —— will to 1 「」， use no has optional、
 * no has this ，none from 。request to after （ api/models.rs providers）： not 、
 * not 。
 *
 * 「 has enabled 」 and ， is as 「、 is empty 」 this 1 ： when not in ， use up 
 * this to —— 1 has 。（） when not ：，
 * new will 。
 */
function scopeProviders(): string[] {
  const ids = new Set(shared().wbModelsPanel?.builtinProviders?.() || [])
  for (const account of accounts()) {
    if (account?.available === false || account?.enabled === false) continue
    ids.add(account?.provider || 'workbuddy')
  }
  return [...ids]
}

/** get that 1 need ： by （ and 1 ） scoped —— `all()` has 
 * ， id scopeProviders not will in this 。 */
function pendingProviders(): ProviderMeta[] {
  const scoped = new Set(scopeProviders())
  return (shared().wbProviders?.all?.() || []).filter(item => scoped.has(item?.id ?? ''))
}

/**
 * 「」 ：`{providerId: accountId}` for 。
 *
 * Why localStorage not is ： this is 1 （「 use this directory」）， not and 
 * ， after no has ；「 up that 」 need ， can 。
 *
 * and old has 1 ****： old is `wbFilterMemory.load(SOURCE_KEY, {})`， that 
 * load will ** by defaults **（`for (const name of Object.keys(defaults))`）—— defaults
 * is empty for when return {}， up from no （ has will in has 
 * ）。 this 1 （ is wbFilterMemory.save that ），「 up that in 
 * use 」 。 if need old as ， 
 * `shared().wbFilterMemory?.load?.(SOURCE_KEY, {}) ?? {}`。
 */
function loadSavedSource(): Record<string, string> {
  try {
    const raw: unknown = JSON.parse(localStorage.getItem(SOURCE_KEY) || '{}')
    if (!raw || typeof raw !== 'object') return {}
    const out: Record<string, string> = {}
    // / empty 1 ： not this down in 1 not in 
    for (const [key, value] of Object.entries(raw as Record<string, unknown>)) {
      if (typeof value === 'string' && value) out[key] = value
    }
    return out
  } catch {
    return {} // 存坏 / 存储不可用：记忆是锦上添花，不能挡弹窗打开
  }
}

/** （ and ，patch —— and wbFilterMemory.save 1 ） */
function saveSavedSource(patch: Record<string, string>) {
  shared().wbFilterMemory?.save?.(SOURCE_KEY, patch)
}

/* ─── ───────────────────────────────── */

/** ：`options` is open() ，`onClose` by */
function ModelsFetchModal({ options, onClose }: { options: OpenOptions; onClose: () => void }) {
  const providerId = options.providerId
  const custom = Boolean(options.custom)
  const name = options.name || providerId
  const { onDone, onRefreshed } = options

  const [status, setStatus] = React.useState<Status>('idle')
  const [errorText, setErrorText] = React.useState('')
  const [hint, setHint] = React.useState('')
  /**
   * in （ / / ，）： not ， will 「 to no 」unknown；
   * by not （ before is 「get in …」， is 「 in …」）， is not is 1 。
   */
  const [busyKind, setBusyKind] = React.useState<'load' | 'import' | 'row' | null>(null)
  /** when ** 1 ** in （`busyKind === 'row'` when has ）： that 1 by 「get in …」 */
  const [busyRow, setBusyRow] = React.useState('')
  const busy = busyKind !== null
  const failed = status === 'error'
  /**
   * will is ** already ** 1 （ when is false）。** not **，「 no has 」
   * is 1 ： by （get / new get） and total。
   */
  const [fetched, setFetched] = React.useState(false)
  /** ： up id（、、） */
  const [upstream, setUpstream] = React.useState<string[]>([])
  /** ： already has id（，「 already 」） */
  const [managed, setManaged] = React.useState<ReadonlySet<string>>(() => new Set<string>())
  /** ： in id（， when by ） */
  const [picked, setPicked] = React.useState<ReadonlySet<string>>(() => new Set<string>())
  /** ： new （status === 'ready' when ） */
  const [results, setResults] = React.useState<RefreshItem[]>([])
  /** ：（； old is input when ） */
  const [keyword, setKeyword] = React.useState('')
  /** 「」 （ when 1 ； in and update） */
  const [savedSource, setSavedSource] = React.useState<Record<string, string>>(loadSavedSource)

  /** ： （ and scopeProviders ， is will request ） */
  const pending = custom ? [] : pendingProviders()
  /** （get + already has ， buildRows）； no has this */
  const rows = custom ? [] : buildRows()

  /**
   * total（， old paintSummary and ）： is 「 already N / up 
   * M / already has K」， is 「N · success X · failure Y」。
   */
  const summary = custom
    // no when is 0， will as 「 up no has 」—— that and 「 no 」 is 
    ? fetched
      ? t('已选 {picked} 个 · 上游共 {upstream} 个 · 清单已有 {managed} 个',
        { picked: picked.size, upstream: upstream.length, managed: managed.size })
      : t('尚未获取')
    : intlSummary(rows)

  /**
   * ： by （= ） ， already has that up that 1 
   * 。 not after to —— 「get」 after has that 1 ， not 。
   */
  function buildRows(): Row[] {
    const byProvider = new Map(results.map(item => [String(item.provider || ''), item]))
    const built: Row[] = pending.map(meta => {
      const id = String(meta.id || '')
      return { provider: id, label: String(meta.label || id), item: byProvider.get(id) }
    })
    const known = new Set(built.map(row => row.provider))
    // after to 、 not in get （ up not this has ）： in ， 
    for (const item of results) {
      const id = String(item.provider || '')
      if (!known.has(id)) built.push({ provider: id, label: String(item.providerLabel || id), item })
    }
    return built
  }

  /**
   * total（get / new in / failure / ）。 by **** not is by 
   * `results.length`： 1 after is ，success this to that 。
   */
  function intlSummary(list: Row[]): string {
    if (status === 'loading') return t('正在刷新…')
    // failure when not up 1 ： that is old ， and 「getfailure」
    if (status === 'error') return ''
    const items = list.map(row => row.item).filter((item): item is RefreshItem => Boolean(item))
    if (!items.length) {
      // 、（ this ） 1 no ：「 no to 」，「get」
      if (fetched) return t('刷新完成，但没有得到任何结果')
      return list.length ? t('共 {n} 家 · 点「获取模型」开始', { n: list.length }) : t('0 家')
    }
    const done = items.filter(item => item.status === 'refreshed').length
    const failed = items.filter(item => item.status === 'failed').length
    return failed
      ? t('{total} 家 · 成功 {done} · 失败 {failed}', { total: list.length, done, failed })
      : t('{total} 家 · 成功 {done}', { total: list.length, done })
  }

  /**
   * 「」 valid in —— （ old sourcePicker ）：** 
   * **（ up that in use ）→ ** after **（`item.accountId`； use no 
   * when is 「 this use is 」，）→ ** 1 **（default = available， and after default
   * 1 ）。noneoptional and not returnempty。
   */
  function selectSource(pid: string, reported?: unknown): { list: Account[]; selected: string } {
    const list = sourceAccounts(pid)
    if (!list.length) return { list, selected: '' }
    const saved = String(savedSource[pid] || '')
    const reportedId = String(reported || '')
    const has = (id: string) => list.some(account => String(account.id) === id)
    return { list, selected: has(saved) ? saved : has(reportedId) ? reportedId : String(list[0].id) }
  }

  /**
   * new 「 use 」 （request `accounts`）。：** down current**
   * （ use ）→ ** **（ up ）→ not this （ after by default = available）。 by 
   * ，「current」 is up that valid in 。
   *
   * and old 1 （ ）： old this 1 is DOM（`#fm-tbody select[data-provider]`），
   * before 「 in get…」， that not to —— has 。 this by 
   * ： current valid in 1 and up ，request and use up down 1 。
   * and after default is 1 （available）， as not 。
   */
  function buildSourceMap(rows: Array<{ provider: string; reported?: unknown }>) {
    const map: Record<string, string> = {}
    for (const [id, accountId] of Object.entries(savedSource)) if (accountId) map[id] = accountId
    for (const row of rows) {
      const { selected } = selectSource(row.provider, row.reported)
      if (selected) map[row.provider] = selected
    }
    return map
  }

  /**
   * 1 「」current（buildSourceMap by 「 down current」）： **get 
   * in **， 1 after ， its down request（ old by 1 ，
   * after that 1 ）。 in / failure when is 1 、 no has down ， when this by 
   * selectSource to ， and will 。
   */
  const sourceRows: Array<{ provider: string; reported?: unknown }> =
    rows.map(row => ({ provider: row.provider, reported: row.item?.accountId }))

  /** in —— down 、「 new get」 as （ loadSavedSource） */
  function rememberSource(pid: string, accountId: string) {
    setSavedSource(prev => ({ ...prev, [pid]: accountId }))
    saveSavedSource({ [pid]: accountId })
  }

  /* ─── and ─────────────────────────── */

  async function load() {
    if (busy) return
    setBusyKind('load')
    setHint('')
    setErrorText('')
    setStatus('loading')
    try {
      if (custom) {
        const data = await shared().wbProviders?.customRequest?.(
          'POST', '/api/custom-providers/fetch-models', { providerId })
        // （empty + not ）、、 —— and old 1 
        const seen = new Set<string>()
        const ids = (Array.isArray(data?.models) ? data.models : [])
          .map(value => String(value ?? '').trim())
          .filter(id => {
            const key = norm(id)
            if (!id || seen.has(key)) return false
            seen.add(key)
            return true
          })
        const managedNow = managedIds(providerId)
        setUpstream(ids)
        setManaged(managedNow)
        // default「」 that —— and OmniProxy 1 default： use is new 
        setPicked(new Set(ids.filter(id => !managedNow.has(norm(id)))))
        setFetched(true)
        setStatus('ready')
        return
      }
      // 「」 down and new request up （ scopeProviders）
      const refresh = shared().workbuddyDesktop?.refreshModels
      if (!refresh) throw new Error(t('模型目录刷新不可用，请重新打开面板'))
      const providers = pending.map(item => item.id).filter((id): id is string => Boolean(id))
      const result = await refresh({ accounts: buildSourceMap(sourceRows), providers })
      if (!Array.isArray(result?.results) || result.results.some(item => !item || typeof item.provider !== 'string'
        || !['refreshed', 'failed', 'skipped'].includes(item.status || ''))
        || providers.some(provider => result.results!.filter(item => item.provider === provider).length !== 1)) {
        throw new Error(t('网关未返回逐家模型目录刷新结果'))
      }
      setResults(result.results)
      setFetched(true)
      setStatus('ready')
      // to new directory when use —— new after that directory， manage 
      // （ / 「」） is 1 （/api/models/manage）， not in old 。
      // failure / when not ： 1 no ， will 1 。 not in 。
      if (Number(result?.refreshed) > 0) onRefreshed?.()
    } catch (error) {
      setErrorText(errorMessage(error))
      setStatus('error')
    } finally {
      setBusyKind(null)
    }
  }

  /**
   * ****（ that ）： this 1 。request `providers` `[row.provider]` —— after 
   * up ， not 、 not （ api/models.rs `parse_providers`）， this is 1 
   * 「 for 1 」 new ， not is 。
   *
   * after ** and **（mergeResults） not is ： its （ no 「get」）
   * ， up has this 1 。
   *
   * failure** not **「getfailure」： its is valid ， 1 failure not this 1 ——
   * + toast Reason， and current as failure； its already has 。
   * and 1 ：`refreshed > 0` use （failure / 1 no ）。
   */
  async function loadRow(row: Row) {
    if (busy || !row.provider) return
    setBusyKind('row')
    setBusyRow(row.provider)
    setHint('')
    try {
      const refresh = shared().workbuddyDesktop?.refreshModels
      if (!refresh) throw new Error(t('模型目录刷新不可用，请重新打开面板'))
      const result = await refresh({
        accounts: buildSourceMap([{ provider: row.provider, reported: row.item?.accountId }]),
        providers: [row.provider],
      })
      const items = Array.isArray(result?.results) ? result.results.filter(item => item?.provider === row.provider) : []
      if (items.length !== 1 || !['refreshed', 'failed', 'skipped'].includes(items[0].status || '')) {
        throw new Error(t('网关未返回逐家模型目录刷新结果'))
      }
      const item = items[0]
      setResults(prev => mergeResults(prev, [item]))
      setFetched(true)
      setStatus('ready')
      if (Number(result?.refreshed) > 0) onRefreshed?.()
    } catch (error) {
      const message = errorMessage(error)
      setResults(prev => mergeResults(prev, [{
        provider: row.provider, providerLabel: row.label, status: 'failed', message,
      }]))
      setFetched(true)
      setStatus('ready')
      setHint(message)
      toast(t('获取失败：{message}', { message }), 'err')
    } finally {
      setBusyKind(null)
      setBusyRow('')
    }
  }

  /** and has ：、 new —— that 1 （ by 
   * provider id， and after 1 ）。 */
  function mergeResults(previous: RefreshItem[], items: RefreshItem[]): RefreshItem[] {
    const byProvider = new Map(items.map(item => [String(item.provider || ''), item]))
    const merged = previous.map(item => byProvider.get(String(item.provider || '')) ?? item)
    const known = new Set(merged.map(item => String(item.provider || '')))
    for (const item of items) {
      if (!known.has(String(item.provider || ''))) merged.push(item)
    }
    return merged
  }

  /**
   * ：（ already in after will ， before by 1 ）。
   * success —— and old （`close()` 、`done?.()` ）， is 
   * 「 after 」 ， not in 。
   */
  async function importPicked() {
    if (!custom || busy || !picked.size) return
    const values = [...picked]
    setBusyKind('import')
    setHint('')
    try {
      const added = Number(await shared().wbModelsCustom?.addModels?.(providerId, values)) || 0
      toast(t('✅ 已导入 {added} 个模型（{skipped} 个已存在，跳过）',
        { added, skipped: values.length - added }))
      setBusyKind(null)
      onClose()
      onDone?.()
    } catch (error) {
      const message = errorMessage(error)
      toast(t('导入失败：{message}', { message }), 'err')
      setHint(message)
      setBusyKind(null)
    }
  }

  /** that （「」/ 「」）： and Esc 1 ， in when not */
  function handleCloseClick() {
    if (!busy) onClose()
  }

  /** ： up all「 not in 」 id up （ in already up ， not empty） */
  function selectAllUnadded() {
    const next = new Set(picked)
    for (const id of upstream) if (!managed.has(norm(id))) next.add(id)
    setPicked(next)
  }

  /** 「」： this down （ use this directory）。noneoptional and not 
   * empty， title Reason（「 this is 」/「 this no has available」）。 */
  function sourceCell(pid: string, reported?: unknown) {
    const { list, selected } = selectSource(pid, reported)
    if (!list.length) {
      const why = ACCOUNTLESS_REFRESH.has(pid)
        ? t('这家的模型清单是全局的，不跟账号走')
        : t('这家还没有可用账号，刷新会使用默认登录态')
      return <span className='text-[11.5px] text-muted-foreground' title={why}>—</span>
    }
    const current = list.find(account => String(account.id) === selected)
    return (
      <Select value={selected} onValueChange={next => rememberSource(pid, String(next))}>
        <SelectTrigger className='w-full' title={t('用哪个账号去拉这家的模型清单')}
          aria-label={t('模型来源')}>
          {/* to ： not value （ is id， this 1 need is ） */}
          <SelectValue>{current ? accountLabel(current) : '—'}</SelectValue>
        </SelectTrigger>
        <SelectContent>
          {list.map(account => (
            <SelectItem key={String(account.id)} value={String(account.id)}>{accountLabel(account)}</SelectItem>
          ))}
        </SelectContent>
      </Select>
    )
  }

  /** / 1 ： and use this 1 ， not will */
  function togglePicked(id: string, next: boolean) {
    setPicked(prev => {
      const out = new Set(prev)
      if (next) out.add(id)
      else out.delete(id)
      return out
    })
  }

  /* ─── ───────────────────────────────── */

  return (
    <Dialog
      open
      onOpenChange={(next, eventDetails) => {
        // request（Esc / / up ✕） to this 。 in when ：request already ， will 
        // 「 to no 」unknown（ old close() in busy when ）。must
        // eventDetails.cancel() —— 「 not update open prop」 not Base UI store。
        if (next) return
        if (busy) {
          eventDetails.cancel()
          return
        }
        onClose()
      }}
    >
      {/* ： is （ old .modal-wide ≈ 920px）， has （ old 
          .modal-narrow ≈ 552px， and ）； 24px and for 。
           in old up 10%（920 → 1012 / 552 → 607）： that in 920px when 
          「」 that ， 1 can 。 */}
      <DialogContent className={failed ? 'w-[min(640px,calc(100vw-48px))]'
        : custom ? 'w-[min(607px,calc(100vw-48px))]' : 'w-[min(1012px,calc(100vw-48px))]'}>
        <DialogHeader>
          <DialogTitle>{t('获取模型 — {name}', { name })}</DialogTitle>
        </DialogHeader>
        <DialogBody>
          {failed ? <ModelsFetchError message={errorText} /> : (
            <>
              <div className='flex flex-wrap items-center gap-2.5'>
                <span className='text-[11.5px] text-muted-foreground'>{summary}</span>
                {custom && (
                  <InputGroup className='w-[240px] flex-none'>
                    <InputGroupInput type='search' placeholder={t('搜索模型 ID…')} autoComplete='off'
                      value={keyword} onChange={event => setKeyword(event.currentTarget.value)} />
                    <InputGroupAddon aria-hidden='true'>⌕</InputGroupAddon>
                  </InputGroup>
                )}
                <div className='mr-auto' />
                {custom && (
                  <>
                    <Button variant='ghost' size='sm' onClick={selectAllUnadded}>{t('全选未添加')}</Button>
                    <Button variant='ghost' size='sm' onClick={() => setPicked(new Set())}>{t('清空')}</Button>
                  </>
                )}
                <Button variant='outline' size='sm' title={t('刷新全部家（逐家打上游）')} disabled={busy}
                  onClick={() => { void load() }}>
                  {busyKind === 'load' ? t('获取中…') : fetched ? t('重新获取') : t('获取模型')}
                </Button>
              </div>
              <ModelsFetchTable custom={custom} status={status} rows={rows}
                upstream={upstream} managed={managed} picked={picked} keyword={keyword}
                busy={busy} busyRow={busyRow} onPick={togglePicked}
                onFetchRow={row => { void loadRow(row) }} sourceCell={sourceCell} formatTime={formatTime}
                refreshedAt={pid => Number(shared().wbModelsPanel?.providerRefreshedAt?.(pid)) || 0} />
            </>
          )}
        </DialogBody>
        <DialogFooter>
          {!failed && hint && (
            <span className='min-w-0 text-[11.5px] text-muted-foreground [overflow-wrap:anywhere]'>{hint}</span>
          )}
          <div className='mr-auto' />
          <Button variant='outline' onClick={handleCloseClick} disabled={busy}>
            {failed ? t('关闭') : custom ? t('取消') : t('完成')}
          </Button>
          {failed ? (
            <Button variant='default' disabled={busy} onClick={() => { void load() }}>{t('重试')}</Button>
          ) : custom && (
            <Button variant='default' disabled={!picked.size || busy} onClick={() => { void importPicked() }}>
              {busyKind === 'import' ? t('导入中…')
                : picked.size ? t('导入选中的 {n} 个模型', { n: picked.size }) : t('导入')}
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

/* ─── ： and old window.wbModelsFetchModal 1 ─── */

let root: ReturnType<typeof createRoot> | null = null
let host: HTMLElement | null = null

function unmountModal() {
  if (root) { root.unmount(); root = null }
  if (host) { host.remove(); host = null }
}

/**
 * ： by need （ and conc-dialog / request-clear-modal 1 ）。`providerId` is current
 * in that ；`custom` by use 。**， not ** —— need use 「get」。
 */
function openModal(options: OpenOptions) {
  if (!options || (options.custom && !options.providerId)) return
  unmountModal() // 重复打开先拆掉上一份（旧实现同样是 close() 打头）
  host = document.createElement('div')
  document.body.append(host)
  root = createRoot(host)
  root.render(<ModelsFetchModal options={options} onClose={unmountModal} />)
}

declare global {
  interface Window {
    /** 「get」（ ui/models-fetch-modal.js， and 1 ） */
    wbModelsFetchModal?: { open(options: OpenOptions): void }
  }
}

window.wbModelsFetchModal = { open: openModal }
