import { useQuery } from '@tanstack/vue-query'
import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import {
  fetchEvents,
  fetchEventChildren,
  fetchEventTraceByCorrelationId,
  fetchEventTraceByEventId
} from './api'
import type { EventListResponse, EventTrace, StoredEventEnvelope } from './types'

export const eventTraceQueryKeys = {
  events: (afterPosition: number, limit: number) => ['events', 'list', afterPosition, limit] as const,
  byEvent: (eventId: string) => ['events', eventId, 'trace'] as const,
  byCorrelation: (correlationId: string) => ['event-traces', correlationId] as const,
  children: (eventId: string) => ['events', eventId, 'children'] as const
}

export function useEventsQuery(
  afterPosition: MaybeRefOrGetter<number> = 0,
  limit: MaybeRefOrGetter<number> = 100
) {
  return useQuery<EventListResponse>({
    queryKey: computed(() =>
      eventTraceQueryKeys.events(normalizePosition(toValue(afterPosition)), normalizeLimit(toValue(limit)))
    ),
    queryFn: () =>
      fetchEvents({
        afterPosition: normalizePosition(toValue(afterPosition)),
        limit: normalizeLimit(toValue(limit)),
        waitSeconds: 0
      }),
    staleTime: 10_000
  })
}

export function useEventTraceByEventIdQuery(
  eventId: MaybeRefOrGetter<string | null | undefined>,
  limit: MaybeRefOrGetter<number> = 1000
) {
  return useQuery<EventTrace | null>({
    queryKey: computed(() => {
      const id = normalizeIdentifier(toValue(eventId))
      return id ? eventTraceQueryKeys.byEvent(id) : ['events', null, 'trace'] as const
    }),
    queryFn: async () => {
      const id = normalizeIdentifier(toValue(eventId))
      if (!id) return null
      return fetchEventTraceByEventId(id, toValue(limit))
    },
    enabled: computed(() => Boolean(normalizeIdentifier(toValue(eventId)))),
    staleTime: 10_000
  })
}

export function useEventTraceByCorrelationIdQuery(
  correlationId: MaybeRefOrGetter<string | null | undefined>,
  limit: MaybeRefOrGetter<number> = 1000
) {
  return useQuery<EventTrace | null>({
    queryKey: computed(() => {
      const id = normalizeIdentifier(toValue(correlationId))
      return id ? eventTraceQueryKeys.byCorrelation(id) : ['event-traces', null] as const
    }),
    queryFn: async () => {
      const id = normalizeIdentifier(toValue(correlationId))
      if (!id) return null
      return fetchEventTraceByCorrelationId(id, toValue(limit))
    },
    enabled: computed(() => Boolean(normalizeIdentifier(toValue(correlationId)))),
    staleTime: 10_000
  })
}

export function useEventChildrenQuery(
  eventId: MaybeRefOrGetter<string | null | undefined>,
  limit: MaybeRefOrGetter<number> = 1000
) {
  return useQuery<StoredEventEnvelope[]>({
    queryKey: computed(() => {
      const id = normalizeIdentifier(toValue(eventId))
      return id ? eventTraceQueryKeys.children(id) : ['events', null, 'children'] as const
    }),
    queryFn: async () => {
      const id = normalizeIdentifier(toValue(eventId))
      if (!id) return []
      return fetchEventChildren(id, toValue(limit))
    },
    enabled: computed(() => Boolean(normalizeIdentifier(toValue(eventId)))),
    staleTime: 10_000
  })
}

function normalizeIdentifier(value: string | null | undefined): string | null {
  const trimmed = value?.trim()
  return trimmed && trimmed.length > 0 ? trimmed : null
}

function normalizePosition(value: number): number {
  if (!Number.isFinite(value)) return 0
  return Math.max(Math.trunc(value), 0)
}

function normalizeLimit(value: number): number {
  if (!Number.isFinite(value)) return 100
  return Math.min(Math.max(Math.trunc(value), 1), 1000)
}
