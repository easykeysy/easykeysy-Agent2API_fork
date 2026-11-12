import { Sheet, SheetContent, SheetHeader, SheetBody, SheetFooter, SheetTitle , Button } from '@ui'
import { Icon } from './Icon'
import { ProviderLogo } from './ProviderLogo'
import { providerLabel } from '../lib/providers'
import { t } from '../i18n'
import { Toggle as UiToggle } from '@ui'

/**
 * （ up that 「」 by ）。
 *
 * ── Why from chip ────────────────────────────
 * has 「 / enabled / / use 」 chip。 down ，
 * need not this 1 ：、valid、 is 1 。
 * will 1 need —— up ， and 
 * use not 「 in to 」。
 *
 * 1 、（），「」 and 「」。
 * ** when **： can to ， not need need use —— up 
 * 1 will no for 。
 *
 * ── Why is not is ────────────────────────────────────
 * will down ， is 「」 。
 * 340px（ 86vw）， can to in 。
 */

export type StatusFilter = 'all' | 'enabled' | 'disabled' | 'limited' | 'relogin' | 'expired'
export type ExpiryFilter = 'all' | 'soon' | 'expired' | 'longlived'
export type UsageFilter = 'all' | 'has' | 'unqueried' | 'failed'

export type AccountFilters = {
  status: StatusFilter
  /** in id；empty = unlimited */
  providers: string[]
  expiry: ExpiryFilter
  usage: UsageFilter
}

export const DEFAULT_FILTERS: AccountFilters = {
  status: 'all',
  providers: [],
  expiry: 'all',
  usage: 'all',
}

/** in （ by up ； by 1 ） */
export function activeFilterCount(filters: AccountFilters): number {
  let count = 0
  if (filters.status !== 'all') count++
  if (filters.providers.length) count++
  if (filters.expiry !== 'all') count++
  if (filters.usage !== 'all') count++
  return count
}

export function AccountsFilterSheet({
  open,
  onClose,
  filters,
  onChange,
  providers,
  counts,
}: {
  open: boolean
  onClose: () => void
  filters: AccountFilters
  onChange: (next: AccountFilters) => void
  /** current （id + ） */
  providers: Array<{ id: string; label: string }>
  /** ， in */
  counts: {
    status: Record<StatusFilter, number>
    expiry: Record<ExpiryFilter, number>
    usage: Record<UsageFilter, number>
  }
}) {
  const patch = (next: Partial<AccountFilters>) => onChange({ ...filters, ...next })

  const toggleProvider = (id: string) => {
    const picked = filters.providers.includes(id)
      ? filters.providers.filter(item => item !== id)
      : [...filters.providers, id]
    patch({ providers: picked })
  }

  return (
    <Sheet open={open} onOpenChange={next => { if (!next) onClose() }}>
      <SheetContent side='right'>
        <SheetHeader>
          <SheetTitle>{t('筛选')}</SheetTitle>
        </SheetHeader>
        <SheetBody>
          <Section title={t('状态')}>
            <Chip
              label={t('全部')}
              count={counts.status.all}
              on={filters.status === 'all'}
              onClick={() => patch({ status: 'all' })}
            />
            <Chip
              label={t('启用')}
              count={counts.status.enabled}
              on={filters.status === 'enabled'}
              onClick={() => patch({ status: 'enabled' })}
            />
            <Chip
              label={t('禁用')}
              count={counts.status.disabled}
              on={filters.status === 'disabled'}
              onClick={() => patch({ status: 'disabled' })}
            />
            <Chip
              label={t('限流中')}
              count={counts.status.limited}
              on={filters.status === 'limited'}
              onClick={() => patch({ status: 'limited' })}
            />
            <Chip
              label={t('需重登')}
              count={counts.status.relogin}
              on={filters.status === 'relogin'}
              onClick={() => patch({ status: 'relogin' })}
            />
            <Chip
              label={t('已过期')}
              count={counts.status.expired}
              on={filters.status === 'expired'}
              onClick={() => patch({ status: 'expired' })}
            />
          </Section>

          {providers.length > 1 ? (
            <Section title={t('提供商')}>
              {providers.map(item => (
                <UiToggle size='default' variant='outline'
                  key={item.id}
                  type='button'
                  className='m-chip rounded-pill'
                  pressed={filters.providers.includes(item.id)}
                  onClick={() => toggleProvider(item.id)}
                  style={{ gap: 6 }}
                >
                  <ProviderLogo id={item.id} label={item.label} />
                  {item.label}
                </UiToggle>
              ))}
            </Section>
          ) : null}

          <Section title={t('有效期')}>
            <Chip
              label={t('全部')}
              count={counts.expiry.all}
              on={filters.expiry === 'all'}
              onClick={() => patch({ expiry: 'all' })}
            />
            <Chip
              label={t('7 天内到期')}
              count={counts.expiry.soon}
              on={filters.expiry === 'soon'}
              onClick={() => patch({ expiry: 'soon' })}
            />
            <Chip
              label={t('已过期')}
              count={counts.expiry.expired}
              on={filters.expiry === 'expired'}
              onClick={() => patch({ expiry: 'expired' })}
            />
            <Chip
              label={t('长期有效')}
              count={counts.expiry.longlived}
              on={filters.expiry === 'longlived'}
              onClick={() => patch({ expiry: 'longlived' })}
            />
          </Section>

          <Section title={t('余额')}>
            <Chip
              label={t('全部')}
              count={counts.usage.all}
              on={filters.usage === 'all'}
              onClick={() => patch({ usage: 'all' })}
            />
            <Chip
              label={t('已查询')}
              count={counts.usage.has}
              on={filters.usage === 'has'}
              onClick={() => patch({ usage: 'has' })}
            />
            <Chip
              label={t('未查询')}
              count={counts.usage.unqueried}
              on={filters.usage === 'unqueried'}
              onClick={() => patch({ usage: 'unqueried' })}
            />
            <Chip
              label={t('查询失败')}
              count={counts.usage.failed}
              on={filters.usage === 'failed'}
              onClick={() => patch({ usage: 'failed' })}
            />
          </Section>

          <p style={{ margin: 0, fontSize: 11.5, lineHeight: 1.7, color: 'var(--m-text-3)' }}>
            {t('条件即时生效，关掉面板就能看到结果。')}
          </p>
        </SheetBody>
        <SheetFooter>
          <Button size='touch' variant='outline'
            type='button'
            className='m-btn m-ghost w-full'
            style={{ flex: 1 }}
            onClick={() => onChange(DEFAULT_FILTERS)}
          >
            <Icon name='refresh' />
            {t('重置')}
          </Button>
          <Button size='touch' variant='default' type='button' className='m-btn w-full' style={{ flex: 1 }} onClick={onClose}>
            {t('完成')}
          </Button>
        </SheetFooter>
      </SheetContent>
    </Sheet>
  )
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div>
      <div style={{ fontSize: 12, fontWeight: 600, color: 'var(--m-text-2)', marginBottom: 8 }}>
        {title}
      </div>
      <div style={{ display: 'flex', flexWrap: 'wrap', gap: 7 }}>{children}</div>
    </div>
  )
}

function Chip({
  label,
  count,
  on,
  onClick,
}: {
  label: string
  count: number
  on: boolean
  onClick: () => void
}) {
  return (
    <UiToggle size='default' variant='outline' type='button' className='m-chip rounded-pill' pressed={on} onClick={onClick}>
      {label}
      <span className='m-n'>{count}</span>
    </UiToggle>
  )
}
