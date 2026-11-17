import { Button, NavItem, NavList, Sheet, SheetContent, SheetHeader, SheetTitle, SheetBody, SheetFooter } from '@ui'
import type { SessionSummary } from '../../api/assistant'
import { Icon } from '../../components/Icon'
import { t } from '../../i18n'

export function AssistantHistory({ open, onClose, sessions, activeId, busy, onOpen, onNew, onDelete, onSettings }: {
  open: boolean
  onClose: () => void
  sessions: SessionSummary[]
  activeId: string
  busy: boolean
  onOpen: (id: string) => void
  onNew: () => void
  onDelete: (id: string) => void
  onSettings: () => void
}) {
  return <Sheet open={open} onOpenChange={value => { if (!value) onClose() }}>
    <SheetContent side='left'>
      <SheetHeader><SheetTitle>{t('Conversation history')}</SheetTitle></SheetHeader>
      <SheetBody>
        <Button disabled={busy} onClick={onNew}><Icon name='plus' />{t('New chat')}</Button>
        <NavList>{sessions.map(session => <div key={session.id} className='flex items-center gap-1'>
          <NavItem active={activeId === session.id} className='min-w-0 flex-1' disabled={busy} onClick={() => onOpen(session.id)}>{session.title || t('Untitled conversation')}</NavItem>
          <Button variant='ghost' size='icon-sm' disabled={busy} aria-label={t('Delete this conversation')} onClick={() => onDelete(session.id)}><Icon name='trash' /></Button>
        </div>)}</NavList>
      </SheetBody>
      <SheetFooter><Button variant='outline' onClick={onSettings}>{t('Title generation settings')}</Button></SheetFooter>
    </SheetContent>
  </Sheet>
}
