import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, toValue } from 'vue'
import {
  fetchMailSyncSettings,
  fetchMailSyncStatus,
  runMailFullResync,
  runMailSyncNow,
  updateMailSyncSettings
} from './syncApi'
import type {
  MailSyncRunResponse,
  MailSyncSettings,
  MailSyncSettingsUpdate,
  MailSyncStatus
} from './types'

type NullableQueryParam<T> = T | null | undefined | (() => T | null | undefined)

export function useSyncStatusesQuery() {
  return useQuery<MailSyncStatus[]>({
    queryKey: ['communications', 'mail', 'sync-statuses'],
    queryFn: async () => {
      const res = await fetchMailSyncStatus()
      return res.items
    }
  })
}

export function useMailSyncSettingsQuery(accountId: NullableQueryParam<string>) {
  return useQuery<MailSyncSettings | null>({
    queryKey: computed(() => {
      const id = toValue(accountId)
      return id
        ? (['communications', 'mail', 'sync-settings', id] as const)
        : (['communications', 'mail', 'sync-settings', null] as const)
    }),
    queryFn: async () => {
      const id = toValue(accountId)
      if (!id) return null
      return fetchMailSyncSettings(id)
    },
    enabled: computed(() => Boolean(toValue(accountId)))
  })
}

export function useUpdateMailSyncSettingsMutation() {
  const queryClient = useQueryClient()
  return useMutation<
    MailSyncSettings,
    Error,
    { accountId: string; settings: MailSyncSettingsUpdate }
  >({
    mutationFn: async ({ accountId, settings }) => updateMailSyncSettings(accountId, settings),
    onSuccess: (_settings, variables) => {
      queryClient.invalidateQueries({
        queryKey: ['communications', 'mail', 'sync-settings', variables.accountId]
      })
      queryClient.invalidateQueries({ queryKey: ['communications', 'mail', 'sync-statuses'] })
    }
  })
}

export function useRunMailSyncNowMutation() {
  const queryClient = useQueryClient()
  return useMutation<MailSyncRunResponse, Error, string>({
    mutationFn: async (accountId) => runMailSyncNow(accountId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['communications-list'] })
      queryClient.invalidateQueries({ queryKey: ['communications-state-counts'] })
      queryClient.invalidateQueries({ queryKey: ['communications', 'mail', 'sync-statuses'] })
      queryClient.invalidateQueries({ queryKey: ['communications', 'mail', 'mailbox-health'] })
    }
  })
}

export function useRunMailFullResyncMutation() {
  const queryClient = useQueryClient()
  return useMutation<MailSyncRunResponse, Error, string>({
    mutationFn: async (accountId) => runMailFullResync(accountId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['communications-list'] })
      queryClient.invalidateQueries({ queryKey: ['communications-state-counts'] })
      queryClient.invalidateQueries({ queryKey: ['communications', 'mail', 'sync-statuses'] })
      queryClient.invalidateQueries({ queryKey: ['communications', 'mail', 'mailbox-health'] })
    }
  })
}
