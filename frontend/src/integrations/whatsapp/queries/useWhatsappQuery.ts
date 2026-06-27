import { useQuery } from '@tanstack/vue-query'
import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import {
  fetchWhatsappAccounts,
  fetchWhatsappAccountCapabilities,
  fetchWhatsappCapabilities,
  fetchWhatsappWebSessions,
} from '../api/whatsapp'
import type {
  WhatsappAccountSummary,
  WhatsappCapabilitiesResponse,
  WhatsappWebSession,
} from '../types/whatsapp'
import { whatsappQueryKeys } from './whatsappQueryKeys'

export { whatsappQueryKeys } from './whatsappQueryKeys'
export * from './useWhatsappRuntimeQuery'

export function useWhatsappCapabilitiesQuery() {
  return useQuery<WhatsappCapabilitiesResponse>({
    queryKey: whatsappQueryKeys.capabilities,
    queryFn: () => fetchWhatsappCapabilities()
  })
}

export function useWhatsappAccountsQuery(
  includeRemoved: MaybeRefOrGetter<boolean> = false
) {
  return useQuery<WhatsappAccountSummary[]>({
    queryKey: computed(() => [
      ...whatsappQueryKeys.accounts,
      toValue(includeRemoved) ? 'with-removed' : 'active',
    ]),
    queryFn: async () => {
      const response = await fetchWhatsappAccounts(toValue(includeRemoved))
      return response.items
    }
  })
}

export function useWhatsappAccountCapabilitiesQuery(
  accountId: MaybeRefOrGetter<string | null | undefined>
) {
  return useQuery<WhatsappCapabilitiesResponse | null>({
    queryKey: computed(() => [
      ...whatsappQueryKeys.accountCapabilities,
      toValue(accountId) ?? 'none',
    ]),
    queryFn: async () => {
      const value = toValue(accountId)
      if (!value) return null
      return fetchWhatsappAccountCapabilities(value)
    },
    enabled: computed(() => Boolean(toValue(accountId))),
  })
}

export function useWhatsappSessionsQuery(
  accountId: MaybeRefOrGetter<string | null | undefined>,
  limit = 50
) {
  return useQuery<WhatsappWebSession[]>({
    queryKey: computed(() => [
      ...whatsappQueryKeys.sessions,
      toValue(accountId) ?? 'all',
      limit,
    ]),
    queryFn: async () => {
      const res = await fetchWhatsappWebSessions(toValue(accountId) ?? undefined, limit)
      return res.items
    },
  })
}
