/**
 * 提供商图标与展示名。
 *
 * ── 图标为什么用 <img> 而不是内联 SVG ────────────────────────
 * 提供商图标是一批彩色 PNG（`ui/assets/providers/*.png`），由桌面端同一套
 * 资源提供。移动端页面与它们同源（都在 `ui/` 下），直接引相对路径即可，
 * 不必再打包一份。
 *
 * ── 为什么没有「第二份提供商清单」─────────────────────────────
 * 项目里明确禁止前端维护写死的 provider 列表（见 account_store 模块头）：
 * 加一家内置提供商时，后端注册表是唯一事实来源。这里的映射只解决一件
 * 后端不管的事 —— **图标文件叫什么、中文展示名怎么念**。认不出的 id
 * 一律回落到首字徽章，界面不会空一块。
 */

import { t } from '../i18n'
import { BUILTIN_PROVIDER_ICONS } from '../../../ui-shared/provider-icons'

/** 后端没给 label 时的兜底展示名（认不出的 id 原样显示） */
const LABELS: Record<string, string> = {
  astudio: 'AStudio',
  quwork: 'QuWork',
  workbuddy: 'WorkBuddy China edition',
  'workbuddy-intl': 'WorkBuddy International edition',
  raccoon: 'Raccoon',
  catpaw: 'CatPaw',
  autoclaw: 'AutoClaw China edition',
  'autoclaw-intl': 'AutoClaw International edition',
  qoder: 'Qoder China edition',
  'qoder-intl': 'Qoder International edition',
  'cline-free': 'Cline The free pool',
  cline: 'Cline Free',
  'cline-pass': 'Cline Pass',
  accio: 'Accio International edition',
  'accio-cn': 'Accio China edition',
  'accio-intl': 'Accio China edition',
  zcode: 'ZCode China edition',
  'zcode-intl': 'ZCode International edition',
  codearts: 'CodeArts',
  trae: 'Trae',
  loomy: 'Loomy',
  kuku: 'KukuAI',
  monkeycode: 'MonkeyCode China edition',
  'monkeycode-intl': 'MonkeyCode International edition',
  commandcode: 'Command Code',
  antigravity: 'Antigravity',
  officeace: 'OfficeAce',
  orcaterm: 'OrcaTerm',
  chatglm: 'Zhipu Qingyan',
  tabbit: 'Tabbit (China)',
  'tabbit-intl': 'Tabbit (International)',
  modelscope: 'ModelScope (China)',
  'modelscope-intl': 'ModelScope (International)',
  atomcode: 'AtomCode',
  lobsterai: 'LobsterAI',
  doubaowork: 'Doubao Work',
  cursor: 'Cursor',
  kiro: 'Kiro',
  devin: 'Devin',
  factory: 'Factory',
  zed: 'Zed',
  grok: 'Grok (SuperGrok)',
  'minimax-code': 'MiniMax Code (China)',
  'minimax-code-global': 'MiniMax Code (Global)',
  qwenwork: 'QwenWork (China)',
  'qwenwork-intl': 'QwenWork (International)',
  'qwenwork-enterprise': 'QwenWork (Enterprise)',
  'kimi-work': 'Kimi Work',
  opendesign: 'Open Design',
  'nous-portal': 'Nous Portal',
  'claude-subscription': 'Claude subscription',
  'codex-subscription': 'Codex subscription',
  'chatgpt-subscription': 'ChatGPT API subscription',
  'copilot-subscription': 'GitHub Copilot',
  'gemini-subscription': 'Gemini CLI',
}

/** 图标地址；认不出的 id 返回空串（调用方画首字徽章） */
export function providerIcon(id: string | undefined): string {
  return (id && BUILTIN_PROVIDER_ICONS[id]) || ''
}

/** 展示名：优先用后端给的 label，其次本地映射，最后原样回显 id */
export function providerLabel(id: string | undefined, fallback?: string): string {
  if (fallback) return t(fallback)
  if (!id) return t('Unknown')
  return t(LABELS[id] || id)
}
