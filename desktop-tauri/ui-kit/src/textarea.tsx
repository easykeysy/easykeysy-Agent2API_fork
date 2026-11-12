import * as React from 'react'
import { cn } from './lib/cn'

/**
 * （， and shadcn/ui Textarea 1 ）。
 *
 * `<textarea>` ：`data-slot="textarea"`、props 。
 * ： 104px、10/12 、 etc （config）、
 * can to ， and ui/css/components.css `textarea` then 1 。
 */
function Textarea({ className, ...props }: React.ComponentProps<'textarea'>) {
  return (
    <textarea
      data-slot='textarea'
      className={cn(
        'min-h-26 w-full resize-y rounded-md border border-control-border bg-control px-3 py-2.5',
        'font-mono text-[12px] leading-[1.6] text-foreground outline-none max-[767px]:text-base',
        'transition-[border-color,box-shadow] duration-150 ease-out',
        'placeholder:text-muted-foreground',
        'hover:border-control-border-hover',
        'focus:border-primary focus:shadow-focus',
        'disabled:cursor-not-allowed disabled:bg-surface-3 disabled:text-muted-foreground',
        className
      )}
      {...props}
    />
  )
}

export { Textarea }
