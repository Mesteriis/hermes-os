import { useQuery } from '@tanstack/vue-query'
import {
  fetchWhatsappCapabilities,
  fetchWhatsappWebSessions,
  fetchWhatsappWebMessages
} from '../api/whatsapp'
import type {
  WhatsappCapabilitiesResponse,
  WhatsappWebSession,
  WhatsappWebMessage
} from '../types/whatsapp'

export function useWhatsappCapabilitiesQuery() {
  return useQuery<WhatsappCapabilitiesResponse>({
    queryKey: ['integrations', 'whatsapp', 'capabilities'],
    queryFn: () => fetchWhatsappCapabilities()
  })
}

export function useWhatsappSessionsQuery(accountId?: string, limit = 50) {
  return useQuery<WhatsappWebSession[]>({
    queryKey: ['communications', 'whatsapp', 'sessions', accountId ?? 'all', limit],
    queryFn: async () => {
      const res = await fetchWhatsappWebSessions(accountId, limit)
      return res.items
    }
  })
}

export function useWhatsappMessagesQuery(accountId?: string, providerChatId?: string, limit = 50) {
  return useQuery<WhatsappWebMessage[]>({
    queryKey: ['communications', 'whatsapp', 'messages', accountId ?? 'all', providerChatId ?? 'all', limit],
    queryFn: async () => {
      const res = await fetchWhatsappWebMessages(accountId, providerChatId, limit)
      return res.items
    }
  })
}
