import * as React from 'react'
import { Popover as PopoverPrimitive } from '@base-ui/react/popover'
import { cn } from './lib/cn'

/**
 * （， and shadcn/ui Popover 1 ）。
 *
 * ：Popover / PopoverTrigger / PopoverContent /
 * PopoverHeader / PopoverTitle / PopoverDescription / PopoverClose。
 * PopoverContent Portal + Positioner + Popup ， use 1 
 * `<PopoverContent>` ， not Base UI 。
 *
 * no has PopoverAnchor：Base UI Popover 「」 to Trigger 
 * （ is ）， no has Anchor —— that is Radix 。
 * need need 「 in up 」 when use PopoverContent `anchor` 
 * （ / ref / return ）， not Trigger 。
 *
 * for ui/css/components.css 1 （ and SelectContent 1 ）：
 * （--ui-raised）+ + --shadow-3、--r-lg 、z-35。
 * z 35 not is 30： need in up （ in need can ），
 * （40） in up 。
 *
 * and Select / Tooltip ：
 * · need need 「 from 1 1 」→ use Select（ in and typeahead）；
 * · need need 「 1 any」→ use Popover（）；
 * · need need 「 when 1 down 」→ use Tooltip。
 *
 * to Base UI Popover：、Esc 、、
 * by to and （floating-ui） —— this is 。
 *
 * use ：
 *   <Popover>
 * <PopoverTrigger render={<Button variant='outline' />}></PopoverTrigger>
 *     <PopoverContent align='end'>…</PopoverContent>
 *   </Popover>
 */

function Popover(props: PopoverPrimitive.Root.Props) {
  return <PopoverPrimitive.Root data-slot='popover' {...props} />
}

function PopoverTrigger(props: PopoverPrimitive.Trigger.Props) {
  return <PopoverPrimitive.Trigger data-slot='popover-trigger' {...props} />
}

function PopoverClose(props: PopoverPrimitive.Close.Props) {
  return <PopoverPrimitive.Close data-slot='popover-close' {...props} />
}

type PopoverContentProps = Omit<PopoverPrimitive.Popup.Props, 'className'> & {
  className?: string
  /** and ，default 6（ and SelectContent 4 has ： has ， will ） */
  sideOffset?: number
  align?: PopoverPrimitive.Positioner.Props['align']
  side?: PopoverPrimitive.Positioner.Props['side']
  alignOffset?: number
  /** when ，default 8（ to use ） */
  collisionPadding?: PopoverPrimitive.Positioner.Props['collisionPadding']
  /**
   * in 1 **** up ， not is PopoverTrigger。
   *
   * in by ： use by has when not React （ by ，
   * in legacy ）。 no has this when can 「 1 Trigger 」
   * —— that will by hover / title， not 。
   * 、ref or return （Base UI VirtualElement ）。
   */
  anchor?: PopoverPrimitive.Positioner.Props['anchor']
}

function PopoverContent({
  className,
  sideOffset = 6,
  align = 'start',
  side,
  alignOffset,
  collisionPadding = 8,
  anchor,
  children,
  ...props
}: PopoverContentProps) {
  return (
    <PopoverPrimitive.Portal>
      <PopoverPrimitive.Positioner
        className='z-[35] outline-none'
        sideOffset={sideOffset}
        align={align}
        side={side}
        alignOffset={alignOffset}
        // need can to ：max-w use --available-width not will 
        collisionPadding={collisionPadding}
        anchor={anchor}
      >
        <PopoverPrimitive.Popup
          data-slot='popover-content'
          className={cn(
            'flex max-h-[min(70vh,560px)] flex-col overflow-hidden rounded-lg border border-border-strong bg-raised text-foreground shadow-3',
            'outline-none',
            // ： + up 。 not —— need 「 after 」，
            // transform will （ and SelectContent 1 ）
            'data-open:animate-in data-open:fade-in-0 data-open:slide-in-from-top-1',
            'data-closed:animate-out data-closed:fade-out-0',
            className
          )}
          {...props}
        >
          {children}
        </PopoverPrimitive.Popup>
      </PopoverPrimitive.Positioner>
    </PopoverPrimitive.Portal>
  )
}

/** ： 1 ， 1 hairline。 need by 。 */
function PopoverHeader({ className, ...props }: React.ComponentProps<'div'>) {
  return (
    <div
      data-slot='popover-header'
      className={cn('flex items-center gap-2 border-b border-hairline px-3 py-2', className)}
      {...props}
    />
  )
}

function PopoverTitle({ className, ...props }: PopoverPrimitive.Title.Props) {
  return (
    <PopoverPrimitive.Title
      data-slot='popover-title'
      className={cn('text-[12.5px] font-semibold text-foreground', className)}
      {...props}
    />
  )
}

function PopoverDescription({ className, ...props }: PopoverPrimitive.Description.Props) {
  return (
    <PopoverPrimitive.Description
      data-slot='popover-description'
      className={cn('text-[11.5px] leading-[1.55] text-muted-foreground', className)}
      {...props}
    />
  )
}

export {
  Popover,
  PopoverTrigger,
  PopoverClose,
  PopoverContent,
  PopoverHeader,
  PopoverTitle,
  PopoverDescription,
}
