import { parseAssistantMessages, type ChatMessage } from '../../../ui-shared/assistant-history'
import type { ToolCardItem, UiMessage } from './assistant-messages'
import { t } from '../i18n'

export function replayMessages(messages: ChatMessage[], model: string | null): UiMessage[] {
  const results = new Map(messages.filter(message => message.role === 'tool').map(message => [message.toolCallId, message]))
  return messages.flatMap((message): UiMessage[] => {
    if (message.role === 'user') return [{ kind: 'user', id: message.id, text: message.content, images: message.images, files: message.files }]
    if (message.role === 'tool') return []
    const items: UiMessage[] = message.content || message.toolCalls?.length
      ? [{ kind: 'bot', id: message.id, model: message.model || model || '', text: message.content }] : []
    for (const call of message.toolCalls || []) {
      const result = results.get(call.id)
      let inferred: ToolCardItem['state'] = result ? 'done' : 'cancelled'
      if (result?.content.includes('The user cancelled') || result?.content.includes('The user has stopped')) inferred = 'cancelled'
      else if (result?.content.startsWith('Tool execution failed') || result?.content.startsWith('Unknown tool')) inferred = 'failed'
      else if (result) {
        try { if (JSON.parse(result.content)?.error) inferred = 'failed' } catch { /* 旧版本只读工具也会返回纯文本。 */ }
      }
      items.push({ kind: 'tool', call, state: result?.outcome?.state || inferred,
        summary: result?.outcome?.summary, auto: result?.outcome?.auto, error: result?.outcome?.error,
        detail: result?.content || t('No completed result was saved for this tool call. Replaying history will not execute it again.'), images: result?.images,
      })
    }
    return items
  })
}

export function normalizeMessages(messages: ChatMessage[]): ChatMessage[] {
  return parseAssistantMessages(messages)
}
