import * as React from 'react'
import { cva, type VariantProps } from 'class-variance-authority'
import { cn } from './lib/cn'
import { Button } from './button'
import { Input } from './input'
import { Textarea } from './textarea'

/**
 * —— before / after 。
 *
 * 、props and as shadcn/ui InputGroup：
 *
 *   InputGroup
 * ├── InputGroupInput / InputGroupTextarea （，data-slot="input-group-control"）
 *   ├── InputGroupAddon                        （align: inline-start | inline-end | block-start | block-end）
 *   ├── InputGroupButton                       （size: xs | sm | icon-xs | icon-sm）
 * └── InputGroupText （ before ， https:// 、$ ）
 *
 * and 1 as ， when ：
 *
 * 1) addon in DOM ** in after **， to `align` order-first / order-last ——
 * this Tab is after addon（addon by not will in before ）。
 * 2) addon is flex ， not is for ； addon empty will 
 * （ by ， has as ）。
 * 3) 、、 and in **** up ， use InputGroupInput that 
 * （otherwise will 、）。
 *
 * ： 30px 、、--r-md ；addon 9px，
 * in 10px —— and ui/css that `.input-affix` 1 。
 */

function InputGroup({ className, ...props }: React.ComponentProps<'div'>) {
  return (
    <div
      data-slot='input-group'
      role='group'
      className={cn(
        'group/input-group relative flex h-[30px] w-full min-w-0 items-center rounded-md border border-control-border bg-control max-[767px]:h-10',
        'transition-[border-color,box-shadow] duration-150 ease-out',
        'outline-none',
        // and focus-visible （ not is ）
        'has-[[data-slot=input-group-control]:focus-visible]:border-primary',
        'has-[[data-slot=input-group-control]:focus-visible]:shadow-focus',
        // ： hover when ， and 1 。
        // mustexcluding focus-visible —— has- ，，
        // not excluding in up will hover not is 。
        'has-[[data-slot=input-group-control]:hover:not(:focus-visible)]:border-control-border-hover',
        'has-[>textarea]:h-auto',
        // before after addon after ， in that 1 need 1 
        'has-[>[data-align=inline-start]]:[&>[data-slot=input-group-control]]:pl-1.5',
        'has-[>[data-align=inline-end]]:[&>[data-slot=input-group-control]]:pr-1.5',
        // up down addon： to ， for 
        'has-[>[data-align=block-start]]:h-auto has-[>[data-align=block-start]]:flex-col',
        'has-[>[data-align=block-start]]:[&>[data-slot=input-group-control]]:pb-1.5',
        'has-[>[data-align=block-end]]:h-auto has-[>[data-align=block-end]]:flex-col',
        'has-[>[data-align=block-end]]:[&>[data-slot=input-group-control]]:pt-1.5',
        className
      )}
      {...props}
    />
  )
}

const inputGroupAddonVariants = cva(
  [
    'flex h-auto cursor-text items-center justify-center gap-1.5 select-none',
    'text-[12px] font-medium text-muted-foreground',
    'group-data-[disabled=true]/input-group:opacity-50',
  ],
  {
    variants: {
      align: {
        'inline-start': 'order-first pl-[9px] has-[>button]:-ml-1',
        'inline-end': 'order-last pr-[9px] has-[>button]:-mr-1',
        'block-start': 'order-first w-full justify-start px-2.5 pt-2 group-has-[>input]/input-group:pt-1.5',
        'block-end': 'order-last w-full justify-start px-2.5 pb-2 group-has-[>input]/input-group:pb-1.5',
      },
    },
    defaultVariants: { align: 'inline-start' },
  }
)

type InputGroupAddonProps = React.ComponentProps<'div'> &
  VariantProps<typeof inputGroupAddonVariants>

function InputGroupAddon({ className, align = 'inline-start', ...props }: InputGroupAddonProps) {
  return (
    <div
      role='group'
      data-slot='input-group-addon'
      data-align={align}
      className={cn(inputGroupAddonVariants({ align }), className)}
      onClick={event => {
        // addon empty； by has as ，
        if ((event.target as HTMLElement).closest('button')) return
        event.currentTarget.parentElement
          ?.querySelector<HTMLInputElement>('input')
          ?.focus()
      }}
      {...props}
    />
  )
}

const inputGroupButtonVariants = cva('flex items-center shadow-none', {
  variants: {
    size: {
      xs: 'h-6 gap-1 rounded-xs px-1.5',
      sm: 'h-8 gap-1.5 rounded-sm px-2.5',
      'icon-xs': 'size-6 rounded-xs p-0',
      'icon-sm': 'size-8 rounded-sm p-0',
    },
  },
  defaultVariants: { size: 'xs' },
})

type InputGroupButtonProps = Omit<React.ComponentProps<typeof Button>, 'size'> &
  VariantProps<typeof inputGroupButtonVariants> & {
    type?: 'button' | 'submit' | 'reset'
  }

function InputGroupButton({
  className,
  type = 'button',
  variant = 'ghost',
  size = 'xs',
  ...props
}: InputGroupButtonProps) {
  return (
    <Button
      type={type}
      data-size={size}
      variant={variant}
      className={cn(inputGroupButtonVariants({ size }), className)}
      {...props}
    />
  )
}

function InputGroupText({ className, ...props }: React.ComponentProps<'span'>) {
  return (
    <span
      className={cn(
        'flex items-center gap-1.5 text-[12px] text-muted-foreground [&_svg]:pointer-events-none',
        className
      )}
      {...props}
    />
  )
}

function InputGroupInput({ className, ...props }: React.ComponentProps<'input'>) {
  return (
    <Input
      data-slot='input-group-control'
      className={cn(
        // Input that ：、、、
        'flex-1 rounded-none border-0 bg-transparent shadow-none focus:border-0 focus:shadow-none',
        className
      )}
      {...props}
    />
  )
}

function InputGroupTextarea({ className, ...props }: React.ComponentProps<'textarea'>) {
  return (
    <Textarea
      data-slot='input-group-control'
      className={cn(
        'flex-1 resize-none rounded-none border-0 bg-transparent py-2 shadow-none focus:border-0 focus:shadow-none',
        className
      )}
      {...props}
    />
  )
}

export {
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  InputGroupText,
  InputGroupInput,
  InputGroupTextarea,
}
