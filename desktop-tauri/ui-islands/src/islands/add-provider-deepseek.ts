import type { ProviderConfig } from './add-account-configs'
import { t } from '../i18n'

export const DEEPSEEK_WEB: ProviderConfig = {
  provider: 'deepseek-web',
  label: t('DeepSeek Web'),
  desktop: false,
  manualTitle: t('Paste a web access token'),
  manualNote: t('Sign in to your own account at chat.deepseek.com, then copy the value field of userToken from Application → Local Storage in your browser\'s developer tools. Cookie is optional; if needed, paste the full Cookie header from a request to that site. If your session expires or human verification is required, sign in manually on the official website and update your credentials. Paid API keys are not accepted here.'),
  fields: [
    {
      key: 'accessToken', label: t('Web access token'), rows: 3,
      placeholder: t('userToken.value from chat.deepseek.com, not a paid API key'),
    },
    {
      key: 'cookie', label: t('Web Cookie'), rows: 3, optional: true,
      placeholder: t('Cookie from a normal sign-in on the official website; update it manually if verification is required'),
    },
    { key: 'name', label: t('Label'), optional: true, placeholder: t('Optional; generated automatically if blank') },
  ],
  fileImport: {
    accept: '.json,application/json',
    note: t('Select a JSON file containing this provider\'s credentials or an individual account JSON file exported by this application. Imported credentials are stored in the gateway and can be used independently.'),
  },
}
