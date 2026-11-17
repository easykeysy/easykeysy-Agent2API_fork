import type { ProviderConfig } from './add-account-configs'
import { t } from '../i18n'

function monkeycodeForm(provider: string, label: string, siteNote: string): ProviderConfig {
  return {
    provider,
    label,
    desktop: false,
    webLogin: {
      desktopOnly: true,
      noteHtml: t('Complete sign-in and verification on the official MonkeyCode page for the selected region. Your session is captured and the account is added automatically; no need to copy cookies.'),
      button: t('Open web login'),
      hint: t('Opens in the embedded window; added to the list when done; closing it cancels the wait'),
      busyText: t('Waiting for MonkeyCode sign-in…'),
      webHint: t('Automatic MonkeyCode web sign-in requires the desktop app. In the browser panel, sign in to the corresponding official site and paste your session.'),
    },
    manualTitle: t('Paste session cookie'),
    manualNoteHtml: t('Paste the value of the web session cookie <code>monkeycode_ai_session</code> (copying the whole Cookie in the browser and pasting it directly also works; the gateway extracts the value itself).<br>{siteNote}<br>MonkeyCode <b>has no renewal API</b>: the session is valid for about 30 days (a hard limit); once it expires, sign in to the web page again and paste it one more time.', { siteNote }),
    fields: [
      { key: 'session', label: 'session', rows: 3, placeholder: t('The value of monkeycode_ai_session (you can also paste the whole Cookie)') },
      { key: 'imageId', label: t('Task image id'), optional: true, placeholder: t('Optional; required to start a conversation. If blank, the gateway discovers it automatically from this account\'s existing tasks; if it cannot find one, fill it in manually later.') },
      { key: 'name', label: t('Label'), optional: true, placeholder: t('Optional; defaults to the account name or account id returned by upstream') },
    ],
  }
}

export const MONKEYCODE = monkeycodeForm(
  'monkeycode',
  t('MonkeyCode'),
  t('This is for the China site (monkeycode-ai.com) account: paste the China site\'s session. The international site (monkeycode-ai.net) is another provider with its own account; credentials are not interchangeable between the two.'),
)

export const MONKEYCODE_INTL = monkeycodeForm(
  'monkeycode-intl',
  t('MonkeyCode International edition'),
  t('This is for the international site (monkeycode-ai.net) account: paste the international site\'s session. The China site (monkeycode-ai.com) is another provider with its own account; credentials are not interchangeable between the two.'),
)
