/**
 * 模型管理页的**批量操作弹窗**（表头勾选列 + 「批量操作」按钮的落地动作）。
 *
 * 入口：勾选若干行后，「获取模型」左边的「批量操作」按钮（models-page.tsx）。
 * 可对选中的行做：启用 / 禁用 / 设置思考等级 / 删除。前三个对**每一行的全部映射**
 * （默认绑定 + 别名 chips）生效 —— 行级「已启用」的判定就是「还有任一条映射开着」
 * （`rowEnabled`），所以批量开关映射才能让整行真的启用 / 禁用；思考等级同样按
 * 映射记（`ManageMapping.reasoning`），设置 / 清除必然落到每条映射上。
 *
 * ── 删除的边界（与操作列那颗「移除」同一条）────────────────────
 * 只有**手动登记**的模型（内置家的 `source === 'manual'`、自定义家的一切）才存在
 * 「删除登记」这回事；上游目录带回来的远程 / 内置行由清单决定存在性，开关映射才是
 * 它的手段。批量删除会跳过不可删除的行并计数，弹窗里先给一行说明。
 *
 * ── 执行是逐条串行 ─────────────────────────────────────────
 * 内置家的每个写操作都返回**最新整份视图**（`accept` 就地替换）；自定义家每次写
 * 都是整表提交。串行执行让中间结果逐条落定，任一失败停下时界面上就是「成功的那
 * 一部分已生效」的真实状态，不会整批回滚成更早期的样子。
 *
 * ── 失败的处理 ─────────────────────────────────────────────
 * 全部成功 → 关弹窗、清空勾选；有失败 → 弹窗留着、勾选保留（可改完条件重试），
 * 结果行列出失败数与首次失败原因。
 */

import * as React from 'react'
import {
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
  RadioGroup,
  RadioGroupItem,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Field, FieldLabel,
} from '@ui'
import { isCustom, type ManageModel } from './models-custom-source'
import { CUSTOM_LEVEL, POSITIVE_REASONING_LEVELS, commonLevels, modelReasoning } from './models-reasoning'
import {
  accept, bindingsOf, errorMessage, getSnapshot, shared, toast, writeBinding, writeRemoveModel, same,
} from './models-panel-state'
import { t } from '../i18n'

/** 批量动作（RadioGroup 的取值；文案与执行动词共用这里） */
const ACTIONS = [
  { value: 'enable', label: t('enabled') },
  { value: 'disable', label: t('Disable') },
  { value: 'reasoning', label: t('Set reasoning level') },
  { value: 'delete', label: t('Delete') },
] as const

type BatchAction = (typeof ACTIONS)[number]['value']

/** 「删除登记」的边界：手动登记（内置家 manual）或自定义家的一切（见文件头） */
function isRemovable(model: ManageModel): boolean {
  return model.source === 'manual' || isCustom(model.provider)
}

/** 模型名（与账号页同一口径：有名字用名字，没有回退 id） */
function modelName(model: ManageModel): string {
  return (model.name && model.name !== model.id ? t('{name} ({id})', { name: model.name, id: model.id }) : model.id) || model.id
}

export function ModelBatchDialog({ models, onClose, onDone }: {
  models: ManageModel[]
  onClose: () => void
  onDone: () => void
}) {
  const [action, setAction] = React.useState<BatchAction>('enable')
  /**
   * 思考等级草稿（照 MappingDialog 的形态）：'' = 清除 / 不覆盖、候选等级、CUSTOM_LEVEL。
   *
   * 候选取各目标可用档位的交集。未声明的目标使用通用候选，明确不可绑定的目标
   * 不回落；没有交集时只允许清除，避免把通用档位误当成共同支持的档位。
   */
  const candidates = commonLevels(getSnapshot().data, models)
  const managedReasoning = models.some(model => typeof model.reasoningBindingSupported === 'boolean')
  const allowsCustomLevel = !managedReasoning
  const [reasoning, setReasoning] = React.useState(() => ({
    select: candidates.find(level => !['off', 'none'].includes(level.toLowerCase())) || '',
    custom: '',
  }))
  const [busy, setBusy] = React.useState(false)
  const [result, setResult] = React.useState<React.ReactNode>('')
  const level = reasoning.select === CUSTOM_LEVEL ? reasoning.custom.trim() : reasoning.select
  const removableCount = models.filter(isRemovable).length
  const label = ACTIONS.find(item => item.value === action)?.label || t('Actions')

  /** 批量删除不可逆，按数量做二次确认（与账号页批量删除同一口子，原生 confirm 在
   *  Tauri WebView 里直接放行，所以一律走 wbConfirm 的应用内确认框） */
  async function confirmRemove(count: number): Promise<boolean> {
    return Promise.resolve(shared().wbConfirm?.ask?.({
      title: t('Delete selected models'),
      html: t('Delete the <strong>{count}</strong> selected model registrations? After deletion, <code>/v1/models</code> will no longer advertise them and requests to them will be rejected.', { count })
        + (models.length > count ? '<br/>' + t('Another {n} models that aren\'t manually registered will be skipped.', { n: models.length - count }) : ''),
      okText: t('Delete'),
      okClass: 'danger',
    }) ?? false)
  }

  async function run(): Promise<void> {
    if (busy) return
    if (!models.length) { toast(t('No models selected'), 'err'); return }
    if (action === 'enable') {
      const selected = new Map<string, string>()
      for (const model of models) {
        for (const binding of bindingsOf(model)) {
          const key = JSON.stringify([(model.provider || '').toLowerCase(), binding.alias.toLowerCase()])
          const previous = selected.get(key)
          if (previous && !same(previous, model.id)) {
            setResult(<span className='text-destructive'>{t('The selected models contain bindings named “{id}” under the same provider. Enable them separately to avoid disabling one another.', { id: binding.alias })}</span>)
            return
          }
          selected.set(key, model.id)
        }
      }
    }
    if (action === 'reasoning' && reasoning.select === CUSTOM_LEVEL && !level) {
      setResult(<span className='text-destructive'>{t('Enter a custom reasoning level, or pick a candidate level / “Clear”')}</span>)
      return
    }
    if (action === 'reasoning' && level && managedReasoning) {
      const rank = POSITIVE_REASONING_LEVELS.findIndex(value => value === level.toLowerCase())
      const incompatible = models.some(model => {
        const info = modelReasoning(model)
        return info.bindingSupported === false || (info.fixedLevel && info.fixedLevel.toLowerCase() !== level.toLowerCase())
          || (rank < 0 && !info.levels.some(value => value.toLowerCase() === level.toLowerCase()))
      })
      if (incompatible) {
        setResult(<span className='text-destructive'>{t('The selected models have no shared supported level. Configure them separately or clear their bindings.')}</span>)
        return
      }
    }
    if (action === 'delete') {
      if (removableCount === 0) {
        setResult(<span className='text-destructive'>{t('None of the selected models are manually registered, so they can\'t be deleted (use the mapping toggles instead)')}</span>)
        return
      }
      if (!(await confirmRemove(removableCount))) return
    }

    setBusy(true)
    setResult('')
    let ok = 0
    let skipped = 0
    let failed = 0
    let firstError = ''
    for (const model of models) {
      const provider = model.provider || ''
      try {
        if (action === 'delete') {
          if (!isRemovable(model)) { skipped += 1; continue }
          accept(await writeRemoveModel(provider, model.id))
        } else if (action === 'enable' || action === 'disable') {
          const next = action === 'enable'
          for (const binding of bindingsOf(model)) {
            accept(await writeBinding(provider, binding.alias, model.id, { enabled: next }))
          }
        } else {
          for (const binding of bindingsOf(model)) {
            accept(await writeBinding(provider, binding.alias, model.id, { reasoning: level }))
          }
        }
        ok += 1
      } catch (error) {
        failed += 1
        if (!firstError) firstError = errorMessage(error)
        setResult(<span className='text-destructive'>
          {t('{n} failed · first failure: {error}', { n: failed, error: firstError })}
        </span>)
      }
    }
    if (failed === 0) {
      toast(skipped
        ? t('✅ Batch {action} complete: {n} models ({skipped} not deletable, skipped)',
          { action: label, n: ok, skipped })
        : t('✅ Batch {action} complete: {n} models', { action: label, n: ok }))
      onDone()
    } else {
      toast(t('Batch {action} finished: {ok} succeeded, {failed} failed', { action: label, ok, failed }), 'err')
      setResult(<span className='text-destructive'>
        {t('{ok} succeeded, {failed} failed', { ok, failed })}
        {skipped ? t(', {n} skipped', { n: skipped }) : ''}
        {firstError ? t('; first failure: {error}', { error: firstError }) : ''}
      </span>)
    }
    setBusy(false)
  }

  return (
    <Dialog open onOpenChange={(next, details) => { if (next) return; if (busy) { details.cancel(); return }; onClose() }}>
      <DialogContent>
        <DialogHeader><DialogTitle>{t('Batch actions · {n} models selected', { n: models.length })}</DialogTitle></DialogHeader>
        <DialogBody>
          <DialogSection>
            <h3>{t('Applies to the following models')}</h3>
            <p style={{ maxHeight: 84, overflowY: 'auto' }}>{models.map(modelName).join(t(', '))}</p>
          </DialogSection>
          <DialogSection>
            <h3>{t('Actions')}</h3>
            <RadioGroup value={action} onValueChange={next => setAction(next as BatchAction)}
              className='flex-row flex-wrap items-center gap-5' aria-label={t('Batch action')}>
              {ACTIONS.map(item => (
                <Label key={item.value} className='inline-flex cursor-pointer items-center gap-2 font-normal'>
                  <RadioGroupItem value={item.value} />{item.label}
                </Label>
              ))}
            </RadioGroup>
            {action === 'reasoning' ? (
              <div className='mt-2.5'>
                <Field orientation='horizontal' className='field-row'>
                  <FieldLabel htmlFor='batch-model-reasoning'>{t('Reasoning level')}</FieldLabel>
                  <Select value={reasoning.select} onValueChange={next => setReasoning(prev => ({ ...prev, select: String(next) }))}>
                    <SelectTrigger id='batch-model-reasoning' className='min-w-[220px]'>
                      <SelectValue>
                        {reasoning.select === CUSTOM_LEVEL ? t('Custom level…')
                          : (reasoning.select || t('Clear (do not override)'))}
                      </SelectValue>
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value=''>{t('Clear (do not override)')}</SelectItem>
                      {candidates.map(levelName => (
                        <SelectItem key={levelName} value={levelName}>{levelName}</SelectItem>
                      ))}
                      {allowsCustomLevel
                        ? <SelectItem value={CUSTOM_LEVEL}>{t('Custom level…')}</SelectItem> : null}
                    </SelectContent>
                  </Select>
                </Field>
                {reasoning.select === CUSTOM_LEVEL ? (
                  <Field orientation='horizontal' className='field-row mt-2'>
                    <FieldLabel htmlFor='batch-model-reasoning-custom'>{t('Custom level')}</FieldLabel>
                    <Input id='batch-model-reasoning-custom' maxLength={32} value={reasoning.custom}
                      onChange={event => setReasoning(prev => ({ ...prev, custom: event.currentTarget.value }))} />
                  </Field>
                ) : null}
                {!candidates.length && managedReasoning
                  ? <p className='detail mt-1.5'>{t('The selected models have no shared supported level. Configure them separately or clear their bindings.')}</p> : null}
                <p className='detail mt-1.5'>
                  {t('Setting / clearing applies to ')}<b>{t('all mappings')}</b>{t(' of the selected models (default binding and alias chips); “Clear” means no level is assigned to these mappings (same as “Do not override” in the mapping dialog)')}
                </p>
              </div>
            ) : null}
            {action === 'delete' ? (
              <p className='detail mt-2'>
                {removableCount === 0
                  ? t('None of the selected models are manually registered; deletion isn\'t available (rows brought back by the upstream catalog exist as long as the catalog lists them)')
                  : t('{n} can be deleted (manually registered / custom provider); ', { n: removableCount })
                    + (models.length > removableCount
                      ? t('the remaining {n} will be skipped.', { n: models.length - removableCount })
                      : '')}
              </p>
            ) : null}
          </DialogSection>
          <div className='detail' style={{ minHeight: 18 }}>{result}</div>
        </DialogBody>
        <DialogFooter>
          <div className='mr-auto' />
          <Button variant='outline' disabled={busy} onClick={onClose}>{t('Close')}</Button>
          <Button variant={action === 'delete' ? 'destructive' : 'default'} disabled={busy} onClick={() => void run()}>
            {busy ? t('Running…') : t('{action}', { action: label })}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
