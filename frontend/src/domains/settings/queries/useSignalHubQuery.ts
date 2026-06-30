import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import {
  applySignalHubProfile,
  createSignalHubProfile,
  createSignalHubReplayRequest,
  createSignalHubPolicy,
  createSignalHubConnection,
  disableSignalHubSignals,
  disableSignalHubSource,
  enableSignalHubSignals,
  enableSignalHubSource,
  fetchSignalHubCapabilities,
  fetchSignalHubProfiles,
  fetchSignalHubConnections,
  fetchSignalHubHealth,
  fetchSignalHubPolicies,
  fetchSignalHubReplayRequests,
  muteSignalHubSignals,
  pauseSignalHubSignals,
  resumeSignalHubSignals,
  fetchSignalHubRuntimeStates,
  fetchSignalHubSources,
  removeSignalHubConnection,
  removeSignalHubProfile,
  runSignalHubHealthCheck,
  unmuteSignalHubSignals,
  updateSignalHubConnection,
  updateSignalHubProfile,
  updateSignalHubRuntimeState
} from '../api/signalHub'

export const signalHubKeys = {
  all: ['signal-hub'] as const,
  sources: () => [...signalHubKeys.all, 'sources'] as const,
  capabilities: () => [...signalHubKeys.all, 'capabilities'] as const,
  connections: () => [...signalHubKeys.all, 'connections'] as const,
  runtimes: () => [...signalHubKeys.all, 'runtimes'] as const,
  health: () => [...signalHubKeys.all, 'health'] as const,
  replay: () => [...signalHubKeys.all, 'replay'] as const,
  policies: () => [...signalHubKeys.all, 'policies'] as const,
  profiles: () => [...signalHubKeys.all, 'profiles'] as const
}

export function useSignalHubSourcesQuery() {
  return useQuery({
    queryKey: signalHubKeys.sources(),
    queryFn: fetchSignalHubSources
  })
}

export function useSignalHubCapabilitiesQuery() {
  return useQuery({
    queryKey: signalHubKeys.capabilities(),
    queryFn: fetchSignalHubCapabilities
  })
}

export function useSignalHubConnectionsQuery() {
  return useQuery({
    queryKey: signalHubKeys.connections(),
    queryFn: fetchSignalHubConnections
  })
}

export function useSignalHubProfilesQuery() {
  return useQuery({
    queryKey: signalHubKeys.profiles(),
    queryFn: fetchSignalHubProfiles
  })
}

export function useApplySignalHubProfileMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: applySignalHubProfile,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: signalHubKeys.all })
    }
  })
}

export function useCreateSignalHubProfileMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: createSignalHubProfile,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: signalHubKeys.all })
    }
  })
}

export function useUpdateSignalHubProfileMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({
      profileCode,
      request
    }: {
      profileCode: string
      request: Parameters<typeof updateSignalHubProfile>[1]
    }) => updateSignalHubProfile(profileCode, request),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: signalHubKeys.all })
    }
  })
}

export function useRemoveSignalHubProfileMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: removeSignalHubProfile,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: signalHubKeys.all })
    }
  })
}

export function useCreateSignalHubConnectionMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: createSignalHubConnection,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: signalHubKeys.all })
    }
  })
}

export function useUpdateSignalHubConnectionMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({
      connectionId,
      request
    }: {
      connectionId: string
      request: Parameters<typeof updateSignalHubConnection>[1]
    }) => updateSignalHubConnection(connectionId, request),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: signalHubKeys.all })
    }
  })
}

export function useRemoveSignalHubConnectionMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: removeSignalHubConnection,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: signalHubKeys.all })
    }
  })
}

export function useSignalHubHealthQuery() {
  return useQuery({
    queryKey: signalHubKeys.health(),
    queryFn: fetchSignalHubHealth
  })
}

export function useRunSignalHubHealthCheckMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: runSignalHubHealthCheck,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: signalHubKeys.all })
    }
  })
}

export function useSignalHubRuntimeStatesQuery() {
  return useQuery({
    queryKey: signalHubKeys.runtimes(),
    queryFn: fetchSignalHubRuntimeStates
  })
}

export function useSignalHubReplayRequestsQuery() {
  return useQuery({
    queryKey: signalHubKeys.replay(),
    queryFn: fetchSignalHubReplayRequests
  })
}

export function useCreateSignalHubReplayRequestMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: createSignalHubReplayRequest,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: signalHubKeys.all })
    }
  })
}

export function useSignalHubPoliciesQuery() {
  return useQuery({
    queryKey: signalHubKeys.policies(),
    queryFn: fetchSignalHubPolicies
  })
}

export function useCreateSignalHubPolicyMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: createSignalHubPolicy,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: signalHubKeys.all })
    }
  })
}

export function useEnableSignalHubSourceMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: enableSignalHubSource,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: signalHubKeys.all })
    }
  })
}

export function useDisableSignalHubSourceMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: disableSignalHubSource,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: signalHubKeys.all })
    }
  })
}

export function useDisableSignalHubMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: disableSignalHubSignals,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: signalHubKeys.all })
    }
  })
}

export function useEnableSignalHubMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: enableSignalHubSignals,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: signalHubKeys.all })
    }
  })
}

export function useMuteSignalHubMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: muteSignalHubSignals,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: signalHubKeys.all })
    }
  })
}

export function useUnmuteSignalHubMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: unmuteSignalHubSignals,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: signalHubKeys.all })
    }
  })
}

export function usePauseSignalHubMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: pauseSignalHubSignals,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: signalHubKeys.all })
    }
  })
}

export function useResumeSignalHubMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: resumeSignalHubSignals,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: signalHubKeys.all })
    }
  })
}

export function useUpdateSignalHubRuntimeStateMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: updateSignalHubRuntimeState,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: signalHubKeys.all })
    }
  })
}
