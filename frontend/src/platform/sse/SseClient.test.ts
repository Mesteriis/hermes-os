import { afterEach, describe, expect, it, vi } from 'vitest'
import { SseClient } from './SseClient'

describe('SseClient', () => {
  afterEach(() => {
    vi.unstubAllGlobals()
    vi.useRealTimers()
  })

  it('reports protected SSE transport status transitions', async () => {
    vi.useFakeTimers()
    const statuses: string[] = []
    const stream = new ReadableStream<Uint8Array>({
      start(controller) {
        controller.enqueue(new TextEncoder().encode('id: 42\nevent: event\ndata: {"ok":true}\n\n'))
        controller.close()
      }
    })
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(stream, {
        status: 200,
        headers: { 'Content-Type': 'text/event-stream' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    const received = new Promise<void>((resolve) => {
      const client = new SseClient({
        url: 'http://127.0.0.1:8080/api/events/stream',
        secret: 'test-secret',
        reconnectDelay: 60_000,
        onStatus: (status) => statuses.push(`${status.transport}:${status.state}`),
        onMessage: () => {
          client.disconnect()
          resolve()
        }
      })
      client.connect()
    })

    await received

    expect(statuses).toEqual(['sse:connecting', 'sse:connected', 'sse:disconnected'])
  })

  it('connects with the local API secret and replays from the last event id', async () => {
    vi.useFakeTimers()
    const stream = new ReadableStream<Uint8Array>({
      start(controller) {
        controller.enqueue(new TextEncoder().encode('id: 42\nevent: event\ndata: {"ok":true}\n\n'))
        controller.close()
      }
    })
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(stream, {
        status: 200,
        headers: { 'Content-Type': 'text/event-stream' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    const received = new Promise<{ id: string; event: string; data: string }>((resolve) => {
      const client = new SseClient({
        url: 'http://127.0.0.1:8080/api/events/stream',
        secret: 'test-secret',
        lastEventId: '41',
        reconnectDelay: 60_000,
        onMessage: (event) => {
          resolve(event)
          client.disconnect()
        }
      })
      client.connect()
    })

    await expect(received).resolves.toEqual({
      id: '42',
      event: 'event',
      data: '{"ok":true}'
    })
    expect(fetchMock).toHaveBeenCalledOnce()
    const [url, init] = fetchMock.mock.calls[0]
    expect(url).toBe('http://127.0.0.1:8080/api/events/stream?after_position=41')
    expect(init.headers).toMatchObject({
      Accept: 'text/event-stream',
      'X-Hermes-Secret': 'test-secret',
      'Last-Event-ID': '41'
    })
  })

  it('builds replay requests for relative stream URLs', async () => {
    vi.useFakeTimers()
    vi.stubGlobal('location', { origin: 'http://127.0.0.1:5174' })
    const stream = new ReadableStream<Uint8Array>({
      start(controller) {
        controller.enqueue(new TextEncoder().encode('id: 10\nevent: event\ndata: {}\n\n'))
        controller.close()
      }
    })
    const fetchMock = vi.fn().mockResolvedValue(new Response(stream, { status: 200 }))
    vi.stubGlobal('fetch', fetchMock)

    const received = new Promise<void>((resolve) => {
      const client = new SseClient({
        url: '/api/events/stream?source=frontend',
        secret: 'test-secret',
        lastEventId: '9',
        reconnectDelay: 60_000,
        onMessage: () => {
          client.disconnect()
          resolve()
        }
      })
      client.connect()
    })

    await received
    const [url] = fetchMock.mock.calls[0]
    expect(url).toBe('http://127.0.0.1:5174/api/events/stream?source=frontend&after_position=9')
  })

  it('falls back to protected long polling after SSE reconnect attempts are exhausted', async () => {
    vi.useFakeTimers()
    const statuses: string[] = []
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(new Response('stream unavailable', { status: 503 }))
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            items: [
              {
                position: 42,
                event: {
                  event_id: 'evt-long-poll',
                  event_type: 'mail.ai_state.changed',
                  payload: { ai_state: 'PROCESSING' }
                }
              }
            ],
            next_after_position: 42,
            has_more: false
          }),
          {
            status: 200,
            headers: { 'Content-Type': 'application/json' }
          }
        )
      )
    vi.stubGlobal('fetch', fetchMock)

    const received = new Promise<{ id: string; event: string; data: string }>((resolve) => {
      const client = new SseClient({
        url: 'http://127.0.0.1:8080/api/events/stream',
        longPollUrl: 'http://127.0.0.1:8080/api/v1/events',
        secret: 'test-secret',
        lastEventId: '41',
        maxReconnectAttempts: 0,
        longPollDelay: 60_000,
        longPollWaitSeconds: 15,
        onStatus: (status) => statuses.push(`${status.transport}:${status.state}`),
        onMessage: (event) => {
          resolve(event)
          client.disconnect()
        }
      })
      client.connect()
    })

    await expect(received).resolves.toEqual({
      id: '42',
      event: 'event',
      data: JSON.stringify({
        position: 42,
        event: {
          event_id: 'evt-long-poll',
          event_type: 'mail.ai_state.changed',
          payload: { ai_state: 'PROCESSING' }
        }
      })
    })
    expect(fetchMock).toHaveBeenCalledTimes(2)
    expect(fetchMock.mock.calls[1][0]).toBe(
      'http://127.0.0.1:8080/api/v1/events?after_position=41&limit=100&wait_seconds=15'
    )
    expect(fetchMock.mock.calls[1][1].headers).toMatchObject({
      Accept: 'application/json',
      'X-Hermes-Secret': 'test-secret',
      'Last-Event-ID': '41'
    })
    expect(statuses).toContain('long_poll:fallback')
    expect(statuses).toContain('long_poll:connected')
  })
})
