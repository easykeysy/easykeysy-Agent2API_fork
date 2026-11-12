import * as React from 'react'
import { Radio } from '@base-ui/react/radio'
import { RadioGroup as RadioGroupPrimitive } from '@base-ui/react/radio-group'
import { cn } from './lib/cn'

/**
 * （， and shadcn/ui RadioGroup 1 ：`data-slot="radio-group"` /
 * `data-slot="radio-group-item"` / `data-slot="radio-group-indicator"`）。
 *
 * （ not is `<input type=radio>`）： by ，
 * accent-color can 。 after 14px（ and Checkbox ）、 in +
 * ， and has `.clear-mode input[type="radio"]` 1 。
 *
 * use （）：
 *   <RadioGroup value={v} onValueChange={setV}>
 *     <RadioGroupItem value="a" />
 *     <RadioGroupItem value="b" />
 *   </RadioGroup>
 *
 * 、roving tabindex、`role="radiogroup"` by Base UI ；
 * to in in ， and as 。
 *
 * Note: Radio.Root is `<span>` + `<input>`， not is `<button>`。
 * `render={<button />}` —— that will CompositeItem to ，
 * to will （ segmented-control.tsx 1 ）。
 */

type RadioGroupProps<Value = string> = Omit<
  React.ComponentProps<typeof RadioGroupPrimitive<Value>>,
  'className'
> & { className?: string }

function RadioGroup<Value = string>({ className, ...props }: RadioGroupProps<Value>) {
  return (
    <RadioGroupPrimitive<Value>
      data-slot='radio-group'
      className={cn('flex flex-col gap-2', className)}
      {...props}
    />
  )
}

type RadioGroupItemProps = Omit<React.ComponentProps<typeof Radio.Root>, 'className'> & {
  className?: string
}

function RadioGroupItem({ className, children, ...props }: RadioGroupItemProps) {
  return (
    <Radio.Root
      data-slot='radio-group-item'
      className={cn(
        'relative inline-flex size-3.5 flex-none cursor-pointer items-center justify-center rounded-full border border-control-border bg-control',
        'outline-none transition-colors duration-150 ease-out',
        'hover:border-control-border-hover',
        'data-checked:border-primary data-checked:bg-primary',
        'focus-visible:shadow-focus',
        'data-disabled:cursor-not-allowed data-disabled:opacity-45',
        className
      )}
      {...props}
    >
      {children}
      <Radio.Indicator
        data-slot='radio-group-indicator'
        className='grid place-content-center data-unchecked:hidden'
      >
        {/* in ： up 1 ， and */}
        <span className='block size-[5px] rounded-full bg-primary-foreground' />
      </Radio.Indicator>
    </Radio.Root>
  )
}

export { RadioGroup, RadioGroupItem, type RadioGroupProps, type RadioGroupItemProps }
