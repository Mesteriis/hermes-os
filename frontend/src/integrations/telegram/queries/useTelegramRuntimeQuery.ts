import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import {
  fetchTelegramRuntimeStatus,
  restartTelegramRuntime,
  startTelegramRuntime,
  stopTelegramRuntime,
} from '../api/telegram'
import type { TelegramRuntimeStatus } from '../types/telegram'
import { telegramQueryKeys } from './useTelegramQuery'

export function useTelegramRuntimeStatusQuery(accountId: MaybeRefOrGetter<string | null>) {
  return useQuery<TelegramRuntimeStatus>({
    queryKey: computed(() => [...telegramQueryKeys.runtime, toValue(accountId) ?? 'none']),
    queryFn: () => fetchTelegramRuntimeStatus(toValue(accountId) ?? ''),
    enabled: computed(() => Boolean(toValue(accountId))),
  })
}

function invalidateTelegramRuntime(queryClient: ReturnType<typeof useQueryClient>) {
  queryClient.invalidateQueries({ queryKey: telegramQueryKeys.runtime })
  queryClient.invalidateQueries({ queryKey: telegramQueryKeys.accounts })
}

export function useStartTelegramRuntimeMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (request: { account_id: string }) => startTelegramRuntime(request),
    onSuccess: () => invalidateTelegramRuntime(queryClient),
  })
}

export function useStopTelegramRuntimeMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (request: { account_id: string }) => stopTelegramRuntime(request),
    onSuccess: () => invalidateTelegramRuntime(queryClient),
  })
}

export function useRestartTelegramRuntimeMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (request: { account_id: string }) => restartTelegramRuntime(request),
    onSuccess: () => invalidateTelegramRuntime(queryClient),
  })
}
