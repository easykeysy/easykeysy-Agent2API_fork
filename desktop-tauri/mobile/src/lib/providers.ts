/**
 * and 。
 *
 * ── Why use <img> not is SVG ────────────────────────
 * is 1 PNG（`ui/assets/providers/*.png`）， by 1 
 * 。 and （ in `ui/` down ）， for path can ，
 * not 1 。
 *
 * ── Why no has 「」─────────────────────────────
 * before provider （ account_store module header）：
 * 1 when ， after is 1 。 this 1 
 * after not —— **file、 in **。 not id
 * 1 to ， not will empty 1 。
 */

import { t } from '../i18n'

/** id → file（ for `ui/` ， in `ui/m.html` before `m/` not use ） */
const ICONS: Record<string, string> = {
  workbuddy: 'assets/providers/workbuddy.png',
  'workbuddy-intl': 'assets/providers/workbuddy.png',
  raccoon: 'assets/providers/raccoon.png',
  catpaw: 'assets/providers/catpaw.png',
  autoclaw: 'assets/providers/autoclaw.png',
  'autoclaw-intl': 'assets/providers/autoclaw.png',
  qoder: 'assets/providers/qoder.png',
  'qoder-intl': 'assets/providers/qoder.png',
  'cline-free': 'assets/providers/cline.png',
  cline: 'assets/providers/cline.png',
  'cline-pass': 'assets/providers/cline.png',
  accio: 'assets/providers/accio.png',
  'accio-cn': 'assets/providers/accio.png',
  'accio-intl': 'assets/providers/accio.png',
  zcode: 'assets/providers/zcode.png',
  'zcode-intl': 'assets/providers/zcode.png',
  codearts: 'assets/providers/codearts.png',
  trae: 'assets/providers/trae.png',
  loomy: 'assets/providers/loomy.png',
  kuku: 'assets/providers/kuku.png',
  monkeycode: 'assets/providers/monkeycode.png',
  'monkeycode-intl': 'assets/providers/monkeycode.png',
  commandcode: 'assets/providers/commandcode.png',
  antigravity: 'assets/providers/antigravity.png',
  officeace: 'assets/providers/officeace.png',
  orcaterm: 'assets/providers/orcaterm.png',
  chatglm: 'assets/providers/chatglm.png',
  'deepseek-web': 'assets/providers/deepseek.png',
  tabbit: 'assets/providers/tabbit.png',
  'tabbit-intl': 'assets/providers/tabbit.png',
  modelscope: 'assets/providers/modelscope.png',
  'modelscope-intl': 'assets/providers/modelscope.png',
  atomcode: 'assets/providers/atomcode.png',
  lobsterai: 'assets/providers/lobsterai.png',
  doubaowork: 'assets/providers/doubaowork.png',
  cursor: 'assets/providers/cursor.svg',
  kiro: 'assets/providers/kiro.svg',
  astudio: 'assets/providers/astudio.png',
  devin: 'assets/providers/devin.svg',
  factory: 'assets/providers/factory.svg',
  zed: 'assets/providers/zed.svg',
  grok: 'assets/providers/grok.svg',
  'minimax-code': 'assets/providers/minimax-code.svg',
  'minimax-code-global': 'assets/providers/minimax-code.svg',
  qwenwork: 'assets/providers/qwenwork.svg',
  'qwenwork-intl': 'assets/providers/qwenwork.svg',
  'qwenwork-enterprise': 'assets/providers/qwenwork.svg',
  'kimi-work': 'assets/providers/kimi-work.svg',
  opendesign: 'assets/providers/opendesign.svg',
  'nous-portal': 'assets/providers/nous-portal.svg',
  'claude-subscription': 'assets/providers/anthropic.png',
  'codex-subscription': 'assets/providers/openai.png',
  'chatgpt-subscription': 'assets/providers/openai.png',
  'copilot-subscription': 'assets/providers/copilot.svg',
  'gemini-subscription': 'assets/providers/gemini-cli.svg',
}

/** after no to label when （ not id ） */
const LABELS: Record<string, string> = {
  astudio: 'AStudio',
  workbuddy: 'WorkBuddy 国内版',
  'workbuddy-intl': 'WorkBuddy 国际版',
  raccoon: '小浣熊',
  catpaw: 'CatPaw',
  autoclaw: 'AutoClaw 国内版',
  'autoclaw-intl': 'AutoClaw 国际版',
  qoder: 'Qoder 中国版',
  'qoder-intl': 'Qoder 国际版',
  'cline-free': 'Cline 免费池',
  cline: 'Cline Free',
  'cline-pass': 'Cline Pass',
  accio: 'Accio 国际版',
  'accio-cn': 'Accio 国内版',
  'accio-intl': 'Accio 国内版',
  zcode: 'ZCode 国内版',
  'zcode-intl': 'ZCode 国际版',
  codearts: 'CodeArts',
  trae: 'Trae',
  loomy: 'Loomy',
  kuku: 'KukuAI',
  monkeycode: 'MonkeyCode 国内版',
  'monkeycode-intl': 'MonkeyCode 国际版',
  commandcode: 'Command Code',
  antigravity: 'Antigravity',
  officeace: 'OfficeAce 果办',
  orcaterm: 'OrcaTerm',
  chatglm: '智谱清言',
  tabbit: 'Tabbit 国内版',
  'tabbit-intl': 'Tabbit 国际版',
  modelscope: 'ModelScope 国内版',
  'modelscope-intl': 'ModelScope 国际版',
  atomcode: 'AtomCode',
  lobsterai: 'LobsterAI',
  doubaowork: '豆包工作',
  cursor: 'Cursor',
  kiro: 'Kiro',
  devin: 'Devin',
  factory: 'Factory',
  zed: 'Zed',
  grok: 'Grok (SuperGrok)',
  'minimax-code': 'MiniMax Code 国内版',
  'minimax-code-global': 'MiniMax Code 国际版',
  qwenwork: '千问办公 国内版',
  'qwenwork-intl': '千问办公 国际版',
  'qwenwork-enterprise': 'QwenWork 千问办公企业版',
  'kimi-work': 'Kimi Work',
  opendesign: 'Open Design',
  'nous-portal': 'Nous Portal',
  'claude-subscription': 'Claude 订阅',
  'codex-subscription': 'Codex 订阅',
  'chatgpt-subscription': 'ChatGPT API 订阅',
  'copilot-subscription': 'GitHub Copilot',
  'gemini-subscription': 'Gemini CLI',
}

/** ； not id returnempty（ use ） */
export function providerIcon(id: string | undefined): string {
  return (id && ICONS[id]) || ''
}

/** ： use after to label， its ， after id */
export function providerLabel(id: string | undefined, fallback?: string): string {
  if (fallback) return t(fallback)
  if (!id) return t('未知')
  return t(LABELS[id] || id)
}
