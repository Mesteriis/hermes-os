import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import {
  createAiProvider,
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
  AiModelRouteUpdateRequest,
  AiProviderAuthStartRequest,
  AiProviderConsentRequest,
  AiProviderCreateRequest,
  AiProviderPatchRequest,
} from '../types/aiControlCenter'

export const aiControlCenterKeys = {
  all: ['settings', 'ai-control-center'] as const,
  overview: () => [...aiControlCenterKeys.all, 'overview'] as const,
  models: () => [...aiControlCenterKeys.all, 'models'] as const,
  providerAuth: (setupId: string) => [...aiControlCenterKeys.all, 'provider-auth', setupId] as const,
}

function invalidateAiControlCenter(queryClient: ReturnType<typeof useQueryClient>) {
  void queryClient.invalidateQueries({ queryKey: aiControlCenterKeys.all })
}

export function useAiSettingsOverviewQuery() {
  return useQuery({
    queryKey: aiControlCenterKeys.overview(),
    queryFn: fetchAiSettingsOverview,
  })
}

export function useAiModelsQuery() {
  return useQuery({
    queryKey: aiControlCenterKeys.models(),
    queryFn: fetchAiModels,
  })
}

export function useCreateAiProviderMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (request: AiProviderCreateRequest) => createAiProvider(request),
    onSuccess: () => invalidateAiControlCenter(queryClient),
  })
}

export function useUpdateAiProviderMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ providerId, request }: { providerId: string; request: AiProviderPatchRequest }) =>
      updateAiProvider(providerId, request),
    onSuccess: () => invalidateAiControlCenter(queryClient),
  })
}

export function useTestAiProviderMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (providerId: string) => testAiProvider(providerId),
    onSuccess: () => invalidateAiControlCenter(queryClient),
  })
}

export function useSyncAiProviderModelsMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (providerId: string) => syncAiProviderModels(providerId),
    onSuccess: () => invalidateAiControlCenter(queryClient),
  })
}

export function useUpdateAiModelAvailabilityMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (request: AiModelAvailabilityUpdateRequest) => updateAiModelAvailability(request),
    onSuccess: () => invalidateAiControlCenter(queryClient),
  })
}

export function useUpdateAiProviderConsentMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ providerId, request }: { providerId: string; request: AiProviderConsentRequest }) =>
      updateAiProviderConsent(providerId, request),
    onSuccess: () => invalidateAiControlCenter(queryClient),
  })
}

export function useStartAiProviderAuthMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (request: AiProviderAuthStartRequest) => startAiProviderAuth(request),
    onSuccess: () => invalidateAiControlCenter(queryClient),
  })
}

export function useFetchAiProviderAuthStatusMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (setupId: string) => fetchAiProviderAuthStatus(setupId),
    onSuccess: () => invalidateAiControlCenter(queryClient),
  })
}

export function useUpdateAiModelRouteMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ slot, request }: { slot: string; request: AiModelRouteUpdateRequest }) =>
      updateAiModelRoute(slot, request),
    onSuccess: () => invalidateAiControlCenter(queryClient),
  })
}
