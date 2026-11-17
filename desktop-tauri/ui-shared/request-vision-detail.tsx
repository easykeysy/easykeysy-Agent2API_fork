import { Alert, Badge } from '@ui'
import type { Translate } from './assistant-read-types'

type VisionDetail = {
  mode: 'assist' | 'reroute' | 'assist_call'
  status?: string
  sourceProvider?: string
  sourceModel?: string
  targetProvider?: string
  targetModel?: string
  durationMs?: number
  imageCount?: number
  cachedImages?: number
  totalTokens?: number
  error?: string
  parentId?: string
  calls?: Array<{ id?: string; provider?: string; model?: string; account?: string; durationMs?: number; totalTokens?: number; status?: number; error?: string }>
}

export function visionDetail(route: unknown): VisionDetail | null {
  if (!route || typeof route !== 'object') return null
  const value = (route as { visionCompatibility?: VisionDetail }).visionCompatibility
  return value && ['assist', 'reroute', 'assist_call'].includes(value.mode) ? value : null
}

export function VisionRequestDetail({ route, t }: { route: unknown; t: Translate }) {
  const value = visionDetail(route)
  if (!value) return null
  const mode = value.mode === 'assist' ? t('Assisted reading · Original model answers') : value.mode === 'reroute' ? t('Switch models · Vision model answers') : t('Assisted image reading call')
  return <div className='space-y-2 text-xs'>
    <div className='flex flex-wrap items-center gap-2'><Badge variant='outline'>{mode}</Badge><span className='text-muted-foreground'>{value.status === 'failed' ? t('Failed') : value.status === 'processing' ? t('Processing') : t('Completed')}</span></div>
    {value.sourceModel ? <p className='break-all'>{t('Original model')}：{value.sourceProvider} · {value.sourceModel}</p> : null}
    {value.targetModel ? <p className='break-all'>{t('Vision model')}：{value.targetProvider} · {value.targetModel}</p> : null}
    {value.parentId ? <p className='break-all text-muted-foreground'>{t('Parent request')}：{value.parentId}</p> : null}
    <p className='text-muted-foreground'>{t('Image processing time: {n} seconds', { n: ((Number(value.durationMs) || 0) / 1000).toFixed(2) })}</p>
    {value.mode === 'assist' ? <>
      <p>{t('{images} images; {cached} readings reused', { images: value.imageCount || 0, cached: value.cachedImages || 0 })}</p>
      <p>{t('Additional vision usage: {n} tokens', { n: (Number(value.totalTokens) || 0).toLocaleString() })}</p>
      <p className='text-muted-foreground'>{t('Vision usage is recorded separately against the actual vision account, not mixed with the original model\'s answer usage.')}</p>
      {value.calls?.map((call, index) => <div key={call.id || index} className='rounded border border-hairline p-2'>
        <p className='break-all'>{call.provider} · {call.model} · {call.account}</p>
        <p className='text-muted-foreground'>{((Number(call.durationMs) || 0) / 1000).toFixed(2)} s · {(Number(call.totalTokens) || 0).toLocaleString()} Token</p>
        {call.error ? <p className='break-words text-destructive'>{call.error}</p> : null}
      </div>)}
    </> : null}
    {value.error ? <Alert variant='destructive'>{value.error}</Alert> : null}
  </div>
}
