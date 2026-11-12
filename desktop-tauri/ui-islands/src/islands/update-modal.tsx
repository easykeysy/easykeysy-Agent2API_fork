import * as React from 'react'
import { createRoot } from 'react-dom/client'
import {
  Alert,
  AlertDescription,
  AlertTitle,
  Button,
  Dialog,
  DialogBody,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@ui'
import {
  downloadAsset, handleExternalClick, isSelfUpdate, isManualUpdate, manualUpdateReason, markdownHtml, missingAssetReasonFor,
  openExternal, safeExternal, shared, serviceUpdateTask, type UpdateInfo,
} from './update-shared'
import { t } from '../i18n'
import { serviceTaskActive } from './update-systemd'
import { HtmlContent } from '@ui'
import { Empty } from '@ui'

/**
 * 「 to update」（ by need 、）—— from update-panel.tsx 。
 *
 * ── （ and port-panel 1 ）──────────────
 * old is index.html `#update-modal` 1 DOM + app.js ；
 * in need need when div body、createRoot， unmount + remove。
 * ** not `#update-modal*` id**， Dialog 1 （Esc / /
 * / ）。
 *
 * ── and update-panel ─────────────────────────────────
 * 「update」 need to set and down ， that is （openAndDownload）——
 * `showUpdateModal` 1 `onGoUpdate` ， by update-panel 
 * openAndDownload ； and not （ / will already / in set）
 * 「update」 localStorage file， use checkUpdate /
 * getUpdateStatus 。
 */

/**
 * 「update」 in localStorage （ = ）。
 *
 * and old app.js UPDATE_SKIP_KEY 1 ，： 1 use in 
 * not this as 1 。 by **** —— that not ， will update 
 * （「」 not ， will promptedUpdateVersion）。
 */
const UPDATE_SKIP_KEY = 'workbuddy-desktop-update-skip'

/** 「update」 （ etc not to no ） */
function readSkippedVersion(): string {
  try { return localStorage.getItem(UPDATE_SKIP_KEY) || '' } catch { return '' }
}

/** 「update」（ not down ， not will ） */
function writeSkippedVersion(version: string): void {
  try { localStorage.setItem(UPDATE_SKIP_KEY, version) } catch { /* ignore： down */ }
}

/** will already （「」 not localStorage， 1 ） */
let promptedUpdateVersion = ''

/**
 * and not （ from old app.js maybeShowUpdateModal ）：
 * · has has new 、 and new empty；
 * · 「update」 is ****： this not ， will update ；
 * · 「」 not ， will 1 not （promptedUpdateVersion）——
 * after when check to will ， no has this will 1 minutes 1 ；
 * · already in set when not —— update in before ， 1 。
 *
 * ── this 1 no has can down when ， by ──────────────────
 * `hasUpdate === true`： no has 「update」， to up 
 * will to 1 「 no has can down 」—— use to 1 。 in 
 * `downloadAsset`（ and 1 ）： no has not to 「update」， Reason
 * and to 「 to GitHub down 」。， is as 「 has new 」 this ——
 * in old up 。
 *
 * use and old 1 ：check / syncFromCache / openAndDownload when 1 （check that 
 * up 「 already in set」， is as and old path 1 1 for ）， as 
 * app.js when 。
 */
export function showUpdateModal(
  info: UpdateInfo | null | undefined,
  onGoUpdate: (info: UpdateInfo) => Promise<void>,
): void {
  if (info?.forceUpdate?.required) return
  if (info?.channelChanged) { unmountUpdateModal(); return }
  if (!info || info.hasUpdate !== true) return
  // update already use to 、（update）：「update」 is ——
  // up that by already 「」， in that 
  if (info.pendingRestart === true || serviceTaskActive(serviceUpdateTask(info))) return
  const latest = String(info.latestVersion || '').trim()
  if (!latest) return
  if (shared().wbApp?.currentPage === 'settings') return
  if (latest === readSkippedVersion() || latest === promptedUpdateVersion) return
  promptedUpdateVersion = latest

  unmountUpdateModal()
  modalHost = document.createElement('div')
  document.body.append(modalHost)
  modalRoot = createRoot(modalHost)
  modalRoot.render(<UpdateModal info={info} onGoUpdate={onGoUpdate} onClose={unmountUpdateModal} />)
}

type UpdateModalProps = {
  /** that 1 check（ and is ， not ） */
  info: UpdateInfo
  /** 「update」： openAndDownload（file ） */
  onGoUpdate: (info: UpdateInfo) => Promise<void>
  onClose: () => void
}

function UpdateModal({ info, onGoUpdate, onClose }: UpdateModalProps) {
  const version = String(info.latestVersion || '')
  const notes = String(info.notes || '').trim()
  const html = notes ? markdownHtml(notes) : ''
  // 「 has no has 」 and 1 （downloadAsset）： no has not to 「update」，
  // that by will to 1 up 
  const canDownload = !!downloadAsset(info)?.url
  const manual = isManualUpdate(info)
  const reason = manual ? manualUpdateReason(info) : canDownload ? '' : missingAssetReasonFor(info)
  const pageUrl = safeExternal(info.pageUrl)

  /** 「update」： by localStorage（ and old app.js 1 ） */
  function handleSkip(): void {
    writeSkippedVersion(version)
    onClose()
  }

  /** no has can down when that ： to 1 （ 1 to ） */
  function handleOpenRelease(): void {
    onClose()
    void openExternal(pageUrl)
  }

  /**
   * 「update」： → to set 「update」 → this down 。
   *
   * Why wbApp.showPage + wbSettingsPanel.showCategory： and old app.js 
   * `#update-modal-go` for （、 not —— this is ，
   * not this use default）。
   */
  function handleGo(): void {
    onClose()
    const app = shared()
    app.wbApp?.showPage?.('settings')
    app.wbSettingsPanel?.showCategory?.('about')
    void onGoUpdate(info)
  }

  return (
    // open（ as true）： 1 by onClose 。Esc / / up ✕ by 
    // Base UI Dialog （ old mask and that ✕ by ）
    <Dialog open onOpenChange={next => { if (!next) onClose() }}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t('检测到更新')}</DialogTitle>
        </DialogHeader>
        <DialogBody>
          <p className='text-[12.5px] leading-[1.6] text-subtle'>
            {t('新版本 ')}<strong className='text-foreground'>{version}</strong>
            {t(' 已发布（当前 {current}），更新日志如下：', { current: info.currentVersion || t('未知') })}
          </p>
          {/* no has can down ： in this Reason，update —— otherwise use 
              「update」none can 。 to that ： 
               not ，「 no has 」 etc no */}
          {canDownload && !manual ? null : (
            <Alert variant='warning'>
              <AlertTitle>
                {manual ? t('当前需要手动升级') : t('未找到适用的更新包')}
              </AlertTitle>
              <AlertDescription>
                {reason}
                {isSelfUpdate(info)
                  ? <div className='mt-1.5'>{t('在宿主机执行 docker compose pull && docker compose up -d 即可升级。')}</div>
                  : null}
              </AlertDescription>
            </Alert>
          )}
          {/* + （ old .update-modal-notes 46vh）， not will 1 ；
              overscroll-contain ， to not */}
          <div className='max-h-[46vh] overflow-y-auto overscroll-contain pr-1.5' onClick={handleExternalClick}>
            {html ? <HtmlContent className='md-body' html={html} /> : (
              <div className='md-body'>
                <Empty className='md-body-empty'>{t('这个版本没有填写发布说明。')}</Empty>
              </div>
            )}
          </div>
        </DialogBody>
        <DialogFooter>
          {/* old .modal-foot ： | spacer | + update。
               no has can down when 「update」 this not in （ handleGo ）：
              「 to GitHub down 」（pageUrl in ， after in no to 
              html_url when will 1 releases ）； not ，
               up that compose is ， this not to by */}
          <Button variant='outline' size='sm' onClick={handleSkip}>{t('跳过此次更新')}</Button>
          <div className='mr-auto' />
          <Button variant='outline' onClick={onClose}>{t('取消')}</Button>
          {canDownload
            ? <Button variant='default' onClick={handleGo}>{manual ? t('下载服务端包') : t('去更新')}</Button>
            : (!isSelfUpdate(info) && pageUrl)
              ? <Button variant='default' onClick={handleOpenRelease}>{t('到 GitHub 发布页下载')}</Button>
              : null}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

let modalRoot: ReturnType<typeof createRoot> | null = null
let modalHost: HTMLElement | null = null

function unmountUpdateModal(): void {
  if (modalRoot) {
    modalRoot.unmount()
    modalRoot = null
  }
  if (modalHost) {
    modalHost.remove()
    modalHost = null
  }
}
