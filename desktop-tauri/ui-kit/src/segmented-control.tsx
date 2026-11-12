import * as React from 'react'
import { Radio } from '@base-ui/react/radio'
import { RadioGroup } from '@base-ui/react/radio-group'
import { cn } from './lib/cn'
import { useSelectionIndicator } from './lib/selection-indicator'

/**
 * ： 1 。 this 1 。
 *
 * as （、 to 、roving tabindex、） to Base UI 
 * RadioGroup； by selection.css 1 。
 *
 * Why use RadioGroup not is ToggleGroup： this is 「 1 」 ，
 * （`role="radiogroup"`， to in ） by 
 * （`aria-pressed`， to 、 need Space/Enter ） use for 
 * ， is 「 by already in 」 not is 「 by down 」。
 *
 * Radio use Base UI default `<span role="radio">`，** not need `render={<button />}`
 * `nativeButton`**： that will to 。RadioRoot in is `CompositeItem`
 * ，props and as `[compositeProps, ...props, elementProps]`， `useButton`
 * props in compositeProps after ， will CompositeItem to 
 * —— button down by to not ，span down 。
 *
 * in use 1 ， not in —— this when can to 
 * ， not is 。 in and 。
 * in not ， new 。
 */

type SegmentedControlOption<Value extends string> = {
  value: Value
  label: React.ReactNode
  /**
   * ： not not 。
   * 0 when by 「empty」， and has 1 。
   */
  count?: number
  /**
   * （ title）。 no has ， can this ——
   * e.g.set「（ IP）」Why in （ by IP ， IP use 1 429）。
   */
  title?: string
  disabled?: boolean
}

type SegmentedControlProps<Value extends string> = {
  options: readonly SegmentedControlOption<Value>[]
  /** in ： not has ， by use to */
  value: Value
  onValueChange: (value: Value) => void
  /** none： this 1 in */
  'aria-label': string
  disabled?: boolean
  /** to up （ is ） */
  className?: string
}

function SegmentedControl<Value extends string>({
  options,
  value,
  onValueChange,
  disabled = false,
  className,
  ...props
}: SegmentedControlProps<Value>) {
  const { containerRef, indicator } = useSelectionIndicator(
    '[data-slot="segmented-item"]', '[data-slot="segmented-item"][data-checked]',
  )

  return (
    <RadioGroup<Value>
      ref={containerRef}
      data-slot='segmented-control'
      className={cn(
        'relative inline-flex gap-0.5 rounded-md border border-control-border bg-control p-0.5',
        className
      )}
      value={value}
      disabled={disabled}
      onValueChange={next => onValueChange(next)}
      {...props}
    >
      {/* ： by JS ， 1 to down */}
      <span aria-hidden='true' data-slot='segmented-thumb' {...indicator} />
      {options.map(option => (
        <Radio.Root
          key={option.value}
          value={option.value}
          data-slot='segmented-item'
          title={option.title}
          disabled={option.disabled}
          data-zero={option.count === 0 ? 'true' : undefined}
          className={cn(
            'group/seg relative z-[1] inline-flex h-6 items-center gap-[5px] rounded-sm border-0 bg-transparent px-2.5',
            'text-xs font-medium whitespace-nowrap text-subtle select-none',
            'transition-[background-color,color] duration-150 ease-out',
            'hover:bg-nav-hover hover:text-foreground',
            // in （），
            'data-checked:bg-transparent data-checked:text-primary-fg data-checked:hover:bg-transparent',
            'data-disabled:cursor-not-allowed data-disabled:opacity-40',
            // as 0 ；current in that 1 as 0 
            'data-[zero=true]:data-unchecked:opacity-45'
          )}
        >
          {option.label}
          {option.count !== undefined && (
            <span
              className={cn(
                'min-w-4 rounded-pill bg-surface-3 px-[5px] text-center text-[10.5px] font-semibold tabular-nums text-muted-foreground',
                'group-data-checked/seg:bg-primary-tint group-data-checked/seg:text-primary-fg'
              )}
            >
              {option.count}
            </span>
          )}
        </Radio.Root>
      ))}
    </RadioGroup>
  )
}

export { SegmentedControl, type SegmentedControlOption, type SegmentedControlProps }
