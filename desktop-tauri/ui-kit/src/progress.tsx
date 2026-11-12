import * as React from 'react'
import { Progress as ProgressPrimitive } from '@base-ui/react/progress'
import { cn } from './lib/cn'

/**
 * （， and shadcn/ui Progress 1 ）。
 *
 * ：Progress is ，default 1 ProgressTrack + ProgressIndicator
 * （ value can use ）； need or when ， use ProgressLabel /
 * ProgressValue， or Track and Indicator。
 *
 * ：6px 、 surface-3、、 0.3s ——
 * for ui/css/components.css `.progress`。
 *
 * use Base UI Progress not is div： up `role="progressbar"` and 
 * `aria-valuenow/valuemin/valuemax`， can 。`value` null 
 * not （ will aria-valuenow）。
 */

function Progress({
  className,
  children,
  value,
  ...props
}: ProgressPrimitive.Root.Props) {
  return (
    <ProgressPrimitive.Root
      value={value}
      data-slot='progress'
      className={cn('flex w-full flex-wrap items-center gap-3', className)}
      {...props}
    >
      {children}
      <ProgressTrack>
        <ProgressIndicator />
      </ProgressTrack>
    </ProgressPrimitive.Root>
  )
}

function ProgressTrack({ className, ...props }: ProgressPrimitive.Track.Props) {
  return (
    <ProgressPrimitive.Track
      data-slot='progress-track'
      className={cn(
        'relative flex h-1.5 w-full items-center overflow-x-hidden overflow-y-hidden rounded-pill bg-surface-3',
        className
      )}
      {...props}
    />
  )
}

function ProgressIndicator({ className, ...props }: ProgressPrimitive.Indicator.Props) {
  return (
    <ProgressPrimitive.Indicator
      data-slot='progress-indicator'
      className={cn('h-full rounded-[inherit] bg-primary transition-[width] duration-300 ease-out data-[indeterminate]:w-1/3 data-[indeterminate]:animate-pulse', className)}
      {...props}
    />
  )
}

function ProgressLabel({ className, ...props }: ProgressPrimitive.Label.Props) {
  return (
    <ProgressPrimitive.Label
      data-slot='progress-label'
      className={cn('text-[11.5px] text-muted-foreground', className)}
      {...props}
    />
  )
}

function ProgressValue({ className, ...props }: ProgressPrimitive.Value.Props) {
  return (
    <ProgressPrimitive.Value
      data-slot='progress-value'
      className={cn('ml-auto text-[11.5px] tabular-nums text-muted-foreground', className)}
      {...props}
    />
  )
}

export { Progress, ProgressTrack, ProgressIndicator, ProgressLabel, ProgressValue }
