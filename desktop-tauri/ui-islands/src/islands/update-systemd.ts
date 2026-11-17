import type { ServiceUpdateTask } from '../../../ui-shared/updates'
import { errorMessage, serviceUpdateTask, shared, type UpdateInfo } from './update-shared'
import { t } from '../i18n'

export type ServiceProgress = {
  info: UpdateInfo
  text: string
  badge: string
  active: boolean
  failed: boolean
  percent: number
}

type Listener = (progress: ServiceProgress) => void
let polling = false
let lastInfo: UpdateInfo | null = null
let loadedVersion = ''
let listener: Listener | null = null

export function serviceTaskActive(task?: ServiceUpdateTask | null): boolean {
  return !!task && ['downloading', 'verifying', 'installing', 'restarting'].includes(task.phase)
}

export function serviceProgress(info: UpdateInfo): ServiceProgress | null {
  const task = serviceUpdateTask(info)
  if (!task) return null
  const names = {
    downloading: t('Downloading'), verifying: t('Verifying'), installing: t('Installing'),
    restarting: t('Restarting'), completed: t('Update complete'), failed: t('Update failed'),
  }
  let text = ''
  switch (task.phase) {
    case 'downloading': text = t('Downloading the server update for {version}…', { version: task.version }); break
    case 'verifying': text = t('Independently verifying the official update. Existing files will not change until verification succeeds…'); break
    case 'installing': text = t('Backing up and replacing the server and panel files. Do not shut down the server…'); break
    case 'restarting': text = t('Restarting the service and verifying the new version. This page will refresh after recovery…'); break
    case 'completed': text = info.currentVersion === task.version
      ? t('Updated to {version}. The service is back online.', { version: task.version })
      : t('The update task ended, but the running version differs from the target. Check the service logs.'); break
    case 'failed': text = t('Update failed: {error}', { error: errorMessage(task.error || t('Please check the service logs')) }); break
  }
  return {
    info, text, badge: names[task.phase], active: serviceTaskActive(task),
    failed: task.phase === 'failed' || (task.phase === 'completed' && info.currentVersion !== task.version),
    percent: 0,
  }
}

export function watchServiceUpdate(info: UpdateInfo, onProgress: Listener): void {
  listener = onProgress
  if (!loadedVersion) loadedVersion = String(info.currentVersion || '')
  lastInfo = info
  const progress = serviceProgress(info)
  if (progress) listener(progress)
  if (!progress?.active || polling) return
  polling = true
  void poll().finally(() => { polling = false })
}

async function poll(): Promise<void> {
  let disconnectedAt = 0
  while (lastInfo && serviceTaskActive(serviceUpdateTask(lastInfo))) {
    await new Promise<void>(resolve => { window.setTimeout(resolve, 1000) })
    try {
      const info = await shared().workbuddyDesktop?.getUpdateStatus()
      const task = serviceUpdateTask(info)
      if (!info || !task) throw new Error('Update status unavailable')
      lastInfo = info
      const progress = serviceProgress(info)
      if (!progress) continue
      if (task.phase === 'downloading') {
        const download = await shared().workbuddyDesktop?.updateProgress()
        progress.percent = Number(download?.percent) || 0
      }
      listener?.(progress)
      disconnectedAt = 0
      if (!progress.active) {
        if (task.phase === 'completed' && info.currentVersion === task.version
          && loadedVersion !== info.currentVersion) window.location.reload()
        return
      }
    } catch {
      if (!disconnectedAt) disconnectedAt = Date.now()
      if (Date.now() - disconnectedAt > 180_000) {
        const info = lastInfo
        listener?.({ info, badge: t('Waiting for recovery'), text: t('Service recovery has not been confirmed. Check the server and refresh this page. Installation will not repeat automatically.'),
          active: false, failed: true, percent: 0 })
        return
      }
    }
  }
}

export async function startServiceUpdate(info: UpdateInfo, onProgress: Listener): Promise<void> {
  const version = info.latestVersion
  const api = shared().workbuddyDesktop
  const start = info.updateCapability?.mode === 'windows-service' ? api?.startWindowsServiceUpdate : api?.startSystemdUpdate
  if (!version || !start) throw new Error(t('One-click updates are unavailable. Refresh the page and try again.'))
  const confirmed = await shared().wbConfirm?.ask({
    title: t('Update the server now?'),
    text: t('Update to {version}? Once downloaded and verified, the service will briefly stop, the files will be replaced and the service will restart automatically. Accounts and settings are preserved. No further installation click is needed.', { version }),
    okText: t('Start update'), cancelText: t('Cancel'), bodyClass: '',
  })
  if (!confirmed) return
  const task = await start({ version })
  watchServiceUpdate({ ...info, serviceTask: task }, onProgress)
}
