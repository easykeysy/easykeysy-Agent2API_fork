import { useEffect, useRef, useState } from 'react'
import { Alert, Badge, Button } from '@ui'
import type { Translate } from './assistant-read-types'
import { requestClientLabel, type RequestIdentity } from './model-api-key'
import { ClientRuleEditor } from './client-identification-editor'
import { clientMethodLabel, type ClientIdentificationApi, type ClientIdentificationView } from './client-identification'

export function ClientIdentificationDetail({ entry, api, t }: { entry: RequestIdentity; api: ClientIdentificationApi; t: Translate }) {
  const [view, setView] = useState<ClientIdentificationView | null>(null)
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState('')
  const [saved, setSaved] = useState(false)
  const alive = useRef(true)
  useEffect(() => { alive.current = true; return () => { alive.current = false } }, [])
  const evidence = entry.clientEvidence
  if (entry.isTest && !evidence) return <span>{requestClientLabel(entry, t)}</span>
  return <div className='space-y-2 text-xs'>
    <div className='flex flex-wrap items-center gap-2'><Badge variant='outline'>{requestClientLabel(entry, t)}</Badge><span className='text-muted-foreground'>{clientMethodLabel(evidence?.method, t)}</span></div>
    {evidence ? <>
      <div className='break-all'><span className='text-muted-foreground'>User-Agent: </span>{evidence.userAgent || '—'}</div>
      {evidence.userAgentTruncated ? <p className='text-muted-foreground'>{t('The software identifier exceeded 1 KB and was truncated. Matching uses only the retained portion.')}</p> : null}
      {Object.entries(evidence.headers || {}).map(([header, value]) => <div key={header} className='break-all'><span className='text-muted-foreground'>{header}: </span>{value}{evidence.truncatedHeaders?.includes(header) ? ` · ${t('Truncated')}` : ''}</div>)}
      {evidence.invalidHeaders?.length ? <p className='break-all text-muted-foreground'>{t('Invalid or duplicate identity headers: {headers}', { headers: evidence.invalidHeaders.join(', ') })}</p> : null}
      {evidence.matchedHeader ? <p className='break-all text-muted-foreground'>{t('Matched evidence: {header} · {pattern}', { header: evidence.matchedHeader, pattern: evidence.matchedPattern })}</p> : null}
      {evidence.ruleId ? <p className='break-all text-muted-foreground'>{t('Rule ID: {id}', { id: evidence.ruleId })}</p> : null}
      {evidence.declarationStatus && evidence.declarationStatus !== 'accepted' ? <p className='text-muted-foreground'>{evidence.declarationStatus === 'disabled' ? t('Origin declarations are disabled and were ignored.') : t('The origin declaration was invalid or unregistered and was ignored.')}</p> : null}
      {evidence.method === 'ambiguous' ? <p className='text-muted-foreground'>{t('The request contains identifiers for different applications and was not classified automatically. Use a custom rule to assign it.')}</p> : null}
      <Button size='sm' variant='outline' disabled={busy} onClick={async () => {
        setBusy(true); setError(''); setSaved(false)
        try { const next = await api.load(); if (alive.current) setView(next) }
        catch (failure) { if (alive.current) setError(failure instanceof Error ? failure.message : String(failure)) }
        finally { if (alive.current) setBusy(false) }
      }}>{busy ? t('Loading…') : t('Set identification rule')}</Button>
    </> : <p className='text-muted-foreground'>{t('This historical request has no saved identification evidence, so it cannot be used to create a rule automatically.')}</p>}
    {error ? <Alert variant='destructive'>{error}</Alert> : null}
    {saved ? <p className='text-muted-foreground'>{t('Rule saved for future requests. This historical record was not changed.')}</p> : null}
    {view && evidence ? <ClientRuleEditor api={api} view={view} evidence={evidence} t={t} onClose={() => setView(null)} onSaved={() => { setView(null); setSaved(true) }} /> : null}
  </div>
}
