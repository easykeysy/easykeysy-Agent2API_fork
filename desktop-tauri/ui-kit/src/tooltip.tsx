import * as React from 'react'
import { Tooltip as BaseTooltip } from '@base-ui/react/tooltip'
import { cn } from './lib/cn'

/**
 * ： and ， and 。
 * use TooltipTrigger； anchor has 。
 * and down 、， not 。
 */

/** by 1 ， as up data-slot （ and ） */
function TooltipProvider(props: BaseTooltip.Provider.Props) {
  return <BaseTooltip.Provider data-slot='tooltip-provider' {...props} />
}

function Tooltip(props: BaseTooltip.Root.Props) {
  return <BaseTooltip.Root data-slot='tooltip' {...props} />
}

function TooltipTrigger(props: BaseTooltip.Trigger.Props) {
  return <BaseTooltip.Trigger data-slot='tooltip-trigger' {...props} />
}

type TooltipContentProps = Omit<React.ComponentProps<typeof BaseTooltip.Popup>, 'className'> & {
  className?: string
  /** and ，default 8px */
  sideOffset?: number
  /** to ，default top（ in up ） */
  side?: 'top' | 'right' | 'bottom' | 'left'
  anchor?: React.ComponentProps<typeof BaseTooltip.Positioner>['anchor']
  variant?: 'default' | 'plain'
}

function TooltipContent({ className, sideOffset = 8, side = 'top', anchor, variant = 'default', children, ...props }: TooltipContentProps) {
  return (
    <BaseTooltip.Portal>
      <BaseTooltip.Positioner anchor={anchor} positionMethod='fixed' collisionPadding={8} sideOffset={sideOffset} side={side} className='z-[90] data-anchor-hidden:hidden'>
        <BaseTooltip.Popup
          data-slot='tooltip-content'
          className={cn(
            'max-w-[min(320px,calc(100vw-16px))] rounded-md px-2.5 py-2 text-[11.5px] leading-[1.55] shadow-3',
            variant === 'plain' ? 'border border-border bg-raised text-foreground' : 'bg-primary text-primary-on',
            'break-words outline-none',
            'transition-[opacity,transform] duration-150 ease-out',
            'data-open:animate-in data-open:fade-in-0 data-open:zoom-in-95',
            'data-closed:animate-out data-closed:fade-out-0 data-closed:zoom-out-95',
            className
          )}
          {...props}
        >
          {children}
        </BaseTooltip.Popup>
      </BaseTooltip.Positioner>
    </BaseTooltip.Portal>
  )
}

/** ： and 45° */
function TooltipArrow({ className, ...props }: React.ComponentProps<typeof BaseTooltip.Arrow>) {
  return (
    <BaseTooltip.Arrow
      className={cn(
        'size-2 rotate-45 rounded-[1px] bg-primary',
        'data-[side=top]:-bottom-1 data-[side=bottom]:-top-1',
        className
      )}
      {...props}
    />
  )
}

export {
  Tooltip,
  TooltipProvider,
  TooltipTrigger,
  TooltipContent,
  TooltipArrow,
  type TooltipContentProps,
}
