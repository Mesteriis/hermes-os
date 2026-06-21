import { useQuery } from '@tanstack/vue-query'
import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import { fetchWhatsappWebBusinessMessages } from '../api/whatsappBusinessApi'
import type { WhatsappWebMessage } from '../../../shared/communications/types/whatsapp'

export const whatsappBusinessQueryKeys = {
  messages: ['communications', 'whatsapp', 'messages'] as const,
}

export function useWhatsappBusinessMessagesQuery(
  accountId?: MaybeRefOrGetter<string | null | undefined>,
  providerChatId?: MaybeRefOrGetter<string | null | undefined>,
  limit: MaybeRefOrGetter<number> = 100
) {
  return useQuery<WhatsappWebMessage[]>({
    queryKey: computed(() => [
      ...whatsappBusinessQueryKeys.messages,
      toValue(accountId) ?? 'all',
      toValue(providerChatId) ?? 'all',
      toValue(limit),
    ]),
    queryFn: async () => {
      const response = await fetchWhatsappWebBusinessMessages(
        toValue(accountId) ?? undefined,
        toValue(providerChatId) ?? undefined,
        toValue(limit)
      )
      return response.items
    },
  })
}

