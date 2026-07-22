import type {
  EventConsumerAnnotation,
  EventDeadLetterAnnotation,
  EventTrace,
  StoredEventEnvelope
} from '../../../platform/event-tracing'

export type TraceLookupMode = 'event' | 'trace'
export type TraceDataTab = 'trace-events' | 'recent-seeds'
export type TraceNodeTone = 'root' | 'normal' | 'warn' | 'bad'

export interface TraceModeOption<T extends string> {
  id: T
  label: string
  icon: string
}

export const traceLookupModes: readonly TraceModeOption<TraceLookupMode>[] = [
  { id: 'event', label: 'Event ID', icon: 'tabler:timeline-event' },
  { id: 'trace', label: 'Trace ID', icon: 'tabler:route' },
]

export const traceDataTabs: readonly TraceModeOption<TraceDataTab>[] = [
  { id: 'trace-events', label: 'Trace events', icon: 'tabler:timeline' },
  { id: 'recent-seeds', label: 'Event log seeds', icon: 'tabler:list-search' },
]

export interface TraceGraphNode {
  eventId: string
  eventType: string
  position: number
  x: number
  y: number
  width: number
  height: number
  tone: TraceNodeTone
  title: string
  subtitle: string
  meta: string
}

export interface TraceGraphEdge {
  id: string
  parentEventId: string
  childEventId: string
  x1: number
  y1: number
  x2: number
  y2: number
  tone: 'normal' | 'warn'
}

export interface TraceGraph {
  width: number
  height: number
  nodes: TraceGraphNode[]
  edges: TraceGraphEdge[]
}

export interface TraceSummaryTile {
  id: string
  label: string
  value: string
  detail: string
  icon: string
  tone: 'good' | 'warn' | 'bad' | 'neutral'
}

export interface TraceEventRow {
  eventId: string
  position: number
  eventType: string
  recordedAt: string
  occurredAt: string
  causationId: string
  correlationId: string
  sourceLabel: string
  subjectLabel: string
  annotationLabel: string
  tone: 'good' | 'warn' | 'bad' | 'neutral'
}

export interface RecentEventRow {
  eventId: string
  eventType: string
  position: number
  recordedAt: string
  correlationId: string
  sourceLabel: string
}

export interface TraceNodeDetail {
  eventId: string
  eventType: string
  position: string
  recordedAt: string
  occurredAt: string
  causationId: string
  correlationId: string
  sourceLabel: string
  subjectLabel: string
  provenanceLabel: string
  consumerLabel: string
  deadLetterLabel: string
}

export interface TracePagedRows<T> {
  rows: T[]
  page: number
  pageCount: number
  pageSize: number
  totalCount: number
  startIndex: number
  endIndex: number
  hasPrevious: boolean
  hasNext: boolean
}

const NODE_WIDTH = 230
const NODE_HEIGHT = 92
const COLUMN_GAP = 84
const ROW_GAP = 38
const PADDING_X = 36
const PADDING_Y = 32

export function buildTraceGraph(trace: EventTrace | null): TraceGraph {
  if (!trace || trace.events.length === 0) {
    return { width: 720, height: 260, nodes: [], edges: [] }
  }

  const eventById = new Map(trace.events.map((stored) => [stored.event.event_id, stored]))
  const childrenByParent = new Map<string, string[]>()
  for (const edge of trace.edges) {
    if (!eventById.has(edge.parent_event_id) || !eventById.has(edge.child_event_id)) continue
    const children = childrenByParent.get(edge.parent_event_id) ?? []
    children.push(edge.child_event_id)
    childrenByParent.set(edge.parent_event_id, children)
  }

  const roots = trace.root_event_ids.filter((eventId) => eventById.has(eventId))
  const fallbackRoots = trace.events
    .filter((stored) => !stored.event.causation_id || !eventById.has(stored.event.causation_id))
    .map((stored) => stored.event.event_id)
  const rootIds = uniquePreservingOrder([...roots, ...fallbackRoots, trace.events[0]?.event.event_id].filter(Boolean))
  const levelByEvent = assignTraceLevels(rootIds, childrenByParent)

  for (const stored of trace.events) {
    if (!levelByEvent.has(stored.event.event_id)) {
      const parentLevel = stored.event.causation_id ? levelByEvent.get(stored.event.causation_id) : undefined
      levelByEvent.set(stored.event.event_id, parentLevel !== undefined ? parentLevel + 1 : 0)
    }
  }

  const columns = groupEventsByLevel(trace.events, levelByEvent)
  const nodes: TraceGraphNode[] = []
  const deadLetterEventIds = new Set(trace.dead_letters.map((deadLetter) => deadLetter.event_id))
  const orphanEventIds = new Set(trace.orphan_event_ids)
  const rootEventIds = new Set(trace.root_event_ids)
  let maxRows = 1

  for (const [level, events] of columns.entries()) {
    maxRows = Math.max(maxRows, events.length)
    events.forEach((stored, row) => {
      nodes.push({
        eventId: stored.event.event_id,
        eventType: stored.event.event_type,
        position: stored.position,
        x: PADDING_X + level * (NODE_WIDTH + COLUMN_GAP),
        y: PADDING_Y + row * (NODE_HEIGHT + ROW_GAP),
        width: NODE_WIDTH,
        height: NODE_HEIGHT,
        tone: nodeTone(stored, rootEventIds, orphanEventIds, deadLetterEventIds),
        title: compactEventType(stored.event.event_type),
        subtitle: compactIdentifier(stored.event.event_id, 30),
        meta: `#${stored.position} / ${formatTraceTimestamp(stored.event.recorded_at)}`
      })
    })
  }

  const nodeById = new Map(nodes.map((node) => [node.eventId, node]))
  const edges = trace.edges
    .map((edge) => {
      const parent = nodeById.get(edge.parent_event_id)
      const child = nodeById.get(edge.child_event_id)
      if (!parent || !child) return null
      return {
        id: `${edge.parent_event_id}->${edge.child_event_id}`,
        parentEventId: edge.parent_event_id,
        childEventId: edge.child_event_id,
        x1: parent.x + parent.width,
        y1: parent.y + parent.height / 2,
        x2: child.x,
        y2: child.y + child.height / 2,
        tone: trace.missing_parent_ids.includes(edge.parent_event_id) ? 'warn' : 'normal'
      } satisfies TraceGraphEdge
    })
    .filter((edge): edge is TraceGraphEdge => edge !== null)

  return {
    width: Math.max(
      760,
      PADDING_X * 2 + Math.max(1, columns.length) * NODE_WIDTH + Math.max(0, columns.length - 1) * COLUMN_GAP
    ),
    height: Math.max(
      360,
      PADDING_Y * 2 + maxRows * NODE_HEIGHT + Math.max(0, maxRows - 1) * ROW_GAP
    ),
    nodes,
    edges
  }
}

export function buildTraceSummaryTiles(trace: EventTrace | null, recentEvents: StoredEventEnvelope[]): TraceSummaryTile[] {
  const events = trace?.events.length ?? 0
  const edges = trace?.edges.length ?? 0
  const roots = trace?.root_event_ids.length ?? 0
  const issues =
    (trace?.orphan_event_ids.length ?? 0) +
    (trace?.missing_parent_ids.length ?? 0) +
    (trace?.dead_letters.length ?? 0)

  return [
    {
      id: 'events',
      label: 'Spans',
      value: String(events),
      detail: 'Event envelopes in trace',
      icon: 'tabler:timeline-event',
      tone: events > 0 ? 'good' : 'neutral'
    },
    {
      id: 'edges',
      label: 'Edges',
      value: String(edges),
      detail: 'Causation links',
      icon: 'tabler:git-branch',
      tone: edges > 0 ? 'good' : 'neutral'
    },
    {
      id: 'roots',
      label: 'Roots',
      value: String(roots),
      detail: 'Root span ids',
      icon: 'tabler:circle-dot',
      tone: roots > 0 ? 'good' : 'neutral'
    },
    {
      id: 'issues',
      label: 'Trace issues',
      value: String(issues),
      detail: `${recentEvents.length} event-log seeds loaded`,
      icon: 'tabler:alert-triangle',
      tone: issues > 0 ? 'warn' : 'good'
    }
  ]
}

export function buildTraceEventRows(trace: EventTrace | null): TraceEventRow[] {
  if (!trace) return []
  return trace.events.map((stored) => {
    const annotations = trace.consumer_annotations.filter((annotation) => annotation.event_id === stored.event.event_id)
    const deadLetters = trace.dead_letters.filter((deadLetter) => deadLetter.event_id === stored.event.event_id)
    return {
      eventId: stored.event.event_id,
      position: stored.position,
      eventType: stored.event.event_type,
      recordedAt: formatTraceTimestamp(stored.event.recorded_at),
      occurredAt: formatTraceTimestamp(stored.event.occurred_at),
      causationId: stored.event.causation_id ?? 'root',
      correlationId: stored.event.correlation_id ?? trace.correlation_id,
      sourceLabel: objectLabel(stored.event.source),
      subjectLabel: objectLabel(stored.event.subject),
      annotationLabel: annotationSummary(annotations, deadLetters),
      tone: eventRowTone(stored, annotations, deadLetters, trace)
    }
  })
}

export function buildRecentEventRows(events: StoredEventEnvelope[]): RecentEventRow[] {
  return events.map((stored) => ({
    eventId: stored.event.event_id,
    eventType: stored.event.event_type,
    position: stored.position,
    recordedAt: formatTraceTimestamp(stored.event.recorded_at),
    correlationId: stored.event.correlation_id ?? stored.event.event_id,
    sourceLabel: objectLabel(stored.event.source)
  }))
}

export function filterTraceEventRows(rows: TraceEventRow[], query: string): TraceEventRow[] {
  return filterRows(rows, query, (row) => [
    row.eventId,
    row.eventType,
    row.correlationId,
    row.causationId,
    row.sourceLabel,
    row.subjectLabel,
    row.annotationLabel,
    row.recordedAt,
    String(row.position)
  ])
}

export function filterRecentEventRows(rows: RecentEventRow[], query: string): RecentEventRow[] {
  return filterRows(rows, query, (row) => [
    row.eventId,
    row.eventType,
    row.correlationId,
    row.sourceLabel,
    row.recordedAt,
    String(row.position)
  ])
}

export function paginateTraceRows<T>(rows: T[], page: number, pageSize: number): TracePagedRows<T> {
  const normalizedPageSize = Math.max(1, Math.trunc(pageSize))
  const pageCount = Math.max(1, Math.ceil(rows.length / normalizedPageSize))
  const normalizedPage = Math.min(Math.max(Math.trunc(page), 1), pageCount)
  const startIndex = rows.length === 0 ? 0 : (normalizedPage - 1) * normalizedPageSize + 1
  const endIndex = Math.min(rows.length, normalizedPage * normalizedPageSize)
  return {
    rows: rows.slice(startIndex > 0 ? startIndex - 1 : 0, endIndex),
    page: normalizedPage,
    pageCount,
    pageSize: normalizedPageSize,
    totalCount: rows.length,
    startIndex,
    endIndex,
    hasPrevious: normalizedPage > 1,
    hasNext: normalizedPage < pageCount
  }
}

export function traceNodeDetail(
  trace: EventTrace | null,
  eventId: string | null
): TraceNodeDetail | null {
  if (!trace || !eventId) return null
  const stored = trace.events.find((item) => item.event.event_id === eventId)
  if (!stored) return null
  const consumers = trace.consumer_annotations.filter((annotation) => annotation.event_id === eventId)
  const deadLetters = trace.dead_letters.filter((deadLetter) => deadLetter.event_id === eventId)
  return {
    eventId: stored.event.event_id,
    eventType: stored.event.event_type,
    position: String(stored.position),
    recordedAt: formatTraceTimestamp(stored.event.recorded_at),
    occurredAt: formatTraceTimestamp(stored.event.occurred_at),
    causationId: stored.event.causation_id ?? 'root',
    correlationId: stored.event.correlation_id ?? trace.correlation_id,
    sourceLabel: objectLabel(stored.event.source),
    subjectLabel: objectLabel(stored.event.subject),
    provenanceLabel: objectLabel(stored.event.provenance),
    consumerLabel: consumers.length > 0
      ? consumers.map((annotation) => `${annotation.consumer_name}:${annotation.status}`).join(', ')
      : 'none',
    deadLetterLabel: deadLetters.length > 0
      ? deadLetters.map((deadLetter) => deadLetter.reason).join(', ')
      : 'none'
  }
}

export function formatTraceTimestamp(value: string | null): string {
  if (!value) return 'n/a'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return value
  return date.toISOString().replace(/\.\d{3}Z$/, 'Z')
}

export function compactIdentifier(value: string, limit = 42): string {
  if (value.length <= limit) return value
  const head = Math.max(8, Math.floor((limit - 3) * 0.58))
  const tail = Math.max(6, limit - 3 - head)
  return `${value.slice(0, head)}...${value.slice(-tail)}`
}

function assignTraceLevels(rootIds: string[], childrenByParent: Map<string, string[]>): Map<string, number> {
  const levels = new Map<string, number>()
  const queue = rootIds.map((eventId) => ({ eventId, level: 0 }))

  while (queue.length > 0) {
    const current = queue.shift()
    if (!current) break
    const previous = levels.get(current.eventId)
    if (previous !== undefined && previous <= current.level) continue
    levels.set(current.eventId, current.level)
    for (const childId of childrenByParent.get(current.eventId) ?? []) {
      queue.push({ eventId: childId, level: current.level + 1 })
    }
  }

  return levels
}

function groupEventsByLevel(
  events: StoredEventEnvelope[],
  levelByEvent: Map<string, number>
): StoredEventEnvelope[][] {
  const columns: StoredEventEnvelope[][] = []
  for (const stored of events) {
    const level = levelByEvent.get(stored.event.event_id) ?? 0
    columns[level] = columns[level] ?? []
    columns[level].push(stored)
  }
  return columns.map((items) => [...items].sort((a, b) => a.position - b.position))
}

function nodeTone(
  stored: StoredEventEnvelope,
  rootEventIds: Set<string>,
  orphanEventIds: Set<string>,
  deadLetterEventIds: Set<string>
): TraceNodeTone {
  if (deadLetterEventIds.has(stored.event.event_id)) return 'bad'
  if (orphanEventIds.has(stored.event.event_id)) return 'warn'
  if (rootEventIds.has(stored.event.event_id)) return 'root'
  return 'normal'
}

function eventRowTone(
  stored: StoredEventEnvelope,
  annotations: EventConsumerAnnotation[],
  deadLetters: EventDeadLetterAnnotation[],
  trace: EventTrace
): TraceEventRow['tone'] {
  if (deadLetters.length > 0) return 'bad'
  if (trace.orphan_event_ids.includes(stored.event.event_id)) return 'warn'
  if (annotations.some((annotation) => annotation.status !== 'completed')) return 'warn'
  if (trace.root_event_ids.includes(stored.event.event_id)) return 'good'
  return 'neutral'
}

function annotationSummary(
  annotations: EventConsumerAnnotation[],
  deadLetters: EventDeadLetterAnnotation[]
): string {
  if (deadLetters.length > 0) return `${deadLetters.length} dead letters`
  if (annotations.length === 0) return 'no consumer annotations'
  const completed = annotations.filter((annotation) => annotation.status === 'completed').length
  return `${completed}/${annotations.length} consumers completed`
}

function compactEventType(eventType: string): string {
  const normalized = eventType.replaceAll('_', '.')
  if (normalized.length <= 34) return normalized
  const parts = normalized.split('.')
  if (parts.length <= 2) return compactIdentifier(normalized, 34)
  return compactIdentifier(`${parts[0]}.${parts.at(-2)}.${parts.at(-1)}`, 34)
}

function objectLabel(value: Record<string, unknown>): string {
  const preferred = stringField(value, ['kind', 'type', 'source_kind', 'subject_kind', 'provider', 'id'])
  if (preferred) return preferred
  const keys = Object.keys(value)
  if (keys.length === 0) return 'empty'
  return keys.slice(0, 3).join(', ')
}

function stringField(value: Record<string, unknown>, keys: string[]): string | null {
  for (const key of keys) {
    const candidate = value[key]
    if (typeof candidate === 'string' && candidate.trim().length > 0) return candidate.trim()
  }
  return null
}

function uniquePreservingOrder(values: string[]): string[] {
  const seen = new Set<string>()
  const result: string[] = []
  for (const value of values) {
    if (seen.has(value)) continue
    seen.add(value)
    result.push(value)
  }
  return result
}

function filterRows<T>(rows: T[], query: string, values: (row: T) => string[]): T[] {
  const normalizedQuery = query.trim().toLowerCase()
  if (!normalizedQuery) return rows
  return rows.filter((row) =>
    values(row).some((value) => value.toLowerCase().includes(normalizedQuery))
  )
}
