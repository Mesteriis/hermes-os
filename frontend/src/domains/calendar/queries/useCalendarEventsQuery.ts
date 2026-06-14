import { useQuery } from '@tanstack/vue-query'
import { fetchCalendarAccounts, fetchCalendarEvents } from '../api/calendar'
import type { CalendarAccount, CalendarEvent } from '../types/calendar'

export function useCalendarAccountsQuery() {
  return useQuery<CalendarAccount[]>({
    queryKey: ['calendar-accounts'],
    queryFn: async () => {
      const res = await fetchCalendarAccounts()
      return res.items
    }
  })
}

export function useCalendarEventsQuery(limit = 200) {
  return useQuery<CalendarEvent[]>({
    queryKey: ['calendar-events', limit],
    queryFn: async () => {
      const res = await fetchCalendarEvents({ limit })
      return res.items
    }
  })
}
