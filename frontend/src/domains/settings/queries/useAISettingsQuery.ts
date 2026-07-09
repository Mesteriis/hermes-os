import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import {
  createAiProvider,
  deleteAiModelRoute,
  downloadAiModel,
  fetchAiHubUsageStats,
  fetchAiProviderAuthStatus,
  fetchAiModels,
  fetchAiSettingsOverview,
  startAiProviderAuth,
  syncAiProviderModels,
  testAiProvider,
  updateAiModelAvailability,
  updateAiModelRoute,
  updateAiProvider,
  updateAiProviderConsent,
} from '../api/aiControlCenter'
import type {
  AiModelAvailabilityUpdateRequest,
  AiModelDownloadRequest,
  AiModelRouteUpdateRequest,
  AiProviderAuthStartRequest,
  AiProviderConsentRequest,
  AiProviderCreateRequest,
  AiProviderPatchRequest,
} from '../types/aiControlCenter'

export const aiHubKeys = {
  all: ['settings', 'ai-hub'] as const,
  overview: () => [...aiHubKeys.all, 'overview'] as const,
  models: () => [...aiHubKeys.all, 'models'] as const,
  providerAuth: (setupId: string) => [...aiHubKeys.all, 'provider-auth', setupId] as const,
  usageStats: (windowHours: number) => [...aiHubKeys.all, 'usage-stats', windowHours] as const,
}

function invalidateAiHub(queryClient: ReturnType<typeof useQueryClient>) {
  void queryClient.invalidateQueries({ queryKey: aiHubKeys.all })
}

export function useAiSettingsOverviewQuery() {
  return useQuery({
    queryKey: aiHubKeys.overview(),
    queryFn: fetchAiSettingsOverview,
  })
}

export function useAiModelsQuery() {
  return useQuery({
    queryKey: aiHubKeys.models(),
    queryFn: fetchAiModels,
  })
}

export function useAiHubUsageStatsQuery(windowHours = 24) {
  return useQuery({
    queryKey: aiHubKeys.usageStats(windowHours),
    queryFn: () => fetchAiHubUsageStats(windowHours),
    refetchInterval: 15000,
  })
}

export function useCreateAiProviderMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (request: AiProviderCreateRequest) => createAiProvider(request),
    onSuccess: () => invalidateAiHub(queryClient),
  })
}

export function useUpdateAiProviderMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ providerId, request }: { providerId: string; request: AiProviderPatchRequest }) =>
      updateAiProvider(providerId, request),
    onSuccess: () => invalidateAiHub(queryClient),
  })
}

export function useTestAiProviderMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (providerId: string) => testAiProvider(providerId),
    onSuccess: () => invalidateAiHub(queryClient),
  })
}

export function useSyncAiProviderModelsMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (providerId: string) => syncAiProviderModels(providerId),
    onSuccess: () => invalidateAiHub(queryClient),
  })
}

export function useUpdateAiModelAvailabilityMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (request: AiModelAvailabilityUpdateRequest) => updateAiModelAvailability(request),
    onSuccess: () => invalidateAiHub(queryClient),
  })
}

export function useDownloadAiModelMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (request: AiModelDownloadRequest) => downloadAiModel(request),
    onSuccess: () => invalidateAiHub(queryClient),
  })
}

export function useUpdateAiProviderConsentMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ providerId, request }: { providerId: string; request: AiProviderConsentRequest }) =>
      updateAiProviderConsent(providerId, request),
    onSuccess: () => invalidateAiHub(queryClient),
  })
}

export function useStartAiProviderAuthMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (request: AiProviderAuthStartRequest) => startAiProviderAuth(request),
    onSuccess: () => invalidateAiHub(queryClient),
  })
}

export function useFetchAiProviderAuthStatusMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (setupId: string) => fetchAiProviderAuthStatus(setupId),
    onSuccess: () => invalidateAiHub(queryClient),
  })
}

export function useUpdateAiModelRouteMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ slot, request }: { slot: string; request: AiModelRouteUpdateRequest }) =>
      updateAiModelRoute(slot, request),
    onSuccess: () => invalidateAiHub(queryClient),
  })
}

export function useDeleteAiModelRouteMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (slot: string) => deleteAiModelRoute(slot),
    onSuccess: () => invalidateAiHub(queryClient),
  })
}
