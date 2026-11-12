import * as React from 'react'
import { Checkbox as CheckboxPrimitive } from '@base-ui/react/checkbox'
import { cn } from './lib/cn'

/**
 * （， and shadcn/ui Checkbox 1 ）。
 *
 * （Base UI Checkbox + ）， not is `<input type=checkbox>`：
 * by ，accent-color can ，// not can 。
 * after 14px、 --r-xs 1 、 in 。
 *
 * need need 「 in 」 when `indeterminate` —— input not this 。
 * and use SVG（shadcn that ， no ）。
 */

function Checkbox({ className, ...props }: CheckboxPrimitive.Root.Props) {
  return (
    <CheckboxPrimitive.Root
      data-slot='checkbox'
      className={cn(
        'group/checkbox relative inline-flex size-3.5 shrink-0 cursor-pointer items-center justify-center rounded-[3px] border border-control-border bg-control',
        'outline-none transition-colors duration-150 ease-out',
        'hover:border-control-border-hover',
        // Base UI in 「」 and 「 in 」 down data-checked，
        // need aria-checked（ in when is "indeterminate"）
        'data-checked:border-primary data-checked:bg-primary',
        'focus-visible:shadow-focus',
        'data-disabled:cursor-not-allowed data-disabled:opacity-45',
        className
      )}
      {...props}
    >
      <CheckboxPrimitive.Indicator
        data-slot='checkbox-indicator'
        className='grid place-content-center text-primary-foreground data-unchecked:hidden'
      >
        <svg viewBox='0 0 12 12' className='size-2.5' aria-hidden='true'>
          {/* in ， for —— and 1 。
               in Root up 、 in Indicator ， need group 。 */}
          <path
            d='M2 6h8'
            stroke='currentColor'
            strokeWidth='2'
            strokeLinecap='round'
            fill='none'
            className='hidden group-aria-[checked=indeterminate]/checkbox:block'
          />
          <path
            d='M2.5 6.2 4.8 8.5 9.5 3.8'
            stroke='currentColor'
            strokeWidth='1.8'
            strokeLinecap='round'
            strokeLinejoin='round'
            fill='none'
            className='group-aria-[checked=indeterminate]/checkbox:hidden'
          />
        </svg>
      </CheckboxPrimitive.Indicator>
    </CheckboxPrimitive.Root>
  )
}

export { Checkbox }
