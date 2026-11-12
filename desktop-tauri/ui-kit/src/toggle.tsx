import * as React from 'react'
import { Toggle as BaseToggle } from '@base-ui/react/toggle'
import { cva, type VariantProps } from 'class-variance-authority'
import { cn } from './lib/cn'

/**
 * by （， and shadcn/ui Toggle 1 ）。
 *
 * and Switch ：Switch is 「set is 」（ has 、 in ），
 * Toggle is 「 this 1 / this in 」（ by is need ，
 * by down is ）。 not ， not need 。
 *
 * and SegmentedControl ： that is 「 1 」， not can ；
 * Toggle has 、 and ** can by ** —— 「only in 」 this need is this 。
 *
 * and shadcn for ：variant default / outline，size default / sm / xs / lg。
 * 1 （--ui-primary-soft + --ui-primary-fg + --ui-primary-bd），
 * and 、 in is 1 。
 *
 * to Base UI Toggle：`<button aria-pressed>`， Space/Enter 。
 */

const toggleVariants = cva(
  [
    'inline-flex shrink-0 cursor-pointer items-center justify-center gap-1.5 whitespace-nowrap border',
    'font-medium transition-[background-color,border-color,color] duration-150 ease-out',
    'outline-none select-none disabled:pointer-events-none disabled:opacity-45',
    '[&_svg]:pointer-events-none [&_svg]:shrink-0',
    // ：。 1 to ， when by down not 
    'data-pressed:border-primary-bd data-pressed:bg-primary-soft data-pressed:font-semibold data-pressed:text-primary-fg',
    'data-pressed:hover:bg-primary-tint',
  ],
  {
    variants: {
      variant: {
        /** ： when is 1 （ ） */
        default: 'border-transparent bg-transparent text-subtle hover:bg-nav-hover hover:text-foreground',
        /** ： when and by */
        outline:
          'border-control-border bg-control text-foreground shadow-1 hover:border-control-border-hover hover:bg-control-hover',
      },
      size: {
        default: 'h-[30px] rounded-md px-3 text-[12.5px]',
        xs: 'h-6 rounded-sm px-2 text-[11.5px]',
        sm: 'h-[26px] rounded-sm px-2.5 text-[12px]',
        lg: 'h-9 rounded-md px-4 text-[13px]',
      },
    },
    defaultVariants: { variant: 'default', size: 'default' },
  }
)

type ToggleProps = Omit<React.ComponentProps<typeof BaseToggle>, 'className'> &
  VariantProps<typeof toggleVariants> & {
    className?: string
  }

function Toggle({ className, variant, size, ...props }: ToggleProps) {
  return (
    <BaseToggle
      data-slot='toggle'
      className={cn(toggleVariants({ variant, size }), className)}
      {...props}
    />
  )
}

export { Toggle, toggleVariants, type ToggleProps }
