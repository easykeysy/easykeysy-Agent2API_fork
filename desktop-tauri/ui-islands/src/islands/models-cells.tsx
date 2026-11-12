import * as React from 'react'
import { Badge, Button, Checkbox, Switch, TableCell, cn } from '@ui'
import { t } from '../i18n'
import * as customSource from './models-custom-source'
import type { ManageModel } from './models-custom-source'
import { bindingReasoningLevel, modelReasoning } from './models-reasoning'
import { testBlockReason, type ModelTestTarget } from './model-test-dialog'
import {
  BOOLEAN_KEYS, CAPABILITY_LABELS, capabilitiesOf, capabilityState, capabilityTip, capSourceOf,
  exactTokens, formatTokens, normalizeOverrides, type CapabilityKey,
} from './model-capability'
import {
  bindingKeyOf, bindingsOf, enabledBindingConflicts, esc, formatTime, getSnapshot, levelOf, models,
  openCapability, openMapping, rowKeyOf, runRowAction, same, shared, subscribe, visibilityOf,
  writeBinding, writeRemoveMapping, writeRemoveModel, type Align, type ColumnView, type Binding,
} from './models-panel-state'

const cellClass = (base: string, align?: Align): string => (align ? `${base} ta-${align}` : base)
const formatCredits = (credits: unknown): string => {
  const match = /x\s*([\d.]+)/i.exec(String(credits || ''))
  return match ? `${match[1]}x` : String(credits || '')
}

/* ─── can （「 can 」 ）──────────────────────
    → ： in 1 ， is ；
    when is 「 」， by 。 and icons.js
    1 （24 / 2 / ）， in file —— icons.js is 
   vanilla ， this not to ， not this as 。 */

const CAP_ICON: Partial<Record<CapabilityKey, React.ReactNode>> = {
  supportsToolCall: (
    // （Feather「tool」）
    <svg viewBox='0 0 24 24' aria-hidden='true'>
      <path fill='none' stroke='currentColor' strokeWidth='2' strokeLinecap='round' strokeLinejoin='round'
        d='M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z' />
    </svg>
  ),
  supportsImages: (
    // （Feather「image」： + + ）
    <svg viewBox='0 0 24 24' aria-hidden='true'>
      <g fill='none' stroke='currentColor' strokeWidth='2' strokeLinecap='round' strokeLinejoin='round'>
        <rect x='3' y='3' width='18' height='18' rx='2.5' /><circle cx='8.5' cy='8.5' r='1.5' />
        <path d='M21 15l-5-5L5 21' />
      </g>
    </svg>
  ),
  supportsVideo: (
    // （ + ）：「」 in down can 
    <svg viewBox='0 0 24 24' aria-hidden='true'>
      <g fill='none' stroke='currentColor' strokeWidth='2' strokeLinecap='round' strokeLinejoin='round'>
        <circle cx='12' cy='12' r='9' /><path d='M10 8.5l6 3.5-6 3.5z' />
      </g>
    </svg>
  ),
  supportsReasoning: (
    // （Lucide「lightbulb」）
    <svg viewBox='0 0 24 24' aria-hidden='true'>
      <g fill='none' stroke='currentColor' strokeWidth='2' strokeLinecap='round' strokeLinejoin='round'>
        <path d='M15 14c.2-1 .7-1.7 1.5-2.5 1-.9 1.5-2.2 1.5-3.5A6 6 0 0 0 6 8c0 1 .2 2.2 1.5 3.5.7.7 1.3 1.5 1.5 2.5' />
        <path d='M9 18h6M10 22h4' />
      </g>
    </svg>
  ),
}

export function ModelCell({ columns, model, selection, togglePick, setTestTarget }: {
  columns: ColumnView[]
  model: ManageModel
  selection: ReadonlySet<string>
  togglePick: (key: string, next: boolean) => void
  setTestTarget: (target: ModelTestTarget) => void
}) {
  const state = React.useSyncExternalStore(subscribe, getSnapshot)
  const busyRow = state.pending.has(rowKeyOf(model))
  /** chip up that etc （ when ，otherwise「 etc 」 will 1 not ） */
  function levelBadge(alias: string, target: string, providerId: string, busy: boolean) {
    const level = levelOf(alias, target, providerId)
    const model = models().find(item => same(item.provider, providerId) && same(item.id, target))
    const info = modelReasoning(model)
    const effective = info.bindingSupported === true ? bindingReasoningLevel(level, model) : ''
    const title = info.bindingSupported === false
      ? t('当前通道未声明可用档位，映射等级不会注入；请先获取模型目录。')
      : info.fixedLevel
        ? t('上游固定为 {level}，绑定其他等级也无法调高。', { level: info.fixedLevel })
        : effective && effective.toLowerCase() !== level.toLowerCase()
          ? t('绑定 {requested}，按当前通道将发送 {effective}。', { requested: level, effective })
          : level ? t('思考等级 {level}（点击修改）', { level }) : t('设置思考等级（当前未绑定）')
    return (
      // 16px （2xs ）： that mono 10px 。 already 、，
      // and old CSS `.alias-level` / `.alias-level.unset` 1 （mono old ）；
      // shadow-none is use button then （ old CSS ）
      <Button variant={level ? 'secondary' : 'dashed'} size='2xs' disabled={busy}
        className='font-mono font-bold shadow-none'
        title={title}
        onClick={() => openMapping({ alias, target, provider: providerId })}>
        {info.fixedLevel ? `${t('固定思考档位')} ${info.fixedLevel}` : effective && effective.toLowerCase() !== level.toLowerCase()
          ? `${level} → ${effective}` : level || t('＋等级')}
      </Button>
    )
  }

  /** chips： chip in that 1 （ × up ），delete when */
  function aliasCell(model: ManageModel) {
    const providerId = model.provider || ''
    const bindings = bindingsOf(model)
    const [head, ...rest] = bindings

    /**
     * delete 1 ：， by （ for + up + ）。
     * not use that （ etc ） down —— 1 ， before can 。
     */
    async function confirmUnmap(alias: string): Promise<void> {
      const ok = await shared().wbConfirm?.ask?.({
        title: t('删除映射'),
        html: t('确定删除映射「<strong>{alias} → {target}</strong>」？', { alias: esc(alias), target: esc(model.id) }),
        okText: t('删除'),
        okClass: 'danger',
      })
      if (!ok) return
      void runRowAction(
        bindingKeyOf(alias, model.id, providerId),
        () => writeRemoveMapping(providerId, alias, model.id),
        t('映射已删除'),
      )
    }

    const chip = (binding: Binding) => {
      const alias = binding.alias
      const on = binding.enabled !== false
      const conflicts = on ? [] : enabledBindingConflicts(providerId, alias, model.id)
      const busy = state.pending.has(bindingKeyOf(alias, model.id, providerId))
      return (
        <span className={cn('alias', !on && 'map-off')} key={`${alias}\u0001${binding.isDefault ? 'd' : 'm'}`}>
          {/* （ = this when not in ， can ）。 use when chips ，
               use map-off —— ， 1 can 。size='sm' is 24×14 ，
               and old CSS `.alias .switch .track` ：（36×21） 22px 
               will chip 1 */}
          <Switch size='sm' checked={on} disabled={busy}
            title={on ? t('映射已启用，点击关闭') : conflicts.length
              ? t('启用后将自动关闭该提供商的其他同名绑定：{bindings}', { bindings: conflicts.map(binding => `${binding.alias} → ${binding.target}`).join('、') })
              : t('映射已关闭，点击启用')}
            aria-label={t('{action}映射 {alias}', { action: on ? t('关闭') : t('启用'), alias })}
            onCheckedChange={next => {
              void runRowAction(
                bindingKeyOf(alias, model.id, providerId),
                () => writeBinding(providerId, alias, model.id, { enabled: next }),
                next ? (conflicts.length ? t('映射已启用，其他同名绑定已自动关闭') : t('映射已启用')) : t('映射已关闭'),
              )
            }} />
          {/* is （data-copy clipboard.js ， 1 down 
              toast「 already 」）；title when —— can 、 
              ellipsis when can */}
          <span className='t' data-copy={alias} title={t('点击复制：{name}', { name: alias })}>{alias}</span>
          {binding.isDefault ? <span className='binding-default'>{t('默认')}</span> : null}
          {levelBadge(alias, model.id, providerId, busy)}
          {binding.isDefault ? null : (
            // 16px delete（icon-2xs ）： / old CSS `.alias .x` to ，
            // × chip 11px 1 
            <Button variant='ghost' size='icon-2xs' className='text-[12px] leading-none'
              disabled={busy} title={t('删除映射 {alias}', { alias })}
              onClick={() => void confirmUnmap(alias)}>×</Button>
          )}
        </span>
      )
    }
    // 「＋ 」 and in **default** that （ is this 1 ， not is 1 ）：default
    // in （ no has when is that ）， this 1 has 
    return (
      <div className='aliases'>
        <div className='alias-row'>
          {head ? chip(head) : null}
          {/* dashed = 「 this can 1 」 （ for old .alias-add）， and that 
               by 1 */}
          <Button variant='dashed' size='xs'
            onClick={() => openMapping({ target: model.id, provider: providerId })}>{t('＋ 映射')}</Button>
        </div>
        {rest.map(chip)}
      </div>
    )
  }

  /**
   * 「」： this 1 current is is （ after to source， before ）。
   * as 「 + 」（.src） not is ： this 1 in 1 ， 
   * will —— page-gateway.css（ = / = / = ）。
   */
  function sourceCell(model: ManageModel) {
    if (model.source === 'manual') {
      return <span className='src manual' title={t('手动登记的上游模型；移除它会直接删掉这条登记')}><i aria-hidden='true' />{t('手动')}</span>
    }
    // （source empty）： in —— in this when this 1 
    // by 。 to 「」 not is 「—」：「—」
    // 「unknown」， is 1 1 （ is use ）
    if (!model.source && customSource.isCustom(model.provider)) {
      return <span className='src manual' title={t('用户登记的自定义模型；整份清单只属于这一家')}><i aria-hidden='true' />{t('自定义')}</span>
    }
    if (model.source !== 'remote' && model.source !== 'builtin') return <span className='rate'>—</span>
    const remote = model.source === 'remote'
    const at = formatTime(Number(model.refreshedAt) || 0)
    const hint = remote
      ? (at ? t('来自上游目录接口，清单拉取于 {at}；刷新失败时保留上一份成功结果', { at }) : t('来自上游目录接口；刷新失败时保留上一份成功结果'))
      : t('上游目录尚未拉到，用的是内置静态清单；点「刷新模型清单」可重试')
    return <span className={cn('src', remote ? 'remote' : 'builtin')} title={hint}><i aria-hidden='true' />{remote ? t('远程') : t('内置')}</span>
  }

  /**
   * 「 can 」： this in **client** will not will 。
   *
   * ── + 1 「 no has 」──────────────────────────────────
   * · ** not **（`hidden`）： up has 、client not 。 already default 
   * default（`model_rules::seed_hidden_disabled`）—— is not in 
   * `/v1/models`， not use 。 use can in this 1 （ is 
   * 「 not ， need use 」）。
   * · ** can **（`shown`）：client will 。
   * · ****（`undeclared`）： this has 、 this up no 。
   * · **`—`**（ no has `visibility` ）： this ** no has ** can 。
   *
   * after must：`undeclared` is 「 this not 」，`—` is 「 this 
   * no this 」。 1 ， use will as all no has 「 can 」。
   */
  function visibilityCell(model: ManageModel) {
    const visibility = visibilityOf(model)
    if (visibility === 'hidden') {
      return (
        <span className='src hidden-visibility'
          title={t('上游能列出这个模型，但官方客户端不展示它；本仓已默认关闭它的默认绑定（点这一行的开关可以手动打开）')}>
          <i aria-hidden='true' />{t('不展示')}
        </span>
      )
    }
    if (visibility === 'visible') {
      return (
        <span className='src shown-visibility' title={t('官方客户端会展示这个模型')}>
          <i aria-hidden='true' />{t('可见')}
        </span>
      )
    }
    if (visibility === 'undeclared') {
      return <span className='rate' title={t('这家有可见性判据，但这一条上游没有表态 —— 按可见处理')}>—</span>
    }
    // no has ： this 1 this by ？ not —— is （
    // has and no in 1 up ）， to 1 `—`。
    return <span className='rate' title={t('这家提供商没有可见性判据（目前只有 Antigravity / Qoder / 小浣熊 / CodeArts 有）')}>—</span>
  }

  /**
   * 「 up down / 」： token in 1 （ up down / ）。
   *
   * is ****（ + ， 1 after ）； that 1 1 
   * （`.cap-mark`，title ）， `—`。 is 1 by ：
   * can （ 1 —— can is 1 ， no need 1 ）。
   *
   * ****： this is models.dev （ up not to ），
   * that 1 （`.filled`）—— up not that ，
   * is 1 1 up ； will 
   * 「； its in Token by models.dev 」。 not ， use no this 
   * is up is —— can not 1 。
   */
  function budgetCell(model: ManageModel) {
    const caps = capabilitiesOf(model)
    const overrides = normalizeOverrides(model.capOverrides)
    const input = typeof caps.maxInputTokens === 'number' ? caps.maxInputTokens : null
    const output = typeof caps.maxOutputTokens === 'number' ? caps.maxOutputTokens : null
    const hint = t('上下文窗口 {input} / 最大输出 Token {output} —— 点击修改模型能力', { input: exactTokens(input), output: exactTokens(output) })
      + tokenOriginSuffix(model)
    return (
      <Button size='default' variant='ghost' type='button' className='caps-open' title={hint}
        onClick={() => openCapability(model.provider || '', model.id)}>
        <span className={cn('cap-num', input === null && 'unset', capSourceOf(model, 'maxInputTokens') === 'modelsdev' && 'filled')}>
          {formatTokens(input)}
          {overrides.includes('maxInputTokens') ? <i className='cap-mark' title={t('已覆盖上游值')} /> : null}
        </span>
        <span className='cap-sep'>/</span>
        <span className={cn('cap-num', output === null && 'unset', capSourceOf(model, 'maxOutputTokens') === 'modelsdev' && 'filled')}>
          {formatTokens(output)}
          {overrides.includes('maxOutputTokens') ? <i className='cap-mark' title={t('已覆盖上游值')} /> : null}
        </span>
      </Button>
    )
  }

  /**
   * ****（「 not is up to 」 that ； up / not ）：
   * 1 can can not ，， not is 1 「
   * models.dev」。
   */
  function tokenOriginSuffix(model: ManageModel): string {
    const filled = (['maxInputTokens', 'maxOutputTokens'] as const)
      .filter(key => capSourceOf(model, key) === 'modelsdev')
      .map(key => CAPABILITY_LABELS[key])
    return filled.length ? t('；其中 {keys} 由 models.dev 补齐', { keys: filled.join(t('、')) }) : ''
  }

  /**
   * 「 can 」：（ / / / ）， has 1 ——
   * supports（）/ unsupported（）/ （）。
   * （`capabilityTip` will 「 up when down by unsupported」 this ），
   * 「 1 has is 」。
   *
   * 1 （[`imageCapsCell`]）： that is ** for ** can ，
   * in up is 「unsupported」，「 this not will 」。
   */
  function capsCell(model: ManageModel) {
    if (model.kind === 'image') return imageCapsCell(model)
    const caps = capabilitiesOf(model)
    const overrides = normalizeOverrides(model.capOverrides)
    return (
      <Button size='default' variant='ghost' type='button' className='caps-open caps-badges'
        title={t('对下游声明的能力（工具调用 / 图片识别 / 视频识别 / 支持思考）—— 点击修改')}
        onClick={() => openCapability(model.provider || '', model.id)}>
        {BOOLEAN_KEYS.map(key => {
          const value = caps[key] ?? null
          const state = capabilityState(value)
          const overridden = overrides.includes(key)
          const source = capSourceOf(model, key)
          return (
            <span key={key} className={cn('cap-dot', state, source === 'modelsdev' && 'filled', overridden && 'marked')}
              title={capabilityTip(key, value, overridden, source)}>
              {CAP_ICON[key]}
              {overridden ? <i className='cap-mark' /> : null}
            </span>
          )
        })}
      </Button>
    )
  }

  /**
   * can ：（ / ） up down 、 in 。
   *
   * ── use can ， not use `kind` ──────────────────────────────
   * 「」 in `supports_image_generation` as when ： has 
   * （ ModelScope Qwen-Image-Edit） not this —— before that 
   * in ID after ，`kind === 'image'` 「」， this 1 。
   * can no when 「」（ and before 1 ）， not this 1 empty by 。
   *
   * ── Why is ────────────────────────────────────────
   * 「 can 」 has 11.5% ， will this 1 （ and ）；
   * is that ， not use 。 and in in CSS（`.caps-open.caps-image`）。
   */
  function imageCapsCell(model: ManageModel) {
    const tags: Array<{ key: string; label: string; title: string }> = []
    if (model.supports_image_generation) {
      tags.push({ key: 'generate', label: t('图片生成'), title: 'POST /v1/images/generations' })
    }
    if (model.supports_image_editing) {
      tags.push({ key: 'edit', label: t('图片编辑'), title: 'POST /v1/images/edits' })
    }
    if (!tags.length) {
      tags.push({ key: 'generate', label: t('图片生成'), title: 'POST /v1/images/generations' })
    }
    return (
      <Button size='default' variant='ghost' type='button' className='caps-open caps-image'
        title={t('这个模型的图片能力（图片生成 / 图片编辑）—— 点击修改')}
        onClick={() => openCapability(model.provider || '', model.id)}>
        {tags.map(tag => (
          <span key={tag.key} className='binding-default' title={tag.title}>{tag.label}</span>
        ))}
      </Button>
    )
  }

  /**
   * （ not for ）。 in 1 switch not is ， is as 「 1 
   * 」 has 1 —— set when not will 1 。
   *
   * 「 can 」： is that （source=manual， up directory by 
   * in ， is ）， 1 is use 、 can 。
   * for 1 in 。
   */
  function cellFor(column: ColumnView, model: ManageModel, busyRow: boolean) {
    /**
     * 「」 is that ****（ in by this ， no has 「 up new will 」
     * this ， after /v1/models、 by when ）。 use after to source， before not 。
     */
    async function confirmRemoveModel(): Promise<void> {
      const ok = await shared().wbConfirm?.ask?.({
        title: t('移除自定义模型'),
        html: t('确定移除自定义模型「<strong>{id}</strong>」？这条登记会被<b>直接移除</b>，之后 <code>/v1/models</code> 不再广告它、请求它也会被拒。', { id: esc(model.id) }),
        okText: t('移除'),
        okClass: 'danger',
      })
      if (!ok) return
      void runRowAction(
        rowKeyOf(model),
        () => writeRemoveModel(model.provider || '', model.id),
        // by **** （ in 1 up ， custom not use ）
        customSource.isCustom(model.provider || '') ? t('模型已移除') : t('自定义模型已移除'),
      )
    }

    switch (column.key) {
      case 'check':
        return (
          <TableCell className={cellClass('cell-check', column.align)}>
            <Checkbox checked={selection.has(rowKeyOf(model))} title={t('勾选后可批量操作')}
              aria-label={t('勾选后可批量操作')}
              onCheckedChange={next => togglePick(rowKeyOf(model), next === true)} />
          </TableCell>
        )
      case 'model':
        return (
          <TableCell className={cellClass('cell-model', column.align)}>
            {/* is （data-copy clipboard.js ， 1 down 
                toast「 already 」）；title when —— can 、 ID 
                ellipsis when can 。 this 1 ⧉ by ， already ：
                ， 1 */}
            <div className='mid'>
              <span className='t' data-copy={model.id} title={t('点击复制：{name}', { name: model.id })}>{model.id}</span>
              {model.upstreamStatus === 'undeployed' ? (
                <Badge variant='warning' shape='tag'
                  title={t('上游目录列出了这个模型，但推理端尚未识别；保留在清单中，后续探测成功后自动清除标记')}>
                  {t('上游未部署')}
                </Badge>
              ) : null}
            </div>
            {model.name && model.name !== model.id ? <div className='mname'>{model.name}</div> : null}
          </TableCell>
        )
      case 'rate':
        return (
          <TableCell className={cellClass('cell-rate', column.align)}>
            <span className='rate'>{model.credits ? formatCredits(model.credits) : '—'}</span>
          </TableCell>
        )
      case 'source':
        return <TableCell className={cellClass('cell-source', column.align)}>{sourceCell(model)}</TableCell>
      case 'visibility':
        return <TableCell className={cellClass('cell-visibility', column.align)}>{visibilityCell(model)}</TableCell>
      case 'budget':
        return <TableCell className={cellClass('cell-budget', column.align)}>{budgetCell(model)}</TableCell>
      case 'caps':
        return <TableCell className={cellClass('cell-caps', column.align)}>{capsCell(model)}</TableCell>
      case 'alias':
        return <TableCell className={cellClass('cell-alias', column.align)}>{aliasCell(model)}</TableCell>
      case 'act': {
        // 「」 （ this no has available） by model-test-dialog 1 ， this 
        // by to title up —— use not Reason etc use 。enabled ** not **：
        // 「、 need not need enabled」 is this by use （ after to ，
        // model-test-dialog testBlockReason）
        const blocked = testBlockReason(model.provider || '', model)
        // by by ****（ and in 1 up ， 
        // custom not use ）： （source=manual， up directory 
        // by in ）， 1 is use 、 can 
        const removable = model.source === 'manual' || customSource.isCustom(model.provider || '')
        return (
          <TableCell className={cellClass('cell-act r', column.align)}>
            <div className='row-actions'>
              <Button variant='ghost' size='sm' disabled={Boolean(blocked)}
                title={blocked || t('以这个上游模型发一次最小请求，走真实转发链路')}
                onClick={() => setTestTarget({ provider: model.provider || '', id: model.id })}>
                {t('测试')}
              </Button>
              {removable ? (
                <Button variant='ghost' size='sm' className='text-destructive' disabled={busyRow}
                  onClick={() => void confirmRemoveModel()}>{t('移除')}</Button>
              ) : null}
            </div>
          </TableCell>
        )
      }
      default:
        return null
    }
  }

  return <>{columns.map(column => <React.Fragment key={column.key}>{cellFor(column, model, busyRow)}</React.Fragment>)}</>
}
