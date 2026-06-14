import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  CalendarAccountsResponse,
  CalendarSourcesResponse,
  CalendarEventsResponse,
  CalendarEvent,
  CalendarAccount,
  EventParticipantsResponse,
  EventContextPack,
  EventAgenda,
  MeetingNotesResponse,
  MeetingOutcomesResponse,
  DeadlinesResponse,
  CalendarFetchParams
} from '../types/calendar'

export async function fetchCalendarAccounts(provider?: string): Promise<CalendarAccountsResponse> {
  const params = new URLSearchParams()
  if (provider) params.set('provider', provider)
  return ApiClient.instance.get<CalendarAccountsResponse>(
    `/api/v1/calendar/accounts?${params.toString()}`,
    'Calendar accounts request failed'
  )
}

export async function createCalendarAccount(
  body: { provider: string; account_name: string; email?: string }
): Promise<CalendarAccount> {
  return ApiClient.instance.post<CalendarAccount>(
    '/api/v1/calendar/accounts',
    body,
    'Create calendar account failed'
  )
}

export async function fetchCalendarSources(accountId: string): Promise<CalendarSourcesResponse> {
  return ApiClient.instance.get<CalendarSourcesResponse>(
    `/api/v1/calendar/accounts/${encodeURIComponent(accountId)}/sources`,
    'Calendar sources request failed'
  )
}

export async function fetchCalendarEvents(
  params: CalendarFetchParams = {}
): Promise<CalendarEventsResponse> {
  const sp = new URLSearchParams()
  if (params.account_id) sp.set('account_id', params.account_id)
  if (params.source_id) sp.set('source_id', params.source_id)
  if (params.from) sp.set('from', params.from)
  if (params.to) sp.set('to', params.to)
  if (params.status) sp.set('status', params.status)
  if (params.event_type) sp.set('event_type', params.event_type)
  if (params.limit) sp.set('limit', String(params.limit))
  return ApiClient.instance.get<CalendarEventsResponse>(
    `/api/v1/calendar/events?${sp.toString()}`,
    'Calendar events request failed'
  )
}

export async function createCalendarEvent(
  body: {
    title: string
    start_at: string
    end_at: string
    description?: string
    location?: string
    event_type?: string
    account_id?: string
    source_id?: string
    timezone?: string
    all_day?: boolean
  }
): Promise<CalendarEvent> {
  return ApiClient.instance.post<CalendarEvent>('/api/v1/calendar/events', body, 'Create event failed')
}

export async function deleteCalendarEvent(eventId: string): Promise<{ deleted: boolean }> {
  return ApiClient.instance.delete<{ deleted: boolean }>(
    `/api/v1/calendar/events/${encodeURIComponent(eventId)}`,
    'Delete event failed'
  )
}

export async function fetchEventParticipants(eventId: string): Promise<EventParticipantsResponse> {
  return ApiClient.instance.get<EventParticipantsResponse>(
    `/api/v1/calendar/events/${encodeURIComponent(eventId)}/participants`,
    'Participants request failed'
  )
}

export async function fetchEventContextPack(eventId: string): Promise<EventContextPack | null> {
  return ApiClient.instance.get<EventContextPack | null>(
    `/api/v1/calendar/events/${encodeURIComponent(eventId)}/context-pack`,
    'Context pack request failed'
  )
}

export async function fetchEventAgenda(eventId: string): Promise<EventAgenda | null> {
  return ApiClient.instance.get<EventAgenda | null>(
    `/api/v1/calendar/events/${encodeURIComponent(eventId)}/agenda`,
    'Agenda request failed'
  )
}

export async function fetchEventBrief(eventId: string): Promise<Record<string, unknown>> {
  return ApiClient.instance.get<Record<string, unknown>>(
    `/api/v1/calendar/events/${encodeURIComponent(eventId)}/brief`,
    'Brief request failed'
  )
}

export async function fetchMeetingNotes(eventId: string): Promise<MeetingNotesResponse> {
  return ApiClient.instance.get<MeetingNotesResponse>(
    `/api/v1/calendar/events/${encodeURIComponent(eventId)}/notes`,
    'Notes request failed'
  )
}

export async function fetchMeetingOutcomes(eventId: string): Promise<MeetingOutcomesResponse> {
  return ApiClient.instance.get<MeetingOutcomesResponse>(
    `/api/v1/calendar/events/${encodeURIComponent(eventId)}/outcomes`,
    'Outcomes request failed'
  )
}

export async function fetchDeadlines(status?: string): Promise<DeadlinesResponse> {
  const params = new URLSearchParams()
  if (status) params.set('status', status)
  return ApiClient.instance.get<DeadlinesResponse>(
    `/api/v1/calendar/deadlines?${params.toString()}`,
    'Deadlines request failed'
  )
}

export async function fetchCalendarWatchtower(): Promise<Record<string, unknown>> {
  return ApiClient.instance.get<Record<string, unknown>>(
    '/api/v1/calendar/watchtower',
    'Watchtower request failed'
  )
}

export async function fetchWeeklyBrief(): Promise<Record<string, unknown>> {
  return ApiClient.instance.get<Record<string, unknown>>(
    '/api/v1/calendar/weekly-brief',
    'Weekly brief request failed'
  )
}

export async function searchCalendarEvents(q: string): Promise<Record<string, unknown>> {
  return ApiClient.instance.get<Record<string, unknown>>(
    `/api/v1/calendar/search?q=${encodeURIComponent(q)}`,
    'Calendar search failed'
  )
}
