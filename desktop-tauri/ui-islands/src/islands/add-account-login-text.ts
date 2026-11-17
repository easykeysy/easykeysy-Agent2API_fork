import { t } from '../i18n'

export function pluginLoginText(value: string): string {
  switch (value.trim()) {
    case 'The local ACP processes still use the previous proxy. Disable and re-enable the ACP plugin on the Plugins page before using a different proxy.':
      return t('The local ACP processes still use the previous proxy. Disable and re-enable the ACP plugin on the Plugins page before using a different proxy.')
    case 'The ACP proxy configuration is invalid. Select the proxy again.': return t('The ACP proxy configuration is invalid. Select the proxy again.')
    case 'A sign-in is already in progress for this provider. Complete or cancel it first.': return t('A sign-in is already in progress for this provider. Complete or cancel it first.')
    case 'ACP has not returned a usable session model catalog. Check the official CLI sign-in status and try again.':
      return t('ACP has not returned a usable session model catalog. Check the official CLI sign-in status and try again.')
    case 'A failed ACP session is not retried automatically or switched to another connection method.':
      return t('A failed ACP session is not retried automatically or switched to another connection method.')
    case 'Google sign-in (ACP)': return t('Google sign-in (ACP)')
    case 'ACP uses the official agy CLI on the gateway computer, not the device running this browser or phone. Install agy 1.5.0 or later first: https://antigravity.google/docs/cli. For a custom installation path, set AGY_PATH in the gateway\'s startup environment.':
      return t('ACP uses the official agy CLI on the gateway computer, not the device running this browser or phone. Install agy 1.5.0 or later first: https://antigravity.google/docs/cli. For a custom installation path, set AGY_PATH in the gateway\'s startup environment.')
    case 'Click Sign in, authorize on Google\'s page, then paste the authorization code or full page URL unchanged. If agy is already signed in, it connects directly. The official client keeps Google\'s credentials; Agent2API stores only a local connection marker. Each gateway uses agy\'s current account, not multiple independent copies.':
      return t('Click Sign in, authorize on Google\'s page, then paste the authorization code or full page URL unchanged. If agy is already signed in, it connects directly. The official client keeps Google\'s credentials; Agent2API stores only a local connection marker. Each gateway uses agy\'s current account, not multiple independent copies.')
    case 'Browser sign-in (ink credits + creates an API key)':
      return t('Browser sign-in (ink credits + automatic API Key creation)')
    case 'Browser sign-in + reuse an existing API key':
      return t('Browser sign-in (reuse an existing API Key)')
    case 'API key only (models, no ink-credit card)':
      return t('API Key only (model access, no ink-credit balance)')
    case 'API key + console token (models + ink-credit card)':
      return t('API Key + console token (model access + ink-credit balance)')
    case 'Leave empty to sign in in a browser (paste an existing key to reuse it)':
      return t('Existing API Key (leave blank to create one after browser sign-in)')
    case 'TokenPlan API key (console → The research model→ API Key)':
      return t('TokenPlan API Key (console → Research Models → API Key)')
    case 'TokenPlan API key':
      return 'TokenPlan API Key'
    case 'Console uaa-token (paste just the token value, or the full "uaa-token=…" cookie entry)':
      return t('Console uaa-token (paste the token value or the full uaa-token=… cookie entry)')
    case 'A browser window opens on the InkStone console. Sign in there (password, SMS or QR code); the plugin reads the completed sign-in by itself and closes the window. You do not need to close it. After the sign-in the plugin creates an ink-* API key on your account — the plaintext is shown once, and the plugin keeps it for you.':
      return t('Sign in to the InkStone console window opened by the plugin using a password, SMS or QR code. The plugin reads the sign-in result and closes the window automatically. It then creates or reuses an API Key according to the selected method. New keys are shown in plaintext only once and are saved by the plugin.')
    case 'A browser window opens on the InkStone console. Sign in there (password, SMS or QR code), then quit that browser yourself — Cmd+Q on macOS (closing its last window is not enough), or close its last window on Linux. The plugin then reads the session from the profile and finishes. After the sign-in the plugin creates an ink-* API key on your account — the plaintext is shown once, and the plugin keeps it for you.':
      return t('Sign in to the InkStone console window opened by the plugin using a password, SMS or QR code, then quit that browser: press Cmd+Q on macOS (closing the window is not enough), or close its last window on Linux. The plugin then reads the sign-in result and creates or reuses an API Key according to the selected method. New keys are shown in plaintext only once and are saved by the plugin.')
    case 'Browser login (log in once, credentials fully automatic)':
      return t('Plugin-managed Chrome sign-in (automatic credential capture)')
    case 'A dedicated Chrome window has opened: sign in to Mobile Cloud (yun.139.com; the web page supports several login methods) inside it. After a successful login, this flow continues automatically and harvests all credentials.",':
      return t('Sign in to China Mobile Cloud (yun.139.com) in the dedicated Chrome window opened by the plugin. After sign-in, credentials are read and verified automatically, then the account is added.')
    case 'Waiting for the plugin to complete sign-in':
      return t('Waiting for the plugin to complete sign-in')
    case 'Waiting for browser sign-in to complete':
      return t('Waiting for browser sign-in to complete')
    case 'Browser authorization received. Retrieving account credentials…':
      return t('Browser authorization received. Retrieving account credentials…')
    case 'After the browser shows sign-in complete, the gateway still needs to retrieve account credentials online. If your browser or Magpie uses a system proxy, click “Fill from system proxy” below first. Direct connections do not use the system proxy.':
      return t('After the browser shows sign-in complete, the gateway still needs to retrieve account credentials online. If your browser or Magpie uses a system proxy, click “Fill from system proxy” below first. Direct connections do not use the system proxy.')
    case 'Browser authorization was received, but the gateway could not reach DeepAI directly. Fill from the system proxy or choose a working sign-in proxy, then sign in again.':
      return t('Browser authorization was received, but the gateway could not reach DeepAI directly. Fill from the system proxy or choose a working sign-in proxy, then sign in again.')
    case 'Browser authorization was received, but retrieving account credentials through the selected proxy failed. Check the proxy and sign in again.':
      return t('Browser authorization was received, but retrieving account credentials through the selected proxy failed. Check the proxy and sign in again.')
    default:
      return t(value)
  }
}

export function pluginBrowserInstructions(provider: string): string {
  if (provider === 'intern-inkstone') {
    return t('The InkStone plugin opens a separate Edge, Chrome or Chromium window on the gateway machine. It does not use the system default browser or the app\'s embedded window.')
  }
  if (provider === 'mclaw') {
    return t('The MClaw plugin opens a dedicated Chrome window on the gateway machine. Its isolated sign-in profile is also used for automatic renewal; it does not use the system default browser or the app\'s embedded window.')
  }
  return t('The plugin starts a separate browser on the gateway machine and reads the sign-in result. It does not use the system\'s default browser.')
}

export function mergeLoginInstructions(...values: Array<string | null | undefined>): string {
  const seen = new Set<string>()
  return values.flatMap(value => value?.split(/\r?\n/) || []).map(line => line.trim()).filter(line => {
    const key = line.replace(/\s+/g, ' ')
    if (!key || seen.has(key)) return false
    seen.add(key)
    return true
  }).join('\n')
}
