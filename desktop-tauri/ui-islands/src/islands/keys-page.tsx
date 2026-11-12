import * as React from 'react'
import { mountOnPage } from '../island-lazy'
import { createRoot } from 'react-dom/client'
import {
  Badge,
  Button,
  Dialog,
  DialogBody,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogSection,
  DialogTitle,
  Input,
  Label,
  MultiSelect,
  Switch,
  type MultiSelectOption,
  Card, CardHeader, Table, TableHeader, TableRow, TableHead, TableBody, TableCell } from '@ui'
import { TableFooter, useClientPaging } from './table-shell'
import { t } from '../i18n'

/**
 * Agent2API · Key （ / new / / delete / available）—— React 。
 *
 * ui/keys-panel.js（ that use innerHTML .models-table 、）。
 * for and ** 1 **：`window.wbKeysPanel = { load, render, visibleColumns }`
 * —— app.js:151 to when load()，table-columns.js by visibleColumns()
 * 「current can 」（ to <col>、 not to ）， use 1 not use 。
 *
 * ── （ old ，）──────────────────────────────────
 * `GET /api/keys`（`{keys, authRequired, providers, modelsByProvider}`，
 * keys key and ）。return new ， after ；default，
 * can 「」 and （ clipboard.js data-copy ）。
 * 「available / available」 **empty = unlimited**（ after core::api_keys）。
 *
 * ── MultiSelect ───────────────────────────
 * 「available / available」 is **** React state（ not `<select multiple>`
 * select.js ）：、、 in this 1 —— then modelOptions，
 * to after pickedFromOptions。
 *
 * ── Dialog（ and request-clear-modal / conc-dialog 1 ）──
 * old `#key-modal`（.modal-mask 1 ） not use ：Esc / 、、
 * by Dialog 。MultiSelect is Base UI （portal to body，
 * z-35 z-30）， and use 1 outside-press ， not need need 。
 *
 * ── set：Why in layout effect ， not is ──────
 * ① table-col-settings.tsx （import.meta.glob by file），
 * that 1 `window.wbColSettings` not in ；
 * ② 「set」 by need ** ** .panel-head .head-actions， React 
 * in after —— querySelector to is index.html 
 * （ up will replaceChildren ）， by will 1 will 。
 * 1 ， and syncStaticHead `wbTableColumns.repaint('keys')`
 * up and ：table-columns.js in after ， has can can —— when 
 * not to （ when React no ）， this repaint up ； this is empty when ， will 。
 * <colgroup> / <thead> by file `data-col` 1 not ，React from not this 
 * （ DOM not ）， / is 。
 */

/* ─── ─────────────────────────────────── */

/** 1 Key（`GET /api/keys` keys[]， for after `api_keys::ApiKeyEntry::public_json`） */
type KeyEntry = {
  id: string
  name?: string
  /** ： need can when to client（used for） */
  key?: string
  masked?: string
  enabled?: boolean
  createdAt?: number
  /** ，**empty = unlimited** */
  allowedProviders?: string[]
  allowedModels?: string[]
}

/**
 * 「available」 ： after —— （）+ already 
 * （ provider ； is ， has after to ）。
 */
type ProviderOption = { id: string; label?: string }

/** response；`created` in POST response（ new after need ） */
type KeysPayload = {
  keys?: KeyEntry[]
  authRequired?: boolean
  usableKeyCount?: number
  providers?: ProviderOption[]
  /** → for ， ** 1 **（ by current and ） */
  modelsByProvider?: Record<string, string[]>
  created?: KeyEntry
}

/** use to after （ bridge.rs 「 Key」 that 1 ） */
type KeysBridge = {
  getKeys(): Promise<KeysPayload | null | undefined>
  createKey(payload: {
    name: string
    /** empty = after */
    key?: string
    allowedProviders: string[]
    allowedModels: string[]
  }): Promise<KeysPayload | null | undefined>
  updateKey(
    id: string,
    patch: { enabled?: boolean; allowedProviders?: string[]; allowedModels?: string[] },
  ): Promise<KeysPayload | null | undefined>
  deleteKey(id: string): Promise<KeysPayload | null | undefined>
}

/**
 * window up by its / its 。
 *
 * use 「 + 」 not is declare global Window up ：
 * workbuddyDesktop / wbApp / wbColSettings / wbProviders is use ， if 
 * declare 1 ， and will not 1 TS2717。file declare
 * wbKeysPanel（file）。
 */
type SharedWindow = {
  workbuddyDesktop?: KeysBridge
  wbApp?: {
    toast?: (message: string, kind?: 'err' | 'ok') => void
    /** when → when （app.js formatTime；createdAt use ） */
    formatTime?: (value: unknown) => string
    /** ： is that （ by id and data-tone） */
    renderTopbarStatus?: () => void
    /** current： in use when after */
    readonly currentPage?: string
  }
  wbConfirm?: {
    ask?: (options: {
      title?: string
      /** ， <strong> etc ； by use */
      html?: string
      okText?: string
      /** danger = not can （） */
      okClass?: string
    }) => Promise<boolean>
  }
  wbColSettings?: {
    register(spec: {
      id: string
      label?: string
      columns: readonly ColumnDecl[]
      mount?: () => Element | null
      onChange?: () => void
    }): ColSettingsHandle
    syncStaticHead(id: string, table: Element | null | undefined): void
  }
  /** （ + ， no has id ） */
  wbProviders?: { labelOf?: (id: string) => string | undefined }
}

function shared(): SharedWindow {
  return window as unknown as SharedWindow
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}

/** toast 1 （ wbApp， not in ） */
function toast(message: string, kind?: 'err' | 'ok'): void {
  shared().wbApp?.toast?.(message, kind)
}

/** is HTML ， 1 （ not wbApp.esc： that is app.js has ） */
function escapeHtml(value: string): string {
  return value.replace(/[&<>"']/g, ch => ({
    '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;',
  }[ch] ?? ch))
}

/* ─── and ───────────────────────────── */

type Align = 'left' | 'center' | 'right'

/** ：key and index.html has `data-col`、<col> data-col */
type ColumnDecl = { key: string; label: string; align?: Align }

const COLUMNS: readonly ColumnDecl[] = [
  { key: 'name', label: t('名称') },
  // 「Key」 is （none in ）， not 
  { key: 'key', label: 'Key' },
  { key: 'time', label: t('创建时间') },
  { key: 'state', label: t('启用') },
  { key: 'act', label: t('操作'), align: 'right' },
]

/** （`state` / `r` is has CSS ， page-gateway.css） */
const CELL_CLASS: Record<string, string> = {
  name: 'cell-name',
  key: 'cell-key',
  time: 'cell-time',
  state: 'cell-state state',
  act: 'cell-act r',
}

/** （React root in up ，file） */
const SECTION = '.page[data-page="keys"]'

/* ─── set： can （register / apply / syncStaticHead）─── */

type ColSettingsHandle = {
  apply<C extends { key: string }>(columns: C[]): (C & { align: Align })[]
  config(): unknown
}

let colSettings: ColSettingsHandle | null = null

/** set after ： after （onChange from React ） */
let onColumnsChanged: (() => void) | null = null

/**
 * this current can （config；set when ）。
 * to table-columns.js： that 1 need by current can （ to <col>、
 * not to ）， 1 config not will 1 。
 */
function visibleColumns(): (ColumnDecl & { align: Align })[] {
  if (colSettings) return colSettings.apply([...COLUMNS])
  // set no （failure etc ）：， for up default
  return COLUMNS.map(column => ({ ...column, align: column.align ?? 'left' }))
}

/** ： / / for （ for 1 ） */
function syncHead(): void {
  shared().wbColSettings?.syncStaticHead?.('keys', document.querySelector('table.keys-table'))
}

/** set（ 1 ） and ；must in React after use （file） */
function setupColumns(): void {
  if (colSettings) return
  const handle = shared().wbColSettings?.register({
    id: 'keys',
    label: t('网关 Key 表'),
    columns: COLUMNS,
    mount: () => document.querySelector('.page[data-page="keys"] .panel-head .head-actions'),
    onChange: () => {
      syncHead()
      onColumnsChanged?.()
    },
  })
  if (!handle) return
  colSettings = handle
  syncHead()
}

/* ─── ： and ───────────────────── */

function keysOf(payload: KeysPayload | null): KeyEntry[] {
  return Array.isArray(payload?.keys) ? payload.keys : []
}

/** after down need （：workbuddy → raccoon → …， in ） */
function providersOf(payload: KeysPayload | null): ProviderOption[] {
  return Array.isArray(payload?.providers) ? payload.providers : []
}

function modelsByProviderOf(payload: KeysPayload | null): Record<string, string[]> {
  const map = payload?.modelsByProvider
  return map && typeof map === 'object' ? map : {}
}

/** ：； no has id （ old can can has already down ） */
function providerLabel(id: string): string {
  return t(shared().wbProviders?.labelOf?.(id) || id)
}

/**
 * by **current ** for and ，。
 * `selected` not in and （ already ）—— use 
 * after ， that has need 、 can ，otherwise「」 will 1 
 * 。 1 no when return**empty**（ no has no has ）。
 */
function modelOptions(
  table: Record<string, string[]>,
  selectedProviders: string[],
  selected: string[],
): MultiSelectOption[] {
  const names = new Map<string, string>() // 小写 → 原始名（先到先得，保住后端给的大小写）
  const put = (value: unknown) => {
    const text = String(value ?? '').trim()
    if (!text) return
    const key = text.toLowerCase()
    if (!names.has(key)) names.set(key, text)
  }
  selectedProviders.forEach(id => {
    const list = table[id] ?? table[String(id).toLowerCase()]
    if (Array.isArray(list)) list.forEach(put)
  })
  // already none is in and need （）
  selected.forEach(put)
  // **** is （ to after ） not ； label t()
  const out = [...names.values()].map(value => ({ value, label: t(value) }))
  out.sort((a, b) => (a.value.toLowerCase() < b.value.toLowerCase() ? -1 : 1))
  return out
}

/**
 * 1 Key down **limit need **。Why need not is empty：limit is ** not ** —— up 
 * not ， use 「」，client 404 when will 、， after to is 
 * Key limit。nonelimit when 「unlimited」 not is ： and 「 no to 」 1 。
 */
function restrictionText(k: KeyEntry): string {
  const providers = Array.isArray(k.allowedProviders) ? k.allowedProviders : []
  const models = Array.isArray(k.allowedModels) ? k.allowedModels : []
  if (!providers.length && !models.length) return t('不限制')
  const parts: string[] = []
  // providerLabel already is （ in this ），
  if (providers.length) parts.push(providers.map(providerLabel).join(t('、')))
  // that ： 1 Row not down ， by title to 
  if (models.length) parts.push(t('{n} 个模型', { n: models.length }))
  return t('限制：{text}', { text: parts.join(' / ') })
}

/** limit need （ title use ； not ， can this ） */
function restrictionTitle(k: KeyEntry): string {
  const providers = Array.isArray(k.allowedProviders) ? k.allowedProviders : []
  const models = Array.isArray(k.allowedModels) ? k.allowedModels : []
  if (!providers.length && !models.length) {
    return t('这把 Key 不限制提供商与模型（可用全部上游与全部对外模型）')
  }
  const lines: string[] = []
  if (providers.length) lines.push(t('可用提供商：{names}', { names: providers.map(providerLabel).join(t('、')) }))
  if (models.length) lines.push(t('可用模型：{names}', { names: models.join(t('、')) }))
  return lines.join('\n')
}

/**
 * 「current need 」。Why need has ：limit is ** not ** —— 1 ， up 
 * 「limit：…」 1 ； after 1 ， will 。
 * use 「」 not to ， / in this to for 1 。
 */
function restrictionSummary(providers: readonly string[], models: readonly string[]): string {
  if (!providers.length && !models.length) {
    return t('当前不限制：这把 Key 可以用全部提供商与全部对外模型')
  }
  const parts: string[] = []
  if (providers.length) parts.push(t('提供商：{names}', { names: providers.map(providerLabel).join(t('、')) }))
  if (models.length) parts.push(t('模型：{names}', { names: models.join(t('、')) }))
  // limit、 no limit（ old can can in this ）： already 
  // that （ no has no has and can ）， need —— otherwise use will as 
  // 「， not new 」
  if (!providers.length) {
    parts.push(t('（模型候选需先选提供商；不选则沿用当前这几项，保存后仍按模型白名单生效）'))
  }
  return parts.join('　')
}

/* ─── and ───────────────────────── */

/** is after to ， can can / not for （ 1 empty， and old ） */
function stringList(value: unknown): string[] {
  return Array.isArray(value) ? value.map(item => String(item)) : []
}

/**
 * state by **** 1 ： in 、、
 * as （ can can and not 1 ， old is by for after DOM）。
 * this is old `[...select.selectedOptions].map(option => option.value)` ——
 * that when 「」 in option up ， no has id not to DOM ， when 
 * 。** to after must this **。
 */
function pickedFromOptions(values: readonly string[], options: readonly MultiSelectOption[]): string[] {
  const canonical = new Map(options.map(option => [option.value.toLowerCase(), option.value]))
  const chosen = new Set<string>()
  values.forEach(value => {
    const hit = canonical.get(String(value).toLowerCase())
    if (hit) chosen.add(hit)
  })
  return options.filter(option => chosen.has(option.value)).map(option => option.value)
}

/* ─── （ new / available use ）──────────────── */

type KeyModalProps = {
  /** Key；null = new */
  target: KeyEntry | null
  providers: ProviderOption[]
  modelsByProvider: Record<string, string[]>
  onClose: () => void
  /** return new （ and old accept ）；revealId = new after need that */
  onSaved: (next: KeysPayload | null | undefined, revealId?: string) => void
}

function KeyModal({ target, providers, modelsByProvider, onClose, onSaved }: KeyModalProps) {
  const editingId = target?.id ?? null
  const editing = Boolean(editingId)

  const [name, setName] = React.useState(target?.name ?? '')
  const [keyValue, setKeyValue] = React.useState('')
  const [status, setStatus] = React.useState('')
  const [saving, setSaving] = React.useState(false)
  /** in ： （Esc / ）must can **** to */
  const savingRef = React.useRef(false)

  /** （；**empty = unlimited**）。 after to ， not in this */
  const [pickedProviders, setPickedProviders] = React.useState<string[]>(
    () => stringList(target?.allowedProviders),
  )
  const [pickedModels, setPickedModels] = React.useState<string[]>(
    () => stringList(target?.allowedModels),
  )

  /**
   * = after down （ + already ）。must in this 1 
   * ：`pickedFromOptions` has id，， already `custom-xxx`
   * will in when （ after `keys_api` `providers` ）。
   */
  const providerOptions = React.useMemo<MultiSelectOption[]>(
    // is id（ to after ） not ； label t()（ after down ）
    () => providers.map(item => ({ value: String(item.id), label: t(String(item.label ?? item.id)) })),
    [providers],
  )

  /**
   * by 1 after —— is ** 1 for ** ：MultiSelect 、 need 、
   * to after use ， and old from `<select>` selectedOptions 1 
   * （ pickedFromOptions ）。
   */
  const allowedProviders = React.useMemo(
    () => pickedFromOptions(pickedProviders, providerOptions),
    [pickedProviders, providerOptions],
  )

  /**
   * 「 already 」： 1 this for ， 1 then 。
   * ** already 1 in **（ modelOptions）—— use no that ，
   * not this 。
   */
  const modelCandidates = React.useMemo(
    () => modelOptions(modelsByProvider, allowedProviders, pickedModels),
    [modelsByProvider, allowedProviders, pickedModels],
  )

  const allowedModels = React.useMemo(
    () => pickedFromOptions(pickedModels, modelCandidates),
    [pickedModels, modelCandidates],
  )

  /**
   * empty ： no when is 「」， is empty is 「 this 
   * in no has available」（ after no to this = no ）。 has when not this ——
   * that when empty can can is no up 。
   */
  const modelEmptyHint = modelCandidates.length
    ? t('没有匹配的选项')
    : allowedProviders.length
      ? t('这几家当前没有可用模型（账号未登录或清单为空）')
      : t('请先在上面选择可用提供商')

  /** ： in （，） */
  function stopSaving(): void {
    savingRef.current = false
    setSaving(false)
  }

  async function save(): Promise<void> {
    if (savingRef.current) return
    // use 1 after （ pickedFromOptions）， and and old 1 
    savingRef.current = true
    setSaving(true)
    setStatus(t('保存中…'))
    try {
      const api = shared().workbuddyDesktop
      if (!api) throw new Error(t('后端桥不可用'))
      if (editingId) {
        // ： and not （update， after api_keys::update）
        const next = await api.updateKey(editingId, { allowedProviders, allowedModels })
        // （ old ：saving = false in closeModal before ）
        stopSaving()
        onSaved(next)
        onClose()
        toast(t('✅ 可用范围已保存'))
        return
      }
      const trimmedKey = keyValue.trim()
      if (trimmedKey && trimmedKey.length < 8) {
        setStatus(t('Key 至少需要 8 个字符'))
        return
      }
      const next = await api.createKey({
        name: name.trim(), key: trimmedKey || undefined, allowedProviders, allowedModels,
      })
      stopSaving()
      onSaved(next, next?.created?.id)
      onClose()
      toast(t('✅ Key 已创建，记得复制给客户端'))
    } catch (error) {
      setStatus(t('保存失败：{reason}', { reason: errorMessage(error) }))
    } finally {
      stopSaving()
    }
  }

  return (
    <Dialog
      open
      onOpenChange={(next, eventDetails) => {
        // request（Esc / / up ✕） to this 。 in must
        // eventDetails.cancel()：「 not update open prop」 not Base UI store。
        if (next) return
        if (savingRef.current) {
          eventDetails.cancel()
          return
        }
        onClose()
      }}
    >
      <DialogContent>
        <DialogHeader>
          <DialogTitle>
            {editing
              ? t('可用范围 · {name}', { name: target?.name || t('未命名') })
              : t('新建 API Key')}
          </DialogTitle>
        </DialogHeader>
        <DialogBody>
          <DialogSection>
            <div className='flex flex-wrap items-center gap-2.5'>
              <Label htmlFor='key-name' className='text-[12.5px] whitespace-nowrap text-subtle'>{t('名称')}</Label>
              <Input id='key-name' type='text' maxLength={60} placeholder={t('例如 Cursor / 公司电脑')}
                autoComplete='off' value={name} disabled={editing}
                onChange={event => setName(event.currentTarget.value)} />
            </div>
            {/* ** not ** Key ： that 1 in when no has （Key not can ）。
                 this no has need no has need ， —— to 
                must （ globals.css [hidden][hidden] already to ）。 */}
            {!editing && (
              <div className='flex flex-wrap items-center gap-2.5'>
                <Label htmlFor='key-value' className='text-[12.5px] whitespace-nowrap text-subtle'>Key</Label>
                <Input id='key-value' type='text' placeholder={t('留空自动生成；手填至少 8 个字符')}
                  autoComplete='off' spellCheck={false} value={keyValue}
                  onChange={event => setKeyValue(event.currentTarget.value)}
                  // = （ old in this 1 up ）
                  onKeyDown={event => { if (event.key === 'Enter') void save() }} />
              </div>
            )}
            {/* MultiSelect（）。 then （min 180 / max 260） is 
                page-gateway.css by `#key-allowed-providers` / `#key-allowed-models` to ，
                 MultiSelect not id（ not ）—— that then then ，
                 etc can ：flex-auto is old CSS `flex: 1 1 auto`。
                 up no has can label and （ no has id can to htmlFor）， aria-label
                must to ，otherwise to 1 。 */}
            <div className='flex flex-wrap items-center gap-2.5'>
              <Label className='text-[12.5px] whitespace-nowrap text-subtle'>{t('可用提供商')}</Label>
              <MultiSelect
                value={allowedProviders}
                onValueChange={setPickedProviders}
                options={providerOptions}
                placeholder={t('留空 = 不限制')}
                searchPlaceholder={t('搜索提供商…')}
                aria-label={t('可用提供商')}
                className='flex-auto min-w-[180px] max-w-[260px]'
              />
            </div>
            <div className='flex flex-wrap items-center gap-2.5'>
              <Label className='text-[12.5px] whitespace-nowrap text-subtle'>{t('可用模型')}</Label>
              <MultiSelect
                value={allowedModels}
                onValueChange={setPickedModels}
                options={modelCandidates}
                placeholder={t('留空 = 不限制')}
                emptyHint={modelEmptyHint}
                searchPlaceholder={t('搜索模型…')}
                aria-label={t('可用模型')}
                className='flex-auto min-w-[180px] max-w-[260px]'
              />
            </div>
            <p>
              {t('留空表示不限制；同时设置时请求需同时满足两个条件（模型在白名单内且路由到允许的提供商）。 「可用模型」的候选跟着上面勾选的提供商走：没勾提供商时它是空的（还没有约束范围）， 勾了几家就列出这几家能收的全部对外名。')}
            </p>
            {/* current need ： up 「 after 1 」， will 
                 —— / need in this 。id use old ：page-gateway.css
                 by to this up （ is 「current」 not is 「 use 」）。 */}
            <p id='key-restrict-summary'>{restrictionSummary(allowedProviders, allowedModels)}</p>
          </DialogSection>
          <div className='min-h-[18px] text-[11.5px] text-muted-foreground'>{status}</div>
        </DialogBody>
        <DialogFooter>
          <div className='mr-auto' />
          <Button variant='outline' onClick={onClose} disabled={saving}>
            {t('取消')}
          </Button>
          <Button variant='default' disabled={saving} onClick={() => void save()}>
            {editing ? t('保存范围') : t('创建')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

/* ─── ───────────────────────────────── */

/** after ： for load / render */
type PageHandle = {
  load(): Promise<void>
  /** old （currentnone use ，） */
  render(): void
}

let handle: PageHandle | null = null

/** app.js to when 1 （ before use file ） */
async function load(): Promise<void> {
  await handle?.load()
}

/** old can ： by current 1 */
function render(): void {
  handle?.render()
}

function KeysPage() {
  const [data, setData] = React.useState<KeysPayload | null>(null)
  /** already to key id */
  const [revealed, setRevealed] = React.useState<ReadonlySet<string>>(() => new Set())
  /** in （ by and use ）； down ref */
  const [pending, setPending] = React.useState<ReadonlySet<string>>(() => new Set())
  /** ：null = ；{ key: null } = new */
  const [modal, setModal] = React.useState<{ key: KeyEntry | null } | null>(null)
  /** set / render() → （ no can ） */
  const [, setVersion] = React.useState(0)

  const pendingRef = React.useRef<Set<string>>(new Set())
  const loadingRef = React.useRef(false)

  const applyData = React.useCallback((next: KeysPayload | null) => {
    setData(next)
  }, [])

  /** return new ： not for （failure） when ， and old accept */
  const accept = React.useCallback((next: KeysPayload | null | undefined) => {
    if (next && Array.isArray(next.keys)) applyData(next)
  }, [applyData])

  const loadPanel = React.useCallback(async (): Promise<void> => {
    if (loadingRef.current) return
    loadingRef.current = true
    try {
      const api = shared().workbuddyDesktop
      if (!api) throw new Error(t('后端桥不可用'))
      applyData((await api.getKeys()) ?? null)
    } catch (error) {
      toast(t('读取 Key 列表失败：{reason}', { reason: errorMessage(error) }), 'err')
    } finally {
      loadingRef.current = false
    }
  }, [applyData])

  /** 1 ： → → use return new → （failure toast） */
  async function runRowAction(
    id: string,
    run: () => Promise<KeysPayload | null | undefined>,
    doneText?: string,
  ): Promise<void> {
    if (pendingRef.current.has(id)) return
    pendingRef.current.add(id)
    setPending(new Set(pendingRef.current))
    try {
      accept(await run())
      if (doneText) toast(doneText)
    } catch (error) {
      toast(t('操作失败：{reason}', { reason: errorMessage(error) }), 'err')
    } finally {
      pendingRef.current.delete(id)
      setPending(new Set(pendingRef.current))
    }
  }

  function toggleReveal(id: string): void {
    setRevealed(prev => {
      const next = new Set(prev)
      if (next.has(id)) next.delete(id)
      else next.add(id)
      return next
    })
  }

  /** delete： confirm in Tauri WebView not 、（ etc no has ） */
  async function removeKey(k: KeyEntry): Promise<void> {
    const api = shared().workbuddyDesktop
    const ask = shared().wbConfirm?.ask
    if (!api || !ask) return
    const label = escapeHtml(k.name || k.masked || k.id)
    const confirmed = await ask({
      title: t('删除网关 Key'),
      html: t('确定删除 Key「<strong>{name}</strong>」？使用它的客户端会立刻无法访问。', { name: label }),
      okText: t('删除'),
      okClass: 'danger',
    })
    if (!confirmed) return
    void runRowAction(k.id, () => api.deleteKey(k.id), t('Key 已删除'))
  }

  /** ： is new ，（ and old runRowAction ） */
  function toggleEnabled(k: KeyEntry, next: boolean): void {
    const api = shared().workbuddyDesktop
    if (!api) return
    void runRowAction(
      k.id,
      () => api.updateKey(k.id, { enabled: next }),
      next ? t('Key 已启用') : t('Key 已停用'),
    )
  }

  /* ─── ： + set（file）────── */

  React.useLayoutEffect(() => {
    handle = {
      load: loadPanel,
      render: () => setVersion(version => version + 1),
    }
    onColumnsChanged = () => setVersion(version => version + 1)
    setupColumns()
    // must 1 ： visibleColumns() is 「set」 
    // （、）， use and will for not up 。layout effect 
    // setState is ， in before ， not to this 1 。
    setVersion(version => version + 1)
    return () => {
      handle = null
      onColumnsChanged = null
    }
  }, [loadPanel])

  /**
   * ：app.js showPage in before （ that when window.wbKeysPanel
   * not in ， that use empty）， use up if in ， this 1 —— and old file
   * `if (wbApp.currentPage === 'keys') load()` etc 。
   */
  React.useEffect(() => {
    if (shared().wbApp?.currentPage === 'keys') void loadPanel()
  }, [loadPanel])

  /** that is （app.js renderTopbarStatus by id and data-tone）：
   * 1 up ，otherwise need etc down 1 （20 seconds） */
  React.useEffect(() => {
    shared().wbApp?.renderTopbarStatus?.()
  }, [data])

  /* ─── ─────────────────────────────── */

  const keys = keysOf(data)
  /** client（ use ）：Key has ， and its 1 */
  const paging = useClientPaging(keys.length, 'keys')
  const list = paging.paged ? paging.slice(keys) : keys
  const columns = visibleColumns()
  const enabledCount = data?.usableKeyCount ?? keys.filter(item => item.enabled).length
  const hasUsableKey = enabledCount > 0
  const badgeText = hasUsableKey
    ? t('已启用鉴权 · {n} 把 Key 生效', { n: enabledCount })
    : t('没有可用的 API Key，请先在「网关 Key」页创建并启用')

  /** 1 （ not <td> ）；「 1 」 has this 1 */
  function cell(columnKey: string, k: KeyEntry, busyRow: boolean): React.ReactNode {
    switch (columnKey) {
      case 'name':
        return (
          <>
            <div className='mid'><span className='t'>{k.name || t('未命名')}</span></div>
            <div className='mname' title={restrictionTitle(k)}>{restrictionText(k)}</div>
          </>
        )
      case 'key': {
        const shown = revealed.has(k.id)
        return (
          <div className='keycell'>
            <code className='kv'>{shown ? k.key : k.masked}</code>
            <Button size='sm' variant='ghost' onClick={() => toggleReveal(k.id)}>
              {shown ? t('隐藏') : t('显示')}
            </Button>
            {/* data-copy is clipboard.js */}
            <Button size='sm' variant='ghost' data-copy={k.key} title={t('复制 Key')}>{t('复制')}</Button>
          </div>
        )
      }
      case 'time':
        return (
          <span className='muted'>{k.createdAt ? shared().wbApp?.formatTime?.(k.createdAt) : '—'}</span>
        )
      case 'state':
        return (
          <Switch checked={k.enabled === true} disabled={busyRow}
            aria-label={t('启用「{name}」', { name: k.name || t('未命名') })}
            onCheckedChange={next => toggleEnabled(k, next)} />
        )
      case 'act':
        return (
          <div className='row-actions'>
            <Button size='sm' variant='ghost' disabled={busyRow} onClick={() => setModal({ key: k })}>
              {t('可用范围')}
            </Button>
            <Button size='sm' variant='destructive' disabled={busyRow}
              onClick={() => void removeKey(k)}>
              {t('删除')}
            </Button>
          </div>
        )
      default:
        return null
    }
  }

  return (
    <Card render={<section />} className='panel'>
      <CardHeader className='panel-head'>
        <h2>{t('API Key 列表')}</h2>
        {/* id and data-tone ：app.js renderTopbarStatus by id this 
             and （data-tone has ， not Badge that Tailwind ） */}
        <Badge id='keys-status' variant={hasUsableKey ? 'success' : 'warning'}
          data-tone={hasUsableKey ? 'ok' : 'warn'}>
          {badgeText}
        </Badge>
        <div className='head-actions'>
          {/* set by by wbColSettings.register this before （，
               and 1 ： by that ，） */}
          <Button variant='default' onClick={() => setModal({ key: null })}>
            {t('＋ 新建 Key')}
          </Button>
        </div>
      </CardHeader>

      <div className='models-table-wrap'>
        <Table container={false} className='models-table keys-table'>
          {/* colgroup / thead by file， data-col 1 not ：set 
              （syncStaticHead） and （table-columns.js） by */}
          <colgroup>
            <col className='k-name' data-col='name' />
            <col className='k-key' data-col='key' />
            <col className='k-time' data-col='time' />
            <col className='k-state' data-col='state' />
            <col className='k-act' data-col='act' />
          </colgroup>
          <TableHeader>
            <TableRow>
              <TableHead data-col='name'>{t('名称')}</TableHead>
              {/* 「Key」 is （none in ）， not */}
              <TableHead data-col='key'>Key</TableHead>
              <TableHead data-col='time'>{t('创建时间')}</TableHead>
              <TableHead data-col='state'>{t('启用')}</TableHead>
              <TableHead className='r' data-col='act'>{t('操作')}</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {list.length ? list.map(k => {
              const busyRow = pending.has(k.id)
              return (
                <TableRow key={k.id} className={k.enabled ? '' : 'off'} data-id={k.id}>
                  {columns.map(column => (
                    <TableCell key={column.key} className={`${CELL_CLASS[column.key] ?? ''} ta-${column.align}`}>
                      {cell(column.key, k, busyRow)}
                    </TableCell>
                  ))}
                </TableRow>
              )
            }) : (
              // empty colspan can ： 5 after ， this 1 will 
              // ， to 
              <TableRow>
                <TableCell colSpan={columns.length} className='empty'>
                  {data ? t('没有可用的 API Key，请先在「网关 Key」页创建并启用') : t('加载中…')}
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </div>

      <TableFooter
        leading={(
          // <code> / <b> （ and docs-page 1 ）；
          // empty is JSX empty，
          <span>
            {t('客户端请求需带 ')}
            <code>{'Authorization: Bearer <key>'}</code>
            {t(' 或 ')}
            <code>{'x-api-key: <key>'}</code>
            {t('；修改后立即生效。模型测试、助手和日活保活必须选择 Key，受限 Key 不能访问管理接口。每把 Key 可单独限制')}
            <b>{t('可用提供商')}</b>
            {t('与')}
            <b>{t('可用模型')}</b>
            {t('（行内「可用范围」）：留空 = 不限制，两个都设时 按交集生效 —— 被限制的模型对这把 Key 表现为「不存在」（拉 /v1/models 也看不到它）， 提供它的家不在可用列表里时请求同样被拒。')}
          </span>
        )}
        total={keys.length}
        range={paging.paged ? { start: paging.rangeStart, end: paging.rangeEnd } : null}
        page={paging.page}
        pageCount={paging.pageCount}
        size={paging.size}
        onSizeChange={paging.setSize}
        onPageChange={paging.goto}
      />

      {modal ? (
        <KeyModal
          target={modal.key}
          providers={providersOf(data)}
          modelsByProvider={modelsByProviderOf(data)}
          onClose={() => setModal(null)}
          onSaved={(next, revealId) => {
            if (revealId) setRevealed(prev => new Set(prev).add(revealId))
            accept(next)
          }}
        />
      ) : null}
    </Card>
  )
}

/* ─── ： index.html has ─────── */

let mounted = false

/**
 * React root in up （ not div： CSS use `.page > *` this 
 * ， in 1 will ）。 —— down by 
 * new ， will and React 。
 */
function mount(): void {
  if (mounted) return
  const section = document.querySelector<HTMLElement>(SECTION)
  if (!section) return
  mounted = true
  section.replaceChildren()
  createRoot(section).render(<KeysPage />)
}

// to when （ island-lazy）： not in ， not 。
mountOnPage('keys', SECTION, mount)

declare global {
  interface Window {
    /** Key （ ui/keys-panel.js， and 1 ） */
    wbKeysPanel?: {
      /** to when 1 （app.js:151） */
      load(): Promise<void>
      /** by current（ old can ，） */
      render(): void
      /** current can （table-columns.js by and ） */
      visibleColumns(): (ColumnDecl & { align: Align })[]
    }
  }
}

window.wbKeysPanel = { load, render, visibleColumns }
