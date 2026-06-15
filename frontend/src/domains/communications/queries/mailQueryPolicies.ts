const SECOND_MS = 1000

export const mailRealtimeQueryOptions = {
  staleTime: 10 * SECOND_MS,
  refetchInterval: 60 * SECOND_MS,
  refetchOnReconnect: true,
  refetchOnWindowFocus: true
} as const

export const mailDetailQueryOptions = {
  staleTime: 30 * SECOND_MS,
  refetchOnReconnect: true,
  refetchOnWindowFocus: true
} as const

export const mailReferenceQueryOptions = {
  staleTime: 5 * 60 * SECOND_MS,
  refetchOnReconnect: true,
  refetchOnWindowFocus: false
} as const
