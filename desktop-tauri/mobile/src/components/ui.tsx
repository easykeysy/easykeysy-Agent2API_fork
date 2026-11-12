import { Icon, type IconName } from './Icon'
import { t } from '../i18n'
import { store } from '../state/store'
import { Alert, Badge as UiBadge, BadgeDot, Button, Empty as UiEmpty, PageHeader, PageTitle, PageDescription, Skeleton, Card } from '@ui'

/** use （ 1 ）： + + */
export function PageHead({
  title,
  subtitle,
  actions,
}: {
  title: string
  subtitle?: string
  actions?: React.ReactNode
}) {
  return (
    <PageHeader className='m-head'>
      <div style={{ minWidth: 0 }}>
        <PageTitle className='text-[26px]'>{title}</PageTitle>
        {subtitle ? <PageDescription className='m-sub'>{subtitle}</PageDescription> : null}
      </div>
      {actions ? <div className='m-head-actions'>{actions}</div> : null}
    </PageHeader>
  )
}

/**
 * 。
 *
 * already by return（NavBar）， 1 is 。
 * （statistics / when ） has ， 1 。 this ，
 * as not is 。
 */
export function PageIntro({
  subtitle,
  actions,
}: {
  subtitle?: string
  actions?: React.ReactNode
}) {
  if (!subtitle && !actions) return null
  return (
    <div
      style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        gap: 10,
        padding: '2px 18px 12px',
      }}
    >
      <span style={{ fontSize: 12.5, color: 'var(--m-text-3)', minWidth: 0 }}>{subtitle}</span>
      {actions ? <div style={{ display: 'flex', gap: 8, flex: 'none' }}>{actions}</div> : null}
    </div>
  )
}

/** by （ / return use ） */
export function IconButton({
  name,
  label,
  onClick,
  primary,
}: {
  name: IconName
  label: string
  onClick?: () => void
  primary?: boolean
}) {
  return (
    <Button size='icon-touch' variant='outline'
      type='button'
      className={primary ? 'm-icon-btn m-primary' : 'm-icon-btn'}
      aria-label={label}
      title={label}
      onClick={onClick}
    >
      <Icon name={name} />
    </Button>
  )
}

/** ： + optional */
export function SectionHead({
  title,
  action,
}: {
  title: string
  action?: { label: string; onClick: () => void }
}) {
  return (
    <div className='m-section-head'>
      <h2>{title}</h2>
      {action ? (
        <Button size='sm' variant='link' type='button' className='m-link px-0' onClick={action.onClick}>
          {action.label}
        </Button>
      ) : null}
    </div>
  )
}

/** ： and 1 （=、=、=、= in 、=） */
export function Badge({
  tone,
  dot,
  children,
}: {
  tone: 'ok' | 'warn' | 'bad' | 'mute' | 'brand'
  dot?: boolean
  children: React.ReactNode
}) {
  const variants = { ok: 'success', warn: 'warning', bad: 'destructive', mute: 'secondary', brand: 'brand' } as const
  return <UiBadge variant={variants[tone]} shape='tag'>
    {dot ? <BadgeDot /> : null}{children}
  </UiBadge>
}

/** empty */
export function Empty({ text }: { text: string }) {
  return <UiEmpty className='m-empty'>{text}</UiEmpty>
}

/** error（， not ） */
export function ErrorNote({ text }: { text: string }) {
  return <Alert variant='destructive' className='mx-4'>{text}</Alert>
}

/** ： to use ，「empty → 」 */
export function SkeletonList({ rows = 4, height = 64 }: { rows?: number; height?: number }) {
  return (
    <div className='m-section'>
      {Array.from({ length: rows }).map((_, index) => (
        <Skeleton
          key={index}
          className='m-skeleton'
          style={{ height, marginTop: index ? 8 : 0 }}
        />
      ))}
    </div>
  )
}

/** to （failure when execCommand， WebView can use ） */
export async function copyText(text: string): Promise<void> {
  try {
    await navigator.clipboard.writeText(text)
  } catch {
    const area = document.createElement('textarea')
    area.value = text
    area.style.position = 'fixed'
    area.style.opacity = '0'
    document.body.appendChild(area)
    area.select()
    try {
      document.execCommand('copy')
    } catch {
      /* failure： by use to */
    }
    area.remove()
  }
}

/** + use */
export async function copyWithToast(text: string, label?: string): Promise<void> {
  await copyText(text)
  store.toast(t('已复制{label}', { label: label ? `：${label}` : '' }))
}

/** （ / use ） */
export function StatCard({
  label,
  value,
  sub,
  tone,
}: {
  label: string
  value: string
  sub?: string
  tone?: 'up' | 'ok'
}) {
  return (
    <Card className='m-card m-card-pad' style={{ padding: '12px 13px' }}>
      <div style={{ fontSize: 11.5, color: 'var(--m-text-3)' }}>{label}</div>
      <div
        style={{
          fontSize: 21,
          fontWeight: 650,
          letterSpacing: '-0.03em',
          marginTop: 3,
          fontVariantNumeric: 'tabular-nums',
        }}
      >
        {value}
      </div>
      {sub ? (
        <div
          style={{
            fontSize: 11,
            marginTop: 2,
            color: tone === 'up' ? 'var(--m-ok)' : tone === 'ok' ? 'var(--m-ok)' : 'var(--m-text-3)',
          }}
        >
          {sub}
        </div>
      ) : null}
    </Card>
  )
}
