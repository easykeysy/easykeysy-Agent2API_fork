import type { ProviderConfig } from './add-account-configs'
import { t } from '../i18n'

export const DOUBAOWORK: ProviderConfig = {
  provider: 'doubaowork',
  label: t('Doubao Work'),
  desktop: true,
  webLogin: {
    noteHtml: t('Sign in on the official Doubao page. The window will read the complete cookies and add the account; the Doubao Work client is not required.'),
    button: t('Open web login'),
    hint: t('Opens in the embedded window; added to the list when done; closing it cancels the wait'),
    busyText: t('Waiting for Doubao Work sign-in…'),
    webHint: t('The browser panel cannot read cookies from other sites. Sign in at www.doubao.com, then paste the complete cookies under Credentials.'),
  },
  manualTitle: t('Paste Cookie'),
  manualNote: t('Paste the complete www.doubao.com cookies, including sessionid. Cookie JSON arrays and objects are also accepted. Uses your Doubao Work account quota; expired cookies require signing in again and cannot be renewed automatically.'),
  fields: [
    { key: 'cookie', label: 'Cookie', rows: 5, placeholder: 'sessionid=…; sessionid_ss=…; ttwid=…' },
    { key: 'name', label: t('Label'), optional: true, placeholder: t('Optional; defaults to the account nickname') },
  ],
  desktopNote: t('Reads cookies from the signed-in local Doubao Work client on each request. Start the client with --remote-debugging-port=9342. This application does not modify, close or restart the client.'),
  desktopHint: t('If the client debugging port is not enabled, use web sign-in or paste cookies.'),
  fileImport: {
    accept: '.json,application/json',
    note: t('Select a JSON object containing cookies, or a single-account JSON exported by this application. The imported account works independently without keeping the Doubao Work client running.'),
  },
}
