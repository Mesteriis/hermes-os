import { ApiClient } from '../api/ApiClient'
import type { EventListResponse, EventTrace, StoredEventEnvelope } from './types'

export interface FetchEventsOptions {
  afterPosition?: number
  limit?: number
  waitSeconds?: number
}

export async function fetchEvents({
  afterPosition = 0,
  limit = 100,
  waitSeconds = 0
}: FetchEventsOptions = {}): Promise<EventListResponse> {
  const params = new URLSearchParams()
  params.set('after_position', String(nonNegativeInteger(afterPosition, 0)))
  params.set('limit', String(clampedInteger(limit, 1, 1000, 100)))
  params.set('wait_seconds', String(clampedInteger(waitSeconds, 0, 30, 0)))

  return ApiClient.instance.get<EventListResponse>(
    `/api/v1/events?${params.toString()}`,
    'Failed to fetch event log'
  )
}

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
  return `?limit=${clampedInteger(limit, 1, 1000, 1000)}`
}

function clampedInteger(value: number, min: number, max: number, fallback: number): number {
  const normalized = Number.isFinite(value) ? Math.trunc(value) : fallback
  return Math.min(Math.max(normalized, min), max)
}

function nonNegativeInteger(value: number, fallback: number): number {
  const normalized = Number.isFinite(value) ? Math.trunc(value) : fallback
  return Math.max(normalized, 0)
}
