/**
 * Agent2API · ****（ store / / / for ）。
 *
 * from models-page.tsx ： that file「 + + 」 already 
 * file， this 1 —— ** has no has JSX**。 is to （ import ， not 
 * ）， not will 。
 *
 * not is ：file is .ts， not will src/index.tsx `islands/*.tsx` glob ； 
 * has models-page.tsx 1 。 （、Why need 、 
 * ） in models-page.tsx file。
 *
 * ── + useSyncExternalStore（ port-panel / update-panel）──
 * for （render / load / selectProvider / refreshAll） from React use ， and must and 
 * use 1 ； useState not to this 1 。 is 1 ， 1 
 * patch()（ new for ）， use useSyncExternalStore 。
 * ** in 1 in patch ** not is ： will and React 。
 *
 * that ， not app.js state（ that is /api/session ， will ）。
 */

import { activeModelBindings } from '../../../ui-shared/models-bindings'
import { buildIndex as buildReasoningIndex } from './models-reasoning'
import * as customSource from './models-custom-source'
import { ALL_CUSTOM } from './models-custom-source'
import type { CustomProviderRecord, ManageMapping, ManageModel, ManageView } from './models-custom-source'
import type { SegmentedControlOption } from '@ui'
import { t } from '../i18n'

/* ─── ───────────────────────────────────── */

export type Align = 'left' | 'center' | 'right'
type ColumnDecl = { key: string; label?: string; align?: Align }
export type ColumnView = ColumnDecl & { align?: Align }
type ColSettingsHandle = { apply<C extends { key: string }>(columns: C[]): (C & { align: Align })[] }
type ColSettingsSpec = {
  id: string
  label?: string
  columns: ColumnDecl[]
  mount?: () => Element | null
  onChange?: () => void
  buttonPlacement?: 'first' | 'last'
}

type ProviderOption = { id: string; label: string }
type UpstreamOption = { id: string; label: string; off: boolean }
/** up down ：「＋」 to `{target, provider}`， chip up etc up `alias` */
export type MappingContext = { alias?: string; target: string; provider: string }
/** up down ： in when this is （ openCustomModel） */
export type CustomModelContext = { provider: string; locked: boolean }
/**
 * can up down ： `(provider, id)` —— when from current
 * that 1 （、current、 from up ）。 in 
 * new （directory new / ） when ， in update、 no by 
 * （ not to up down ）。
 */
export type CapabilityContext = { provider: string; id: string }

type Filters = { provider: string; state: string; search: string; sort: string; visibility: string }

/** （default alias == target by ， not in mappings 1 ） */
export type Binding = {
  alias: string
  target: string
  provider: string
  enabled: boolean
  isDefault?: boolean
}

/**
 * window up by its / its 。
 *
 * use 「 + 」 not is declare global Window up ：workbuddyDesktop /
 * wbApp / wbConfirm / wbProviders this is use ， if declare 1 ， and will 
 * not 1 TS2717 —— and when 。file declare 
 * wbModelsPanel（file）。
 */
export type SharedWindow = {
  workbuddyDesktop?: {
    getModelManage(): Promise<ManageView | null | undefined>
    addModelMapping(
      alias: string, target: string, provider: string, reasoning?: string, enabled?: boolean, create?: boolean,
    ): Promise<ManageView | null | undefined>
    removeModelMapping(alias: string, target: string, provider: string): Promise<ManageView | null | undefined>
    addCustomModel(provider: string, id: string): Promise<ManageView | null | undefined>
    removeCustomModel(provider: string, id: string): Promise<ManageView | null | undefined>
    /** can （； wbModelsCustom.setCapabilities） */
    setModelCapabilities(
      provider: string, id: string, patch: Record<string, number | boolean | null>,
    ): Promise<ManageView | null | undefined>
  }
  wbApp?: {
    toast?: (message: string, kind?: 'err' | 'ok') => void
    /** HTML （app.js ， to use ） */
    esc?: (value: unknown) => string
    formatTime?: (value: unknown) => string
    /** current： in use when 1 */
    readonly currentPage?: string
  }
  /** ：app.js ， old formatTime(...) */
  formatTime?: (value: unknown) => string
  wbConfirm?: {
    ask?: (options: { title?: string; html?: string; okText?: string; okClass?: string }) => Promise<boolean>
  }
  wbColSettings?: {
    register(spec: ColSettingsSpec): ColSettingsHandle
    syncStaticHead(id: string, table: Element | null | undefined, viewHidden?: ReadonlySet<string> | null): void
  }
  wbFilterMemory?: {
    load<T extends Record<string, string>>(key: string, defaults: T): T
    save(key: string, patch: Record<string, string>): void
  }
  wbProviders?: {
    all?: () => Array<{ id?: string; label?: string }>
    labelOf?: (id: string) => string
    refreshCustom?: () => Promise<unknown>
  }
  /** directory（preset-providers.js）： not ， by match use */
  wbPresetProviders?: {
    list?: Array<{ key?: string; name?: string; icon?: string }>
    presetOf?: (key: string) => { key?: string; name?: string; icon?: string } | null
    iconOf?: (key: string) => string
  }
  wbCustomProvidersUi?: { remove?: (providerId: string) => Promise<boolean> }
  wbAccountAddForms?: { openNewCustomForm?: () => void }
  wbModelsFetchModal?: {
    open(options: {
      providerId: string; custom?: boolean; name?: string
      onDone?: () => void; onRefreshed?: () => void
    }): void
  }
}

export function shared(): SharedWindow {
  return window as unknown as SharedWindow
}

export function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}

/** toast 1 （ wbApp， not in ） */
export function toast(message: string, kind?: 'err' | 'ok'): void {
  shared().wbApp?.toast?.(message, kind)
}

/**
 * ：file use 1 wbApp（esc is app.js ）， has no 
 * when 1 —— can 1 ， not can （wbConfirm.ask is HTML ）。
 */
export function esc(value: unknown): string {
  const fn = shared().wbApp?.esc
  if (fn) return fn(value)
  return String(value ?? '').replace(/[&<>"']/g, char => (
    { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[char] || char
  ))
}

/** when → when （0 and returnempty， use that ） */
export function formatTime(value: unknown): string {
  const fn = shared().formatTime ?? shared().wbApp?.formatTime
  return String(fn?.(value) ?? '')
}

/** 1 ：empty + not （ and after 1 ） */
const norm = (value: unknown): string => String(value ?? '').trim().toLowerCase()
export const same = (left: unknown, right: unknown): boolean => norm(left) === norm(right)

/* ─── ───────────────────────────────────── */

/** 「」default this ， its 1 「 its N 」 */
export const GROUP_LIMIT = 8

/** （ wbFilterMemory：provider / state / search ） */
const FILTERS_KEY = 'workbuddy-desktop-models-filters'

const MODEL_STATES: readonly string[] = ['all', 'enabled', 'disabled', 'mapped']
/** up （ and MODEL_STATES 1 1 for ，； has t()，value is / use ） */
const MODEL_STATE_LABEL: Record<string, string> = {
  all: t('全部'), enabled: t('已启用'), disabled: t('已禁用'), mapped: t('有映射'),
}
/** to ：SegmentedControl to new need new ， can this */
export const MODEL_STATE_OPTIONS: readonly SegmentedControlOption<string>[] = MODEL_STATES.map(value => ({
  value, label: MODEL_STATE_LABEL[value],
}))

/**
 * （issue #161： by ， ）。
 *
 * `default` is 「 not 」—— by after （ use to ，
 * models-page ）， and this before ** 1 **。
 *
 * up `credits` （ `x0.7 credits`）， not 1 in after ：
 * 「 not to 」 and 「 is 0」 is ， by 0 and will unknown to before 。
 */
const MODEL_SORTS: readonly string[] = ['default', 'rate-asc', 'rate-desc']

/* ─── can ───────────────────────────────── */

/**
 * can 。
 *
 * ── Why「」 1 ────────────────────────────────────
 * and up （`MODEL_STATES` = `all / enabled / disabled / mapped`） 1 
 * ：「」 is ****， in 。 from 1 ，
 * use 「」 when down ， in will 1 。
 *
 * Note「 1 」 not etc 「default in 」：default is `visible`（
 * `DEFAULT_FILTERS`）， by down 。
 *
 * ── Whydefault is `visible` not is `all` ─────────────────────
 * up directory is ****：client not （ Antigravity 
 * 2.5 / 2.0 、Qoder `enable:false` ）。 this in **default already 
 * default**（ `model_rules::seed_hidden_disabled`）， is 
 * unavailable； 1 ， use can use down 。
 * default can ， need （ to 「」 will ）。
 *
 * ── Why has `undeclared` this 1 ────────────────────────────
 * has （Antigravity / Qoder / Raccoon / CodeArts） has can ， its 
 * no has `visibility` 。「 this no this 」 and 「 this not can 」
 * must： before not unavailable， `hidden` will use as that 
 * 。`undeclared` in up `visible`（ not ）—— and 「 no has 
 * not 」 1 。
 */
const MODEL_VISIBILITIES: readonly string[] = ['all', 'visible', 'hidden', 'undeclared']
const MODEL_VISIBILITY_LABEL: Record<string, string> = {
  all: t('全部'), visible: t('可见'), hidden: t('不展示'), undeclared: t('未声明'),
}
export const MODEL_VISIBILITY_OPTIONS: readonly SegmentedControlOption<string>[] = MODEL_VISIBILITIES.map(value => ({
  value, label: MODEL_VISIBILITY_LABEL[value],
}))

/** 1 1 can ：`'shown'` → `visible`，`'hidden'` → `hidden`， its → `undeclared` */
export function visibilityOf(model: ManageModel): 'visible' | 'hidden' | 'undeclared' {
  const raw = typeof model.visibility === 'string' ? model.visibility.trim().toLowerCase() : ''
  if (raw === 'hidden') return 'hidden'
  if (raw === 'shown' || raw === 'visible') return 'visible'
  return 'undeclared'
}

/**
 * key use has `data-col`（index.html `<col class="c-xxx" data-col="xxx">`、
 * th、table-columns.js ， has 1 ）。default。
 */
const COLUMNS: ColumnDecl[] = [
  // 「」： （ —— set need has ，
  // that is 「」， no has ）
  { key: 'check', label: t('选择'), align: 'center' },
  { key: 'model', label: t('上游模型') },
  { key: 'rate', label: t('倍率') },
  // 「 can 」「」： is 1 （ this / up 
  // not ）， and 。 no `—`（ models-page 
  // `visibilityCell`）—— that is 「 this no this 」， not is 「 can 」。
  { key: 'visibility', label: t('可见性') },
  { key: 'source', label: t('来源') },
  // can （、 and model-capability for ）： and in 
  // 「 up down / 」 1 ， and 1 —— will 
  // 「」 to unavailable（）
  { key: 'budget', label: t('上下文 / 输出') },
  { key: 'caps', label: t('能力') },
  { key: 'alias', label: t('模型映射') },
  { key: 'act', label: t('操作'), align: 'right' },
]

/**
 * no has 「」「」 this （ is ）： in when this 
 * ** by **，set config not —— 。（syncStaticHead 
 * parameter） and （visibleColumns） 1 ， not will 1 。
 */
const CUSTOM_HIDDEN_COLUMNS: ReadonlySet<string> = new Set(['rate', 'source', 'visibility'])

const EMPTY_VIEW: ManageView = { models: [], mappings: [] }

/**
 * default。
 *
 * `visibility: 'visible'` is ** has 「」** 1 1 ： up directory
 * client not （ already default default）， in 
 * will use can use that 。 to 「 / 」 can （， will ）。
 */
const DEFAULT_FILTERS: Filters = { provider: 'all', state: 'all', search: '', sort: 'default', visibility: 'visible' }

/* ─── （ store）──────────────────── */

export type PanelState = {
  /** in ：'all' / id / custom- id */
  provider: string
  stateFilter: string
  /** can （ / can / not / ）， MODEL_VISIBILITIES */
  visibilityFilter: string
  search: string
  /** （default / / ）， MODEL_SORTS */
  sort: string
  /** manage （null = no to ） */
  data: ManageView | null
  /** already */
  expanded: ReadonlySet<string>
  /** in （ 1 ） */
  pending: ReadonlySet<string>
  /** up down ；null = （） */
  mapping: MappingContext | null
  /** up down ；null = */
  customModel: CustomModelContext | null
  /** can up down ；null = */
  capability: CapabilityContext | null
}

/**
 * up will 。 not can ：wbFilterMemory by filter-memory.js in window
 * up ， that file in islands/ui.js ** after **（index.html 1455 ）， is already 
 * has no has in —— not to returndefault and ， by 
 * restoreSavedFilters in after 1 ， 「」 。
 */
function readSavedFilters(): Filters {
  const memory = shared().wbFilterMemory
  if (!memory) return { ...DEFAULT_FILTERS }
  const saved = memory.load(FILTERS_KEY, { ...DEFAULT_FILTERS })
  return {
    provider: String(saved.provider || 'all'),
    // 「」（ select 1 not in is this ）
    state: MODEL_STATES.includes(saved.state) ? saved.state : 'all',
    search: String(saved.search || ''),
    // 「default」（ and 1 ）
    sort: MODEL_SORTS.includes(saved.sort) ? saved.sort : 'default',
    // can 「 can 」（ and default 1 ， not is 「」）
    visibility: MODEL_VISIBILITIES.includes(saved.visibility) ? saved.visibility : 'visible',
  }
}

const initialFilters = readSavedFilters()
/** up will is already （ 1 ） */
let filtersRestored = Boolean(shared().wbFilterMemory)
/** use is already ： not ， use */
let filtersTouched = false

let snapshot: PanelState = {
  provider: initialFilters.provider,
  stateFilter: initialFilters.state,
  visibilityFilter: initialFilters.visibility,
  search: initialFilters.search,
  sort: initialFilters.sort,
  data: null,
  expanded: new Set<string>(),
  pending: new Set<string>(),
  mapping: null,
  customModel: null,
  capability: null,
}

/** （current has that 1 root） */
const subscribers = new Set<() => void>()

export function getSnapshot(): PanelState {
  return snapshot
}

export function subscribe(listener: () => void): () => void {
  subscribers.add(listener)
  return () => {
    subscribers.delete(listener)
  }
}

/**
 * and ：useSyncExternalStore use ，must new for 。
 *
 * in 1 （ not 「」） in this ， by resolveProvider。
 */
function patch(next: Partial<PanelState>): void {
  const merged = { ...snapshot, ...next }
  const provider = resolveProvider(merged)
  const switched = resolveProvider(snapshot) !== provider
  snapshot = provider === merged.provider ? merged : { ...merged, provider }
  // 「 → etc 」：chip up that etc ， 1 ，
  // otherwise etc 、 after chip up is old that （ old is render 1 ，
  // this —— allpath， and no has 「 render 」 before ）
  rebuildReasoningIndex()
  // （ ↔ ）： by / 。
  // in this not is in selectProvider —— in can can 1 （ that ），
  // that will ， old in selectProvider （ 1 that 1 ）。
  if (switched) syncHead()
  for (const listener of subscribers) listener()
}

/** （ ，patch —— and wbFilterMemory.save 1 ） */
function saveFilters(next: Record<string, string>): void {
  shared().wbFilterMemory?.save(FILTERS_KEY, next)
}

/** filter-memory.js in after ： no to 1 （ use already not ） */
export function restoreSavedFilters(): void {
  if (filtersRestored || filtersTouched || !shared().wbFilterMemory) return
  filtersRestored = true
  const saved = readSavedFilters()
  if (saved.provider !== snapshot.provider || saved.state !== snapshot.stateFilter
    || saved.search !== snapshot.search || saved.sort !== snapshot.sort
    || saved.visibility !== snapshot.visibilityFilter) {
    patch({
      provider: saved.provider, stateFilter: saved.state, search: saved.search,
      sort: saved.sort, visibilityFilter: saved.visibility,
    })
  }
}

/* ─── ───────────────────────────────── */

/**
 * in ： that can can already （ or this no to ）， not 
 * 「」—— otherwise will is 1 empty ， use not to Reason。
 *
 * ** to **： 1 can can is 「 no 」（providers.js in after 
 * 、 that /api/models/manage ）， is use in 。
 */
export function resolveProvider(state: PanelState): string {
  const id = state.provider
  if (id === 'all' || id === ALL_CUSTOM) return id
  if (customSource.list().some(provider => provider.id === id)) return id
  if (state.data?.providers?.some(provider => provider.provider === id)) return id
  if (Array.isArray(state.data?.models) && state.data.models.some(model => (model.provider || '') === id)) return id
  if (!state.data || !directoryReady()) return id
  return 'all'
}

/** 1 after in （ and all， / / is 1 ） */
export function currentProvider(): string {
  return resolveProvider(snapshot)
}

/**
 * directory（providers.js） is already 。 in ** after **， that 1 can can no has ——
 * that when not can 「 not to this 」「 this already delete」。
 */
export function directoryReady(): boolean {
  return Boolean(shared().wbProviders)
}

/** current in is not is 「 · 」 */
export function isAllCustomView(): boolean {
  return currentProvider() === ALL_CUSTOM
}

/** current in is not is （ or 「 · 」 —— 1 ） */
function isCustomView(): boolean {
  return isAllCustomView() || customSource.isCustom(currentProvider())
}

/**
 * ：directory is 、， 1 will 
 * （ + + ）， by 「 this use 」 1 —— directory 1 new （ new for ）
 * ， to is new 。
 */
let customViewCache: { id: string; record: CustomProviderRecord | null; view: ManageView } | null = null

function customViewOf(id: string): ManageView {
  const record = customSource.record(id)
  if (!customViewCache || customViewCache.id !== id || customViewCache.record !== record) {
    customViewCache = { id, record, view: customSource.buildView(id) || EMPTY_VIEW }
  }
  return customViewCache.view
}

/**
 * 「 · 」 ： 1 （** not **
 * —— that 「」 is ， this is ）， 1 
 * （ + + + ）。 is **directory use **：providers.js
 * fetchCustomProviders new for ，listSource to use 1 ； use 
 * etc = directory no = 1 ， use 。emptydirectory to EMPTY_VIEW（ and old 
 * no has when empty 1 ，empty by by ）。
 */
let aggregateCache: { directory: readonly unknown[]; view: ManageView } | null = null

function aggregateView(): ManageView {
  const directory = customSource.listSource()
  if (aggregateCache && aggregateCache.directory === directory) {
    return aggregateCache.view
  }
  const models: ManageModel[] = []
  const mappings: ManageMapping[] = []
  for (const provider of customSource.list()) {
    const view = customViewOf(String(provider.id || ''))
    models.push(...(view.models || []))
    mappings.push(...(view.mappings || []))
  }
  aggregateCache = { directory, view: models.length || mappings.length ? { models, mappings } : EMPTY_VIEW }
  return aggregateCache.view
}

/** current （ in this ， down / / in ） */
export function viewData(): ManageView {
  const provider = currentProvider()
  if (provider === ALL_CUSTOM) return aggregateView()
  if (customSource.isCustom(provider)) return customViewOf(provider)
  return snapshot.data || EMPTY_VIEW
}

export function models(): ManageModel[] {
  const view = viewData()
  return Array.isArray(view.models) ? view.models : []
}

export function mappings(): ManageMapping[] {
  const view = viewData()
  return Array.isArray(view.mappings) ? view.mappings : []
}

/**
 * 「 → etc 」。must** before 1 **：，otherwise
 * use etc 、，chip up is old that 。 to 1 returnempty —— in 1 
 * render before use （ up not will ， is ） not will ， is 「」。
 */
let reasoningOf = (_alias: unknown, _target: unknown, _provider: unknown): string => ''

function rebuildReasoningIndex(): void {
  reasoningOf = buildReasoningIndex(mappings())
}

/** need emptydirectory； old server no has need when from statistics。 */
export function builtinRailCounts(): Map<string, { label: string; n: number }> {
  const counts = new Map<string, { label: string; n: number }>()
  for (const provider of snapshot.data?.providers || []) {
    if (!provider.provider) continue
    counts.set(provider.provider, { label: provider.providerLabel || provider.provider, n: 0 })
  }
  const rows = Array.isArray(snapshot.data?.models) ? snapshot.data.models : []
  for (const model of rows) {
    const key = model.provider || ''
    if (!key) continue
    const entry = counts.get(key) || { label: model.providerLabel || key, n: 0 }
    entry.n++
    counts.set(key, entry)
  }
  return counts
}

/**
 * in （ id → {label, n}）。
 *
 * 「 · 」 total and 。 buildView（
 * ）；directory when is empty Map， as and old 1 。
 */
export function customRailItems(): Map<string, { label: string; n: number }> {
  const counts = new Map<string, { label: string; n: number }>()
  for (const provider of customSource.list()) {
    const id = String(provider.id || '')
    if (!id) continue
    const view = customViewOf(id)
    counts.set(id, {
      label: String(provider.name || id),
      n: (view.models || []).length,
    })
  }
  return counts
}

/** id， etc getdirectory ；「get」 new 。 */
function builtinProviders(): string[] {
  return [...builtinRailCounts().keys()]
}
/**
 * when （ms；0 = unknown / from success）。
 * to 「get」 「update」 use （ that when not ， 1 can this 
 * has old ）。 manage `refreshedAt`， and 「」 。
 */
function providerRefreshedAt(providerId: string): number {
  const summary = snapshot.data?.providers?.find(provider => provider.provider === providerId)
  if (summary) return Number(summary.refreshedAt) || 0
  const rows = Array.isArray(snapshot.data?.models) ? snapshot.data.models : []
  const hit = rows.find(model => model.provider === providerId)
  return Number(hit?.refreshedAt) || 0
}

/** down （id + ； by ） */
export function providerOptions(): ProviderOption[] {
  const seen = new Map<string, string>()
  for (const model of models()) {
    const key = model.provider || ''
    // ： after to providerLabel is （「WorkBuddy 」 this ），
    // not in will in 
    if (key && !seen.has(key)) seen.set(key, t(model.providerLabel || key))
  }
  return [...seen].map(([id, label]) => ({ id, label }))
}

/**
 * 1 current （ 「 up 」 down ）。
 *
 * already use ： is 「 → 」 then ， and —— use can can 、
 * after that 。 will 「Why not in down 」 1 not 。
 * ：enabled in before （ and 1 ）， after 。
 */
export function upstreamOptions(providerId: string): UpstreamOption[] {
  if (!providerId) return []
  return models()
    .filter(model => (model.provider || '') === providerId)
    .sort((a, b) => Number(a.enabled === false) - Number(b.enabled === false))
    .map(model => ({
      id: model.id,
      // and id not in ，「GLM-5.3（GLM-5.3）」 this 
      label: model.name && model.name !== model.id ? t('{id}（{name}）', { id: model.id, name: model.name }) : model.id,
      off: model.enabled === false,
    }))
}

/**
 * 。
 *
 * and `providerOptions()` ** not **： that 「 」（ as must
 * to 1 up ）， this need ** can ** —— use can can to no ，
 * etc up 。`wbProviders.all()` is /api/session （ count=0 ），
 * up （ not in need ）。path：wbProviders no when to 
 * （ ， not will empty not ）。
 */
export function customProviderOptions(): ProviderOption[] {
  const options: ProviderOption[] = []
  const builtin = shared().wbProviders?.all?.()
  if (Array.isArray(builtin)) {
    for (const item of builtin) {
      const id = String(item?.id || '')
      if (id) options.push({ id, label: t(String(item?.label || id)) })
    }
  }
  for (const provider of customSource.list()) {
    const id = String(provider.id || '')
    if (id) options.push({ id, label: String(provider.name || id) })
  }
  return options.length ? options : providerOptions()
}

/* ─── set ─────────────────────────────────── */

let colSettings: ColSettingsHandle | null = null
/** （ need use ； by <table ref> in when ） */
let tableEl: HTMLTableElement | null = null

/** this current can （config；set when ） */
export function visibleColumns(): ColumnView[] {
  const columns: ColumnView[] = colSettings ? colSettings.apply(COLUMNS) : COLUMNS
  if (!isCustomView()) return columns
  return columns.filter(column => !CUSTOM_HIDDEN_COLUMNS.has(column.key))
}

/** （ / / for ）， and that 1 by current can for 1 */
export function syncHead(): void {
  shared().wbColSettings?.syncStaticHead('models', tableEl, isCustomView() ? CUSTOM_HIDDEN_COLUMNS : null)
}

/**
 * this ：config， and 「set」 by （ in that by ** after **：
 * this 1 has —— / new is ，set is 「 this 」 ）。
 *
 * must in after ： by need React `.head-actions` ， and `syncStaticHead` need 
 * already in （ by id config， no return）。
 */
export function registerColumnSettings(): void {
  if (colSettings) return
  const handle = shared().wbColSettings?.register({
    id: 'models',
    label: t('模型管理表'),
    columns: COLUMNS,
    mount: () => document.querySelector('.page[data-page="gateway"] .panel-head .head-actions'),
    buttonPlacement: 'last',
    // + by new （ 1 config， not will 1 ）
    onChange: () => { syncHead(); render() },
  })
  colSettings = handle || null
  // before that 1 is 「」： use / need in to up 
  if (colSettings) render()
}

/* ─── and ─────────────────────────────── */

/** in ：force when use that response */
let loadSeq = 0
let loading = false

/**
 * manage and 。
 *
 * `force` 「 new 」 this **must** use use ：default down loading will use 
 * and ， when use in （ / ）， this new etc no new 
 * —— force and ， and use ** has after 1 **。
 *
 * failure need ： and is directory， not this this failure 1 
 * （ that up 1 success）。
 */
export async function load({ force = false }: { force?: boolean } = {}): Promise<void> {
  if (loading && !force) return
  const seq = ++loadSeq
  loading = true
  // 1 （ that 1 filter-memory.js can can no ， restoreSavedFilters）
  restoreSavedFilters()
  try {
    const next = await shared().workbuddyDesktop?.getModelManage()
    // if has update 1 ，
    if (seq !== loadSeq) return
    patch({ data: next && Array.isArray(next.models) ? next : null })
  } catch (error) {
    if (seq !== loadSeq) return
    render()
    toast(t('读取模型清单失败：{message}', { message: errorMessage(error) }), 'err')
  } finally {
    if (seq === loadSeq) loading = false
  }
}

/**
 * to when new （app.js showPage ），：
 * 1. **directory**（providers.js ）： that 、 and 
 * 「 this in not in 」。 is ， can can in / / delete， this 1 
 * 、 not update；
 * 2. ** manage **： up directory can can 「get」 new 。
 * not can ：directory is ，， and is 1 。
 * directory new failure not —— by has ，total。
 */
export async function refreshAll(): Promise<void> {
  try {
    await shared().wbProviders?.refreshCustom?.()
  } catch { /* directory not ： use has ， new no */ }
  await load({ force: true })
}

/** （app.js / set / directory can can when use ） */
export function render(): void {
  patch({})
}

/**
 * return 。 return new manage_view，；
 * 、 no has return（next as null）—— new already directory， when to 。
 */
export function accept(next: unknown): void {
  const view = next as ManageView | null
  // return not manage ， by ** that **（、 not ），
  // and current in none —— 「 · 」 1 when ，
  // return is manage ，
  if (view && Array.isArray(view.models)) {
    loadSeq++
    loading = false
    patch({ data: view })
    return
  }
  render()
}

let writeQueue: Promise<unknown> = Promise.resolve()

function enqueueWrite(run: () => Promise<unknown>): Promise<unknown> {
  const next = writeQueue.catch(() => undefined).then(run)
  writeQueue = next
  return next
}

function providerView(provider: string): ManageView {
  return customSource.isCustom(provider) ? customViewOf(provider) : (snapshot.data || EMPTY_VIEW)
}

export function enabledBindingConflicts(provider: string, alias: string, target: string): Array<{ alias: string; target: string }> {
  const view = providerView(provider)
  return activeModelBindings(
    (view.models || []).filter(model => same(model.provider, provider)),
    (view.mappings || []).filter(mapping => !mapping.provider || same(mapping.provider, provider)),
    alias, target,
  )
}

export function addModelError(provider: string, id: string): string {
  const view = providerView(provider)
  if ((view.models || []).some(model => same(model.provider, provider) && same(model.id, id))) {
    return t('模型 ID「{id}」已存在，请使用已有模型', { id })
  }
  return enabledBindingConflicts(provider, id, id).length
    ? t('模型 ID「{id}」在该提供商已有启用绑定，请先关闭它再添加', { id }) : ''
}

export function addMappingError(provider: string, alias: string, target: string): string {
  const view = providerView(provider)
  if (same(alias, target) || (view.mappings || []).some(mapping => same(mapping.alias, alias)
    && same(mapping.target, target) && (!mapping.provider || same(mapping.provider, provider)))) {
    return t('模型 ID「{id}」已绑定到此上游模型，请直接使用已有绑定', { id: alias })
  }
  return enabledBindingConflicts(provider, alias, target).length
    ? t('模型 ID「{id}」在该提供商已有启用绑定，请先关闭它再添加', { id: alias }) : ''
}

/** / add / 1 （alias == target when this default） */
export async function writeBinding(
  provider: string, alias: string, target: string, change: { reasoning?: string; enabled?: boolean; create?: boolean },
): Promise<unknown> {
  if (change.create) {
    const error = addMappingError(provider, alias, target)
    if (error) throw new Error(error)
  }
  if (customSource.isCustom(provider)) return customSource.setBinding(provider, alias, target, { ...change, create: change.create ?? false })
  const api = shared().workbuddyDesktop
  if (!api) throw new Error(t('后端桥不可用'))
  return enqueueWrite(() => api.addModelMapping(alias, target, provider, change.reasoning, change.enabled, change.create ?? false))
}

export async function writeRemoveMapping(provider: string, alias: string, target: string): Promise<unknown> {
  if (customSource.isCustom(provider)) return customSource.removeMapping(provider, alias, target)
  const api = shared().workbuddyDesktop
  if (!api) throw new Error(t('后端桥不可用'))
  return enqueueWrite(() => api.removeModelMapping(alias, target, provider))
}

export async function writeAddModel(provider: string, id: string): Promise<unknown> {
  const error = addModelError(provider, id)
  if (error) throw new Error(error)
  if (customSource.isCustom(provider)) return customSource.addModel(provider, id)
  const api = shared().workbuddyDesktop
  if (!api) throw new Error(t('后端桥不可用'))
  return enqueueWrite(() => api.addCustomModel(provider, id))
}

export async function writeRemoveModel(provider: string, id: string): Promise<unknown> {
  if (customSource.isCustom(provider)) return customSource.removeModel(provider, id)
  const api = shared().workbuddyDesktop
  if (!api) throw new Error(t('后端桥不可用'))
  return enqueueWrite(() => api.removeCustomModel(provider, id))
}

/**
 * 1 can （ path， and `writeBinding` 1 ）：
 * · ：（`customSource.setCapabilities`）；
 * · ：`POST /api/models/capabilities`（ in `modelRules.capabilities`，
 * 1 use ， after `model_rules` 「 can 」）。
 *
 * `capabilities` is ****（ 1 ）：`null` = / 、
 * has = —— and after that 。
 */
export async function writeCapabilities(
  provider: string,
  id: string,
  capabilities: Record<string, number | boolean | null>,
): Promise<unknown> {
  if (customSource.isCustom(provider)) return customSource.setCapabilities(provider, id, capabilities)
  const api = shared().workbuddyDesktop
  if (!api) throw new Error(t('后端桥不可用'))
  return enqueueWrite(() => api.setModelCapabilities(provider, id, capabilities))
}

/* ─── ───────────────────────────────── */

/**
 * 。`key` is （: id， or alias:target:provider）—— 
 * in when in ， by id will 1 「 in 」。
 */
export async function runRowAction(key: string, run: () => Promise<unknown>, doneText?: string): Promise<void> {
  if (snapshot.pending.has(key)) return
  patch({ pending: new Set(snapshot.pending).add(key) })
  try {
    accept(await run())
    if (doneText) toast(doneText)
  } catch (error) {
    toast(t('操作失败：{message}', { message: errorMessage(error) }), 'err')
  } finally {
    const pending = new Set(snapshot.pending)
    pending.delete(key)
    patch({ pending })
  }
}

/**
 * ： has 1 （default or ）enabled， use 。
 * 「 already enabled / already use 」 and use this 1 ， 。
 */
export function rowEnabled(model: ManageModel): boolean {
  return bindingsOf(model).some(binding => binding.enabled !== false)
}

/**
 * 1 （ OmniProxy chips）：default（alias == target） by ，
 * its by target + 。 1 for in is ， chips 
 * that （ provider ； old provider ， for in ）。
 */
export function bindingsOf(model: ManageModel): Binding[] {
  const provider = model.provider || ''
  const matched = mappings().filter(mapping => same(mapping.target, model.id)
    && (!mapping.provider || same(mapping.provider, provider)))
  const unique = new Map<string, ManageMapping>()
  for (const binding of matched) {
    const key = String(binding.alias).toLowerCase()
    if (!unique.has(key) || binding.provider) unique.set(key, binding)
  }
  const idKey = String(model.id).toLowerCase()
  const defaults = unique.get(idKey)
  unique.delete(idKey)
  const rows: Binding[] = [{
    alias: model.id,
    target: model.id,
    provider,
    enabled: model.enabled !== false && defaults?.enabled !== false,
    isDefault: true,
  }]
  rows.push(...unique.values())
  return rows
}

/** ： in when in when ，`id` not 1 */
export const rowKeyOf = (model: ManageModel): string => `${model.provider || ''}:${model.id}`
/** chip ： 1 for + up + 1 1 */
export const bindingKeyOf = (alias: string, target: string, provider: string): string => `${alias}:${target}:${provider || ''}`

export function expandGroup(provider: string): void {
  patch({ expanded: new Set(snapshot.expanded).add(provider) })
}

export function collapseGroup(provider: string): void {
  const expanded = new Set(snapshot.expanded)
  expanded.delete(provider)
  patch({ expanded })
}

export function setStateFilter(next: string): void {
  const value = MODEL_STATES.includes(next) ? next : 'all'
  if (value === snapshot.stateFilter) return
  filtersTouched = true
  patch({ stateFilter: value })
  saveFilters({ state: value })
}

/** （ and 1 ：，） */
export function setSort(next: string): void {
  const value = MODEL_SORTS.includes(next) ? next : 'default'
  if (value === snapshot.sort) return
  filtersTouched = true
  patch({ sort: value })
  saveFilters({ sort: value })
}

/** can （ and 1 ） */
export function setVisibilityFilter(next: string): void {
  const value = MODEL_VISIBILITIES.includes(next) ? next : 'visible'
  if (value === snapshot.visibilityFilter) return
  filtersTouched = true
  patch({ visibilityFilter: value })
  saveFilters({ visibility: value })
}

export function setSearch(next: string): void {
  filtersTouched = true
  patch({ search: next })
  // 1 1 localStorage，none（ is ：，
  // and old 1 innerHTML 1 1 ）
  saveFilters({ search: next })
}

/**
 * in 。 and 「 →」。 in （）；
 * by 1 （ / ）， its to 。
 */
export function selectProvider(id: string): void {
  patch({ provider: String(id ?? '').trim() || 'all' })
  saveFilters({ provider: snapshot.provider })
  syncHead()
}

/**
 * delete 1 （ up that ×）。
 *
 * delete（ / / directory and new ） in custom-provider-ui.js ， and 
 * set that 「delete」 1 ； this ：， after 1 。
 * is **current in that ** when not use ：patch in 1 not already ， will 
 * 「」。
 */
export async function removeCustomProvider(providerId: string): Promise<void> {
  const api = shared().wbCustomProvidersUi
  // down 1 in （custom-provider-ui.js in after ）。 1 ——
  // × not 
  if (!api?.remove) { toast(t('删除入口未就绪，请重试或重启应用'), 'err'); return }
  if (!(await api.remove(providerId))) return
  // ： this from after directory（ not and by ）， that manage
  // can can 
  await load({ force: true })
}

/** 「＋ new 」：「」 and new （ not ） */
export function openAddCustomProvider(): void {
  shared().wbAccountAddForms?.openNewCustomForm?.()
}

/** 。`context` is up down （ + up ， for ）。 */
export function openMapping(context: MappingContext): void {
  // not use in this ：patch （ patch ）， is new that 
  patch({ mapping: context })
}

export function closeMapping(): void {
  patch({ mapping: null })
}

/**
 * 「」。none up down （ has by 1 ）， is add。
 * in when this is **** ： to down ， not （ not this ） 1 ； then 
 * current in that 1 （ when ， this is need ）。
 */
export function openCustomModel(): void {
  const provider = currentProvider()
  // 「 · 」 not ： need to 1 down ， use no to 1 ，
  // to （ and 「」 1 as ）； that 1 
  const custom = customSource.isCustom(provider)
  const preselect = custom || (provider !== 'all' && provider !== ALL_CUSTOM) ? provider : ''
  patch({ customModel: { provider: preselect, locked: custom } })
}

export function closeCustomModel(): void {
  patch({ customModel: null })
}

/**
 * can （ 「 up down / 」 and 「 can 」，）。
 * up down `(provider, id)`： can can new ， from current
 * （ `modelRowOf`）， not up down 。
 */
export function openCapability(provider: string, id: string): void {
  patch({ capability: { provider, id } })
}

export function closeCapability(): void {
  patch({ capability: null })
}

/**
 * by `(provider, id)` 1 （ can / / ）。
 *
 * id ignore（ and for 1 ）； not in （directory new after 
 * 、 ）return null， by + 。
 */
export function modelRowOf(provider: string, id: string): ManageModel | null {
  const view = providerView(provider)
  return (view.models || []).find(model => same(model.provider, provider) && same(model.id, id)) || null
}

/**
 * ：directory ，， no has when 
 * id。 provider id when will 1 `custom-3f2a91b04c7e`，
 * use not is 1 。
 */
export function providerLabelOf(provider: string): string {
  if (customSource.isCustom(provider)) {
    // is use （）， not 
    const name = customSource.record(provider)?.name
    return (typeof name === 'string' && name) || provider
  }
  return t(shared().wbProviders?.labelOf?.(provider) || provider)
}

/**
 * 「get」：， in models-fetch-modal.tsx（ wbModelsFetchModal）。
 * up down （ in 、、success）， not and 。
 *
 * new = has ∪ has enabled （），
 * 「」， by 。
 *
 * new after must** 1 **： is ， new is that 
 * /api/models/refresh —— directory in after 1 ， manage （、 
 * 「」、 new ） is old 。
 */
export function refreshModels(): void {
  const provider = currentProvider()
  const custom = customSource.isCustom(provider)
  const name = custom
    ? (customSource.record(provider)?.name || provider)
    : t('内置提供商')
  shared().wbModelsFetchModal?.open({
    // 「 · 」 no has 1 up can （ Base URL not ），
    // providerId empty， by 「」 new 
    providerId: provider === ALL_CUSTOM ? '' : provider,
    custom,
    name,
    // ：success → directory already new ， can （ is ）
    onDone: () => render(),
    // ：directory → after ，must（force： this is 「 new already 」 ，
    // not can 1 in ）
    onRefreshed: () => { void load({ force: true }) },
  })
}

/* ─── for ───────────────────────────────── */

/**
 * visibleColumns to table-columns.js： that 1 need by current can （ to <col>、
 * not to ）， 1 config not will 1 。
 * selectProvider to （「 →」 and in this ）。
 * providerRefreshedAt to 「get」 「update」；builtinProviders to new 。
 */
window.wbModelsPanel = {
  render, load, refreshAll, refreshModels, visibleColumns, selectProvider, builtinProviders,
  providerRefreshedAt,
}

declare global {
  interface Window {
    /** （ ui/models-panel.js， and 1 ） */
    wbModelsPanel?: {
      render(): void
      load(options?: { force?: boolean }): Promise<void>
      refreshAll(): Promise<void>
      refreshModels(): void
      visibleColumns(): ColumnView[]
      selectProvider(id: string): void
      builtinProviders(): string[]
      providerRefreshedAt(providerId: string): number
    }
  }
}

/**
 * 「 → etc 」 。（reasoningOf） by patch （ that ），
 * this 1 is as can —— to is current that 1 。
 */
export function levelOf(alias: unknown, target: unknown, provider: unknown): string {
  return reasoningOf(alias, target, provider)
}

/**
 * （ need use ）。 not can to `tableEl` 
 *（ is ）， this setter —— by `<table ref>` in when use 。
 */
export function setTableEl(el: HTMLTableElement | null): void {
  tableEl = el
}

/**
 * ** not React ** （ #195， by after use ）。
 *
 * `<table>` this has colgroup / thead / tbody#models ， by 
 * React 。 React ** not will ** ：，
 * tbody 、 or table tbody， after render() / load() not will 
 * ， will 1 in 。issue #195 is this 1 ： not current、 not 
 * statistics， and in that tbody down 1 （tbody after 1 no has 
 * down ）。
 *
 * is React to fiber （`__reactFiber$…` / old 
 * `__reactInternalInstance$…`）： no has this 。 is 「 not 
 * React」 ， not will and React diff ， 1 not will 。
 */
export function pruneForeignTableNodes(): void {
  const table = tableEl
  if (!table) return
  const owned = (el: Element): boolean => Object.keys(el).some(
    key => key.startsWith('__reactFiber$') || key.startsWith('__reactInternalInstance$'))
  for (const child of [...table.children]) {
    if (child.tagName === 'TBODY') {
      if (child.id !== 'models') child.remove()
    } else if (child.tagName === 'TR' && !owned(child)) {
      // in table down tr will by 「 tbody」 1 ， need 
      child.remove()
    }
  }
  const tbody = table.querySelector('tbody#models')
  if (!tbody) return
  for (const row of [...tbody.children]) {
    if (!owned(row)) row.remove()
  }
}
