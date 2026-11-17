import * as React from 'react'
import { Badge, Button, Spinner } from '@ui'
import { t } from '../i18n'

export type TestModelResponse = {
  success?: boolean
  status?: number
  error?: string | null
  reply?: string
  reasoning?: string
  provider?: string
  model?: string
  protocol?: string
  proxy?: string | null
  account_id?: string | null
  account_name?: string | null
  upstream_model?: string | null
  upstream_reasoning?: string | null
  duration_ms?: number
  ttfb_ms?: number | null
  attempts?: number
  prompt_tokens?: number
  completion_tokens?: number
  total_tokens?: number
  apiKeyId?: string
  apiKeyName?: string
  apiKeySource?: string
}

export type ModelTestSlot = {
  accountId: string
  label: string
  position: number
  testId: string
  startedAt: number
  state: 'running' | 'done'
  result?: TestModelResponse
  transport?: string
}

export function modelTestOk(slot: ModelTestSlot): boolean {
  if (slot.transport || !slot.result || slot.result.error) return false
  const status = Number(slot.result.status) || 0
  return status >= 200 && status < 300
}

function formatMs(value: unknown): string {
  const time = Number(value)
  if (!Number.isFinite(time) || time <= 0) return '—'
  return time < 1000 ? `${Math.round(time)} ms` : `${(time / 1000).toFixed(time < 10000 ? 2 : 1)} s`
}

const PROTOCOL_SHORT: Record<string, string> = {
  chat_completions: 'Chat Completions', responses: 'Responses', anthropic: 'Anthropic Messages',
}

const STATUS_HINT: Array<{ test: (status: number) => boolean; hint: string }> = [
  { test: status => status === 429, hint: t('This account is cooling down on rate limits for this model; it will recover on its own shortly. You can also test another account first.') },
  { test: status => status === 401 || status === 403, hint: t('Upstream rejected this login session: sign in again, or delete this account.') },
  { test: status => status === 404, hint: t('Upstream doesn\'t recognize this model name: click “Fetch models” to refresh the catalog, or check the upstream model the default binding points to.') },
  { test: status => status === 504, hint: t('Upstream didn\'t finish its answer within the budget. Test again with only one account checked to see whether it is slow in general or only on one account.') },
  { test: status => status >= 500, hint: t('This is most likely a problem on the upstream side; try again later. When the same error appears on all accounts, check the model catalog and mappings first.') },
]

export function ModelTestResult({ slot, model, now, expanded, onToggle }: {
  slot: ModelTestSlot; model: string; now: number; expanded: boolean; onToggle: () => void
}) {
  const head = (badge: React.ReactNode, metrics: string) => (
    <div className='flex items-center gap-2.5 border-b border-hairline px-3 py-2'>
      {badge}
      <b className='min-w-0 truncate text-[12.5px] text-foreground' title={slot.label}>{slot.label}</b>
      {slot.position ? <span className='flex-none text-xs text-subtle'>#{slot.position}</span> : null}
      <span className='ml-auto flex-none font-mono text-[11.5px] tabular-nums text-subtle'>{metrics}</span>
    </div>
  )
  if (slot.state === 'running') return (
    <div className='flex items-center gap-2.5 rounded-md border border-border bg-surface-2 px-3 py-2.5 text-xs text-subtle'>
      <Spinner className='flex-none' />
      <span className='min-w-0 truncate'>{t('Account ')}<b className='text-foreground'>{slot.label}</b>{t('\'s request has been sent; waiting for the upstream\'s first frame…')}</span>
      <span className='ml-auto flex-none font-mono tabular-nums'>{t('Waited {seconds}s', { seconds: ((now - slot.startedAt) / 1000).toFixed(1) })}</span>
    </div>
  )
  const result = slot.result
  if (!modelTestOk(slot)) {
    const status = Number(result?.status) || 0
    const hint = STATUS_HINT.find(item => item.test(status))?.hint ?? (slot.transport ? t('Make sure the desktop app is still running, then try again.') : '')
    return (
      <div className='rounded-md border border-border bg-surface-2'>
        {head(<Badge shape='tag' variant='destructive' className='flex-none'>{status || t('Failed')}</Badge>, t('Took {time}', { time: formatMs(result?.duration_ms) }))}
        <div className='flex flex-col gap-1.5 px-3 py-2.5'>
          <div className='text-[12.5px] leading-[1.7]'>
            {slot.transport
              ? <><b className='text-destructive'>{t('The local gateway returned no result')}</b>{t(': ')}{slot.transport}</>
              : <><b className='text-destructive'>{t('Test failed')}</b>{t(': ')}{result?.error || t('Upstream returned no readable error description')}</>}
            {Number(result?.attempts) > 1 ? t(' ({n} attempts in total)', { n: Number(result?.attempts) }) : ''}
          </div>
          {hint ? <p className='text-xs leading-[1.65] text-subtle'>{hint}</p> : null}
        </div>
      </div>
    )
  }
  const reply = String(result?.reply ?? '')
  const reasoning = String(result?.reasoning ?? '')
  const meta = [
    result?.upstream_model && result.upstream_model !== model ? t('Upstream {model}', { model: result.upstream_model }) : '',
    result?.upstream_reasoning ? t('Reasoning level {level}', { level: result.upstream_reasoning }) : '',
    result?.protocol && result.protocol !== 'chat_completions' ? PROTOCOL_SHORT[result.protocol] || result.protocol : '',
    result?.proxy ? t('Node {proxy}', { proxy: result.proxy }) : '',
    result?.apiKeyName ? `API Key: ${result.apiKeyName}` : '',
    Number(result?.attempts) > 1 ? t('{n} attempts', { n: Number(result?.attempts) }) : '',
  ].filter(Boolean).join(' · ')
  return (
    <div className='rounded-md border border-border bg-surface-2'>
      {head(<Badge shape='tag' variant='success' className='flex-none'>{Number(result?.status) || 200}</Badge>,
        t('Took {time} · first token {ttfb}', { time: formatMs(result?.duration_ms), ttfb: formatMs(result?.ttfb_ms) }))}
      <div className='flex flex-col gap-2 px-3 py-2.5'>
        <div className='flex items-center gap-2 text-[11.5px] text-subtle'>
          <span className='flex-none'>{t('Reply')}</span>
          {meta ? <span className='min-w-0 truncate' title={meta}>{meta}</span> : null}
          <Button variant='ghost' size='2xs' className='ml-auto flex-none' data-copy={reply}>{t('Copy')}</Button>
        </div>
        <div className='max-h-[220px] overflow-y-auto whitespace-pre-wrap rounded-sm border border-border bg-surface px-3 py-2 text-[12.5px] leading-[1.7]'>
          {reply || <span className='text-subtle'>{t('(Upstream returned an empty body this time)')}</span>}
        </div>
        {reasoning ? <div className='overflow-hidden rounded-sm border border-border'>
          <Button variant='ghost' size='xs' className='w-full' aria-expanded={expanded} onClick={onToggle}>
            <span className='truncate'>{t('Reasoning process ({n} chars)', { n: reasoning.length })}</span>
            <span aria-hidden='true' className='ml-auto text-[10px]'>{expanded ? '▴' : '▾'}</span>
          </Button>
          {expanded ? <div className='border-t border-hairline bg-surface px-2.5 py-2'>
            <pre className='whitespace-pre-wrap font-mono text-[11.5px] leading-[1.65] text-subtle'>{reasoning}</pre>
          </div> : null}
        </div> : null}
      </div>
    </div>
  )
}
