import * as React from 'react'
import {
  Button, Dialog, DialogBody, DialogContent, DialogFooter, DialogHeader, DialogTitle,
  Input, Label, Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
  Tooltip, TooltipContent, TooltipTrigger,
} from '@ui'
import * as customSource from './models-custom-source'
import type { ManageView } from './models-custom-source'
import { CUSTOM_LEVEL, bindingReasoningLevel, modelReasoning, levels as reasoningLevels } from './models-reasoning'
import {
  accept, addMappingError, addModelError, customProviderOptions, errorMessage, getSnapshot,
  levelOf, modelRowOf, providerOptions, shared, toast, upstreamOptions, writeAddModel, writeBinding,
  type CustomModelContext, type MappingContext,
} from './models-panel-state'
import { t } from '../i18n'

/* ─── ───────────────────────────────── */

/** etc in clientparameter when as default。 */
const REASONING_TIP = t('思考等级绑定是这条映射的默认值，客户端显式指定时不覆盖。WorkBuddy 按当前地区的上游目录注入 reasoning_effort；不支持的正向等级取不高于请求的最高可用档，固定档位不能调高，目录未声明时跳过。off / none 不由映射注入；自定义等级只有提供商能确认支持时才发送。注入、归一与跳过原因写入详细日志。')

/** up down ： has use （value must and option will in ） */
function pickUpstream(providerId: string, keep: string, locked: boolean): string {
  const options = upstreamOptions(providerId)
  const wanted = (keep || '').trim()
  const hit = options.find(item => item.id.toLowerCase() === wanted.toLowerCase())
  if (hit) return hit.id
  // **** keep： target not in this （ that is not up Reason），
  // will 「set etc 」 when 1 ， etc to up not 
  if (locked && wanted) return wanted
  return options[0]?.id || ''
}

/**
 * 。 —— 「＋ 」 new 、 chip etc —— context
 * total is this 1 ： and up （ 1 ），`context.alias` has when 
 * for ， etc can （「set etc 」）。
 *
 * up ** can is down **（ this current）。 this 「 up ID」，
 * already delete： for has in already when will ， as ——
 * 1 no has ， not 。 need use ， is 
 * 「＋ 」 this ， after in this down 。
 */
export function MappingDialog({ context, onClose }: { context: MappingContext; onClose: () => void }) {
  const editing = Boolean(context.alias)
  const [provider, setProvider] = React.useState(() => context.provider || providerOptions()[0]?.id || '')
  const [alias, setAlias] = React.useState(context.alias || '')
  // use up that provider（context.provider as empty when already to ），must
  const [upstream, setUpstream] = React.useState(() => pickUpstream(provider, context.target, true))
  const [status, setStatus] = React.useState('')
  const [saving, setSaving] = React.useState(false)
  /**
   * etc ：`select` is down current（'' = not / etc / CUSTOM_LEVEL），`custom` is 
   * 。keep not in （ use up 、 or after ） when to 
   * 「」 that 1 and —— will 「 when etc 」。
   *
   * by **current in **（ `models-reasoning.levels`）： etc in 
   * A up 、 up B，B not this when to 「」， not 。
   */
  const [reasoning, setReasoning] = React.useState(() => {
    const candidates = reasoningLevels(getSnapshot().data, modelRowOf(provider, upstream))
    const current = editing ? levelOf(context.alias, context.target, context.provider) : ''
    if (current && candidates.includes(current)) return { select: current, custom: '' }
    return { select: current ? CUSTOM_LEVEL : '', custom: current }
  })

  const targetModel = modelRowOf(provider, upstream)
  const reasoningInfo = modelReasoning(targetModel)
  const candidates = reasoningLevels(getSnapshot().data, targetModel)
  const allowsCustomLevel = reasoningInfo.bindingSupported === null
  /** ： ， up up down that 1 （ can can 「 no 」 ） */
  const providerChoices = (() => {
    const options = providerOptions()
    for (const id of [context.provider, provider]) {
      if (id && !options.some(item => item.id === id)) {
        options.push({ id, label: t(shared().wbProviders?.labelOf?.(id) || id) })
      }
    }
    return options
  })()
  /** up ： down keep not in when （ pickUpstream ） */
  const upstreamChoices = (() => {
    const options = upstreamOptions(provider)
    const wanted = (upstream || '').trim()
    if (wanted && !options.some(item => item.id.toLowerCase() === wanted.toLowerCase())) {
      options.unshift({ id: wanted, label: t('{name}（不在该家当前清单里）', { name: wanted }), off: true })
    }
    return options
  })()

  const providerLabel = providerChoices.find(item => item.id === provider)?.label || provider || t('(全局)')
  /**
   * use etc ： already not in current（ up / after ） when by 「」
   * —— and 1 then ， in not is （， not state）。
   */
  const active = reasoning.select && reasoning.select !== CUSTOM_LEVEL && !candidates.includes(reasoning.select)
    ? { select: CUSTOM_LEVEL, custom: reasoning.custom || reasoning.select }
    : reasoning
  const level = active.select === CUSTOM_LEVEL ? active.custom.trim() : active.select
  const showCustomLevel = active.select === CUSTOM_LEVEL
  const effectiveLevel = reasoningInfo.bindingSupported === true
    ? bindingReasoningLevel(level, targetModel) : ''
  const reasoningNotice = reasoningInfo.bindingSupported === false
    ? t('当前通道未声明可用档位，映射等级不会注入；请先获取模型目录。')
    : reasoningInfo.fixedLevel
      ? t('上游固定为 {level}，绑定其他等级也无法调高。', { level: reasoningInfo.fixedLevel })
      : effectiveLevel && effectiveLevel.toLowerCase() !== level.toLowerCase()
        ? t('绑定 {requested}，按当前通道将发送 {effective}。', { requested: level, effective: effectiveLevel })
        : reasoningInfo.source === 'modelsdev'
          ? t('社区参考档位，不代表当前上游通道支持；实际以该通道目录为准。')
          : reasoningInfo.declared && reasoningInfo.levels.length
            ? t('上游可用档位：{levels}', { levels: reasoningInfo.levels.join(' / ') })
            : ''

  /** 1 ， up （， when not use ） */
  function changeProvider(next: string): void {
    setProvider(next)
    setUpstream(pickUpstream(next, upstream, true))
  }

  const duplicateError = !editing && alias.trim() ? addMappingError(provider, alias.trim(), upstream) : ''
  async function save(): Promise<void> {
    if (saving) return
    const wanted = editing ? (context.alias || '') : alias.trim()
    const target = upstream
    if (!wanted) { setStatus(t('请填写对外映射名')); return }
    // down as empty = this 1 1 no has （ no / no to ）
    if (!target) { setStatus(t('该提供商当前没有可选的上游模型')); return }
    if (!provider) { setStatus(t('请选择提供商')); return }
    if (level && reasoningInfo.bindingSupported === true && !effectiveLevel) {
      setStatus(t('该等级无法注入，请选择上游可用档位或清除绑定。'))
      return
    }
    if (!editing) {
      const error = addMappingError(provider, wanted, target)
      if (error) { setStatus(error); return }
    }
    setSaving(true)
    setStatus(t('保存中…'))
    try {
      // `reasoning` **total is to **（empty = empty）： is this ， use 
      // in this to is need —— undefined（= not ） will 「 from high not 」
      // this 1 invalid
      accept(await writeBinding(provider, wanted, target, { reasoning: level, create: !editing }))
      setSaving(false)
      onClose()
      const suffix = level ? t(' · 思考等级 {level}', { level }) : ''
      // / directory： provider id when ， will 1 
      // custom-3f2a91b04c7e， use not is 1 
      const label = t(shared().wbProviders?.labelOf?.(provider) || provider)
      toast(editing ? t('✅ 已更新 {name} 的思考等级', { name: wanted }) : t('✅ 已添加映射 {name} → {target}（{label}）{suffix}', { name: wanted, target, label, suffix }))
    } catch (error) {
      setStatus(t('保存失败：{message}', { message: errorMessage(error) }))
      setSaving(false)
    }
  }

  const onEnter = (event: React.KeyboardEvent) => {
    if (event.key !== 'Enter') return
    event.preventDefault()
    void save()
  }

  return (
    <Dialog open onOpenChange={(next, eventDetails) => {
      if (next) return
      // in not ： will 「 to no 」unknown。must eventDetails.cancel()
      // —— 「 not update open prop」 not Base UI store
      if (saving) { eventDetails.cancel(); return }
      onClose()
    }}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{editing ? t('设置思考等级') : t('添加模型映射')}</DialogTitle>
        </DialogHeader>
        <DialogBody>
          <div className='flex flex-col gap-1.5'>
            <Label htmlFor='mapping-provider'>{t('提供商')}</Label>
            {/* = ： and up is this 1 ， not */}
            <Select value={provider} disabled onValueChange={next => changeProvider(String(next))}>
              <SelectTrigger id='mapping-provider' className='w-full'>
                <SelectValue>{providerChoices.find(item => item.id === provider)?.label || provider}</SelectValue>
              </SelectTrigger>
              <SelectContent>
                {providerChoices.map(item => (
                  <SelectItem key={item.id} value={item.id}>{item.label}</SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className='grid grid-cols-[1fr_24px_1fr] items-end gap-2.5'>
            <div className='flex min-w-0 flex-col gap-1.5'>
              <Label htmlFor='mapping-alias'>{t('对外映射名（下游请求时用）')}</Label>
              <Input id='mapping-alias' maxLength={128} placeholder={t('例如 gpt-4o')} autoComplete='off'
                autoFocus={!editing} value={alias} disabled={editing}
                onChange={event => setAlias(event.currentTarget.value)} onKeyDown={onEnter} />
            </div>
            <div className='pb-2 text-center text-muted-foreground'>→</div>
            <div className='flex min-w-0 flex-col gap-1.5'>
              <Label htmlFor='mapping-upstream'>{t('转发到上游模型')}</Label>
              <Select value={upstream} disabled onValueChange={next => setUpstream(String(next))}>
                <SelectTrigger id='mapping-upstream' className='w-full'>
                  <SelectValue>{upstreamChoices.find(item => item.id === upstream)?.label || upstream}</SelectValue>
                </SelectTrigger>
                <SelectContent>
                  {upstreamChoices.map(item => (
                    <SelectItem key={item.id} value={item.id}>
                      {item.label}{item.off ? t('（已禁用）') : ''}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>
          {/* ：current will 1 （ etc 1 ， before can for ） */}
          <div className='rounded-md border border-border bg-surface-inset px-3 py-2.5 font-mono text-[12px] text-subtle'>
            {t('下游请求 ')}<b className='text-primary-fg'>{alias.trim() || t('<对外名>')}</b>{t(' → 转发')}{' '}
            <b className='text-primary-fg'>{upstream || t('<上游模型>')}</b>{t('（{label}）', { label: providerLabel })}
            {level ? <>{t(' · 思考等级 ')}<b className='text-primary-fg'>{level}</b></> : null}
          </div>
          <div className='flex flex-col gap-1.5'>
            <div className='flex items-center gap-[7px]'>
              <Label htmlFor='mapping-reasoning'>{t('思考等级')}</Label>
              {/* Tooltip（ old data-tip + desktop-tooltip.tsx is 1 ） */}
              <Tooltip>
                <TooltipTrigger render={<span className='tip-q' tabIndex={0} aria-label={t('思考等级的注入规则')} />}>?</TooltipTrigger>
                <TooltipContent>{REASONING_TIP}</TooltipContent>
              </Tooltip>
            </div>
            <Select value={active.select} onValueChange={next => {
              const value = String(next)
              setReasoning(prev => ({ ...prev, select: value }))
            }}>
              {/* etc no can ， in etc down up （ old is setTimeout focus） */}
              <SelectTrigger id='mapping-reasoning' className='w-full' autoFocus={editing}>
                <SelectValue>{active.select === CUSTOM_LEVEL
                  ? t('自定义等级…')
                  : (active.select || t('不覆盖'))}</SelectValue>
              </SelectTrigger>
              <SelectContent>
                <SelectItem value=''>{t('不覆盖')}</SelectItem>
                {candidates.map(candidate => (
                  <SelectItem key={candidate} value={candidate}>{candidate}</SelectItem>
                ))}
                {allowsCustomLevel || showCustomLevel
                  ? <SelectItem value={CUSTOM_LEVEL}>{t('自定义等级…')}</SelectItem> : null}
              </SelectContent>
            </Select>

            {showCustomLevel ? (
              <Input id='mapping-reasoning-custom' maxLength={32} autoFocus
                placeholder={t('自定义等级（例如 custom-high）')} autoComplete='off'
                disabled={saving} value={active.custom}
                onChange={event => setReasoning(prev => ({ ...prev, custom: event.currentTarget.value }))}
                onKeyDown={onEnter} />
            ) : null}
            {reasoningNotice ? <p className='text-xs leading-[1.65] text-subtle'>{reasoningNotice}</p> : null}
            {reasoningInfo.defaultLevel && !reasoningInfo.fixedLevel
              ? <p className='text-xs text-subtle'>{t('默认档位：{level}', { level: reasoningInfo.defaultLevel })}</p> : null}
          </div>
          <p className='text-xs leading-[1.65] text-subtle'>
            {t('对外名可以是中文、字母、数字与 ')}<code>- _ . / :</code>{t(' 这类符号，但不能含空格或换行，且不超过 128 个字符。同一提供商内，同名绑定最多启用一条；已有启用绑定时不能新增，启用备用绑定会自动关闭其他同名绑定。不同提供商可各启用一条同名绑定，用于主备切换。')}
          </p>
          {/* ：，error when not */}
          <div className='min-h-[18px] text-xs text-subtle' role='status'>{duplicateError || status}</div>
        </DialogBody>
        <DialogFooter>
          <div className='mr-auto' />
          <Button variant='outline' disabled={saving} onClick={onClose}>{t('取消')}</Button>
          <Button variant='default' disabled={saving || Boolean(duplicateError)} onClick={() => void save()}>
            {editing ? t('保存等级') : t('保存映射')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

/* ─── ───────────────────────────── */

/**
 * 「＋ 」： 1 「 up directory no has 、 can by 」 （ in new 、 by 
 * down no directory ）。 after ** this **（ after in `catalog::manifest_for`
 * ）， is this 1 、`/v1/models` will 、 by and —— and 
 * can （ use none from that ， 1 etc for down ）。
 */
export function CustomModelDialog({ initial, onClose }: { initial: CustomModelContext; onClose: () => void }) {
  const [provider, setProvider] = React.useState(initial.provider)
  const [id, setId] = React.useState('')
  const [status, setStatus] = React.useState('')
  const [saving, setSaving] = React.useState(false)

  const choices = (() => {
    const options = customProviderOptions()
    if (provider && !options.some(item => item.id === provider)) {
      options.push({ id: provider, label: t(shared().wbProviders?.labelOf?.(provider) || provider) })
    }
    return options
  })()
  // no has current when （ select 1 not in is this ）
  const providerValue = choices.some(item => item.id === provider) ? provider : (choices[0]?.id || '')
  const providerLabel = choices.find(item => item.id === providerValue)?.label || providerValue || t('(未选)')
  const duplicateError = id.trim() ? addModelError(providerValue, id.trim()) : ''

  async function save(): Promise<void> {
    if (saving) return
    const value = id.trim()
    if (!providerValue) { setStatus(t('请选择提供商')); return }
    if (!value) { setStatus(t('请填写上游模型 ID')); return }
    const error = addModelError(providerValue, value)
    if (error) { setStatus(error); return }
    setSaving(true)
    setStatus(t('保存中…'))
    try {
      const next = await writeAddModel(providerValue, value)
      accept(next)
      setSaving(false)
      onClose()
      const label = choices.find(item => item.id === providerValue)?.label || providerValue
      if (customSource.isCustom(providerValue)) {
        // is use ， in ， has 1 success
        toast(t('✅ 已登记模型 {name}（{label}）', { name: value, label }))
      } else {
        // ：success not to this 1 when ，mustWhy —— 「current has available
        // 」 ， to 1 no not will 。 not use will as no success
        const view = next as ManageView | null
        const visible = Array.isArray(view?.models)
          && view.models.some(item => item.id === value && (item.provider || '') === providerValue)
        if (visible) toast(t('✅ 已登记自定义模型 {name}（{label}）', { name: value, label }))
        else toast(t('✅ 已登记 {name}（{label}），但该提供商还没有可用账号，这一行要加上账号后才会显示', { name: value, label }), 'err')
      }
    } catch (error) {
      setStatus(t('保存失败：{message}', { message: errorMessage(error) }))
      setSaving(false)
    }
  }

  const onEnter = (event: React.KeyboardEvent) => {
    if (event.key !== 'Enter') return
    event.preventDefault()
    void save()
  }

  return (
    <Dialog open onOpenChange={(next, eventDetails) => {
      // in not ： will 「 to no 」unknown
      if (next) return
      if (saving) { eventDetails.cancel(); return }
      onClose()
    }}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t('添加自定义模型')}</DialogTitle>
        </DialogHeader>
        <DialogBody>
          <div className='flex flex-col gap-1.5'>
            <Label htmlFor='custom-model-provider'>{t('提供商')}</Label>
            <Select value={providerValue} disabled={initial.locked}
              onValueChange={next => setProvider(String(next))}>
              <SelectTrigger id='custom-model-provider' className='w-full'>
                <SelectValue>{providerLabel}</SelectValue>
              </SelectTrigger>
              <SelectContent>
                {choices.map(item => (
                  <SelectItem key={item.id} value={item.id}>{item.label}</SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className='flex flex-col gap-1.5'>
            <Label htmlFor='custom-model-id'>{t('上游模型 ID')}</Label>
            <Input id='custom-model-id' maxLength={128} placeholder={t('例如 gpt-5.5-preview')}
              autoComplete='off' spellCheck={false} autoFocus value={id}
              onChange={event => setId(event.currentTarget.value)} onKeyDown={onEnter} />
          </div>
          <div className='rounded-md border border-border bg-surface-inset px-3 py-2.5 font-mono text-[12px] text-subtle'>
            {t('在 ')}<b className='text-primary-fg'>{providerLabel}</b>{t(' 上登记上游模型')}{' '}
            <b className='text-primary-fg'>{id.trim() || t('<上游模型 ID>')}</b>{t('（登记后即可用这个名字请求）')}
          </div>
          <p className='text-xs leading-[1.65] text-subtle'>
            {t('填上游真正认识的那个模型 ID（不是给下游用的名字）。登记后它会进入该家的模型清单： 表格里出现这一行、')}<code>/v1/models</code>{t(' 会广告它、请求它也会被转发到这家。 支持字母、数字与 ')}<code>- _ . / :</code>{t('。')}
          </p>
          <p className='text-xs leading-[1.65] text-subtle'>
            {t('「来源」列会标成')}<b>{t('手动')}</b>{t('，与远程目录 / 内置清单区分开。删除自定义模型是')}<b>{t('直接移除')}</b>{t('这条登记（不像内置模型那样只是隐藏），因为它的存在完全由这次登记决定。')}
          </p>
          <div className='min-h-[18px] text-xs text-subtle' role='status'>{duplicateError || status}</div>
        </DialogBody>
        <DialogFooter>
          <div className='mr-auto' />
          <Button variant='outline' disabled={saving} onClick={onClose}>{t('取消')}</Button>
          <Button variant='default' disabled={saving || Boolean(duplicateError)} onClick={() => void save()}>{t('保存')}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

