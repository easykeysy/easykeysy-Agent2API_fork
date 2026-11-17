import * as React from 'react'
import {
  Alert, Button, Card, CardContent, CardHeader, CardTitle, Field, FieldDescription, FieldLabel,
  Input, Spinner, Switch,
} from '@ui'
import { SearchableSelect } from './searchable-select'
import { t } from '../ui-kit/src/i18n'
import { emptyImageTarget, type ImageConfigView, type ImageGenerationConfig, type ImageTarget, type ImageToolApi } from './tools-image-types'
import { ToolImageHistory } from './tools-image-history'

function TargetFields({ target, view, editing, onChange, disabled }: {
  target: ImageTarget; view: ImageConfigView; editing: boolean; onChange: (target: ImageTarget) => void; disabled: boolean
}) {
  const supported = view.models.filter(model => editing ? model.editing : model.generation)
  const providers = [...new Map(supported.map(model => [model.provider, model.provider_label || model.provider])).entries()]
  const models = supported.filter(model => model.provider === target.provider)
  const accounts = view.accounts.filter(account => account.provider === target.provider && account.enabled)
  const accountLabel = accounts.find(account => account.id === target.account_id)?.name
    || (target.account_id ? t('The bound account is unavailable; select another account') : t('Select account automatically'))
  return <div className='grid grid-cols-1 gap-3 md:grid-cols-3'>
    <Field>
      <FieldLabel>{t('Upstream provider')}</FieldLabel>
      <SearchableSelect value={target.provider} disabled={disabled} className='w-full'
        aria-label={t('Upstream provider')} searchPlaceholder={t('Search providers')}
        onValueChange={provider => onChange({ provider, model: '', account_id: '' })}
        displayValue={providers.find(([id]) => id === target.provider)?.[1] || target.provider || t('Select a provider')}
        options={[
          ...(target.provider && !providers.some(([id]) => id === target.provider)
            ? [{ value: target.provider, label: `${target.provider}（${t('Invalid')}）`, disabled: true }] : []),
          ...providers.map(([value, label]) => ({ value, label })),
        ]} />
    </Field>
    <Field>
      <FieldLabel>{editing ? t('Editing model') : t('Image generation model')}</FieldLabel>
      <SearchableSelect value={target.model} disabled={disabled || !target.provider} className='w-full'
        aria-label={editing ? t('Editing model') : t('Image generation model')} searchPlaceholder={t('Search models')}
        onValueChange={model => onChange({ ...target, model })} displayValue={target.model || t('Select a model')}
        options={[
          ...(target.model && !models.some(model => model.id === target.model)
            ? [{ value: target.model, label: `${target.model}（${t('Invalid')}）`, disabled: true }] : []),
          ...models.map(model => ({ value: model.id, label: model.id })),
        ]} />
    </Field>
    <Field>
      <FieldLabel>{t('Image generation account')}</FieldLabel>
      <SearchableSelect value={target.account_id || '__auto__'} disabled={disabled || !target.provider} className='w-full'
        aria-label={t('Image generation account')} searchPlaceholder={t('Search accounts…')} displayValue={accountLabel}
        onValueChange={value => onChange({ ...target, account_id: value === '__auto__' ? '' : value })}
        options={[
          { value: '__auto__', label: t('Select account automatically') },
          ...(target.account_id && !accounts.some(account => account.id === target.account_id)
            ? [{ value: target.account_id, label: accountLabel, disabled: true }] : []),
          ...accounts.map(account => ({ value: account.id, label: account.name })),
        ]} />
    </Field>
  </div>
}

export function ImageGenerationPanel({ api, onChanged, showTimeout = false }: {
  api: ImageToolApi; onChanged?: () => void; showTimeout?: boolean
}) {
  const [view, setView] = React.useState<ImageConfigView | null>(null)
  const [draft, setDraft] = React.useState<ImageGenerationConfig | null>(null)
  const [loading, setLoading] = React.useState(true)
  const [busy, setBusy] = React.useState(false)
  const [error, setError] = React.useState('')
  const [notice, setNotice] = React.useState('')
  const [timeout, setTimeoutValue] = React.useState(300)

  React.useEffect(() => {
    let alive = true
    api.load().then(result => {
      if (alive) { setView(result); setDraft(result.config || result.defaults); setTimeoutValue(result.timeout_seconds) }
    }).catch(failure => { if (alive) setError(t(failure instanceof Error ? failure.message : String(failure))) })
      .finally(() => { if (alive) setLoading(false) })
    return () => { alive = false }
  }, [api])

  const save = async () => {
    if (!draft) return
    if (showTimeout && (!Number.isInteger(timeout) || timeout < 5 || timeout > 600)) {
      setError(t('Invalid request parameters')); return
    }
    setBusy(true); setError(''); setNotice('')
    try {
      const next = await api.save(draft)
      setView(next); setDraft(next.config || next.defaults)
      onChanged?.()
      if (showTimeout && api.saveTimeout && timeout !== next.timeout_seconds) {
        await api.saveTimeout(timeout)
        setView(current => current ? { ...current, timeout_seconds: timeout } : current)
        onChanged?.()
      }
      setNotice(t('Image generation settings saved'))
    } catch (failure) { setError(t(failure instanceof Error ? failure.message : String(failure))) }
    finally { setBusy(false) }
  }
  const check = async () => {
    setBusy(true); setError(''); setNotice('')
    try {
      const next = await api.load()
      setView(next)
      if (!draft) {
        setDraft(next.config || next.defaults)
        setTimeoutValue(next.timeout_seconds)
      }
      setNotice(t('Saved settings checked without generating an image. Actual account quota and upstream availability are determined when making a call.'))
    } catch (failure) { setError(t(failure instanceof Error ? failure.message : String(failure))) }
    finally { setBusy(false) }
  }
  const setNumber = (key: 'max_images' | 'max_concurrency' | 'retention_hours' | 'storage_mb', value: string) => {
    setDraft(current => current ? { ...current, [key]: Number(value) } : current)
  }

  if (loading) return <div className='flex items-center gap-2 p-4 text-sm text-subtle'><Spinner />{t('Loading tool service configuration…')}</div>
  return <div className='flex flex-col gap-4'>
    <Card>
      <CardHeader>
        <CardTitle>{t('Image generation settings')}</CardTitle>
        <Button className='ml-auto' variant='outline' size='sm' disabled={busy} onClick={() => void check()}>{busy ? <Spinner /> : null}{t('Check configuration')}</Button>
      </CardHeader>
      <CardContent className='flex flex-col gap-4'>
        <p className='text-xs text-subtle'>{t('Generate images with existing accounts without changing the agent\'s chat model. Calls follow the connecting key\'s model and provider permissions.')}</p>
        {!view?.config ? <Alert variant='warning'>{t('Image generation is not configured. Select an available image model and save.')}</Alert> : null}
        {view?.generation_error ? <Alert variant='warning'>{t(view.generation_error)}</Alert> : null}
        {view?.editing_error ? <Alert variant='warning'>{t(view.editing_error)}</Alert> : null}
        {error ? <Alert variant='destructive'>{error}</Alert> : null}
        {notice ? <Alert variant='success'>{notice}</Alert> : null}
        {draft && view ? <>
          <TargetFields target={draft.generation} view={view} editing={false} disabled={busy}
            onChange={generation => setDraft(current => current ? { ...current, generation } : current)} />
          <p className='text-xs text-subtle'>{t('Automatic selection follows existing routing within the selected provider. Fixed accounts still obey disabled status, cooldowns and quota limits; they are never replaced automatically.')}</p>
          {!view.models.some(model => model.generation) ? <Alert variant='warning'>{t('No image model is available. Add an account, enable its plugin and refresh the model catalog.')}</Alert> : null}
          <div className='flex items-center gap-2'>
            <span className='text-sm font-medium'>{t('Reference-image editing')}</span>
            <Switch checked={Boolean(draft.editing)} disabled={busy} onCheckedChange={checked => setDraft(current => current
              ? { ...current, editing: checked ? emptyImageTarget() : null } : current)} aria-label={t('Reference-image editing')} />
          </div>
          {draft.editing ? <TargetFields target={draft.editing} view={view} editing disabled={busy}
            onChange={editing => setDraft(current => current ? { ...current, editing } : current)} /> : null}
          <p className='m-0 text-[11.5px] leading-relaxed text-subtle'>{t('Editing can use a different model. The editing tool is not exposed without an available editing model.')}</p>
          <div className='grid grid-cols-2 gap-3 md:grid-cols-4'>
            <Field><FieldLabel>{t('Images per request')}</FieldLabel><Input type='number' min={1} max={10} step={1} value={draft.max_images} disabled={busy} onChange={event => setNumber('max_images', event.currentTarget.value)} /></Field>
            <Field><FieldLabel>{t('Concurrent generation limit')}</FieldLabel><Input type='number' min={1} max={8} step={1} value={draft.max_concurrency} disabled={busy} onChange={event => setNumber('max_concurrency', event.currentTarget.value)} /></Field>
            <Field><FieldLabel>{t('Retention (hours)')}</FieldLabel><Input type='number' min={1} max={168} step={1} value={draft.retention_hours} disabled={busy} onChange={event => setNumber('retention_hours', event.currentTarget.value)} /></Field>
            <Field><FieldLabel>{t('Storage limit (MiB)')}</FieldLabel><Input type='number' min={64} max={4096} step={1} value={draft.storage_mb} disabled={busy} onChange={event => setNumber('storage_mb', event.currentTarget.value)} /></Field>
          </div>
          <p className='text-xs text-subtle'>{t('One image is generated by default. Insufficient storage rejects the request before credits are spent. Only expired results are cleaned up; locally saved files are unaffected.')}</p>
          {showTimeout ? <Field>
            <FieldLabel>{t('Tool execution timeout')}（{t('seconds')}）</FieldLabel>
            <Input type='number' min={5} max={600} step={1} value={timeout} disabled={busy} onChange={event => setTimeoutValue(Number(event.currentTarget.value))} />
            <FieldDescription>{t('This limits the background generation task. Each MCP call waits up to 20 seconds, then returns a job ID if unfinished. Queries never generate again.')}</FieldDescription>
          </Field> : null}
          <Alert>{t('Slow generations return a job ID first. Query the original job after reconnecting; no automatic regeneration. A timeout does not mean no credits were charged.')}</Alert>
          <div><Button disabled={busy || !draft.generation.model || Boolean(draft.editing && !draft.editing.model)} onClick={() => void save()}>{busy ? <Spinner /> : null}{t('Save')}</Button></div>
        </> : null}
      </CardContent>
    </Card>
    <ToolImageHistory api={api} />
  </div>
}
