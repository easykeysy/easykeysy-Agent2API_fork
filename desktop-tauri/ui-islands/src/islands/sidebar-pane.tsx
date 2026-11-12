/**
 * Agent2API · set → → （React ）
 *
 * use ：** / / **。 OmniProxy 
 * 「set → and → 」（ 1 ）， this ：
 * （）、（ new / / delete）、。
 *
 * ── not in this ─────────────────────────────────────────
 * 、、default、 in `ui/sidebar-layout.js`
 * （， `window.wbSidebarLayout` use ）。Why not ：
 * by `ui/app.js` 、 is React ， 1 1 1 
 * then ，「default + + new 」。file：
 * ① that React ；② ；③ and `wb:sidebar-layout`
 * app.js 。
 *
 * ── in ────────────────────────────────────────────
 * in （`kv` `sidebarLayout` ），`api.writeLayout` after 
 * will to after —— this 1 in `sidebar-layout.js` ，file not need need 。
 * must： and is origin、localStorage ，
 * 「 in WebUI client old 」。localStorage 
 * ， that module header。
 *
 * ── Why not use / HTML5 ────────────────────────
 * and 、set 1 ： in config 1 need ， HTML5
 * will in ** that ** up ， 1 ； in JS ，
 * not will 。 old ： 、 1 in 1 、
 * 。
 *
 * ── Why not in 1 ───────────────────────
 * in 1 ， when —— 1 is ，
 * will （ this 1 is ）。 this 「 + 」。
 *
 * ── and ────────────────────────────────────────
 * file settings-page.tsx DisplayPane use 1 ； not settings-state /
 * settings-model （ kv ， not after config config），
 * not 「」「」 that 。
 */

import * as React from 'react'
import { Badge, Button, Input, Popover, PopoverContent, PopoverTrigger, Switch, cn , Card, CardHeader, CardContent } from '@ui'
import { t } from '../i18n'
import { Empty } from '@ui'

/* ─── from ─────────────────────── */

/**
 * `ui/sidebar-layout.js` up 。 this 1 ， not is import that .js：
 * is （ to app.js use ）， no has 。must 1 ——
 * that when this ，TypeScript will in use ， not will 。
 */
type MenuDef = { key: string; label: string; icon: string; fixed?: boolean }
type LayoutGroup = { id: string; label: string; custom: boolean; hidden: boolean; items: string[] }
type Layout = { groups: LayoutGroup[]; hidden: string[] }
type RenderGroup = { id: string; label: string; items: MenuDef[] }

type LayoutApi = {
  STORAGE_KEY: string
  MENUS: MenuDef[]
  MENU_KEYS: string[]
  defaultLayout(): Layout
  normalizeLayout(raw: unknown): Layout
  readLayout(): Layout
  writeLayout(layout: Layout): void
  /** and for 1 （ and ；unavailable when is none empty） */
  syncFromServer(): Promise<boolean>
  cloneLayout(layout: Layout): Layout
  menuOf(key: string): MenuDef | undefined
  /** ：， use return */
  groupLabel(label: string): string
  isFixedMenu(key: string): boolean
  locate(layout: Layout, key: string): { gi: number; index: number } | null
  groupHasFixed(group: LayoutGroup): boolean
  visibleCount(group: LayoutGroup, hidden: string[]): number
  moveItem(layout: Layout, key: string, toGi: number, toIndex: number): Layout | null
  moveGroup(layout: Layout, gid: string, toIndex: number): Layout | null
  setItemHidden(layout: Layout, key: string, hidden: boolean): Layout | null
  setGroupHidden(layout: Layout, gid: string, hidden: boolean): Layout | null
  renameGroup(layout: Layout, gid: string, label: string): Layout | null
  addGroup(layout: Layout, label: string): Layout | null
  removeGroup(layout: Layout, gid: string): Layout | null
  renderGroups(layout: Layout): RenderGroup[]
  firstVisibleKey(layout: Layout): string | null
  isVisible(layout: Layout, key: string): boolean
}

function layoutApi(): LayoutApi | null {
  return (window as unknown as { wbSidebarLayout?: LayoutApi }).wbSidebarLayout ?? null
}

/** （icons.js SVG ）： and set 1 */
function iconHtml(name: string, size: number): string {
  return (window as unknown as { wbIcons?: { icon?: (n: string, s?: number) => string } })
    .wbIcons?.icon?.(name, size) || ''
}

/** toast 1 （ wbApp， not in —— and settings-model ） */
function toast(message: string, kind?: 'err' | 'ok'): void {
  (window as unknown as { wbApp?: { toast?: (m: string, k?: string) => void } })
    .wbApp?.toast?.(message, kind)
}

/* ─── ─────────────────────────────────── */

/**
 * use （icons.js no has ， after can ）。
 * and icons.js that 1 （1.8 、），
 * and in 1 not will is 。
 */
const PLUS_ICON = '<svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" '
  + 'stroke-width="1.8" stroke-linecap="round" aria-hidden="true"><path d="M12 5v14M5 12h14"/></svg>'
const FOLDER_PLUS_ICON = '<svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" '
  + 'stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">'
  + '<path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7Z"/>'
  + '<path d="M12 11v6M9 14h6"/></svg>'

const GRIP_ICON = '<svg viewBox="0 0 24 24" width="15" height="15" fill="currentColor" aria-hidden="true">'
  + '<circle cx="9" cy="6" r="1.4"/><circle cx="15" cy="6" r="1.4"/>'
  + '<circle cx="9" cy="12" r="1.4"/><circle cx="15" cy="12" r="1.4"/>'
  + '<circle cx="9" cy="18" r="1.4"/><circle cx="15" cy="18" r="1.4"/></svg>'

/** 1 / 1 ： / / / */
const ITEM_GRID = 'grid grid-cols-[18px_17px_minmax(0,1fr)_auto] items-center gap-2'

/* ─── ───────────────────────────────── */

/**
 * 。
 *
 * ：`layout` is 1 ，all `apply(next)` ——
 * that 1 「empty = none」、、、 new 。
 * when （ in 、 will ） use useState / useRef，
 * not layout， 「 」。
 */
export function SidebarPane() {
  const api = layoutApi()

  // no （index.html sidebar-layout.js / failure）：
  // as 1 。 not 、 not —— set its available。
  if (!api) {
    return (
      <Card render={<section />} className='panel'>
        <CardHeader className='panel-head'><h2>{t('侧边栏菜单')}</h2></CardHeader>
        <CardContent className='panel-body'>
          <div className='settings-state'>{t('侧边栏布局模块未加载（ui/sidebar-layout.js 应在 app.js 之前引入）。')}</div>
        </CardContent>
      </Card>
    )
  }

  // up already ： api as prop to ， not to empty
  return <SidebarPaneBody api={api} />
}

/**
 * 。 as 1 ： `api` in up empty ——
 * / this 1 need use ， `api?.` will 
 * 「 no 」none。
 */
function SidebarPaneBody({ api }: { api: LayoutApi }) {
  const [layout, setLayout] = React.useState<Layout>(() => api.readLayout())
  /** in id（null = no has in ） */
  const [renaming, setRenaming] = React.useState<string | null>(null)
  /** ： in renaming has */
  const [renameDraft, setRenameDraft] = React.useState('')
  /** new is */
  const [adding, setAdding] = React.useState(false)
  const [addDraft, setAddDraft] = React.useState('')
  /** for （used for， not and ） */
  const [dragId, setDragId] = React.useState<string | null>(null)
  const dragRef = React.useRef<{ kind: 'group' | 'item'; id: string } | null>(null)
  const dragSessionRef = React.useRef<{ teardown: () => void } | null>(null)
  /** new ： in pointermove not to after state， can from this */
  const layoutRef = React.useRef<Layout>(layout)

  /**
   * 1 。
   *
   * ：
   * · return null = no （ / 1 ）——
   * not 、 not 、 not ，none will ；
   * · `api.writeLayout` ：（）+ to （，
   * as ）。 after `wb:sidebar-layout`，app.js 
   * —— not etc that ，must when response；
   * · state use ** after that **（api.readLayout），
   * not is return —— when will / ，must 1 。
   */
  const apply = React.useCallback((next: Layout | null): boolean => {
    if (!next) return false
    api.writeLayout(next)
    const normalized = api.readLayout()
    // ref update： in down 1 pointermove can can React ，
    // to old will by down 1 ， 
    layoutRef.current = normalized
    setLayout(normalized)
    window.dispatchEvent(new CustomEvent('wb:sidebar-layout'))
    return true
  }, [api])

  // from new ， ：
  // · `storage`： 1 up 1 / （，localStorage ）；
  // · `wb:sidebar-layout`： ， and `syncFromServer` from 
  // 1 （ ↔ ， origin） 。
  React.useEffect(() => {
    const sync = () => {
      const next = api.readLayout()
      layoutRef.current = next
      setLayout(next)
    }
    window.addEventListener('storage', sync)
    window.addEventListener('wb:sidebar-layout', sync)
    return () => {
      window.removeEventListener('storage', sync)
      window.removeEventListener('wb:sidebar-layout', sync)
    }
  }, [api])

  // when no will （ and table-col-settings 1 ）
  React.useEffect(() => () => { dragSessionRef.current?.teardown() }, [])

  const menus = api.MENUS
  const visibleMenus = menus.filter(menu => !layout.hidden.includes(menu.key)).length
  const hiddenGroups = layout.groups.filter(group => group.hidden).length

  /* ── ── */

  const toggleItem = (key: string, visible: boolean) => {
    apply(api.setItemHidden(layout, key, !visible))
  }

  const toggleGroup = (gid: string) => {
    const group = layout.groups.find(item => item.id === gid)
    if (!group) return
    if (!group.hidden && api.groupHasFixed(group)) {
      // not by ： by down none， use will as 。
      // ， 「Why not 、 this 」。
      toast(t('「设置」是固定项，含它的分组不能整组隐藏；可以单独隐藏组内其它菜单'))
      return
    }
    apply(api.setGroupHidden(layout, gid, !group.hidden))
  }

  const commitRename = (gid: string) => {
    const next = api.renameGroup(layout, gid, renameDraft)
    setRenaming(null)
    if (next) apply(next)
  }

  const commitAdd = () => {
    const next = api.addGroup(layout, addDraft)
    if (!next) { toast(t('请填一个分组名'), 'err'); return }
    setAdding(false)
    setAddDraft('')
    apply(next)
    toast(t('已新建分组「{name}」，点组头的 ＋ 把菜单移进来', { name: addDraft.trim() }))
  }

  const removeGroup = (gid: string) => {
    const group = layout.groups.find(item => item.id === gid)
    if (!group) return
    const target = layout.groups[0].id === gid ? layout.groups[1] : layout.groups[0]
    const ask = (window as unknown as {
      wbConfirm?: { ask?: (options: Record<string, unknown>) => Promise<boolean> }
    }).wbConfirm?.ask
    const message = group.items.length
      ? t('删除分组「{name}」？组内 {n} 个菜单会移到「{target}」，不会丢。', {
          name: group.label, n: group.items.length, target: target?.label ?? '',
        })
      : t('删除空分组「{name}」？', { name: group.label })
    const finish = (ok: boolean) => {
      if (!ok) return
      if (apply(api.removeGroup(layout, gid))) toast(t('已删除分组「{name}」', { name: group.label }))
    }
    // no has （ when ） when ： not ， is can 
    if (ask) void ask({ title: t('删除分组'), text: message, okText: t('删除'), bodyClass: '' }).then(finish)
    else finish(true)
  }

  const resetAll = () => {
    const ask = (window as unknown as {
      wbConfirm?: { ask?: (options: Record<string, unknown>) => Promise<boolean> }
    }).wbConfirm?.ask
    const finish = (ok: boolean) => {
      if (!ok) return
      apply(api.defaultLayout())
      toast(t('已恢复默认分组：概览 / 转发链路 / 诊断 / 系统'))
    }
    if (ask) {
      void ask({
        title: t('恢复默认侧边栏'),
        text: t('会恢复默认的分组、顺序与显隐设置。你自建的分组会被删除（组内菜单不会丢）。'),
        okText: t('恢复默认'),
        bodyClass: '',
      }).then(finish)
    } else finish(true)
  }

  /* ── ── */

  /**
   * 。 use not is HTML5 （ by file）。
   *
   * ：（ in 1 up = to before / after ），otherwise to 
   * （ in or empty = to this ）。 use `elementFromPoint` not is 
   * target —— in down is ， target not can 。
   */
  function startDrag(kind: 'group' | 'item', id: string, event: React.PointerEvent<HTMLElement>) {
    if (event.button !== 0) return
    event.preventDefault()
    dragSessionRef.current?.teardown()
    dragRef.current = { kind, id }
    setDragId(id)
    document.body.classList.add('sc-dragging')
    // to up 「 in 」 ， not etc React 。
    //
    // Whymust： use elementFromPoint， down ** is 
    // ** —— need is not （.sc-item.is-dragging / .sc-group.is-dragging
    // is pointer-events:none），elementFromPoint return， not ，
    // 「 can 、 no 」。setDragId is ，
    // 1 pointermove can can in before ， this DOM。
    const source = event.currentTarget.closest<HTMLElement>(
      kind === 'group' ? '[data-sc-group]' : '[data-sc-item]',
    )
    source?.classList.add('is-dragging')
    // new ref： in will 1 layout， is 
    // that 1 old —— old down ， down 。
    layoutRef.current = layout

    const onMove = (moveEvent: PointerEvent) => {
      const current = dragRef.current
      if (!current) return
      const el = document.elementFromPoint(moveEvent.clientX, moveEvent.clientY) as HTMLElement | null
      if (!el) return
      const live = layoutRef.current

      if (current.kind === 'group') {
        const card = el.closest<HTMLElement>('[data-sc-group]')
        if (!card) return
        const gid = card.dataset.scGroup || ''
        if (!gid || gid === current.id) return
        const targetIndex = live.groups.findIndex(group => group.id === gid)
        if (targetIndex < 0) return
        const rect = card.getBoundingClientRect()
        const after = moveEvent.clientY > rect.top + rect.height / 2
        apply(api.moveGroup(live, current.id, after ? targetIndex + 1 : targetIndex))
        return
      }

      const row = el.closest<HTMLElement>('[data-sc-item]')
      if (row) {
        const targetKey = row.dataset.scItem || ''
        const gid = row.dataset.scGid || ''
        if (!targetKey || targetKey === current.id) return
        const gi = live.groups.findIndex(group => group.id === gid)
        if (gi < 0) return
        const index = live.groups[gi].items.indexOf(targetKey)
        if (index < 0) return
        const rect = row.getBoundingClientRect()
        const after = moveEvent.clientY > rect.top + rect.height / 2
        apply(api.moveItem(live, current.id, gi, after ? index + 1 : index))
        return
      }

      // not in up ： to to this 
      const card = el.closest<HTMLElement>('[data-sc-group]')
      if (!card) return
      const gi = live.groups.findIndex(group => group.id === (card.dataset.scGroup || ''))
      if (gi < 0) return
      apply(api.moveItem(live, current.id, gi, live.groups[gi].items.length))
    }

    const teardown = () => {
      window.removeEventListener('pointermove', onMove)
      window.removeEventListener('pointerup', onUp)
      window.removeEventListener('pointercancel', onUp)
      document.body.classList.remove('sc-dragging')
      // can can already ， by current that 1 （ not to none）
      document.querySelectorAll('.is-dragging').forEach(node => node.classList.remove('is-dragging'))
      dragRef.current = null
      dragSessionRef.current = null
      setDragId(null)
    }
    const onUp = () => { teardown() }

    dragSessionRef.current = { teardown }
    window.addEventListener('pointermove', onMove)
    window.addEventListener('pointerup', onUp)
    window.addEventListener('pointercancel', onUp)
  }

  /**
   * ： after Alt+↑/↓ 1 ， when to 。
   * not this 1 ，「」 for can use is unavailable ——
   * is this 。
   */
  function moveByKeyboard(kind: 'group' | 'item', id: string, up: boolean) {
    if (kind === 'group') {
      const index = layout.groups.findIndex(group => group.id === id)
      if (index < 0) return
      apply(api.moveGroup(layout, id, up ? index : index + 2))
      return
    }
    const from = api.locate(layout, id)
    if (!from) return
    let toGi = from.gi
    let toIndex = from.index + (up ? 0 : 1)
    const group = layout.groups[from.gi]
    if (up && from.index === 0 && from.gi > 0) {
      toGi = from.gi - 1
      toIndex = layout.groups[toGi].items.length
    } else if (!up && from.index === group.items.length - 1 && from.gi < layout.groups.length - 1) {
      toGi = from.gi + 1
      toIndex = 0
    }
    apply(api.moveItem(layout, id, toGi, toIndex))
  }

  /* ── ── */

  return (
    <>
      <Card render={<section />} className='panel'>
        <CardHeader className='panel-head'>
          <h2>{t('侧边栏菜单')}</h2>
          <span
            className='tip-q'
            data-tip={t('控制主侧边栏显示哪些菜单、按什么顺序、怎么分组。隐藏只影响导航显示，不会撤销页面访问权限；被隐藏的页面仍可通过地址直接打开。')}
          />
          <div className='head-actions'>
            <Button variant='outline' size='sm' onClick={resetAll}>{t('恢复默认')}</Button>
          </div>
        </CardHeader>

        <CardContent className='panel-body'>
          <div className='settings-state'>
            {t('拖动左侧 ⠿ 调整顺序（菜单可以跨组拖）；拨开关控制显隐；点分组名可重命名；组头的 ＋ 把别的菜单移进来；眼睛按钮隐藏整组。改动即时生效并记住，左侧侧边栏就是结果。键盘也能排序：把手聚焦后按 Alt+↑ / ↓。')}
          </div>

          <div className='sc-groups'>
            {layout.groups.map(group => {
              const shown = api.visibleCount(group, layout.hidden)
              const locked = api.groupHasFixed(group)
              const isRenaming = renaming === group.id
              return (
                <div
                  key={group.id}
                  data-sc-group={group.id}
                  className={cn('sc-group', group.hidden && 'is-hidden', dragId === group.id && 'is-dragging')}
                >
                  <div className='sc-group-head'>
                    <span
                      className='sc-grip'
                      role='button'
                      tabIndex={0}
                      title={t('拖动调整分组顺序（Alt+↑ / ↓ 也可）')}
                      aria-label={t('拖动分组 {name}', { name: group.label })}
                      onPointerDown={event => startDrag('group', group.id, event)}
                      onKeyDown={event => {
                        if (!event.altKey) return
                        if (event.key === 'ArrowUp') { event.preventDefault(); moveByKeyboard('group', group.id, true) }
                        if (event.key === 'ArrowDown') { event.preventDefault(); moveByKeyboard('group', group.id, false) }
                      }}
                      dangerouslySetInnerHTML={{ __html: GRIP_ICON }}
                    />

                    {isRenaming ? (
                      <Input
                        className='sc-rename'
                        value={renameDraft}
                        maxLength={12}
                        autoFocus
                        aria-label={t('分组名')}
                        onChange={event => setRenameDraft(event.target.value)}
                        onBlur={() => commitRename(group.id)}
                        onKeyDown={event => {
                          if (event.key === 'Enter') { event.preventDefault(); commitRename(group.id) }
                          if (event.key === 'Escape') { event.preventDefault(); setRenaming(null) }
                        }}
                      />
                    ) : (
                      <Button size='default' variant='ghost'
                        type='button'
                        className='sc-group-title'
                        title={t('点击重命名')}
                        onClick={() => { setRenaming(group.id); setRenameDraft(group.label) }}
                      >
                        {api.groupLabel(group.label)}
                      </Button>
                    )}

                    <span className='sc-group-meta'>{t('{n} 项 · {shown} 显示', { n: group.items.length, shown })}</span>

                    <div className='sc-group-actions'>
                      <GroupPicker
                        layout={layout}
                        api={api}
                        group={group}
                        onPick={key => apply(api.moveItem(layout, key, layout.groups.findIndex(item => item.id === group.id), group.items.length))}
                      />
                      <Button size='default' variant='ghost'
                        type='button'
                        className='sc-icon-btn'
                        title={locked
                          ? t('含固定项「设置」，不可整组隐藏（可以单独隐藏组内其它菜单）')
                          : (group.hidden ? t('显示这一组') : t('隐藏这一组'))}
                        aria-label={group.hidden ? t('显示分组 {name}', { name: group.label }) : t('隐藏分组 {name}', { name: group.label })}
                        aria-pressed={!group.hidden}
                        onClick={() => toggleGroup(group.id)}
                        dangerouslySetInnerHTML={{ __html: iconHtml(group.hidden ? 'eyeOff' : 'eye', 15) }}
                      />
                      <Button size='default' variant='ghost'
                        type='button'
                        className='sc-text-btn'
                        disabled={layout.groups.length <= 1}
                        title={layout.groups.length <= 1 ? t('至少要保留一个分组') : t('删除这个分组（组内菜单移到第一组）')}
                        onClick={() => removeGroup(group.id)}
                      >
                        {t('删除')}
                      </Button>
                    </div>
                  </div>

                  <div className='sc-items'>
                    {group.items.length === 0 ? (
                      <Empty className='sc-empty'>{t('空分组 · 点组头的 ＋ 把菜单移进来')}</Empty>
                    ) : group.items.map(key => {
                      const menu = api.menuOf(key)
                      if (!menu) return null
                      const isHidden = layout.hidden.includes(key)
                      return (
                        <div
                          key={key}
                          data-sc-item={key}
                          data-sc-gid={group.id}
                          className={cn('sc-item', ITEM_GRID, isHidden && 'is-hidden', dragId === key && 'is-dragging')}
                        >
                          <span
                            className='sc-grip'
                            role='button'
                            tabIndex={0}
                            title={t('拖动调整顺序，可跨组（Alt+↑ / ↓ 也可）')}
                            aria-label={t('拖动菜单 {name}', { name: menu.label })}
                            onPointerDown={event => startDrag('item', key, event)}
                            onKeyDown={event => {
                              if (!event.altKey) return
                              if (event.key === 'ArrowUp') { event.preventDefault(); moveByKeyboard('item', key, true) }
                              if (event.key === 'ArrowDown') { event.preventDefault(); moveByKeyboard('item', key, false) }
                            }}
                            dangerouslySetInnerHTML={{ __html: GRIP_ICON }}
                          />
                          <span className='sc-item-ico' dangerouslySetInnerHTML={{ __html: iconHtml(menu.icon, 15) }} />
                          <span className='sc-item-label' title={menu.label}>{menu.label}</span>
                          {menu.fixed ? (
                            <Badge variant='outline' shape='tag' title={t('设置不可隐藏（隐藏了只能清缓存找回来）')}>
                              {t('固定')}
                            </Badge>
                          ) : (
                            <Switch
                              checked={!isHidden}
                              onCheckedChange={next => toggleItem(key, next)}
                              aria-label={isHidden ? t('显示 {name}', { name: menu.label }) : t('隐藏 {name}', { name: menu.label })}
                              title={isHidden ? t('这一项已隐藏') : t('这一项正在显示')}
                            />
                          )}
                        </div>
                      )
                    })}
                  </div>
                </div>
              )
            })}
          </div>

          <div className='sc-add'>
            {adding ? (
              <div className='sc-add-form'>
                <Input
                  value={addDraft}
                  maxLength={12}
                  autoFocus
                  placeholder={t('分组名，例如「常用」')}
                  aria-label={t('新分组名')}
                  onChange={event => setAddDraft(event.target.value)}
                  onKeyDown={event => {
                    if (event.key === 'Enter') { event.preventDefault(); commitAdd() }
                    if (event.key === 'Escape') { event.preventDefault(); setAdding(false); setAddDraft('') }
                  }}
                />
                <Button size='sm' onClick={commitAdd}>{t('添加')}</Button>
                <Button variant='ghost' size='sm' onClick={() => { setAdding(false); setAddDraft('') }}>{t('取消')}</Button>
              </div>
            ) : (
              <Button variant='dashed' className='w-full' onClick={() => setAdding(true)}>
                <span dangerouslySetInnerHTML={{ __html: FOLDER_PLUS_ICON }} />
                {t('新建分组')}
              </Button>
            )}
          </div>

          <div className='settings-state'>
            {t('{total} 个菜单 · 显示 {shown} · 隐藏 {hidden} · {groups} 个分组', {
              total: menus.length,
              shown: visibleMenus,
              hidden: menus.length - visibleMenus,
              groups: layout.groups.length,
            })}
            {hiddenGroups ? t('（{n} 个整组隐藏）', { n: hiddenGroups }) : ''}
          </div>
        </CardContent>
      </Card>
    </>
  )
}

/* ─── 「＋」： this 1 ─────────── */

/**
 * 1 Popover，「current not in this 1 」 ， and in in 1 。
 *
 * Why in not is 1 「 to 」：13 1 will ，
 * 「 this 」 is 。 can to 1 ，
 * need need ， when and not use —— 。
 */
function GroupPicker({ layout, api, group, onPick }: {
  layout: Layout
  api: LayoutApi
  group: LayoutGroup
  onPick: (key: string) => void
}) {
  const [open, setOpen] = React.useState(false)
  const candidates = api.MENU_KEYS.filter(key => !group.items.includes(key))

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger
        render={
          <Button size='default' variant='ghost'
            type='button'
            className='sc-icon-btn'
            title={t('把菜单移入「{name}」', { name: group.label })}
            aria-label={t('把菜单移入 {name}', { name: group.label })}
          />
        }
      >
        <span dangerouslySetInnerHTML={{ __html: PLUS_ICON }} />
      </PopoverTrigger>
      <PopoverContent align='end' side='bottom' className='w-[228px] p-1'>
        <div className='px-2 pt-1.5 pb-1.5 text-[10.5px] font-semibold tracking-[.04em] text-muted-foreground'>
          {t('移入「{name}」', { name: api.groupLabel(group.label) })}
        </div>
        {candidates.length === 0 ? (
          <div className='px-2 py-2 text-center text-[11px] text-muted-foreground'>
            {t('所有菜单都已在这一组')}
          </div>
        ) : (
          <div className='max-h-[280px] overflow-y-auto'>
            {candidates.map(key => {
              const menu = api.menuOf(key)
              if (!menu) return null
              const from = api.locate(layout, key)
              return (
                <Button size='default' variant='ghost'
                  key={key}
                  type='button'
                  className='flex h-7 w-full items-center gap-2 rounded-xs px-2 text-left text-[12.5px] text-foreground hover:bg-nav-hover'
                  onClick={() => { setOpen(false); onPick(key) }}
                >
                  <span className='grid size-[15px] flex-none place-items-center text-muted-foreground'
                    dangerouslySetInnerHTML={{ __html: iconHtml(menu.icon, 14) }} />
                  <span className='min-w-0 flex-1 truncate'>{menu.label}</span>
                  <span className='flex-none text-[10.5px] text-muted-foreground'>
                    {from ? api.groupLabel(layout.groups[from.gi].label) : ''}
                  </span>
                </Button>
              )
            })}
          </div>
        )}
      </PopoverContent>
    </Popover>
  )
}
