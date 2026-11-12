/**
 * ****（ + 「」 by ）。
 *
 * ： if after ，「get」 「」 by （models-page.tsx）。
 * can for in ：enabled / use / set etc / delete。 before for ** 1 **
 * （default + chips） —— 「 already enabled」 is 「 has 1 」
 * （`rowEnabled`）， can enabled / use ； etc by 
 * （`ManageMapping.reasoning`），set / to up 。
 *
 * ── delete （ and that 「」 1 ）────────────────────
 * has **** （ `source === 'manual'`、 1 ） in 
 * 「delete」 this ； up directory / by in ， is 
 * 。delete will not can delete and ， to 1 。
 *
 * ── is ─────────────────────────────────────────
 * return** new **（`accept` ）；
 * is 。 in ， 1 failure down when up is 「success that 
 * 1 already 」 ， not will 。
 *
 * ── failure ─────────────────────────────────────────────
 * success → 、empty； has failure → 、（ can ），
 * failure and failureReason。
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

/** （RadioGroup ； and use this ） */
const ACTIONS = [
  { value: 'enable', label: t('启用') },
  { value: 'disable', label: t('禁用') },
  { value: 'reasoning', label: t('设置思考等级') },
  { value: 'delete', label: t('删除') },
] as const

type BatchAction = (typeof ACTIONS)[number]['value']

/** 「delete」 ：（ manual） or 1 （file） */
function isRemovable(model: ManageModel): boolean {
  return model.source === 'manual' || isCustom(model.provider)
}

/** （ and 1 ： has use ， no has id） */
function modelName(model: ManageModel): string {
  return (model.name && model.name !== model.id ? t('{name}（{id}）', { name: model.name, id: model.id }) : model.id) || model.id
}

export function ModelBatchDialog({ models, onClose, onDone }: {
  models: ManageModel[]
  onClose: () => void
  onDone: () => void
}) {
  const [action, setAction] = React.useState<BatchAction>('enable')
  /**
   * etc （ MappingDialog ）：'' = / not 、 etc 、CUSTOM_LEVEL。
   *
   * available 。 use use ， not can 
   * not ； no has when ， use supports 。
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
  const label = ACTIONS.find(item => item.value === action)?.label || t('操作')

  /** delete not can ， by （ and delete 1 ， confirm in 
   * Tauri WebView ， 1 wbConfirm use ） */
  async function confirmRemove(count: number): Promise<boolean> {
    return Promise.resolve(shared().wbConfirm?.ask?.({
      title: t('删除选中的模型'),
      html: t('确定删除选中的 <strong>{count}</strong> 个模型登记？删除后 <code>/v1/models</code> 不再广告它们、请求也会被拒。', { count })
        + (models.length > count ? '<br/>' + t('另有 {n} 个非手动登记的模型将被跳过。', { n: models.length - count }) : ''),
      okText: t('删除'),
      okClass: 'danger',
    }) ?? false)
  }

  async function run(): Promise<void> {
    if (busy) return
    if (!models.length) { toast(t('没有选中的模型'), 'err'); return }
    if (action === 'enable') {
      const selected = new Map<string, string>()
      for (const model of models) {
        for (const binding of bindingsOf(model)) {
          const key = JSON.stringify([(model.provider || '').toLowerCase(), binding.alias.toLowerCase()])
          const previous = selected.get(key)
          if (previous && !same(previous, model.id)) {
            setResult(<span className='text-destructive'>{t('所选模型含同一提供商的同名绑定「{id}」，请分别启用，避免相互关闭', { id: binding.alias })}</span>)
            return
          }
          selected.set(key, model.id)
        }
      }
    }
    if (action === 'reasoning' && reasoning.select === CUSTOM_LEVEL && !level) {
      setResult(<span className='text-destructive'>{t('请填写自定义思考等级，或改选候选档 / 「清除」')}</span>)
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
        setResult(<span className='text-destructive'>{t('所选模型没有共同可用档位，请分别设置或清除绑定。')}</span>)
        return
      }
    }
    if (action === 'delete') {
      if (removableCount === 0) {
        setResult(<span className='text-destructive'>{t('选中的模型都不是手动登记，无法删除（开关映射才是它们的手段）')}</span>)
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
          {t('失败 {n} 个 · 首个失败：{error}', { n: failed, error: firstError })}
        </span>)
      }
    }
    if (failed === 0) {
      toast(skipped
        ? t('✅ 批量{action}完成：共 {n} 个模型（跳过不可删除的 {skipped} 个）',
          { action: label, n: ok, skipped })
        : t('✅ 批量{action}完成：共 {n} 个模型', { action: label, n: ok }))
      onDone()
    } else {
      toast(t('批量{action}完成：成功 {ok} 个，失败 {failed} 个', { action: label, ok, failed }), 'err')
      setResult(<span className='text-destructive'>
        {t('成功 {ok} 个，失败 {failed} 个', { ok, failed })}
        {skipped ? t('，跳过 {n} 个', { n: skipped }) : ''}
        {firstError ? t('；首个失败：{error}', { error: firstError }) : ''}
      </span>)
    }
    setBusy(false)
  }

  return (
    <Dialog open onOpenChange={(next, details) => { if (next) return; if (busy) { details.cancel(); return }; onClose() }}>
      <DialogContent>
        <DialogHeader><DialogTitle>{t('批量操作 · 已选 {n} 个模型', { n: models.length })}</DialogTitle></DialogHeader>
        <DialogBody>
          <DialogSection>
            <h3>{t('将作用于以下模型')}</h3>
            <p style={{ maxHeight: 84, overflowY: 'auto' }}>{models.map(modelName).join(t('、'))}</p>
          </DialogSection>
          <DialogSection>
            <h3>{t('操作')}</h3>
            <RadioGroup value={action} onValueChange={next => setAction(next as BatchAction)}
              className='flex-row flex-wrap items-center gap-5' aria-label={t('批量动作')}>
              {ACTIONS.map(item => (
                <Label key={item.value} className='inline-flex cursor-pointer items-center gap-2 font-normal'>
                  <RadioGroupItem value={item.value} />{item.label}
                </Label>
              ))}
            </RadioGroup>
            {action === 'reasoning' ? (
              <div className='mt-2.5'>
                <Field orientation='horizontal' className='field-row'>
                  <FieldLabel htmlFor='batch-model-reasoning'>{t('思考等级')}</FieldLabel>
                  <Select value={reasoning.select} onValueChange={next => setReasoning(prev => ({ ...prev, select: String(next) }))}>
                    <SelectTrigger id='batch-model-reasoning' className='min-w-[220px]'>
                      <SelectValue>
                        {reasoning.select === CUSTOM_LEVEL ? t('自定义等级…')
                          : (reasoning.select || t('清除（不覆盖）'))}
                      </SelectValue>
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value=''>{t('清除（不覆盖）')}</SelectItem>
                      {candidates.map(levelName => (
                        <SelectItem key={levelName} value={levelName}>{levelName}</SelectItem>
                      ))}
                      {allowsCustomLevel
                        ? <SelectItem value={CUSTOM_LEVEL}>{t('自定义等级…')}</SelectItem> : null}
                    </SelectContent>
                  </Select>
                </Field>
                {reasoning.select === CUSTOM_LEVEL ? (
                  <Field orientation='horizontal' className='field-row mt-2'>
                    <FieldLabel htmlFor='batch-model-reasoning-custom'>{t('自定义等级')}</FieldLabel>
                    <Input id='batch-model-reasoning-custom' maxLength={32} value={reasoning.custom}
                      onChange={event => setReasoning(prev => ({ ...prev, custom: event.currentTarget.value }))} />
                  </Field>
                ) : null}
                {!candidates.length && managedReasoning
                  ? <p className='detail mt-1.5'>{t('所选模型没有共同可用档位，请分别设置或清除绑定。')}</p> : null}
                <p className='detail mt-1.5'>
                  {t('设置 / 清除会应用到选中模型的')}<b>{t('全部映射')}</b>{t('（默认绑定与别名 chips）；「清除」= 不给这些映射指定等级（选「不覆盖」的口径与映射弹窗一致）')}
                </p>
              </div>
            ) : null}
            {action === 'delete' ? (
              <p className='detail mt-2'>
                {removableCount === 0
                  ? t('选中的都不是手动登记的模型，删除不可用（上游目录带回来的行由清单决定存在性）')
                  : t('可删除 {n} 个（手动登记 / 自定义家）；', { n: removableCount })
                    + (models.length > removableCount
                      ? t('其余 {n} 个会被跳过。', { n: models.length - removableCount })
                      : '')}
              </p>
            ) : null}
          </DialogSection>
          <div className='detail' style={{ minHeight: 18 }}>{result}</div>
        </DialogBody>
        <DialogFooter>
          <div className='mr-auto' />
          <Button variant='outline' disabled={busy} onClick={onClose}>{t('关闭')}</Button>
          <Button variant={action === 'delete' ? 'destructive' : 'default'} disabled={busy} onClick={() => void run()}>
            {busy ? t('执行中…') : t('执行{action}', { action: label })}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
