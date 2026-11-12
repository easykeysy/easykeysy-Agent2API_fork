import * as React from 'react'
import { Switch as BaseSwitch } from '@base-ui/react/switch'
import { cn } from './lib/cn'

/**
 * 。
 *
 * for ui/css/components.css `.switch`：36×21 + 17px ，
 * in when 、 15px。 1 （cubic-bezier(.3,1.4,.6,1)），
 * —— this is has 1 ， ease。
 *
 * to Base UI Switch： is `<button role="switch">`（ not is 
 * checkbox）， Space/Enter、aria-checked by 。
 * that 「 input + 」 in and up 。
 *
 * use ：`<Switch checked={on} onCheckedChange={setOn} />`， need 1 
 * `<label>`（ not ）。
 */

type SwitchProps = Omit<React.ComponentProps<typeof BaseSwitch.Root>, 'className'> & {
  className?: string
  /**
   * 。`sm` is 24×14 ， to （chip） use —— has 22px ，
   * （36×21） will chip ， chip is 1 。
   */
  size?: 'sm' | 'default' | 'touch'
  variant?: 'default' | 'success'
}

function Switch({ className, size = 'default', variant = 'default', ...props }: SwitchProps) {
  const small = size === 'sm'
  return (
    <BaseSwitch.Root
      data-slot='switch'
      className={cn(
        'relative inline-flex flex-none cursor-pointer items-center rounded-pill border-0 bg-border-strong p-0',
        'transition-colors duration-[180ms] ease-out',
        variant === 'success' ? 'data-checked:bg-success' : 'data-checked:bg-primary',
        'focus-visible:shadow-focus focus-visible:outline-none',
        'data-disabled:cursor-not-allowed data-disabled:opacity-50',
        small ? 'h-[14px] w-6' : size === 'touch' ? 'h-6 w-10' : 'h-[21px] w-9',
        className
      )}
      {...props}
    >
      <BaseSwitch.Thumb
        data-slot='switch-thumb'
        className={cn(
          'block rounded-full bg-primary-on shadow-1',
          'transition-transform duration-[180ms] [transition-timing-function:cubic-bezier(.3,1.4,.6,1)]',
          small
            ? 'size-[10px] translate-x-0.5 data-checked:translate-x-[10px]'
            : size === 'touch'
              ? 'size-5 translate-x-0.5 data-checked:translate-x-[18px]'
              : 'size-[17px] translate-x-0.5 data-checked:translate-x-[17px]'
        )}
      />
    </BaseSwitch.Root>
  )
}

export { Switch, type SwitchProps }
