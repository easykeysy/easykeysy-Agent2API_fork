/**
 * ** and **（ ui/accounts-table.js + ui/accounts-model.js
 * ⋯ / HTML + ui/accounts-view.js ）。
 *
 * ── Why legacy ─────────────────────────────────
 * 1 has page-accounts-table.css and （、
 * `.acct-actions` by …）。****，：
 * （Button / Badge / Switch / Checkbox / Select / Popover），
 * `.usage-sum`（， not can ） and `.order-cell`（ + ）
 * this no has for ， —— 「」。
 *
 * ── （2026-10） ────────────────────────────────
 * · `PriorityStepper`（ + ）： by ，
 * 「↑ = 」 that to 1 down —— in has 「 to 1 」
 * 1 （ in up / down ， by accounts-queue-drag ）。
 * · `ProviderCell`（ by provider up ）： and to 
 * （`GroupBandRow`）， not 。`.pbadge` this 1 in CSS ——
 * in use （report-page.tsx that 1 ）。
 */

import * as React from 'react'
import {
  Badge,
  BadgeDot,
  Meter,
  Button,
  Checkbox,
  Popover,
  PopoverContent,
  PopoverTrigger,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Switch,
  Tooltip,
  TooltipArrow,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
  cn,
  TableRow, TableCell, TableHeader, TableHead as UiTableHead } from '@ui'
import { formatTime, numberText, poolItemLabel, POOL_VALUE_PREFIX, shared, type AccountRecord, type LimiterRule, type TokenReading, type UsageEntry } from './accounts-shared'
import {
  accountTags, activeLimits, balanceBlockedOf,
  displayNameOf, expiryMillis, formatIntervalSeconds, formatResetText,
  formatTokenCount, identifierKeyOf, identifierOf, isDesktopAccount, isEnabled, isFreePlan, planBadgeLabel,
  providerFeatures,
  providerOf, supportsCreditDetail, supportsUsage, supportsUsageDetail,
  tokenBlockedOf, tokenCountdownText, tokenDisableTriggeredOf, tokenExpiryOf, tokenReadingForRule,
  tokenRulesOf, tokenWindowInfo, zcodePlanStateLabel,
} from './accounts-domain'
import { CreditDetailButton, creditRowsOf } from './accounts-credit-detail'
import { creditExpirySummaryOf, creditExpiryText, creditExpiryUnitOf, expiringCreditsOf, type CreditExpirySummary, type ExpiringCredit } from '../../../ui-shared/credit-expiry'
import { useDisplayClock } from '../../../ui-shared/use-display-clock'
import { UsageWindowsCell, usageWindows } from './accounts-usage-windows'
import { accountReloginText } from './accounts-authorization'
import { credentialExpiryRowsOf, credentialExpiryText, type CredentialExpiryRow } from '../../../ui-shared/credential-expiry'
import { creditStatisticsWalletsOf, isSnapshotWallet, snapshotReadingsOf, usageAggregateOf, usageSupplementReadingsOf, usageUnitLabel, usageWarningsOf, usageWindowLabel, usageWindowView } from '../../../ui-shared/usage-readings'
import { LimitDetailButton, limitRowsOf } from './accounts-limit-detail'
import { upstreamCooldownOf, upstreamSuspendedReasonOf } from '../../../ui-shared/account-upstream-state'
import { LimiterStatus } from './accounts-limiter-status'
import { limiterHit } from './accounts-limiter-model'
import {
  PROXY_CUSTOM_CURRENT, PROXY_CUSTOM_EDIT, applyProxyPick,
  connectionsOf, getStore, groupSortRules, maskName, openBatchDialog, openGroupDialog, openSettingsDialog,
  poolError, proxyPoolSnapshot, queryUsageOnce, setAccountEnabled, setGroupEnabled,
  tokenUsageOf, toggleGroupCollapsed, toggleNamesHidden, usageEntryOf, usageFailureOf, usageFailureStatusOf, usageUpdatedAtOf,
} from './accounts-data'
import type { AccountGroup } from './accounts-domain'
import { activeGroupSortRules, groupSortSummary, manualOrderBlockedMessage } from './accounts-group-settings'
import { exportAccountFiles } from './accounts-export'
import { PROVIDER_ICONS } from './add-provider-pick'
import { t } from '../i18n'
import { Empty } from '@ui'

/** （icons.js SVG ）： use 1 ， this */
function iconHtml(name: string, size: number): string {
  return shared().wbIcons?.icon?.(name, size) || ''
}

/* ─── （）──────────────────────────── */

/**
 * ：`⠿ #N` —— is ， is ****。
 *
 * ── Why not has and ↑↓ ────────────────────────────────
 * by （、 this 1 ）， not is use need ：
 * `priority` by 「 → 」（ after `reorder_accounts` ），
 * up 1 can ， etc to 1 。 old （ + 
 * ） down ，`commitPriority` / `moveAccount` not use 。
 *
 * is ** 1 ** （ not is can ）： has 、 down 、 by ， can will 
 * 「 1 down 」「 1 」。 is in **** ， and 
 * after by —— is 。
 *
 * ── `sortActive`： this 1 then when ────────────────
 * when by then ， no has （ after 1 will by then ）。
 * 、， to 1 （ by accounts-queue-drag onLockedAttempt
 * 1 ）—— not response will as 。
 */
export function OrderCell({ account, rank, sortActive }: {
  account: AccountRecord
  rank: number
  sortActive?: boolean
}) {
  const label = t('组内第 {rank} 位', { rank })
  return (
    <div className='order-cell'>
      <span className={`order-grip${sortActive ? ' locked' : ''}`} role='button' tabIndex={-1}
        aria-label={t('拖动调整组内顺序')}
        aria-disabled={sortActive || undefined}
        title={sortActive
          ? manualOrderBlockedMessage(activeGroupSortRules(providerOf(account), groupSortRules(providerOf(account))))
          : t('按住拖动，调整这个账号在本组内的顺序')}
        dangerouslySetInnerHTML={{ __html: iconHtml('gripVertical', 14) }} />
      <span className='order-rank' title={label}>{`#${rank}`}</span>
    </div>
  )
}

/* ─── ────────────────────────────────── */

/**
 * ⋯ ：enabled / use 、、。
 *
 * 「enabled / use 」（ 1 1 ， and 1 ）；
 * 「」 has that 1 （、set and limit this need parameter 
 * use ）—— parameter need in to 1 ，** this 1 **，
 * not as parameter in 1 ； that 1 and that 1 ——
 * ** is **，mustcurrent： already that 1 ， this 1 
 * 「 this 1 」 is 。
 *
 * this 1 ** not **（「 …」 ）： its not ，
 * after empty， up that 1 … will 「 」。
 */
function GroupMenu({ group, ids, collapsed }: { group: AccountGroup; ids: string[]; collapsed: boolean }) {
  const [open, setOpen] = React.useState(false)
  const itemClass = 'w-full justify-start px-2 font-normal'
  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger render={<Button size='default' variant='ghost' type='button' className='grp-iconbtn' title={t('本组操作')} />}>⋯</PopoverTrigger>
      <PopoverContent align='end' sideOffset={4} className='w-[196px] p-1.5'>
        <div className='flex flex-col gap-0.5'>
          <Button variant='ghost' size='sm' className={itemClass}
            title={t('启用这一组里的全部账号（含被筛选隐藏的）')}
            onClick={() => { setOpen(false); void setGroupEnabled(ids, true) }}>
            {t('全部启用（{n}）', { n: ids.length })}
          </Button>
          <Button variant='ghost' size='sm' className={itemClass}
            title={t('禁用这一组里的全部账号（含被筛选隐藏的）')}
            onClick={() => { setOpen(false); void setGroupEnabled(ids, false) }}>
            {t('全部禁用（{n}）', { n: ids.length })}
          </Button>
          <Button variant='ghost' size='sm' className={itemClass}
            title={t('打开批量操作弹窗，并预选这一组的账号（改代理等动作在弹窗里挑参数）')}
            onClick={() => { setOpen(false); openBatchDialog(ids, 'proxy') }}>
            {t('批量操作本组')}
          </Button>
          <Button variant='ghost' size='sm' className={itemClass}
            title={t('导出本组全部账号，每个账号一个 JSON 文件（含被筛选隐藏的）')}
            onClick={() => { setOpen(false); void exportAccountFiles({ provider: group.provider }) }}>
            {t('导出本组账号')}
          </Button>
          <div className='my-1 h-px bg-hairline' />
          <Button variant='ghost' size='sm' className={itemClass}
            onClick={() => { setOpen(false); toggleGroupCollapsed(group.provider) }}>
            {collapsed ? t('展开这一组') : t('折叠这一组')}
          </Button>
        </div>
      </PopoverContent>
    </Popover>
  )
}

/* ─── （available / available）────────────── */

/**
 * in （`groupBalanceOf` ）。
 *
 * by in tooltip：`counted` is to ， its 
 * （ no / failure / no has available） is 「Why no 」 —— 1 
 * is 「 ， its 1 」。
 *
 * ： for （ / / ） `sum`；
 * `lowest` / `ratioWindows` —— not can ， `groupBalanceOf` 
 * 「 not can 」 1 。
 */
type GroupBalance = {
  /** to （ for or ） */
  counted: number
  /** and statistics （ = ； by = can that ） */
  total: number
  /** for ； 1 not when null（ not use ） */
  sum: number | null
  /** not （>0 when ∞） */
  unlimited: number
  /** （ 1 ；empty = no to ） */
  unit: string
  /** failure （「not configured」 is in ， not failure） */
  failed: number
  /** to 、 no has for no has available */
  uncountable: number
  /** no to （ / in ） */
  pending: number
  /** failure（ has 1 no ）：， need ⚠ */
  partial: number
  /** statistics：（default） is current after can —— tooltip need */
  scope: 'group' | 'filtered'
  /** is can ： for can ， not can */
  additive: boolean
  /** not can when （ + ；label as empty = ） */
  lowest: GroupRatioReading | null
  /** not can when （tooltip ； by ） */
  ratioWindows: GroupRatioReading[]
  expiring: CreditExpirySummary
}

/** 1 ： + remaining（0–100） */
type GroupRatioReading = {
  /** （ before ， `Claude · 5 when `）；empty = */
  label: string
  /** remaining */
  percent: number
}

/** ： this is 「 1 」， not is can */
function isRatioUnit(unit: string): boolean {
  return unit.trim() === '%'
}

/**
 * ： this 1 available and （ for ） or remaining（）。
 *
 * ── （set → → ）──────────────
 * · ****（default）： this 1 ****， not / / 。
 * is 「 this 1 has 」， after no ， this 
 * is 1 —— need is ， no has 。
 * · ** by **（ use ）：current can ， and 。
 * total will ： 1 100， is 1,000（ 1 900
 * ）—— this to 100。
 *
 * ** not ** and this ： is 「 / 」， is 1 
 * not ， 「current and 」 will 1 down 1 。
 *
 * ── Why use ────────────────────────────────────
 * is **** not is ： use not and ， in up 。enabled ，
 * will and that 「enabled 10 · use 0」。： by 
 * set 「statisticscurrent 」， this not 1 enabled / use 。
 *
 * ── 「」──────────────────────────────────────────
 * to 1 **available**：`totalLeft`（WorkBuddy has ）、`available`
 * （ its ）， or 1 （Antigravity not `available`）。
 * down not ， by （tooltip ）：
 * · no / in （`undefined` / `null`）；
 * · failure（`usageFailureOf` ， and ）；
 * · no has for ， no has available。
 * not not ： 1 up not will ， by 1 ，
 * its 「 no 」—— 1 is 。
 *
 * ── not can ───────────────────────────────────────
 * `available` not is 「」：OpenCode Go / SenseNova to is remaining，
 * Antigravity to `windows[].remainingPercent`。 and no has （94% + 2%
 * not is 96% ， 90% not this is 270%）， this not and ，**
 * remaining**： is 「 this 1 that 」。 has `windows[]`
 * when （tooltip ）， no has when 。
 */
function groupBalanceOf(members: AccountRecord[], scope: 'group' | 'filtered' = 'group', now = Date.now()): GroupBalance {
  const out: GroupBalance = {
    counted: 0, total: members.length, sum: null, unlimited: 0, unit: '',
    failed: 0, uncountable: 0, pending: 0, partial: 0, scope,
    additive: true, lowest: null, ratioWindows: [], expiring: creditExpirySummaryOf([], now),
  }
  const expiring: ExpiringCredit[] = []
  let sum = 0
  /** this 1 ： 1 valid， after not 1 「 no 」 */
  let mode: 'additive' | 'ratio' | null = null
  /** ： is （ before ）， is this remaining */
  const windowMin = new Map<string, GroupRatioReading>()
  for (const account of members) {
    const entry = usageEntryOf(account)
    if (entry === undefined || entry === null) { out.pending++; continue }
    const failure = usageFailureOf(entry)
    if (failure) {
      // 「not configured」 in is in （ not is ）， this not ⚠
      if (!failure.notConfigured) out.failed++
      continue
    }
    if (typeof entry !== 'object' || Array.isArray(entry)) { out.uncountable++; continue }
    const data = entry as Record<string, unknown>
    if (data.statisticsError || usageWarningsOf(data).length > 0) out.partial++
    if (supportsCreditDetail(account)) expiring.push(...expiringCreditsOf(entry, now))
    // not ： this not ， can is ∞（`totalLeft` is ）
    if (data.unlimited === true) { out.unlimited++; continue }
    const hasTotalLeft = Object.prototype.hasOwnProperty.call(data, 'totalLeft')
    const raw = hasTotalLeft ? data.totalLeft : data.available
    // null / undefined must down ：`Number(null)` is 0， will 「 no to 」
    // 0（ and accounts-domain balanceBlockedOf 1 ）
    const rawNumber = raw === null || raw === undefined || raw === ''
      ? null
      : Number(raw)
    const rawValue = rawNumber !== null && Number.isFinite(rawNumber) ? rawNumber : null
    const windows = usageWindows(entry).filter(row => row.percent !== null)
    const unit = String(data.unit || t('积分')).trim()
    // ： for can ； / has not can 。
    // 「 has 」 —— Antigravity available is not ，
    // available will 「 no has for 」 。
    const entryMode: 'additive' | 'ratio' | null =
      rawValue === null && windows.length === 0
        ? null
        : isRatioUnit(unit) || rawValue === null ? 'ratio' : 'additive'
    if (entryMode === null) { out.uncountable++; continue }
    if (mode === null) mode = entryMode
    // 1 must 1 ： for and in 1 when not 
    if (mode !== entryMode) { out.uncountable++; continue }
    if (entryMode === 'additive' && out.unit && unit !== out.unit) { out.uncountable++; continue }
    if (!out.unit) out.unit = unit
    out.counted++
    if (entryMode === 'additive') {
      if (rawValue !== null) sum += rawValue
      continue
    }
    if (windows.length) {
      for (const row of windows) {
        const percent = row.percent
        if (percent === null) continue
        const label = row.group ? `${row.group} · ${row.window}` : row.window
        const prev = windowMin.get(label)
        if (!prev || percent < prev.percent) windowMin.set(label, { label, percent })
        if (!out.lowest || percent < out.lowest.percent) out.lowest = { label, percent }
      }
    } else if (rawValue !== null) {
      if (!out.lowest || rawValue < out.lowest.percent) out.lowest = { label: '', percent: rawValue }
    }
  }
  out.additive = mode !== 'ratio'
  out.ratioWindows = [...windowMin.values()]
  out.expiring = creditExpirySummaryOf(expiring, now)
  if (out.additive && out.counted) out.sum = Number(sum.toFixed(2))
  return out
}

/** ： 1 to （）， up by / （ or k / M）
 * —— and units.js 「 1 not use 」 1 ，ZCode that not */
function balanceAmountText(value: number): string {
  const api = (window as unknown as { wbUnits?: UnitsBridge }).wbUnits
  if (value < 10_000) {
    return Number.isInteger(value) && api?.formatInt ? api.formatInt(value) : numberText(value)
  }
  return api?.formatTokens ? api.formatTokens(value) : numberText(value)
}

/**
 * ：（ and / ， / by ），
 * and no 。
 */
function groupBalanceTitle(balance: GroupBalance): string {
  const parts = [
    balance.additive
      ? balance.scope === 'filtered'
        ? t('当前筛选下 {n} 个账号的可用余额之和（随筛选、搜索变化，不受分页影响）', { n: balance.total })
        : t('本组 {n} 个账号的可用余额之和，按整组统计（不受筛选、搜索影响）', { n: balance.total })
      : balance.scope === 'filtered'
        ? t('当前筛选下 {n} 个账号的百分比读数不能相加，取最低的剩余比例（随筛选、搜索变化，不受分页影响）', { n: balance.total })
        : t('本组 {n} 个账号的百分比读数不能相加，取组内最低的剩余比例（按整组统计，不受筛选、搜索影响）', { n: balance.total }),
  ]
  parts.push(t('{n} 个已计入', { n: balance.counted }))
  if (balance.unlimited) parts.push(t('含 {n} 个不限额度账号，合计记为 ∞', { n: balance.unlimited }))
  if (!balance.additive && balance.ratioWindows.length) {
    parts.push(t('逐窗口最低：{list}', {
      list: balance.ratioWindows.map(row => `${row.label} ${numberText(row.percent)}%`).join(' · '),
    }))
  }
  if (balance.pending) parts.push(t('{n} 个还没查到（未查询或正在查询）', { n: balance.pending }))
  if (balance.failed) parts.push(t('{n} 个查询失败', { n: balance.failed }))
  if (balance.uncountable) {
    parts.push(balance.additive
      ? t('{n} 个读数没有绝对数（只给比例或无额度），未计入', { n: balance.uncountable })
      : t('{n} 个读数里没有可用比例，未计入', { n: balance.uncountable }))
  }
  if (balance.partial) parts.push(t('{n} 个读数只读到一半（读数可信但不完整）', { n: balance.partial }))
  const expiryText = creditExpiryText(balance.expiring, t, balanceAmountText, balance.unit === t('积分') ? '积分' : balance.unit)
  if (expiryText.detail) parts.push(expiryText.detail)
  return parts.join('；')
}

/**
 * up ：`available 1,347 · 8/10 ⚠`；
 * as `available 2% · 2/2`（ `groupBalanceOf` 
 * 「 not can 」 1 ）。
 *
 * and `.usage-sum` for ： = to ， = to has no /
 * to 1 / already 20%（ and 1 ）， = 1 
 * no to （`—`）。`—` not is 0 —— 「 no to 」 and 「 use 」 in this 1 is ，
 * and 1 。
 */
function GroupBalanceBadge({ balance }: { balance: GroupBalance }) {
  const expiryText = creditExpiryText(balance.expiring, t, balanceAmountText, balance.unit === t('积分') ? '积分' : balance.unit)
  const warned = balance.failed + balance.partial > 0
  const empty = balance.additive
    ? balance.sum === null && balance.unlimited === 0
    : balance.lowest === null && balance.unlimited === 0
  // no has 「」，： 20% and 1 ，
  // 1 
  const low = !balance.additive && balance.lowest !== null && balance.lowest.percent < 20
  // has failure when ： 1 no to + has failure，「 this no to 」
  // （⚠ and ）， not is 「 this no has 」 that in 
  const tone = warned || low ? 'warn' : empty ? 'muted' : 'ok'
  const amount = balance.unlimited > 0
    ? '∞'
    : balance.sum === null ? '—' : balanceAmountText(balance.sum)
  // in not is 「」 when ： is ， is 
  const unit = balance.additive && balance.unit && balance.unit !== '积分' && balance.unit !== t('积分')
    ? usageUnitLabel(balance.unit, t)
    : ''
  // has ∞ when not 「 x/y」： that will as ∞ is 
  const note = balance.unlimited === 0 && balance.counted < balance.total
    ? t('计入 {counted}/{total}', { counted: balance.counted, total: balance.total })
    : ''
  return (
    <span className={`grp-balance ${tone}`} title={groupBalanceTitle(balance)}>
      <span className='k'>{balance.additive ? t('可用合计') : t('可用最低')}</span>
      {balance.additive ? (
        <>
          <b className='v'>{amount}</b>
          {unit ? <span className='u'>{unit}</span> : null}
        </>
      ) : balance.unlimited > 0 ? (
        <b className='v'>∞</b>
      ) : balance.lowest ? (
        <>
          {balance.lowest.label ? <span className='k'>{balance.lowest.label}</span> : null}
          <b className='v'>{numberText(balance.lowest.percent)}%</b>
        </>
      ) : (
        <b className='v'>—</b>
      )}
      {note ? <span className='n'>{note}</span> : null}
      {expiryText.group ? <span className='n soon' title={expiryText.detail}>{expiryText.group}</span> : null}
      {warned ? <span className='warn-mark' aria-hidden>⚠</span> : null}
    </span>
  )
}

/**
 * ： 1 （`<tbody>` 1 ， 1 ）。
 *
 * from to ： （ 1 ）→ → + → statistics
 * （N · enabled x · use y · z）→ → and ⋯ 。
 * statistics is ****（「 this 1 has 」 not ）； by set
 * （ `groupBalanceOf`）：default， after current can 。
 *
 * in （surface + 1 hairline）： 1 1 will ，
 * 「 1 is 1 」 by and already 。
 */
export function GroupBandRow({ group, collapsed, colSpan }: {
  group: AccountGroup
  collapsed: boolean
  colSpan: number
}) {
  const now = useDisplayClock()
  const icon = PROVIDER_ICONS[group.provider]
  // ：default（`members`），set「 by 」 after use can （`accounts`）。
  // not and —— groupBalanceOf 。
  const filtered = getStore().groupTotalFiltered
  const members = filtered ? group.accounts : group.members
  // 「 not 」 is **** has （ 1 ， and down that 
  // ）；**** is current。 not can 1 ： by down 
  // 1 1 when ， is use need that （100）， not can as 「 can 1 」。
  const worthSumming = group.members.length > 1 && members.length > 0
  const balance = worthSumming && members.some(supportsUsage)
    ? groupBalanceOf(members, filtered ? 'filtered' : 'group', now)
    : null
  /**
   * can 。
   *
   * 「 this 1 down not 」 to ： in by / / / 
   * up has ， not can 。 in up not is up ，
   * is as has 、statistics、 this —— use when is 「」。
   */
  const onBandClick = (event: React.MouseEvent<HTMLTableCellElement>) => {
    if ((event.target as Element | null)?.closest?.('button, a, input, select, label, .queue-grip')) return
    // when （ in ） not ： that 1 down is not is 
    if (String(window.getSelection?.() || '').length > 0) return
    toggleGroupCollapsed(group.provider)
  }
  const sortSummary = groupSortSummary(groupSortRules(group.provider))
  return (
    <TableRow className='grp-band' data-provider={group.provider}>
      <TableCell colSpan={colSpan} onClick={onBandClick}>
        <div className='grp-band-inner'>
          <span className='queue-grip' role='button' tabIndex={-1} aria-label={t('拖动调整整组顺序')}
            title={t('按住拖动，调整这一家的组顺位（整组一起动）')}
            dangerouslySetInnerHTML={{ __html: iconHtml('gripVertical', 14) }} />
          <span className='grp-rank' title={t('组顺位：越小越先用（拖动分组带调整）')}>
            {t('第 {rank} 顺位', { rank: group.rank })}
          </span>
          <span className='grp-logo'>
            {icon
              ? <img src={icon} alt='' loading='lazy' />
              : <span className='grp-logo-txt'>{String(group.label || '?').trim().slice(0, 1).toUpperCase()}</span>}
          </span>
          <span className='grp-name'>{t(group.label)}</span>
          <span className='grp-stat'>{t('{total} 个账号 · 启用 {enabled} · 禁用 {disabled}{limited}', {
            total: group.members.length,
            enabled: group.enabled,
            disabled: group.disabled,
            limited: group.limited > 0 ? t(' · 限流 {n}', { n: group.limited }) : '',
          })}</span>
          {balance ? <GroupBalanceBadge balance={balance} /> : null}
          {sortSummary ? (
            <Button size='default' variant='ghost' type='button' className='grp-sort-mark'
              title={t('当前排序规则：{rules}。点击调整', { rules: sortSummary })}
              onClick={() => openGroupDialog(group.provider)}>
              <span dangerouslySetInnerHTML={{ __html: iconHtml('sort', 10) }} />
              {sortSummary}
            </Button>
          ) : null}
          <span className='grp-spacer' />
          <Button size='default' variant='ghost' type='button' className='grp-iconbtn grp-fold' aria-expanded={!collapsed}
            title={collapsed ? t('展开这一组') : t('折叠这一组')}
            onClick={() => toggleGroupCollapsed(group.provider)}
            dangerouslySetInnerHTML={{ __html: iconHtml('chevronDown', 14) }} />
          <Button size='default' variant='ghost' type='button' className='grp-iconbtn' title={t('分组设置')}
            aria-label={t('分组设置')}
            onClick={() => openGroupDialog(group.provider)}
            dangerouslySetInnerHTML={{ __html: iconHtml('settings', 14) }} />
          <GroupMenu group={group} ids={group.accounts.map(account => account.id)} collapsed={collapsed} />
        </div>
      </TableCell>
    </TableRow>
  )
}

/* ─── ────────────────────────────────── */

/**
 * ：enabled / use （+ need need when ）。
 * `PATCH { enabled }`， not —— this can ， and after in 
 * 。 in need need when （ 1 when accountTags returnempty）—— enabled by 
 * and ， 1 「enabled」 is 1 。
 */
export function StatusCell({ account }: { account: AccountRecord }) {
  const enabled = isEnabled(account)
  const tags = accountTags(account)
  // Token use in and already use ： 1 「Why is 」
  // —— use not 「Why」 ，current is （ and after 
  // use ， tokenDisableTriggeredOf）。
  if (!enabled && tokenDisableTriggeredOf(account, tokenUsageOf(account))) {
    tags.push({
      text: t('Token 限额 · 已禁用'),
      kind: 'bad' as const,
      title: t('本周期 Token 消耗达到限制器的上限，账号已被自动禁用（窗口重置也不恢复，需手动启用）'),
    })
  }
  const who = displayNameOf(account) || account.id
  return (
    <>
      <Switch checked={enabled} aria-label={t('{action}{who}', { action: enabled ? t('禁用') : t('启用'), who })}
        title={enabled ? t('已启用，点击禁用（不参与转发）') : t('已禁用，点击启用')}
        onCheckedChange={next => void setAccountEnabled(account.id, next)} />
      {tags.length ? (
        <div className='status-tags'>
          {tags.map(tag => (
            <Badge key={tag.text} shape='tag' variant={tag.kind === 'bad' ? 'destructive' : 'secondary'}
              title={tag.title}>{tag.text}</Badge>
          ))}
        </div>
      ) : null}
    </>
  )
}

/* ─── ────────────────────────────────── */

/**
 * ： this **current in **。 in after by 「 × 」， use ——
 * this 1 for ， not need need 「 」 。
 * has when is 1 can （ down ， accounts-limit-detail），
 * when is 「」。
 *
 * 「 already 」 limit （isLimitedNow ）， this 1 must：
 * no has limit down not ， to 1 「 not / Token 」
 * as already （ by ， this not ）。
 *
 * is 「 has no has in 」， not as 1 ——
 * （429 / / up Retry-After） is ， this 1 。
 */
export function LimitsCell({ account, onClear }: {
  account: AccountRecord
  onClear: (model: string) => void
}) {
  const entries = activeLimits(account)
  const cooldown = upstreamCooldownOf(account)
  const suspendedReason = upstreamSuspendedReasonOf(account)
  if (suspendedReason) {
    return <Badge variant='destructive' shape='tag' title={suspendedReason}>{t('上游限制 · 已禁用')}</Badge>
  }
  if (!entries.length && !cooldown) {
    if (Array.isArray(account.limiterGroups) && limiterHit(account)) {
      return <LimiterStatus account={account} onOpen={() => openSettingsDialog(account.id)} />
    }
    if (balanceBlockedOf(account, usageEntryOf(account))) {
      return (
        <Badge variant='warning' shape='tag'
          title={t('余额低于限制器的阈值，转发时会跳过该账号（余额回升自动恢复）')}>
          {t('余额不足')}
        </Badge>
      )
    }
    if (tokenBlockedOf(account, tokenUsageOf(account))) {
      return (
        <Badge variant='warning' shape='tag'
          title={t('本周期 Token 消耗达到限制器的上限，转发时会跳过该账号（窗口重置自动恢复）')}>
          {t('Token 限额')}
        </Badge>
      )
    }
    return (
      <Badge variant='success' shape='tag' title={t('当前没有任何模型处于限流中')}>
        <BadgeDot />{t('正常')}
      </Badge>
    )
  }
  return (
    <LimitDetailButton
      accountName={displayNameOf(account)}
      rows={limitRowsOf(account)}
      count={entries.length}
      cooldown={cooldown}
      soonest={formatResetText(cooldown ? cooldown.resetAt : entries[0].resetAt)}
      onClear={onClear}
    />
  )
}

/* ─── valid / / ──────────────── */

/**
 * valid： by 「 this has no has 」（workbuddy is expiresAt， its is 
 * tokenExpiresAt）， and tokenExpiryOf 。「30 after 」，
 * in title —— has 。
 *
 * `expiryMillis` ** 1 to ms**： to in seconds and ms
 * （Trae is from CPA auth file and ， that is 10 seconds； and up 
 * new response is 13 ms）， not 1 when `1791009732` ms is 1970-01-21， 1 
 * 「expired」。 1 then and by in accounts-domain.ts that 1 ，
 * after `providers::trae::Credential::expires_at_ms` is 1 。
 *
 * ── 「valid」 and 「—」 is ────────────────────────────
 * no has to when （Antigravity Google refresh token、、
 * ModelScope、Command Code）「valid」—— is 「 need not need 」。
 * 「—」「 this not to to when 」（KukuAI BDUSS is this ： not 
 * when ， up when can can ）。 is can `longLived` ， not is 「 as empty」。
 */
export function ExpiryCell({ account }: { account: AccountRecord }) {
  const features = providerFeatures(providerOf(account))
  const expiresAt = features.edition ? expiryMillis(account.expiresAt) : tokenExpiryOf(account)
  const rows = credentialExpiryRowsOf(account, expiresAt)
  if (rows.length) return <div className='flex flex-col items-center gap-1'>
    {rows.map(row => <CredentialExpiryLine key={row.label} row={row} />)}
  </div>
  if (features.longLived) {
    return <span title={t('主凭证没有固定到期时刻，到期由网关自动续期或本来就不会过期')}>{t('长期有效')}</span>
  }
  if (!expiresAt) return <span className='muted' title={t('记录里没有过期时间')}>—</span>
  const left = expiresAt - Date.now()
  if (left <= 0) return <Badge variant='destructive' shape='tag' title={t('凭证已过期；能否续期取决于登录方式，本机引用需由原客户端处理')}>{t('已过期')}</Badge>
  const text = left < 3600e3 ? t('{n} 分钟后', { n: Math.max(1, Math.round(left / 60e3)) })
    : left < 48 * 3600e3 ? t('{n} 小时后', { n: (left / 3600e3).toFixed(1) })
      : t('{n} 天后', { n: Math.floor(left / 24 / 3600e3) })
  // when in when title：formatTime for when returnempty，
  // will down 1 empty 「（）」
  const full = formatTime(expiresAt)
  return <span title={full ? t('{text}过期（{full}）', { text, full }) : t('{text}过期', { text })}>{text}</span>
}

/**
 * 1 ： and in （`Access Token · 317 after `）。
 *
 * ** not use ** `justify-between` to 、 to —— that will in 
 * 1 empty， and its （ / / ） in 
 * not 1 。 and 、 in ， is 「 1 」。
 *
 * to 9px is as 「Refresh Token + 」 in 132px down ；
 * and to when in title ， not not will 。
 */
function CredentialExpiryLine({ row }: { row: CredentialExpiryRow }) {
  const expired = row.expiresAt !== null && row.expiresAt <= Date.now()
  const title = row.expiresAt === null ? t('记录里没有过期时间') : formatTime(row.expiresAt)
  return <div className='flex items-center justify-center gap-1.5 whitespace-nowrap' title={`${row.label} · ${title}`}>
    <span className='text-[9px] text-subtle' translate='no'>{row.label}</span>
    {expired ? <Badge variant='destructive' shape='tag'>{t('已过期')}</Badge>
      : <span className={row.expiresAt === null && !row.longLived ? 'muted' : undefined}>{credentialExpiryText(row, t)}</span>}
  </div>
}

/**
 * ： in use this request（2 seconds 1 when ）。
 * and OmniProxy up 「」 1 —— has when 、 as 0 when ** not 
 * **（empty）。 0 will in no ； need 「 is 0」 when empty
 * is 。（ no to 、 after not can ） and 0 ： 1 unknown 0
 * will 「 this no in use 」， can can is 「 no to 」。
 */
export function ConnectionsCell({ account }: { account: AccountRecord }) {
  const value = connectionsOf(account.id)
  if (value <= 0) return null
  return (
    <span className='conn-count' title={t('{n} 个请求正在使用该账号（含还在下发内容的流式请求）', { n: value })}>
      {value}
    </span>
  )
}

/**
 * → that 1 （ / / to / ）。
 *
 * and use `subscriptionText`： that 1 is has ， 1 
 * will and not 1 （ 1 ， use can for ）。
 * old by （ not is by provider）。
 */
function creditSubscriptionOf(entry: UsageEntry): string {
  if (!entry || typeof entry !== 'object' || Array.isArray(entry)) return ''
  return subscriptionText((entry as Record<string, unknown>).subscription)
}

/**
 * → need title 1 （ 1 `subscription`）。
 * `expireAt` not （ to is up ，AutoClaw can can to when ）：
 * can by when ，otherwise —— not 、 not 。
 */
function subscriptionText(subscription: unknown): string {
  if (!subscription || typeof subscription !== 'object') return ''
  const info = subscription as Record<string, unknown>
  const parts: string[] = []
  if (info.planName) parts.push(t('套餐 {name}', { name: String(info.planName) }))
  if (info.status) parts.push(t('状态 {status}', { status: String(info.status) }))
  // 「 in 」 need ： can can is **** （ to no to 
  // when ， after `plan_state`），「 X to …」 will in in use 
  if (info.state && String(info.state) !== 'active') {
    parts.push(zcodePlanStateLabel(info.state))
  }
  const expireAt = info.expireAt
  if (expireAt !== null && expireAt !== undefined && expireAt !== '') {
    const asNumber = Number(expireAt)
    const text = Number.isFinite(asNumber) && asNumber > 1e11 ? formatTime(asNumber) : String(expireAt)
    if (text) parts.push(t('到期 {time}', { time: text }))
  }
  if (Number.isFinite(Number(info.remainQuota))) parts.push(t('余量 {value}', { value: numberText(info.remainQuota) }))
  if (Number.isFinite(Number(info.totalQuota))) parts.push(t('总量 {value}', { value: numberText(info.totalQuota) }))
  return parts.join(' ')
}

/**
 * **** （`plans` `state === 'pending'` that ）。
 * no has this （ its ） and old after 1 returnempty —— ， not provider。
 *
 * Why this 1 ： not is （client
 * 「 23:00 · expiration time 1012 09:00」）， to that no 。
 * up not ， use will as no up ； 1 「 to 、 no to when 」。
 */
export function pendingPlans(entry: UsageEntry): Array<Record<string, unknown>> {
  if (!entry || typeof entry !== 'object' || Array.isArray(entry)) return []
  const plans = (entry as Record<string, unknown>).plans
  if (!Array.isArray(plans)) return []
  return plans.filter(item => item && typeof item === 'object'
    && String((item as Record<string, unknown>).state || '') === 'pending') as Array<Record<string, unknown>>
}

/**
 * ：「（ when ）」—— when can can in ， that 。
 *
 * use after `pendingUntil`（ 1 no to ** when **， after 
 * `plan_effective_times`）； when `startsAt`（ that is when ， in no has 
 * up etc when ）。
 */
export function pendingPlansTitle(plans: Array<Record<string, unknown>>): string {
  const lines = plans.map(plan => {
    const name = String(plan.name || plan.planId || t('套餐'))
    const starts = Number(plan.pendingUntil ?? plan.startsAt)
    return Number.isFinite(starts) && starts > 0
      ? t('{name}（{time} 起生效）', { name, time: formatTime(starts * 1000) })
      : name
  })
  return t('已领到、还没到生效时间的套餐（生效后余额列会按它显示）：\n{lines}', { lines: lines.join('\n') })
}

/**
 * → 1 need （`{text, kind, title}`）。
 *
 * by **** not is by provider（`totalLeft` = workbuddy has ，otherwise
 * `available` / `wallets`）：provider 「」， not 「」。
 * failure and 「not configured」 usageFailureOf（ 1 ， and that toast ）。
 * need to 「 + 」，（ / / to ） title ——
 * is this 1 。
 */
export function usageSummary(entry: UsageEntry): { text: string; kind: string; title: string } {
  if (entry === undefined) return { text: t('未查询'), kind: 'muted', title: t('尚未查询该账号的余额') }
  if (entry === null) return { text: t('查询中…'), kind: 'muted', title: t('正在查询') }
  const failure = usageFailureOf(entry)
  if (failure) {
    return failure.notConfigured
      ? { text: t('未配置'), kind: 'muted', title: t('{message}（去该账号的「设置」里填上查询凭证即可）', { message: failure.message }) }
      : { text: t('查询失败'), kind: 'bad', title: failure.message }
  }
  if (typeof entry !== 'object' || entry === null) return { text: t('无数据'), kind: 'muted', title: String(entry) }
  const data = entry as Record<string, unknown>
  if (Object.prototype.hasOwnProperty.call(data, 'totalLeft')) {
    const total = data.unlimited ? '∞' : numberText(data.totalLeft)
    return {
      text: t('可用 {total}', { total }),
      kind: 'ok',
      title: t('总剩余 {total} · 套餐 {plan} · 奖励 {bonus}', {
        total, plan: numberText(data.planLeft), bonus: numberText(data.bonusLeft),
      }),
    }
  }
  if (Array.isArray(data.windows) && data.windows.length) {
    const windows = data.windows as Array<Record<string, unknown>>
    const missing = usageWarningsOf(data)
    return {
      text: t('分窗口额度'),
      kind: missing.length ? 'warn' : 'ok',
      title: [...new Set([...usageSupplementReadingsOf(data, t, numberText), subscriptionText(data.subscription), ...windows.filter(row => row && typeof row === 'object').map(row => [usageWindowLabel(row.label, t), usageWindowView(row, t, numberText)].filter(Boolean).join(' ')), ...missing.map(line => t(line))].filter(Boolean))].join(' · '),
    }
  }
  if (Object.prototype.hasOwnProperty.call(data, 'available') || Array.isArray(data.wallets)) {
    const unit = usageUnitLabel(String(data.unit || t('积分')), t)
    const wallets = creditStatisticsWalletsOf(data) ?? (Array.isArray(data.wallets) ? data.wallets as Array<Record<string, unknown>> : [])
    // up to （ / ）， no has by 
    const detail = wallets
      .filter(wallet => !isSnapshotWallet(wallet))
      .map(wallet => `${wallet?.displayName || wallet?.type || t('明细')} `
        + `${wallet?.balanceView ? String(wallet.balanceView) : numberText(wallet?.balance)}`)
      .join(' · ')
    const readings = snapshotReadingsOf(data, t, numberText)
      .map(reading => `${reading.name} ${reading.view}`)
      .join(' · ')
    const subscription = subscriptionText(data.subscription)
    // ── no has when （， is **** not is ）───────
    // this （ in that up ） will ：
    // already use /total， has 「 x」 not —— need ，
    // otherwise「 3 token」 will 「 3 」。
    // ： if is from （`walletsFrom`）， need 1 
    // —— that is ** 1 ** （ N when / remaining）， and to none。
    const notes: string[] = []
    if (wallets.length === 0 && Array.isArray(data.plans) && data.plans.length > 0) {
      const plan = leadPlan(data)
      notes.push(plan && grantUsage(plan)
        ? t('读数里没有额度桶，这里按套餐权益里的已用 / 总量显示（上游给的数）')
        : t('上游的余额读数里没有这个套餐的额度桶，剩余量读不到；余额列显示的是套餐发放的权益额度，不是剩余量'))
    }
    if (String(data.walletsFrom || '') === 'monitor') {
      notes.push(t('额度窗口读数来自开放平台监控通道（套餐 JWT 那条通道没有额度桶）：读的是每 N 小时 / 每周窗口的剩余比例'))
    }
    // ：ZCode is token（1 = 9 ）， has 
    // —— after to `availableView`（`1 token` this ）。 no has this 
    // （ its ） is 「 + 」 that ， as 1 。
    // `available` as null is 「 up has 、 no has for remaining」 （Accio 
    // `usagePercent`：issue #242 「available — %」）—— when ** **
    // （`balance` + `unit`）， has can not 「—」。 no has 
    // （empty）「—」， no to 。
    const fallbackWallet = data.available == null
      ? wallets.find(wallet => numberOrNull(wallet?.balance) !== null)
      : undefined
    const available = data.availableView
      ? String(data.availableView)
      : fallbackWallet
        ? t('可用 {value} {unit}', {
            value: numberText(fallbackWallet.balance),
            unit: usageUnitLabel(String(fallbackWallet.unit || unit), t),
          })
        : t('可用 {value} {unit}', { value: numberText(data.available), unit })
    // when already ， and 。
    const missing = [
      ...usageWarningsOf(data).map(line => t(line)),
      data.statisticsError ? t('订阅统计未读到：{error}', { error: String(data.statisticsError) }) : '',
    ].filter(Boolean)
    const supplement = usageSupplementReadingsOf(data, t, numberText)
    const missingNumber = numberOrNull(data.available) === null && !fallbackWallet && !data.availableView
    const knownView = missingNumber && supplement.length ? supplement[0] : available
    return {
      text: knownView + (missing.length ? ' ⚠' : ''),
      kind: missing.length ? 'warn' : missingNumber ? 'muted' : 'ok',
      title: [...new Set([knownView, detail, readings, subscription, ...supplement, ...notes, ...missing].filter(Boolean))].join(' · '),
    }
  }
  return { text: t('无数据'), kind: 'muted', title: t('未返回可识别的余额数据') }
}

/**
 * 「 this 」 ： in ， its （ and after `pick_plan` 1 
 * —— 1 to 、 no 1 already not old this this 
 * ）。 not （ / up new ） in after —— has total no has 。
 */
function leadPlan(data: Record<string, unknown>): Record<string, unknown> | null {
  const plans = Array.isArray(data.plans) ? data.plans as Array<Record<string, unknown>> : []
  const stateOf = (plan: Record<string, unknown>) => String(plan?.state || '')
  return plans.find(plan => stateOf(plan) === 'active')
    || plans.find(plan => stateOf(plan) === 'pending')
    || plans[0]
    || null
}

/**
 * 1 ****（`{units, unit}`； 1 can no has when null）。
 *
 * use to 「 no has 」 in 1 —— that is **** not is remaining，
 * use 「」and not （ `usagePool` ）。
 */
function biggestGrant(plan: Record<string, unknown>): { units: number; unit: string } | null {
  const items = Array.isArray(plan.entitlements) ? plan.entitlements as Array<Record<string, unknown>> : []
  let best: { units: number; unit: string } | null = null
  for (const item of items) {
    const units = Number(item?.grantUnits)
    if (!Number.isFinite(units) || units <= 0) continue
    const unit = String(item?.unitType || '').trim()
    if (!best || units > best.units) best = { units, unit }
  }
  return best
}

/**
 * ** can ** usage（`{used, total, unit}`； to not when null）。
 *
 * up and not 1 to （`used_units` / `total_units` / `remaining_units` is optional，
 * after ， is null）—— to 1 ， no to 「」
 * that （ `usagePool` ）。** 1 total as ** ：
 * is ， this 1 「」。
 */
function grantUsage(plan: Record<string, unknown>): { used: number; total: number; unit: string } | null {
  const items = Array.isArray(plan.entitlements) ? plan.entitlements as Array<Record<string, unknown>> : []
  for (const item of items) {
    const total = numberOrNull(item?.totalUnits)
    if (total === null || total <= 0) continue
    const used = numberOrNull(item?.usedUnits)
    const remaining = numberOrNull(item?.remainingUnits)
    const usedValue = used === null
      ? (remaining === null ? null : Math.max(0, total - remaining))
      : used
    if (usedValue === null) continue
    return { used: usedValue, total, unit: String(item?.unitType || '').trim() }
  }
  return null
}

/**
 * 「」： use and （`UsageCell` ）。
 *
 * ── （ by 「 can 」）────────────────────────────
 * ① ** has total **total that （）；
 * ② 1 「 has total」 no has when ，** has ** remaining（） that 
 * —— that is （2026-02 up not to for ，
 * after `monitor.rs`）， this 1 is 「 not use 」， that is ；
 * ③ no has （ not in ） when ， ：
 * already use /total****（「 x / y」）， has 
 * 「 x」and** not ** —— that is not is remaining， 1 
 * will 「 this 」。
 * not to （ no ）return null， that 1 「none」 is for 。
 *
 * ── Why「total that 」──────────────────────────────
 * 1 when ： （ZCode Trust Build 1 ）
 * and （Start Plan 300 ）。 use this 1 is ** that **
 * ，「total」 is —— daily in 1 
 * 。total 1 not and this 1 （ no has total not 「 use 」），
 * is no has （ its ） up that 。
 *
 * ── 1 「 not 」─────────────────────────────────────
 * `remainingPercent` when ** not **（ and to ），
 * not 0 or 100 —— 1 and 1 empty is ，
 * can can is 。
 */
export function usagePool(entry: UsageEntry): { planName: string; text: string; percent: number | null } | null {
  if (!entry || typeof entry !== 'object' || Array.isArray(entry)) return null
  const data = entry as Record<string, unknown>
  const wallets = Array.isArray(data.wallets)
    ? (data.wallets as Array<Record<string, unknown>>).filter(wallet => !isSnapshotWallet(wallet))
    : []
  let lead: Record<string, unknown> | null = null
  for (const wallet of wallets) {
    const total = Number(wallet?.total)
    if (!Number.isFinite(total) || total <= 0) continue
    if (!lead || total > Number(lead.total)) lead = wallet
  }
  const subscription = (data.subscription && typeof data.subscription === 'object'
    ? data.subscription
    : {}) as Record<string, unknown>
  if (!lead) {
    // ② has （）：remaining that 
    let tightest: { wallet: Record<string, unknown>; percent: number } | null = null
    for (const wallet of wallets) {
      const percent = numberOrNull(wallet?.remainingPercent)
      if (percent === null) continue
      if (!tightest || percent < tightest.percent) tightest = { wallet, percent }
    }
    if (tightest) {
      const percent = Math.max(0, Math.min(100, tightest.percent))
      // need ** before **： is 「 72% · 10-11 03:00 」，
      // when that 1 in （`usageSummary` detail use ）——
      // this 1 has ， will 。「 · 」 is after 
      // `monitor::wallet_view` （ that 1 that ）， before that is 。
      const view = String(tightest.wallet.balanceView || '').split(' · ')[0].trim()
      return {
        planName: String(tightest.wallet.planName || subscription.planName || ''),
        text: view || t('剩余 {percent}%', { percent: Math.round(percent) }),
        percent,
      }
    }
    // ③ no has ：（ has already use /total， has ）
    const plan = leadPlan(data)
    if (!plan) return null
    const usage = grantUsage(plan)
    const planName = String(plan.name || plan.planId || subscription.planName || '')
    if (usage) {
      const remaining = Math.max(0, usage.total - usage.used)
      return {
        planName,
        text: t('剩 {left} / {total}{unit}', {
          left: compactNumber(remaining),
          total: compactNumber(usage.total),
          unit: usage.unit ? ` ${usage.unit}` : '',
        }),
        percent: Math.max(0, Math.min(100, (remaining / usage.total) * 100)),
      }
    }
    const grant = biggestGrant(plan)
    if (!grant) return null
    return {
      planName,
      text: grant.unit
        ? t('权益 {value} {unit}', { value: compactNumber(grant.units), unit: grant.unit })
        : t('权益 {value}', { value: compactNumber(grant.units) }),
      // not is remaining： not （ up that ）
      percent: null,
    }
  }
  // this （`wallet.planName`）， that ——
  // not when （ up no to plan_id） has can 
  const planName = String(lead.planName || subscription.planName || lead.displayName || '')
  const remaining = numberOrNull(lead.balance)
  const total = numberOrNull(lead.total)
  const percent = numberOrNull(lead.remainingPercent)
    ?? (remaining !== null && total !== null && total > 0 ? remaining / total * 100 : null)
  const text = remaining !== null && total !== null && total > 0
    ? t('剩 {remaining} / {total}', { remaining: compactNumber(remaining), total: compactNumber(total) })
    : t('可用 {value} {unit}', { value: numberText(lead.balance), unit: usageUnitLabel(String(lead.unit || ''), t) }).trim()
  return {
    planName,
    text: String(lead.balanceView || text),
    percent: percent === null ? null : Math.max(0, Math.min(100, percent)),
  }
}

/**
 * → number or null。
 *
 * must null/undefined/empty：`Number(null)` is 0（ not is NaN）， will 
 * 「 up no to this 」「 this is 0」—— will 1 empty， and 「remaining 0%」
 * 1 1 ， that is （ `usagePool` ）。
 */
function numberOrNull(value: unknown): number | null {
  if (value === null || value === undefined || value === '') return null
  const parsed = Number(value)
  return Number.isFinite(parsed) ? parsed : null
}

/* ─── （usageDetail can that ， has CodeArts）─── */

/** 1 （`UsageDetailCell` ） */
type UsageDetailRow = {
  name: string
  /** ：「 x / y」，token use after balanceView（ in after ） */
  view: string
  tone: 'ok' | 'warn' | 'bad' | 'muted'
  /** **remaining**（ is remaining， and usage-pool to ）；null = not */
  percent: number | null
  /** ： + （ up to limit after ，） */
  over: boolean
  /** （Why no / / 4004 —— up not Reason） */
  title?: string
}

/**
 * 1 （、token 、 use this 1 ）。
 *
 * and ：** can **（ 1 use 1 when 1 ），
 * ** and not ** —— 1 down 1 in that ，
 * 1 1 will 1 、 not is 。
 */
type UsageFlatRow = {
  /** （；none empty —— e.g. OpenCode has no has ） */
  name: string
  /** （`5h` / `` / ``）， 1 */
  win?: string
  /** （/； no has when ） */
  view: string
  /** （`72%`）： and ， has `view` */
  percentText?: string
  tone: 'ok' | 'warn' | 'bad' | 'muted'
  /** **remaining**（ is remaining， and to ）；null = up no to ， not */
  percent: number | null
  /** ： */
  over: boolean
  /** （ after balanceView + ） */
  title?: string
  /** React ： （ after `windows[].key`， `gemini-5h`）。
   * no has id name —— use in 1 ， not */
  id?: string
}

/** no has remaining when ， not use emptyunknown。 */
function UsageFlatRowView({ row }: { row: UsageFlatRow }) {
  const value = row.percentText || row.view
  return (
    <span className={`usage-meter ${row.tone}`} title={row.title}>
      <span className='usage-meter-head'>
        {row.name ? <span className='usage-meter-name'>{row.name}</span> : null}
        {row.win ? <span className='usage-meter-win'>{row.win}</span> : null}
        <span className='usage-meter-val'>{value}</span>
      </span>
      {row.percent !== null ? (
        <Meter value={row.percent} variant={row.over || row.tone === 'bad' ? 'destructive' : row.tone === 'ok' ? 'success' : row.tone === 'warn' ? 'warning' : 'muted'}
          className='mt-1' trackClassName='h-[3px]' aria-label={t('剩余 {percent}%', { percent: Math.round(row.percent) })} />
      ) : null}
    </span>
  )
}

/** （`usageDetailOf` ） */
type UsageDetail = {
  credit: UsageDetailRow[]
  /** available（remaining and ； not when null， not ） */
  total: number | null
  unit: string
  /** （ use / / ） and */
  plan: string
  /** up is free（`plan-chip.free` by —— not can plan ，
   * after ， accounts-domain isFreePlan） */
  free: boolean
  resetDate: string
  /** 。 */
  missing: string[]
}

/** units.js （file formatTokens / formatInt）：（ / and k / M）
 * already by in that 1 ， this not 。 + not is 
 * declare global —— wbUnits is ， declare 1 will TS2717
 * （ report-charts ）。 */
type UnitsBridge = {
  formatTokens?: (value: unknown) => string
  /** （ in 1 use ， and 1 ） */
  formatInt?: (value: unknown) => string
}

/**
 * → （token is ， has 148px， is ）：
 * and to units.js formatTokens（「354」、「354k」，
 * set that 1 down 1 ）； not to when 。
 *
 * before by 1 ：`formatTokens` for 1 ，
 * `toLocaleString` （`39.4321` → 「39.432」）—— and `numberText`
 * limit not 1 ， 1 in will not 。
 */
function compactNumber(value: number): string {
  const rounded = Number.isFinite(value) ? Number(value.toFixed(2)) : value
  const api = (window as unknown as { wbUnits?: UnitsBridge }).wbUnits
  return api?.formatTokens ? api.formatTokens(rounded) : String(rounded)
}

/**
 * → 。 1 ， can by use 。
 *
 * ── Why in before ─────────────────────────────
 * after `Meter::view()` for 「 has remaining no has total」 （ this ） will 
 * 「 already use 0%」—— that is already （ balance.rs that TODO ）。 this 
 * by creditTotal / creditRemaining 「 x / y」， in up ，
 * not that 。
 *
 * ── ───────────────────────────────────────────
 * **remaining**（ and usage-pool to ）：≥20% 、<20% （ need ）、
 * 。 0 no has （ by need ） is —— 「0 not is 」；
 * nonetotal not （ not not has ）， up no to is no to 。
 */
function usageDetailOf(entry: UsageEntry): UsageDetail | null {
  if (!entry || typeof entry !== 'object' || Array.isArray(entry)) return null
  const data = entry as Record<string, unknown>
  const wallets = Array.isArray(data.wallets) ? data.wallets as Array<Record<string, unknown>> : []
  if (!wallets.length) return null
  const unit = usageUnitLabel(String(data.unit || '积分'), t)
  const credit: UsageDetailRow[] = []
  for (const wallet of wallets) {
    const name = String(wallet.displayName || wallet.type || t('明细'))
    const remaining = numberOrNull(wallet.creditRemaining)
    const total = numberOrNull(wallet.creditTotal)
    if (remaining === null && total === null && wallet.creditUsed == null) continue
    if (remaining !== null) {
      // （）：total when not 
      const hasTotal = total !== null && total > 0
      const percent = hasTotal ? Math.max(0, Math.min(100, remaining / total * 100)) : null
      credit.push({
        name,
        view: hasTotal
          ? t('剩 {remaining} / {total}', { remaining: numberText(remaining), total: numberText(total) })
          : t('剩 {remaining}', { remaining: numberText(remaining) }),
        tone: remaining <= 0 ? 'muted' : percent !== null && percent < 20 ? 'warn' : 'ok',
        percent,
        over: false,
        title: hasTotal ? undefined : t('上游未给总量，画不出剩余比例'),
      })
      continue
    }
    credit.push({ name, view: wallet.creditUsed == null ? t('总量 {value}', { value: numberText(total) })
      : t('已用 {used}', { used: numberText(wallet.creditUsed) }), tone: 'muted', percent: null, over: false })
  }
  const subscription = (data.subscription && typeof data.subscription === 'object'
    ? data.subscription : {}) as Record<string, unknown>
  const missing = [
    data.statisticsError ? t('订阅统计未读到：{error}', { error: String(data.statisticsError) }) : '',
  ].filter(Boolean)
  return {
    credit,
    total: numberOrNull(data.available),
    unit,
    plan: planBadgeLabel(subscription.planName),
    free: isFreePlan(subscription.planName),
    resetDate: String(subscription.resetDate || ''),
    missing,
  }
}

/**
 * ：****（ not can ）—— by in ， this 1 is 
 * 「 1 」。 not Badge： is not is ，
 * in `.usage-sum` （：ok / bad / muted / warn —— `warn` is 
 * 「 to not 」 that 1 ， up `usageSummary` failure）。
 *
 * ── 1 ：（`usagePool`，ZCode that ）──────────
 * 1 is （ use 1 is 「 this 1 is to 」），
 * is + 「remaining / total」。 is **remaining** ——
 * and 1 to ，otherwise「」 and 「 8800 」 will 。
 * （ 、 to 、available） in ， this ：
 * 、。
 *
 * ── ： + （`usageDetail` can ，CodeArts）──
 * ，total can 。
 */
/**
 * Token limit ****： Token then 
 * 「 Token already use / limit · N 」。 then 「 that 」——
 * already ，otherwiseusage 1 ； its then in set。
 */
function prominentTokenLimit(account: AccountRecord): React.ReactNode {
  const readings = tokenUsageOf(account)
  let best: { rule: LimiterRule; reading: TokenReading; ratio: number } | null = null
  for (const rule of tokenRulesOf(account)) {
    if (rule.enabled === false) continue
    const reading = tokenReadingForRule(rule, readings)
    if (!reading) continue
    const ratio = rule.threshold > 0 ? reading.used / rule.threshold : 0
    if (ratio >= 1) {
      best = { rule, reading, ratio }
      break
    }
    if (!best || ratio > best.ratio) best = { rule, reading, ratio }
  }
  if (!best) return null
  const { rule, reading } = best
  const hit = reading.used >= rule.threshold
  const info = tokenWindowInfo(rule)
  const countdown = !hit ? ` · ${tokenCountdownText(info.remainingMs, info.daily)}` : ''
  const scope = info.daily ? t('今日已用') : t('周期 Token')
  const windowText = rule.reset === 'daily' ? t('今天（本地 0 点重置）') : t('{period}窗口', { period: formatIntervalSeconds(rule.period ?? 0) })
  const title = t('限制器：{window}已用 {used} / {threshold} Token', {
    window: windowText,
    used: Math.round(reading.used).toLocaleString('en-US'),
    threshold: Math.round(rule.threshold).toLocaleString('en-US'),
  })
    + (hit ? t('，已达到上限') : t('，{countdown}重置', { countdown: tokenCountdownText(info.remainingMs, info.daily) }))
  return (
    <span className='usage-token-sub' title={title}>
      {scope} <span className={hit ? 'hit' : undefined}>{formatTokenCount(reading.used)}</span>
      {' '}/ {formatTokenCount(rule.threshold)}{countdown}
    </span>
  )
}

export function UsageCell({ account }: { account: AccountRecord }) {
  const entry = usageEntryOf(account)
  const failureStatus = supportsUsage(account) ? usageFailureStatusOf(account) : null
  const lastUsage = failureStatus && entry && typeof entry === 'object' ? entry.lastUsage : null
  if (!failureStatus || !lastUsage || typeof lastUsage !== 'object' || Array.isArray(lastUsage)) {
    return <UsageReadingCell account={account} entry={entry} />
  }
  return (
    <span className='usage-sum-wrap flex flex-col gap-1' title={failureStatus.detail}>
      <UsageReadingCell account={account} entry={lastUsage as Record<string, unknown>} />
      <span className='text-[11px] text-destructive whitespace-normal leading-snug' title={failureStatus.detail}>
        {failureStatus.text}
      </span>
    </span>
  )
}

function UsageReadingCell({ account, entry }: { account: AccountRecord; entry: UsageEntry }) {
  const now = useDisplayClock()
  if (!supportsUsage(account)) {
    // and 「 this no has 」 is not ， need ： before is 
    // 「 this no has 」， after is 「 this 1 no has can 」。 1 
    // will use as unsupported（ its can ）。
    return (
      <span className='muted'
        title={account.isGuest === true
          ? t('访客账号没有积分体系，因此不查询余额（登录账号可以查）')
          : t('该提供商尚未接入余额查询')}>—</span>
    )
  }
  const rawSummary = usageSummary(entry)
  const creditRows = supportsCreditDetail(account) && !usageFailureOf(entry) ? creditRowsOf(entry, now) : []
  const expiring = creditExpirySummaryOf(creditRows.flatMap(row => row.expiry ? [row.expiry] : []), now)
  const expiryText = creditExpiryText(expiring, t, numberText, creditExpiryUnitOf(entry))
  const summary = expiryText.detail ? { ...rawSummary, title: [rawSummary.title, expiryText.detail].filter(Boolean).join(' · ') } : rawSummary
  // limit ： and after 1 （balanceBlockedOf / tokenBlockedOf），
  // 「Why this not request」 in up has can 。 use not —— that 1 
  // 「 already use 」 is ； is enabled ， not not 。
  const blockedBadge = Array.isArray(account.limiterGroups) ? (
    <LimiterStatus account={account} onOpen={() => openSettingsDialog(account.id)} />
  ) : balanceBlockedOf(account, entry) ? (
    <Badge variant='warning' shape='tag'
      title={t('余额低于限制器的阈值，转发时会跳过该账号（余额回升自动恢复）')}>
      {t('余额不足 · 已跳过')}
    </Badge>
  ) : tokenBlockedOf(account, tokenUsageOf(account)) ? (
    <Badge variant='warning' shape='tag'
      title={t('本周期 Token 消耗达到限制器的上限，转发时会跳过该账号（窗口重置自动恢复）')}>
      {t('Token 限额 · 已跳过')}
    </Badge>
  ) : null
  const tokenSub = prominentTokenLimit(account)
  // 「 N 」：Note in available（ok / warn） when has —— failure /
  // not configured that no has ， 1 「 to 」。usageDetail that 1 
  // （CodeArts） no has ， not 。
  const pending = summary.kind === 'ok' || summary.kind === 'warn' ? pendingPlans(entry) : []
  const pendingBadge = pending.length ? (
    <Badge variant='warning' shape='tag' title={pendingPlansTitle(pending)}>
      {t('待生效 {n} 份', { n: pending.length })}
    </Badge>
  ) : null
  // can can has to when no has total； this ，failure and not configured when not empty。
  const creditDetail = creditRows.length
    ? {
      rows: creditRows,
      now,
      expiryUnit: creditExpiryUnitOf(entry),
      subscriptionLine: creditSubscriptionOf(entry),
      aggregate: usageAggregateOf(entry, t),
      kind: summary.kind,
      title: summary.title,
    }
    : null
  const windows = summary.kind === 'ok' || summary.kind === 'warn' ? usageWindows(entry) : []
  if (windows.length) {
    const detail = creditDetail ? (
      <CreditDetailButton variant='sum' {...creditDetail}>{t('积分明细')}</CreditDetailButton>
    ) : null
    return (
      <UsageWindowsCell entry={entry} rows={windows} updatedAt={usageUpdatedAtOf(account)} creditDetail={detail}>
        {blockedBadge}
        {pendingBadge}
        {tokenSub}
      </UsageWindowsCell>
    )
  }
  if (supportsUsageDetail(account)) {
    const usable = summary.kind === 'ok' || summary.kind === 'warn'
    const detail = usable ? usageDetailOf(entry) : null
    if (detail) {
      return <UsageDetailCell summary={summary} detail={detail} blockedBadge={blockedBadge} tokenSub={tokenSub} />
    }
    // no to / failure / no ： 
    return (
      <span className='usage-sum-wrap'>
        <span className={`usage-sum ${summary.kind}`} title={summary.title}>{summary.text}</span>
        {blockedBadge}
        {pendingBadge}
        {tokenSub}
      </span>
    )
  }
  // failure / not configured that not ： not is 「」，
  // to will 1 error 1 can 
  const pool = summary.kind === 'ok' || summary.kind === 'warn' ? usagePool(entry) : null
  if (!pool) {
    // has when is by （`.usage-toggle`， and CodeArts that ）；
    // no has when is that not can 
    return (
      <span className='usage-sum-wrap'>
        {creditDetail ? (
          <CreditDetailButton variant='sum' {...creditDetail}>{summary.text}</CreditDetailButton>
        ) : (
          <span className={`usage-sum ${summary.kind}`} title={summary.title}>{summary.text}</span>
        )}
        {blockedBadge}
        {pendingBadge}
        {tokenSub}
      </span>
    )
  }
  // ── ： and 1 （ …… + ）──────────
  // is Progress（6px ） in 1 ， and that 
  // 3px not is 1 ； in 1 ： 1 （ use that 
  // 「 this 1 is to 」）， down 1 + 。
  const poolRow: UsageFlatRow = {
    name: pool.planName || '',
    view: pool.text,
    tone: summary.kind === 'warn' ? 'warn' : 'ok',
    percent: pool.percent,
    over: false,
    title: summary.title,
  }
  return (
    <span className='usage-cell' title={summary.title}>
      {/* and 1 in by ：（ can ，
           this 1 that in ） */}
      {creditDetail ? (
        <CreditDetailButton variant='pool' {...creditDetail}>
          <UsageFlatRowView row={poolRow} />
          {expiryText.badge ? <span className='usage-pool-soon'>{expiryText.badge}</span> : null}
        </CreditDetailButton>
      ) : (
        <UsageFlatRowView row={poolRow} />
      )}
      {blockedBadge}
      {pendingBadge}
      {tokenSub}
    </span>
  )
}

/**
 * （`usageDetail` can ）。
 *
 * hooks in this ：`UsageCell` for failure / has ，
 * useState in this not will 「 after hook」 then 。
 *
 * ， and total； use by 。
 */
function UsageDetailCell({ summary, detail, blockedBadge, tokenSub }: {
  summary: { text: string; kind: string; title: string }
  detail: UsageDetail
  blockedBadge: React.ReactNode
  /** Token limit （； then has ） */
  tokenSub: React.ReactNode
}) {
  const [open, setOpen] = React.useState(false)
  const usable = summary.kind === 'ok' || summary.kind === 'warn'
  return (
    <span className={open ? 'usage-cell open' : 'usage-cell'}>
      <span className='usage-plan'>
        {detail.plan
          ? <Badge variant={detail.free ? 'secondary' : 'brand'} shape='tag' className={`plan-chip${detail.free ? ' free' : ''}`}>{detail.plan}</Badge>
          : null}
        {detail.resetDate
          ? <span className='plan-sub' title={t('套餐积分的重置日期')}>{t('重置 {date}', { date: detail.resetDate })}</span>
          : null}
      </span>
      <Popover open={open} onOpenChange={setOpen}>
        <PopoverTrigger
          render={<Button size='default' variant='ghost' type='button' className={`usage-toggle ${summary.kind}`}
            title={summary.title} aria-label={t('积分明细')} />}
        >
          <span className='val'>{summary.text}</span>
          <span className='caret' aria-hidden>▾</span>
        </PopoverTrigger>
        {/* `align='center'`： and accounts-credit-detail / accounts-limit-detail
             1 —— by ，'end' will to 。 */}
        <PopoverContent align='center' sideOffset={6} className='usage-pop w-[288px]'>
          {detail.total !== null ? (
            <div className='usage-pop-total'>
              <span className='k'>{t('总积分')}</span>
              <span className='v'>{numberText(detail.total)} <small>{detail.unit}</small></span>
            </div>
          ) : null}
          {usable && detail.missing.length
            ? detail.missing.map(line => <div key={line} className='usage-pop-missing' title={line}>{line}</div>)
            : null}
          {detail.credit.length ? (
            <>
              <div className='usage-pop-group'>{t('积分')}</div>
              {detail.credit.map(row => <UsageDetailRowView key={row.name} row={row} />)}
            </>
          ) : null}
          {!detail.credit.length && detail.total === null ? (
            <Empty className='usage-pop-empty'>{t('未返回可识别的余额数据')}</Empty>
          ) : null}
        </PopoverContent>
      </Popover>
      {blockedBadge}
      {tokenSub}
    </span>
  )
}

/** 1 （ + + optional remaining； in ）。
 * and is 1 （3px、）， is 1 ——
 * 1 「remaining」 in and not this is 。 */
function UsageDetailRowView({ row }: { row: UsageDetailRow }) {
  return (
    <div className='usage-pop-row' title={row.title}>
      <div className='usage-pop-line'>
        <span className='usage-pop-name'>{row.name}</span>
        <span className={`usage-pop-view ${row.tone}`}>{row.view}</span>
      </div>
      {row.percent !== null ? (
        <Meter value={row.percent} variant={row.over || row.tone === 'bad' ? 'destructive' : row.tone === 'ok' ? 'success' : row.tone === 'warn' ? 'warning' : 'muted'}
          className='mt-1.5' trackClassName='h-[3px]' aria-label={t('剩余 {percent}%', { percent: Math.round(row.percent) })} />
      ) : null}
    </div>
  )
}

/* ─── / / ─────────────────── */

/**
 * ： 1 ，（ has ）， in when （unavailableReason）。
 *
 * displayNameOf nameCustom ：set（） 
 * as ； —— （Qoder / AutoClaw
 * / Accio）， its 。update before set old no has ，
 * to set 1 （ not ）。
 *
 * is this 1 「」：** 1 1 **， Tooltip 
 * （ title by when and ， not use ）， to ——
 * 、、（ when can ）、 up 、update when 、。
 * （UID / userId） and Token not up not ： for 「 this can not can use 」
 * no has （Token in ， use ）。
 * when / in ——
 * not to 「 1 down 」 。
 */
export function AccountCell({ account, namesHidden }: { account: AccountRecord; namesHidden: boolean }) {
  const ident = identifierOf(account)
  const name = displayNameOf(account) || t('未命名账号')
  const email = String(account.email || '').trim()
  const nickname = String(account.nickname || '').trim()
  // （ displayNameOf ）： when when has ，
  // / ， this in can 
  const rawName = String(account.name || '').trim()
  const mask = (value: string): string => (namesHidden ? maskName(value) : value)
  const titleLines = [
    email && email !== name ? t('邮箱 {email}', { email: mask(email) }) : '',
    ident ? `${identifierKeyOf(account) || 'id'} ${ident}` : '',
    !account.nameCustom && rawName && rawName !== email && rawName !== nickname
      ? t('备注名 {name}', { name: mask(rawName) })
      : '',
    nickname && nickname !== name && nickname !== email ? t('昵称 {name}', { name: mask(nickname) }) : '',
    isDesktopAccount(account) ? t('桌面端实时登录态（凭证每次从客户端登录态文件读取）') : '',
    account.updatedAt ? t('更新于 {time}', { time: formatTime(account.updatedAt) }) : '',
    account.needsRelogin ? accountReloginText(account) : '',
    account.source ? t('来源 {source}', { source: account.source === 'imported' ? t('旧数据导入') : t('手动添加') }) : '',
  ].filter(Boolean)

  const showEmail = email && email !== name
  const proxyError = account.proxy?.error
  // title when by /（ 1 seconds， not ），
  // Tooltip：Provider delay=0 ；Portal not ；
  // 1 1 div（ use need 「 1 1 」）。
  const nameNode = titleLines.length ? (
    <TooltipProvider delay={0}>
      <Tooltip>
        <TooltipTrigger render={<div className='acct-name' />}>
          <span className='name'>{mask(name)}</span>
        </TooltipTrigger>
        <TooltipContent>
          <TooltipArrow />
          {titleLines.map((line, index) => <div key={index}>{line}</div>)}
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  ) : (
    <div className='acct-name'>
      <span className='name'>{mask(name)}</span>
    </div>
  )
  return (
    <>
      {nameNode}
      {showEmail ? (
        <div className='acct-sub'>
          <span className='acct-email' title={t('账号邮箱')}>{mask(email)}</span>
        </div>
      ) : null}
      {account.needsRelogin ? <div className='acct-note bad'>{accountReloginText(account)}</div> : null}
      {proxyError ? (
        <div className='acct-note bad'
          title={t('代理不可用：{error}', { error: proxyError })}>{t('代理不可用：{error}', { error: proxyError })}</div>
      ) : null}
    </>
  )
}


export function ProxyCell({ account }: { account: AccountRecord }) {
  const proxy = account.proxy
  const source = proxy?.config?.source || proxy?.source
  const label = proxy?.label || (source === 'custom' ? t('自定义代理') : t('已设置'))
  const broken = proxy?.error
  const pool = proxyPoolSnapshot()
  const poolItems = Array.isArray(pool) ? pool : []

  // current down ： use `pool:<id>`、 its （Clash / custom /
  // / ） to —— not can 「」： that will 「config」「 no 」
  let current = ''
  if (source === 'pool' && proxy?.config?.proxyId) current = `${POOL_VALUE_PREFIX}${proxy.config.proxyId}`
  else if (proxy) current = PROXY_CUSTOM_CURRENT

  const items: Array<{ value: string; label: string; disabled?: boolean }> = [{ value: '', label: t('直连') }]
  for (const item of poolItems) {
    // is 「（ :）」—— is use in 「」 ，
    // is after when （ poolItemLabel）。 1 when 
    // is 1 can ， not can 。
    //
    // use ** not optional**： will 「success → failure → 」，
    // use to is 「 already use 」 this after 。 need 「」enabled。
    // （current is use when —— is this 1 label，「 already use 」）
    items.push({
      value: `${POOL_VALUE_PREFIX}${item.id}`,
      label: poolItemLabel(item),
      disabled: item.enabled === false,
    })
  }
  if (pool === null) {
    // no to （ / effect ）： to 1 「 in 」 not is 
    // 「 no has 」—— after will use as is empty 
    items.push({ value: '__hint_pool__', label: t('正在读取代理列表…'), disabled: true })
  } else if (!poolItems.length) {
    // as empty / failure 1 ： after is （ in ）， before is 
    // 「 no 」—— not this 「 has 」
    items.push({
      value: '__hint_pool__',
      label: poolError() ? t('代理列表读取失败（重试中）') : t('还没有代理（去「网络代理」页添加）'),
      disabled: true,
    })
  }
  if (current === PROXY_CUSTOM_CURRENT) {
    // ： not （PROXY_CUSTOM_CURRENT in applyProxyPick ignore），
    // is current。 not before —— after to label is 
    // 「（N ）」 this 1 （ describe_account_proxy）
    const prefix = source === 'clash' ? t('Clash 出口：') : source === 'custom' ? t('自定义：') : ''
    items.push({ value: PROXY_CUSTOM_CURRENT, label: `${prefix}${label}${broken ? t('（不可用）') : ''}` })
  } else if (source === 'pool' && !poolItems.some(item => `${POOL_VALUE_PREFIX}${item.id}` === current)) {
    // use already not in （ / Clash ）：current
    items.push({ value: current, label: `${label}${broken ? t('（不可用）') : ''}` })
  }
  items.push({ value: PROXY_CUSTOM_EDIT, label: t('自定义代理…') })

  const title = broken
    ? t('代理不可用：{error}（转发时会回退直连）；选「自定义代理…」去修改', { error: broken })
    : source === 'rotate'
      ? t('当前：{label}（每条请求换一个出口）；「自定义代理…」打开完整设置', { label })
      : source === 'clash'
        ? t('当前：{label}（Clash Verge 直接引用）；可在「网络代理」页同步端点后改选', { label })
        : source === 'custom'
          ? t('当前：{label}（未经过代理池 —— 可在「网络代理」页添加相同地址后来这里改选）', { label })
          : t('当前：{label}；选项来自「网络代理」页；「自定义代理…」打开完整设置', {
          label: proxy ? label : t('直连'),
        })
  const selected = items.find(item => item.value === current)

  return (
    <Select value={current} onValueChange={value => void applyProxyPick(account.id, String(value), current)}>
      {/* failure when ： old `.cell-proxy.err .select-trigger` that CSS，
           Select after no has .select-trigger this （ data-slot），
           in up （ CSS） */}
      <SelectTrigger className={cn('w-full', broken && 'border-destructive-bd')}
        title={title} aria-label={t('出网代理')}>
        <SelectValue>{selected?.label || t('直连')}</SelectValue>
      </SelectTrigger>
      <SelectContent>
        {items.map(item => (
          <SelectItem key={item.value} value={item.value} disabled={item.disabled}>{item.label}</SelectItem>
        ))}
      </SelectContent>
    </Select>
  )
}

/* ─── ────────────────────────────────── */

/**
 * ： / set / ⋯，。
 *
 * by 「 」， by will ，** not **。
 * 「 as 」 not in this ： in ⋯ （ up 1 by 
 * ， is 50px， that 50px is from ）。
 *
 * （ to is this 1 by ， already to can to 「 to in 」——
 * checkin-page.tsx daily to ， to and 。
 * CodeArts 「」 1 ： to in 「」
 * （ in that ）， not 1 by 。
 * ZCode 「」： to in 「」 that 1 is （）
 * is （ 1 ）， this is 。）
 */
export function ActionsCell({ account, atFront }: { account: AccountRecord; atFront: boolean }) {
  const [usageBusy, setUsageBusy] = React.useState(false)
  const canUsage = supportsUsage(account)

  return (
    <div className='acct-actions'>
      {canUsage ? (
        <Button variant='outline' size='xs' disabled={usageBusy}
          title={t('查询该账号剩余余额（读数显示在余额列）')}
          onClick={() => {
            setUsageBusy(true)
            void queryUsageOnce(account.id).finally(() => setUsageBusy(false))
          }}>{t('余额')}</Button>
      ) : null}
      <Button variant='outline' size='xs' title={t('备注名 / 启用 / 代理')}
        onClick={() => openSettingsDialog(account.id)}>{t('设置')}</Button>
      <MoreMenu account={account} atFront={atFront} />
    </div>
  )
}

/* ─── ⋯ ───────────────────────────────── */

/**
 * ⋯ （ is 「 div.more-menu .cell-actions」 ， need 
 * to up is to down ）。 in Popover：、、、、
 * Esc、 by Base UI floating-ui that ， when 。
 *
 * by 「 for 」 from to ：enabled/ use ， in before ；「 as 」
 * （ not enabled）， in after ；「 and limit」 is （current）；
 * 「delete」， in after and 1 。 danger： will 
 * available。
 * 「 new Token」 by `hasRefreshToken` （ not refreshToken，
 * this 1 for not —— new is 「 not return」 path）。
 */
export function MoreMenu({ account, atFront }: { account: AccountRecord; atFront: boolean }) {
  const [open, setOpen] = React.useState(false)
  const enabled = isEnabled(account)
  const sortActive = groupSortRules(providerOf(account)).some(rule => rule.on)
  const maxConcurrent = Number(account.maxConcurrent) || 0
  const itemClass = 'w-full justify-start px-2 font-normal'
  const dangerClass = `${itemClass} text-destructive hover:text-destructive`

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger render={<Button variant='outline' size='xs' title={t('更多操作')} />}>⋯</PopoverTrigger>
      <PopoverContent align='end' sideOffset={4} className='w-[172px] p-1.5'>
        <div className='flex flex-col gap-0.5'>
          <Button variant='ghost' size='sm' className={dangerClass}
            onClick={() => { setOpen(false); void setAccountEnabled(account.id, !enabled) }}>
            {enabled ? t('禁用') : t('启用')}
          </Button>
          <Button variant='ghost' size='sm' className={itemClass} disabled={atFront || sortActive}
            title={sortActive ? t('自动消耗排序已启用，请先关闭规则再手动调整') : atFront ? t('已在全局队列第一位') : t('仅将优先级调整到全局第一位，不改变启用状态')}
            onClick={() => { setOpen(false); shared().wbApp?.runAccountAction?.('switch', account.id) }}>
            {t('设为首选')}
          </Button>
          <Button variant='ghost' size='sm' className={itemClass}
            title={t('在账号设置的「基本」分组里设置该账号同时最多处理的请求数')}
            onClick={() => { setOpen(false); openSettingsDialog(account.id) }}>
            {t('并发上限：{value}', { value: maxConcurrent > 0 ? maxConcurrent : t('不限') })}
          </Button>
          {account.desktop !== true && (account.canRefresh === true || account.hasRefreshToken === true) ? (
            <Button variant='ghost' size='sm' className={itemClass} disabled={account.needsRelogin === true}
              title={account.needsRelogin ? accountReloginText(account) : undefined}
              onClick={() => { setOpen(false); shared().wbApp?.runAccountAction?.('refresh', account.id) }}>
              {t('刷新 Token')}
            </Button>
          ) : null}
          <Button variant='ghost' size='sm' className={itemClass}
            title={t('将这个账号及其登录凭证保存为单独的 JSON 文件')}
            onClick={() => { setOpen(false); void exportAccountFiles({ id: account.id }) }}>
            {t('导出账号')}
          </Button>
          <div className='my-1 h-px bg-hairline' />
          <Button variant='ghost' size='sm' className={dangerClass}
            title={isDesktopAccount(account)
              ? t('删除这条账号记录（不会影响客户端自己的登录态；之后可再点「导入桌面端登录态」加回来）')
              : undefined}
            onClick={() => { setOpen(false); shared().wbApp?.runAccountAction?.('remove', account.id) }}>
            {t('删除账号')}
          </Button>
        </div>
      </PopoverContent>
    </Popover>
  )
}

/* ─── ──────────────────────────────────── */

/**
 * 。** JSX、、 not **： and by 
 * `wbColSettings.syncStaticHead` has （ is from DOM not is 
 * display:none）， not can by —— `<col>` up inline ，`<th>` 
 * ， will 1 。React in 「 1 、 1 」 up 
 * diff， this th props and not ， when 1 DOM not will 。
 *
 * for （ta-*） by syncStaticHead 1 （ by use config to ）， this not ——
 * will and 。： by visibleColumns() 。
 */
export function TableHead({ namesHidden, allPicked, somePicked, disabled, onToggleAll }: {
  namesHidden: boolean
  allPicked: boolean
  somePicked: boolean
  disabled: boolean
  onToggleAll: (picked: boolean) => void
}) {
  const grip = <span className='col-grip' title={t('拖动调整列宽（双击还原）')} />
  return (
    <TableHeader>
      <TableRow>
        <UiTableHead className='cell-pick' data-col='pick'>
          {/* this 「」 and that is ** 1 **（ is after ） */}
          <Checkbox id='acct-select-all' checked={allPicked} indeterminate={!allPicked && somePicked}
            disabled={disabled} aria-label={t('全选当前筛选结果')}
            title={t('全选 / 取消全选当前筛选结果（与批量栏同一个选择）')}
            onCheckedChange={next => onToggleAll(next)} />
        </UiTableHead>
        <UiTableHead className='cell-order' data-col='order'>
          <span className='th-label' title={t('组内先后：按住左边的把手上下拖动即可调整；整组的先后拖分组带的把手')}>
            {t('顺序')}
          </span>
          {grip}
        </UiTableHead>
        <UiTableHead className='cell-account' data-col='account'>
          <span className='th-label'>
            {t('账号')}
            <NameEyeButton hidden={namesHidden} />
          </span>
          {grip}
        </UiTableHead>
        <UiTableHead className='cell-proxy' data-col='proxy'>
          <span className='th-label' title={t('该账号出网走的代理（已保存的代理 / 自定义 / 直连）；点击可修改')}>{t('代理')}</span>
          {grip}
        </UiTableHead>
        <UiTableHead className='cell-connections' data-col='connections'>
          <span className='th-label' title={t('此刻正在使用这个账号的请求数（含还在下发内容的流式请求）；为 0 时不显示')}>{t('连接数')}</span>
          {grip}
        </UiTableHead>
        <UiTableHead className='cell-status' data-col='status'>
          <span className='th-label'>{t('状态')}</span>{grip}
        </UiTableHead>
        <UiTableHead className='cell-limits' data-col='limits'>
          <span className='th-label' title={t('该账号当前限流中的模型；点徽章看明细')}>
            {t('限流')}
          </span>
          {grip}
        </UiTableHead>
        <UiTableHead className='cell-expiry' data-col='expiry'>
          <span className='th-label'>{t('有效期')}</span>{grip}
        </UiTableHead>
        <UiTableHead className='cell-usage' data-col='usage'>
          <span className='th-label'>{t('余额')}</span>{grip}
        </UiTableHead>
        {/* after 1 not to ： for in ， in will 1 to */}
        <UiTableHead className='cell-actions' data-col='actions'><span className='th-label'>{t('操作')}</span></UiTableHead>
      </TableRow>
    </TableHeader>
  )
}

/**
 * 「」（ in ）： 1 down ，
 * / when not 。 / ， when ——
 * 「 in 」 not use can 。
 */
export function NameEyeButton({ hidden }: { hidden: boolean }) {
  const label = hidden ? t('显示账号名') : t('隐藏账号名（名字显示为星号）')
  return (
    <Button size='default' variant='ghost' type='button' className={`name-eye${hidden ? ' on' : ''}`} title={label} aria-label={label}
      aria-pressed={hidden} onClick={() => toggleNamesHidden()}
      dangerouslySetInnerHTML={{ __html: iconHtml(hidden ? 'eyeOff' : 'eye', 13) }} />
  )
}
