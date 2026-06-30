import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import {
  authorizeZoomServerToServer,
  cleanupZoomRetention,
  completeZoomOAuth,
  fetchZoomCallTranscript,
  fetchZoomAccounts,
  fetchZoomAuditEvents,
  fetchZoomCapabilities,
  fetchZoomRecordingImports,
  removeZoomRecordingImport,
  fetchZoomProviderCalls,
  fetchZoomRuntimeStatus,
  fetchZoomWebhookSubscriptionStatus,
  maintainZoomTokens,
  reconcileZoomWebhookSubscription,
  refreshZoomToken,
  removeZoomRuntime,
  removeZoomWebhookSubscription,
  setupZoomLiveAccount,
  syncZoomRecordings,
  startZoomOAuth,
  startZoomRuntime,
  stopZoomRuntime,
} from '../api/zoom'
import type {
  ZoomAccount,
  ZoomAccountSetupResponse,
  ZoomAuditEventItem,
  ZoomAuthorizationResult,
  ZoomCallTranscript,
  ZoomCapabilitiesResponse,
  ZoomLiveAccountSetupRequest,
  ZoomOAuthCompleteRequest,
  ZoomOAuthStartRequest,
  ZoomOAuthStartResponse,
  ZoomProviderCall,
  ZoomRecordingImportAuditItem,
  ZoomRecordingImportRemoveRequest,
  ZoomRecordingImportRemoveResponse,
  ZoomRecordingSyncRequest,
  ZoomRecordingSyncResult,
  ZoomRetentionCleanupRequest,
  ZoomRetentionCleanupResponse,
  ZoomRuntimeRemoveRequest,
  ZoomRuntimeRemoveResponse,
  ZoomRuntimeStartRequest,
  ZoomRuntimeStatus,
  ZoomRuntimeStopRequest,
  ZoomServerToServerAuthorizeRequest,
  ZoomTokenMaintenanceRequest,
  ZoomTokenMaintenanceResult,
  ZoomTokenRefreshRequest,
  ZoomTokenRefreshResult,
  ZoomWebhookSubscriptionReconcileRequest,
  ZoomWebhookSubscriptionReconcileResult,
  ZoomWebhookSubscriptionRemoveRequest,
  ZoomWebhookSubscriptionRemoveResult,
  ZoomWebhookSubscriptionStatusResult,
} from '../types/zoom'
import { settingsKeys } from '../../../shared/zoom/settingsBridge'
import { zoomQueryKeys } from './zoomQueryKeys'

export function useZoomCapabilitiesQuery() {
  return useQuery<ZoomCapabilitiesResponse>({
    queryKey: zoomQueryKeys.capabilities,
    queryFn: fetchZoomCapabilities,
  })
}

export function useZoomAccountsQuery(includeRemoved: MaybeRefOrGetter<boolean> = false) {
  return useQuery<ZoomAccount[]>({
    queryKey: computed(() => [...zoomQueryKeys.accounts, toValue(includeRemoved)]),
    queryFn: async () => (await fetchZoomAccounts(toValue(includeRemoved))).items,
  })
}

export function useZoomRuntimeStatusQuery(
  accountId: MaybeRefOrGetter<string | null | undefined>
) {
  return useQuery<ZoomRuntimeStatus | null>({
    queryKey: computed(() => [...zoomQueryKeys.runtimeStatus, toValue(accountId) ?? 'none']),
    queryFn: async () => {
      const value = toValue(accountId)
      if (!value) return null
      return fetchZoomRuntimeStatus(value)
    },
    enabled: computed(() => Boolean(toValue(accountId))),
  })
}

export function useZoomWebhookSubscriptionStatusQuery(
  accountId: MaybeRefOrGetter<string | null | undefined>,
  apiBaseUrl: MaybeRefOrGetter<string | null | undefined> = null
) {
  return useQuery<ZoomWebhookSubscriptionStatusResult | null>({
    queryKey: computed(() => [
      ...zoomQueryKeys.webhookSubscriptions,
      'status',
      toValue(accountId) ?? 'none',
      toValue(apiBaseUrl) ?? 'default',
    ]),
    queryFn: async () => {
      const value = toValue(accountId)
      if (!value) return null
      return fetchZoomWebhookSubscriptionStatus(value, toValue(apiBaseUrl))
    },
    enabled: computed(() => Boolean(toValue(accountId))),
  })
}

export function useZoomProviderCallsQuery(
  accountId: MaybeRefOrGetter<string | null | undefined>,
  limit: MaybeRefOrGetter<number> = 20
) {
  return useQuery<ZoomProviderCall[]>({
    queryKey: computed(() => [
      ...zoomQueryKeys.providerCalls,
      toValue(accountId) ?? 'none',
      toValue(limit),
    ]),
    queryFn: async () => {
      const value = toValue(accountId)
      if (!value) return []
      return (await fetchZoomProviderCalls(value, toValue(limit))).items
    },
    enabled: computed(() => Boolean(toValue(accountId))),
  })
}

export function useZoomRecordingImportsQuery(
  accountId: MaybeRefOrGetter<string | null | undefined>,
  limit: MaybeRefOrGetter<number> = 20
) {
  return useQuery<ZoomRecordingImportAuditItem[]>({
    queryKey: computed(() => [
      ...zoomQueryKeys.recordingImports,
      toValue(accountId) ?? 'none',
      toValue(limit),
    ]),
    queryFn: async () => {
      const value = toValue(accountId)
      if (!value) return []
      return (await fetchZoomRecordingImports(value, toValue(limit))).items
    },
    enabled: computed(() => Boolean(toValue(accountId))),
  })
}

export function useRemoveZoomRecordingImportMutation(
  accountId: MaybeRefOrGetter<string | null | undefined>
) {
  const queryClient = useQueryClient()
  return useMutation<
    ZoomRecordingImportRemoveResponse,
    Error,
    { attachmentId: string; request?: ZoomRecordingImportRemoveRequest }
  >({
    mutationFn: async ({ attachmentId, request }) => {
      const value = toValue(accountId)
      if (!value) {
        throw new Error('Zoom account id is required to remove an imported recording')
      }
      return removeZoomRecordingImport(value, attachmentId, request ?? {})
    },
    onSuccess: () => {
      invalidateZoomDerived(queryClient)
    },
  })
}

export function useCleanupZoomRetentionMutation(
  accountId: MaybeRefOrGetter<string | null | undefined>
) {
  const queryClient = useQueryClient()
  return useMutation<ZoomRetentionCleanupResponse, Error, ZoomRetentionCleanupRequest | undefined>(
    {
      mutationFn: async (request) => {
        const value = toValue(accountId)
        if (!value) {
          throw new Error('Zoom account id is required to run retention cleanup')
        }
        return cleanupZoomRetention(value, request ?? {})
      },
      onSuccess: () => {
        invalidateZoomDerived(queryClient)
      },
    }
  )
}

export function useZoomAuditEventsQuery(
  accountId: MaybeRefOrGetter<string | null | undefined>,
  limit: MaybeRefOrGetter<number> = 25
) {
  return useQuery<ZoomAuditEventItem[]>({
    queryKey: computed(() => [
      ...zoomQueryKeys.auditEvents,
      toValue(accountId) ?? 'none',
      toValue(limit),
    ]),
    queryFn: async () => {
      const value = toValue(accountId)
      if (!value) return []
      return (await fetchZoomAuditEvents(value, toValue(limit))).items
    },
    enabled: computed(() => Boolean(toValue(accountId))),
  })
}

export function useZoomCallTranscriptQuery(
  callId: MaybeRefOrGetter<string | null | undefined>
) {
  return useQuery<ZoomCallTranscript | null>({
    queryKey: computed(() => [
      ...zoomQueryKeys.callTranscript,
      toValue(callId) ?? 'none',
    ]),
    queryFn: async () => {
      const value = toValue(callId)
      if (!value) return null
      return (await fetchZoomCallTranscript(value)).transcript
    },
    enabled: computed(() => Boolean(toValue(callId))),
  })
}

function invalidateZoomRuntime(queryClient: ReturnType<typeof useQueryClient>) {
  queryClient.invalidateQueries({ queryKey: zoomQueryKeys.accounts })
  queryClient.invalidateQueries({ queryKey: zoomQueryKeys.capabilities })
  queryClient.invalidateQueries({ queryKey: zoomQueryKeys.runtimeStatus })
  queryClient.invalidateQueries({ queryKey: zoomQueryKeys.webhookSubscriptions })
  queryClient.invalidateQueries({ queryKey: settingsKeys.workspace() })
}

function invalidateZoomDerived(queryClient: ReturnType<typeof useQueryClient>) {
  queryClient.invalidateQueries({ queryKey: zoomQueryKeys.providerCalls })
  queryClient.invalidateQueries({ queryKey: zoomQueryKeys.callTranscript })
  queryClient.invalidateQueries({ queryKey: zoomQueryKeys.recordingImports })
  queryClient.invalidateQueries({ queryKey: zoomQueryKeys.auditEvents })
  queryClient.invalidateQueries({ queryKey: settingsKeys.workspace() })
}

export function useSetupZoomLiveAccountMutation() {
  const queryClient = useQueryClient()
  return useMutation<ZoomAccountSetupResponse, Error, ZoomLiveAccountSetupRequest>({
    mutationFn: setupZoomLiveAccount,
    onSuccess: () => invalidateZoomRuntime(queryClient),
  })
}

export function useStartZoomOAuthMutation() {
  const queryClient = useQueryClient()
  return useMutation<ZoomOAuthStartResponse, Error, ZoomOAuthStartRequest>({
    mutationFn: startZoomOAuth,
    onSuccess: () => invalidateZoomRuntime(queryClient),
  })
}

export function useCompleteZoomOAuthMutation() {
  const queryClient = useQueryClient()
  return useMutation<ZoomAuthorizationResult, Error, ZoomOAuthCompleteRequest>({
    mutationFn: completeZoomOAuth,
    onSuccess: () => invalidateZoomRuntime(queryClient),
  })
}

export function useAuthorizeZoomServerToServerMutation() {
  const queryClient = useQueryClient()
  return useMutation<ZoomAuthorizationResult, Error, ZoomServerToServerAuthorizeRequest>({
    mutationFn: authorizeZoomServerToServer,
    onSuccess: () => invalidateZoomRuntime(queryClient),
  })
}

export function useRefreshZoomTokenMutation() {
  const queryClient = useQueryClient()
  return useMutation<ZoomTokenRefreshResult, Error, ZoomTokenRefreshRequest>({
    mutationFn: refreshZoomToken,
    onSuccess: () => invalidateZoomRuntime(queryClient),
  })
}

export function useMaintainZoomTokensMutation() {
  const queryClient = useQueryClient()
  return useMutation<ZoomTokenMaintenanceResult, Error, ZoomTokenMaintenanceRequest | undefined>({
    mutationFn: (request) => maintainZoomTokens(request ?? {}),
    onSuccess: () => invalidateZoomRuntime(queryClient),
  })
}

export function useReconcileZoomWebhookSubscriptionMutation() {
  const queryClient = useQueryClient()
  return useMutation<
    ZoomWebhookSubscriptionReconcileResult,
    Error,
    ZoomWebhookSubscriptionReconcileRequest
  >({
    mutationFn: reconcileZoomWebhookSubscription,
    onSuccess: () => invalidateZoomRuntime(queryClient),
  })
}

export function useRemoveZoomWebhookSubscriptionMutation() {
  const queryClient = useQueryClient()
  return useMutation<
    ZoomWebhookSubscriptionRemoveResult,
    Error,
    ZoomWebhookSubscriptionRemoveRequest
  >({
    mutationFn: removeZoomWebhookSubscription,
    onSuccess: () => invalidateZoomRuntime(queryClient),
  })
}

export function useStartZoomRuntimeMutation() {
  const queryClient = useQueryClient()
  return useMutation<ZoomRuntimeStatus, Error, ZoomRuntimeStartRequest>({
    mutationFn: startZoomRuntime,
    onSuccess: () => invalidateZoomRuntime(queryClient),
  })
}

export function useStopZoomRuntimeMutation() {
  const queryClient = useQueryClient()
  return useMutation<ZoomRuntimeStatus, Error, ZoomRuntimeStopRequest>({
    mutationFn: stopZoomRuntime,
    onSuccess: () => invalidateZoomRuntime(queryClient),
  })
}

export function useRemoveZoomRuntimeMutation() {
  const queryClient = useQueryClient()
  return useMutation<ZoomRuntimeRemoveResponse, Error, ZoomRuntimeRemoveRequest>({
    mutationFn: removeZoomRuntime,
    onSuccess: () => invalidateZoomRuntime(queryClient),
  })
}

export function useSyncZoomRecordingsMutation() {
  const queryClient = useQueryClient()
  return useMutation<ZoomRecordingSyncResult, Error, ZoomRecordingSyncRequest>({
    mutationFn: syncZoomRecordings,
    onSuccess: () => {
      invalidateZoomRuntime(queryClient)
      invalidateZoomDerived(queryClient)
    },
  })
}
