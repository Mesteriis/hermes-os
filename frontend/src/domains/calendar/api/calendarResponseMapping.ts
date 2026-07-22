import type { CalendarEvent } from '../types/calendar'

export function mapCalendarSearchResponse(value: Record<string, unknown>): CalendarEvent[] {
  if (!Array.isArray(value.results)) {
    throw new Error('Calendar search response has no results array')
  }

  const events = value.results.map(parseCalendarEvent)
  if (events.some((event) => event === null)) {
    throw new Error('Calendar search response contains an invalid event')
  }
  return events.filter((event): event is CalendarEvent => event !== null)
}

function parseCalendarEvent(value: unknown): CalendarEvent | null {
  if (!isRecord(value)) return null

  const eventId = requiredString(value.event_id)
  const title = requiredString(value.title)
  const startAt = requiredString(value.start_at)
  const endAt = requiredString(value.end_at)
  const status = requiredString(value.status)
  const visibility = requiredString(value.visibility)
  const syncStatus = requiredString(value.sync_status)
  const createdAt = requiredString(value.created_at)
  const updatedAt = requiredString(value.updated_at)
  const allDay = value.all_day
  if (
    !eventId || !title || !startAt || !endAt || !status || !visibility ||
    !syncStatus || !createdAt || !updatedAt || typeof allDay !== 'boolean'
  ) return null

  const importanceScore = nullableNumber(value.importance_score)
  const readinessScore = nullableNumber(value.readiness_score)
  if (importanceScore === undefined || readinessScore === undefined) return null

  return {
    event_id: eventId,
    source_event_id: nullableString(value.source_event_id),
    account_id: nullableString(value.account_id),
    source_id: nullableString(value.source_id),
    title,
    description: nullableString(value.description),
    location: nullableString(value.location),
    start_at: startAt,
    end_at: endAt,
    timezone: nullableString(value.timezone),
    all_day: allDay,
    recurrence_rule: nullableString(value.recurrence_rule),
    status,
    visibility,
    event_type: nullableString(value.event_type),
    importance_score: importanceScore,
    readiness_score: readinessScore,
    sync_status: syncStatus,
    created_at: createdAt,
    updated_at: updatedAt
  }
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
}

function requiredString(value: unknown): string | null {
  return typeof value === 'string' && value.length > 0 ? value : null
}

function nullableString(value: unknown): string | null {
  return value === null || value === undefined
    ? null
    : typeof value === 'string'
      ? value
      : null
}

function nullableNumber(value: unknown): number | null | undefined {
  if (value === null || value === undefined) return null
  return typeof value === 'number' && Number.isFinite(value) ? value : undefined
}
