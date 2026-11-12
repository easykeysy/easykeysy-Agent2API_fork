import { useState } from 'react'
import { Icon } from './Icon'
import { providerIcon, providerLabel } from '../lib/providers'

/**
 * ： has use ， no use 。
 *
 * and `.m-logo` 1 （34px / `lg` 40px）， this 。
 */
export function ProviderLogo({
  id,
  label,
  size = 'default',
}: {
  id?: string
  label?: string
  size?: 'default' | 'lg'
}) {
  const icon = providerIcon(id)
  const [failedIcon, setFailedIcon] = useState('')
  const text = providerLabel(id, label)
  return (
    <span className={size === 'lg' ? 'm-logo m-lg' : 'm-logo'}>
      {icon && failedIcon !== icon ? (
        <img src={icon} alt='' loading='lazy' onError={() => setFailedIcon(icon)} />
      ) : (
        <span style={{ fontSize: size === 'lg' ? 16 : 14, fontWeight: 600 }}>{text.slice(0, 1)}</span>
      )}
    </span>
  )
}

/** use ： to 「 not is 」 use （ to 、Key、…） */
export function GlyphLogo({
  name,
  size = 'default',
}: {
  name: Parameters<typeof Icon>[0]['name']
  size?: 'default' | 'lg'
}) {
  return (
    <span className={size === 'lg' ? 'm-logo m-lg' : 'm-logo'}>
      <Icon name={name} />
    </span>
  )
}
