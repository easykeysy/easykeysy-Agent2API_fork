import * as React from 'react'
import { mountOnPage } from '../island-lazy'
import { createRoot } from 'react-dom/client'
import {
  Badge, Button, Checkbox, InputGroup, InputGroupAddon, InputGroupInput, NavList, NavItem,
  SegmentedControl, cn, TableCell, TableRow, Card, CardHeader, ResponsiveSheet,
  Table, TableHeader, TableHead, TableBody, Sidebar, Empty,
} from '@ui'
import { TableFooter, useClientPaging } from './table-shell'
import { CapabilityDialog } from './model-capability-dialog'
import { ModelTestDialog, type ModelTestTarget } from './model-test-dialog'
import { ModelBatchDialog } from './models-batch-dialog'
import { CompactRoutesDialog } from './models-compact-dialog'
import { ModelCell } from './models-cells'
import { MappingDialog, CustomModelDialog } from './models-dialogs'
import { DanglingMappings } from './models-dangling'
import * as customSource from './models-custom-source'
import { ALL_CUSTOM, type ManageModel } from './models-custom-source'
import { PROVIDER_ICONS } from './add-provider-pick'
import { providerPresetIconOf } from './add-provider-metadata'
import {
  GROUP_LIMIT, MODEL_STATE_OPTIONS, MODEL_VISIBILITY_OPTIONS, builtinRailCounts,
  closeCapability, closeCustomModel, closeMapping, collapseGroup, customRailItems, directoryReady,
  expandGroup, getSnapshot, isAllCustomView, openAddCustomProvider, openCustomModel, providerLabelOf,
  refreshAll, refreshModels, registerColumnSettings, removeCustomProvider, render, resolveProvider,
  restoreSavedFilters, pruneForeignTableNodes, rowEnabled, rowKeyOf, selectProvider, setSearch, setSort,
  setStateFilter, setVisibilityFilter, setTableEl, visibilityOf, shared, subscribe, syncHead, viewData,
  visibleColumns,
} from './models-panel-state'
import { t } from '../i18n'

/* ─── ───────────────────────────────── */

/**
 * when to delete ×（ 1 ， not will ）。
 * this as by page-gateway.css `.pv-row:hover .pv .n` ； NavItem after is 
 * by `data-slot` span， by `.n` in that then not ， is use 。
 * has delete × （） need need ， not 。
 */
const RAIL_HIDE_COUNT_ON_HOVER = '[.pv-row:hover_&_[data-slot=nav-item-count]]:invisible'

/**
 * ****（ use ； not return null）。
 *
 * is up use 1 
 * （`x0.7 credits`， `attach_rates`），「 not 」 and 「 —」
 * is 1 —— not will 「 and not is 1 」。
 *
 * `null` not by 0 and ：0 is （ / ）， null is 「 up no to 」。
 * in 1 will unknown to that 1 （ models-panel-state MODEL_SORTS）。
 */
function rateOf(credits: unknown): number | null {
  const match = /x\s*([\d.]+)/i.exec(String(credits || ''))
  if (!match) return null
  const value = Number(match[1])
  return Number.isFinite(value) ? value : null
}

/* ─── （ + ）───────────────────────
    1 monogram： = 、 = （ 1 ，
    page-gateway.css .pv-ico）。 by id to 1 （ 
    use can ）， no （ / add ） by id
    to up —— ， not 。：， in 
   （ → ）—— will 1 「/A/」， 。 */

/** （oklch ）。 and after ProviderMeta id 1 。 */
const PROVIDER_HUES: Record<string, number> = {
  workbuddy: 250, 'workbuddy-intl': 295,
  raccoon: 60, catpaw: 85,
  autoclaw: 250, 'autoclaw-intl': 295,
  qoder: 330, 'qoder-intl': 350, 'cline-free': 200, 'cline-pass': 200,
  // （ and Qoder 1 1 ）： to 1 for 
  // can ， not is id to up （ that is ）。
  qwenwork: 300, 'qwenwork-intl': 315,
  codearts: 210, loomy: 155, kuku: 15,
  accio: 320, 'accio-intl': 335, trae: 170, zcode: 285,
}

/** id → ：， by （ 360 can ） */
function providerHue(id: string): number {
  const known = PROVIDER_HUES[id]
  if (known !== undefined) return known
  let hash = 0
  for (let i = 0; i < id.length; i++) hash = (hash * 31 + id.charCodeAt(i)) >>> 0
  return hash % 360
}

/** → ： / CJK ；empty */
function providerGlyph(label: string): string {
  const text = String(label || '').trim()
  if (!text) return '?'
  const ascii = text.match(/[A-Za-z]/)
  return (ascii ? ascii[0] : text[text.length - 1]).toUpperCase()
}

/** that （ style； in page-gateway.css by --av-h ） */
const avatarStyle = (id: string): React.CSSProperties => ({ '--av-h': String(providerHue(id)) } as React.CSSProperties)

/** 「」 ：（ not is 1 ， use not is ） */
const GRID_GLYPH = (
  <svg viewBox='0 0 24 24' aria-hidden='true'>
    <g fill='none' stroke='currentColor' strokeWidth='2' strokeLinecap='round' strokeLinejoin='round'>
      <rect x='3.5' y='3.5' width='7' height='7' rx='1.5' /><rect x='13.5' y='3.5' width='7' height='7' rx='1.5' />
      <rect x='13.5' y='13.5' width='7' height='7' rx='1.5' /><rect x='3.5' y='13.5' width='7' height='7' rx='1.5' />
    </g>
  </svg>
)

function providerIconOf(id: string): string {
  if (customSource.isCustom(id)) return providerPresetIconOf(id)
  return PROVIDER_ICONS[id === 'workbuddy-intl' ? 'workbuddy' : id] || ''
}

/** / use （glyph ReactNode when ，「」 ）。
     has （ / ） use <img>（.img ， in ），
    otherwise monogram；「」（ / ） not is 1 ，
     .mute in ， not and 。 */
function providerAvatar(id: string, label: string, glyph?: React.ReactNode): React.ReactNode {
  const icon = glyph ? '' : providerIconOf(id)
  const sentinel = id === 'all' || id === ALL_CUSTOM
  return (
    <span className={cn('pv-ico', sentinel && 'mute', icon && 'img')}
      style={icon ? undefined : avatarStyle(id)} aria-hidden='true'>
      {icon ? <img src={icon} alt='' /> : (glyph ?? providerGlyph(label))}
    </span>
  )
}

/** / / can / （ and old for ） */
function matches(model: ManageModel, keyword: string, provider: string, stateFilter: string, visibilityFilter: string): boolean {
  if (stateFilter === 'enabled' && !rowEnabled(model)) return false
  if (stateFilter === 'disabled' && rowEnabled(model)) return false
  if (stateFilter === 'mapped' && !(model.aliases || []).length) return false
  // can ：`visible` **** `undeclared` —— has has ， its 
  // no has this 。「 no has 」 not etc 「unavailable」， hidden will use as 
  // that （ and 「 no not 」 ）。
  const visibility = visibilityOf(model)
  if (visibilityFilter === 'visible' && visibility === 'hidden') return false
  if (visibilityFilter === 'hidden' && visibility !== 'hidden') return false
  if (visibilityFilter === 'undeclared' && visibility !== 'undeclared') return false
  if (provider !== 'all' && provider !== ALL_CUSTOM && (model.provider || '') !== provider) return false
  if (!keyword) return true
  const hay = [model.id, model.name, ...(model.aliases || [])].join(' ').toLowerCase()
  return hay.includes(keyword)
}

function ModelsPage() {
  const state = React.useSyncExternalStore(subscribe, getSnapshot)
  const provider = resolveProvider(state)
  /** current is not is （ or 「 · 」 —— / 1 ） */
  const custom = customSource.isCustom(provider) || provider === ALL_CUSTOM
  /**
   * 「」 。 this 1 ** not **（ and context not ）：
   * use 、 no has from React use （ is that by ），
   * in 。 as null = 。
   */
  const [testTarget, setTestTarget] = React.useState<ModelTestTarget | null>(null)
  /** （ if after by by ） */
  const [batchOpen, setBatchOpen] = React.useState(false)
  const [compactOpen, setCompactOpen] = React.useState(false)
  /**
   * （`rowKeyOf` = provider:id， not ）。
   * ** not **： has ， no has from React use ——
   * （ and 「」 target 1 ）。
   * after in can can already not in （ / up new ）：
   * 1 by 「current」（`selectedModels`），。
   */
  const [selection, setSelection] = React.useState<ReadonlySet<string>>(new Set())
  /** （ css/mobile.css .mm-filter-*） */
  const [filterOpen, setFilterOpen] = React.useState(false)
  /** in 1 already delete （directory no has this ， not is 「 no 」）；
      「 · 」 is 、directory no has ， not 「 already delete」 */
  const customMissing = custom && !isAllCustomView() && directoryReady() && !customSource.record(provider)

  /**
   * 1 use ：set（ and by ）→ → → by need 。
   * app.js showPage() in before ： when if is ， that use not to 
   * wbModelsPanel， this 1 new 。
   */
  React.useEffect(() => {
    registerColumnSettings()
    syncHead()
    restoreSavedFilters()
    if (shared().wbApp?.currentPage === 'gateway') {
      // providers.js in after ：refreshAll when can can no ， that directory new will 。
      // after 1 directory and —— directoryrequest and and ， not will 
      void refreshAll()
        .then(() => shared().wbProviders?.refreshCustom?.())
        .then(() => { render() })
    }
    // in when 1 （ up ， not need need ）
  }, [])

  /**
   * （ #195）： after not React 
   *（ table tbody / tr、 and tbody#models ）。
   * and by `pruneForeignTableNodes` 。
   */
  React.useEffect(() => {
    pruneForeignTableNodes()
  })

  const view = viewData()
  const all = Array.isArray(view.models) ? view.models : []
  const providerModels = all.filter(model => provider === 'all' || provider === ALL_CUSTOM || (model.provider || '') === provider)
  const keyword = state.search.trim().toLowerCase()
  const shown = all.filter(model => matches(model, keyword, provider, state.stateFilter, state.visibilityFilter))
  /** （'default' / 'rate-asc' / 'rate-desc'）；， rows() that 1 */
  const sort = state.sort
  const columns = visibleColumns()
  const columnCount = columns.length
  /**
   * and statistics（2026-10 ）： = current （「」/ /
   * ），statistics and 1 —— already enabled by `rowEnabled`（ has ），
   * has by `model.aliases`（ and 「 has 」 1 ）。
   */
  const titleLabel = provider === 'all'
    ? t('全部')
    : (provider === ALL_CUSTOM
      ? t('自定义 · 全部')
      : (custom
        ? String(customSource.record(provider)?.name || provider)
        : (builtinRailCounts().get(provider)?.label || providerLabelOf(provider))))
  const enabledCount = providerModels.filter(rowEnabled).length
  const mappedCount = providerModels.filter(model => (model.aliases || []).length > 0).length
  /**
   * client（ use ）： 1 can can up ，。
   *
   * default 50 /。 down ****（「 its N 」）： already 
   * ， and when will （「Qoder 120 」， up 
   * has 8 ）。「」 this 1 when as ， 1 not 。
   */
  const paging = useClientPaging(shown.length, 'models')

  /* ─── （ 1 + + 「」 by ）────────── */
  /** current in （， selection ） */
  const selectedModels = all.filter(model => selection.has(rowKeyOf(model)))
  const togglePick = (key: string, next: boolean): void => {
    const nextSet = new Set(selection)
    if (next) nextSet.add(key)
    else nextSet.delete(key)
    setSelection(nextSet)
  }
  /** = current（ and 「current」） */
  const visibleKeys = shown.map(rowKeyOf)
  const allVisiblePicked = visibleKeys.length > 0 && visibleKeys.every(key => selection.has(key))
  const someVisiblePicked = visibleKeys.some(key => selection.has(key))
  /** that no has （：set need need ）， has */
  const allPickNode = (
    <Checkbox checked={allVisiblePicked} indeterminate={!allVisiblePicked && someVisiblePicked}
      disabled={!visibleKeys.length} aria-label={t('全选当前筛选结果')}
      title={t('全选当前筛选出的模型（跨分页）；再点取消')}
      onCheckedChange={next => {
        const nextSet = new Set(selection)
        for (const key of visibleKeys) {
          if (next === true) nextSet.add(key)
          else nextSet.delete(key)
        }
        setSelection(nextSet)
      }} />
  )

  /** ：（ + ）+ （ + new ）。
      （.rail-scroll），「＋ new 」 in （.rail-foot） not ——
       is not is ， not this 。 */
  function rail() {
    const counts = builtinRailCounts()
    const customs = customSource.list()
    const customCounts = customRailItems()
    const total = [...counts.values()].reduce((sum, item) => sum + item.n, 0)
    const allLabel = t('全部（{n} 家）', { n: counts.size })
    // 「」 = and ；「N 」 by has ，
    // empty（ no to ） not 
    const customTotal = [...customCounts.values()].reduce((sum, item) => sum + item.n, 0)
    const customAllLabel = t('全部（{n} 家）', { n: [...customCounts.values()].filter(item => item.n > 0).length })
    return (
      <Sidebar className='prov-rail' id='prov-rail' aria-label={t('按提供商选择')}>
        <NavList className='rail-scroll'>
          <div className='rail-label'>{t('内置提供商')}</div>
          {/* is not is by ， NavItem（ in / / / in ，
               and .pv is 1 ）。（icon=） by to 1 monogram，
               providerAvatar 。shadow-none is as ui/css/components.css
               use `button { box-shadow: var(--shadow-1) }` —— not this has ，
              .pv / .nav-item is */}
          <NavItem active={provider === 'all'} data-provider='all' title={allLabel} count={total}
            icon={providerAvatar('all', '', GRID_GLYPH)}
            className='shadow-none' onClick={() => selectProvider('all')}>{allLabel}</NavItem>
          {[...counts].map(([key, entry]) => (
            <NavItem key={key} active={provider === key} data-provider={key}
              title={entry.n ? entry.label : t('{provider}：模型清单为空，等待获取', { provider: entry.label })}
              count={entry.n} icon={providerAvatar(key, entry.label)} className='shadow-none'
              trailing={entry.n === 0 ? <Badge variant='outline'>{t('待获取')}</Badge> : undefined}
              onClick={() => selectProvider(key)}>{entry.label}</NavItem>
          ))}
          <div className='rail-label'>{t('自定义提供商')}</div>
          {customs.length ? [
            // 「」： in when all （ not ——
            // and up that 「」 is and ）。：
            // 1 no when has 1 empty，
            ...(customTotal > 0 ? [(
              <NavItem key={ALL_CUSTOM} active={isAllCustomView()} data-provider={ALL_CUSTOM}
                title={customAllLabel} count={customTotal}
                icon={providerAvatar(ALL_CUSTOM, '', GRID_GLYPH)}
                className='shadow-none' onClick={() => selectProvider(ALL_CUSTOM)}>{customAllLabel}</NavItem>
            )] : []),
            ...customs.map(item => {
              const id = String(item.id || '')
              const label = String(item.name || id)
              return (
                // delete by not can NavItem（button button is invalid HTML， will to 、
                // for ）， 1 ，× and 
                <div className='pv-row' key={id}>
                  <NavItem active={provider === id} data-provider={id} title={label}
                    count={customCounts.get(id)?.n ?? 0}
                    icon={providerAvatar(id, label)}
                    className={cn('shadow-none', RAIL_HIDE_COUNT_ON_HOVER)}
                    onClick={() => selectProvider(id)}>{label}</NavItem>
                  <Button size='default' variant='ghost' type='button' className='pv-del' aria-label={t('删除自定义提供商')}
                    title={t('删除这个自定义提供商（连同名下账号）')}
                    onClick={() => void removeCustomProvider(id)}>×</Button>
                </div>
              )
            }),
          ] : <Empty className='rail-empty'>{t('还没有自定义提供商')}</Empty>}
        </NavList>
        <div className='rail-foot'>
          <NavItem variant='add' id='rail-add-custom'
            title={t('新建一个自定义提供商（同时创建它的第一个账号）')}
            onClick={() => openAddCustomProvider()}>{t('＋ 新建自定义提供商')}</NavItem>
        </div>
      </Sidebar>
    )
  }

  function modelRow(model: ManageModel) {
    // .sel：（ hover 1 ），「 in is 」 in after can 
    const picked = selection.has(rowKeyOf(model))
    return (
      <TableRow key={`${model.provider || ''}\u0001${model.id}`} className={cn(picked && 'sel')}>
        <ModelCell model={model} selection={selection} togglePick={togglePick}
          setTestTarget={setTestTarget} columns={columns} />
      </TableRow>
    )
  }

  /**
   * ：empty / none / （「」 by ， = after = by ）。
   * has or 「」 when not ： use in ， will as no has 。
   */
  function body() {
    if (!providerModels.length) {
      // 1 no has ： and directoryget， is ，
      // 「」 is 「 1 no has （ or no to ）」。
      // down 1 this has use ；directory no when not can 「 already delete」 not can 「 no has 」
      const empty = custom
        ? (!directoryReady()
          ? t('加载中…')
          : isAllCustomView()
            ? t('自定义提供商还没有模型：先在左栏选一家，点「添加模型」登记，或「获取上游模型」从上游拉取')
            : customMissing
              ? t('该提供商已不存在（可能已被删除），请刷新列表')
              : t('这家还没有模型：点「添加模型」登记，或「获取上游模型」从上游拉取'))
        : (!state.data
          ? t('加载中…')
          : builtinRailCounts().size
            ? t('模型清单为空：已有可用账号，请点「获取模型」拉取目录')
            : t('暂无模型（请先添加账号）'))
      return <TableRow><TableCell colSpan={columnCount} className='empty'>{empty}</TableCell></TableRow>
    }
    if (!shown.length) {
      return <TableRow><TableCell colSpan={columnCount} className='empty'>{keyword ? t('没有匹配「{keyword}」的模型', { keyword }) : t('没有匹配当前筛选的模型')}</TableCell></TableRow>
    }
    // in 「」： in when already is 1 ， 1 is 
    // in （「」/ 「」）： in when already 
    // is 1 ， 1 is 
    const showGroups = provider === 'all' || isAllCustomView()
    // down ** not **（ paging ）： ，
    // in 1 will not 「 to has 」。「」 when as 。
    const collapsible = showGroups && !keyword && state.stateFilter === 'all' && !paging.paged
    const groups = new Map<string, { label: string; items: ManageModel[] }>()
    for (const model of shown) {
      const key = model.provider || ''
      if (!groups.has(key)) groups.set(key, { label: model.providerLabel || key || t('未知'), items: [] })
      groups.get(key)?.items.push(model)
    }
    const rows: React.ReactNode[] = []
    /** 「」： by ****， and 「」 not */
    let rowIndex = 0
    for (const [key, group] of groups) {
      // ： use to this ，enabled before ；Array#sort ， by after 
      //（ and 「 already enabled / already use 」 1 ， rowEnabled）
      //
      // 「 by 」 in this after ** 1 enabled** 1 ： use ，
      // not will 1 use to before （ that will 「 can use 」「 」）。
      // not in ， to not —— rateOf 。
      group.items.sort((a, b) => {
        const byEnabled = Number(!rowEnabled(a)) - Number(!rowEnabled(b))
        if (byEnabled !== 0 || sort === 'default') return byEnabled
        const left = rateOf(a.credits)
        const right = rateOf(b.credits)
        if (left === null || right === null) {
          if (left === right) return 0
          return left === null ? 1 : -1
        }
        return sort === 'rate-asc' ? left - right : right - left
      })
      const open = state.expanded.has(key) || !collapsible
      const items = open ? group.items : group.items.slice(0, GROUP_LIMIT)
      const rest = group.items.length - items.length
      // in current that 1 （ when is ）
      const from = paging.paged ? Math.max(0, paging.rangeStart - 1 - rowIndex) : 0
      const to = paging.paged ? Math.min(items.length, paging.rangeEnd - rowIndex) : items.length
      const onPage = from < to ? items.slice(from, to) : []
      // ： in this 1 up has 。 up that 1 「（）」—— otherwise will 
      // as this 1 has current this （ up total is **** ， not is this 1 ）
      if (showGroups && onPage.length) {
        rows.push(
          <TableRow className='tr-group' key={`group:${key}`}>
            <TableCell colSpan={columnCount}>
              <span className='prov-tag'>{group.label}</span>{from > 0 ? t('（续）') : ''}{t('{n} 个模型', { n: group.items.length })}
            </TableCell>
          </TableRow>,
        )
      }
      for (const model of onPage) rows.push(modelRow(model))
      rowIndex += items.length
      if (rest > 0) {
        rows.push(
          <TableRow className='tr-more' key={`more:${key}`}>
            <TableCell colSpan={columnCount}>
              <Button variant='ghost' size='sm' onClick={() => expandGroup(key)}>{t('展开其余 {n} 个模型 ▾', { n: rest })}</Button>
            </TableCell>
          </TableRow>,
        )
      } else if (open && collapsible && group.items.length > GROUP_LIMIT) {
        rows.push(
          <TableRow className='tr-more' key={`collapse:${key}`}>
            <TableCell colSpan={columnCount}>
              <Button variant='ghost' size='sm' onClick={() => collapseGroup(key)}>{t('收起 ▴')}</Button>
            </TableCell>
          </TableRow>,
        )
      }
    }
    return rows
  }

  return (
    <>
      <Card render={<section />} className='panel'>
        <div className='mm'>
          {rail()}
          <div className='mm-main'>
            {/* （2026-10 ， prototype/model-redesign.html）：
                 1 「 this is 」—— + statistics + by ；「 this 」
                —— + + 。 is ， by flex-wrap down 。
                「set」 by by wbColSettings.register .head-actions 
                （mount .panel-head .head-actions in this down old in ）。 */}
            <CardHeader className='panel-head'>
              <div className='mm-titlebar'>
                <div className='mm-title'>
                  {provider === 'all' || provider === ALL_CUSTOM
                    ? providerAvatar(provider, '', GRID_GLYPH)
                    : providerAvatar(provider, titleLabel)}
                  <h2>{titleLabel}</h2>
                  <span className='mm-meta'>
                    <b>{providerModels.length}</b>{t(' 个模型 · 已启用 ')}<b>{enabledCount}</b>{t(' · 有映射 ')}<b>{mappedCount}</b>
                  </span>
                </div>
                <div className='head-actions'>
                  {/* 「get」 is this 1 （ ）， use by ；
                      「」 is ，。 and title has 1 （ this ），
                      index.html not */}
                  <Button id='btn-refresh-models'
                    title={customSource.isCustom(provider)
                      ? t('从这一家的上游拉一份模型清单，勾选要哪些再导入（已添加的不会重复导入）')
                      : t('刷新模型管理页里各提供商的远程模型目录，逐家结果列在弹窗里')}
                    onClick={() => refreshModels()}>{t('获取模型')}</Button>
                  <Button id='btn-add-custom-model' variant='outline' size='sm'
                    onClick={() => openCustomModel()}>{t('＋ 添加模型')}</Button>
                  {/* 「」：（CSS ）。 is */}
                  <Button size='default' variant='ghost' type='button' className='mm-filter-open hidden max-[767px]:inline-flex' onClick={() => setFilterOpen(true)}
                    aria-label={t('打开筛选')}>
                    {t('筛选')}
                    {(() => {
                      const n = (state.stateFilter !== 'all' ? 1 : 0)
                        + (state.visibilityFilter !== 'all' ? 1 : 0)
                        + (String(state.search || '').trim() ? 1 : 0)
                      return n > 0 ? <Badge variant='brand' shape='tag' className='mm-filter-badge'>{n}</Badge> : null
                    })()}
                  </Button>
                  <Button id='btn-compact-routes' variant='outline' size='sm'
                    onClick={() => setCompactOpen(true)}>{t('压缩路由')}</Button>
                </div>
              </div>
              <ResponsiveSheet open={filterOpen} onOpenChange={setFilterOpen} title={t('筛选')}
                inlineClassName='mm-filters' bodyClassName='model-filter-body'
                actions={<Button variant='ghost' size='sm' onClick={() => {
                  setStateFilter('all')
                  setVisibilityFilter('all')
                  setSearch('')
                }}>{t('清空')}</Button>}>
                {/* ：、、 in ， as （） */}
                <SegmentedControl options={MODEL_STATE_OPTIONS} value={state.stateFilter}
                  onValueChange={setStateFilter} aria-label={t('按状态筛选')} className='shrink-0' />
                {/* can ： up directory is ，client not 
                    （ already default default）。default「 can 」—— 1 
                     will use can use down 。 to 「」 can 
                     to （， down ）。 */}
                <SegmentedControl options={MODEL_VISIBILITY_OPTIONS} value={state.visibilityFilter}
                  onValueChange={setVisibilityFilter} aria-label={t('按可见性筛选')} className='shrink-0' />
                {/* ：InputGroup + addon （ and input-control.tsx use 1 ）。
                    ** not ** data-island-input： that is （） ，
                     when 1 will 。： 240px； up 
                    （ already to ， 240px will and up down not ）。 */}
                <InputGroup className='w-full min-[768px]:w-[240px] flex-none min-[768px]:flex-none' id='models-search'>
                  <InputGroupInput type='search' placeholder={t('搜索模型 ID / 名称 / 映射名…')}
                    aria-label={t('搜索模型')} autoComplete='off' value={state.search}
                    onChange={event => setSearch(event.currentTarget.value)} />
                  <InputGroupAddon aria-hidden='true'>⌕</InputGroupAddon>
                </InputGroup>
                {/* ： after in （ ， when ——
                     not not ）。 can delete / enabled / use / set etc */}
                {selectedModels.length > 0 ? (
                  <Badge variant='brand' className='batch-chip'>
                    {t('已选 ')}<b>{selectedModels.length}</b>{t(' 个')}
                    <Button id='btn-batch-models' variant='outline' size='xs'
                      title={t('对选中的 {n} 个模型执行批量操作', { n: selectedModels.length })}
                      onClick={() => setBatchOpen(true)}>{t('批量操作')}</Button>
                  </Badge>
                ) : null}
              </ResponsiveSheet>
            </CardHeader>
            <div className='models-table-wrap'>
              {/* set will or down th / col，，
                   not by can ；update， and 。
                   by visibleColumns() 。 */}
              <Table container={false} className='models-table' ref={el => setTableEl(el)}>
                <colgroup>
                  <col className='c-check' data-col='check' />
                  <col className='c-model' data-col='model' />
                  <col className='c-rate' data-col='rate' />
                  <col className='c-visibility' data-col='visibility' />
                  <col className='c-source' data-col='source' />
                  <col className='c-budget' data-col='budget' />
                  <col className='c-caps' data-col='caps' />
                  <col className='c-alias' data-col='alias' />
                  <col className='c-act' data-col='act' />
                </colgroup>
                <TableHeader><TableRow>
                  <TableHead data-col='check'>{allPickNode}</TableHead>
                  <TableHead data-col='model'>{t('上游模型')}</TableHead>
                  <TableHead data-col='rate'
                    sortDirection={sort === 'rate-asc' ? 'asc' : sort === 'rate-desc' ? 'desc' : false}
                    sortLabel={sort === 'rate-asc'
                      ? t('倍率升序，点击切换为降序')
                      : sort === 'rate-desc'
                        ? t('倍率降序，点击切换为升序')
                        : t('按倍率升序排列')}
                    title={t('按提供商分组排序，已启用模型优先；未知倍率排在末尾')}
                    onSortChange={direction => {
                      setSort(`rate-${direction}`)
                      paging.goto(1)
                    }}>{t('倍率')}</TableHead>
                  <TableHead data-col='visibility' title={t('这个模型在官方客户端里是否会被展列；不展示的那些默认已关闭（含默认绑定）')}>{t('可见性')}</TableHead>
                  <TableHead data-col='source'>{t('来源')}</TableHead>
                  {/* ，（）。
                       this is ： when not ， not will 「」 。
                       this is ** 1 **（ this to is not is up ）：
                       / = models.dev ， up = */}
                  <TableHead data-col='budget'>
                    {t('上下文 / 输出')}
                    <span className='th-help' aria-hidden='true'
                      title={t('数字带虚线底 = 由 models.dev 补齐；右上角小点 = 你手动覆盖过；— = 未声明。点任意一格可改。')}>
                      <svg viewBox='0 0 24 24'>
                        <g fill='none' stroke='currentColor' strokeWidth='2' strokeLinecap='round'>
                          <circle cx='12' cy='12' r='9' /><path d='M12 16v-4' /><path d='M12 8h.01' />
                        </g>
                      </svg>
                    </span>
                  </TableHead>
                  <TableHead data-col='caps'>
                    {t('能力')}
                    <span className='th-help' aria-hidden='true'
                      title={t('圆点带内侧描边 = 由 models.dev 补齐；右上角小点 = 你手动覆盖过；虚线圈 = 未声明。悬停某一枚可看逐项出处。')}>
                      <svg viewBox='0 0 24 24'>
                        <g fill='none' stroke='currentColor' strokeWidth='2' strokeLinecap='round'>
                          <circle cx='12' cy='12' r='9' /><path d='M12 16v-4' /><path d='M12 8h.01' />
                        </g>
                      </svg>
                    </span>
                  </TableHead>
                  {/* ，（）。
                       this is ： when not ， not will 「」 */}
                  <TableHead data-col='alias'>
                    {t('模型映射')}
                    <span className='th-help' aria-hidden='true'
                      title={t('每条映射 = 对外名 → 上游模型；原始 ID 与别名有独立开关，默认绑定永远存在（没有映射时是合成的那条）。')}>
                      <svg viewBox='0 0 24 24'>
                        <g fill='none' stroke='currentColor' strokeWidth='2' strokeLinecap='round'>
                          <circle cx='12' cy='12' r='9' /><path d='M12 16v-4' /><path d='M12 8h.01' />
                        </g>
                      </svg>
                    </span>
                  </TableHead>
                  <TableHead className='r' data-col='act'>{t('操作')}</TableHead>
                </TableRow></TableHeader>
                <TableBody id='models'>{body()}</TableBody>
              </Table>
            </div>
            {/* ： up already not in up （ up down / that 1 no ），
                chip none。 in and —— is 「 this has 」 ，
                 not this by / （ models-dangling.tsx file）。 */}
            <DanglingMappings />
            {/* ： / / / by use （table-shell.tsx） 1 。
                「」 that 1 （ that by use need ）； its 
                 ， in is 1 */}
            <TableFooter
              className='models-panel-foot'
              leading={isAllCustomView()
                ? <>{t('这里是')}<b>{t('全部自定义提供商')}</b>{t('的模型合集：每家的清单互相独立，别名也只在自己那家内生效；要改名称 / 协议 / Base URL，到账号页该家账号的「设置」→ 提供商一栏。')}</>
                : (custom
                  ? <>{t('自定义提供商的清单')}<b>{t('只属于这一家')}</b>{t('：这里的模型不会出现在其他家，别名也只在这一家内生效。改名称 / 协议 / Base URL 在账号页该家账号的「设置」→ 提供商一栏；整家不要了，鼠标移到左栏这家上点 × 删除（连同名下账号）。')}</>
                  : undefined)}
              total={shown.length}
              range={paging.paged ? { start: paging.rangeStart, end: paging.rangeEnd } : null}
              page={paging.page}
              pageCount={paging.pageCount}
              size={paging.size}
              onSizeChange={paging.setSize}
              onPageChange={paging.goto}
            />
          </div>
        </div>
      </Card>
      {/* Dialog（Esc / 、、）， by need 、 */}
      {compactOpen ? <CompactRoutesDialog onClose={() => setCompactOpen(false)} /> : null}
      {state.mapping ? <MappingDialog context={state.mapping} onClose={closeMapping} /> : null}
      {state.customModel
        ? <CustomModelDialog initial={state.customModel} onClose={closeCustomModel} />
        : null}
      {state.capability
        ? <CapabilityDialog context={state.capability} onClose={closeCapability} />
        : null}
      {testTarget
        ? <ModelTestDialog target={testTarget} onClose={() => setTestTarget(null)} />
        : null}
      {batchOpen
        ? <ModelBatchDialog models={selectedModels} onClose={() => setBatchOpen(false)}
          onDone={() => { setSelection(new Set()); setBatchOpen(false) }} />
        : null}
    </>
  )
}

/* ─── ： index.html has ─────── */

const PAGE_SELECTOR = '.page[data-page="gateway"]'

let root: ReturnType<typeof createRoot> | null = null

/**
 * React root in `.page[data-page="gateway"]` up （ not div： CSS use 
 * `.page[data-page="gateway"] .panel` this ， in 1 will ）。
 *
 * （ that `.panel`）： down JSX will by new ， will and 
 * React 。createRoot not ， replaceChildren()； 
 * class / data-page by ，。
 */
function mount(): void {
  if (root) return
  const section = document.querySelector<HTMLElement>(PAGE_SELECTOR)
  if (!section) return
  section.replaceChildren()
  root = createRoot(section)
  root.render(<ModelsPage />)
}

// to when （ island-lazy）： not in ， not 。
mountOnPage('gateway', PAGE_SELECTOR, mount)
