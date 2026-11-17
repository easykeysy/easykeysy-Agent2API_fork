import * as React from 'react'
import { Alert, Button, Dialog, DialogBody, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, Field, FieldDescription, FieldLabel, Input, Spinner, Switch } from '@ui'
import { t } from '../ui-kit/src/i18n'
import { maintenanceLabel } from './maintenance-format'
import type { MaintenanceApi, MaintenanceTask, MaintenanceTaskId } from './maintenance-types'

type Draft = { enabled: boolean; interval: string }

/**
 * 维护设置。
 *
 * 两个任务收在一张分组表里（行之间只用发丝线），而不是两个带描边的小方块：
 * 开关与说明同一行、间隔输入贴在下面，一屏看完两项设置；底栏只有「取消 / 保存设置」。
 * 校验与保存流程与原实现完全一致：先整体校验间隔，再逐项提交，中途失败要提示「部分可能已保存」。
 */
export function MaintenanceSettings({ tasks, api, onClose, onSaved }: {
  tasks: MaintenanceTask[]
  api: MaintenanceApi
  onClose(): void
  onSaved(): void
}) {
  const [drafts, setDrafts] = React.useState<Record<string, Draft>>(() => Object.fromEntries(tasks.map(task => [task.id, { enabled: task.enabled, interval: String(task.interval) }])))
  const [busy, setBusy] = React.useState(false)
  const [error, setError] = React.useState('')
  const saving = React.useRef(false)
  const unsaved = React.useRef(new Set<MaintenanceTaskId>())
  const patch = (id: MaintenanceTaskId, next: Partial<Draft>) => setDrafts(previous => ({ ...previous, [id]: { ...previous[id], ...next } }))
  const save = async (event: React.FormEvent) => {
    event.preventDefault()
    if (saving.current) return
    for (const task of tasks) {
      const value = drafts[task.id]?.interval.trim() ?? ''
      if (!/^\d+$/.test(value) || Number(value) < task.min || Number(value) > task.max) {
        setError(t('The {name} interval must be a whole number from {min} to {max} minutes', { name: maintenanceLabel(task.id), min: task.min, max: task.max }))
        return
      }
    }
    saving.current = true
    setBusy(true)
    setError('')
    try {
      for (const task of tasks) {
        const draft = drafts[task.id]
        if (!unsaved.current.has(task.id) && task.enabled === draft.enabled && task.interval === Number(draft.interval)) continue
        unsaved.current.add(task.id)
        const result = await api.save(task.id, { enabled: draft.enabled, interval: Number(draft.interval) })
        if (result.saved === false) throw new Error(t('Maintenance settings could not be saved to the database. Check disk space and permissions.'))
        unsaved.current.delete(task.id)
      }
      onSaved()
      onClose()
    } catch (failure) {
      setError(t('Some settings may have been saved. Review them before retrying: {reason}', { reason: failure instanceof Error ? failure.message : String(failure) }))
      onSaved()
    } finally {
      saving.current = false
      setBusy(false)
    }
  }
  return <Dialog open onOpenChange={(open, event) => { if (!open) { if (busy) event.cancel(); else onClose() } }}>
    <DialogContent className='w-[min(600px,calc(100vw-32px))]'>
      <form className='flex min-h-0 flex-col' onSubmit={event => void save(event)}>
        <DialogHeader className='shrink-0 flex-col items-start gap-1'>
          <DialogTitle>{t('Maintenance settings')}</DialogTitle>
          <DialogDescription className='text-xs leading-relaxed text-muted-foreground'>{t('Automatic and manual runs share the same result history')}</DialogDescription>
        </DialogHeader>
        <DialogBody className='flex min-h-0 flex-col gap-3'>
          {error ? <Alert variant='destructive'>{error}</Alert> : null}
          <div className='maintenance-setting-group'>
            {tasks.map((task, index) => <div key={task.id} className={index ? 'maintenance-setting maintenance-divided' : 'maintenance-setting'}>
              <div className='maintenance-setting-heading'>
                <div>
                  <h3>{maintenanceLabel(task.id)}</h3>
                  <p>{task.id === 'credentialMaintenance' ? t('Renew only expired or soon-to-expire account credentials') : t('Periodically fetch each provider\'s latest model list')}</p>
                </div>
                <Switch checked={drafts[task.id].enabled} disabled={busy} aria-label={maintenanceLabel(task.id)} onCheckedChange={enabled => patch(task.id, { enabled })} />
              </div>
              <Field className='w-[220px]'>
                <FieldLabel>{t('Interval (minutes)')}</FieldLabel>
                <Input type='number' inputMode='numeric' min={task.min} max={task.max} step={1} disabled={busy || !drafts[task.id].enabled}
                  value={drafts[task.id].interval} onChange={event => patch(task.id, { interval: event.target.value })} />
                <FieldDescription>{t('Allowed: {min}–{max}', { min: task.min, max: task.max })}</FieldDescription>
              </Field>
            </div>)}
          </div>
          <p className='maintenance-muted'>{t('Turning off automatic maintenance does not affect manual runs or on-demand credential renewal for chat requests.')}</p>
        </DialogBody>
        <DialogFooter className='shrink-0 flex-wrap'>
          <Button type='button' variant='ghost' disabled={busy} onClick={onClose}>{t('Cancel')}</Button>
          <Button type='submit' disabled={busy}>{busy ? <Spinner /> : null}{busy ? t('Saving…') : t('Save settings')}</Button>
        </DialogFooter>
      </form>
    </DialogContent>
  </Dialog>
}
