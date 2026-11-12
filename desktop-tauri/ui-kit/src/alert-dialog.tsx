import * as React from 'react'
import { AlertDialog as AlertDialogPrimitive } from '@base-ui/react/alert-dialog'
import { cn } from './lib/cn'
import { Button } from './button'
import { t } from './i18n'

/**
 * for （， and shadcn/ui AlertDialog 1 ）。
 *
 * ：AlertDialog / AlertDialogTrigger / AlertDialogPortal / AlertDialogClose /
 * AlertDialogOverlay / AlertDialogContent / AlertDialogHeader / AlertDialogTitle /
 * AlertDialogDescription / AlertDialogFooter / AlertDialogAction /
 * AlertDialogCancel。
 *
 * ── and Dialog （， use ）──────────────────────
 * AlertDialog is ****：、 by Esc not ，must its in 1 。
 * Dialog is ： / Esc 。
 * 「 not can 」 use AlertDialog —— use not can empty
 * ，must。
 *
 * has wbConfirm.ask / Esc （ has 13 use 
 * 「return false」 this 1 as ）， ui/islands is 
 * AlertDialog + （ wbConfirm ）。
 * new when use 、 not that ， to is as 。
 *
 * ：z-38（ z-30 、 toast z-40 ）。 from 
 * （「request」「delete」…）， when will after ，must ——
 * this and ui/css/components.css `#confirm-modal { z-index: 38 }` 1 。
 */

const AlertDialog = AlertDialogPrimitive.Root
const AlertDialogTrigger = AlertDialogPrimitive.Trigger
const AlertDialogPortal = AlertDialogPrimitive.Portal

const AlertDialogTitle = (props: AlertDialogPrimitive.Title.Props) => (
  <AlertDialogPrimitive.Title data-slot='alert-dialog-title' {...props} />
)

const AlertDialogDescription = (props: AlertDialogPrimitive.Description.Props) => (
  <AlertDialogPrimitive.Description data-slot='alert-dialog-description' {...props} />
)

function AlertDialogOverlay({ className, ...props }: AlertDialogPrimitive.Backdrop.Props) {
  return (
    <AlertDialogPrimitive.Backdrop
      data-slot='alert-dialog-overlay'
      className={cn(
        'fixed inset-0 isolate z-[38] bg-mask backdrop-blur-[3px]',
        'transition-opacity duration-150',
        'data-open:animate-in data-open:fade-in-0',
        'data-closed:animate-out data-closed:fade-out-0 data-closed:fill-mode-forwards',
        className
      )}
      {...props}
    />
  )
}

type AlertDialogContentProps = AlertDialogPrimitive.Popup.Props & {
  /** is up by ，default false —— not 「✕」 this */
  showCloseButton?: boolean
}

function AlertDialogContent({
  className,
  children,
  showCloseButton = false,
  ...props
}: AlertDialogContentProps) {
  return (
    <AlertDialogPortal>
      <AlertDialogOverlay />
      <AlertDialogPrimitive.Popup
        data-slot='alert-dialog-content'
        className={cn(
          'fixed top-1/2 left-1/2 z-[38] flex max-h-[calc(100vh-48px)] w-[min(440px,calc(100vw-48px))] -translate-x-1/2 -translate-y-1/2 flex-col',
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
          <AlertDialogPrimitive.Close
            data-slot='alert-dialog-close'
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
          </AlertDialogPrimitive.Close>
        )}
      </AlertDialogPrimitive.Popup>
    </AlertDialogPortal>
  )
}

/** ： and DialogHeader （、、 to ） */
function AlertDialogHeader({ className, ...props }: React.ComponentProps<'div'>) {
  return (
    <div
      data-slot='alert-dialog-header'
      className={cn(
        'flex items-center gap-2.5 border-b border-hairline px-5 py-[15px]',
        '[&_h2]:flex-1 [&_h2]:text-[14.5px] [&_h2]:font-semibold',
        className
      )}
      {...props}
    />
  )
}

/** （， for `.modal-body`）： to 、 */
function AlertDialogBody({ className, ...props }: React.ComponentProps<'div'>) {
  return (
    <div
      data-slot='alert-dialog-body'
      className={cn('flex flex-col gap-4 overflow-y-auto px-5 py-[18px]', className)}
      {...props}
    />
  )
}

/** ： by ， use mr-auto */
function AlertDialogFooter({ className, ...props }: React.ComponentProps<'div'>) {
  return (
    <div
      data-slot='alert-dialog-footer'
      className={cn(
        'flex items-center justify-end gap-2 border-t border-hairline bg-surface-2 px-5 py-[13px]',
        className
      )}
      {...props}
    />
  )
}

/** （ for ui/css `.danger-zone`： + + ）
 * `tone` neutral when is （、）。 */
function AlertDialogBanner({
  className,
  tone = 'danger',
  ...props
}: React.ComponentProps<'div'> & { tone?: 'danger' | 'neutral' }) {
  return (
    <div
      data-slot='alert-dialog-banner'
      data-tone={tone}
      className={cn(
        'rounded-md border px-3.5 py-3 text-xs leading-[1.65] text-subtle',
        '[&_strong]:font-semibold',
        tone === 'danger'
          ? 'border-destructive-bd bg-destructive-soft [&_strong]:text-destructive'
          : 'border-border bg-surface-inset [&_strong]:text-foreground',
        className
      )}
      {...props}
    />
  )
}

export {
  AlertDialog,
  AlertDialogTrigger,
  AlertDialogPortal,
  AlertDialogOverlay,
  AlertDialogContent,
  AlertDialogHeader,
  AlertDialogBody,
  AlertDialogFooter,
  AlertDialogBanner,
  AlertDialogTitle,
  AlertDialogDescription,
}
