import { useRef } from 'react'
import {
  Button, Dialog, DialogBody, DialogContent, DialogDescription, DialogFooter,
  DialogHeader, DialogTitle,
} from '@ui'
import type { PluginTranslate } from './plugins-store'

/** 安装前的兼容性须知：从「安装脚本插件」弹窗里叠起来的一层。
 *
 * 用普通 Dialog 而不是 AlertDialog：这一层只是把「不保证 100% 可用」讲清楚，
 * 不是危险操作的强制表态（信任勾选已经在上层做过了），所以照其他弹窗的规矩来 ——
 * 点空白处、按 Esc、右上角 ✕ 都能退出。
 *
 * overlayForceRender：Base UI 的嵌套约定是「子层不画遮罩」，只画的话下层（含上层
 * 弹窗）不会被压暗、也没有高斯模糊，点空白处更不会关闭（点空白处关闭认的就是遮罩
 * 本身）。所以这一层必须显式要回自己的遮罩，颜色与模糊与普通弹窗同一套。
 */
export function ScriptPluginInstallNotice({ open, source, version, disabled, busy, error, onCancel, onConfirm, t }: {
  open: boolean
  source: string
  version: string
  disabled: boolean
  busy: boolean
  error?: string
  onCancel: () => void
  onConfirm: () => void
  t: PluginTranslate
}) {
  const cancelRef = useRef<HTMLButtonElement>(null)
  return <Dialog open={open} onOpenChange={(next, details) => {
    // 正在安装时不许靠点空白处 / Esc 撤走，避免用户以为没装上。
    if (!next) { if (busy) details.cancel(); else onCancel() }
  }}>
    <DialogContent overlayForceRender className='w-[min(480px,calc(100vw-48px))] text-foreground' initialFocus={cancelRef}>
      <DialogHeader><DialogTitle>{t('Magpie plugin installation notice')}</DialogTitle></DialogHeader>
      <DialogBody className='gap-3 text-xs leading-relaxed text-subtle'>
        <div className='rounded-md border border-hairline bg-surface-inset px-3 py-2'>
          <p className='m-0 break-all font-medium text-foreground'>{source}</p>
          {version ? <p className='m-0 mt-1 break-all text-muted-foreground'>{version}</p> : null}
        </div>
        <DialogDescription className='m-0 text-sm font-medium leading-relaxed text-foreground'>
          {t('Official and community Magpie plugins are integrated with Agent2API for compatibility only; 100% availability is not guaranteed.')}
        </DialogDescription>
        <p className='m-0'>{t('Login, model calls, model lists, balance queries and check-ins may become unavailable depending on how the plugin is implemented or on upstream changes.')}</p>
        <p className='m-0'>{t('Check-ins and the side tasks that come with them are allowed by default; actual execution follows the switches and schedules in the check-in center.')}</p>
        <p className='m-0 text-muted-foreground'>{t('Side tasks of some plugins may consume quota, send messages, or create and dismiss group chats.')}</p>
        {error ? <p role='alert' className='m-0 break-words text-destructive'>{error}</p> : null}
      </DialogBody>
      <DialogFooter>
        <Button ref={cancelRef} variant='outline' disabled={busy} onClick={onCancel}>{t('Cancel')}</Button>
        <Button disabled={disabled} onClick={onConfirm}>{busy ? t('Installing…') : t('I understand, install')}</Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
}
