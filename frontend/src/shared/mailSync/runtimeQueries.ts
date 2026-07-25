import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, toValue } from 'vue'
import {
  fetchMailContentEgressSettings,
  deleteMailSensitiveForwardingPolicy,
  fetchMailSensitiveForwardingPolicies,
  fetchMailSyncSettings,
  fetchMailSyncStatus,
  runMailFullResync,
  runMailSyncNow,
  updateMailContentEgressSettings,
  upsertMailSensitiveForwardingPolicy,
  updateMailSyncSettings,
} from './syncApi'
import {
  fetchMailProviderResources,
  updateMailProviderResourceMapping,
} from './providerResources'
import { fetchMailLocalFolders } from './localFolders'
import type {
  MailProviderResource,
  MailProviderResourceListResponse,
  MailProviderResourceMappingUpdate,
} from './providerResources'
import type { MailLocalFolder } from './localFolders'
import type {
  MailContentEgressSettings,
  MailSensitiveForwardingPolicy,
  MailSensitiveForwardingPolicyInput,
  MailSyncRunRequest,
  MailSyncRunResponse,
  MailSyncSettings,
  MailSyncSettingsUpdate,
  MailSyncStatus,
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

export function useMailContentEgressSettingsQuery(accountId: NullableQueryParam<string>) {
  return useQuery<MailContentEgressSettings | null>({
    queryKey: computed(() => [
      'communications',
      'mail',
      'content-egress',
      toValue(accountId) ?? null,
    ] as const),
    queryFn: async () => {
      const id = toValue(accountId)
      return id ? fetchMailContentEgressSettings(id) : null
    },
    enabled: computed(() => Boolean(toValue(accountId)))
  })
}

export function useUpdateMailContentEgressSettingsMutation() {
  const queryClient = useQueryClient()
  return useMutation<
    MailContentEgressSettings,
    Error,
    { accountId: string; settings: Partial<MailContentEgressSettings> }
  >({
    mutationFn: ({ accountId, settings }) => updateMailContentEgressSettings(accountId, settings),
    onSuccess: (_value, variables) => {
      queryClient.invalidateQueries({
        queryKey: ['communications', 'mail', 'content-egress', variables.accountId],
      })
    },
  })
}

export function useMailSensitiveForwardingPoliciesQuery(accountId: NullableQueryParam<string>) {
  return useQuery<MailSensitiveForwardingPolicy[]>({
    queryKey: computed(() => [
      'communications',
      'mail',
      'sensitive-forwarding-policies',
      toValue(accountId) ?? null,
    ] as const),
    queryFn: async () => {
      const id = toValue(accountId)
      return id ? (await fetchMailSensitiveForwardingPolicies(id)).items : []
    },
    enabled: computed(() => Boolean(toValue(accountId))),
  })
}

export function useUpsertMailSensitiveForwardingPolicyMutation() {
  const queryClient = useQueryClient()
  return useMutation<
    MailSensitiveForwardingPolicy[],
    Error,
    { accountId: string; policy: MailSensitiveForwardingPolicyInput }
  >({
    mutationFn: async ({ accountId, policy }) =>
      (await upsertMailSensitiveForwardingPolicy(accountId, policy)).items,
    onSuccess: (_items, variables) => {
      queryClient.invalidateQueries({
        queryKey: ['communications', 'mail', 'sensitive-forwarding-policies', variables.accountId],
      })
    },
  })
}

export function useDeleteMailSensitiveForwardingPolicyMutation() {
  const queryClient = useQueryClient()
  return useMutation<void, Error, { accountId: string; policyId: string }>({
    mutationFn: async ({ accountId, policyId }) => {
      await deleteMailSensitiveForwardingPolicy(accountId, policyId)
    },
    onSuccess: (_value, variables) => {
      queryClient.invalidateQueries({
        queryKey: ['communications', 'mail', 'sensitive-forwarding-policies', variables.accountId],
      })
    },
  })
}

export function useMailProviderResourcesQuery(accountId: NullableQueryParam<string>) {
  return useQuery<MailProviderResourceListResponse | null>({
    queryKey: computed(() => {
      const id = toValue(accountId)
      return id
        ? (['communications', 'mail', 'provider-resources', id] as const)
        : (['communications', 'mail', 'provider-resources', null] as const)
    }),
    queryFn: async () => {
      const id = toValue(accountId)
      if (!id) return null
      return fetchMailProviderResources(id)
    },
    enabled: computed(() => Boolean(toValue(accountId)))
  })
}

export function useMailLocalFoldersQuery(accountId: NullableQueryParam<string>) {
  return useQuery<MailLocalFolder[]>({
    queryKey: computed(() => {
      const id = toValue(accountId)
      return id
        ? (['communications', 'mail', 'local-folders', id] as const)
        : (['communications', 'mail', 'local-folders', null] as const)
    }),
    queryFn: async () => {
      const id = toValue(accountId)
      if (!id) return []
      return fetchMailLocalFolders(id)
    },
    enabled: computed(() => Boolean(toValue(accountId)))
  })
}

export function useUpdateMailProviderResourceMappingMutation() {
  const queryClient = useQueryClient()
  return useMutation<
    MailProviderResource,
    Error,
    { accountId: string; mappingId: string; update: MailProviderResourceMappingUpdate }
  >({
    mutationFn: async ({ accountId, mappingId, update }) =>
      updateMailProviderResourceMapping(accountId, mappingId, update),
    onSuccess: (_resource, variables) => {
      queryClient.invalidateQueries({
        queryKey: ['communications', 'mail', 'provider-resources', variables.accountId]
      })
    }
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
  return useMutation<
    MailSyncRunResponse,
    Error,
    string | { accountId: string; request?: MailSyncRunRequest }
  >({
    mutationFn: async (input) => {
      const variables =
        typeof input === 'string' ? { accountId: input } : input
      return runMailSyncNow(variables.accountId, variables.request)
    },
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
  return useMutation<
    MailSyncRunResponse,
    Error,
    string | { accountId: string; request?: MailSyncRunRequest }
  >({
    mutationFn: async (input) => {
      const variables =
        typeof input === 'string' ? { accountId: input } : input
      return runMailFullResync(variables.accountId, variables.request)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['communications-list'] })
      queryClient.invalidateQueries({ queryKey: ['communications-state-counts'] })
      queryClient.invalidateQueries({ queryKey: ['communications', 'mail', 'sync-statuses'] })
      queryClient.invalidateQueries({ queryKey: ['communications', 'mail', 'mailbox-health'] })
    }
  })
}
