import * as React from 'react'
import { Combobox as ComboboxPrimitive } from '@base-ui/react/combobox'
import { cn } from './lib/cn'
import { t } from './i18n'

/**
 * down 。
 *
 * and shadcn/ui Combobox 1 （shadcn is Combobox `multiple`，
 * no has 1 MultiSelect ）—— use Combobox ，
 * this is 「、」 that 1 。
 *
 * Why is Combobox not is Select multiple：Base UI Select is ，
 * can Checkbox ， and aria 。Combobox supports
 * `multiple`，and** by ** —— 1 ， can in is need 。
 *
 * for has down （ui/css/select.css `.select-shell-multi`）：
 * and SelectTrigger 1 ， 1 no when `placeholder`
 * （empty is 「unlimited」，must —— 1 empty will as 「 not ＝ not can use 」）；
 * in when by `、` （ and select.js selectedText 1 ）。
 *
 * use ：
 *   <MultiSelect
 *     value={ids}
 *     onValueChange={setIds}
 *     options={[{ value: 'wb', label: 'WorkBuddy' }]}
 * placeholder='empty = unlimited'
 *   />
 */

type MultiSelectOption<Value extends string = string> = {
  value: Value
  label: string
  /**
   * not optional（ in ）。
   *
   * use 「 this 1 in 、 in not can 」—— e.g.「」 already use 
   * ： from will use as ，。
   * and SelectItem `disabled` 1 。
   */
  disabled?: boolean
}

type MultiSelectProps<Value extends string = string> = {
  /** in ； not has */
  value: readonly Value[]
  onValueChange: (value: Value[]) => void
  options: readonly MultiSelectOption<Value>[]
  /** 1 no when up */
  placeholder?: string
  /** not to when */
  emptyHint?: string
  /** */
  searchPlaceholder?: string
  disabled?: boolean
  className?: string
  /** up id： to can `<label htmlFor>` use （ no has can label when use aria-label ） */
  id?: string
  /** */
  title?: string
  /** none（ up no has can label when ） */
  'aria-label'?: string
}

/** in （ and select.js selectedText 1 ） */
const JOINER = '、'

function MultiSelect<Value extends string = string>({
  value,
  onValueChange,
  options,
  placeholder = '',
  emptyHint = t('没有匹配的选项'),
  searchPlaceholder = t('搜索…'),
  disabled = false,
  className,
  id,
  title,
  ...props
}: MultiSelectProps<Value>) {
  /**
   * for is value ，Base UI is for 。
   * to in this ， use not for use is ——
   * `isItemEqualToValue` must by value ， not can use default Object.is：
   * use render options when ，Object.is will not 「 is that 1 」，
   * in will no 。
   */
  const selected = React.useMemo(
    () => options.filter(option => value.includes(option.value)),
    [options, value]
  )

  const summary = selected.length
    ? selected.map(option => option.label).join(JOINER)
    : placeholder

  return (
    <ComboboxPrimitive.Root
      items={options}
      multiple
      value={selected}
      onValueChange={next => onValueChange(next.map(option => option.value))}
      isItemEqualToValue={(a, b) => a?.value === b?.value}
      disabled={disabled}
    >
      <ComboboxPrimitive.Trigger
        data-slot='multi-select-trigger'
        id={id}
        title={title}
        aria-label={props['aria-label']}
        className={cn(
          'inline-flex h-[30px] min-w-0 cursor-pointer items-center justify-between gap-1.5 rounded-md border border-control-border bg-control px-2.5',
          'text-[12.5px] font-normal text-foreground',
          'transition-colors duration-150 ease-out',
          'hover:border-control-border-hover hover:bg-control-hover',
          'focus-visible:border-primary focus-visible:shadow-focus focus-visible:outline-none',
          'data-disabled:cursor-not-allowed data-disabled:bg-control data-disabled:text-muted-foreground data-disabled:opacity-60',
          className
        )}
      >
        {/* 1 no when data-placeholder ， and Select 1 */}
        <span
          data-slot='multi-select-value'
          className='min-w-0 flex-1 truncate text-left data-[placeholder]:text-muted-foreground'
          data-placeholder={selected.length ? undefined : ''}
          title={summary}
        >
          {summary}
        </span>
        <ComboboxPrimitive.Icon className='flex flex-none text-muted-foreground transition-transform duration-[180ms] data-popup-open:rotate-180'>
          <svg viewBox='0 0 10 6' className='size-2.5' aria-hidden='true'>
            <path
              d='M1 1l4 4 4-4'
              stroke='currentColor'
              strokeWidth='1.4'
              fill='none'
              strokeLinecap='round'
              strokeLinejoin='round'
            />
          </svg>
        </ComboboxPrimitive.Icon>
      </ComboboxPrimitive.Trigger>

      <ComboboxPrimitive.Portal>
        <ComboboxPrimitive.Positioner sideOffset={4} className='z-[35]'>
          <ComboboxPrimitive.Popup
            data-slot='multi-select-content'
            className={cn(
              // （--anchor-width）， to 1 down ：，
              // （ / ）
              'flex max-h-[280px] w-[max(var(--anchor-width),220px)] flex-col overflow-hidden',
              'rounded-lg border border-border-strong bg-raised shadow-3 outline-none',
              'data-open:animate-in data-open:fade-in-0',
              'data-closed:animate-out data-closed:fade-out-0'
            )}
          >
            {/* in （input-inside-popup）：「」 ，
                 not is 1 can —— that will as can 。 */}
            <div className='flex flex-none items-center gap-1.5 border-b border-hairline px-2.5 py-1.5'>
              <svg viewBox='0 0 12 12' className='size-3 flex-none text-muted-foreground' aria-hidden='true'>
                <circle cx='5' cy='5' r='3.4' stroke='currentColor' strokeWidth='1.3' fill='none' />
                <path d='M7.6 7.6 10.5 10.5' stroke='currentColor' strokeWidth='1.3' strokeLinecap='round' />
              </svg>
              <ComboboxPrimitive.Input
                placeholder={searchPlaceholder}
                className='min-w-0 flex-1 border-0 bg-transparent p-0 text-[12.5px] text-foreground outline-none placeholder:text-muted-foreground'
              />
            </div>
            <ComboboxPrimitive.Empty>
              <div className='px-2.5 py-3 text-[11.5px] leading-[1.5] text-muted-foreground'>
                {emptyHint}
              </div>
            </ComboboxPrimitive.Empty>
            <ComboboxPrimitive.List className='min-h-0 flex-1 overflow-y-auto overscroll-contain p-[5px]'>
              {(option: MultiSelectOption<Value>) => (
                <ComboboxPrimitive.Item
                  key={option.value}
                  value={option}
                  disabled={option.disabled === true}
                  className={cn(
                    'grid cursor-pointer grid-cols-[14px_minmax(0,1fr)] items-center gap-2 rounded-sm px-[9px] py-[5px]',
                    'text-[12.5px] leading-[1.4] text-foreground select-none outline-none',
                    'transition-colors duration-[120ms] ease-out',
                    'data-[highlighted]:bg-nav-hover',
                    // in + （ and SelectItem 1 ）。
                    // use data-[selected] not is shadcn data-selected ：
                    // Base UI is data-selected=""，empty not up true。
                    'data-[selected]:bg-primary-soft data-[selected]:font-semibold data-[selected]:text-primary-fg',
                    'data-[highlighted]:data-[selected]:bg-primary-tint',
                    'data-disabled:pointer-events-none data-disabled:text-muted-foreground'
                  )}
                >
                  <ComboboxPrimitive.ItemIndicator className='col-start-1 flex text-primary'>
                    <svg viewBox='0 0 12 12' className='size-3' aria-hidden='true'>
                      <path
                        d='M2.5 6.2 4.8 8.5 9.5 3.8'
                        stroke='currentColor'
                        strokeWidth='1.8'
                        fill='none'
                        strokeLinecap='round'
                        strokeLinejoin='round'
                      />
                    </svg>
                  </ComboboxPrimitive.ItemIndicator>
                  <span className='col-start-2 min-w-0 truncate'>{option.label}</span>
                </ComboboxPrimitive.Item>
              )}
            </ComboboxPrimitive.List>
          </ComboboxPrimitive.Popup>
        </ComboboxPrimitive.Positioner>
      </ComboboxPrimitive.Portal>
    </ComboboxPrimitive.Root>
  )
}

export { MultiSelect, type MultiSelectOption, type MultiSelectProps }
