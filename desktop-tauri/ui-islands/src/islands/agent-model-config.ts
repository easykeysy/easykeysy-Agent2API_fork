import { t } from '../i18n'
import type { AgentClient, AgentModel, AgentProfile } from './agent-types'

export function modelFields(client: AgentClient, profile?: AgentProfile): Array<keyof AgentModel> {
  if (client.modelFields) return client.modelFields.filter(field => client.id !== 'kimi-code' || field !== 'maxOutputTokens' || profile?.protocol === 'anthropic')
  if (client.id === 'claude-code') return []
  if (client.id === 'codex') return ['contextWindow', 'supportsTools', 'supportsImages']
  return ['contextWindow', 'maxOutputTokens', 'supportsTools', 'supportsImages', 'supportsReasoning']
}

export function modelLevels(client: AgentClient, levels: string[], protocol?: string): string[] {
  if (client.id === 'opencode' && protocol === 'anthropic') return [...new Set(levels.map(level => ['off', 'disabled'].includes(level) ? 'none' : level).filter(level => ['none', 'low', 'medium', 'high', 'xhigh', 'max'].includes(level)))]
  const supported: Record<string, string[]> = {
    'claude-code': ['low', 'medium', 'high', 'xhigh'],
    codex: ['none', 'minimal', 'low', 'medium', 'high', 'xhigh', 'max', 'ultra'],
    zed: ['none', 'minimal', 'low', 'medium', 'high', 'xhigh', 'max'],
    dsh: ['off', 'minimal', 'low', 'medium', 'high', 'xhigh', 'max'],
  }
  return [...new Set(levels.map(level => client.id === 'dsh' ? ['none', 'disabled'].includes(level) ? 'off' : level === 'mid' ? 'medium' : level
    : client.id === 'codex' && ['off', 'disabled'].includes(level) ? 'none' : level)
    .filter(level => !supported[client.id] || supported[client.id].includes(level)))]
}

export function modelFieldNote(client: AgentClient): string {
  if (client.id === 'claude-code') return t('Claude Code supports a multi-model menu and per-model reasoning effort. Context and input capabilities are recognized by the client; the auto-compaction threshold is not treated as the context window.')
  if (client.id === 'codex') return t('The Codex model catalog supports per-model context windows, image input, and reasoning levels, but has no writable per-model output limit.')
  if (client.id === 'kimi-code') return t('Kimi Code can add capability declarations but cannot disable built-in capabilities. Output limits apply only to the Anthropic protocol in newer versions; reasoning levels are not written for legacy versions.')
  return ''
}
