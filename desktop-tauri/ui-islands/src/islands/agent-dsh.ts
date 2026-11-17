import { t } from '../i18n'

export function dshTargetLabel(profile: string, fallback: string): string {
  switch (profile) {
    case 'web': return t('Web app')
    case 'desktop': return t('Desktop app')
    case 'legacy': return t('Legacy shared settings')
    case 'home': return t('All profiles (global override)')
    default: return fallback
  }
}
