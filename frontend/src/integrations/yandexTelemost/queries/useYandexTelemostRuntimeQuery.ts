import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import {
  fetchYandexTelemostAccounts,
  fetchYandexTelemostCapabilities,
  fetchYandexTelemostRuntimeStatus,
  setupYandexTelemostAccount,
} from '../api/yandexTelemost'
import type {
  YandexTelemostAccount,
  YandexTelemostAccountSetupRequest,
  YandexTelemostAccountSetupResponse,
  YandexTelemostCapabilitiesResponse,
  YandexTelemostRuntimeStatus,
} from '../types/yandexTelemost'
import { yandexTelemostQueryKeys } from './yandexTelemostQueryKeys'

export function useYandexTelemostCapabilitiesQuery() {
  return useQuery<YandexTelemostCapabilitiesResponse>({
    queryKey: yandexTelemostQueryKeys.capabilities,
    queryFn: fetchYandexTelemostCapabilities,
  })
}

export function useYandexTelemostAccountsQuery(includeRemoved: MaybeRefOrGetter<boolean> = false) {
  return useQuery<YandexTelemostAccount[]>({
    queryKey: computed(() => [...yandexTelemostQueryKeys.accounts, toValue(includeRemoved)]),
    queryFn: async () => (await fetchYandexTelemostAccounts(toValue(includeRemoved))).items,
  })
}

export function useYandexTelemostRuntimeStatusQuery(
  accountId: MaybeRefOrGetter<string | null | undefined>
) {
  return useQuery<YandexTelemostRuntimeStatus | null>({
    queryKey: computed(() => [...yandexTelemostQueryKeys.runtimeStatus, toValue(accountId) ?? 'none']),
    queryFn: async () => {
      const value = toValue(accountId)
      if (!value) return null
      return fetchYandexTelemostRuntimeStatus(value)
    },
    enabled: computed(() => Boolean(toValue(accountId))),
  })
}

export function useSetupYandexTelemostAccountMutation() {
  const queryClient = useQueryClient()
  return useMutation<YandexTelemostAccountSetupResponse, Error, YandexTelemostAccountSetupRequest>({
    mutationFn: setupYandexTelemostAccount,
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: yandexTelemostQueryKeys.accounts })
    },
  })
}
