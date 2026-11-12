import * as React from 'react'
import { cn } from './lib/cn'

/**
 * （， and shadcn/ui Label 1 ：`data-slot="label"`）。
 *
 * for ui/css/components.css `.field .label`：11.5px、。
 * used for not in Field ； need need when use FieldLabel。
 */
function Label({ className, ...props }: React.ComponentProps<'label'>) {
  return (
    <label
      data-slot='label'
      className={cn('flex items-center gap-[6px] text-[11.5px] text-muted-foreground', className)}
      {...props}
    />
  )
}

export { Label }
