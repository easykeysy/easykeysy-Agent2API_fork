/**
 * set（ up that by ）。
 *
 * ── Why is ， not is ──────────────────────────
 * 1 has 「」 1 ， down ； this 1 after will has default / 、
 * new to is 、 and limit、 this set —— not down 
 * ， no has 「 + can + 」 this can 。 after ，
 * set = 1 `GroupSettingSection`， 1 not use 。
 *
 * ── then ────────────────────────────────────────────
 * then ** by **， and up from up to down is 1 ；`manual`
 * not in this —— is all then etc when （ ），
 * 、 not can 。 accounts-domain sortGroupMembers。
 *
 * 「 to when 」 for to （`supportsCreditExpiry`）：
 * its that 1 「unavailable」** not is ** —— will as up 
 * 1 ， can 「 this no this 」。
 *
 * then by ， and by use ； down 1 。
 */

import * as React from 'react'
import {
  Button, Checkbox, Dialog, DialogBody, DialogContent, DialogFooter, DialogHeader, DialogTitle,
  Select, SelectContent, SelectItem, SelectTrigger, SelectValue, cn,
} from '@ui'
import { t } from '../i18n'
import { shared, type AccountRecord, type AccountSorting } from './accounts-shared'
import {
  creditExpiryOf, providerOf, supportsCreditExpiry, type GroupSortRule,
} from './accounts-domain'
import {
  allAccounts, getStore, groupSortRules, hasCustomGroupSort, setGroupSortRules, subscribe,
  usageEntryOf,
} from './accounts-data'
import { PROVIDER_ICONS } from './add-provider-pick'

/**
 * then in up and to 。
 *
 * and ** t()** not is in ： `t('…')` is 
 * （ / 1 not ， i18n/scan.mjs ）， 
 * this will in when 、 in 。
 */
function ruleMeta(key: GroupSortRule['key']): {
  name: string
  hint: string
  options: Array<{ value: string; label: string }>
} {
  if (key === 'credit') {
    return {
      name: t('积分到期时间'),
      hint: t('按账号最早到期、且还有剩余的积分包比较'),
      options: [
        { value: 'soon', label: t('最快到期优先') },
        { value: 'late', label: t('最晚到期优先') },
      ],
    }
  }
  return {
    name: t('账号添加时间'),
    hint: t('所有提供商都有这个字段'),
    options: [
      { value: 'new', label: t('最新添加优先') },
      { value: 'old', label: t('最早添加优先') },
    ],
  }
}

export function activeGroupSortRules(provider: string, rules: GroupSortRule[]): GroupSortRule[] {
  return rules.filter(rule => rule.on && (rule.key !== 'credit' || supportsCreditExpiry(provider)))
}

export function manualOrderBlockedMessage(rules: GroupSortRule[]): string {
  const credit = rules.some(rule => rule.on && rule.key === 'credit')
  const added = rules.some(rule => rule.on && rule.key === 'added')
  if (credit && added) return t('无法拖动：当前已按积分到期时间和账号添加时间自动排序。请先在「消耗顺序」中关闭这两项规则，再手动调整。')
  if (credit) return t('无法拖动：当前已按积分到期时间自动排序。请先在「消耗顺序」中关闭该规则，再手动调整。')
  if (added) return t('无法拖动：当前已按账号添加时间自动排序。请先在「消耗顺序」中关闭该规则，再手动调整。')
  return t('自动消耗排序已启用，请先关闭规则再手动调整')
}

/**
 * then need （ up that 、 「current then 」 use ）。
 * ** ** that —— then not and ， need will 「current then 」。
 */
export function groupSortSummary(rules: GroupSortRule[]): string {
  return rules
    .filter(rule => rule.on)
    .map(rule => {
      if (rule.key === 'credit') return rule.dir === 'soon' ? t('到期 近→远') : t('到期 远→近')
      return rule.dir === 'new' ? t('添加 新→旧') : t('添加 旧→新')
    })
    .join(' · ')
}

/**
 * 1 then ： / / and / to / up down 。
 *
 * `state` has ， and can not ：
 * · `ready` —— available（ can 、 can to ）
 * · `no-data` —— this 1 after will to ，** this 1 1 no to **
 * （ no 「」， or ）。 can —— use 
 * then 、 after to ；「none」。
 * · `unsupported` —— this 1 no has to 。 use 、 to 
 * 「unavailable」， not （ is 「 this no 」 ）。
 */
function RuleRow({ rule, priority, isLast, state, busy, onChange }: {
  rule: GroupSortRule
  priority: number
  isLast: boolean
  state: 'ready' | 'no-data' | 'unsupported'
  busy: boolean
  onChange: (patch: Partial<GroupSortRule>) => void
}) {
  const meta = ruleMeta(rule.key)
  const blocked = state === 'unsupported'
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
        <span className={cn(
          'overflow-hidden text-ellipsis whitespace-nowrap text-[10.5px]',
          blocked ? 'text-warning' : state === 'no-data' ? 'text-subtle' : 'text-subtle',
        )} title={blocked ? t('这一家查不到积分到期时间') : undefined}>
          {blocked
            ? t('这一家查不到积分到期时间')
            : state === 'no-data'
              ? rule.on
                ? t('暂无数据：先查询一次余额，之后会自动按到期排序')
                : t('暂无数据：先查询一次余额，可按需启用到期排序')
              : meta.hint}
        </span>
      </span>
      {!blocked ? (
        <Select value={rule.dir} disabled={!rule.on || busy}
          onValueChange={value => onChange({ dir: value } as Partial<GroupSortRule>)}>
          <SelectTrigger className='h-[26px] w-full text-[11px]' aria-label={t('{name}的方向', { name: meta.name })}>
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
          {t('不可用')}
        </span>
      )}
      <span className='flex flex-col gap-px'>
        <Button size='default' variant='ghost' type='button' disabled={priority <= 1 || busy}
          className='h-3 w-4 rounded-[3px] text-subtle hover:bg-nav-hover hover:text-foreground disabled:cursor-default disabled:opacity-30'
          title={t('提高优先级')} aria-label={t('提高优先级')}
          onClick={() => onChange({ __move: 'up' } as unknown as Partial<GroupSortRule>)}>
          <svg viewBox='0 0 24 24' width='10' height='10' className='mx-auto block'>
            <path fill='none' stroke='currentColor' strokeWidth='2.4' strokeLinecap='round' strokeLinejoin='round' d='m6 14 6-6 6 6' />
          </svg>
        </Button>
        <Button size='default' variant='ghost' type='button' disabled={isLast || busy}
          className='h-3 w-4 rounded-[3px] text-subtle hover:bg-nav-hover hover:text-foreground disabled:cursor-default disabled:opacity-30'
          title={t('降低优先级')} aria-label={t('降低优先级')}
          onClick={() => onChange({ __move: 'down' } as unknown as Partial<GroupSortRule>)}>
          <svg viewBox='0 0 24 24' width='10' height='10' className='mx-auto block'>
            <path fill='none' stroke='currentColor' strokeWidth='2.4' strokeLinecap='round' strokeLinejoin='round' d='m6 10 6 6 6-6' />
          </svg>
        </Button>
      </span>
    </div>
  )
}

/** 1 ： + 1 + 。 after set 1 */
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
 * 。`provider` by AccountsDialogs from store dialog ；
 * when `closeDialog()`（ and use 1 ）。
 */
export function GroupSettingsDialog({ provider, onClose, accounts, sorting, routeContext }: {
  provider: string; onClose: () => void; accounts?: AccountRecord[]; sorting?: AccountSorting | null; routeContext?: React.ReactNode
}) {
  // store： then 1 need （ is store ， not is ）
  React.useSyncExternalStore(subscribe, () => getStore().version)
  const rules = groupSortRules(provider)
  const label = shared().wbProviders?.labelOf?.(provider) || provider
  const icon = PROVIDER_ICONS[provider]
  /**
   * A： can （ this 1 after ** will not will ** to ）。
   * B： when is ** ** to —— if this 1 all no to 
   * to （ no 「」、or）， this 1 not this available。
   */
  const creditSupported = supportsCreditExpiry(provider)
  // store version not is empty：（usageEntryOf has new ） after 
  // this 1 need from 「none」available， when 
  const version = getStore().version
  const creditHasData = React.useMemo(
    () => (accounts || allAccounts())
      .filter(account => providerOf(account) === provider)
      .some(account => sorting || getStore().sorting
        ? (sorting || getStore().sorting)?.creditExpiresAt[account.id] != null
        : creditExpiryOf(usageEntryOf(account)) !== null),
    [provider, version, accounts, sorting],
  )
  const creditState: 'ready' | 'no-data' | 'unsupported' =
    !creditSupported ? 'unsupported' : creditHasData ? 'ready' : 'no-data'
  const summary = groupSortSummary(rules)
  const customized = hasCustomGroupSort(provider)
  const busy = getStore().sortBusy

  /** 1 then ： / to ； up down is */
  const changeRule = (key: GroupSortRule['key'], patch: Partial<GroupSortRule>) => {
    const op = (patch as unknown as { __move?: 'up' | 'down' }).__move
    const next = rules.map(rule => ({ ...rule }))
    const index = next.findIndex(rule => rule.key === key)
    if (index < 0) return
    if (op) {
      // up / down is ****（）； is 、 not in ，
      // is before —— down is next.length - 1
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
              ? <img src={icon} alt='' className='size-[26px] rounded-md object-contain' loading='lazy' />
              : <span className='inline-flex size-[26px] items-center justify-center rounded-md bg-surface-3 text-[13px] font-bold text-muted-foreground'>
                  {String(label || '?').trim().slice(0, 1).toUpperCase()}
                </span>}
          </span>
          <DialogTitle>{t('{name} · 分组设置', { name: t(label) })}</DialogTitle>
        </DialogHeader>

        <DialogBody>
          {routeContext}
          <GroupSettingSection title={t('消耗顺序')} hint={t('按优先级从上到下比较，勾选的项才参与')}>
            {rules.map((rule, index) => (
              <RuleRow
                key={rule.key}
                rule={rule}
                priority={index + 1}
                isLast={index === rules.length - 1}
                state={rule.key === 'credit' ? creditState : 'ready'}
                busy={busy}
                onChange={patch => changeRule(rule.key, patch)}
              />
            ))}
            {/* ：、 not can —— is not is 1 then */}
            <div className='grid grid-cols-[16px_15px_minmax(0,1fr)] items-center gap-2 px-1.5 py-1.5'>
              <span className='text-center text-[10.5px] font-bold text-subtle tabular-nums'>{rules.length + 1}</span>
              <span className='inline-flex items-center justify-center text-subtle' title={t('固定兜底，不受开关影响')}>
                <svg viewBox='0 0 24 24' width='12' height='12'>
                  <g fill='none' stroke='currentColor' strokeWidth='2' strokeLinecap='round' strokeLinejoin='round'>
                    <rect x='4.5' y='10' width='15' height='10.5' rx='2' />
                    <path d='M8 10V7a4 4 0 0 1 8 0v3' />
                  </g>
                </svg>
              </span>
              <span className='flex min-w-0 flex-col gap-px'>
                <span className='text-xs font-medium'>{t('组内手动顺序')}</span>
                <span className='text-[10.5px] text-subtle'>{t('上面所有规则都相同时，保持你拖出来的顺序')}</span>
              </span>
            </div>
            <p className='m-0 mt-2 rounded-xs bg-surface-inset px-2.5 py-2 text-[10.5px] leading-relaxed text-subtle'>
              {t('当前规则：{rules}。自动排序生效时，组内拖动把手会锁定；关掉全部规则即回到手动顺序。', {
                rules: summary || t('仅按手动顺序'),
              })}
            </p>
            <p className='m-0 mt-1.5 px-2.5 text-[10.5px] leading-relaxed text-subtle'>
              {t('账号页与全局路由共用此消耗顺序，网关按最新余额数据选路，不需要保持页面打开。模型独立顺序会覆盖全局。')}
            </p>
            <p className='m-0 mt-1.5 px-2.5 text-[10.5px] leading-relaxed text-subtle'>
              {creditSupported
                ? t('本提供商默认启用最快到期优先，账号添加时间默认关闭；关闭全部规则可恢复手动顺序。')
                : t('本提供商不支持积分到期排序，默认保持手动顺序；可按需启用账号添加时间排序。')}
            </p>
          </GroupSettingSection>

          {/* after set ： 「 this will has 」， when this 1 */}
          <GroupSettingSection title={t('更多设置')} hint={t('后续加入')}>
            <div className='flex flex-wrap gap-1.5'>
              {[t('默认展开 / 收起'), t('新账号落到组首 / 组尾'), t('组内并发上限'), t('余额低于阈值时的提示'), t('本组代理')]
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
              ? creditSupported ? t('恢复默认：最快到期优先，账号添加时间关闭') : t('恢复默认：仅按手动顺序')
              : t('当前已经是默认规则')}
            onClick={() => void setGroupSortRules(provider, null)}>
            {t('恢复默认')}
          </Button>
          <span className='mr-auto text-[10.5px] text-subtle'>{busy ? t('保存中…') : t('改动即时生效')}</span>
          <Button variant='default' disabled={busy} onClick={onClose}>{t('完成')}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
