import * as React from 'react'
import { Dialog as DialogPrimitive } from '@base-ui/react/dialog'
import { cn } from './lib/cn'
import { Button } from './button'
import { t } from './i18n'

/**
 * （， and shadcn/ui Dialog 1 ）。
 *
 * ：Dialog / DialogTrigger / DialogPortal / DialogClose /
 * DialogOverlay / DialogContent / DialogHeader / DialogTitle / DialogDescription /
 * DialogFooter。DialogContent default and up by （showCloseButton），
 * not need need when false。
 *
 * is ， for ui/css/components.css ，
 * shadcn no has ：
 * · DialogBody ← `.modal-body`（ to 、 ）
 * · DialogSection ← `.modal-section`（ ， + ）
 *
 * for has `.modal-mask` / `.modal` 1 ： 46% + 3px ，
 * min(620px, 100%)、--r-lg 、--shadow-3 ； is 「 + up 10px
 * 1.5%」， to 。z-index 30（ and .modal-mask ， in 40 up ）。
 *
 * `fill-mode-forwards`：tw-animate default fill-mode:none， 1 
 * （ not ）， Base UI need etc that —— in this 
 * 1 is 「 1 down 」。forwards in （opacity 0） up ， etc 。
 *
 * to Base UI Dialog：、Esc 、、aria-modal and 
 * after 。
 */

function Dialog(props: DialogPrimitive.Root.Props) {
  return <DialogPrimitive.Root data-slot='dialog' {...props} />
}

function DialogTrigger(props: DialogPrimitive.Trigger.Props) {
  return <DialogPrimitive.Trigger data-slot='dialog-trigger' {...props} />
}

function DialogPortal(props: DialogPrimitive.Portal.Props) {
  return <DialogPrimitive.Portal data-slot='dialog-portal' {...props} />
}

function DialogClose(props: DialogPrimitive.Close.Props) {
  return <DialogPrimitive.Close data-slot='dialog-close' {...props} />
}

const DialogTitle = (props: DialogPrimitive.Title.Props) => (
  <DialogPrimitive.Title data-slot='dialog-title' {...props} />
)

const DialogDescription = (props: DialogPrimitive.Description.Props) => (
  <DialogPrimitive.Description data-slot='dialog-description' {...props} />
)

function DialogOverlay({ className, ...props }: DialogPrimitive.Backdrop.Props) {
  return (
    <DialogPrimitive.Backdrop
      data-slot='dialog-overlay'
      className={cn(
        'fixed inset-0 isolate z-30 bg-mask backdrop-blur-[3px]',
        'transition-opacity duration-150',
        'data-open:animate-in data-open:fade-in-0',
        'data-closed:animate-out data-closed:fade-out-0 data-closed:fill-mode-forwards',
        className
      )}
      {...props}
    />
  )
}

type DialogContentProps = DialogPrimitive.Popup.Props & {
  /** is up by ，default true */
  showCloseButton?: boolean
  /**
   * after is Portal in DOM （、 is ），default false（）。
   *
   * need need ： 1 ** has DOM**（e.g. ，
   * has has deviceId、）。 will this 1 ，
   * use 「 → → new → 」 will to 1 already deviceId。
   * this 1 ，、 in （Base UI will when as inert，
   * and not will ）。
   */
  keepMounted?: boolean
  /**
   * in 1 when that ，default false。
   *
   * Base UI is 「 not 」， in after （ DialogBackdrop
   * `enabled: forceRender || !nested`）。 need in up ：
   * · down （）—— no has has ， not ；
   * · empty this 1 （useDialogRoot outsidePress is ：
   * `backdrop === target`）， no has when empty not will 。
   * in Popup after ， and and 1 ， is 
   * 1 、 in up 。
   */
  overlayForceRender?: boolean
}

function DialogContent({
  className,
  children,
  showCloseButton = true,
  keepMounted = false,
  overlayForceRender = false,
  ...props
}: DialogContentProps) {
  return (
    <DialogPortal keepMounted={keepMounted}>
      <DialogOverlay forceRender={overlayForceRender} />
      <DialogPrimitive.Popup
        data-slot='dialog-content'
        className={cn(
          'fixed top-1/2 left-1/2 z-30 flex max-h-[calc(100vh-48px)] w-[min(620px,calc(100vw-48px))] -translate-x-1/2 -translate-y-1/2 flex-col',
          'overflow-hidden rounded-lg border border-border bg-surface shadow-3 outline-none',
          'transition-[opacity,transform] duration-200 [transition-timing-function:cubic-bezier(.2,.9,.3,1)]',
          'data-open:animate-in data-open:fade-in-0 data-open:zoom-in-[.985] data-open:slide-in-from-bottom-2.5',
          'data-closed:animate-out data-closed:fade-out-0 data-closed:fill-mode-forwards',
          className
        )}
        {...props}
      >
        {children}
        {showCloseButton && (
          <DialogPrimitive.Close
            data-slot='dialog-close'
            render={<Button variant='ghost' size='icon-sm' className='absolute top-3 right-3' />}
          >
            <svg viewBox='0 0 12 12' className='size-3' aria-hidden='true'>
              <path
                d='M2.5 2.5l7 7M9.5 2.5l-7 7'
                stroke='currentColor'
                strokeWidth='1.5'
                strokeLinecap='round'
                fill='none'
              />
            </svg>
            <span className='sr-only'>{t('关闭')}</span>
          </DialogPrimitive.Close>
        )}
      </DialogPrimitive.Popup>
    </DialogPortal>
  )
}

/** ：、， 1 hairline。
     40px to DialogContent by （26px by + 12px + 2px ），
     not will to down 。 */
function DialogHeader({ className, ...props }: React.ComponentProps<'div'>) {
  return (
    <div
      data-slot='dialog-header'
      className={cn(
        'flex items-center gap-2.5 border-b border-hairline px-5 py-[15px] pr-10',
        '[&_h2]:flex-1 [&_h2]:text-[14.5px] [&_h2]:font-semibold',
        className
      )}
      {...props}
    />
  )
}

/** （， for `.modal-body`）： to 、 */
function DialogBody({ className, ...props }: React.ComponentProps<'div'>) {
  return (
    <div
      data-slot='dialog-body'
      className={cn('flex flex-col gap-4 overflow-y-auto px-5 py-[18px]', className)}
      {...props}
    />
  )
}

/** ： by ， use mr-auto */
function DialogFooter({ className, ...props }: React.ComponentProps<'div'>) {
  return (
    <div
      data-slot='dialog-footer'
      className={cn(
        'flex items-center justify-end gap-2 border-t border-hairline bg-surface-2 px-5 py-[13px]',
        className
      )}
      {...props}
    />
  )
}

/** （， for `.modal-section`）： + */
function DialogSection({ className, ...props }: React.ComponentProps<'div'>) {
  return (
    <div
      data-slot='dialog-section'
      className={cn(
        'flex flex-col gap-2.5 rounded-md border border-border bg-surface-inset p-3.5',
        '[&_h3]:text-[12.5px] [&_h3]:font-semibold',
        '[&_p]:text-xs [&_p]:leading-[1.65] [&_p]:text-subtle',
        className
      )}
      {...props}
    />
  )
}

export {
  Dialog,
  DialogTrigger,
  DialogPortal,
  DialogClose,
  DialogOverlay,
  DialogContent,
  DialogTitle,
  DialogDescription,
  DialogHeader,
  DialogBody,
  DialogFooter,
  DialogSection,
}
