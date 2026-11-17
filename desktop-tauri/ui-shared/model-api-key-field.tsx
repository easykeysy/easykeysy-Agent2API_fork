import { Alert, Button, Label } from '@ui'
import { SearchableSelect } from './searchable-select'
import { modelApiKeyLabel, type ModelApiKey } from './model-api-key'

type Props = {
  keys: ModelApiKey[]
  value: string | null
  ready: boolean
  loading: boolean
  error: string
  disabled?: boolean
  rememberSelection?: boolean
  id?: string
  t: (text: string) => string
  onChange: (id: string) => void
  onRefresh: () => void
}

export function ModelApiKeyField({ keys, value, ready, loading, error, disabled, rememberSelection, id = 'model-api-key', t, onChange, onRefresh }: Props) {
  const selected = keys.find(key => key.id === value)
  const label = selected ? modelApiKeyLabel(selected)
    : value ? t('The selected API Key is unavailable') : t('Select API Key…')
  const warning = error || (!ready ? t('Loading API keys…')
    : !keys.length ? t('No API key is available. Create and enable one on the Gateway Key page.')
    : value && !selected ? t('The selected API Key is unavailable. Select another key.') : '')
  return (
    <div className='flex flex-col gap-1.5'>
      <div className='flex items-center gap-2'>
        <Label htmlFor={id}>API Key</Label>
        <Button variant='ghost' size='xs' className='ml-auto' disabled={disabled || loading} onClick={onRefresh}>{t('Refresh')}</Button>
      </div>
      <SearchableSelect value={selected?.id ?? ''} disabled={disabled || !ready || !keys.length}
        onValueChange={next => { if (next) onChange(next) }} id={id} className='w-full' displayValue={label}
        options={keys.map(key => ({ value: key.id, label: modelApiKeyLabel(key) }))} />
      <p className='text-xs leading-[1.6] text-subtle'>{t('Model calls use the selected key\'s permissions. Logs record its name, never the secret value.')}</p>
      {rememberSelection ? <p className='text-xs leading-[1.6] text-subtle'>{t('Your selection is remembered for next time. Select another key if it is disabled or deleted.')}</p> : null}
      {warning ? <Alert variant='warning' role='status'>{warning}</Alert> : null}
    </div>
  )
}
