/**
 * up 「」 —— （ / already use / remaining + → to ）。
 *
 * ── Why 1 ， not is this usageDetail ──────
 * `usageDetail` is ****（ + by + token ，CodeArts）：
 * enabled will in 。 that is 「 + + x / y」，
 * etc —— that is this 1 1 。****
 * 1 ， has 1 not ：、、、 is 。
 *
 * ── is （ and CodeArts that 1 ）────────────
 * not 「 + 1 by 」 that ： has ， is this 1 
 * in （ 1 ，）。 this CodeArts ——
 * ** is by **（`.usage-toggle`： and to 、 has 、
 * caret for not 、）， 1 ：
 * * `sum` （WorkBuddy that ）：`.val` in ， and CodeArts that ；
 * * `pool` （ZCode that ，`usagePool` ）： and 1 
 * in by ， for （ has ， in will and ）。
 *
 * ── （ after `credits[]`）──────────────────────────
 * 1 1 up ， for （ in ，
 * Trae：`trae::usage::credit_rows`；WorkBuddy：`billing::credit_packs_from`）：
 * ```text
 * name this 1 is （ up / ）
 * total/used/remain / already use / remaining（ = up no to ， not 0）
 * grantAt （） when ，ms
 * startAt （） when ，ms —— when when up that 1 
 * expireAt to when ，ms
 * unit and not when in （Trae vs ）
 * daily daily （ to for is 「 24 」 ）
 * grantReason WorkBuddy Reason（ / new …）， can none
 * snapshot / ， not and / to 
 * balance snapshot （ use use used）， when as null
 * ```
 * this not ： not that 1 。`null` and `0` in this 1 ——
 * `Number(null) === 0`， " no to " 0 will 「 0 / 0」 this to 
 * （ after module that ， before this is after 1 ）。
 *
 * ── when ──────────────────────────────────────────
 * up 1 `MM-DD HH:mm`（ need 1 ， is ）， when （） in 
 * title —— and `pendingPlansTitle` 1 。expired not to " to when "，
 * 「expired」 and ： 1 and 1 「expired」 is 1 ， after 。
 *
 * ── not in ， and default ──────────────────────────
 * can in before （ after to ： to in ），**remaining 0 and expired **
 * that 「 already use / expired」，default、（ in `CreditRow.spent`）。
 * up when this 1 、 is ， is " to 1 no use "。
 * ** from up **（ in up ， by `initialFocus`），
 * that when 1 new 1 （ that layout effect）。
 */

import * as React from 'react'
import { Meter, Popover, PopoverContent, PopoverTrigger, Button } from '@ui'
import { t } from '../i18n'
import { formatTime, numberText, type UsageEntry } from './accounts-shared'
import { creditPacksOf, snapshotReadingOf, usageUnitLabel, type UsageAggregate } from '../../../ui-shared/usage-readings'
import { creditExpiryLabel, creditExpirySummaryOf, creditExpiryText, creditExpiryUnitOf, expiringCreditOf, type ExpiringCredit } from '../../../ui-shared/credit-expiry'

/** 1 （`creditRowsOf` ； and `usageDetailOf` ） */
export type CreditRow = {
  name: string
  /** ：『 x / y []』； 1 not to when to 「—」 */
  view: string
  tone: 'ok' | 'warn' | 'bad' | 'muted'
  /** **remaining**（ is remaining， and its to ）；null = not */
  percent: number | null
  /** 「 … · to …」 that 1 （ no has null） */
  window: string | null
  /** （ when 、、Why no —— up not Reason） */
  title: string
  /**
   * this 1 can not can 。：**remaining to 0**（ or already use ≥ total），
   * ** to when already ** —— for 「 not 」 is 1 ，
   * that （ `CreditDetailButton`）。
   *
   * " use 、 its no " must in ：**daily** 
   * （`remain` in is 0），** not ** （`-1` ） not 。
   *
   * ** as false**（ {@link CreditRow.amount}）： 1 already no has 
   * " can not can " can ， 1 `-0.96` not is " use "。
   */
  spent: boolean
  /** / ， not 1 will or 。 */
  snapshot?: boolean
  /**
   * this 1 is ** 1 **（`record: true`） not is ： has when is 
   * （`+1000` / `-0.96`）， no has 「 / 」 that 、 not 。
   *
   * ** after to ** not is 「 has no has total」 this ： up to 1 
   * no has remaining ， and to 1 no ， in up can 1 ，
   * has that 1 is 1 。
   */
  amount: number | null
  expiry: ExpiringCredit | null
}

/**
 * → number or null。
 *
 * and `usagePool` `numberOrNull` 1 and not can ：`Number(null)` is 0，
 * will 「 up no to 」「 this is 0」。 after `credits[]` already not ，
 * this is 。
 */
function numberOrNull(value: unknown): number | null {
  if (value === null || value === undefined || value === '') return null
  const parsed = Number(value)
  return Number.isFinite(parsed) ? parsed : null
}

/** ms when → `MM-DD HH:mm`（ in when ； not to empty） */
function shortTime(value: number): string {
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return ''
  const pad = (part: number): string => String(part).padStart(2, '0')
  return `${pad(date.getMonth() + 1)}-${pad(date.getDate())} ${pad(date.getHours())}:${pad(date.getMinutes())}`
}

/**
 * → 。
 *
 * after `credits[].unit` is ****（ / ）， not is ； that 1 
 * （`usageSummary` 「available {value} {unit}」）， in down 1 。
 * this can 1 ，「」 
 * （ its —— not ， and `quota_meter_label`
 * 1 to ）。
 */
function unitLabel(unit: string): string {
  return usageUnitLabel(unit, t)
}

/**
 * → （ no has `credits` 、 or as empty → empty）。
 *
 * ** after to that **： in after （ Trae by to in before 、WorkBuddy
 * by + to ； and ）。 before 1 will 
 * 。
 */
export function creditRowsOf(entry: UsageEntry, now = Date.now()): CreditRow[] {
  const packs = creditPacksOf(entry)
  const defaultUnit = creditExpiryUnitOf(entry)
  return packs.map(pack => {
    const snapshot = snapshotReadingOf(pack, t, numberText)
    if (snapshot) {
      const window = snapshot.daily ? t('每日重置') : null
      return {
        name: snapshot.name,
        view: snapshot.view,
        tone: snapshot.tone,
        percent: null,
        window,
        title: [snapshot.name, snapshot.view, window].filter(Boolean).join(' · '),
        spent: false,
        snapshot: true,
        amount: null,
        expiry: null,
      }
    }
    const name = String(pack?.name || '').trim() || t('积分包')

    // ── 1 ： 1 （ CreditRow.amount）──────────────
    // is ** **， not is 「 x / y」： 1 already no has 
    // remaining can 。must —— for after ，「 1000」 and 「 1000」
    // in up 1 1 ， this 1 is 。
    if (pack?.record === true) {
      const amount = numberOrNull(pack?.amount)
      const unit = String(pack?.unit || '').trim()
      const at = numberOrNull(pack?.at)
      const expireAt = numberOrNull(pack?.expireAt)
      const value = amount === null ? '—' : `${amount > 0 ? '+' : ''}${numberText(amount)}`
      const view = unit && value !== '—' ? `${value} ${unitLabel(unit)}` : value
      const parts: string[] = []
      if (at !== null && at > 0) parts.push(shortTime(at))
      if (expireAt !== null && expireAt > 0) {
        parts.push(expireAt <= now ? t('已过期') : t('{time} 到期', { time: shortTime(expireAt) }))
      }
      const remark = String(pack?.remark || '').trim()
      return {
        name,
        view,
        // is this 1 ""， not this Note； not use 
        tone: amount !== null && amount < 0 ? 'muted' : 'ok',
        percent: null,
        window: parts.length ? parts.join(' · ') : null,
        title: [
          name,
          view,
          at !== null && at > 0 ? formatTime(at) : '',
          expireAt !== null && expireAt > 0
            ? t('到期 {time}', { time: formatTime(expireAt) })
            : '',
          remark,
        ].filter(Boolean).join(' · '),
        // not 「 already use / expired」 that （ CreditRow.spent）
        spent: false,
        amount,
        expiry: null,
      }
    }

    const unit = String(pack?.unit || '').trim()
    const total = numberOrNull(pack?.total)
    const used = numberOrNull(pack?.used)
    const remain = numberOrNull(pack?.remain)
    const unlimited = pack?.unlimited === true
    const grantAt = numberOrNull(pack?.grantAt)
    const startAt = numberOrNull(pack?.startAt)
    const expireAt = numberOrNull(pack?.expireAt)
    const daily = pack?.daily === true
    const rolling = pack?.rolling === true
    const resetAt = numberOrNull(pack?.resetAt)
    const declaredPercent = numberOrNull(pack?.remainingPercent)
    const reason = String(pack?.grantReason || '').trim()
    const expired = expireAt !== null && expireAt > 0 && expireAt <= now

    // ── ────────────────────────────────────────────────
    // 「 not 」 in not is 1 ：-1 that up will ""。
    // in （「 400 / 500 」）， not in in —— in that 1 
    // 「 400 / 500 」 ， and 。
    // `numberText`： up to is （`39.430000335` this ），
    // will （ accounts-shared that ）。
    const hasTotal = total !== null && total > 0
    const amount = unlimited
      ? t('不限')
      : remain !== null
        ? hasTotal
          ? t('剩 {remaining} / {total}', { remaining: numberText(remain), total: numberText(total) })
          : t('剩 {remaining}', { remaining: numberText(remain) })
        : used !== null
          ? t('已用 {used}', { used: numberText(used) })
          : declaredPercent !== null
            ? t('剩余 {percent}%', { percent: Math.round(declaredPercent) })
            : '—'
    const view = !unlimited && unit && amount !== '—' ? `${amount} ${unitLabel(unit)}` : amount
    const percent = hasTotal && remain !== null
      ? Math.max(0, Math.min(100, (remain / total) * 100))
      : declaredPercent === null ? null : Math.max(0, Math.min(100, declaredPercent))

    // ── can not can （ use ， CreditRow.spent）────────────────
    // 「 use 」 ：remaining to 0；remaining not to 「 already use ≥ total」 ——
    // up to used/total that 1 this ， not this as remain 。
    // not to when ** not **（ in ， not empty ）。
    const exhausted = (remain !== null && remain <= 0)
      || (remain === null && used !== null && hasTotal && used >= total)
    const spent = !unlimited && !daily && !rolling && (exhausted || expired)

    // ── → to ─────────────────────────────────────────
    // that 1 ，**must**：`grantAt`（ up when ）
    // 「」； `startAt`（） when 「」—— this not can ，
    // when 「」 etc up 1 to when 。
    const parts: string[] = []
    if (grantAt !== null && grantAt > 0) {
      parts.push(t('{time} 领取', { time: shortTime(grantAt) }))
    } else if (startAt !== null && startAt > 0) {
      parts.push(t('{time} 生效', { time: shortTime(startAt) }))
    }
    if (daily) {
      // 「 to 」 is 24 （）， not is this ——
      // 「 to 」 will as no （ is ）
      parts.push(t('每日重置'))
    } else if (rolling && resetAt !== null && resetAt > 0) {
      parts.push(t('重置 {date}', { date: shortTime(resetAt) }))
    } else if (expireAt !== null && expireAt > 0) {
      parts.push(expired ? t('已过期') : t('{time} 到期', { time: shortTime(expireAt) }))
    }
    if (reason) parts.push(reason)

    // ── and ────────────────────────────────────────
    const tone: CreditRow['tone'] = expired || remain === 0
      ? 'muted'
      : percent !== null && percent < 20
        ? 'warn'
        : remain === null && used === null && !unlimited
          ? 'muted'
          : 'ok'
    const title = [
      name,
      view,
      grantAt !== null && grantAt > 0
        ? t('领取 {time}', { time: formatTime(grantAt) })
        : startAt !== null && startAt > 0
          ? t('生效 {time}', { time: formatTime(startAt) })
          : '',
      expireAt !== null && expireAt > 0
        ? (expired
          ? t('已过期（{time}）', { time: formatTime(expireAt) })
          : t('到期 {time}', { time: formatTime(expireAt) }))
        : '',
      daily ? t('每日重置（额度按周期回满，不是作废）') : '',
      reason,
      percent === null && remain !== null ? t('上游未给总量，画不出剩余比例') : '',
      used !== null && total !== null ? t('已用 {used} / 共 {total}', { used: numberText(used), total: numberText(total) }) : '',
    ].filter(Boolean).join(' · ')
    return {
      name, view, tone, percent, window: parts.length ? parts.join(' · ') : null, title, spent, amount: null,
      expiry: expiringCreditOf(pack, defaultUnit, now),
    }
  })
}

/** 1 （ and CodeArts that 1 ， 1 ） */
function CreditRowView({ row, now }: { row: CreditRow; now: number }) {
  return (
    <div className='usage-pop-row' title={row.title}>
      <div className='usage-pop-line'>
        <span className='usage-pop-name'>{row.name}</span>
        {row.expiry ? <span className='usage-pop-soon-tag'>{creditExpiryLabel(row.expiry.expireAt, t, now)}</span> : null}
        <span className={`usage-pop-view ${row.tone}`}>{row.view}</span>
      </div>
      {row.window ? <div className='usage-pop-window'>{row.window}</div> : null}
      {row.percent !== null ? (
        <Meter value={row.percent} variant={row.tone === 'ok' ? 'success' : row.tone === 'warn' ? 'warning' : 'muted'}
          className='mt-1.5' trackClassName='h-[3px]' aria-label={t('剩余 {percent}%', { percent: Math.round(row.percent) })} />
      ) : null}
    </div>
  )
}

/**
 * ： can ， is this （ and CodeArts that ）。
 *
 * `variant` this 1 （file）：`sum` is in 
 * （`.usage-toggle` + `.val`， and CodeArts ），`pool` is 「 + 」
 * （ 1 `.usage-toggle` `.usage-pool-line` and for that ）。
 * `children` is that DOM —— by use （`UsageCell`），
 * this " by "。
 *
 * `subscriptionLine` by use use `subscriptionText` （ / to /
 * that 1 ）： that is has ， 1 will 。
 */
export function CreditDetailButton({ rows, now, expiryUnit, subscriptionLine, aggregate, variant, kind, title, children }: {
  rows: CreditRow[]
  now: number
  expiryUnit: string
  subscriptionLine: string
  aggregate: UsageAggregate | null
  variant: 'sum' | 'pool'
  /** （ok / warn / …）：`.usage-toggle` by up ， and 1 */
  kind: string
  /** （ that need ， and no when ） */
  title: string
  children: React.ReactNode
}) {
  const [open, setOpen] = React.useState(false)
  /**
   * that 「 already use / expired」 ，**default**： up when this 1 can has 
   * ， is not ， in up will can to not —— 1 this to 
   * is " can use "。 this （ is ）：
   * 1 in new when need 1 ， will 。
   */
  const [spentOpen, setSpentOpen] = React.useState(false)
  const liveRows = rows.filter(row => !row.spent)
  const spentRows = rows.filter(row => row.spent)
  const detailLabel = rows.length > 0 && rows.every(row => row.snapshot) ? t('余额详情') : t('积分明细')
  const soonRows = liveRows.filter(row => row.expiry !== null)
  const restRows = liveRows.filter(row => row.expiry === null)
  const expiring = creditExpirySummaryOf(soonRows.flatMap(row => row.expiry ? [row.expiry] : []), now)
  const expiryText = creditExpiryText(expiring, t, numberText, expiryUnit)
  const className = variant === 'pool' ? `usage-toggle usage-pool-line ${kind}` : `usage-toggle ${kind}`

  /**
   * （ when 、， down ）。
   *
   * use ** ref** not is useRef + useEffect： need in " 
   * " before down ， ref is DOM that 1 ，。
   */
  const boxRef = React.useRef<HTMLDivElement | null>(null)
  const attachBox = React.useCallback((node: HTMLDivElement | null) => {
    boxRef.current = node
    // from up 。 is new ， this is 0 —— is as 
    // this ： from up down （ can in before ）， not this up 。
    if (node) node.scrollTop = 0
  }, [])

  /**
   * in **** up ， not is " 1 can "。
   *
   * default as is after （ PopoverPopup `initialFocus` ）， this 1 
   * can is that 「 already use / expired」 —— in 。 is 
   * as ， will to ： use " 1 
   * in 、 need up " is this （：，
   * scrollTop is 59/236 this " that by " ， not is 0）。
   *
   * in up no ： can ， not to ； use by Tab 
   * （ down 1 can is that ）。 tabIndex={-1}。
   */
  const initialFocus = React.useCallback(() => boxRef.current ?? null, [])

  /**
   * after new 1 。
   *
   * that when in ****， after new in down ——
   * not this 1 down ， can 1 ， no 。
   * `block: 'nearest'` required that ， not 。
   *
   * must in layout effect （ not is onClick rAF ）：React 19 is ，
   * rAF in new no DOM when ，`querySelector` not to （ 1 is this 
   * ）。 is ** 1 new ** not is ： when ， for use 
   * 'nearest' will **** for to （），" 1 in "。
   */
  React.useLayoutEffect(() => {
    if (!spentOpen) return
    const first = boxRef.current?.querySelector('.usage-pop-spent .usage-pop-row')
    if (first instanceof HTMLElement) first.scrollIntoView({ block: 'nearest' })
  }, [spentOpen])

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger
        render={<Button size='default' variant='ghost' type='button' className={className} title={title}
          aria-label={detailLabel} />}
      >
        {variant === 'pool' ? children : <span className='val'>{children}</span>}
        {variant === 'sum' && expiryText.badge ? <span className='usage-toggle-soon'>{expiryText.badge}</span> : null}
        <span className='caret' aria-hidden>▾</span>
      </PopoverTrigger>
      {/* `align='center'`： by as in 。 is 'end'（ for ），
           this 1 by ， to to ，
           in 「」 down 。 in after in up ， and 1 is 1 for up 。 */}
      <PopoverContent align='center' sideOffset={6} className='w-[320px]' initialFocus={initialFocus}>

        <div className='usage-pop-credit' ref={attachBox} tabIndex={-1}>
          {subscriptionLine ? <div className='usage-pop-sub'>{subscriptionLine}</div> : null}
          {aggregate ? <div className='usage-pop-total'>
            <span className='k'>{aggregate.label}</span>
            <span className='v'>{aggregate.unlimited ? t('不限') : numberText(aggregate.remaining)}
              {!aggregate.unlimited && aggregate.unit ? <> <small>{aggregate.unit}</small></> : null}
            </span>
          </div> : null}
          <div className='usage-pop-group'>{detailLabel}</div>
          {expiring.count > 0 ? (
            <>
              <div className='usage-pop-soon'>
                <span>{expiryText.summary}</span>
                {expiryText.today ? <span>{expiryText.today}</span> : null}
              </div>
              {soonRows.map((row, index) => <CreditRowView key={`soon-${row.name}-${index}`} row={row} now={now} />)}
            </>
          ) : null}
          {restRows.map((row, index) => <CreditRowView key={`${row.name}-${index}`} row={row} now={now} />)}
          {spentRows.length > 0 ? (
            <div className='usage-pop-spent'>
              <Button size='default' variant='ghost'
                type='button'
                className='usage-pop-spent-head'
                aria-expanded={spentOpen}
                title={t('展开或收起（默认收起）')}
                onClick={() => setSpentOpen(value => !value)}
              >
                {/* ⚠️ this span must**empty**： by CSS border 
                    （ page-accounts-table.css .usage-pop-spent-head .caret），
                     1 ▾ will —— 1 border 、
                     1 （width/height is 0，）。 */}
                <span className='caret' aria-hidden />
                <span>{t('已用完 / 已过期')}</span>
                <span className='n'>{t('{count} 份', { count: spentRows.length })}</span>
              </Button>
              {spentOpen
                ? spentRows.map((row, index) => (
                  <CreditRowView key={`spent-${row.name}-${index}`} row={row} now={now} />
                ))
                : null}
            </div>
          ) : null}
        </div>
      </PopoverContent>
    </Popover>
  )
}
