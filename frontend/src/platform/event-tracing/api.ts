import { ApiClient } from '../api/ApiClient'
import type { EventTrace, StoredEventEnvelope } from './types'

export async function fetchEventTraceByEventId(
  eventId: string,
  limit = 1000
): Promise<EventTrace> {
  return ApiClient.instance.get<EventTrace>(
    `/api/v1/events/${encodeURIComponent(requiredIdentifier('eventId', eventId))}/trace${limitQuery(limit)}`,
    'Failed to fetch event trace'
  )
}

export async function fetchEventTraceByCorrelationId(
  correlationId: string,
  limit = 1000
): Promise<EventTrace> {
  return ApiClient.instance.get<EventTrace>(
    `/api/v1/event-traces/${encodeURIComponent(requiredIdentifier('correlationId', correlationId))}${limitQuery(limit)}`,
    'Failed to fetch event trace'
  )
}

export async function fetchEventChildren(
  eventId: string,
  limit = 1000
): Promise<StoredEventEnvelope[]> {
  return ApiClient.instance.get<StoredEventEnvelope[]>(
    `/api/v1/events/${encodeURIComponent(requiredIdentifier('eventId', eventId))}/children${limitQuery(limit)}`,
    'Failed to fetch event children'
  )
}

function requiredIdentifier(name: string, value: string): string {
  const trimmed = value.trim()
  if (trimmed.length === 0) {
    throw new Error(`${name} cannot be empty`)
  }
  return trimmed
}

function limitQuery(limit: number): string {
  const normalized = Number.isFinite(limit) ? Math.trunc(limit) : 1000
  const clamped = Math.min(Math.max(normalized, 1), 1000)
  return `?limit=${clamped}`
}
