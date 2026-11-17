import * as React from 'react'
import {
  Button, Dialog, DialogBody, DialogContent, DialogFooter, DialogHeader, DialogTitle,
  Input, NavItem, NavList, Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
  Sidebar, Switch,
} from '@ui'
import { GithubUpdateSettings } from './update-github-settings'
import { api, toast } from './plugins-shared'
import type { PluginSettings } from './plugins-types'
import { t } from '../i18n'

type PaneId = 'check' | 'channel' | 'automatic' | 'network' | 'token'

export function PluginSettingsDialog({ open, onClose, onChanged }: {
  open: boolean
  onClose: () => void
  onChanged: () => Promise<void>
}) {
  const [settings, setSettings] = React.useState<PluginSettings | null>(null)
  const [draft, setDraft] = React.useState('')
  const [busy, setBusy] = React.useState(false)
  const [pane, setPane] = React.useState<PaneId>('check')
  const contentRef = React.useRef<HTMLDivElement>(null)
  const panes: Array<[PaneId, string]> = [
    ['check', t('Check schedule')], ['channel', t('Update channel')], ['automatic', t('Automatically update plugins')],
    ['network', t('Network')], ['token', t('GitHub Token')],
  ]
  const modes = [
    { value: 'follow', label: t('Follow software settings') },
    { value: 'custom', label: t('Configure separately') },
  ] as const

  React.useEffect(() => {
    if (!open) return
    let alive = true
    setPane('check')
    setSettings(null)
    void api<PluginSettings>('GET', '/api/plugins/settings').then(value => {
      if (alive) { setSettings(value); setDraft(String(value.interval)) }
    }).catch(error => toast(t('Could not load settings: {error}', { error: String(error) }), 'err'))
    return () => { alive = false }
  }, [open])

  async function save(patch: Partial<PluginSettings>): Promise<void> {
    if (busy || !settings) return
    const next = { ...settings, ...patch }
    if (patch.channelMode !== 'follow' && !next.stable && !next.beta) {
      toast(t('Follow at least one channel: stable or beta.'), 'err')
      return
    }
    setBusy(true)
    try {
      const result = await api<PluginSettings>('PATCH', '/api/plugins/settings', patch)
      setSettings(result)
      setDraft(String(result.interval))
      toast(result.saved === false
        ? t('Settings applied, but could not be saved to disk (will revert after restart)') : t('Plugin settings saved'),
      result.saved === false ? 'err' : 'ok')
      if ('beta' in patch || 'stable' in patch || 'channelMode' in patch) await onChanged()
    } catch (error) {
      toast(t('Save failed: {error}', { error: error instanceof Error ? error.message : String(error) }), 'err')
    } finally { setBusy(false) }
  }

  async function saveInterval(): Promise<void> {
    const interval = Number(draft)
    if (!Number.isInteger(interval) || interval < 1 || interval > 1440) {
      toast(t('The check interval must be {min}–{max} minutes', { min: 1, max: 1440 }), 'err')
      return
    }
    await save({ interval })
  }

  const channelLabel = settings?.stable && settings.beta ? t('Stable and beta')
    : settings?.beta ? t('Beta only') : t('Stable only')
  return (
    <Dialog open={open} onOpenChange={next => { if (!next) onClose() }}>
      <DialogContent className='settings-dlg'>
        <DialogHeader><DialogTitle>{t('Plugin settings')}</DialogTitle></DialogHeader>
        <DialogBody className='settings-dlg-body'>
          <Sidebar className='prov-rail settings-dlg-rail' aria-label={t('Plugin settings')}>
            <NavList className='rail-scroll' render={<nav />}>
              <div className='rail-label'>{t('Plugin settings')}</div>
              {panes.map(([id, label]) => (
                <NavItem key={id} active={pane === id} title={label} onClick={() => {
                  setPane(id)
                  if (contentRef.current) contentRef.current.scrollTop = 0
                }}>{label}</NavItem>
              ))}
            </NavList>
          </Sidebar>
          <div className='settings-dlg-main' ref={contentRef}>
            {pane === 'check' ? <div>
              <div className='settings-dlg-pane-head'>
                <h3>{t('Check schedule')}</h3>
                <p>{t('Automatic checks and intervals follow the app by default. Turning off automatic checks pauses automatic plugin updates; manual checks, installation and updates remain available.')}</p>
              </div>
              <div className='settings-dlg-fields'>
                <Select value={settings?.checkMode || 'follow'} disabled={busy || !settings}
                  onValueChange={value => void save({ checkMode: value as PluginSettings['checkMode'] })}>
                  <SelectTrigger className='w-full' aria-label={t('Check schedule')}>
                    <SelectValue>{modes.find(mode => mode.value === (settings?.checkMode || 'follow'))?.label}</SelectValue>
                  </SelectTrigger>
                  <SelectContent>{modes.map(mode => <SelectItem key={mode.value} value={mode.value}>{mode.label}</SelectItem>)}</SelectContent>
                </Select>
                {settings?.checkMode === 'custom' ? <>
                  <div className='flex items-center justify-between gap-2'>
                    <div className='text-[13px] text-foreground'>{t('Automatic update checks')}</div>
                    <Switch checked={settings.autoCheck} disabled={busy}
                      onCheckedChange={value => void save({ autoCheck: value === true })}
                      aria-label={t('Automatically check plugin updates')} />
                  </div>
                  <div className='flex flex-wrap items-center gap-2'>
                    <Input type='number' min={1} max={1440} step={1} className='w-[110px]'
                      disabled={busy || !settings.autoCheck} value={draft}
                      onChange={event => setDraft(event.currentTarget.value)}
                      onKeyDown={event => { if (event.key === 'Enter') { event.preventDefault(); void saveInterval() } }}
                      aria-label={t('Check interval (minutes)')} />
                    <span className='text-[12px] text-subtle'>{t('minutes ({min}–{max})', { min: 1, max: 1440 })}</span>
                    <Button variant='outline' size='sm' disabled={busy || !settings.autoCheck || Number(draft) === settings.interval}
                      onClick={() => void saveInterval()}>{t('Save')}</Button>
                  </div>
                </> : <p className='text-[12px] text-subtle'>
                  {settings ? settings.autoCheck ? t('Automatic checks are on, every {n} minutes', { n: settings.interval })
                    : t('Automatic checks are off') : t('Loading')}
                </p>}
              </div>
            </div> : null}
            {pane === 'channel' ? <div>
              <div className='settings-dlg-pane-head'>
                <h3>{t('Update channel')}</h3>
                <p>{t('Follows the software’s stable and beta channel choices by default. Separate settings affect only plugins. Updates are selected by version and never downgrade.')}</p>
              </div>
              <div className='settings-dlg-fields'>
                <Select value={settings?.channelMode || 'follow'} disabled={busy || !settings}
                  onValueChange={value => void save({ channelMode: value as PluginSettings['channelMode'] })}>
                  <SelectTrigger className='w-full' aria-label={t('Update channel')}>
                    <SelectValue>{modes.find(mode => mode.value === (settings?.channelMode || 'follow'))?.label}</SelectValue>
                  </SelectTrigger>
                  <SelectContent>{modes.map(mode => <SelectItem key={mode.value} value={mode.value}>{mode.label}</SelectItem>)}</SelectContent>
                </Select>
                {settings?.channelMode === 'custom' ? <>
                  <div className='flex items-center justify-between gap-2'>
                    <span className='text-[13px] text-foreground'>{t('Follow stable updates')}</span>
                    <Switch checked={settings.stable} disabled={busy} aria-label={t('Follow stable updates')}
                      onCheckedChange={value => void save({ stable: value === true })} />
                  </div>
                  <div className='flex items-center justify-between gap-2'>
                    <span className='text-[13px] text-foreground'>{t('Follow beta updates')}</span>
                    <Switch checked={settings.beta} disabled={busy} aria-label={t('Follow beta updates')}
                      onCheckedChange={value => void save({ beta: value === true })} />
                  </div>
                </> : <p className='text-[12px] text-subtle'>{settings ? t('Following: {channels}', { channels: channelLabel }) : t('Loading')}</p>}
              </div>
            </div> : null}
            {pane === 'automatic' ? <div>
              <div className='settings-dlg-pane-head'><h3>{t('Automatically update plugins')}</h3></div>
              <div className='settings-dlg-fields'>
                <div>
                  <div className='flex items-center justify-between gap-2'>
                    <div className='text-[13px] font-semibold text-foreground'>{t('Automatically update plugins')}</div>
                    <Switch checked={settings?.autoUpdate === true} disabled={busy || !settings}
                      onCheckedChange={value => void save({ autoUpdate: value === true })} aria-label={t('Automatically update plugins')} />
                  </div>
                  <p className='mt-1.5 text-[12px] text-subtle'>
                    {t('Only automatically updates installed, enabled community plugins with verified trusted signatures. New plugins require manual installation and are not automatically reinstalled after removal. Unsigned plugins require manual updates.')}
                  </p>
                </div>
                <p className='text-[12px] text-subtle'>
                  {t('Automatic checks and intervals follow the app by default. Turning off automatic checks pauses automatic plugin updates; manual checks, installation and updates remain available.')}
                </p>
              </div>
            </div> : null}
            {open ? <GithubUpdateSettings plugin pane={pane === 'network' || pane === 'token' ? pane : null} /> : null}
          </div>
        </DialogBody>
        <DialogFooter className='settings-dlg-footer'>
          <p className='mr-auto text-[12px] text-subtle'>{t('Switches and options save immediately. For text and number fields, click the corresponding Save button.')}</p>
          <Button variant='outline' onClick={onClose}>{t('Close')}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
