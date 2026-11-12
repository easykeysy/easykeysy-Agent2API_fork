/**
 * 「」 up —— this **current in **：、 need etc 、
 * when and up to Reason， and 「」（） and 「」。
 *
 * ── Why is ， not is down 1 ────────────────────────
 * old is 「 → 1 」。 ：
 * ① by （ 1330px）， 1 ， that empty is 
 * 「」 by —— need 1 ；
 * ② Reason need and 、 when 、 by in 1 ， can ， need ；
 * ③ will ， when ， and 1 when can 1 。
 * and 「」 （ accounts-credit-detail.tsx） after ：
 * 、 in up ， use 「 need etc 」 and when down 。
 *
 * ── and 「」 use 1 and ───────────────────────
 * is `.usage-toggle`（ can 、 has 、caret for 、
 * aria-expanded）， is `.usage-pop-*`（ / / and for ）。
 * 、 is 1 ， use not 。
 *
 * ── when to ： for used for not ， for used for to no ─────────────
 * is for when （「 has 14 minutes」）， is for when （「 14:20 」）。
 * old has for when （` {soonest} `），「 need etc 」 use ；
 * （ / / Md） 1 ， for when not not 。
 * and up —— has 132px， not down ， down 。
 *
 * ── is ** when **， not is remaining ──────────────────
 * is 「 this 」（ = up ）， already has `at`
 * （`store_admin::mark_rate_limited` will ）。 this ** not use ** 
 * `.usage-pop-track` —— that 「remaining」（ has ）， to ，
 * use 1 will 。`at` （ old ） or and resetAt not when 
 * not （`progress: null`）， and 「 not not has 」 1 。
 */

import * as React from 'react'
import { Button, Meter, Popover, PopoverContent, PopoverTrigger } from '@ui'
import { t } from '../i18n'
import { activeLimits, formatResetText, RESET_UNKNOWN } from './accounts-domain'
import type { AccountRecord } from './accounts-shared'
import type { AccountCooldown } from '../../../ui-shared/account-upstream-state'

/** 1 （`limitRowsOf` ） */
export type LimitRow = {
  /** （ after to ，） */
  model: string
  /** ： for when ； when not to when is 「 when unknown」 */
  wait: string
  /** ： has when = warn， not to = muted */
  tone: 'warn' | 'muted'
  /** ：` 14:20 `； not to when as empty */
  reset: string
  /** ： up to Reason（ can can as empty） */
  reason: string
  /** （ for + for + Reason + this ） */
  title: string
  /** （0–100）； not to when when null = not */
  progress: number | null
}

/**
 * remainingms → when 。
 *
 * by 「 need 」： 1 when to minutes（ this when is need ），
 * 1 to when + minutes， 1 to （ that when minutes is ）。 use `Math.round`：
 * 「 has 1 minutes」「 has 0 minutes」 has ， after will already etc 30 seconds as this 。
 */
function waitText(remainingMs: number): string {
  const minutes = Math.round(remainingMs / 60000)
  if (minutes < 1) return t('不到 1 分钟')
  if (minutes < 60) return t('还有 {n} 分钟', { n: minutes })
  const hours = Math.floor(minutes / 60)
  const rest = minutes % 60
  if (hours < 24) {
    return rest ? t('还有 {h} 小时 {m} 分', { h: hours, m: rest }) : t('还有 {h} 小时', { h: hours })
  }
  return t('还有 {d} 天', { d: Math.floor(hours / 24) })
}

/**
 * **current** → 。
 *
 * `activeLimits` to （ by when ， before ）： before ，
 * and up that 「 … 」 is 1 ， not will for not up 。
 */
export function limitRowsOf(account: AccountRecord | null | undefined): LimitRow[] {
  const now = Date.now()
  return activeLimits(account).map(entry => {
    const resetAt = Number(entry.resetAt)
    const started = Number(entry.at)
    const reset = formatResetText(entry.resetAt)
    const unknown = reset === RESET_UNKNOWN
    const reason = entry.message || (entry.status ? t('上游返回 {status}', { status: entry.status }) : '')
    const span = resetAt - started
    const progress = started > 0 && Number.isFinite(span) && span > 0
      ? Math.max(0, Math.min(100, ((now - started) / span) * 100))
      : null
    const wait = unknown ? t('恢复时间未知') : waitText(resetAt - now)
    const resetLine = unknown ? '' : t('{time} 恢复', { time: reset })
    return {
      model: entry.model,
      wait,
      tone: unknown ? 'muted' : 'warn',
      reset: resetLine,
      reason,
      title: [
        entry.model,
        wait,
        resetLine,
        reason,
        t('到恢复时间后自动解除，无需手动操作'),
      ].filter(Boolean).join(' · '),
      progress,
    }
  })
}

/**
 * + 。
 *
 * can （`.usage-toggle` to to ）， down that 「 … 」
 * in **** —— is ， not this by 1 。
 *
 * `soonest` in when when ，otherwise 。
 */
export function LimitDetailButton({ accountName, rows, count, cooldown, soonest, onClear }: {
  accountName: string
  rows: LimitRow[]
  count: number
  cooldown: AccountCooldown | null
  soonest: string
  onClear: (model: string) => void
}) {
  const [open, setOpen] = React.useState(false)

  /**
   * in up not is 「 1 can 」， and 1 
   * （ by accounts-credit-detail.tsx： default as will to 1 can 、
   * ， 1 " 1 in in "）。 `tabIndex={-1}`。
   */
  const boxRef = React.useRef<HTMLDivElement | null>(null)
  const attachBox = React.useCallback((node: HTMLDivElement | null) => {
    boxRef.current = node
    if (node) node.scrollTop = 0
  }, [])
  const initialFocus = React.useCallback(() => boxRef.current ?? null, [])

  const earliest = soonest === RESET_UNKNOWN ? t('待定') : soonest

  return (
    <>
      <Popover open={open} onOpenChange={setOpen}>
        <PopoverTrigger
          render={<Button size='default' variant='ghost' type='button' className='usage-toggle warn'
            title={cooldown ? t('点击查看账号冷却与模型限流明细') : t('点击查看各模型的限流明细')} aria-label={t('限流明细')} />}
        >
          <span className='val'>{cooldown ? t('账号冷却') : t('{n} 个模型', { n: count })}</span>
          <span className='caret' aria-hidden>▾</span>
        </PopoverTrigger>
        {/* `align='center'`： and 「」 1 （ by accounts-credit-detail
             parameter）—— this by ，'end' will to 。 */}
        <PopoverContent align='center' sideOffset={6} className='w-[320px]' initialFocus={initialFocus}>

          <div className='usage-pop-limit' ref={attachBox} tabIndex={-1}>
            {accountName ? <div className='usage-pop-sub'>{accountName}</div> : null}
            <div className='usage-pop-group'>{t('限流明细')}</div>
            {cooldown ? (
              <div className='usage-pop-row'>
                <div className='usage-pop-line'>
                  <span className='usage-pop-name'>{t('账号冷却')}</span>
                  <span className='usage-pop-view warn'>{waitText(cooldown.resetAt - Date.now())}</span>
                </div>
                <div className='lp-pop-subline'>
                  <span className='lp-pop-reason'>
                    {t('{time} 恢复', { time: soonest })}{cooldown.message ? ` · ${cooldown.message}` : ''}
                  </span>
                </div>
              </div>
            ) : null}
            {rows.map(row => (
              <div className='usage-pop-row' key={row.model} title={row.title}>
                <div className='usage-pop-line'>
                  <span className='usage-pop-name lp-pop-model'>{row.model}</span>
                  <span className={`usage-pop-view ${row.tone}`}>{row.wait}</span>
                </div>
                <div className='lp-pop-subline'>
                  <span className='lp-pop-reason'>
                    {row.reset ? `${row.reset} · ` : ''}{row.reason}
                  </span>
                  <Button variant='ghost' size='xs'
                    title={t('清掉本机的限流标记，立刻重新尝试该模型（上游若仍在限流会再次被标记）')}
                    onClick={() => onClear(row.model)}>{t('清除')}</Button>
                </div>
                {row.progress !== null ? (
                  <Meter value={row.progress} variant='muted' className='mt-1.5' trackClassName='h-[3px]'
                    aria-label={t('冷却进度 {percent}%', { percent: Math.round(row.progress) })} />
                ) : null}
              </div>
            ))}
            <div className='lp-pop-foot'>
              <span className='lp-pop-note'>{cooldown
                ? t('账号全部模型暂时停止转发 · 到恢复时间自动解除')
                : t('{n} 个模型限流中 · 到恢复时间自动解除', { n: count })}</span>
              <Button variant='outline' size='xs' title={t('清除该账号的冷却与全部模型限流标记')}
                onClick={() => onClear('')}>{t('全部清除')}</Button>
            </div>
          </div>
        </PopoverContent>
      </Popover>
      <span className='lim-sub' title={cooldown ? t('账号冷却') : t('最早恢复')}>
        {cooldown ? t('{time} 恢复', { time: earliest }) : t('最早 {soonest} 恢复', { soonest: earliest })}
      </span>
    </>
  )
}
