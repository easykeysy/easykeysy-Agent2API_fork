import * as React from 'react'
import { Alert, Button, Card, CardContent, CardHeader, CardTitle, Input, Label } from '@ui'
import { BrandLogo } from '../../../ui-shared/brand-logo'
import { DEFAULT_BRANDING, MAX_LOGO_BYTES } from '../../../ui-shared/branding'
import { desktopBranding, saveBranding } from '../branding'
import { t } from '../i18n'
import { errorMessage, toast } from './settings-model'

export function BrandingSettingsPanel({ disabled = false }: { disabled?: boolean }) {
  const branding = desktopBranding.useValue()
  const [name, setName] = React.useState(branding.displayName)
  const [logo, setLogo] = React.useState<string | null | undefined>(undefined)
  const [ready, setReady] = React.useState(false)
  const [reload, setReload] = React.useState(0)
  const [dirty, setDirty] = React.useState(false)
  const [busy, setBusy] = React.useState(false)
  const [error, setError] = React.useState('')
  const pending = React.useRef(false)
  const file = React.useRef<HTMLInputElement>(null)
  const id = React.useId()

  React.useEffect(() => {
    let alive = true
    setError('')
    void desktopBranding.refresh().then(value => {
      if (alive) { setName(value.displayName); setReady(true) }
    }).catch(failure => { if (alive) setError(errorMessage(failure)) })
    return () => { alive = false }
  }, [reload])

  React.useEffect(() => {
    if (!dirty) setName(branding.displayName)
  }, [branding.displayName, dirty])

  async function pick(selected?: File) {
    if (!selected || pending.current || disabled) return
    setError('')
    if (!['image/png', 'image/jpeg', 'image/webp'].includes(selected.type)) {
      setError(t('Only PNG, JPG and WebP images are supported for logos'))
      return
    }
    if (!selected.size || selected.size > MAX_LOGO_BYTES) {
      setError(t('The logo must not exceed 2 MiB'))
      return
    }
    pending.current = true
    setBusy(true)
    try {
      const data = await new Promise<string>((resolve, reject) => {
        const reader = new FileReader()
        reader.onload = () => typeof reader.result === 'string' ? resolve(reader.result) : reject(new Error(t('Could not read the image. Select it again.')))
        reader.onerror = () => reject(new Error(t('Could not read the image. Select it again.')))
        reader.readAsDataURL(selected)
      })
      setLogo(data)
      setDirty(true)
    } catch (failure) { setError(errorMessage(failure)) }
    finally { pending.current = false; setBusy(false) }
  }

  async function save() {
    if (pending.current || disabled || !ready) return
    if (Array.from(name.trim()).length > 64) { setError(t('The display name can contain up to 64 characters')); return }
    pending.current = true
    setBusy(true)
    setError('')
    try {
      const value = await saveBranding({ displayName: name.trim(), ...(logo !== undefined ? { logo } : {}) })
      setName(value.displayName)
      setLogo(undefined)
      setDirty(false)
      setReady(true)
      toast(t('Brand settings saved'))
    } catch (failure) { setError(errorMessage(failure)) }
    finally { pending.current = false; setBusy(false) }
  }

  const preview = { displayName: name.trim() || DEFAULT_BRANDING.displayName, logoUrl: logo === undefined ? branding.logoUrl : logo }
  const locked = disabled || busy || !ready
  return <Card render={<section />} className='panel'>
    <CardHeader><CardTitle>{t('Brand settings')}</CardTitle></CardHeader>
    <CardContent className='panel-body flex flex-col gap-4'>
      <p className='m-0 text-xs leading-relaxed text-muted-foreground'>{t('Customize the name and logo displayed on the website and in the application.')}</p>
      <div className='flex min-w-0 items-center gap-4 rounded-lg border border-hairline bg-surface-inset p-4'>
        <BrandLogo branding={preview} size={48} />
        <div className='min-w-0'><div className='truncate text-base font-semibold'>{preview.displayName}</div>
          <div className='text-xs text-muted-foreground'>{t('Brand preview')}</div></div>
      </div>
      <div className='space-y-2'>
        <Label htmlFor={`${id}-name`}>{t('Display name')}</Label>
        <Input id={`${id}-name`} value={name} disabled={locked} autoComplete='off' placeholder='Agent2API'
          onChange={event => { setName(event.target.value); setDirty(true) }} />
        <p className='m-0 text-xs text-muted-foreground'>{t('Up to 64 characters. Leave blank to restore Agent2API.')}</p>
      </div>
      <div className='space-y-2'>
        <Label htmlFor={`${id}-logo`}>{t('Brand logo')}</Label>
        <input id={`${id}-logo`} ref={file} type='file' className='sr-only' accept='image/png,image/jpeg,image/webp' disabled={locked}
          onChange={event => { const selected = event.target.files?.[0]; event.target.value = ''; void pick(selected) }} />
        <div className='flex flex-wrap gap-2'>
          <Button variant='outline' disabled={locked} onClick={() => file.current?.click()}>{t('Upload logo')}</Button>
          <Button variant='ghost' disabled={locked || !preview.logoUrl} onClick={() => { setLogo(null); setDirty(true) }}>{t('Restore default logo')}</Button>
        </div>
        <p className='m-0 text-xs leading-relaxed text-muted-foreground'>{t('Supports PNG, JPG and WebP up to 2 MiB and 4096 × 4096 pixels. Images are resized to fit within 512 pixels. Square images are recommended.')}</p>
      </div>
      <p className='m-0 text-xs leading-relaxed text-muted-foreground'>{t('Saved on the current gateway and displayed publicly on the sign-in page. Does not change the installer, shortcuts, system tray icon or software version information.')}</p>
      {error ? <Alert variant='destructive' className='break-words'>{error}</Alert> : null}
      {!ready && error ? <Button variant='outline' disabled={disabled} onClick={() => setReload(value => value + 1)}>{t('Retry')}</Button> : null}
      <div className='flex items-center justify-end gap-2'>
        <Button variant='ghost' disabled={locked} onClick={() => { setName(DEFAULT_BRANDING.displayName); setLogo(null); setDirty(true); setError('') }}>{t('Restore default brand')}</Button>
        <Button disabled={locked} onClick={() => void save()}>{busy ? t('Saving…') : t('Save')}</Button>
      </div>
    </CardContent>
  </Card>
}
