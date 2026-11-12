import * as React from 'react'
import { Button } from './button'
import { cva, type VariantProps } from 'class-variance-authority'
import { useRender } from '@base-ui/react/use-render'
import { useSelectionIndicator } from './lib/selection-indicator'
import { cn } from './lib/cn'

const NavListContext = React.createContext(false)

function NavList({ className, render, children, ...props }: useRender.ComponentProps<'div'>) {
  const { containerRef, indicator } = useSelectionIndicator(
    '[data-slot="nav-item"]', '[data-slot="nav-item"][data-active]',
  )
  const element = useRender({
    defaultTagName: 'div', render, ref: containerRef,
    props: {
      ...props,
      'data-slot': 'nav-list',
      className: cn('relative isolate flex min-h-0 flex-col gap-0.5', className),
      children: <>
        <span data-slot='nav-list-indicator' aria-hidden='true' {...indicator} />
        {children}
      </>,
    },
  })
  return <NavListContext.Provider value>{element}</NavListContext.Provider>
}

/**
 * / （，shadcn no has for ）。
 *
 * use Button ， in in 、 and 。
 *
 * `.nav-item` and `.pv`（ is 1 ）：
 * by padding 、 --r-md —— is 30px ， and 30px by 
 * 1 ， use 「 and by not 1 」 is this （ styles/theme.css）。
 *
 * ：
 * · default （optional in ）
 * · add 「＋ new …」： not is 1 、 not will has in ，
 * is as and up that 
 *
 * Note: is `<button>`，** not can by **。 need in 1 
 * by （delete ×）， by use in 1 、 that by 
 * —— HTML not button ， will 。
 */

const navItemVariants = cva(
  [
    'relative flex h-auto w-full items-center justify-start gap-2.5 whitespace-normal text-left active:translate-y-0',
    'border border-transparent rounded-md',
    'shadow-none',
    'transition-[background-color,border-color,color] duration-150 ease-out',
    'outline-none focus-visible:shadow-focus',
    'disabled:pointer-events-none disabled:opacity-45',
    '[&_svg]:pointer-events-none [&_svg]:shrink-0',
  ],
  {
    variants: {
      variant: {
        default: [
          'cursor-pointer px-2.5 py-2',
          'text-[12.5px] font-medium text-sidebar-fg',
          'hover:bg-nav-hover hover:text-foreground',
          // in ：（ / / 1 in 
          // styles/nav.css and theme.css --ui-nav-selected-* ， that ）。
          // from 400 to 500： will 1 。
          'data-[active]:border-nav-selected-border data-[active]:bg-nav-selected-bg data-[active]:font-medium data-[active]:text-nav-selected-fg',
        ],
        add: [
          'cursor-pointer justify-center px-2 py-1.5',
          'border-dashed border-border-strong bg-transparent',
          'text-[11.5px] text-muted-foreground',
          'hover:border-primary hover:text-primary-fg',
        ],
      },
    },
    defaultVariants: { variant: 'default' },
  }
)

type NavItemProps = Omit<React.ComponentProps<'button'>, 'className'> &
  VariantProps<typeof navItemVariants> & {
    /** in 。 data-active not is ， is as Tailwind can in */
    active?: boolean
    /** （ / ）； undefined not */
    count?: number | string
    /** */
    icon?: React.ReactNode
    /** （ ，） */
    trailing?: React.ReactNode
    className?: string
  }

function NavItem({
  className,
  variant,
  active = false,
  count,
  icon,
  trailing,
  children,
  ...props
}: NavItemProps) {
  const sliding = React.useContext(NavListContext)
  return (
    <Button
      type='button'
      variant='ghost'
      data-slot='nav-item'
      data-active={active ? '' : undefined}
      data-sliding={sliding ? '' : undefined}
      aria-current={active ? 'true' : undefined}
      className={cn(navItemVariants({ variant }), sliding && 'data-[active]:border-transparent data-[active]:bg-transparent data-[active]:hover:bg-transparent', className)}
      {...props}
    >
      {icon}
      <span data-slot='nav-item-label' className='min-w-0 flex-1 truncate'>
        {children}
      </span>
      {trailing}
      {count !== undefined && count !== null && (
        <span
          data-slot='nav-item-count'
          className={cn(
            'flex-none text-[10.5px] text-sidebar-muted [font-variant-numeric:tabular-nums]',
            // in down ，otherwise in up will 1 
            active && 'text-inherit'
          )}
        >
          {count}
        </span>
      )}
    </Button>
  )
}

export { NavList, NavItem, navItemVariants, type NavItemProps }
