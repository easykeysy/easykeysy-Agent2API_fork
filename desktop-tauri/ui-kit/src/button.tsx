import * as React from 'react'
import { Button as BaseButton } from '@base-ui/react/button'
import { cva, type VariantProps } from 'class-variance-authority'
import { cn } from './lib/cn'

/**
 * by 。
 *
 * and shadcn/ui Button for ：variant default / outline / secondary /
 * ghost / destructive / link，size default / xs / sm / lg / icon / icon-xs /
 * icon-sm / icon-lg —— and ， by shadcn can in 。
 * （ styles/theme.css）， and has 1 。
 *
 * （shadcn no has ， has need need ）：
 * · `variant='dashed'` ，「」 not is 「」；
 * · `size='2xs'` / `'icon-2xs'` 16px ， to （chip） use 。
 * is variant / size ， not prop。
 *
 * and ui/css/components.css that for ：button.primary → default，
 * button → outline，button.danger → destructive，button.sm → sm，button.icon → icon。
 *
 * to Base UI Button： `<button>` ， data-pressed /
 * data-disabled etc 。
 */

const buttonVariants = cva(
  [
    'group/button inline-flex shrink-0 items-center justify-center gap-1.5 whitespace-nowrap border [font-family:inherit]',
    'font-medium transition-[background-color,border-color,color,transform,box-shadow] duration-150 ease-out',
    'outline-none select-none active:translate-y-[0.5px] focus-visible:shadow-focus',
    'disabled:pointer-events-none disabled:opacity-45 disabled:shadow-none',
    '[&_svg]:pointer-events-none [&_svg]:shrink-0',
  ],
  {
    variants: {
      variant: {
        /** by ： + */
        default:
          'border-primary bg-primary font-semibold text-primary-on shadow-glow hover:border-primary-hover hover:bg-primary-hover',
        /** default by ： + */
        outline:
          'border-control-border bg-control text-foreground shadow-1 hover:border-control-border-hover hover:bg-control-hover',
        secondary: 'border-transparent bg-secondary text-secondary-foreground hover:bg-secondary/80',
        ghost:
          'border-transparent bg-transparent text-subtle shadow-none hover:bg-control-hover hover:text-foreground',
        /** by ： + */
        destructive:
          'border-destructive-bd bg-destructive-soft text-destructive hover:bg-destructive-hover',
        /**
         * by ： is 「」 not is 「」—— 1 、 1 
         * this 「 this can 1 」 use ， and that by 1 
         * （ for ui/css `.alias-add` / `.pv-add`）。
         */
        dashed:
          'border-dashed border-border-strong bg-transparent text-muted-foreground shadow-none hover:border-primary hover:text-primary-fg',
        link: 'border-transparent bg-transparent text-primary shadow-none underline-offset-4 hover:underline',
      },
      size: {
        default: 'h-[30px] rounded-md px-3 text-[12.5px]',
        /**
         * （16px）： chip etc and delete ×。
         * down 24px（xs / icon-xs） 22px will ，
         * chip is 1 —— 1 is 。
         */
        '2xs': 'h-4 rounded-pill px-1.5 text-[10px]',
        xs: 'h-6 rounded-sm px-2 text-[11.5px]',
        sm: 'h-[26px] rounded-sm px-2.5 text-[12px]',
        lg: 'h-9 rounded-md px-4 text-[13px]',
        icon: 'h-[30px] w-[30px] rounded-md p-0',
        'icon-2xs': 'size-4 rounded-pill p-0',
        'icon-xs': 'size-6 rounded-sm p-0',
        'icon-sm': 'h-[26px] w-[26px] rounded-sm p-0',
        'icon-lg': 'size-9 rounded-md p-0',
        touch: 'h-[46px] rounded-md px-4 text-[14px]',
        'touch-sm': 'h-[34px] rounded-sm px-3 text-[12.5px]',
        'icon-touch': 'size-9 rounded-full p-0',
      },
    },
    defaultVariants: { variant: 'default', size: 'default' },
  }
)

type ButtonProps = Omit<React.ComponentProps<typeof BaseButton>, 'className'> &
  VariantProps<typeof buttonVariants> & {
    className?: string
  }

function Button({ className, variant, size, ...props }: ButtonProps) {
  return <BaseButton data-slot='button' className={cn(buttonVariants({ variant, size }), className)} {...props} />
}

export { Button, buttonVariants, type ButtonProps }
