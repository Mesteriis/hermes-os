import { useQuery } from '@tanstack/vue-query'
import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import { fetchTelegramRawMessageEvidence } from '../api/telegramRawEvidence'
import type { TelegramRawMessageResponse } from '../types/telegramRawEvidence'

export function useTelegramRawMessageEvidenceQuery(
  messageId: MaybeRefOrGetter<string | null | undefined>,
  enabled: MaybeRefOrGetter<boolean> = true
) {
  return useQuery<TelegramRawMessageResponse>({
    queryKey: computed(() => ['telegram', 'raw-message-evidence', toValue(messageId)]),
    queryFn: () => fetchTelegramRawMessageEvidence(toValue(messageId) as string),
    enabled: computed(() => Boolean(toValue(messageId)) && Boolean(toValue(enabled))),
  })
}
