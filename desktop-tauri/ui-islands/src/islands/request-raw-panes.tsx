import * as React from 'react'
import { Button, Empty, HtmlContent, Spinner, Switch } from '@ui'
import { t } from '../i18n'
import { NOTES } from './settings-model'
import { loadDebug, saveDebug, useSettings } from './settings-state'

export function RequestDebugSwitch() {
  const { debug, busy } = useSettings()
  const [loading, setLoading] = React.useState(true)
  const alive = React.useRef(false)
  const loadingRef = React.useRef(false)

  const refresh = React.useCallback(async () => {
    if (loadingRef.current) return
    loadingRef.current = true
    setLoading(true)
    try { await loadDebug() }
    finally {
      loadingRef.current = false
      if (alive.current) setLoading(false)
    }
  }, [])

  React.useEffect(() => {
    alive.current = true
    void refresh()
    return () => { alive.current = false }
  }, [refresh])

  const saving = busy === 'debug'
  const unavailable = !loading && debug.status === 'unavailable'
  return (
    <div className='ml-auto flex min-h-[30px] flex-wrap items-center justify-end gap-2'>
      <label className='inline-flex items-center gap-2 whitespace-nowrap text-[12px] text-subtle' title={NOTES.debug}>
        <span>{t('Debug mode')}</span>
        <Switch checked={debug.on} disabled={loading || debug.status !== 'ready' || busy !== null}
          aria-label={t('Debug mode')} aria-busy={loading || saving}
          onCheckedChange={next => void saveDebug(next)} />
      </label>
      {loading || saving ? <Spinner className='size-3.5 text-muted-foreground'
        aria-label={loading ? t('Loading…') : t('Saving…')} /> : null}
      {unavailable ? <>
        <span className='text-[11.5px] text-destructive' role='status'
          title={t('Could not read the debug mode settings. Please try again later.')}>{t('Read failed')}</span>
        <Button variant='ghost' size='xs' disabled={busy !== null} onClick={() => void refresh()}>{t('Retry')}</Button>
      </> : null}
    </div>
  )
}

export type RawBody = Record<string, unknown>
export type DebugTraffic = Record<string, unknown>
export type RawState = null | 'gone' | { body: RawBody; truncated: boolean }
export type DebugState = null | 'gone' | { data: DebugTraffic }

function jsonText(value: unknown): string {
  if (value === null || value === undefined) return ''
  if (typeof value === 'string') return value
  try { return JSON.stringify(value, null, 2) } catch { return '' }
}

const DEBUG_SEGS = [
  { label: t('Request headers'), pick: (data: DebugTraffic) => jsonText(data.requestHeaders) },
  { label: t('Request body'), pick: (data: DebugTraffic) => jsonText(data.requestBody) },
  { label: t('Response headers'), pick: (data: DebugTraffic) => jsonText(data.responseHeaders) },
  { label: t('Response body'), pick: (data: DebugTraffic) => data.responseBody ? String(data.responseBody) : '' },
]

export function hasDebugTraffic(data: DebugTraffic): boolean {
  return [data.requestHeaders, data.requestBody, data.responseHeaders, data.responseBody]
    .some(value => typeof value === 'string' ? value.trim() !== '' : value != null)
}

export function RequestEmpty({ text, loading = false }: { text: string; loading?: boolean }) {
  return (
    <Empty className='flex-row py-6'>
      {loading ? <Spinner className='size-3.5' /> : null}
      {text}
    </Empty>
  )
}

export function PreviewPane({ state }: { state: RawState }) {
  if (!state) return <RequestEmpty text={t('Loading…')} loading />
  const body = state === 'gone' ? null : state.body
  const bridge = window as unknown as {
    wbConversationPreview?: { render?: (request: unknown, response: unknown) => string }
  }
  const html = body ? bridge.wbConversationPreview?.render?.(body.requestBody, body.responseBody) || '' : ''
  if (!html) return <RequestEmpty text={t('No raw payload (debug mode was off, or it has been cleared)')} />
  return (
    <div>
      {state !== 'gone' && state.truncated && (
        <div className='mb-2 text-[11.5px] text-warning'>{t('The body saved by an older version may be truncated; missing content cannot be recovered')}</div>
      )}
      <HtmlContent html={html} />
    </div>
  )
}

export function RawPane({ data }: { data: DebugTraffic }) {
  const status = data.status === null || data.status === undefined ? '-' : String(data.status)
  const meta = [
    data.url ? `URL: ${data.url}` : '',
    data.provider ? t('Provider: {provider}', { provider: String(data.provider) }) : '',
    t('Upstream status code: {status}', { status }),
  ].filter(Boolean)
  return (
    <div className='flex flex-col gap-4'>
      {meta.length > 0 && (
        <div className='border-b border-hairline pb-2.5 text-xs leading-[1.7] break-all text-subtle'>
          {meta.map((line, index) => <div key={index}>{line}</div>)}
        </div>
      )}
      {DEBUG_SEGS.map(seg => {
        const text = seg.pick(data)
        return (
          <div key={seg.label} className='flex min-w-0 flex-col gap-1.5'>
            <div className='flex items-center gap-2'>
              <h3 className='text-[11.5px] font-semibold tracking-[.02em] text-muted-foreground'>{seg.label}</h3>
              {text && <Button variant='outline' size='xs' className='ml-auto' data-copy={text}>{t('Copy')}</Button>}
            </div>
            {text ? (
              <pre tabIndex={0} aria-label={seg.label}
                className='m-0 max-h-80 overflow-auto rounded-md border border-border bg-surface-2 p-[10px_12px] font-mono text-[11.5px] leading-[1.6] whitespace-pre-wrap text-subtle [overflow-wrap:anywhere]'>
                {text}
              </pre>
            ) : <RequestEmpty text={t('This request did not save this payload section')} />}
          </div>
        )
      })}
    </div>
  )
}
