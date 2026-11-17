/**
 * 路由页「特殊请求路由」的三块配置面板。
 *
 * ── 版面口径（2026-11 改版）──────────────────────────────────
 * 三块配置的真实形状都是「一份规则列表」：压缩上下文是「哪个客户端发来的压缩请求
 * 改成哪只模型」，图片兼容是「哪些模型看不了图、交给谁」，标题生成是「认哪些客户端、
 * 用哪只模型」。原来把它们画成「触发 → 改写 → 目标」三个方框 + 一堆徽章，一屏里
 * 十来个描边矩形，读者得自己拼出「谁触发、改成什么」。
 *
 * 现在改成系统设置那种分组列表（与设置页的客户端识别面板同一口径）：
 *   一句话说清这一类请求怎么被改写 → 一份规则列表（每行一句话）→
 *   点列表里的一行改这条规则，「添加规则」开弹窗填字段。
 *
 * 三块共用同一套零件（RulePanel / RuleSection / RuleRow / 弹窗字段），
 * 差别只在表单字段 —— 状态机也一样（读取 / 改草稿 / 保存 / 撤销），
 * 由 routing-special-state 的 useSpecialEditor 收口。
 *
 * ── 迁移进度 ──────────────────────────────────────────────────
 * 已迁到新版式：压缩上下文。
 * 仍用旧版式（SpecialShell / SpecialFlow，下面那一段）：标题生成 / 图片兼容 ——
 * 它们要连同后端契约一起改（标题生成从「一套模型管所有客户端」改成按规则，
 * 图片兼容从「两块固定设置」改成按规则），改完一起删掉旧壳子。
 */

import * as React from 'react'
import {
  Button, Dialog, DialogBody, DialogContent, DialogFooter, DialogHeader, DialogTitle,
  Label, MultiSelect, Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
  Spinner, Switch,
} from '@ui'
import { t } from '../i18n'
import { RouteIcon } from './routing-ui'
import { SearchableSelect } from '../../../ui-shared/searchable-select'
import {
  modelKey, type CompactConfig, type CompactRoute, type CompactView, type SpecialEditor, type TitleConfig, type TitleRule,
  type TitleView, type VisionConfig, type VisionModelRef, type VisionRuleItem, type VisionTarget, type VisionView,
} from './routing-special-state'
import '../styles/routing-special.css'

type ModelChoice = { id: string; name?: string; providerLabel?: string; provider?: string; kind?: string }

/** 模型下拉显示 ID，名称与提供商仅用于搜索。 */
function ModelSelect({ value, models, exclude, onChange, disabled, ariaLabel }: {
  value: string; models: ModelChoice[]; exclude?: string[]; onChange(next: string): void
  disabled?: boolean; ariaLabel: string
}) {
  // 同一个模型 ID 可能来自多家（别名映射），按 key 去重后再列出。
  const ids = [...new Map(models.map(model => [modelKey(model.id), model.id])).values()]
    .filter(id => !exclude?.includes(modelKey(id)) || modelKey(id) === modelKey(value))
  const selectedId = ids.find(id => modelKey(id) === modelKey(value)) || value
  return <SearchableSelect value={selectedId} disabled={disabled} onValueChange={onChange}
    className='w-full' aria-label={ariaLabel} searchPlaceholder={t('Search models')}
    displayValue={selectedId || t('Select a model')}
    options={ids.map(id => ({ value: id, label: id,
      searchText: models.filter(model => modelKey(model.id) === modelKey(id))
        .map(model => `${model.name || ''} ${model.providerLabel || ''} ${model.provider || ''}`).join(' '),
    }))} />
}

/* ═══════════════ 新版式：整页 = 一句话 + 一份规则列表 ═══════════════ */

/**
 * 面板外壳：名字 + 一句话规则 + 总开关 + 正文 + 底部保存条。
 *
 * 保存条留在面板底部而不是提到页头工具条：三块配置各有一份修订号、各自独立保存，
 * 页头那一格同时只能属于一块，「保存」按钮离它管的那块内容太远反而容易点错。
 */
function RulePanel({ title, lede, enabled, onEnabled, busy, dirty, error, loading, onReload, onSave, onDiscard, children }: {
  title: string
  lede: string
  enabled: boolean
  onEnabled(next: boolean): void
  busy: boolean
  dirty: boolean
  error: string
  loading: boolean
  onReload(): void
  onSave(): void
  onDiscard(): void
  children: React.ReactNode
}) {
  return <section className='rp-panel'>
    <header className='rp-head'>
      <div className='rp-head-text'>
        <h2>{title}</h2>
        <p className='rp-lede'>{lede}</p>
      </div>
      <label className='rp-enable'>
        <span>{enabled ? t('Enabled') : t('Disabled')}</span>
        <Switch checked={enabled} disabled={busy} aria-label={enabled ? t('Close') : t('enabled')}
          onCheckedChange={onEnabled} />
      </label>
    </header>
    <p className='rp-note'>{t('Rules apply globally, without being limited to the original routing pool. Selected models use their own routing to choose accounts; gateway key permissions still apply.')}</p>
    {loading ? <div className='rp-loading'><Spinner />{t('Loading…')}</div> : children}
    {error ? <p className='routing-config-error' role='alert'>{error}</p> : null}
    <footer className='rp-foot'>
      <span className='rp-foot-state'>{dirty ? t('There are unsaved changes') : t('Saved')}</span>
      <div className='rp-foot-actions'>
        <Button variant='ghost' size='sm' disabled={busy} onClick={onReload}>{t('Reload')}</Button>
        <Button variant='ghost' size='sm' disabled={!dirty || busy} onClick={onDiscard}>{t('Discard changes')}</Button>
        <Button size='sm' disabled={!dirty || busy} onClick={onSave}>{busy ? t('Saving…') : t('Save')}</Button>
      </div>
    </footer>
  </section>
}

/** 一个分组：13px 半粗标题（带计数与行尾操作）+ 一张无描边白底圆角卡 */
function RuleSection({ title, count, action, children }: {
  title: string; count?: string; action?: React.ReactNode; children: React.ReactNode
}) {
  return <section className='rp-section'>
    <div className='rp-section-head'>
      <h3>{title}</h3>
      {count ? <span className='rp-section-count'>{count}</span> : null}
      {action ? <span className='rp-section-action'>{action}</span> : null}
    </div>
    <div className='rp-rows'>{children}</div>
  </section>
}

/** 一行规则：整行可点（真按钮，键盘能 Tab 到），右侧一枚雪佛龙暗示「点进去改」 */
function RuleRow({ title, detail, side, off, onClick }: {
  title: React.ReactNode
  detail: React.ReactNode
  side?: React.ReactNode
  off?: boolean
  onClick(): void
}) {
  return <button type='button' className='rp-row' data-off={off ? '' : undefined} onClick={onClick}>
    <span className='rp-row-main'>
      <span className='rp-row-title'>{title}</span>
      <span className='rp-row-detail'>{detail}</span>
    </span>
    <span className='rp-row-side'>
      {side}
      <span className='rp-caret'><RouteIcon name='chevronRight' size={13} /></span>
    </span>
  </button>
}

/** 弹窗表单的一格：标签在上、控件在下（与设置页的规则编辑弹窗同一口径） */
function DlgField({ label, children }: { label: string; children: React.ReactNode }) {
  return <div className='rp-field'><Label>{label}</Label>{children}</div>
}

/** 弹窗顶部那条「启用规则」：开关在左、状态右对齐 */
function RuleSwitchRow({ enabled, disabled, onChange }: {
  enabled: boolean; disabled?: boolean; onChange(next: boolean): void
}) {
  return <div className='rp-switch-row'>
    <Switch checked={enabled} disabled={disabled} aria-label={t('Enable rule')} onCheckedChange={onChange} />
    <span>{t('Enable rule')}</span>
    <small>{enabled ? t('Enabled') : t('Disabled')}</small>
  </div>
}

/** 弹窗底栏：左边一枚删除（只编辑已有规则时出现），右边取消 + 保存 */
function RuleDialogFooter({ onClose, onSave, onDelete, disabled }: {
  onClose(): void; onSave(): void; onDelete?(): void; disabled?: boolean
}) {
  return <DialogFooter>
    {onDelete ? <Button variant='destructive' className='mr-auto' disabled={disabled}
      onClick={() => { onDelete(); onClose() }}>{t('Delete rule')}</Button> : null}
    <Button variant='ghost' disabled={disabled} onClick={onClose}>{t('Cancel')}</Button>
    <Button disabled={disabled} onClick={onSave}>{t('Save rule')}</Button>
  </DialogFooter>
}

/* ─────────────────── 压缩上下文 ─────────────────── */

function CompactRuleDialog({ rule, models, clients, saving, onClose, onSave, onDelete }: {
  rule: CompactRoute
  models: ModelChoice[]
  clients: CompactView['clients']
  saving: boolean
  onClose(): void
  onSave(rule: CompactRoute): void
  onDelete?(): void
}) {
  const [draft, setDraft] = React.useState(rule)
  const update = (patch: Partial<CompactRoute>) => setDraft(current => ({ ...current, ...patch }))
  const clientLabel = clients.find(client => client.id === draft.client)?.label || draft.client
  return <Dialog open onOpenChange={next => { if (!next) onClose() }}>
    <DialogContent className='w-[min(520px,calc(100vw-24px))]'>
      <DialogHeader><DialogTitle>{onDelete ? t('Edit rule') : t('Add rule')}</DialogTitle></DialogHeader>
      <DialogBody>
        <p className='text-[12px] leading-[1.7] text-muted-foreground'>
          {t('Compaction requests matching this rule are re-sent once with the compaction model; the account is then chosen by that model\'s own route.')}
        </p>
        <RuleSwitchRow enabled={draft.enabled} disabled={saving} onChange={enabled => update({ enabled })} />
        <DlgField label={t('Triggering client')}>
          <Select value={draft.client} disabled={saving}
            onValueChange={next => update({ client: String(next) })}>
            <SelectTrigger aria-label={t('Triggering client')}><SelectValue>{clientLabel}</SelectValue></SelectTrigger>
            <SelectContent>{clients.map(client =>
              <SelectItem key={client.id} value={client.id}>{client.label}</SelectItem>)}</SelectContent>
          </Select>
        </DlgField>
        <DlgField label={t('Original model')}>
          <ModelSelect value={draft.source} models={models} exclude={[modelKey(draft.target)]} disabled={saving}
            ariaLabel={t('Original model')} onChange={source => update({ source })} />
        </DlgField>
        <DlgField label={t('Compaction model')}>
          <ModelSelect value={draft.target} models={models} exclude={[modelKey(draft.source)]} disabled={saving}
            ariaLabel={t('Compaction model')} onChange={target => update({ target })} />
        </DlgField>
      </DialogBody>
      <RuleDialogFooter disabled={saving} onClose={onClose} onSave={() => onSave(draft)} onDelete={onDelete} />
    </DialogContent>
  </Dialog>
}

export function CompactSpecialPanel({ editor }: { editor: SpecialEditor<CompactConfig, CompactView> }) {
  const { view, draft, loading, saving, dirty, error } = editor
  const models = view?.models || []
  const clients = view?.clients || []
  const maxRules = view?.maxRules || 0
  const rules = draft?.rules || []
  /** 正在编辑的那条规则：index = -1 表示新增（保存时追加到末尾） */
  const [editing, setEditing] = React.useState<{ rule: CompactRoute; index: number } | null>(null)
  const clientLabel = (id: string) => clients.find(client => client.id === id)?.label || id
  const full = maxRules > 0 && rules.length >= maxRules
  const write = (index: number, rule: CompactRoute) => editor.update(current => ({
    ...current,
    rules: index < 0 ? [...current.rules, rule] : current.rules.map((item, at) => at === index ? rule : item),
  }))
  const blank: CompactRoute = { client: clients[0]?.id || '', source: '', target: '', enabled: true }
  return <RulePanel title={t('Context compaction')}
    lede={t('Context compaction requests matching a rule below are compacted once with the model you pick; the account is then chosen by that model\'s own route.')}
    enabled={Boolean(draft?.enabled)} onEnabled={next => editor.update(current => ({ ...current, enabled: next }))}
    busy={saving || loading} dirty={dirty} error={error} loading={loading}
    onReload={() => void editor.reload()} onSave={() => void editor.save()} onDiscard={editor.discard}>
    {draft ? <>
      <RuleSection title={t('Rules')} count={t('{n} entries', { n: rules.length })}
        action={<Button variant='outline' size='sm' disabled={saving || full || !models.length}
          onClick={() => setEditing({ rule: blank, index: -1 })}>
          <RouteIcon name='plus' size={13} />{t('Add rule')}
        </Button>}>
        {rules.length ? rules.map((rule, index) => <RuleRow key={index} off={!rule.enabled}
          title={t('Compaction request from {client}', { client: clientLabel(rule.client) })}
          detail={<>
            <span>{rule.source || t('No source model selected')}</span>
            <span aria-hidden='true'>→</span>
            <span className='rp-row-target'>{rule.target || t('No compaction model selected')}</span>
          </>}
          side={rule.enabled ? null : t('Disabled')}
          onClick={() => setEditing({ rule, index })} />)
          : <p className='rp-empty'>{t('No rules yet. Compaction requests are forwarded with the original model.')}</p>}
      </RuleSection>
      {!models.length ? <p className='rp-note'>{t('No chat models are available. Enable a model and its account first.')}</p> : null}
      {full ? <p className='rp-note'>{t('Up to {n} rules.', { n: maxRules })}</p> : null}
    </> : null}
    {editing ? <CompactRuleDialog key={editing.index} rule={editing.rule} models={models} clients={clients} saving={saving}
      onClose={() => setEditing(null)}
      onSave={rule => { write(editing.index, rule); setEditing(null) }}
      onDelete={editing.index < 0 ? undefined : () => editor.update(current => ({
        ...current, rules: current.rules.filter((_, at) => at !== editing.index),
      }))} /> : null}
  </RulePanel>
}

/* ─────────────────── 图片兼容 ─────────────────── */

const visionKey = (target: { provider: string; model: string }) => JSON.stringify([target.provider, target.model])

const parseVisionKey = (key: string): VisionModelRef | null => {
  try {
    const parsed: unknown = JSON.parse(key)
    return Array.isArray(parsed) && parsed.length === 2 && parsed.every(item => typeof item === 'string')
      ? { provider: String(parsed[0]), model: String(parsed[1]) } : null
  } catch { return null }
}

/** 图片兼容按提供商绑定模型，显示 ID 时仍需保留提供商以区分同名模型。 */
function visionLabel(models: VisionView['models'], ref: { provider: string; model: string } | null): string {
  if (!ref) return ''
  const found = models.find(model => model.provider === ref.provider && modelKey(model.model) === modelKey(ref.model))
  return `${found?.providerLabel || ref.provider} · ${found?.model || ref.model}`
}

function VisionRuleDialog({ rule, view, saving, onClose, onSave, onDelete }: {
  rule: VisionRuleItem
  view: VisionView
  saving: boolean
  onClose(): void
  onSave(rule: VisionRuleItem): void
  onDelete?(): void
}) {
  const [draft, setDraft] = React.useState(rule)
  const [candidate, setCandidate] = React.useState('')
  const update = (patch: Partial<VisionRuleItem>) => setDraft(current => ({ ...current, ...patch }))
  const listChoices = view.models.filter(model => model.capabilities?.supportsImages !== true)
    .map(model => ({ value: visionKey(model), label: `${model.providerLabel || model.provider} · ${model.model}` }))
  const targetChoices = view.models.filter(model => model.capabilities?.supportsImages === true)
    .map(model => ({ value: visionKey(model), label: `${model.providerLabel || model.provider} · ${model.model}`,
      searchText: `${model.name || ''} ${model.provider}` }))
  const [primary, ...backups] = draft.targets
  const setTargets = (targets: VisionTarget[]) =>
    update({ targets, fallback: targets.length ? 'designated' : 'stop' })
  const pick = (value: string): VisionTarget | null => {
    const ref = parseVisionKey(value)
    return ref ? { ...ref, reasoning: '' } : null
  }
  const move = (index: number, direction: number) => {
    const next = [...draft.targets]
    const to = index + direction
    if (to < 0 || to >= next.length) return
    ;[next[index], next[to]] = [next[to], next[index]]
    setTargets(next)
  }
  return <Dialog open onOpenChange={next => { if (!next) onClose() }}>
    <DialogContent className='w-[min(560px,calc(100vw-24px))]'>
      <DialogHeader><DialogTitle>{onDelete ? t('Edit rule') : t('Add rule')}</DialogTitle></DialogHeader>
      <DialogBody>
        <p className='text-[12px] leading-[1.7] text-muted-foreground'>
          {t('Requests for the models in this rule are handled the chosen way when they contain images; models not selected by any rule pass through unchanged.')}
        </p>
        <RuleSwitchRow enabled={draft.enabled} disabled={saving} onChange={enabled => update({ enabled })} />
        <DlgField label={t('Which models use this way')}>
          <MultiSelect className='w-full' value={draft.models.map(visionKey)} disabled={saving}
            options={listChoices} placeholder={t('Select one or more models')} searchPlaceholder={t('Search providers or models')}
            aria-label={t('Which models use this way')}
            onValueChange={keys => update({ models: keys.map(parseVisionKey).filter((ref): ref is VisionModelRef => ref !== null) })} />
        </DlgField>
        <DlgField label={t('Processing mode')}>
          <Select value={draft.mode} disabled={saving}
            onValueChange={next => update({ mode: String(next) as VisionRuleItem['mode'] })}>
            <SelectTrigger className='w-full' aria-label={t('Processing mode')}><SelectValue>
              {draft.mode === 'assist' ? t('Assisted image reading') : t('Take over entirely')}
            </SelectValue></SelectTrigger>
            <SelectContent>
              <SelectItem value='assist'>{t('Assisted image reading')}</SelectItem>
              <SelectItem value='reroute'>{t('Take over entirely')}</SelectItem>
            </SelectContent>
          </Select>
        </DlgField>
        {draft.mode === 'assist' ? <>
          <DlgField label={t('Which model reads the image')}>
            <SearchableSelect className='w-full' value={draft.source === 'shared' ? 'shared' : (draft.target ? visionKey(draft.target) : '')}
              disabled={saving} aria-label={t('Which model reads the image')} searchPlaceholder={t('Search models')}
              displayValue={draft.source === 'shared' ? t('Use the tool service vision model') : (visionLabel(view.models, draft.target) || t('Select a model'))}
              options={[{ value: 'shared', label: t('Use the tool service vision model') }, ...targetChoices]}
              onValueChange={next => {
                if (next === 'shared') { update({ source: 'shared' }); return }
                const target = pick(next)
                if (target) update({ source: 'custom', target: { ...target, reasoning: draft.target?.reasoning || '' } })
              }} />
          </DlgField>
          <DlgField label={t('Reuse readings of identical images')}>
            <div className='flex items-center gap-3'>
              <Switch checked={draft.reuseResults} disabled={saving} aria-label={t('Reuse readings of identical images')}
                onCheckedChange={next => update({ reuseResults: next === true })} />
              <span className='text-[12px] text-muted-foreground'>{t('Reused within the same session, question and permission scope; asking about new details triggers a fresh read.')}</span>
            </div>
          </DlgField>
        </> : <>
          <DlgField label={t('Prefer vision models in the current route')}>
            <div className='flex items-center gap-3'>
              <Switch checked={draft.preferCurrentRoute} disabled={saving} aria-label={t('Prefer vision models in the current route')}
                onCheckedChange={next => update({ preferCurrentRoute: next === true })} />
              <span className='text-[12px] text-muted-foreground'>{t('Try vision models in the current route first, then try the selected models in order if unavailable. Selected models can use a different routing pool.')}</span>
            </div>
          </DlgField>
          <DlgField label={t('Model that takes over the answer')}>
            <SearchableSelect className='w-full' value={primary ? visionKey(primary) : ''} disabled={saving}
              aria-label={t('Model that takes over the answer')} searchPlaceholder={t('Search models')}
              displayValue={primary ? visionLabel(view.models, primary) : t('Select a model')}
              options={targetChoices}
              onValueChange={next => {
                const target = pick(next)
                if (target) setTargets(target ? [{ ...target, reasoning: primary?.reasoning || '' }, ...backups] : backups)
              }} />
          </DlgField>
          <DlgField label={t('Backup model')}>
            <div className='flex flex-col gap-2'>
              <div className='flex items-center gap-2'>
                <SearchableSelect className='min-w-0 flex-1' value={candidate} disabled={saving || draft.targets.length >= view.maxTargets}
                  aria-label={t('Backup model')} searchPlaceholder={t('Search models')}
                  displayValue={visionLabel(view.models, parseVisionKey(candidate)) || t('Select a model')}
                  options={targetChoices} onValueChange={setCandidate} />
                <Button variant='outline' size='sm' disabled={saving || !candidate || draft.targets.length >= view.maxTargets}
                  onClick={() => { const target = pick(candidate); if (target) { setTargets([...draft.targets, target]); setCandidate('') } }}>
                  {t('Add as backup')}
                </Button>
              </div>
              {backups.map((backup, index) => <div key={visionKey(backup)} className='flex items-center gap-2 text-[12px]'>
                <span className='min-w-0 flex-1 truncate'>{visionLabel(view.models, backup)}</span>
                <Button variant='ghost' size='xs' disabled={saving || index === 0} onClick={() => move(index + 1, -1)}>{t('Move up')}</Button>
                <Button variant='ghost' size='xs' disabled={saving || index === backups.length - 1} onClick={() => move(index + 1, 1)}>{t('Move down')}</Button>
                <Button variant='ghost' size='xs' disabled={saving}
                  onClick={() => setTargets(draft.targets.filter((_, at) => at !== index + 1))}>{t('Remove')}</Button>
              </div>)}
              <span className='text-[11.5px] leading-[1.6] text-muted-foreground'>
                {t('Fallbacks are tried in order; with none added, the request stops with an explanation when no vision model is found in the current route.')}
              </span>
            </div>
          </DlgField>
        </>}
      </DialogBody>
      <RuleDialogFooter disabled={saving} onClose={onClose} onSave={() => onSave(draft)} onDelete={onDelete} />
    </DialogContent>
  </Dialog>
}

export function VisionSpecialPanel({ editor }: { editor: SpecialEditor<VisionConfig, VisionView> }) {
  const { view, draft, loading, saving, dirty, error } = editor
  const rules = draft?.rules || []
  const [editing, setEditing] = React.useState<{ rule: VisionRuleItem; index: number } | null>(null)
  const blank: VisionRuleItem = {
    enabled: true, models: [], mode: 'assist', source: 'shared', target: null,
    reuseResults: true, preferCurrentRoute: true, fallback: 'stop', targets: [],
  }
  const write = (index: number, rule: VisionRuleItem) => editor.update(current => ({
    ...current,
    rules: index < 0 ? [...current.rules, rule] : current.rules.map((item, at) => at === index ? rule : item),
  }))
  const detail = (rule: VisionRuleItem) => {
    const label = visionLabel(view?.models || [], rule.targets[0] || rule.target)
    if (rule.mode === 'assist') {
      const reader = rule.source === 'shared' ? t('the tool service vision model') : (label || t('No vision model selected'))
      return <><span>{t('Reads the image first, then hands back to')}</span><span className='rp-row-target'>{reader}</span><span>· {t('The original model continues answering')}</span></>
    }
    const taker = rule.preferCurrentRoute ? t('a vision model in this route') : (label || t('No vision model selected'))
    return <><span>{t('Hands the whole request to')}</span><span className='rp-row-target'>{taker}</span></>
  }
  return <RulePanel title={t('Image compatibility')}
    lede={t('When a model cannot read images, the rules below hand them to a vision model; models not configured here — and those that can read images anyway — pass through unchanged.')}
    enabled={Boolean(draft?.enabled)} onEnabled={next => editor.update(current => ({ ...current, enabled: next }))}
    busy={saving || loading} dirty={dirty} error={error} loading={loading}
    onReload={() => void editor.reload()} onSave={() => void editor.save()} onDiscard={editor.discard}>
    {draft && view ? <>
      <RuleSection title={t('Rules')} count={t('{n} entries', { n: rules.length })}
        action={<Button variant='outline' size='sm' disabled={saving}
          onClick={() => setEditing({ rule: blank, index: -1 })}>
          <RouteIcon name='plus' size={13} />{t('Add rule')}
        </Button>}>
        {rules.length ? rules.map((rule, index) => <RuleRow key={index} off={!rule.enabled}
          title={rule.models.length
            ? t('When {models} cannot read images', { models: rule.models.map(model => visionLabel(view.models, model)).join(', ') })
            : t('No model selected')}
          detail={detail(rule)}
          side={rule.enabled ? null : t('Disabled')}
          onClick={() => setEditing({ rule, index })} />)
          : <p className='rp-empty'>{t('No rules yet. Images go to the original model as usual; models that cannot read them will fail.')}</p>}
      </RuleSection>
      {!view.models.length ? <p className='rp-note'>{t('No models available yet; enable models and their accounts first.')}</p> : null}
    </> : null}
    {editing && view ? <VisionRuleDialog key={editing.index} rule={editing.rule} view={view} saving={saving}
      onClose={() => setEditing(null)}
      onSave={rule => { write(editing.index, rule); setEditing(null) }}
      onDelete={editing.index < 0 ? undefined : () => editor.update(current => ({
        ...current, rules: current.rules.filter((_, at) => at !== editing.index),
      }))} /> : null}
  </RulePanel>
}

/* ─────────────────── 标题生成 ─────────────────── */

function TitleRuleDialog({ rule, models, clients, saving, onClose, onSave, onDelete }: {
  rule: TitleRule
  models: ModelChoice[]
  clients: TitleView['clients']
  saving: boolean
  onClose(): void
  onSave(rule: TitleRule): void
  onDelete?(): void
}) {
  const [draft, setDraft] = React.useState(rule)
  const update = (patch: Partial<TitleRule>) => setDraft(current => ({ ...current, ...patch }))
  return <Dialog open onOpenChange={next => { if (!next) onClose() }}>
    <DialogContent className='w-[min(520px,calc(100vw-24px))]'>
      <DialogHeader><DialogTitle>{onDelete ? t('Edit rule') : t('Add rule')}</DialogTitle></DialogHeader>
      <DialogBody>
        <p className='text-[12px] leading-[1.7] text-muted-foreground'>
          {t('This rule only rewrites small requests recognised as title generation; normal conversations are unaffected.')}
        </p>
        <RuleSwitchRow enabled={draft.enabled} disabled={saving} onChange={enabled => update({ enabled })} />
        <DlgField label={t('Recognised clients')}>
          <MultiSelect className='w-full' value={draft.clients} disabled={saving}
            options={clients.map(client => ({ value: client.id, label: client.label, disabled: !client.supported }))}
            placeholder={t('Select one or more clients')} searchPlaceholder={t('Search clients')}
            aria-label={t('Recognised clients')} onValueChange={next => update({ clients: next })} />
        </DlgField>
        <DlgField label={t('The title model')}>
          <ModelSelect value={draft.model} models={models} disabled={saving}
            ariaLabel={t('The title model')} onChange={model => update({ model })} />
        </DlgField>
        <DlgField label={t('When the title model fails')}>
          <Select value={draft.fallback} disabled={saving}
            onValueChange={next => update({ fallback: next as TitleRule['fallback'] })}>
            <SelectTrigger className='w-full' aria-label={t('When the title model fails')}><SelectValue>
              {draft.fallback === 'original' ? t('Fall back to the original model') : t('Error out directly')}
            </SelectValue></SelectTrigger>
            <SelectContent>
              <SelectItem value='original'>{t('Fall back to the original model')}</SelectItem>
              <SelectItem value='error'>{t('Error out directly')}</SelectItem>
            </SelectContent>
          </Select>
        </DlgField>
      </DialogBody>
      <RuleDialogFooter disabled={saving} onClose={onClose} onSave={() => onSave(draft)} onDelete={onDelete} />
    </DialogContent>
  </Dialog>
}

export function TitleSpecialPanel({ editor }: { editor: SpecialEditor<TitleConfig, TitleView> }) {
  const { view, draft, loading, saving, dirty, error } = editor
  const models = view?.models || []
  const clients = view?.clients || []
  const rules = draft?.rules || []
  const [editing, setEditing] = React.useState<{ rule: TitleRule; index: number } | null>(null)
  const clientLabel = (id: string) => clients.find(client => client.id === id)?.label || id
  const write = (index: number, rule: TitleRule) => editor.update(current => ({
    ...current,
    rules: index < 0 ? [...current.rules, rule] : current.rules.map((item, at) => at === index ? rule : item),
  }))
  const blank: TitleRule = { enabled: true, clients: [], model: '', fallback: 'original' }
  return <RulePanel title={t('Title generation')}
    lede={t('Only small requests recognised as title generation are rewritten; normal conversations are unaffected.')}
    enabled={Boolean(draft?.enabled)} onEnabled={next => editor.update(current => ({ ...current, enabled: next }))}
    busy={saving || loading} dirty={dirty} error={error} loading={loading}
    onReload={() => void editor.reload()} onSave={() => void editor.save()} onDiscard={editor.discard}>
    {draft ? <>
      <RuleSection title={t('Rules')} count={t('{n} entries', { n: rules.length })}
        action={<Button variant='outline' size='sm' disabled={saving || !models.length}
          onClick={() => setEditing({ rule: blank, index: -1 })}>
          <RouteIcon name='plus' size={13} />{t('Add rule')}
        </Button>}>
        {rules.length ? rules.map((rule, index) => <RuleRow key={index} off={!rule.enabled}
          title={rule.clients.length
            ? t('Title request from {clients}', { clients: rule.clients.map(clientLabel).join(', ') })
            : t('No client selected')}
          detail={<>
            <span>{t('Switch to')}</span>
            <span className='rp-row-target'>{rule.model || t('No title model selected')}</span>
            <span>·</span>
            <span>{rule.fallback === 'original' ? t('Fall back to the original model on failure') : t('Report the error on failure')}</span>
          </>}
          side={rule.enabled ? null : t('Disabled')}
          onClick={() => setEditing({ rule, index })} />)
          : <p className='rp-empty'>{t('No rules yet. Title requests go to the original model as usual.')}</p>}
      </RuleSection>
      {!models.length ? <p className='rp-note'>{t('No chat models are available. Enable a model and its account first.')}</p> : null}
    </> : null}
    {editing ? <TitleRuleDialog key={editing.index} rule={editing.rule} models={models} clients={clients} saving={saving}
      onClose={() => setEditing(null)}
      onSave={rule => { write(editing.index, rule); setEditing(null) }}
      onDelete={editing.index < 0 ? undefined : () => editor.update(current => ({
        ...current, rules: current.rules.filter((_, at) => at !== editing.index),
      }))} /> : null}
  </RulePanel>
}
