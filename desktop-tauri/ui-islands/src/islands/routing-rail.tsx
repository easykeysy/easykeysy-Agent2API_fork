import * as React from 'react'
import { Button, NavItem, NavList, Sidebar, SidebarContent, SidebarFooter, SidebarGroupLabel, SidebarHeader, StatusDot } from '@ui'
import { t } from '../i18n'
import { groupDirty, type RouteGroupEditor } from './routing-group-state'
import { draftDirty, type RoutingEditorState } from './routing-editor-state'
import type { RoutingWorkspace } from './routing-types'
import { RouteIcon } from './routing-ui'

/** 特殊请求路由的三个入口（与账号路由并列，但规则独立）。 */
export type SpecialKey = 'compact' | 'vision' | 'title'

/**
 * 三项只留「图标 + 名字」。
 *
 * 以前每项下面还挂一句说明（「为压缩请求指定模型」「辅助识图 · 接管」…），
 * 那些话在右侧面板里已经用一句话讲得更清楚，放在这里只会让左栏变成两行一条、
 * 一屏看不下几条。左栏的职责是「切到哪一块」，不是解释每一块在干什么。
 */
export const SPECIAL_RAIL: Array<{ key: SpecialKey; label: string; icon: string }> = [
  { key: 'compact', label: t('Context compaction'), icon: 'database' },
  { key: 'vision', label: t('Image compatibility'), icon: 'eye' },
  { key: 'title', label: t('Title generation'), icon: 'spark' },
]

export function RoutingRail({ editor, groups, workspace, onModel, onGroup, onAdd, special, onSpecial }: {
  editor: RoutingEditorState; groups: RouteGroupEditor; workspace?: RoutingWorkspace | null
  onModel(model: string): void; onGroup(id: string): void; onAdd(): void
  special: SpecialKey | null; onSpecial(key: SpecialKey): void
}) {
  const configured = editor.draft?.workspace.configuredModels || workspace?.configuredModels || []
  const modelKeys = new Map(configured.map(model => [model.toLowerCase(), model]))
  for (const model of Object.keys(editor.drafts).filter(model => model && draftDirty(editor.drafts[model]))) modelKeys.set(model.toLowerCase(), model)
  if (editor.scope && !modelKeys.has(editor.scope.toLowerCase())) modelKeys.set(editor.scope.toLowerCase(), editor.scope)
  const models = [...modelKeys.values()]
    .filter(model => !Object.values(groups.drafts).some(draft => draft.group.model.toLowerCase() === model.toLowerCase()))
  const disabled = editor.busy || groups.busy || groups.loading
  const addDisabled = disabled || editor.loading || !(editor.draft || workspace) || !groups.data
  const groupDrafts = Object.values(groups.drafts)
  const globalSelected = !special && !groups.selected && !editor.scope
  const dirtyDot = (dirty: boolean) => dirty ? <span className='routing-rail-dirty' title={t('There are unsaved changes')} aria-label={t('There are unsaved changes')}><StatusDot variant='warning' /></span> : undefined
  return <Sidebar className='routing-config-rail'>
    <SidebarHeader className='routing-config-rail-head'><b>{t('Routing configuration')}</b><span>{1 + SPECIAL_RAIL.length + models.length + groupDrafts.length}</span>
      <Button variant='ghost' size='icon-sm' disabled={addDisabled} onClick={onAdd}
        title={t('Add routing configuration')} aria-label={t('Add routing configuration')}><RouteIcon name='plus' size={14} /></Button>
    </SidebarHeader>
    <NavList render={<SidebarContent />} className='routing-config-rail-list' aria-label={t('Route configuration list')}>
      <SidebarGroupLabel>{t('Default rules')}</SidebarGroupLabel>
      <NavItem className='routing-config-scope-item' active={globalSelected} disabled={disabled}
        trailing={dirtyDot(Boolean(editor.drafts[''] && draftDirty(editor.drafts[''])))} onClick={() => onModel('')}>
        <b>{t('Global defaults')}</b>
      </NavItem>
      <SidebarGroupLabel>{t('Special request routing')}</SidebarGroupLabel>
      {SPECIAL_RAIL.map(item => <NavItem key={item.key} className='routing-config-scope-item' active={special === item.key}
        onClick={() => onSpecial(item.key)} title={item.label}>
        <b><span className='routing-rail-special'><RouteIcon name={item.icon} size={13} />{item.label}</span></b>
      </NavItem>)}
      {models.length ? <SidebarGroupLabel>{t('Model routes')}</SidebarGroupLabel> : null}
      {models.map(model => <NavItem key={model} className='routing-config-scope-item' active={!special && !groups.selected && model.toLowerCase() === editor.scope.toLowerCase()}
        disabled={disabled} trailing={dirtyDot(Boolean(editor.drafts[model.toLowerCase()] && draftDirty(editor.drafts[model.toLowerCase()])))} onClick={() => onModel(model)} title={model}>
        <b>{model}</b>
      </NavItem>)}
      {groupDrafts.length ? <SidebarGroupLabel>{t('Custom route groups')}</SidebarGroupLabel> : null}
      {groupDrafts.map(draft => <NavItem key={draft.group.id} disabled={disabled} className='routing-config-scope-item'
        active={draft.group.id === groups.selected} trailing={dirtyDot(groupDirty(draft))} onClick={() => onGroup(draft.group.id)} title={draft.group.model}>
        <b>{draft.group.name}</b>
      </NavItem>)}
      {groups.error ? <p className='routing-config-error' role='alert'>{groups.error}<Button variant='link' disabled={groups.loading} onClick={() => void groups.load()}>{t('Reload')}</Button></p> : null}
    </NavList>
    <SidebarFooter className='routing-config-rail-foot'>
      <Button variant='dashed' disabled={addDisabled} onClick={onAdd}><RouteIcon name='plus' size={14} />{t('新增路由配置')}</Button>
    </SidebarFooter>
  </Sidebar>
}
