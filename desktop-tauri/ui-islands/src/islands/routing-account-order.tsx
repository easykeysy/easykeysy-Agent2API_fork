import * as React from 'react'
import { Button } from '@ui'
import { t } from '../i18n'
import { CREDIT_EXPIRY_WINDOW_MS, creditExpiryLabel } from '../../../ui-shared/credit-expiry'
import type { AccountSorting, GroupSortRule } from './accounts-shared'
import { saveGroupSortRules } from './accounts-sort'
import { RoutingSelect } from './routing-select'
import { supportsCreditExpiry } from './accounts-domain'
import { activeGroupSortRules, groupSortSummary, supportsBalanceSort } from './accounts-group-settings'
import { RouteIcon } from './routing-ui'

export function RoutingCreditExpiry({ id, provider, sorting, now }: {
  id: string; provider: string; sorting?: AccountSorting | null; now: number
}) {
  if (!supportsCreditExpiry(provider)) return null
  const expiry = sorting?.creditExpiresAt[id]
  const text = expiry && expiry > now ? creditExpiryLabel(expiry, t, now) : ''
  return <small className='routing-credit-expiry' data-soon={text && expiry! - now <= CREDIT_EXPIRY_WINDOW_MS ? '' : undefined}
    title={text ? t('Earliest-expiring credit pack with a remaining balance: {time}', { time: new Date(expiry!).toLocaleString() }) : t('No valid credit expiry time has been loaded yet')}>
    {text ? t('Credits: {time}', { time: text }) : t('Credit expiry: no data')}
  </small>
}

export function RoutingConsumptionOrder({ provider, sorting, independent, disabled, onSettings }: {
  provider: string; sorting?: AccountSorting | null; independent: boolean; disabled: boolean; onSettings(): void
}) {
  const [saving, setSaving] = React.useState(false)
  const pending = React.useRef(false)
  const configured = sorting?.rules[provider] || []
  const rules = activeGroupSortRules(provider, configured, sorting)
  const credit = rules.find(rule => rule.key === 'credit')
  const balance = rules.find(rule => rule.key === 'balance')
  const balanceSupported = supportsBalanceSort(provider, sorting)
  const summary = independent ? t('Independent account order') : groupSortSummary(rules) || t('Manual order only')
  const changeRule = async (key: 'credit' | 'balance', value: string) => {
    if (disabled || pending.current || independent) return
    pending.current = true; setSaving(true)
    try {
      const rule: GroupSortRule = key === 'credit'
        ? { key, on: value !== 'off', dir: value === 'late' ? 'late' : 'soon' }
        : { key, on: value !== 'off', dir: value === 'low' ? 'low' : 'high' }
      const next = configured.map(item => item.key === key ? rule : { ...item })
      if (!next.some(item => item.key === key)) next.push(rule)
      await saveGroupSortRules(provider, next)
    } finally { pending.current = false; setSaving(false) }
  }
  return <div className='routing-consumption-order'>
    <RouteIcon name='sort' size={13} /><span>{t('消耗顺序')}</span>
    {!independent && supportsCreditExpiry(provider) ? <div className='routing-credit-sort'>
      <RoutingSelect label={t('Credit expiry')} value={credit?.dir || 'off'} disabled={disabled || saving} onChange={value => void changeRule('credit', value)}
        options={[[ 'off', t('Do not sort by expiry') ], [ 'soon', t('Expiry near→far') ], [ 'late', t('Expiry far→near') ]]} />
    </div> : null}
    {!independent && balanceSupported ? <div className='routing-credit-sort'>
      <RoutingSelect label={t('Available balance')} value={balance?.dir || 'off'} disabled={disabled || saving} onChange={value => void changeRule('balance', value)}
        options={[[ 'off', t('Do not sort by balance') ], [ 'high', t('Higher balance first') ], [ 'low', t('Lower balance first') ]]} />
    </div> : null}
    <Button variant='ghost' size='sm' disabled={disabled || saving} onClick={onSettings}
      title={independent ? t('The account order in an independent route is not affected by global consumption rules') : t('Adjust the priority of expiry, available balance and other consumption rules')}>
      {!independent && (supportsCreditExpiry(provider) || balanceSupported) ? t('Consumption rules') : summary}<RouteIcon name='chevronRight' size={11} />
    </Button>
    {!independent && rules.length ? <small className='basis-full leading-relaxed'>{t('Priority: {rules}. Later rules apply only when earlier rules tie.', { rules: summary })}</small> : null}
    {!independent && balance && sorting?.balanceSortStates?.[provider] === 'mixedUnits'
      ? <small className='basis-full text-warning'>{t('Account balances use different units. Balance sorting is skipped; other rules still apply.')}</small> : null}
    {!independent && balance && sorting?.balanceSortStates?.[provider] === 'noData'
      ? <small className='basis-full'>{t('No comparable balances yet. Sorting applies once numeric balances in the same unit are available; percentages and unlimited quotas are excluded.')}</small> : null}
  </div>
}
