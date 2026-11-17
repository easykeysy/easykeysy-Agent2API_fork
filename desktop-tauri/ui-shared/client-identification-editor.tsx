import { useEffect, useRef, useState } from 'react'
import { Alert, Button, Dialog, DialogBody, DialogContent, DialogFooter, DialogHeader, DialogTitle, Input, Label, Select, SelectContent, SelectItem, SelectTrigger, SelectValue, Switch } from '@ui'
import type { Translate } from './assistant-read-types'
import { blankRule, clientSettingsError, notifyIdentificationChanged, randomId, type ClientEvidence, type ClientIdentificationApi, type ClientIdentificationView, type ClientRule, type ClientSettings, type IdentificationPreview } from './client-identification'

const NEW_CLIENT = '__new_client__'

export function ClientRuleEditor({ api, view: initialView, rule, evidence, onClose, onSaved, t }: {
  api: ClientIdentificationApi
  view: ClientIdentificationView
  rule?: ClientRule
  evidence?: ClientEvidence
  onClose: () => void
  onSaved: (view: ClientIdentificationView) => void
  t: Translate
}) {
  const [view, setView] = useState(initialView)
  const [draft, setDraft] = useState<ClientRule>(() => rule ? { ...rule } : blankRule(evidence))
  const [name, setName] = useState(evidence?.candidate || '')
  const [customId] = useState(() => `custom:${randomId()}`)
  const [preview, setPreview] = useState<IdentificationPreview | null>(null)
  const [busy, setBusy] = useState<'preview' | 'save' | null>(null)
  const [error, setError] = useState('')
  const alive = useRef(true)
  useEffect(() => { alive.current = true; return () => { alive.current = false } }, [])
  const clients = [...view.builtins, ...view.config.clients]
  const clientName = draft.clientId === NEW_CLIENT ? t('New client category') : clients.find(client => client.id === draft.clientId)?.name || t('Select the client to assign')
  const update = (patch: Partial<ClientRule>) => { setDraft(previous => ({ ...previous, ...patch })); setPreview(null); setError('') }
  const config = (): ClientSettings => {
    const next = { ...draft, pattern: draft.pattern.trim(), clientId: draft.clientId === NEW_CLIENT ? customId : draft.clientId }
    return {
      ...view.config,
      clients: draft.clientId === NEW_CLIENT ? [...view.config.clients, { id: customId, name: name.trim() }] : view.config.clients,
      // 新规则放在自定义规则最前面；预览使用的顺序与保存后的顺序完全相同。
      rules: rule ? view.config.rules.map(item => item.id === rule.id ? next : item) : [next, ...view.config.rules],
    }
  }
  const execute = async (operation: 'preview' | 'save') => {
    if (rule && !view.config.rules.some(item => item.id === rule.id)) { setError(t('This rule was deleted. Close the editor and add it again.')); return }
    const next = config()
    const invalid = clientSettingsError(next, view, t)
    if (invalid) { setError(invalid); return }
    setBusy(operation); setError('')
    try {
      if (operation === 'preview') {
        const result = await api.preview({ config: next, ruleId: draft.id, sample: evidence })
        if (alive.current) setPreview(result)
      } else {
        const saved = await api.save({ config: next, revision: view.revision })
        notifyIdentificationChanged()
        if (alive.current) onSaved(saved)
      }
    } catch (failure) {
      if (alive.current) setError(failure instanceof Error ? failure.message : String(failure))
    } finally { if (alive.current) setBusy(null) }
  }
  const modes = [
    ...(draft.header === 'user-agent' ? [{ value: 'product', label: t('Product name') }] : []),
    { value: 'prefix', label: t('Prefix') }, { value: 'contains', label: t('Contains') }, { value: 'exact', label: t('Exact match') },
  ]
  return <Dialog open onOpenChange={open => { if (!open && !busy) onClose() }}>
    <DialogContent className='w-[min(640px,calc(100vw-24px))]'>
      <DialogHeader><DialogTitle>{rule ? t('Edit identification rule') : t('Set identification rule')}</DialogTitle></DialogHeader>
      <DialogBody>
        <p className='max-w-[62ch] text-[12px] leading-[1.7] text-muted-foreground'>{t('Rules ignore case and affect only logs and statistics for new requests, not model selection or permissions.')}</p>
        {/* 与面板同一套行形态：底色分层而不是描边，开关在左、状态右对齐 */}
        <div className='flex items-center gap-3 rounded-lg bg-surface-inset px-4 py-3'>
          <Switch checked={draft.enabled} disabled={Boolean(busy)} onCheckedChange={enabled => update({ enabled })} aria-label={t('Enable rule')} />
          <span className='text-[13px]'>{t('Enable rule')}</span>
          <span className='ml-auto text-[12px] text-muted-foreground'>{draft.enabled ? t('Enabled') : t('Disabled')}</span>
        </div>
        <Label>{t('Identification evidence')}</Label>
        <Select value={draft.header} disabled={Boolean(busy)} onValueChange={value => update({ header: String(value), matchType: value === 'user-agent' ? 'product' : 'exact' })}>
          <SelectTrigger aria-label={t('Identification evidence')}><SelectValue>{draft.header}</SelectValue></SelectTrigger>
          <SelectContent>{view.headers.map(header => <SelectItem key={header} value={header}>{header}</SelectItem>)}</SelectContent>
        </Select>
        <div className='grid grid-cols-[120px_minmax(0,1fr)] gap-3'>
          <div className='flex flex-col gap-2'><Label>{t('Match type')}</Label>
            <Select value={draft.matchType} disabled={Boolean(busy)} onValueChange={value => update({ matchType: value as ClientRule['matchType'] })}>
              <SelectTrigger aria-label={t('Match type')}><SelectValue>{modes.find(mode => mode.value === draft.matchType)?.label}</SelectValue></SelectTrigger>
              <SelectContent>{modes.map(mode => <SelectItem key={mode.value} value={mode.value}>{mode.label}</SelectItem>)}</SelectContent>
            </Select>
          </div>
          <div className='flex flex-col gap-2'><Label>{t('Match pattern')}</Label><Input value={draft.pattern} maxLength={256} disabled={Boolean(busy)} onChange={event => update({ pattern: event.currentTarget.value })} aria-label={t('Match pattern')} placeholder={draft.header === 'user-agent' ? 'magpie' : 'Cline'} /></div>
        </div>
        <Label>{t('Assigned client')}</Label>
        <Select value={draft.clientId || '__choose__'} disabled={Boolean(busy)} onValueChange={value => update({ clientId: String(value) })}>
          <SelectTrigger aria-label={t('Assigned client')}><SelectValue>{clientName}</SelectValue></SelectTrigger>
          <SelectContent><SelectItem value='__choose__' disabled>{t('Select the client to assign')}</SelectItem>
            {clients.map(client => <SelectItem key={client.id} value={client.id}>{client.name}</SelectItem>)}
            <SelectItem value={NEW_CLIENT}>{t('New client category')}</SelectItem>
          </SelectContent>
        </Select>
        {draft.clientId === NEW_CLIENT ? <Input value={name} maxLength={80} disabled={Boolean(busy)} onChange={event => { setName(event.currentTarget.value); setPreview(null); setError('') }} aria-label={t('Client name')} placeholder={t('Client name')} /> : null}
        {draft.header === view.declarationHeader && !view.config.acceptDeclarations ? <Alert>{t('Caller declarations are disabled, so this rule will not apply. Enable them in client identification settings first.')}</Alert> : null}
        {error ? <Alert variant='destructive'><p>{error}</p><Button size='sm' variant='outline' disabled={Boolean(busy)} onClick={async () => {
          setBusy('preview')
          try { const next = await api.load(); if (alive.current) { setView(next); setPreview(null); setError('') } }
          catch (failure) { if (alive.current) setError(failure instanceof Error ? failure.message : String(failure)) }
          finally { if (alive.current) setBusy(null) }
        }}>{t('Keep draft and reload settings')}</Button></Alert> : null}
        {preview ? <div className='flex flex-col gap-2 rounded-lg bg-surface-inset p-3 text-[12px] leading-[1.7]'>
          <p>{t('Of the latest {window} requests, {scanned} have evidence, {matched} match, and {applied} would use this rule.', { window: preview.window, scanned: preview.scanned, matched: preview.matched, applied: preview.applied })}</p>
          {preview.sample ? <p>{t('Current sample result: {name}', { name: preview.sample.clientLabel || preview.sample.evidence.candidate || t('Unknown client') })}
            {!preview.sample.ruleApplied ? ` · ${t('This rule was not used. Check its switch, caller declarations, or rule order.')}` : ''}</p> : null}
          {preview.samples.slice(0, 5).map((sample, index) => <div key={index} className='break-all border-t border-hairline pt-2'>
            <div>{sample.evidence.userAgent || sample.evidence.headers[draft.header] || '—'}</div>
            <div className='text-muted-foreground'>{sample.clientLabel || t('Unknown client')} · {sample.ruleApplied ? t('Uses this rule') : t('Overridden by higher priority')}</div>
          </div>)}
          {!preview.matched ? <p>{t('No recent samples matched; future requests may still match.')}</p> : null}
          <p className='text-muted-foreground'>{t('Preview does not call a model or modify historical records.')}</p>
        </div> : null}
      </DialogBody>
      <DialogFooter>
        <Button variant='ghost' disabled={Boolean(busy)} onClick={onClose}>{t('Cancel')}</Button>
        <Button variant='outline' disabled={Boolean(busy)} onClick={() => void execute('preview')}>{busy === 'preview' ? t('Previewing…') : t('Preview matches')}</Button>
        <Button disabled={Boolean(busy) || !preview} onClick={() => void execute('save')}>{busy === 'save' ? t('Saving…') : t('Save rule')}</Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
}
