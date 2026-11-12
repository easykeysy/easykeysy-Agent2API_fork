import { Icon } from './Icon'
import { t } from '../i18n'
import { Button, PageHeader } from '@ui'

/**
 * return。
 *
 * and setreturn（return by + in + ），
 * this use ，all use ， 1 。
 *
 * `actions` by to （ new 、 new …）， no has not 。
 */
export function NavBar({
  title,
  onBack,
  actions,
}: {
  title: string
  onBack: () => void
  actions?: React.ReactNode
}) {
  return (
    <PageHeader className='m-navbar'>
      <Button size='icon-touch' variant='outline' type='button' className='m-icon-btn' onClick={onBack} aria-label={t('返回')}>
        <Icon name='back' />
      </Button>
      <span className='m-navbar-title'>{title}</span>
      <span className='m-navbar-slot'>{actions}</span>
    </PageHeader>
  )
}
