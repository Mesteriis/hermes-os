export type JsonObject = Record<string, unknown>

export type EventEnvelope = {
  event_id: string
  event_type: string
  schema_version: number
  occurred_at: string
  recorded_at: string
  source: JsonObject
  actor: JsonObject | null
  subject: JsonObject
  payload: unknown
  provenance: JsonObject
  causation_id: string | null
  correlation_id: string | null
}

export type StoredEventEnvelope = {
  position: number
  event: EventEnvelope
}

export type EventTraceEdge = {
  parent_event_id: string
  child_event_id: string
}

export type EventConsumerAnnotation = {
  event_id: string
  consumer_name: string
  status: string
  processed_at: string | null
  attempts: number | null
}

export type EventDeadLetterAnnotation = {
  event_id: string
  consumer_name: string | null
  reason: string
  failed_at: string | null
}

export type EventTrace = {
  correlation_id: string
  root_event_ids: string[]
  events: StoredEventEnvelope[]
  edges: EventTraceEdge[]
  orphan_event_ids: string[]
  missing_parent_ids: string[]
  consumer_annotations: EventConsumerAnnotation[]
  dead_letters: EventDeadLetterAnnotation[]
}
