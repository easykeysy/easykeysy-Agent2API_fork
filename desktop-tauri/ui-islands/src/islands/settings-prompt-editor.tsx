/**
 * set · 「」 by and 。
 *
 * from settings-page.tsx ： that 1 already set and ，
 * this and ， will no 。 is to 
 * （file import settings-state / settings-model， not this ），
 * by to for 。
 *
 * ── is ───────────────────────────────────────
 * ① （ / down need that ）；
 * ② 1 （ 1 ， for this 1 ）；
 * ③ 1 ****（ up ，
 *      `core::providers::zcode::OFFICIAL_PROMPT_NOTE`）。
 * ①② is 「client system 」 that 1 ，③ is 「」 that 1 
 * —— and not ， up 。
 *
 * ── Why is not is ──────────────────────────
 * is ， not before after no ； this 1 
 * 1 is 1 「set」， will 。 to 
 * and ， 1 by and 1 。
 *
 * ── （ and after 1 ， in by and ）──────────────
 * · ① ②： after ** this as **（file）；empty = this 1 not need 、
 * file / default（「default」= 1 empty， not is config）；
 * · ③：** 1 **（ up is 「」 this ， is 
 * and ）； 1 empty = that 1 to ，empty = to 。
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
  Label,
  Textarea,
} from '@ui'
import { NOTES, type GatewayBlocks } from './settings-model'
import {
  savePromptText,
  saveProviderGatewayText,
  saveProviderPromptText,
  type PromptState,
  type ProviderPromptOption,
  type ProviderPromptState,
} from './settings-state'
import { t } from '../i18n'

/** and （ is after `GatewayBlocks::FIELDS` that ； in this 1 ）。
 * `short` is ：gatewayEditedText use 「 already 」。 */
const BLOCK_FIELDS: { key: keyof GatewayBlocks; label: string; short: string; hint: string }[] = [
  { key: 'identity', label: t('第一段（身份句）'), short: t('第一段'), hint: t('官方原文只有一句：声明自己是 ZCode。') },
  { key: 'stable', label: t('第二段（稳定段）'), short: t('第二段'), hint: t('工具用法与环境说明那一大段。') },
  { key: 'dynamic', label: t('第三段（动态段）'), short: t('第三段'), hint: t('沟通方式 / 上下文管理 + Environment 段。') },
]

/** in （'inline' = this is ；'none' = ， no has ） */
function sourceText(source: string): string {
  if (source === 'inline') return t('界面里编辑的正文')
  if (source === 'file') return t('提示词文件')
  if (source === 'builtin') return t('内置默认提示词')
  return t('（未生效）')
}

/* ─── ①② （ / 1 ）──────────── */

/**
 * 。
 *
 * `save` return is success：failure when in （ in ， use can ），
 * success —— failure use ， is that 。
 */
function BodyEditor({ title, text, source, file, disabled, save, onClose }: {
  title: string
  /** when （ is no has when ） */
  text: string
  source: string
  /** current file（used for「 after not use 」） */
  file: string
  disabled: boolean
  save: (text: string) => Promise<boolean>
  onClose: () => void
}) {
  const [draft, setDraft] = React.useState(text)
  const [saving, setSaving] = React.useState(false)
  const busy = saving || disabled
  const edited = draft !== text

  async function commit(next: string): Promise<void> {
    setSaving(true)
    try {
      if (await save(next)) onClose()
    } finally {
      setSaving(false)
    }
  }

  return (
    <Dialog open onOpenChange={open => {
      // in not ： will 「 to no 」unknown（ and ）
      if (open || busy) return
      onClose()
    }}>
      <DialogContent className='w-[min(880px,calc(100vw-48px))]'>
        <DialogHeader><DialogTitle>{title}</DialogTitle></DialogHeader>
        <DialogBody>
          <DialogSection>
            <h3>{t('保存后以这里的文本为准')}</h3>
            <p>
              {t('这一份就是「替换 / 追加」模式下网关要发的 system 正文。')}
              {/* `source === 'none'` and 「current is 」 etc ： in when to 
                  none（ no has ）—— `config::resolve_choice` */}
              {source === 'none'
                ? t('当前是透传模式：存下来也不会发出，切成「替换 / 追加」之后才生效。')
                : t('当前生效的正文来自{source}{file}。保存后这一份会盖过提示词文件与内置默认{note}。', {
                    source: sourceText(source),
                    file: file.trim() && source !== 'inline' ? t('（文件：{path}）', { path: file.trim() }) : '',
                    note: source !== 'inline' && file.trim()
                      ? t('，文件里后续的改动在清空这里之前不再生效')
                      : '',
                  })}
              {t('想回到原来那份，把这里清空再保存即可（不会删掉你配的文件路径）。')}
            </p>
          </DialogSection>
          <div className='flex flex-col gap-1.5'>
            <Label htmlFor='settings-prompt-body'>{t('提示词正文')}</Label>
            <Textarea
              id='settings-prompt-body'
              rows={18}
              className='min-h-[320px]'
              spellCheck={false}
              disabled={busy}
              value={draft}
              onChange={event => setDraft(event.currentTarget.value)}
            />
            <div className='hint'>
              {draft.trim()
                ? t('{lines} 行、{chars} 字符（按当前草稿计）', { lines: draft.split('\n').length, chars: draft.length })
                : t('空 = 不要这一份，回落提示词文件 / 内置默认提示词。')}
            </div>
          </div>
        </DialogBody>
        <DialogFooter>
          <Button
            variant='outline'
            className='mr-auto'
            // in ** has 1 ** when can ： no has this 1 when not will has 
            // （ after empty「」，empty = not ）， no 
            disabled={busy || source !== 'inline'}
            onClick={() => void commit('')}
          >
            {t('清除这里的正文')}
          </Button>
          <Button variant='outline' disabled={busy} onClick={onClose}>{t('取消')}</Button>
          <Button
            variant='default'
            disabled={busy || !edited}
            onClick={() => void commit(draft)}
          >
            {saving ? t('保存中…') : t('保存')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

/** 「」 by （ for by 1 ） */
export function PromptTextButton({ prompt, locked, busy }: {
  prompt: PromptState
  locked: boolean
  busy: boolean
}) {
  const [open, setOpen] = React.useState(false)
  return (
    <>
      <Button
        id='btn-prompt-edit-body'
        variant='outline'
        disabled={locked || busy}
        onClick={() => setOpen(true)}
      >
        {t('编辑正文')}
      </Button>
      {open ? (
        <BodyEditor
          title={t('编辑提示词正文 · 全局')}
          text={prompt.text}
          source={prompt.source}
          file={prompt.file}
          disabled={busy}
          save={savePromptText}
          onClose={() => setOpen(false)}
        />
      ) : null}
    </>
  )
}

/** 1 「」 by （ and that ， in this 1 ） */
export function ProviderPromptTextButton({ item, label, locked, busy }: {
  item: ProviderPromptState
  label: string
  locked: boolean
  busy: boolean
}) {
  const [open, setOpen] = React.useState(false)
  return (
    <>
      <Button
        id={`btn-prompt-edit-body-${item.id}`}
        variant='outline'
        disabled={locked || busy}
        onClick={() => setOpen(true)}
      >
        {t('编辑正文')}
      </Button>
      {open ? (
        <BodyEditor
          title={t('编辑提示词正文 · {label}', { label })}
          text={item.text}
          source={item.source}
          file={item.file}
          disabled={busy}
          save={text => saveProviderPromptText(item, text)}
          onClose={() => setOpen(false)}
        />
      ) : null}
    </>
  )
}

/* ─── ③ ─────────────────── */

/** （ to after ， in prompt.gatewayText ） */
function GatewayEditor({ id, title, official, over, enabled, disabled, onClose }: {
  id: string
  title: string
  official: GatewayBlocks
  /** already down （ ； use official ） */
  over: GatewayBlocks | undefined
  /** this 1 「」（ when not ， down that ） */
  enabled: boolean
  disabled: boolean
  onClose: () => void
}) {
  const [draft, setDraft] = React.useState<GatewayBlocks>(() => ({
    identity: over?.identity?.trim() ? over.identity : official.identity,
    stable: over?.stable?.trim() ? over.stable : official.stable,
    dynamic: over?.dynamic?.trim() ? over.dynamic : official.dynamic,
  }))
  const [saving, setSaving] = React.useState(false)
  const busy = saving || disabled
  const edited = BLOCK_FIELDS.some(field => draft[field.key] !== official[field.key])
  const editedNow = BLOCK_FIELDS.filter(field => over?.[field.key]?.trim()).map(field => field.label)

  async function commit(next: GatewayBlocks | null): Promise<void> {
    setSaving(true)
    try {
      if (await saveProviderGatewayText(id, next)) onClose()
    } finally {
      setSaving(false)
    }
  }

  return (
    <Dialog open onOpenChange={open => {
      if (open || busy) return
      onClose()
    }}>
      <DialogContent className='w-[min(880px,calc(100vw-48px))]'>
        <DialogHeader><DialogTitle>{title}</DialogTitle></DialogHeader>
        <DialogBody>
          <DialogSection>
            <h3>{t('三段各自成块，所以分开编辑')}</h3>
            <p>
              {t('上游按')}<strong>{t('结构')}</strong>
              {t('校验身份：这三段必须各自成块排在最前（实测：三段并成一段会被回 405 / 3012）。因此这里按段编辑，不能合成一段。改过的段以你的文本为准，没改的段继续用官方原文；清空某一段 = 那一段恢复官方原文。')}
              {editedNow.length ? t('当前已改过：{edited}。', { edited: editedNow.join(t('、')) }) : t('当前三段都是官方原文。')}
              {/* when this not will （`apply_start_plan` ，
                   1 ）—— not this ， use will as */}
              {enabled ? '' : t('注意：这一家的「网关自带」开关现在是关的，改过的正文要等开关打开才生效。')}
            </p>
            <p>{NOTES.promptGatewayText}</p>
          </DialogSection>
          {BLOCK_FIELDS.map(field => (
            <div className='flex flex-col gap-1.5' key={field.key}>
              <Label htmlFor={`settings-prompt-block-${id}-${field.key}`}>
                {field.label}
                <span className='ml-2 font-normal text-subtle'>
                  {draft[field.key] === official[field.key] ? t('（官方原文）') : t('（已改）')}
                </span>
              </Label>
              <Textarea
                id={`settings-prompt-block-${id}-${field.key}`}
                rows={field.key === 'identity' ? 2 : 10}
                className={field.key === 'identity' ? 'min-h-[64px]' : 'min-h-[200px]'}
                spellCheck={false}
                disabled={busy}
                value={draft[field.key]}
                onChange={event => {
                  const value = event.currentTarget.value
                  setDraft(current => ({ ...current, [field.key]: value }))
                }}
              />
              <div className='hint'>{field.hint}</div>
            </div>
          ))}
        </DialogBody>
        <DialogFooter>
          <Button
            variant='outline'
            className='mr-auto'
            disabled={busy || !editedNow.length}
            onClick={() => void commit(null)}
          >
            {t('恢复官方原文')}
          </Button>
          <Button variant='outline' disabled={busy} onClick={onClose}>{t('取消')}</Button>
          <Button variant='default' disabled={busy || !edited} onClick={() => void commit(draft)}>
            {saving ? t('保存中…') : t('保存')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

/**
 * 「」 by 。
 *
 * （） by after `promptProviderOptions` 1 to ： when is null，
 * when ** not by ** —— no has can ， use for 1 empty will 
 * 。 and is ： old ， not as this 。
 */
export function GatewayTextButton({ item, option, over, locked, busy }: {
  item: ProviderPromptState
  option: ProviderPromptOption
  /** this 1 already （ = is ） */
  over: GatewayBlocks | undefined
  locked: boolean
  busy: boolean
}) {
  const [open, setOpen] = React.useState(false)
  const official = option.gatewayBlocks
  if (!official) return null
  return (
    <>
      <Button
        id={`btn-prompt-gateway-text-${item.id}`}
        variant='outline'
        disabled={locked || busy}
        onClick={() => setOpen(true)}
      >
        {t('编辑正文')}
      </Button>
      {open ? (
        <GatewayEditor
          id={item.id}
          title={t('编辑网关自带提示词 · {label}', { label: option.label })}
          official={official}
          over={over}
          enabled={item.gateway}
          disabled={busy}
          onClose={() => setOpen(false)}
        />
      ) : null}
    </>
  )
}

/** this 1 （used for ； no to empty） */
export function gatewayEditedText(over: GatewayBlocks | undefined): string {
  if (!over) return ''
  return BLOCK_FIELDS
    .filter(field => (over[field.key] || '').trim())
    .map(field => field.short)
    .join(t('、'))
}
