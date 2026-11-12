/**
 * Agent2API · 「 can 」： 1 ** for down ** can 。
 *
 * ── ────────────────────────────────────────────────
 * down （client） by `/v1/models` `max_input_tokens` / `supports_images`
 * this request —— up directory to can can is （directory、
 * not up up ）， then is ** no has **。 before 1 
 * is ； this is that in up （ and after 
 * `core::capability` module header）。
 *
 * ── （ and 1 1 for ）───────────────────────
 * 1 is 「 / 」， and use ：
 * · （ up down / Token）： is ****，empty =
 * （placeholder to up ）。 already 「」；
 * · （ / / / ）： —— up / supports /
 *     unsupported。
 * no has 「 up 」， 「」「 / 」（
 * `custom` ）—— that can is use 。
 *
 * ── = 1 to ───────────────────────────────────
 * （ not is ） not ： use in this to is 
 * need 。 and after 1 ：`null` = / 、 has = 。
 *
 * ── ────────────────────────────────────────────
 * up down `(provider, id)`， from （`modelRowOf`）：
 * when directory new / ， new ； no 
 * （ 、 ） then to 1 and need ， not by empty。
 */

import * as React from 'react'
import {
  Button, Dialog, DialogBody, DialogContent, DialogFooter, DialogHeader, DialogTitle,
  Input, Label, SegmentedControl, type SegmentedControlOption,
} from '@ui'

import {
  BOOLEAN_KEYS, CAPABILITY_KEYS, CAPABILITY_LABELS, TOKEN_KEYS,
  capabilitiesOf, capabilityState, capSourceOf, exactTokens, normalizeCapability, normalizeOverrides,
  patchFromDraft, reasoningLevelsOf, type CapabilityDraft, type CapabilityKey, type CapabilitySource,
} from './model-capability'
import * as customSource from './models-custom-source'
import {
  accept, errorMessage, modelRowOf, providerLabelOf, toast, writeCapabilities,
  type CapabilityContext,
} from './models-panel-state'
import { t } from '../i18n'

/** （label ，`inheritLabel` by use ） */
const BOOLEAN_OPTIONS = (inheritLabel: string): SegmentedControlOption<string>[] => [
  { value: 'inherit', label: inheritLabel },
  { value: 'on', label: t('支持') },
  { value: 'off', label: t('不支持') },
]

/** （ and tooltip use ） */
function booleanText(value: number | boolean | null): string {
  const state = capabilityState(value)
  if (state === 'on') return t('支持')
  if (state === 'off') return t('不支持')
  return t('未声明')
}

/**
 * down ： this 1 **current for down ** is 、 from 。
 * 「」 need down will （ by unsupported、 by unknown），otherwise use will 
 * 「 not 」「unsupported」。
 *
 * `source` is after （`upstream` / `modelsdev` / `user`，
 * `core::model_meta`）： by models.dev that ** not can **「 up 」——
 * up no ， use as that is 。
 */
function statusText(
  key: CapabilityKey,
  value: number | boolean | null,
  set: boolean,
  custom: boolean,
  source: CapabilitySource | '' = '',
): string {
  const has = value !== null && value !== undefined
  const shown = TOKEN_KEYS.includes(key) ? exactTokens(value) : booleanText(value)
  if (set) {
    return custom
      ? t('已填写：对下游声明 {value}', { value: shown })
      : t('已覆盖：对下游声明 {value}', { value: shown })
  }
  if (custom) return t('未填写（下游按未声明处理）')
  if (!has) return t('上游未声明（下游会按不支持处理）')
  return source === 'modelsdev'
    ? t('models.dev 补齐：对下游声明 {value}', { value: shown })
    : t('继承上游：{value}', { value: shown })
}

/**
 * 「 this 1 in has no has 」 （ this is ** 1 **）：
 * · ： `capOverrides`（）—— from ，「 has no has 
 * 」 is use ；
 * · ：`capOverrides` empty（ that no has 「 up 」 this ，
 * models-custom-source ），「 this 1 no 」—— not this ，
 * use will set： as empty、 when 。
 */
function setKeysOf(capabilities: ReturnType<typeof capabilitiesOf>, overrides: CapabilityKey[], custom: boolean): CapabilityKey[] {
  if (!custom) return overrides
  return CAPABILITY_KEYS.filter(key => {
    const value = capabilities[key]
    return value !== undefined && value !== null
  })
}

/**
 * ：** already set ** to （ to ， to 'on' / 'off'），
 * its empty / 'inherit'（ placeholder will up ）。
 * and `patchFromDraft` as 。
 */
function draftFrom(capabilities: ReturnType<typeof capabilitiesOf>, setKeys: CapabilityKey[]): CapabilityDraft {
  const draft = {} as CapabilityDraft
  for (const key of CAPABILITY_KEYS) {
    const set = setKeys.includes(key)
    const value = capabilities[key]
    if (TOKEN_KEYS.includes(key)) {
      draft[key] = set && typeof value === 'number' ? String(value) : ''
    } else {
      draft[key] = set ? (value === true ? 'on' : 'off') : 'inherit'
    }
  }
  return draft
}

export function CapabilityDialog({ context, onClose }: { context: CapabilityContext; onClose: () => void }) {
  const custom = customSource.isCustom(context.provider)
  const row = modelRowOf(context.provider, context.id)
  // can （`null` = up no → that 1 not ）
  const levelsInfo = reasoningLevelsOf(row)
  const capabilities = capabilitiesOf(row)
  const setKeys = setKeysOf(capabilities, normalizeOverrides(row?.capOverrides), custom)
  const [draft, setDraft] = React.useState<CapabilityDraft>(() => draftFrom(capabilities, setKeys))
  const [status, setStatus] = React.useState('')
  const [saving, setSaving] = React.useState(false)

  /** no ： to 1 + can （ by empty this ） */
  if (!row) {
    return (
      <Dialog open onOpenChange={next => { if (!next) onClose() }}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('模型能力')}</DialogTitle>
          </DialogHeader>
          <DialogBody>
            <p className='text-sm leading-[1.7] text-subtle'>
              {t('这一行已不在当前清单里（模型被移除、或目录刷新后上游不再提供它）。关闭后刷新列表再试。')}
            </p>
          </DialogBody>
          <DialogFooter>
            <div className='mr-auto' />
            <Button variant='outline' onClick={onClose}>{t('关闭')}</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    )
  }

  // after const：`save` is （ will ），TS not in 
  // `if (!row)` —— use row.id will by `possibly null` 
  const rowId = row.id
  const providerLabel = row.providerLabel || providerLabelOf(context.provider)
  const inheritLabel = custom ? t('未声明') : t('继承上游')
  const clearLabel = custom ? t('清除') : t('恢复继承')

  /** placeholder： this 1 no has / when up or 「」 */
  function tokenPlaceholder(key: CapabilityKey): string {
    const current = capabilities[key]
    if (isSet(key)) return ''
    if (custom) return t('未填写')
    return typeof current === 'number' ? t('继承上游 {value}', { value: exactTokens(current) }) : t('上游未声明')
  }

  /** this 1 current has no has （ setKeysOf） */
  function isSet(key: CapabilityKey): boolean {
    return setKeys.includes(key)
  }

  function setValue(key: CapabilityKey, value: string): void {
    setDraft(prev => ({ ...prev, [key]: value }))
  }

  async function save(): Promise<void> {
    if (saving) return
    // ：empty = （）；must is 1 ~ 1 
    for (const key of TOKEN_KEYS) {
      const raw = draft[key].trim()
      if (!raw) continue
      if (normalizeCapability(key, raw) === null) {
        setStatus(t('{label}要填 1 ~ 1 亿之间的整数（留空 = {inherit}）',
          { label: CAPABILITY_LABELS[key], inherit: inheritLabel }))
        return
      }
    }
    setSaving(true)
    setStatus(t('保存中…'))
    try {
      accept(await writeCapabilities(context.provider, context.id, patchFromDraft(draft)))
      onClose()
      toast(t('✅ 已保存 {id} 的能力位（{provider}）', { id: rowId, provider: providerLabel }))
    } catch (error) {
      setStatus(t('保存失败：{message}', { message: errorMessage(error) }))
      setSaving(false)
    }
  }

  return (
    <Dialog open onOpenChange={(next, eventDetails) => {
      if (next) return
      // in not ： will 「 to no 」unknown
      if (saving) { eventDetails.cancel(); return }
      onClose()
    }}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t('模型能力')}</DialogTitle>
        </DialogHeader>
        <DialogBody>
          {/* ： is 1 （ and 1 use ） */}
          <div className='rounded-md border border-border bg-surface-inset px-3 py-2.5 font-mono text-[12px] text-subtle'>
            <b className='text-primary-fg'>{row.id}</b>{t('（{provider}）', { provider: providerLabel })}
          </div>

          {TOKEN_KEYS.map(key => (
            <div key={key} className='flex flex-col gap-1.5'>
              <div className='flex items-center gap-[7px]'>
                <Label htmlFor={`cap-${key}`}>{CAPABILITY_LABELS[key]}</Label>
                {isSet(key) ? (
                  <Button variant='ghost' size='2xs' disabled={saving}
                    title={custom ? t('清除这一项（回到未声明）') : t('清除覆盖，回到上游声明值')}
                    onClick={() => setValue(key, '')}>{clearLabel}</Button>
                ) : null}
              </div>
              <Input id={`cap-${key}`} inputMode='numeric' autoComplete='off' spellCheck={false}
                placeholder={tokenPlaceholder(key)} disabled={saving} value={draft[key]}
                onChange={event => setValue(key, event.currentTarget.value)}
                onKeyDown={event => { if (event.key === 'Enter') { event.preventDefault(); void save() } }} />
              <p className='text-xs leading-[1.6] text-subtle'>
                {statusText(key, capabilities[key] ?? null, isSet(key), custom, capSourceOf(row, key))}
              </p>
            </div>
          ))}

          {BOOLEAN_KEYS.map(key => (
            <div key={key} className='flex flex-col gap-1.5'>
              <Label>{CAPABILITY_LABELS[key]}</Label>
              <SegmentedControl options={BOOLEAN_OPTIONS(inheritLabel)} value={draft[key]}
                aria-label={CAPABILITY_LABELS[key]} className='self-start'
                onValueChange={next => setValue(key, String(next))} />
              <p className='text-xs leading-[1.6] text-subtle'>
                {statusText(key, capabilities[key] ?? null, isSet(key), custom, capSourceOf(row, key))}
              </p>
            </div>
          ))}

          {levelsInfo ? (
            <div className='rounded-md border border-border bg-surface-inset px-3 py-2.5 text-xs leading-[1.7] text-subtle'>
              <div className='flex flex-wrap items-center gap-x-2 gap-y-1'>
                <span>{levelsInfo.fixedLevel ? t('固定思考档位') : t('可配思考档位')}</span>
                <span className='font-mono text-[12px] text-primary-fg'>
                  {levelsInfo.levels.join(' / ')}
                </span>
                {levelsInfo.defaultLevel ? (
                  <span>{t('（默认 ')}<span className='font-mono text-[12px] text-primary-fg'>{levelsInfo.defaultLevel}</span>{t('）')}</span>
                ) : null}
              </div>
              <div className='mt-1'>
                {levelsInfo.fixedLevel
                  ? t('上游固定为 {level}，绑定其他等级也无法调高。', { level: levelsInfo.fixedLevel })
                  : levelsInfo.source === 'modelsdev'
                    ? t('社区参考档位，不代表当前上游通道支持；实际以该通道目录为准。')
                    : t('这是模型自己能配的档位（来自上游目录），只读；要改某一档请到映射弹窗里选。')}
              </div>
            </div>
          ) : null}

          <p className='text-xs leading-[1.65] text-subtle'>
            {t('这几项是给')}<b>{t('下游客户端')}</b>{t('看的能力声明（')}
            <code>/v1/models</code>{t(' 里的 ')}<code>max_input_tokens</code> / <code>supports_*</code>{t(' 与 ')}<code>input_modalities</code>
            {t('），下游按它们决定发不发图片、按多大的窗口堆历史。')}
            <b>{t('不影响网关的转发与路由')}</b>{t('。上游清单给的值不准时，在这里改；')}
            {custom ? t('这家的清单是你自己登记的，不填的项下游按未声明处理。') : t('「恢复继承」会把这一项交还给上游清单的值。')}
          </p>
          {/* ：，error when not */}
          <div className='min-h-[18px] text-xs text-subtle'>{status}</div>
        </DialogBody>
        <DialogFooter>
          <div className='mr-auto' />
          <Button variant='outline' disabled={saving} onClick={onClose}>{t('取消')}</Button>
          <Button variant='default' disabled={saving} onClick={() => void save()}>{t('保存')}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
