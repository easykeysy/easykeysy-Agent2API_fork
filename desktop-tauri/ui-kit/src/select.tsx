import * as React from 'react'
import { Select as BaseSelect } from '@base-ui/react/select'
import { cn } from './lib/cn'

/**
 * down 。
 *
 * for ui/css/select.css： 30px 、、1px 、--r-md 、
 * 12.5px 、none（ and input ）； z-35、--r-lg 、
 * （--ui-raised）+ + --shadow-3， 28px 、 in 。
 *
 * and has select.js ： that is 「 `<select>`」—— 1 
 * and ， as 。 this is （Base UI Select），
 * 、 to 、typeahead、`aria-activedescendant` ，
 * new use ； has etc when ， can and 。
 *
 * use ：
 *   <Select value={v} onValueChange={setV}>
 *     <SelectTrigger><SelectValue /></SelectTrigger>
 *     <SelectContent>
 * <SelectItem value="a"> A</SelectItem>
 *     </SelectContent>
 *   </Select>
 */

const Select = BaseSelect.Root
const SelectValue = BaseSelect.Value
const SelectGroup = BaseSelect.Group
/** ：shadcn SelectLabel（Base UI GroupLabel） */
const SelectLabel = BaseSelect.GroupLabel
const SelectSeparator = BaseSelect.Separator

type SelectTriggerProps = Omit<React.ComponentProps<typeof BaseSelect.Trigger>, 'className'> & {
  className?: string
}

function SelectTrigger({ className, children, ...props }: SelectTriggerProps) {
  return (
    <BaseSelect.Trigger
      data-slot='select-trigger'
      className={cn(
        'inline-flex h-[30px] min-w-0 cursor-pointer items-center justify-between gap-1.5 rounded-md border border-control-border bg-control px-2.5',
        // in （SelectValue span） is flex ，default
        // min-width:auto not ： when will 、
        // （）。truncate overflow hidden，
        // ， —— in and title 。
        // this will span 1 in ，truncate not display， for none。
        '[&>span]:min-w-0 [&>span]:truncate',
        'text-[12.5px] font-normal text-foreground max-[767px]:h-10 max-[767px]:text-base',
        'transition-colors duration-150 ease-out',
        'hover:border-control-border-hover hover:bg-control-hover',
        'focus-visible:border-primary focus-visible:shadow-focus focus-visible:outline-none',
        'data-disabled:cursor-not-allowed data-disabled:bg-control data-disabled:text-muted-foreground data-disabled:opacity-60',
        className
      )}
      {...props}
    >
      {children}
      <BaseSelect.Icon className='flex flex-none text-muted-foreground transition-transform duration-[180ms] data-popup-open:rotate-180'>
        <svg viewBox='0 0 10 6' className='size-2.5' aria-hidden='true'>
          <path d='M1 1l4 4 4-4' stroke='currentColor' strokeWidth='1.4' fill='none' strokeLinecap='round' strokeLinejoin='round' />
        </svg>
      </BaseSelect.Icon>
    </BaseSelect.Trigger>
  )
}

type SelectContentProps = Omit<React.ComponentProps<typeof BaseSelect.Popup>, 'className'> & {
  className?: string
  /** and */
  sideOffset?: number
}

function SelectContent({ className, sideOffset = 4, children, ...props }: SelectContentProps) {
  return (
    <BaseSelect.Portal>
      {/* alignItemWithTrigger={false}： Base UI default 「 in for 」。
           that by and top/bottom（fixed + from ），
           and in Popup up max-h limit： in in 、
           when （ —— in 
          40+ ）， will to ，。
           after to ： down 、empty not to up ，
           to in 。 for default and 
          base-ui.com/react/components/select Positioning 1 。 */}
      <BaseSelect.Positioner sideOffset={sideOffset} alignItemWithTrigger={false} className='z-[35]'>
        <BaseSelect.Popup
          data-slot='select-content'
          className={cn(
            'max-h-[260px] min-w-[var(--anchor-width)] overflow-y-auto overscroll-contain rounded-lg border border-border-strong bg-raised p-[5px] shadow-3',
            'outline-none',
            // not /： after need ，transform will 
            'data-open:animate-in data-open:fade-in-0',
            'data-closed:animate-out data-closed:fade-out-0',
            className
          )}
          {...props}
        >
          {children}
        </BaseSelect.Popup>
      </BaseSelect.Positioner>
    </BaseSelect.Portal>
  )
}

type SelectItemProps = Omit<React.ComponentProps<typeof BaseSelect.Item>, 'className'> & {
  className?: string
}

function SelectItem({ className, children, ...props }: SelectItemProps) {
  return (
    <BaseSelect.Item
      data-slot='select-item'
      className={cn(
        'flex min-h-7 cursor-pointer items-center gap-2 rounded-sm px-[9px] text-[12.5px] leading-[1.4] text-foreground select-none',
        'transition-colors duration-[120ms] ease-out',
        'data-[highlighted]:bg-nav-hover',
        // use data-[selected] not is shadcn data-selected ： after 
        // [data-selected="true"]， Base UI is data-selected=""（ in 
        // in ），empty not up ， in will and 。
        'data-[selected]:bg-primary-soft data-[selected]:font-semibold data-[selected]:text-primary-fg',
        'data-[highlighted]:data-[selected]:bg-primary-tint',
        'data-disabled:pointer-events-none data-disabled:font-normal data-disabled:text-muted-foreground',
        className
      )}
      {...props}
    >
      <BaseSelect.ItemText className='min-w-0 flex-1 truncate'>{children}</BaseSelect.ItemText>
      <BaseSelect.ItemIndicator className='flex flex-none text-primary'>
        <svg viewBox='0 0 12 12' className='size-3' aria-hidden='true'>
          <path d='M2.5 6.2 4.8 8.5 9.5 3.8' stroke='currentColor' strokeWidth='1.8' fill='none' strokeLinecap='round' strokeLinejoin='round' />
        </svg>
      </BaseSelect.ItemIndicator>
    </BaseSelect.Item>
  )
}

export {
  Select,
  SelectTrigger,
  SelectValue,
  SelectContent,
  SelectItem,
  SelectGroup,
  SelectLabel,
  SelectSeparator,
}
