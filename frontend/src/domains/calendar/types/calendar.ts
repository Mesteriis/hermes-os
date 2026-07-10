export interface CalendarAccount {
  account_id: string
  provider: string
  account_name: string
  email: string | null
  credentials_reference: string | null
  sync_status: string
  capabilities: Record<string, unknown>
  created_at: string
  updated_at: string
}

export interface CalendarAccountsResponse {
  items: CalendarAccount[]
}

export interface CalendarSource {
  source_id: string
  account_id: string
  provider_calendar_id: string | null
  name: string
  color: string | null
  timezone: string | null
  visibility: string
  read_only: boolean
  sync_enabled: boolean
  capabilities: Record<string, unknown>
  created_at: string
  updated_at: string
}

export interface CalendarSourcesResponse {
  items: CalendarSource[]
}

export interface CalendarEvent {
  event_id: string
  source_event_id: string | null
  account_id: string | null
  source_id: string | null
  title: string
  description: string | null
  location: string | null
  start_at: string
  end_at: string
  timezone: string | null
  all_day: boolean
  recurrence_rule: string | null
  status: string
  visibility: string
  event_type: string | null
  importance_score: number | null
  readiness_score: number | null
  sync_status: string
  created_at: string
  updated_at: string
}

export interface CalendarEventsResponse {
  items: CalendarEvent[]
}

export interface EventParticipant {
  id: string
  event_id: string
  persona_id: string | null
  email: string
  display_name: string | null
  role: string
  response_status: string
  organization_id: string | null
  timezone: string | null
  confidence: number
  created_at: string
}

export interface EventParticipantsResponse {
  items: EventParticipant[]
}

export interface EventContextPack {
  id: string
  event_id: string
  summary: string | null
  participants_summary: string | null
  documents: unknown[]
  tasks: unknown[]
  open_questions: unknown[]
  risks: unknown[]
  suggested_agenda: unknown[]
  suggested_actions: unknown[]
  generated_at: string
  model: string | null
  created_at: string
  updated_at: string
}

export interface EventAgenda {
  id: string
  event_id: string
  items: unknown[]
  source: string
  created_by: string | null
  created_at: string
  updated_at: string
}

export interface MeetingNote {
  id: string
  event_id: string
  content: string
  format: string
  source: string
  linked_note_id: string | null
  created_at: string
  updated_at: string
}

export interface MeetingNotesResponse {
  items: MeetingNote[]
}

export interface MeetingOutcome {
  id: string
  event_id: string
  outcome_type: string
  title: string
  description: string | null
  owner_person_id: string | null
  due_date: string | null
  source: string
  confidence: number
  linked_entity_id: string | null
  created_at: string
  updated_at: string
}

export interface MeetingOutcomesResponse {
  items: MeetingOutcome[]
}

export interface DeadlineEvent {
  id: string
  source_entity_type: string | null
  source_entity_id: string | null
  title: string
  due_at: string
  severity: string
  status: string
  linked_calendar_event_id: string | null
  created_at: string
  updated_at: string
}

export interface DeadlinesResponse {
  items: DeadlineEvent[]
}

export type CalendarViewMode = 'day' | 'week' | 'month' | 'agenda'

export interface WeeklyBrief {
  upcoming_events_this_week: number
  overdue_deadlines: number
  past_events_without_notes: number
  [key: string]: unknown
}

export interface CalendarFetchParams {
  account_id?: string
  source_id?: string
  from?: string
  to?: string
  status?: string
  event_type?: string
  limit?: number
}
