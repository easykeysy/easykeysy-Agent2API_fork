import * as React from 'react'
import { cn } from './lib/cn'
import { t } from './i18n'

/**
 * （， and shadcn/ui Spinner 1 ： 1 loader ）。
 *
 * shadcn that use is （lucide Loader2Icon etc ）， no ，
 * this use etc SVG：24 、stroke 2、 in up —— and 1 
 * （`role="status"` + `aria-label` + `data-slot="spinner"`）， is path。
 *
 * currentColor： by 、 not use ， in 。
 * default size-4（ and 1 ）， need that 12px `className="size-3"`。
 * by 1 （0.7s，Tailwind default 1s）。
 */

function Spinner({ className, ...props }: React.ComponentProps<'svg'>) {
  return (
    <svg
      data-slot='spinner'
      role='status'
      aria-label={t('加载中')}
      viewBox='0 0 24 24'
      fill='none'
      stroke='currentColor'
      strokeWidth='2'
      strokeLinecap='round'
      className={cn('size-4 animate-spin [animation-duration:0.7s]', className)}
      {...props}
    >
      {/* loader ： 1 ， has to */}
      <path d='M21 12a9 9 0 1 1-6.219-8.56' />
    </svg>
  )
}

export { Spinner }
