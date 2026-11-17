/**
 * 分组设置弹窗（分组带上齿轮那颗按钮打开的）。
 *
 * ── 为什么是弹窗，不是贴着齿轮的气泡 ──────────────────────────
 * 第一版只有「排序」一节，气泡装得下；但这一处以后还会有默认展开 / 收起、
 * 新账号落到组首还是组尾、本组并发上限、本组代理这些设置 —— 气泡既装不下
 * 几节内容，也没有「标题 + 可滚动正文 + 固定底栏」这套骨架可依。改成弹窗之后，
 * 加设置 = 往正文里追加一个 `GroupSettingSection`，骨架一行不用动。
 *
 * ── 排序规则的口径 ────────────────────────────────────────────
 * 规则**按数组顺序比较**，与界面上从上到下的优先级是同一个东西；`manual`
 * 不在这份数组里 —— 它是所有规则都相等时的兜底（保持拖出来的顺序），
 * 固定末位、不可勾选。见 accounts-domain 的 sortGroupMembers。
 *
 * 「积分到期时间」只对查得到逐份积分包的家开放（`supportsCreditExpiry`）：
 * 其余家那一行置灰成「不可用」而**不是隐藏** —— 藏起来会让人以为界面上少了
 * 一块，而置灰还能说明「这家确实没这个概念」。
 *
 * 消耗规则由网关保存，账号页和全局路由共用；改动影响下一次选路。
 */

import * as React from 'react'
import {
  Button, Checkbox, Dialog, DialogBody, DialogContent, DialogFooter, DialogHeader, DialogTitle,
  Select, SelectContent, SelectItem, SelectTrigger, SelectValue, cn,
} from '@ui'
import { t } from '../i18n'
import { shared, type AccountRecord, type AccountSorting } from './accounts-shared'
import {
  creditExpiryOf, providerFeatures, providerOf, supportsCreditExpiry, supportsUsage, type GroupSortRule,
} from './accounts-domain'
import {
  allAccounts, getStore, groupSortRules, hasCustomGroupSort, setGroupSortRules, subscribe,
  usageEntryOf,
} from './accounts-data'
import { useProviderIcon } from './provider-icon'
import { accountSortBalanceOf, balanceSortStateOf, type BalanceSortState } from '../../../ui-shared/account-sorting'

/**
 * 每条规则在界面上的名字与两档方向。
 *
 * 名字与说明写**字面量 t()** 而不是拼在表里：词典扫描器只认 `t('…')` 的首参是
 * 字符串字面量（变量 / 模板串一律不提取，见 i18n/scan.mjs 的说明），把它们塞进
 * 这张表就会在五种语言的词典里同时缺键、界面回落成中文。
 */
function ruleMeta(key: GroupSortRule['key']): {
  name: string
  hint: string
  options: Array<{ value: string; label: string }>
} {
  if (key === 'credit') {
    return {
      name: t('Credit expiry'),
      hint: t('Compares each account by its earliest credit pack that still has balance and has not expired'),
      options: [
        { value: 'soon', label: t('Soonest expiry first') },
        { value: 'late', label: t('Latest expiry first') },
      ],
    }
  }
  if (key === 'balance') {
    return {
      name: t('Available balance'),
      hint: t('Compare total available account balances without adding credit details again'),
      options: [
        { value: 'high', label: t('Higher balance first') },
        { value: 'low', label: t('Lower balance first') },
      ],
    }
  }
  return {
    name: t('Account added time'),
    hint: t('Every provider has this field'),
    options: [
      { value: 'new', label: t('Newest first') },
      { value: 'old', label: t('Oldest first') },
    ],
  }
}

export function supportsBalanceSort(provider: string, sorting?: AccountSorting | null): boolean {
  const snapshot = sorting || getStore().sorting
  if (snapshot?.balanceSortStates) return snapshot.balanceSortStates[provider] !== undefined
    && snapshot.balanceSortStates[provider] !== 'unsupported'
  if (snapshot) return false
  return providerFeatures(provider).usage || allAccounts().some(account => providerOf(account) === provider && supportsUsage(account))
}

export function activeGroupSortRules(provider: string, rules: GroupSortRule[], sorting?: AccountSorting | null): GroupSortRule[] {
  return rules.filter(rule => rule.on && (rule.key === 'credit' ? supportsCreditExpiry(provider)
    : rule.key === 'balance' ? supportsBalanceSort(provider, sorting) : true))
}

export function manualOrderBlockedMessage(rules: GroupSortRule[]): string {
  const names = rules.filter(rule => rule.on).map(rule => ruleMeta(rule.key).name)
  return names.length ? t('Cannot drag while automatic consumption rules are enabled ({rules}). Turn them off under Consumption order before reordering manually.', { rules: names.join(', ') })
    : t('Automatic consumption sorting is enabled. Turn off the rules before reordering manually')
}

/**
 * 规则摘要（分组带上那枚小标记、弹窗里的「当前规则」都用它）。
 * 只列**勾选的**那些 —— 关掉的规则不参与比较，写进摘要只会让「当前规则」说谎。
 */
export function groupSortSummary(rules: GroupSortRule[]): string {
  return rules
    .filter(rule => rule.on)
    .map(rule => {
      if (rule.key === 'credit') return rule.dir === 'soon' ? t('Expiry near→far') : t('Expiry far→near')
      if (rule.key === 'balance') return rule.dir === 'high' ? t('Higher balance first') : t('Lower balance first')
      return rule.dir === 'new' ? t('Added new→old') : t('Added old→new')
    })
    .join(' · ')
}

// 无读数时仍允许预设规则，之后查询成功会自动生效。
type RuleState = 'ready' | 'no-data' | 'mixed-units' | 'unsupported' | 'upgrade'

function ruleHint(rule: GroupSortRule, state: RuleState): string {
  if (state === 'upgrade') return t('This gateway does not support balance sorting. Update the gateway first.')
  if (rule.key === 'balance') {
    if (state === 'unsupported') return t('This provider does not support available balance queries')
    if (state === 'mixed-units') return t('Account balances use different units. Balance sorting is skipped; other rules still apply.')
    if (state === 'no-data') return t('No comparable balances yet. Sorting applies once numeric balances in the same unit are available; percentages and unlimited quotas are excluded.')
  }
  if (state === 'unsupported') return t('This provider has no credit expiry data')
  if (state === 'no-data') return rule.on
    ? t('No data yet: query the balance once and expiry sorting takes over automatically')
    : t('No data yet: query the balance once; enable expiry sorting if needed')
  return ruleMeta(rule.key).hint
}

function RuleRow({ rule, priority, isLast, state, busy, onChange }: {
  rule: GroupSortRule
  priority: number
  isLast: boolean
  state: RuleState
  busy: boolean
  onChange: (patch: Partial<GroupSortRule>) => void
}) {
  const meta = ruleMeta(rule.key)
  const blocked = state === 'unsupported' || state === 'upgrade'
  const hint = ruleHint(rule, state)
  const off = !blocked && !rule.on
  return (
    <div className={cn(
      'grid items-center gap-2 rounded-sm px-1.5 py-1.5',
      'grid-cols-[16px_15px_minmax(0,1fr)_132px_16px]',
      'hover:bg-nav-hover',
      off && 'opacity-[.58]',
    )}>
      <span className='text-center text-[10.5px] font-bold text-subtle tabular-nums'>{priority}</span>
      <Checkbox checked={rule.on && !blocked} disabled={blocked || busy}
        aria-label={meta.name}
        onCheckedChange={next => onChange({ on: next === true })} />
      <span className='flex min-w-0 flex-col gap-px'>
        <span className={cn('text-xs font-medium', blocked && 'text-subtle', off && 'text-subtle')}>
          {meta.name}
        </span>
        <span className={cn('text-[10.5px] leading-relaxed',
          blocked || state === 'mixed-units' ? 'text-warning' : 'text-subtle')} title={hint}>
          {hint}
        </span>
      </span>
      {!blocked ? (
        <Select value={rule.dir} disabled={!rule.on || busy}
          onValueChange={value => onChange({ dir: value } as Partial<GroupSortRule>)}>
          <SelectTrigger className='h-[26px] w-full text-[11px]' aria-label={t('Direction of {name}', { name: meta.name })}>
            <SelectValue>{meta.options.find(item => item.value === rule.dir)?.label || ''}</SelectValue>
          </SelectTrigger>
          <SelectContent>
            {meta.options.map(item => (
              <SelectItem key={item.value} value={item.value}>{item.label}</SelectItem>
            ))}
          </SelectContent>
        </Select>
      ) : (
        <span className='rounded-xs border border-dashed border-border py-1 text-center text-[10.5px] text-subtle'>
          {t('Unavailable')}
        </span>
      )}
      <span className='flex flex-col gap-px'>
        <Button size='default' variant='ghost' type='button' disabled={priority <= 1 || busy}
          className='h-3 w-4 rounded-[3px] text-subtle hover:bg-nav-hover hover:text-foreground disabled:cursor-default disabled:opacity-30'
          title={t('Raise priority')} aria-label={t('Raise priority')}
          onClick={() => onChange({ __move: 'up' } as unknown as Partial<GroupSortRule>)}>
          <svg viewBox='0 0 24 24' width='10' height='10' className='mx-auto block'>
            <path fill='none' stroke='currentColor' strokeWidth='2.4' strokeLinecap='round' strokeLinejoin='round' d='m6 14 6-6 6 6' />
          </svg>
        </Button>
        <Button size='default' variant='ghost' type='button' disabled={isLast || busy}
          className='h-3 w-4 rounded-[3px] text-subtle hover:bg-nav-hover hover:text-foreground disabled:cursor-default disabled:opacity-30'
          title={t('Lower priority')} aria-label={t('Lower priority')}
          onClick={() => onChange({ __move: 'down' } as unknown as Partial<GroupSortRule>)}>
          <svg viewBox='0 0 24 24' width='10' height='10' className='mx-auto block'>
            <path fill='none' stroke='currentColor' strokeWidth='2.4' strokeLinecap='round' strokeLinejoin='round' d='m6 10 6 6 6-6' />
          </svg>
        </Button>
      </span>
    </div>
  )
}

/** 正文里的一节：标题 + 一句口径 + 内容。后续加设置就再加一个它 */
function GroupSettingSection({ title, hint, children }: {
  title: string
  hint?: string
  children: React.ReactNode
}) {
  return (
    <section className='flex flex-col gap-1 border-t border-hairline pt-3.5 first:border-t-0 first:pt-0'>
      <div className='mb-1 flex items-baseline gap-2'>
        <h4 className='m-0 text-xs font-bold'>{title}</h4>
        {hint ? <span className='text-[10.5px] text-subtle'>{hint}</span> : null}
      </div>
      {children}
    </section>
  )
}

/**
 * 弹窗主体。`provider` 由 AccountsDialogs 从 store 的 dialog 状态传入；
 * 关闭时走 `closeDialog()`（列表页与弹窗共用同一个状态位）。
 */
export function GroupSettingsDialog({ provider, onClose, accounts, sorting, routeContext }: {
  provider: string; onClose: () => void; accounts?: AccountRecord[]; sorting?: AccountSorting | null; routeContext?: React.ReactNode
}) {
  // 订阅 store：规则一改就要重画（本弹窗是 store 的读者，不只是写者）
  React.useSyncExternalStore(subscribe, () => getStore().version)
  const rules = groupSortRules(provider)
  const label = shared().wbProviders?.labelOf?.(provider) || provider
  const { src: icon, onError: onIconError } = useProviderIcon(provider)
  /**
   * 方案 A：静态能力名单（这一家的后端**会不会**产出逐份到期）。
   * 方案 B：运行时是否**真的**读到了 —— 名单里的家如果这一组所有账号都还没查到
   * 到期（没点过「查询余额」、或者都过期了），这一项不该假装可用。
   */
  const creditSupported = supportsCreditExpiry(provider)
  const version = getStore().version
  const sortSnapshot = sorting || getStore().sorting
  const members = (accounts || allAccounts()).filter(account => providerOf(account) === provider)
  const creditHasData = React.useMemo(
    () => members.some(account => sortSnapshot
      ? sortSnapshot.creditExpiresAt[account.id] != null
      : creditExpiryOf(usageEntryOf(account)) !== null),
    [provider, version, accounts, sorting, sortSnapshot],
  )
  const creditState: RuleState = !creditSupported ? 'unsupported' : creditHasData ? 'ready' : 'no-data'
  const balanceState: BalanceSortState = sortSnapshot?.balanceSortStates?.[provider]
    || (supportsBalanceSort(provider, sortSnapshot)
      ? balanceSortStateOf(members.map(account => accountSortBalanceOf(usageEntryOf(account)))) : 'unsupported')
  const balanceRuleState: RuleState = sortSnapshot && !sortSnapshot.balanceSortStates ? 'upgrade'
    : balanceState === 'mixedUnits' ? 'mixed-units' : balanceState === 'noData' ? 'no-data' : balanceState
  const activeRules = activeGroupSortRules(provider, rules, sortSnapshot)
  const summary = groupSortSummary(activeRules)
  const balanceEnabled = activeRules.some(rule => rule.key === 'balance')
  const manualLocked = activeRules.length > 0
  const customized = hasCustomGroupSort(provider)
  const busy = getStore().sortBusy

  /** 改一条规则：勾选 / 方向直接落库；上移下移改的是数组里的位置 */
  const changeRule = (key: GroupSortRule['key'], patch: Partial<GroupSortRule>) => {
    const op = (patch as unknown as { __move?: 'up' | 'down' }).__move
    const next = rules.map(rule => ({ ...rule }))
    const index = next.findIndex(rule => rule.key === key)
    if (index < 0) return
    if (op) {
      // 上移 / 下移改的是**优先级**（数组位置）；手动顺序是兜底、不在数组里，
      // 所以数组末尾就是它前面 —— 下移的边界是 next.length - 1
      const to = op === 'up' ? index - 1 : index + 1
      if (to < 0 || to >= next.length) return
      const [moved] = next.splice(index, 1)
      next.splice(to, 0, moved)
    } else {
      next[index] = { ...next[index], ...patch } as GroupSortRule
    }
    void setGroupSortRules(provider, next)
  }

  return (
    <Dialog open onOpenChange={next => { if (!next && !busy) onClose() }}>
      <DialogContent className='w-[min(560px,calc(100vw-48px))]' showCloseButton>
        <DialogHeader>
          <span className='inline-flex size-[26px] flex-none items-center justify-center'>
            {icon
              ? <img src={icon} alt='' className='size-[26px] rounded-md object-contain' loading='lazy' onError={onIconError} />
              : <span className='inline-flex size-[26px] items-center justify-center rounded-md bg-surface-3 text-[13px] font-bold text-muted-foreground'>
                  {String(label || '?').trim().slice(0, 1).toUpperCase()}
                </span>}
          </span>
          <DialogTitle>{t('{name} · Group settings', { name: t(label) })}</DialogTitle>
        </DialogHeader>

        <DialogBody>
          {routeContext}
          <GroupSettingSection title={t('Consumption order')} hint={t('Compared top to bottom; only checked rules take part')}>
            {rules.map((rule, index) => (
              <RuleRow
                key={rule.key}
                rule={rule}
                priority={index + 1}
                isLast={index === rules.length - 1}
                state={rule.key === 'credit' ? creditState : rule.key === 'balance' ? balanceRuleState : 'ready'}
                busy={busy}
                onChange={patch => changeRule(rule.key, patch)}
              />
            ))}
            {/* 手动顺序：固定末位、不可勾选 —— 它是兜底而不是一条规则 */}
            <div className='grid grid-cols-[16px_15px_minmax(0,1fr)] items-center gap-2 px-1.5 py-1.5'>
              <span className='text-center text-[10.5px] font-bold text-subtle tabular-nums'>{rules.length + 1}</span>
              <span className='inline-flex items-center justify-center text-subtle' title={t('Fixed fallback, not affected by the toggles')}>
                <svg viewBox='0 0 24 24' width='12' height='12'>
                  <g fill='none' stroke='currentColor' strokeWidth='2' strokeLinecap='round' strokeLinejoin='round'>
                    <rect x='4.5' y='10' width='15' height='10.5' rx='2' />
                    <path d='M8 10V7a4 4 0 0 1 8 0v3' />
                  </g>
                </svg>
              </span>
              <span className='flex min-w-0 flex-col gap-px'>
                <span className='text-xs font-medium'>{t('Manual order within the group')}</span>
                <span className='text-[10.5px] text-subtle'>{t('When every rule above ties, keep the order you dragged')}</span>
              </span>
            </div>
            <p className='m-0 mt-2 rounded-xs bg-surface-inset px-2.5 py-2 text-[10.5px] leading-relaxed text-subtle'>
              {t('Current rules: {rules}.', { rules: summary || t('Manual order only') })}{' '}
              {manualLocked ? t('Automatic sorting is enabled and dragging within the group is locked. Turn off all rules to restore the original manual order.') : t('You can currently reorder accounts in this group manually.')}
            </p>
            <p className='m-0 mt-1.5 px-2.5 text-[10.5px] leading-relaxed text-subtle'>
              {t('Each rule is used only when the preceding rules tie. Move Available balance to the top to prioritize consumption by balance.')}
            </p>
            {balanceEnabled ? <p className='m-0 mt-1.5 px-2.5 text-[10.5px] leading-relaxed text-subtle'>
              {t('Uses the total balance from the last successful query and updates as balances refresh. Failed queries retain the previous reading; query frequency is unchanged.')}{' '}
              {t('Only numeric balances from the same provider and in the same unit are compared. Unknown, percentage and unlimited quotas are not treated as zero; accounts with known balances come first.')}
            </p> : null}
            <p className='m-0 mt-1.5 px-2.5 text-[10.5px] leading-relaxed text-subtle'>
              {t('Accounts and global routing share this consumption order. The gateway routes using the latest balance data, even when the page is closed. Independent model order overrides the global order.')}
            </p>
            <p className='m-0 mt-1.5 px-2.5 text-[10.5px] leading-relaxed text-subtle'>
              {t('Select Priority order to use accounts strictly in consumption order. Rotation, session stickiness and account protection keep their existing settings.')}
            </p>
            <p className='m-0 mt-1.5 px-2.5 text-[10.5px] leading-relaxed text-subtle'>
              {creditSupported
                ? t('Only earliest expiry first is enabled by default. Available balance and account creation time sorting are off.')
                : t('Manual order is the default. Enable supported automatic sorting rules as needed.')}
            </p>
          </GroupSettingSection>

          {/* 后续设置的位置：先把「这里还会有东西」说清楚，加小节时直接替换这一块 */}
          <GroupSettingSection title={t('More settings')} hint={t('Coming later')}>
            <div className='flex flex-wrap gap-1.5'>
              {[t('Default expanded / collapsed'), t('New accounts go to the top / bottom'), t('Group concurrency limit'), t('Warn when balance drops below a threshold'), t('Proxy for this group')]
                .map(item => (
                  <span key={item}
                    className='inline-flex h-[22px] items-center rounded-pill border border-dashed border-border px-2.5 text-[10.5px] text-subtle'>
                    {item}
                  </span>
                ))}
            </div>
          </GroupSettingSection>
        </DialogBody>

        <DialogFooter>
          <Button variant='ghost' size='sm' disabled={!customized || busy}
            title={customized
              ? creditSupported ? t('Restore defaults: earliest expiry first only') : t('Restore defaults: manual order only')
              : t('This is already the default rule set')}
            onClick={() => void setGroupSortRules(provider, null)}>
            {t('Restore Defaults')}
          </Button>
          <span className='mr-auto text-[10.5px] text-subtle'>{busy ? t('Saving…') : t('Changes apply immediately')}</span>
          <Button variant='default' disabled={busy} onClick={onClose}>{t('Done')}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
