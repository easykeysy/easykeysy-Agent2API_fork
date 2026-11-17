import { useEffect, useMemo, useState } from 'react'
import { Alert, Button, Empty, InputGroup, InputGroupAddon, InputGroupInput, Sheet, SheetBody, SheetContent, SheetHeader, SheetTitle } from '@ui'
import { selectedModel, type ModelOption } from '../../../ui-shared/assistant-models'
import { Icon } from './Icon'
import { t } from '../i18n'

type Props = {
  open: boolean
  onClose: () => void
  models: ModelOption[]
  current: string
  defaultModel: string
  followingDefault: boolean
  ready: boolean
  loading: boolean
  warning: string
  onRefresh: () => void
  onPick: (value: string) => void
}

export function AssistantModelPicker({
  open, onClose, models, current, defaultModel, followingDefault, ready, loading, warning, onRefresh, onPick,
}: Props) {
  const [query, setQuery] = useState('')
  useEffect(() => { if (!open) setQuery('') }, [open])
  const filtered = useMemo(() => {
    const keyword = query.trim().toLowerCase()
    return keyword ? models.filter(item => item.name.toLowerCase().includes(keyword)) : models
  }, [models, query])
  const currentOption = selectedModel(models, current)
  const defaultAvailable = Boolean(selectedModel(models, defaultModel))

  return (
    <Sheet open={open} onOpenChange={next => { if (!next) onClose() }}>
      <SheetContent>
        <SheetHeader>
          <SheetTitle>{t('Choose chat model')}</SheetTitle>
          <Button size='icon-sm' variant='ghost' disabled={loading} onClick={onRefresh}
            title={t('Refresh list')} aria-label={t('Refresh list')}><Icon name='refresh' /></Button>
        </SheetHeader>
        <SheetBody>
          <InputGroup>
            <InputGroupInput value={query} onChange={event => setQuery(event.currentTarget.value)}
              placeholder={t('Search models…')} aria-label={t('Search models…')} />
            <InputGroupAddon><Icon name='search' style={{ width: 16, height: 16 }} /></InputGroupAddon>
          </InputGroup>
          {warning && !loading ? <Alert variant='destructive' role='status'>{warning}</Alert> : null}
          <Button size='default' variant='ghost' type='button' aria-pressed={followingDefault}
            disabled={!ready || !defaultAvailable}
            className='m-row h-auto min-h-[52px] w-full justify-start rounded-none border-0 px-3.5 py-3 text-left'
            onClick={() => onPick('')}
            style={{
              borderRadius: 'var(--m-r-md)',
              border: '1px solid var(--m-line)',
              background: followingDefault ? 'var(--m-primary-soft)' : 'var(--m-surface)',
            }}
          >
            <span className='m-logo'><Icon name='settings' /></span>
            <div className='m-grow'>
              <div className='m-t1'>{t('Use gateway setting')}</div>
              <div className='m-t2 m-mono' style={{ fontSize: 11.5 }}>
                {defaultModel || t('No default model configured yet')}
                {defaultModel && ready && !defaultAvailable ? ` · ${t('Unavailable')}` : ''}
              </div>
            </div>
            {followingDefault ? <Icon name='check' style={{ width: 16, height: 16, color: 'var(--m-primary)' }} /> : null}
          </Button>
          {filtered.length ? filtered.map(item => {
            const picked = !followingDefault && currentOption?.name === item.name
            return (
              <Button size='default' variant='ghost' key={item.name} type='button' aria-pressed={picked}
                disabled={!ready}
                className='m-row h-auto min-h-[44px] w-full justify-start rounded-none border-0 px-3.5 py-2.5 text-left'
                onClick={() => onPick(item.name)}
                style={{
                  borderRadius: 'var(--m-r-md)',
                  border: '1px solid var(--m-line)',
                  background: picked ? 'var(--m-primary-soft)' : 'var(--m-surface)',
                }}
              >
                <span className='m-grow m-t1 m-mono min-w-0 truncate text-[12.5px]' title={item.name}>{item.name}</span>
                {picked ? <Icon name='check' style={{ width: 16, height: 16, color: 'var(--m-primary)' }} /> : null}
              </Button>
            )
          }) : <Empty>
            {loading ? t('Loading available models…') : query.trim() ? t('No matching models') : t('No models available')}
          </Empty>}
          <p className='text-[11.5px] leading-[1.7] text-subtle'>
            {t('This only changes the model for this session; the default saved in the gateway is untouched.')}
          </p>
        </SheetBody>
      </SheetContent>
    </Sheet>
  )
}
