import { useEffect, useState } from 'react'
import { Alert, Button, Dialog, DialogContent, DialogHeader, DialogTitle, DialogBody, DialogFooter, Label, Textarea } from '@ui'
import { SearchableSelect } from './searchable-select'
import { DEFAULT_TITLE_PROMPT } from './assistant-history'
import type { Translate } from './assistant-read-types'

export function AssistantTitleSettings({ open, onOpenChange, model, prompt, models, disabled, onSave, t }: {
  open: boolean
  onOpenChange: (open: boolean) => void
  model: string | null
  prompt: string
  models: string[]
  disabled?: boolean
  onSave: (model: string | null, prompt: string) => void | Promise<void>
  t: Translate
}) {
  const [selected, setSelected] = useState('')
  const [draft, setDraft] = useState('')
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState('')
  useEffect(() => {
    if (open) { setSelected(model || ''); setDraft(prompt || DEFAULT_TITLE_PROMPT); setError('') }
  }, [open, model, prompt])
  return <Dialog open={open} onOpenChange={value => { if (!saving) onOpenChange(value) }}>
    <DialogContent>
      <DialogHeader><DialogTitle>{t('Title generation settings')}</DialogTitle></DialogHeader>
      <DialogBody>
        <Label>{t('Title generation model')}</Label>
        <SearchableSelect value={selected || '__follow__'} disabled={disabled || saving}
          onValueChange={value => setSelected(value === '__follow__' ? '' : value)}
          aria-label={t('Title generation model')} searchPlaceholder={t('Search models')}
          displayValue={selected || t('Follow chat model')} options={[
            { value: '__follow__', label: t('Follow chat model') },
            ...(selected && !models.includes(selected) ? [{ value: selected, label: `${selected} · ${t('Unavailable')}`, disabled: true }] : []),
            ...models.map(name => ({ value: name, label: name })),
          ]} />
        <Label>{t('Title generation prompt')}</Label>
        <Textarea rows={7} value={draft} maxLength={8000} disabled={saving} onChange={event => setDraft(event.currentTarget.value)} />
        <p className='text-xs text-muted-foreground'>{t('Generate a title after the first reply completes. This calls the selected model and consumes quota.')}</p>
        {error ? <Alert variant='destructive'>{error}</Alert> : null}
      </DialogBody>
      <DialogFooter>
        <Button variant='outline' disabled={saving} onClick={() => setDraft(DEFAULT_TITLE_PROMPT)}>{t('Restore Defaults')}</Button>
        <Button variant='outline' disabled={saving} onClick={() => onOpenChange(false)}>{t('Cancel')}</Button>
        <Button disabled={disabled || saving} onClick={async () => {
          setSaving(true); setError('')
          try { await onSave(selected || null, draft.trim() === DEFAULT_TITLE_PROMPT ? '' : draft); onOpenChange(false) }
          catch (failure) { setError(t('Save failed: {message}', { message: failure instanceof Error ? failure.message : String(failure) })) }
          finally { setSaving(false) }
        }}>{t('Save')}</Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
}
