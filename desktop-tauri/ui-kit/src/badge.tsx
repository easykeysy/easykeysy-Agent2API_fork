import * as React from 'react'
import { useRender } from '@base-ui/react/use-render'
import { mergeProps } from '@base-ui/react/merge-props'
import { cva, type VariantProps } from 'class-variance-authority'
import { cn } from './lib/cn'

/**
 * / 。
 *
 * and shadcn/ui Badge for ：variant default / secondary / destructive /
 * outline / ghost / link， and 1 ； after is 
 * （success / warning / info / brand / cn / intl / sensitive）—— 、
 * 、 in need need this ， is variant ， not 1 prop。
 *
 * and has for ：ui/css `.badge`（ + ）＝ `outline`，
 * `.badge.ok` ＝ `success`，`.badge.warn` ＝ `warning`，`.badge.bad` ＝ `destructive`，
 * `.badge.brand` ＝ `brand`，`.badge.tag` ＝ `shape="tag"`。
 *
 * shape is （ and Button size ）：pill is default ，
 * tag is （， --r-xs）。
 */

const badgeVariants = cva(
  'inline-flex w-fit shrink-0 items-center justify-center gap-[5px] overflow-hidden whitespace-nowrap border font-semibold',
  {
    variants: {
      variant: {
        default: 'border-transparent bg-primary text-primary-foreground',
        secondary: 'border-transparent bg-secondary text-secondary-foreground',
        destructive: 'border-destructive-bd bg-destructive-soft text-destructive',
        outline: 'border-control-border bg-control text-subtle',
        ghost: 'border-transparent bg-transparent text-muted-foreground',
        link: 'border-transparent bg-transparent text-primary underline-offset-4 hover:underline',
        /* down as ： and */
        success: 'border-success-bd bg-success-soft text-success',
        warning: 'border-warning-bd bg-warning-soft text-warning',
        info: 'border-info-bd bg-info-soft text-info',
        brand: 'border-primary-bd bg-primary-soft text-primary-fg',
        cn: 'border-cn-bd bg-cn-soft text-cn',
        intl: 'border-intl-bd bg-intl-soft text-intl',
        /* in ： not is 「failure」（） not is 「」（），
            is ， 1 （ theme.css --ui-sensitive*） */
        sensitive: 'border-sensitive-bd bg-sensitive-soft text-sensitive',
      },
      shape: {
        pill: 'min-h-[21px] rounded-pill px-[9px] py-px text-[11.5px]',
        tag: 'min-h-[19px] rounded-xs px-2 py-0 text-[11px]',
      },
    },
    defaultVariants: { variant: 'default', shape: 'pill' },
  }
)

type BadgeProps = useRender.ComponentProps<'span'> & VariantProps<typeof badgeVariants>

function Badge({ className, variant, shape, render, ...props }: BadgeProps) {
  /**
   * Base UI useRender， is supports `render` —— 1 （
   * `render={<button type='button' />}`） can ，
   * by Base UI and （ not is ）。 this is shadcn `asChild` in Base UI this 1 
   * for 。
   *
   * Why need need ：request 「」「」 is ** **，
   * must is can （ need can Tab to 、 need can to ），
   * use span not to 。 before can 、 1 button。
   *
   * className and to useRender： （ to + use to ）
   * in this already 1 ， not mergeProps for className and then 。
   */
  return useRender({
    render,
    defaultTagName: 'span',
    // use mergeProps not is useRender props ： after in and ，
    // for ， need ，check。
    props: mergeProps({ 'data-slot': 'badge' }, props, {
      className: cn(badgeVariants({ variant, shape }), className),
    }),
  })
}

/** ： */
function BadgeDot({ className, ...props }: React.ComponentProps<'span'>) {
  return <span className={cn('size-1.5 flex-none rounded-full bg-current', className)} {...props} />
}

export { Badge, BadgeDot, badgeVariants, type BadgeProps }
