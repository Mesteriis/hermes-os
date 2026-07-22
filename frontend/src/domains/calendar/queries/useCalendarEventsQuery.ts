import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import {
  createCalendarEvent,
  fetchCalendarAccounts,
  fetchCalendarEvents,
  fetchCalendarSources,
  fetchEventAgenda,
  fetchEventBrief,
  fetchEventContextPack,
  fetchWeeklyBrief,
  searchCalendarEvents,
} from '../api/calendar'
import type {
  CalendarAccount,
  CalendarEvent,
  CalendarSource,
  EventAgenda,
  EventContextPack,
  WeeklyBrief,
} from '../types/calendar'

export const calendarQueryKeys = {
  accounts: ['calendar-accounts'] as const,
  events: (limit: number) => ['calendar-events', limit] as const,
  sources: (accountIds: string[]) => ['calendar-sources', ...accountIds] as const,
  weeklyBrief: ['calendar-weekly-brief'] as const,
  eventContext: (eventId: string | null) => ['calendar-event-context', eventId ?? 'none'] as const,
  eventBrief: (eventId: string | null) => ['calendar-event-brief', eventId ?? 'none'] as const,
  eventAgenda: (eventId: string | null) => ['calendar-event-agenda', eventId ?? 'none'] as const,
}

export function useCalendarAccountsQuery() {
  return useQuery<CalendarAccount[]>({
    queryKey: calendarQueryKeys.accounts,
    queryFn: async () => {
      const res = await fetchCalendarAccounts()
      return res.items
    }
  })
}

export function useCalendarEventsQuery(limit = 200) {
  return useQuery<CalendarEvent[]>({
    queryKey: calendarQueryKeys.events(limit),
    queryFn: async () => {
      const res = await fetchCalendarEvents({ limit })
      return res.items
    }
  })
}

export function useCalendarSourcesQuery(accountIds: MaybeRefOrGetter<string[]>) {
  return useQuery<CalendarSource[]>({
    queryKey: computed(() => calendarQueryKeys.sources(toValue(accountIds))),
    queryFn: async () => {
      const ids = toValue(accountIds)
      const responses = await Promise.all(ids.map((accountId) => fetchCalendarSources(accountId)))
      return responses.flatMap((response) => response.items)
    },
    enabled: computed(() => toValue(accountIds).length > 0),
  })
}

export function useCalendarWeeklyBriefQuery() {
  return useQuery<WeeklyBrief | null>({
    queryKey: calendarQueryKeys.weeklyBrief,
    queryFn: async () => {
      return fetchWeeklyBrief()
    },
  })
}

export function useCalendarEventContextPackQuery(
  eventId: MaybeRefOrGetter<string | null | undefined>
) {
  return useQuery<EventContextPack | null>({
    queryKey: computed(() => calendarQueryKeys.eventContext(toValue(eventId) ?? null)),
    queryFn: async () => {
      const value = toValue(eventId)
      if (!value) return null
      return fetchEventContextPack(value)
    },
    enabled: computed(() => Boolean(toValue(eventId))),
  })
}

export function useCalendarEventBriefQuery(
  eventId: MaybeRefOrGetter<string | null | undefined>
) {
  return useQuery<Record<string, unknown> | null>({
    queryKey: computed(() => calendarQueryKeys.eventBrief(toValue(eventId) ?? null)),
    queryFn: async () => {
      const value = toValue(eventId)
      if (!value) return null
      return fetchEventBrief(value)
    },
    enabled: computed(() => Boolean(toValue(eventId))),
  })
}

export function useCalendarEventAgendaQuery(
  eventId: MaybeRefOrGetter<string | null | undefined>
) {
  return useQuery<EventAgenda | null>({
    queryKey: computed(() => calendarQueryKeys.eventAgenda(toValue(eventId) ?? null)),
    queryFn: async () => {
      const value = toValue(eventId)
      if (!value) return null
      return fetchEventAgenda(value)
    },
    enabled: computed(() => Boolean(toValue(eventId))),
  })
}

export function useSearchCalendarEventsMutation() {
  return useMutation({
    mutationFn: async (query: string) => {
      return searchCalendarEvents(query)
    },
  })
}

export function useCreateCalendarEventMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: createCalendarEvent,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['calendar-events'] })
      queryClient.invalidateQueries({ queryKey: calendarQueryKeys.weeklyBrief })
    },
  })
}
