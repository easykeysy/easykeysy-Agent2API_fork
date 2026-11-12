// ID used for；Rust before not and use 。
// Kiro already by 「」（2026-11）， and after 
// `core/providers/availability.rs` SUSPENDED_IDS 。
export const SUSPENDED_PROVIDER_IDS: ReadonlySet<string> = new Set([
  'cursor',
  'devin',
  'factory',
  'zed',
  'grok',
  'minimax-code',
  'minimax-code-global',
  // ： up （ ），
  // not ， from can ； (`qwenwork`) not 。
  'qwenwork-intl',
  'qwenwork-enterprise',
  'kimi-work',
  'opendesign',
  'nous-portal',
  'claude-subscription',
  'codex-subscription',
  'chatgpt-subscription',
  'copilot-subscription',
  'gemini-subscription',
])

export function isSuspendedProvider(id: string): boolean {
  return SUSPENDED_PROVIDER_IDS.has(id)
}

// LobsterAI ；add， not and use 。
const HIDDEN_ADD_ACCOUNT_PROVIDER_IDS: ReadonlySet<string> = new Set(['lobsterai'])

export function isAddAccountProviderHidden(id: string): boolean {
  return isSuspendedProvider(id) || HIDDEN_ADD_ACCOUNT_PROVIDER_IDS.has(id)
}
