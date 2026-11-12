import * as React from 'react'
import { cn } from './lib/cn'

/**
 * （， and shadcn/ui Skeleton 1 ）。
 *
 * --ui-skeleton-base（、 1 ）， pulse 。
 * and to use （w-/h-/rounded-*）， and 。
 */
function Skeleton({ className, ...props }: React.ComponentProps<'div'>) {
  return (
    <div
      data-slot='skeleton'
      className={cn('animate-pulse rounded-md bg-skeleton-base', className)}
      {...props}
    />
  )
}

export { Skeleton }
