import type { CommunicationSectionId } from '../types/communications'

export const communicationSectionTabs: {
  id: CommunicationSectionId
  label: string
  icon: string
}[] = [
  { id: 'unified', label: 'Unified', icon: 'tabler:inbox' },
  { id: 'inbox', label: 'Inbox', icon: 'tabler:mail' },
  { id: 'needs_reply', label: 'Need Reply', icon: 'tabler:message-reply' },
  { id: 'waiting', label: 'Waiting', icon: 'tabler:clock' },
  { id: 'done', label: 'Done', icon: 'tabler:check' },
  { id: 'archived', label: 'Archived', icon: 'tabler:archive' }
]
