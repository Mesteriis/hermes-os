import { useQuery } from '@tanstack/vue-query'
import { fetchCommunicationMessages, fetchMailboxHealth } from '../api/home'
import type { CommunicationMessageSummary, MailboxHealth } from '../types/api'

export function useCommunicationMessagesQuery(limit = 50) {
  return useQuery<CommunicationMessageSummary[]>({
    queryKey: ['home', 'communication-messages', limit],
    queryFn: async () => {
      const res = await fetchCommunicationMessages(limit)
      return res.items
    }
  })
}

export function useMailboxHealthQuery() {
  return useQuery<MailboxHealth>({
    queryKey: ['home', 'mailbox-health'],
    queryFn: fetchMailboxHealth
  })
}
