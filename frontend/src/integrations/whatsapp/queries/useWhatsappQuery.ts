import { useQuery } from '@tanstack/vue-query'
import {
  fetchWhatsappCapabilities,
  fetchWhatsappWebSessions,
} from '../api/whatsapp'
import type {
  WhatsappCapabilitiesResponse,
  WhatsappWebSession,
} from '../types/whatsapp'

export function useWhatsappCapabilitiesQuery() {
  return useQuery<WhatsappCapabilitiesResponse>({
    queryKey: ['integrations', 'whatsapp', 'capabilities'],
    queryFn: () => fetchWhatsappCapabilities()
  })
}

export function useWhatsappSessionsQuery(accountId?: string, limit = 50) {
  return useQuery<WhatsappWebSession[]>({
    queryKey: ['integrations', 'whatsapp', 'sessions', accountId ?? 'all', limit],
    queryFn: async () => {
      const res = await fetchWhatsappWebSessions(accountId, limit)
      return res.items
    }
  })
}
