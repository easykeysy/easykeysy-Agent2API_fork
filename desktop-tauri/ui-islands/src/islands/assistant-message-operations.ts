import { useCallback } from 'react'
import { removeAssistantMessage, type ChatMessage } from '../../../ui-shared/assistant-history'
import { forkSession, toast, type SessionDetail } from './assistant-state'
import { errorText } from './assistant-tools'
import { replayMessages } from './assistant-replay'
import type { UiMessage } from './assistant-messages'
import { t } from '../i18n'

type Options = {
  messages: { current: ChatMessage[] }
  ui: { current: UiMessage[] }
  activeId: { current: string }
  busy: () => boolean
  setBusy: (value: boolean) => void
  model: () => string | null
  sync: () => void
  persist: () => Promise<void>
  open: (detail: SessionDetail) => void
}

export function useAssistantMessageOperations(options: Options) {
  const edit = useCallback(async (id: string, text: string) => {
    if (options.busy()) return
    options.setBusy(true)
    try {
      options.messages.current = options.messages.current.map(message => message.id === id ? { ...message, content: text } : message)
      options.ui.current = replayMessages(options.messages.current, options.model())
      options.sync()
      await options.persist()
    } finally { options.setBusy(false) }
  }, [options])

  const remove = useCallback(async (id: string) => {
    if (options.busy()) return
    options.setBusy(true)
    try {
      options.messages.current = removeAssistantMessage(options.messages.current, id)
      options.ui.current = replayMessages(options.messages.current, options.model())
      options.sync()
      await options.persist()
    } finally { options.setBusy(false) }
  }, [options])

  const fork = useCallback(async (id: string) => {
    if (options.busy()) return
    options.setBusy(true)
    try {
      await options.persist()
      const detail = await forkSession(options.activeId.current, id)
      options.open(detail)
      toast(t('A new conversation was forked from this message'))
    } catch (failure) {
      toast(t('Failed to fork conversation: {message}', { message: errorText(failure) }), 'err')
    } finally { options.setBusy(false) }
  }, [options])

  return { onEdit: edit, onDelete: remove, onFork: fork }
}
