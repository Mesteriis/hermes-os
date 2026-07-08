import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../api/ApiClient'
import {
  fetchEventChildren,
  fetchEvents,
  fetchEventTraceByCorrelationId,
  fetchEventTraceByEventId
} from './api'

describe('event tracing API', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    ApiClient.resetForTests()
  })

  it('fetches traces by event id and correlation id through platform endpoints', async () => {
    const fetchMock = vi.fn().mockImplementation(async () =>
      new Response(JSON.stringify(emptyTrace()), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await fetchEventTraceByEventId('event:v1:root', 25)
    await fetchEventTraceByCorrelationId('trace:root', 50)

    expect(fetchMock.mock.calls[0][0]).toBe(
      'http://127.0.0.1:8080/api/v1/events/event%3Av1%3Aroot/trace?limit=25'
    )
    expect(fetchMock.mock.calls[1][0]).toBe(
      'http://127.0.0.1:8080/api/v1/event-traces/trace%3Aroot?limit=50'
    )
  })

  it('fetches event children and clamps invalid limits', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify([]), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await fetchEventChildren('event:v1:parent', 5000)

    expect(fetchMock.mock.calls[0][0]).toBe(
      'http://127.0.0.1:8080/api/v1/events/event%3Av1%3Aparent/children?limit=1000'
    )
  })

  it('fetches event log batches through the platform event endpoint', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ items: [], next_after_position: 0, has_more: false }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await fetchEvents({ afterPosition: -10, limit: 5000, waitSeconds: 99 })

    expect(fetchMock.mock.calls[0][0]).toBe(
      'http://127.0.0.1:8080/api/v1/events?after_position=0&limit=1000&wait_seconds=30'
    )
  })
})

function emptyTrace() {
  return {
    correlation_id: 'trace:root',
    root_event_ids: [],
    events: [],
    edges: [],
    orphan_event_ids: [],
    missing_parent_ids: [],
    consumer_annotations: [],
    dead_letters: []
  }
}
