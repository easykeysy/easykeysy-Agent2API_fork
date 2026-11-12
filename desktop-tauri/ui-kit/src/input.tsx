import * as React from 'react'
import { Input as InputPrimitive } from '@base-ui/react/input'
import { cn } from './lib/cn'

/**
 * （， and shadcn/ui Input 1 ）。
 *
 * is `<input>` ：`data-slot="input"`、props ，
 * not prop。 need / / by before after ， use InputGroup —— that is shadcn
 * to this need （ before has 1 `icon` prop， already ）。
 *
 * ：30px 、、1px 、12.5px ，hover 、
 * focus and —— for ui/css/components.css `input[type=…]` 1 。
 *
 * use Base UI Input not is `<input>`： data-disabled / data-valid etc 
 * ， will Field when not use 。
 */

function Input({ className, type, ref, ...props }: React.ComponentProps<'input'>) {
  if (type === 'file' || type === 'hidden') {
    return <input ref={ref} type={type} data-slot='input' className={cn(className)} {...props} />
  }
  return (
    <InputPrimitive
      ref={ref as React.Ref<HTMLElement>}
      type={type}
      data-slot='input'
      className={cn(
        'h-[30px] w-full min-w-0 rounded-md border border-control-border bg-control px-2.5',
        'text-[12.5px] text-foreground outline-none max-[767px]:h-10 max-[767px]:text-base',
        'transition-[border-color,box-shadow] duration-150 ease-out',
        'placeholder:text-muted-foreground',
        'hover:border-control-border-hover',
        'focus:border-primary focus:shadow-focus',
        'disabled:pointer-events-none disabled:cursor-not-allowed disabled:bg-surface-3 disabled:text-muted-foreground',
        'file:inline-flex file:border-0 file:bg-transparent file:text-foreground',
        className
      )}
      {...props}
    />
  )
}

export { Input }
