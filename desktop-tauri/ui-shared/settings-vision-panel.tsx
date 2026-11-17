/**
 * 图片兼容设置面板（桌面与手机端共用）。
 *
 * ── 设计口径 ──────────────────────────────────────────────────
 *   · 模型看不了图片时只有两种办法，所以这里是**两块固定设置**，不是可以增删的
 *     配置列表：每块只回答两件事 —— 「哪些模型」和「用哪只视觉模型」；
 *   · 没被任何名单选中的模型、以及本来就能看图的模型，一律原样放行，不需要
 *     再写一条「不做兼容」的规则去声明；
 *   · **没有保存步骤**：跟设置页里网关开关、重试、超时一样，改完立即提交。
 *     所以这里没有草稿、没有「撤销 / 保存」，也就没有保存前的授权弹窗 ——
 *     「图片会交给哪只模型」由每块底部的「实际处理」那行直接说明；
 *   · 两块名单重叠时按辅助识图处理，界面把这件事直接指出来，而不是静默二选一。
 *
 * ── 与后端契约 ────────────────────────────────────────────────
 * 读写仍然是 `/api/vision-compatibility` 的 GET / PUT（`revision`）。
 * 旧版（全局二选一 + 模型单独配置）由服务端按相同语义展开成名单返回，
 * 展开结果只在用户真的改了一处之后才按新形式存回去。
 */

import { useCallback, useEffect, useRef, useState } from 'react'
import {
  Alert, Button, Card, CardAction, CardContent, CardHeader, CardTitle, Collapsible, CollapsibleContent,
  CollapsibleTrigger, Input, Select, SelectContent, SelectItem, SelectTrigger, SelectValue, StatusDot, cn,
} from '@ui'
import type { Translate } from './assistant-read-types'
import { FooterNote, Group, Row, SwitchRow, Tip } from './settings-vision-rows'
import { VisionReasoningSelect } from './settings-vision-editors'
import { VisionAssistCard, VisionRerouteCard, visionModelLabel } from './settings-vision-way'
import { visionOverlap, type VisionCompatApi, type VisionSettings, type VisionView } from './settings-vision-types'

/** 「更多设置」入口上的小箭头（与项目其它内联图标同一套几何：24 画布、currentColor） */
function Chevron({ open }: { open: boolean }) {
  return <svg viewBox='0 0 24 24' width={15} height={15} aria-hidden='true' fill='none' stroke='currentColor'
    strokeWidth='1.8' strokeLinecap='round' strokeLinejoin='round'
    className={cn('flex-none text-muted-foreground transition-transform duration-[150ms]', open && 'rotate-90')}>
    <path d='M10 6.5 15.5 12 10 17.5' />
  </svg>
}

function InfoIcon() {
  return <svg viewBox='0 0 24 24' width={14} height={14} aria-hidden='true' fill='none' stroke='currentColor'
    strokeWidth='1.8' strokeLinecap='round' className='mt-0.5 flex-none'>
    <circle cx='12' cy='12' r='9' /><path d='M12 8h.01M11 11.5h1V16h1' />
  </svg>
}

/** 数字项：编辑期间只改本地文案，失焦 / 回车才提交（与设置页其它数字项同一套做法） */
function TimeoutField({ value, disabled, onCommit, t }: {
  value: number
  disabled: boolean
  onCommit(next: number): void
  t: Translate
}) {
  const [text, setText] = useState<string | null>(null)
  const shown = text ?? String(value)
  const parsed = Number(shown)
  const valid = Number.isInteger(parsed) && parsed >= 5 && parsed <= 120
  return <span className='flex items-center gap-2'>
    <Input className={cn('w-20', !valid && 'border-destructive')} type='number' min={5} max={120} step={1}
      disabled={disabled} aria-label={t('Longest wait for compatibility processing')} value={shown}
      onChange={event => setText(event.currentTarget.value)}
      onKeyDown={event => { if (event.key === 'Enter') event.currentTarget.blur() }}
      onBlur={() => { if (text !== null) { if (valid) onCommit(parsed); setText(null) } }} />
    <span className='text-[12px] text-muted-foreground'>{t('seconds')}</span>
  </span>
}

function message(failure: unknown): string {
  return failure instanceof Error ? failure.message : String(failure)
}

export function VisionCompatibilityPanel({ api, t, active = true }: { api: VisionCompatApi; t: Translate; active?: boolean }) {
  const [view, setView] = useState<VisionView | null>(null)
  const [draft, setDraft] = useState<VisionSettings | null>(null)
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState('')
  const [more, setMore] = useState(false)
  const alive = useRef(true)
  const sequence = useRef(0)
  /** 服务端最新状态：提交要用它的 revision，失败回滚也回到它 */
  const latest = useRef<VisionView | null>(null)
  /** 提交串行化：改动是逐次点出来的，但在途请求不能并发（否则后一次会拿到旧的 revision） */
  const chain = useRef<Promise<unknown>>(Promise.resolve())

  const adopt = (next: VisionView) => {
    latest.current = next
    setView(next)
    setDraft(structuredClone(next.config))
  }

  const load = useCallback(async () => {
    const token = ++sequence.current
    setBusy(true); setError('')
    try {
      const next = await api.load()
      if (alive.current && token === sequence.current) adopt(next)
    } catch (failure) {
      if (alive.current && token === sequence.current) setError(message(failure))
    } finally {
      if (alive.current && token === sequence.current) setBusy(false)
    }
  }, [api])

  useEffect(() => {
    alive.current = true
    return () => { alive.current = false; ++sequence.current }
  }, [])
  useEffect(() => { if (active && !view) void load() }, [active, view, load])

  /** 改一处就提交一次：界面上没有保存按钮，这里也不留草稿状态 */
  const commit = (next: VisionSettings) => {
    setDraft(next)
    setError('')
    chain.current = chain.current.then(async () => {
      const current = latest.current
      if (!current) return
      setBusy(true)
      try {
        const saved = await api.save({ config: next, revision: current.revision })
        if (!alive.current) return
        latest.current = saved
        setView(saved)
      } catch (failure) {
        if (!alive.current) return
        setError(message(failure))
        setDraft(structuredClone(latest.current?.config ?? current.config))
      } finally {
        if (alive.current) setBusy(false)
      }
    })
  }

  if (!view || !draft) {
    return <Card render={<section />} className='panel'>
      <CardHeader><CardTitle>{t('Image compatibility')}</CardTitle></CardHeader>
      <CardContent className='flex flex-col gap-4'>
        <p className='m-0 text-[12.5px] text-muted-foreground'>{busy ? t('Loading…') : t('Failed to load image compatibility settings')}</p>
        {error ? <Alert variant='destructive'>{error}</Alert> : null}
      </CardContent>
    </Card>
  }

  const overlap = visionOverlap(draft)
  const patch = (next: Partial<VisionSettings>) => commit({ ...draft, ...next })
  // 思考等级是逐只视觉模型存的；两块用同一个选择，所以改一次就写到当前所有目标上
  const levels = [
    ...(draft.assist.target ? [draft.assist.target.reasoning] : []),
    ...draft.reroute.targets.map(target => target.reasoning),
  ]
  const reasoning = levels.length > 0 && levels.every(level => level === levels[0]) ? levels[0] : ''
  const setReasoning = (next: string) => patch({
    assist: { ...draft.assist, target: draft.assist.target ? { ...draft.assist.target, reasoning: next } : null },
    reroute: { ...draft.reroute, targets: draft.reroute.targets.map(target => ({ ...target, reasoning: next })) },
  })

  return <Card render={<section />} className='panel'>
    <CardHeader>
      <CardTitle>{t('Image compatibility')}</CardTitle>
      <Tip label={t('Image compatibility')} text={t('A model that cannot read images has only two options: let another model read the image for it, or hand the whole request to a model that can. Anything not covered here — including models that already read images — passes through untouched. Each request uses exactly one of the two ways; they are never chained, and a failure does not fall back to the other. Changes take effect immediately; there is nothing to save.')} />
      <span className='flex items-center gap-1.5 text-[12px] text-muted-foreground'>
        <StatusDot variant={draft.enabled ? 'success' : 'muted'} />{draft.enabled ? t('Enabled') : t('Disabled')}
      </span>
      <CardAction>
        <Button size='sm' variant='ghost' disabled={busy} onClick={() => void load()}>{t('Refresh')}</Button>
      </CardAction>
    </CardHeader>
    <CardContent className='flex flex-col gap-4'>
      <p className='m-0 max-w-[760px] text-[11.5px] leading-relaxed text-muted-foreground'>
        {t('Rules apply globally, without being limited to the original routing pool. Selected models use their own routing to choose accounts; gateway key permissions still apply.')}
      </p>
      {view.legacyExpanded ? <p className='m-0 max-w-[760px] text-[11.5px] leading-relaxed text-muted-foreground'>
        {t('Your previous "global mode + per-model overrides" has been filled into the model lists below with the same meaning. Keep it as is or adjust it; once you change anything it is stored in the new form.')}
      </p> : null}
      {error ? <Alert variant='destructive'>{error}</Alert> : null}

      <Group>
        <SwitchRow first label={t('Enable image compatibility')}
          checked={draft.enabled} disabled={busy} onCheckedChange={enabled => patch({ enabled })} />
      </Group>
      {!draft.enabled ? <p className='m-0 max-w-[760px] text-[11.5px] leading-relaxed text-muted-foreground'>
        {t('Image compatibility is currently off: the two blocks below can still be edited and are kept, but nothing here applies to any request.')}
      </p> : null}

      <div className='grid items-start gap-[18px] [grid-template-columns:repeat(auto-fit,minmax(min(430px,100%),1fr))]'>
        <VisionAssistCard value={draft.assist} models={view.models} reasoning={reasoning} disabled={busy} t={t}
          onChange={assist => patch({ assist: { ...draft.assist, ...assist } })} />
        <VisionRerouteCard value={draft.reroute} models={view.models} reasoning={reasoning}
          maxTargets={view.maxTargets} disabled={busy} t={t}
          onChange={reroute => patch({ reroute: { ...draft.reroute, ...reroute } })} />
      </div>

      {overlap.length ? <div className='flex items-start gap-2 rounded-md border border-warning-bd bg-warning-soft px-3 py-2.5 text-[12px] leading-relaxed text-warning'>
        <InfoIcon />
        <span>{t('{models} appears in both blocks and will be handled by assisted image reading. To change that, remove it from one of the model lists.', {
          models: overlap.map(entry => visionModelLabel(view.models, entry, t)).join(', '),
        })}</span>
      </div> : null}

      <Collapsible open={more} onOpenChange={setMore}>
        <div className='flex flex-col gap-2'>
          <CollapsibleTrigger render={<Button variant='ghost'
            className='h-auto w-full justify-between gap-3 rounded-sm px-1 py-1.5' />}>
            <span className='flex min-w-0 flex-wrap items-baseline gap-2'>
              <span className='text-[13px] font-medium text-foreground'>{t('More settings')}</span>
              <span className='text-[12px] text-muted-foreground'>{t('Reasoning level, unknown image capability, wait limit')}</span>
            </span>
            <Chevron open={more} />
          </CollapsibleTrigger>
          <CollapsibleContent className='flex flex-col gap-2 pt-2'>
            <Group>
              <Row first label={t('Reasoning level of the vision model')} tip={t('Only affects how deeply the image-reading step thinks; both blocks share this choice.')}
                className='items-center max-[767px]:flex-col max-[767px]:items-stretch max-[767px]:gap-2' control={
                  <VisionReasoningSelect t={t} value={reasoning} disabled={busy} onValueChange={setReasoning} />
                } />
              <Row label={t('When image capability is unknown')} tip={t('Used when the model catalog does not say whether images are supported.')} className='items-center max-[767px]:flex-col max-[767px]:items-stretch max-[767px]:gap-2' control={
                <Select value={draft.unknownCapability} disabled={busy}
                  onValueChange={unknownCapability => patch({ unknownCapability: unknownCapability as VisionSettings['unknownCapability'] })}>
                  <SelectTrigger className='w-[min(260px,100%)]' aria-label={t('When image capability is unknown')}>
                    <SelectValue>{draft.unknownCapability === 'try_original' ? t('Send original images first; adapt only after an explicit rejection') : t('Apply the settings above right away')}</SelectValue>
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value='try_original'>{t('Send original images first; adapt only after an explicit rejection')}</SelectItem>
                    <SelectItem value='compat'>{t('Apply the settings above right away')}</SelectItem>
                  </SelectContent>
                </Select>
              } />
              <Row label={t('Longest wait for compatibility processing')} tip={t('Range 5–120 seconds, also bounded by the original request timeout.')} className='items-center max-[767px]:flex-col max-[767px]:items-stretch max-[767px]:gap-2' control={
                <TimeoutField value={draft.timeoutSeconds} disabled={busy} t={t}
                  onCommit={timeoutSeconds => patch({ timeoutSeconds })} />
              } />
            </Group>
            <FooterNote>{t('Oversized, malformed or undownloadable images are not treated as unsupported by the model.')}</FooterNote>
          </CollapsibleContent>
        </div>
      </Collapsible>
    </CardContent>
  </Card>
}
