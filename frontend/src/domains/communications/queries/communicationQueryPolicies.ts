const SECOND_MS = 1000

export const communicationRealtimeQueryOptions = {
  staleTime: 10 * SECOND_MS,
  refetchInterval: 60 * SECOND_MS,
  refetchOnReconnect: true,
  refetchOnWindowFocus: true
} as const

export const communicationDetailQueryOptions = {
  staleTime: 30 * SECOND_MS,
  refetchOnReconnect: true,
  refetchOnWindowFocus: true
} as const

export const communicationReferenceQueryOptions = {
  staleTime: 5 * 60 * SECOND_MS,
  refetchOnReconnect: true,
  refetchOnWindowFocus: false
} as const
