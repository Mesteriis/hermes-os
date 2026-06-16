import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import {
  cancelTelegramQrLogin,
  pollTelegramQrLogin,
  startTelegramQrLogin,
  submitTelegramQrPassword,
} from '../api/telegram'
import type {
  TelegramQrLoginPasswordRequest,
  TelegramQrLoginStartRequest,
  TelegramQrLoginStatusResponse,
} from '../types/telegram'

export const telegramQrLoginQueryKeys = {
  status: ['telegram', 'qr-login-status'] as const,
}

export function useStartTelegramQrLoginMutation() {
  return useMutation({
    mutationFn: (request: TelegramQrLoginStartRequest) => startTelegramQrLogin(request),
  })
}

export function useTelegramQrLoginStatusQuery(
  setupId: MaybeRefOrGetter<string | null | undefined>
) {
  return useQuery<TelegramQrLoginStatusResponse | null>({
    queryKey: computed(() => [
      ...telegramQrLoginQueryKeys.status,
      toValue(setupId) ?? 'none',
    ]),
    queryFn: async () => {
      const value = toValue(setupId)
      if (!value) return null
      return pollTelegramQrLogin(value)
    },
    enabled: computed(() => Boolean(toValue(setupId))),
    refetchInterval: (query) => {
      const status = query.state.data?.status
      if (status === 'waiting_qr_scan' || status === 'waiting_password') {
        return query.state.data?.poll_after_ms ?? 3000
      }
      return false
    },
  })
}

export function useCancelTelegramQrLoginMutation(
  setupId: MaybeRefOrGetter<string | null | undefined>
) {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async () => {
      const value = toValue(setupId)
      if (!value) {
        throw new Error('Telegram QR setup ID is required')
      }
      return cancelTelegramQrLogin(value)
    },
    onSuccess: () => {
      const value = toValue(setupId)
      if (!value) return
      queryClient.removeQueries({
        queryKey: [...telegramQrLoginQueryKeys.status, value],
      })
    },
  })
}

export function useSubmitTelegramQrPasswordMutation(
  setupId: MaybeRefOrGetter<string | null | undefined>
) {
  return useMutation({
    mutationFn: async (request: TelegramQrLoginPasswordRequest) => {
      const value = toValue(setupId)
      if (!value) {
        throw new Error('Telegram QR setup ID is required')
      }
      return submitTelegramQrPassword(value, request)
    },
  })
}
