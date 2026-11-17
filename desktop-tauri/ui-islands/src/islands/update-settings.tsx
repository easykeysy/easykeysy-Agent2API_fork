import * as React from 'react'
import {
  Button, Dialog, DialogBody, DialogContent, DialogFooter, DialogHeader, DialogTitle,
  Input, NavItem, NavList, Sidebar, Switch,
} from '@ui'
import { errorMessage, forwardUpdateResult, shared, toast, type UpdateChannels } from './update-shared'
import { GithubUpdateSettings } from './update-github-settings'
import { t } from '../i18n'

const TASK_ID = 'updateCheck'
const MIN_INTERVAL = 1
const MAX_INTERVAL = 1440

type PaneId = 'check' | 'channel' | 'network' | 'token'

export function UpdateSettingsDialog({ open, onClose }: { open: boolean; onClose: () => void }) {
  const [checkTask, setCheckTask] = React.useState<{ enabled: boolean; interval: number } | null>(null)
  const [intervalDraft, setIntervalDraft] = React.useState('')
  const [checkBusy, setCheckBusy] = React.useState(false)
  const [channels, setChannels] = React.useState<UpdateChannels | null>(null)
  const [channelBusy, setChannelBusy] = React.useState(false)
  const [pane, setPane] = React.useState<PaneId>('check')
  const contentRef = React.useRef<HTMLDivElement>(null)
  const panes: Array<[PaneId, string]> = [
    ['check', t('Check schedule')], ['channel', t('Update channel')],
    ['network', t('Network')], ['token', t('GitHub Token')],
  ]

  React.useEffect(() => {
    if (!open) return
    let alive = true
    setPane('check')
    setCheckTask(null)
    setChannels(null)
    void shared().workbuddyDesktop?.getUpdateChannels().then(result => {
      if (alive && result) setChannels(result)
    }).catch(error => toast(t('Could not load settings: {error}', { error: errorMessage(error) }), 'err'))
    void shared().workbuddyDesktop?.getScheduledTasks?.().then(result => {
      const task = result?.tasks?.find(task => task.id === TASK_ID)
      if (!alive || !task) return
      const interval = Math.max(MIN_INTERVAL, Math.round(Number(task.interval) || 0))
      setCheckTask({ enabled: task.enabled === true, interval })
      setIntervalDraft(String(interval))
    }).catch(error => toast(t('Could not load settings: {error}', { error: errorMessage(error) }), 'err'))
    return () => { alive = false }
  }, [open])

  async function saveCheck(patch: { enabled?: boolean; interval?: number }): Promise<void> {
    if (checkBusy) return
    setCheckBusy(true)
    try {
      const result = await shared().workbuddyDesktop?.saveScheduledTask?.(TASK_ID, patch)
      if (!result) throw new Error(t('Request failed'))
      const interval = Math.max(MIN_INTERVAL, Math.round(Number(result.interval) || 0))
      setCheckTask({ enabled: result.enabled === true, interval })
      setIntervalDraft(String(interval))
      toast(result.saved === false
        ? t('Settings applied, but could not be saved to disk (will revert after restart)') : t('✅ Automatic update checks saved'),
      result.saved === false ? 'err' : 'ok')
    } catch (error) {
      toast(t('Save failed: {error}', { error: errorMessage(error) }), 'err')
    } finally { setCheckBusy(false) }
  }

  async function saveInterval(): Promise<void> {
    const interval = Number(intervalDraft)
    if (!Number.isInteger(interval) || interval < MIN_INTERVAL || interval > MAX_INTERVAL) {
      toast(t('The check interval must be {min}–{max} minutes', { min: MIN_INTERVAL, max: MAX_INTERVAL }), 'err')
      return
    }
    if (interval !== checkTask?.interval) await saveCheck({ interval })
  }

  async function saveChannel(patch: Partial<Pick<UpdateChannels, 'stable' | 'beta'>>): Promise<void> {
    if (channelBusy || !channels) return
    const next = { ...channels, ...patch }
    if (!next.stable && !next.beta) {
      toast(t('Follow at least one channel: stable or beta.'), 'err')
      return
    }
    setChannelBusy(true)
    try {
      const result = await shared().workbuddyDesktop?.setUpdateChannels(patch)
      if (!result) throw new Error(t('Request failed'))
      setChannels(result)
      toast(result.saved === false
        ? t('Channel switched, but writing to disk failed (the old setting will be restored after a restart)') : t('Update channels saved'),
      result.saved === false ? 'err' : 'ok')
      await forwardUpdateResult()
      try { await shared().workbuddyDesktop?.checkUpdate() } catch { /* 检查仍遵守请求间隔和冷却 */ }
      await forwardUpdateResult()
    } catch (error) {
      toast(t('Save failed: {error}', { error: errorMessage(error) }), 'err')
    } finally { setChannelBusy(false) }
  }

  return (
    <Dialog open={open} onOpenChange={next => { if (!next) onClose() }}>
      <DialogContent className='settings-dlg'>
        <DialogHeader><DialogTitle>{t('Update settings')}</DialogTitle></DialogHeader>
        <DialogBody className='settings-dlg-body'>
          <Sidebar className='prov-rail settings-dlg-rail' aria-label={t('Update settings')}>
            <NavList className='rail-scroll' render={<nav />}>
              <div className='rail-label'>{t('Update settings')}</div>
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
                <p>{t('Plugins follow these update channels, automatic checks and interval by default. You can configure them separately in Plugin settings.')}</p>
              </div>
              <div className='settings-dlg-fields'>
                <div className='flex items-center justify-between gap-2'>
                  <div className='text-[13px] font-semibold text-foreground'>{t('Automatic update checks')}</div>
                  <Switch checked={checkTask?.enabled === true} disabled={checkBusy || !checkTask}
                    onCheckedChange={next => void saveCheck({ enabled: next === true })}
                    aria-label={t('Automatic update checks')} />
                </div>
                <div className='flex flex-wrap items-center gap-2'>
                  <Input type='number' min={MIN_INTERVAL} max={MAX_INTERVAL} step={1} className='w-[110px]'
                    disabled={checkBusy || !checkTask?.enabled} value={intervalDraft}
                    onChange={event => setIntervalDraft(event.currentTarget.value)}
                    onKeyDown={event => { if (event.key === 'Enter') { event.preventDefault(); void saveInterval() } }}
                    aria-label={t('Check interval (minutes)')} />
                  <span className='text-[12px] text-subtle'>{t('minutes ({min}–{max})', {
                    min: MIN_INTERVAL, max: MAX_INTERVAL,
                  })}</span>
                  <Button variant='outline' size='sm' disabled={checkBusy || !checkTask?.enabled
                    || Number(intervalDraft) === checkTask.interval} onClick={() => void saveInterval()}>{t('Save')}</Button>
                </div>
                <p className='text-[12px] text-subtle'>
                  {t('Checks GitHub for new versions on schedule and opens the "Update available" dialog when one is found; dismissing it still leaves manual checks available via the Check for updates button')}
                </p>
              </div>
            </div> : null}
            {pane === 'channel' ? <div>
              <div className='settings-dlg-pane-head'>
                <h3>{t('Update channel')}</h3>
                <p>{t('Plugins follow these update channels, automatic checks and interval by default. You can configure them separately in Plugin settings.')}</p>
              </div>
              <div className='settings-dlg-fields'>
                <div>
                  <div className='flex items-center justify-between gap-2'>
                    <span className='text-[13px] text-foreground'>{t('Follow stable updates')}</span>
                    <Switch checked={channels?.stable === true} disabled={channelBusy || !channels}
                      onCheckedChange={next => void saveChannel({ stable: next === true })} aria-label={t('Follow stable updates')} />
                  </div>
                  <p className='mt-1.5 text-[12px] text-subtle'>{t('Enabled by default. Even on a beta build, you can receive stable releases with a higher version.')}</p>
                </div>
                <div className='h-px bg-hairline' />
                <div>
                  <div className='flex items-center justify-between gap-2'>
                    <span className='text-[13px] text-foreground'>{t('Follow beta updates')}</span>
                    <Switch checked={channels?.beta === true} disabled={channelBusy || !channels}
                      onCheckedChange={next => void saveChannel({ beta: next === true })} aria-label={t('Follow beta updates')} />
                  </div>
                  <p className='mt-1.5 text-[12px] text-subtle'>{t('Beta builds may be unstable. With both channels enabled, the highest version is selected without downgrading. Your channel choices remain unchanged after installing either a stable or beta release.')}</p>
                </div>
              </div>
            </div> : null}
            {open ? <GithubUpdateSettings pane={pane === 'network' || pane === 'token' ? pane : null} /> : null}
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
