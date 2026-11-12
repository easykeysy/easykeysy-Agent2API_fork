import * as React from 'react'
import { Dialog as DialogPrimitive } from '@base-ui/react/dialog'
import { cn } from './lib/cn'
import { Button } from './button'
import { t } from './i18n'

/**
 * （Sheet）—— and shadcn/ui Sheet ，**** not is 。
 *
 * shadcn ：Sheet / SheetTrigger / SheetClose / SheetPortal /
 * SheetOverlay / SheetContent / SheetHeader / SheetTitle / SheetDescription /
 * SheetBody / SheetFooter。SheetContent default and up by 。
 *
 * ── Why need ────────────────────────────────────────────────
 * Dialog is in ， in up not to up 、 will up down 。
 * （、request、set） 1 this from ：
 * can and 、 up down 、「 can 」。
 *
 * ── and Dialog ──────────────────────────────────────
 * · ：`inset-x-0 bottom-0`， up （`rounded-t-xl`）；
 * · ：`max-h-[85svh]`，（SheetBody），；
 * · ： 1 38×4 ， is 「」「」 1 。
 * `showHandle={false}` can （ in 1 when use ）。
 *
 * to Base UI Dialog：、Esc、、 after 。
 * / `fill-mode-forwards` —— by and DialogContent （ that ）。
 *
 * `side` in has ：`side='center'` when by in ，
 * used for 1 in when （ not can ）。
 */

function Sheet(props: DialogPrimitive.Root.Props) {
  return <DialogPrimitive.Root data-slot='sheet' {...props} />
}

function SheetTrigger(props: DialogPrimitive.Trigger.Props) {
  return <DialogPrimitive.Trigger data-slot='sheet-trigger' {...props} />
}

function SheetClose(props: DialogPrimitive.Close.Props) {
  return <DialogPrimitive.Close data-slot='sheet-close' {...props} />
}

function SheetPortal(props: DialogPrimitive.Portal.Props) {
  return <DialogPrimitive.Portal data-slot='sheet-portal' {...props} />
}

const SheetTitle = (props: DialogPrimitive.Title.Props) => (
  <DialogPrimitive.Title data-slot='sheet-title' {...props} />
)

const SheetDescription = (props: DialogPrimitive.Description.Props) => (
  <DialogPrimitive.Description data-slot='sheet-description' {...props} />
)

function SheetOverlay({ className, ...props }: DialogPrimitive.Backdrop.Props) {
  return (
    <DialogPrimitive.Backdrop
      data-slot='sheet-overlay'
      className={cn(
        'fixed inset-0 isolate z-30 bg-mask backdrop-blur-[3px]',
        'transition-opacity duration-200',
        'data-open:animate-in data-open:fade-in-0',
        'data-closed:animate-out data-closed:fade-out-0 data-closed:fill-mode-forwards',
        className
      )}
      {...props}
    />
  )
}

type SheetContentProps = DialogPrimitive.Popup.Props & {
  /** is up by ，default true */
  showCloseButton?: boolean
  /** ，default true。， in 1 */
  showHandle?: boolean
  /**
   * ：`bottom` （default）；`center` in （ when use ）；
   * `right` （ this 「 1 」 ）。
   *
   * default `bottom` —— this in by is 。
   */
  side?: 'bottom' | 'center' | 'right' | 'left'
  /**
   * after is Portal in DOM （、 is ），default false。
   * and DialogContent keepMounted ： has DOM
   * （ 、） when need need 。
   */
  keepMounted?: boolean
}

function SheetContent({
  className,
  children,
  showCloseButton = true,
  showHandle = true,
  side = 'bottom',
  keepMounted = false,
  ...props
}: SheetContentProps) {
  const centered = side === 'center'
  const right = side === 'right'
  const left = side === 'left'
  return (
    <SheetPortal keepMounted={keepMounted}>
      <SheetOverlay />
      <DialogPrimitive.Popup
        data-slot='sheet-content'
        className={cn(
          'fixed z-30 flex flex-col overflow-hidden border border-border bg-surface shadow-3 outline-none',
          'transition-[opacity,transform] duration-200 [transition-timing-function:cubic-bezier(.32,.9,.3,1)]',
          centered
            ? [
                'top-1/2 left-1/2 max-h-[calc(100vh-48px)] w-[min(560px,calc(100vw-48px))]',
                '-translate-x-1/2 -translate-y-1/2 rounded-lg',
                'data-open:animate-in data-open:fade-in-0 data-open:zoom-in-[.985]',
                'data-closed:animate-out data-closed:fade-out-0 data-closed:fill-mode-forwards',
              ]
            : right
              ? [
                  // 86vw， and up down 。
                  'top-0 right-0 bottom-0 h-full w-[min(340px,86vw)] rounded-l-xl',
                  'data-open:animate-in data-open:slide-in-from-right data-open:fade-in-0',
                  'data-closed:animate-out data-closed:slide-out-to-right data-closed:fade-out-0 data-closed:fill-mode-forwards',
                ]
              : left
                ? [
                    'top-0 left-0 bottom-0 h-full w-[min(340px,86vw)] rounded-r-xl',
                    'data-open:animate-in data-open:slide-in-from-left data-open:fade-in-0',
                    'data-closed:animate-out data-closed:slide-out-to-left data-closed:fade-out-0 data-closed:fill-mode-forwards',
                  ]
                : [
                  'inset-x-0 bottom-0 max-h-[85svh] rounded-t-xl',
                  'data-open:animate-in data-open:slide-in-from-bottom data-open:fade-in-0',
                  'data-closed:animate-out data-closed:slide-out-to-bottom data-closed:fade-out-0 data-closed:fill-mode-forwards',
                ],
          className
        )}
        {...props}
      >
        {showHandle && !centered && !right && !left && (
          <div
            aria-hidden='true'
            className='mx-auto mt-2 mb-1 h-1 w-9.5 shrink-0 rounded-pill bg-border-strong'
          />
        )}
        {children}
        {showCloseButton && (
          <DialogPrimitive.Close
            data-slot='sheet-close'
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
    </SheetPortal>
  )
}

/** ：、， 1 hairline。 40px to by */
function SheetHeader({ className, ...props }: React.ComponentProps<'div'>) {
  return (
    <div
      data-slot='sheet-header'
      className={cn(
        'flex shrink-0 items-center gap-2.5 border-b border-hairline px-5 py-3 pr-10',
        '[&_h2]:flex-1 [&_h2]:text-[15px] [&_h2]:font-semibold',
        className
      )}
      {...props}
    />
  )
}

/** ： 1 can ， when 、 */
function SheetBody({ className, ...props }: React.ComponentProps<'div'>) {
  return (
    <div
      data-slot='sheet-body'
      className={cn('flex min-h-0 flex-1 flex-col gap-3.5 overflow-y-auto px-5 py-4', className)}
      {...props}
    />
  )
}

/** ： by ； use 「 by 」， use flex-col */
function SheetFooter({ className, ...props }: React.ComponentProps<'div'>) {
  return (
    <div
      data-slot='sheet-footer'
      className={cn(
        'flex shrink-0 items-center gap-2 border-t border-hairline bg-surface-2 px-5 py-3 pb-[max(12px,env(safe-area-inset-bottom))]',
        className
      )}
      {...props}
    />
  )
}

export {
  Sheet,
  SheetTrigger,
  SheetClose,
  SheetPortal,
  SheetOverlay,
  SheetContent,
  SheetTitle,
  SheetDescription,
  SheetHeader,
  SheetBody,
  SheetFooter,
}
