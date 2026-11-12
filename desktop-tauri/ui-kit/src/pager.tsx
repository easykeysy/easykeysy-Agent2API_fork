import * as React from 'react'
import { Button } from './button'
import { cn } from './lib/cn'
import { t } from './i18n'

/**
 * （，shadcn no has for ）。
 *
 * Why 1 not is shadcn Pagination： that is 「」
 * （ 1 href ， by ），； use 
 * （request、） is **offset + after to total**，
 * is 「 up 1 / down 1 / N / M 」， no has 、
 * will in when 1 。
 *
 * this this 1 ： by + 1 。 by use （ not can 、
 * not can ） 1 and ， use 1 。
 *
 * for ui/css `.log-pager` / `.log-pager-info`： sm outline by ，
 * use etc （tabular-nums）， when not 。
 *
 * use ：
 *   <Pager page={page} pageCount={pageCount} onPageChange={setPage} disabled={loading} />
 */

type PagerProps = {
  /** current，1 */
  page: number
  /** total（ 1； use not when 1） */
  pageCount: number
  onPageChange: (page: number) => void
  /** use （e.g.request in ）： by not response */
  disabled?: boolean
  /** before ，default「」；「 1 / 3 」 */
  className?: string
  /** for when use （ in by ） */
  infoFirst?: boolean
}

function Pager({
  page,
  pageCount,
  onPageChange,
  disabled = false,
  className,
  infoFirst = false,
}: PagerProps) {
  // ： after no when pageCount can can is 0 or NaN，「 1 / 0 」
  const total = Math.max(1, Number(pageCount) || 1)
  const current = Math.min(Math.max(1, Number(page) || 1), total)

  const info = (
    <span
      data-slot='pager-info'
      // text-subtle（--text-2） not is muted-foreground（--text-3）： and 
      // is 1 ， use that in down will 
      className='text-[11.5px] text-subtle [font-variant-numeric:tabular-nums]'
    >
      {t('第 {current} / {total} 页', { current, total })}
    </span>
  )

  return (
    <div data-slot='pager' className={cn('flex items-center gap-2', className)}>
      {infoFirst && info}
      <Button
        type='button'
        variant='outline'
        size='sm'
        disabled={disabled || current <= 1}
        onClick={() => onPageChange(current - 1)}
      >
        {t('上一页')}
      </Button>
      <Button
        type='button'
        variant='outline'
        size='sm'
        disabled={disabled || current >= total}
        onClick={() => onPageChange(current + 1)}
      >
        {t('下一页')}
      </Button>
      {!infoFirst && info}
    </div>
  )
}

export { Pager, type PagerProps }
