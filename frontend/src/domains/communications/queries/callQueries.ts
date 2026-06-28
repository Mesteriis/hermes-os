import { useQuery } from '@tanstack/vue-query'
import { computed, toValue } from 'vue'
import {
  fetchProviderCalls,
  fetchProviderCallTranscript,
} from '../api/communications'
import type {
  ProviderCall,
  ProviderCallTranscript,
} from '../types/communications'
import {
  communicationDetailQueryOptions,
  communicationRealtimeQueryOptions,
} from './communicationQueryPolicies'
import type { NullableQueryParam, QueryParam } from './queryTypes'

export function useProviderCallsQuery(
  accountId?: QueryParam<string>,
  limit: QueryParam<number> = 50,
  provider?: QueryParam<string>
) {
  return useQuery<ProviderCall[]>({
    queryKey: computed(() => [
      'communications-calls',
      toValue(accountId),
      toValue(limit),
      toValue(provider),
    ]),
    queryFn: async () => {
      const response = await fetchProviderCalls(
        toValue(accountId),
        toValue(limit),
        toValue(provider)
      )
      return response.items
    },
    ...communicationRealtimeQueryOptions,
  })
}

export function useProviderCallTranscriptQuery(
  callId: NullableQueryParam<string>
) {
  return useQuery<ProviderCallTranscript | null>({
    queryKey: computed(() => ['communications-call-transcript', toValue(callId)]),
    queryFn: async () => {
      const currentCallId = toValue(callId)?.trim() ?? ''
      if (!currentCallId) return null
      return (await fetchProviderCallTranscript(currentCallId)).transcript
    },
    enabled: computed(() => Boolean(toValue(callId)?.trim())),
    ...communicationDetailQueryOptions,
  })
}
