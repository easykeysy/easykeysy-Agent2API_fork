import { useEffect, useState } from 'react'
import { Button, Input, Select, SelectContent, SelectItem, SelectTrigger, SelectValue, Sheet, SheetBody, SheetContent, SheetHeader, SheetTitle, Switch , Label } from '@ui'
import { getPluginSettings, savePluginSettings, type PluginSettings } from '../../api/plugins'
import { GithubAccessSettingsPanel } from '../../components/GithubAccessSettings'
import { serverIdentity } from '../../lib/servers'
import { store } from '../../state/store'
import { t } from '../../i18n'

export function PluginSettingsSheet({ onClose, onChanged }: { onClose: () => void; onChanged: () => Promise<void> }) {
  const [settings, setSettings] = useState<PluginSettings | null>(null)
  const [draft, setDraft] = useState('')
  const [error, setError] = useState('')
  const [busy, setBusy] = useState(false)
  const identity = serverIdentity()
  const modes = [{ value: 'follow', label: t('Follow software settings') }, { value: 'custom', label: t('Configure separately') }] as const

  useEffect(() => {
    const controller = new AbortController()
    void getPluginSettings(controller.signal).then(next => {
      if (!controller.signal.aborted) { setSettings(next); setDraft(String(next.interval)) }
    }).catch(cause => {
      if (!controller.signal.aborted) setError(cause instanceof Error ? cause.message : String(cause))
    })
    return () => controller.abort()
  }, [])

  async function save(patch: Partial<PluginSettings>) {
    if (busy || !settings) return
    const next = { ...settings, ...patch }
    if (patch.channelMode !== 'follow' && !next.stable && !next.beta) {
      setError(t('Follow at least one channel: stable or beta.'))
      return
    }
    setBusy(true)
    try {
      const next = await savePluginSettings(patch)
      if (serverIdentity() !== identity) return
      setSettings(next)
      setDraft(String(next.interval))
      setError('')
      store.toast(next.saved === false ? t('Settings applied, but could not be saved to disk (will revert after restart)') : t('Plugin settings saved'), next.saved === false ? 'err' : 'ok')
      if ('beta' in patch || 'stable' in patch || 'channelMode' in patch) await onChanged()
    } catch (cause) {
      if (serverIdentity() === identity) setError(cause instanceof Error ? cause.message : String(cause))
    } finally { setBusy(false) }
  }

  const saveInterval = () => {
    const interval = Number(draft)
    if (!Number.isInteger(interval) || interval < 1 || interval > 1440) {
      setError(t('The check interval must be {min}–{max} minutes', { min: 1, max: 1440 }))
      return
    }
    void save({ interval })
  }
  const channelLabel = settings?.stable && settings.beta ? t('Stable and beta')
    : settings?.beta ? t('Beta only') : t('Stable only')

  return <Sheet open onOpenChange={(open, details) => {
    if (open) return
    if (busy) { details.cancel(); return }
    onClose()
  }}>
    <SheetContent showCloseButton={!busy}>
      <SheetHeader><SheetTitle>{t('Plugin settings')}</SheetTitle></SheetHeader>
      <SheetBody>
        <p className='m-ck-note'>{t('These settings belong to the connected gateway, not this phone.')}</p>
        <div className='m-ck-title'>{t('Check schedule')}</div>
        <Select value={settings?.checkMode || 'follow'} disabled={busy || !settings}
          onValueChange={value => void save({ checkMode: value as PluginSettings['checkMode'] })}>
          <SelectTrigger className='w-full' aria-label={t('Check schedule')}>
            <SelectValue>{modes.find(mode => mode.value === (settings?.checkMode || 'follow'))?.label}</SelectValue>
          </SelectTrigger>
          <SelectContent>{modes.map(mode => <SelectItem key={mode.value} value={mode.value}>{mode.label}</SelectItem>)}</SelectContent>
        </Select>
        {settings?.checkMode === 'custom' ? <>
          <div className='m-ck-toolbar'>
            <span className='m-ck-title m-ck-grow'>{t('Automatically check plugin updates')}</span>
            <Switch checked={settings.autoCheck} disabled={busy} aria-label={t('Automatically check plugin updates')}
              onCheckedChange={value => void save({ autoCheck: value === true })} />
          </div>
          <Label className='m-ck-field'>{t('Check interval (minutes)')}
            <Input type='number' min={1} max={1440} step={1} className='text-base' value={draft} disabled={busy || !settings.autoCheck}
              onChange={event => setDraft(event.currentTarget.value)} />
          </Label>
          <Button variant='outline' disabled={busy || !settings.autoCheck || Number(draft) === settings.interval} onClick={saveInterval}>{t('Save')}</Button>
        </> : <p className='m-ck-note'>{settings ? settings.autoCheck ? t('Automatic checks are on, every {n} minutes', { n: settings.interval })
          : t('Automatic checks are off') : t('Loading')}</p>}
        <p className='m-ck-note'>{t('Automatic checks and intervals follow the app by default. Turning off automatic checks pauses automatic plugin updates; manual checks, installation and updates remain available.')}</p>
        <div className='m-ck-title'>{t('Update channel')}</div>
        <Select value={settings?.channelMode || 'follow'} disabled={busy || !settings}
          onValueChange={value => void save({ channelMode: value as PluginSettings['channelMode'] })}>
          <SelectTrigger className='w-full' aria-label={t('Update channel')}>
            <SelectValue>{modes.find(mode => mode.value === (settings?.channelMode || 'follow'))?.label}</SelectValue>
          </SelectTrigger>
          <SelectContent>{modes.map(mode => <SelectItem key={mode.value} value={mode.value}>{mode.label}</SelectItem>)}</SelectContent>
        </Select>
        {settings?.channelMode === 'custom' ? <>
          <div className='m-ck-toolbar'>
            <span className='m-ck-title m-ck-grow'>{t('Follow stable updates')}</span>
            <Switch checked={settings.stable} disabled={busy} aria-label={t('Follow stable updates')}
              onCheckedChange={value => void save({ stable: value === true })} />
          </div>
          <div className='m-ck-toolbar'>
            <span className='m-ck-title m-ck-grow'>{t('Follow beta updates')}</span>
            <Switch checked={settings.beta} disabled={busy} aria-label={t('Follow beta updates')}
              onCheckedChange={value => void save({ beta: value === true })} />
          </div>
        </> : <p className='m-ck-note'>{settings ? t('Following: {channels}', { channels: channelLabel }) : t('Loading')}</p>}
        <p className='m-ck-note'>{t('Follows the software’s stable and beta channel choices by default. Separate settings affect only plugins. Updates are selected by version and never downgrade.')}</p>
        <div>
          <div className='m-ck-toolbar'>
            <span className='m-ck-title m-ck-grow'>{t('Automatically update plugins')}</span>
            <Switch checked={settings?.autoUpdate === true} disabled={busy || !settings} aria-label={t('Automatically update plugins')}
              onCheckedChange={value => void save({ autoUpdate: value === true })} />
          </div>
          <p className='m-ck-note'>{t('Only automatically updates installed, enabled community plugins with verified trusted signatures. New plugins require manual installation and are not automatically reinstalled after removal. Unsigned plugins require manual updates.')}</p>
        </div>
        {error ? <p className='m-ck-error'>{error}</p> : null}
        <GithubAccessSettingsPanel />
      </SheetBody>
    </SheetContent>
  </Sheet>
}
