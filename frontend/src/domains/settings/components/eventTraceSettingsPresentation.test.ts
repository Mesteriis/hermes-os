import { describe, expect, it } from 'vitest'
import {
  buildRecentEventRows,
  buildTraceEventRows,
  buildTraceGraph,
  buildTraceSummaryTiles,
  filterRecentEventRows,
  filterTraceEventRows,
  paginateTraceRows,
  traceNodeDetail
} from './eventTraceSettingsPresentation'
import type { EventTrace, StoredEventEnvelope } from '../../../platform/event-tracing'

describe('eventTraceSettingsPresentation', () => {
  it('builds a causal graph from event trace edges', () => {
    const trace = sampleTrace()
    const graph = buildTraceGraph(trace)

    expect(graph.nodes).toHaveLength(3)
    expect(graph.edges).toHaveLength(2)
    expect(graph.nodes.find((node) => node.eventId === 'event-root')?.tone).toBe('root')
    expect(graph.nodes.find((node) => node.eventId === 'event-child-2')?.x).toBeGreaterThan(
      graph.nodes.find((node) => node.eventId === 'event-root')?.x ?? 0
    )
  })

  it('summarizes trace rows and selected node details without payload bodies', () => {
    const trace = sampleTrace()
    const rows = buildTraceEventRows(trace)
    const detail = traceNodeDetail(trace, 'event-child-2')
    const summary = buildTraceSummaryTiles(trace, trace.events)

    expect(rows.find((row) => row.eventId === 'event-child-2')?.annotationLabel).toBe('1 dead letters')
    expect(rows.find((row) => row.eventId === 'event-child-2')?.tone).toBe('bad')
    expect(detail?.deadLetterLabel).toBe('projection failed')
    expect(summary.find((tile) => tile.id === 'issues')?.value).toBe('1')
  })

  it('builds recent event seed rows from stored envelopes', () => {
    const rows = buildRecentEventRows(sampleTrace().events)

    expect(rows[0]).toMatchObject({
      eventId: 'event-root',
      correlationId: 'trace-1',
      sourceLabel: 'mail'
    })
  })

  it('filters and paginates trace data rows for settings tabs', () => {
    const traceRows = buildTraceEventRows(sampleTrace())
    const recentRows = buildRecentEventRows(sampleTrace().events)

    expect(filterTraceEventRows(traceRows, 'recorded')).toHaveLength(1)
    expect(filterRecentEventRows(recentRows, 'trace-1')).toHaveLength(3)
    expect(paginateTraceRows(traceRows, 2, 2)).toMatchObject({
      page: 2,
      pageCount: 2,
      totalCount: 3,
      startIndex: 3,
      endIndex: 3,
      hasPrevious: true,
      hasNext: false
    })
    expect(paginateTraceRows([], 4, 20)).toMatchObject({
      page: 1,
      pageCount: 1,
      totalCount: 0,
      startIndex: 0,
      endIndex: 0
    })
  })
})

function sampleTrace(): EventTrace {
  return {
    correlation_id: 'trace-1',
    root_event_ids: ['event-root'],
    events: [
      storedEvent(1, 'event-root', 'observation.captured.v1', null),
      storedEvent(2, 'event-child-1', 'signal.raw.mail.message', 'event-root'),
      storedEvent(3, 'event-child-2', 'communication.message.recorded', 'event-child-1')
    ],
    edges: [
      { parent_event_id: 'event-root', child_event_id: 'event-child-1' },
      { parent_event_id: 'event-child-1', child_event_id: 'event-child-2' }
    ],
    orphan_event_ids: [],
    missing_parent_ids: [],
    consumer_annotations: [
      {
        event_id: 'event-child-1',
        consumer_name: 'signal_hub_raw_signal_dispatcher',
        status: 'completed',
        processed_at: '2026-07-08T10:00:03Z',
        attempts: 1
      }
    ],
    dead_letters: [
      {
        event_id: 'event-child-2',
        consumer_name: 'communication_projection',
        reason: 'projection failed',
        failed_at: '2026-07-08T10:00:04Z'
      }
    ]
  }
}

function storedEvent(
  position: number,
  eventId: string,
  eventType: string,
  causationId: string | null
): StoredEventEnvelope {
  return {
    position,
    event: {
      event_id: eventId,
      event_type: eventType,
      schema_version: 1,
      occurred_at: '2026-07-08T10:00:00Z',
      recorded_at: '2026-07-08T10:00:01Z',
      source: { kind: 'mail' },
      actor: null,
      subject: { kind: 'message' },
      payload: { redacted: true },
      provenance: { source: 'fixture' },
      causation_id: causationId,
      correlation_id: 'trace-1'
    }
  }
}
