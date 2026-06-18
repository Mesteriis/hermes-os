import { afterEach, describe, expect, it, vi } from 'vitest'
import { initializeRealtime, handleRealtimeEvent } from './realtime'
import type { RealtimeClientOptions } from './realtime'
import type { SseClientOptions, WebSocketClientOptions } from '../sse'

describe('realtime bootstrap', () => {
  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('creates a protected SSE client and connects it', () => {
    const connect = vi.fn()
    const queryClient = { invalidateQueries: vi.fn() }
    const onStatus = vi.fn()
    let capturedOptions: RealtimeClientOptions | null = null

    const client = initializeRealtime(
      {
        apiBaseUrl: 'http://127.0.0.1:8080',
        apiSecret: 'test-secret',
        sseUrl: 'http://127.0.0.1:8080/api/events/stream',
        webSocketUrl: 'ws://127.0.0.1:8080/api/events/ws',
        realtimeTransport: 'sse'
      },
      queryClient,
      {
        onStatus,
        createClient: (options) => {
          capturedOptions = options
          return { connect, disconnect: vi.fn(), reconnect: vi.fn() }
        }
      }
    )

    expect(client).toBeDefined()
    expect(connect).toHaveBeenCalledOnce()
    expect(capturedOptions).not.toBeNull()
    const options = capturedOptions as unknown as SseClientOptions
    expect(options).toMatchObject({
      url: 'http://127.0.0.1:8080/api/events/stream',
      longPollUrl: 'http://127.0.0.1:8080/api/v1/events',
      secret: 'test-secret'
    })
    options.onMessage?.({ id: '10', event: 'event', data: '{}' })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['communications-mail-list']
    })
    options.onStatus?.({ transport: 'sse', state: 'connected' })
    expect(onStatus).toHaveBeenCalledWith({ transport: 'sse', state: 'connected' })
  })

  it('starts WebSocket transport first and falls back to SSE when it disconnects', () => {
    const queryClient = { invalidateQueries: vi.fn() }
    const createdOptions: RealtimeClientOptions[] = []
    const connectedUrls: string[] = []

    initializeRealtime(
      {
        apiBaseUrl: 'http://127.0.0.1:8080',
        apiSecret: 'test-secret',
        sseUrl: 'http://127.0.0.1:8080/api/events/stream',
        webSocketUrl: 'ws://127.0.0.1:8080/api/events/ws',
        realtimeTransport: 'websocket'
      },
      queryClient,
      {
        createClient: (options) => {
          createdOptions.push(options)
          return {
            connect: () => connectedUrls.push(options.url),
            disconnect: vi.fn(),
            reconnect: vi.fn()
          }
        }
      }
    )

    expect(createdOptions[0].url).toBe('ws://127.0.0.1:8080/api/events/ws')
    expect(connectedUrls).toEqual(['ws://127.0.0.1:8080/api/events/ws'])

    ;(createdOptions[0] as WebSocketClientOptions).onStatus?.({
      transport: 'websocket',
      state: 'disconnected',
      error: 'WebSocket reconnect attempts exhausted'
    })

    expect(createdOptions[1].url).toBe('http://127.0.0.1:8080/api/events/stream')
    expect(connectedUrls).toEqual([
      'ws://127.0.0.1:8080/api/events/ws',
      'http://127.0.0.1:8080/api/events/stream'
    ])
  })

  it('allows manual reconnect to prefer the primary WebSocket transport again', () => {
    const createdOptions: RealtimeClientOptions[] = []
    const connectedUrls: string[] = []
    const disconnectedUrls: string[] = []

    const client = initializeRealtime(
      {
        apiBaseUrl: 'http://127.0.0.1:8080',
        apiSecret: 'test-secret',
        sseUrl: 'http://127.0.0.1:8080/api/events/stream',
        webSocketUrl: 'ws://127.0.0.1:8080/api/events/ws',
        realtimeTransport: 'websocket'
      },
      { invalidateQueries: vi.fn() },
      {
        createClient: (options) => {
          createdOptions.push(options)
          return {
            connect: () => connectedUrls.push(options.url),
            disconnect: () => disconnectedUrls.push(options.url),
            reconnect: vi.fn()
          }
        }
      }
    )

    ;(createdOptions[0] as WebSocketClientOptions).onStatus?.({
      transport: 'websocket',
      state: 'disconnected',
      error: 'WebSocket reconnect attempts exhausted'
    })

    expect(connectedUrls).toEqual([
      'ws://127.0.0.1:8080/api/events/ws',
      'http://127.0.0.1:8080/api/events/stream'
    ])

    client.reconnect()

    expect(disconnectedUrls).toContain('http://127.0.0.1:8080/api/events/stream')
    expect(disconnectedUrls).toContain('ws://127.0.0.1:8080/api/events/ws')
    expect(connectedUrls.at(-1)).toBe('ws://127.0.0.1:8080/api/events/ws')
  })

  it('loads and persists the replay cursor', () => {
    const connect = vi.fn()
    const queryClient = { invalidateQueries: vi.fn() }
    const storage = {
      getItem: vi.fn().mockReturnValue('41'),
      setItem: vi.fn(),
      removeItem: vi.fn(),
      clear: vi.fn(),
      key: vi.fn(),
      length: 1
    }
    vi.stubGlobal('localStorage', storage)
    let capturedOptions: RealtimeClientOptions | null = null

    initializeRealtime(
      {
        apiBaseUrl: 'http://127.0.0.1:8080',
        apiSecret: 'test-secret',
        sseUrl: 'http://127.0.0.1:8080/api/events/stream',
        webSocketUrl: 'ws://127.0.0.1:8080/api/events/ws',
        realtimeTransport: 'sse'
      },
      queryClient,
      (options) => {
        capturedOptions = options
        return { connect, disconnect: vi.fn(), reconnect: vi.fn() }
      }
    )

    const options = capturedOptions as unknown as SseClientOptions
    expect(options.lastEventId).toBe('41')
    options.onMessage?.({ id: '42', event: 'event', data: '{}' })
    expect(storage.setItem).toHaveBeenCalledWith('hermes.realtime.lastEventId', '42')
  })

  it('reports lagged realtime gaps without advancing the replay cursor', () => {
    const connect = vi.fn()
    const queryClient = { invalidateQueries: vi.fn() }
    const storage = {
      getItem: vi.fn().mockReturnValue('41'),
      setItem: vi.fn(),
      removeItem: vi.fn(),
      clear: vi.fn(),
      key: vi.fn(),
      length: 1
    }
    vi.stubGlobal('localStorage', storage)
    const onLaggedObserved = vi.fn()
    let capturedOptions: RealtimeClientOptions | null = null

    initializeRealtime(
      {
        apiBaseUrl: 'http://127.0.0.1:8080',
        apiSecret: 'test-secret',
        sseUrl: 'http://127.0.0.1:8080/api/events/stream',
        webSocketUrl: 'ws://127.0.0.1:8080/api/events/ws',
        realtimeTransport: 'sse'
      },
      queryClient,
      {
        onLaggedObserved,
        createClient: (options) => {
          capturedOptions = options
          return { connect, disconnect: vi.fn(), reconnect: vi.fn() }
        }
      }
    )

    const options = capturedOptions as unknown as SseClientOptions
    options.onMessage?.({ id: '41', event: 'lagged', data: JSON.stringify({ skipped: 3 }) })

    expect(onLaggedObserved).toHaveBeenCalledWith(3)
    expect(storage.setItem).not.toHaveBeenCalled()
  })

  it('does not rewind the persisted replay cursor when an older event arrives', () => {
    const connect = vi.fn()
    const queryClient = { invalidateQueries: vi.fn() }
    const storage = {
      getItem: vi.fn().mockReturnValue('50'),
      setItem: vi.fn(),
      removeItem: vi.fn(),
      clear: vi.fn(),
      key: vi.fn(),
      length: 1
    }
    vi.stubGlobal('localStorage', storage)
    let capturedOptions: RealtimeClientOptions | null = null

    initializeRealtime(
      {
        apiBaseUrl: 'http://127.0.0.1:8080',
        apiSecret: 'test-secret',
        sseUrl: 'http://127.0.0.1:8080/api/events/stream',
        webSocketUrl: 'ws://127.0.0.1:8080/api/events/ws',
        realtimeTransport: 'sse'
      },
      queryClient,
      (options) => {
        capturedOptions = options
        return { connect, disconnect: vi.fn(), reconnect: vi.fn() }
      }
    )

    const options = capturedOptions as unknown as SseClientOptions
    options.onMessage?.({ id: '49', event: 'event', data: '{}' })
    expect(storage.setItem).not.toHaveBeenCalled()

    options.onMessage?.({ id: '51', event: 'event', data: '{}' })
    expect(storage.setItem).toHaveBeenCalledOnce()
    expect(storage.setItem).toHaveBeenCalledWith('hermes.realtime.lastEventId', '51')
  })

  it('invalidates broad communication and telegram queries when realtime reports a replay gap', () => {
    const queryClient = { invalidateQueries: vi.fn() }

    handleRealtimeEvent(
      {
        id: '52',
        event: 'lagged',
        data: JSON.stringify({ skipped: 4 })
      },
      queryClient
    )

    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['communications-mail-list']
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['telegram', 'messages']
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['telegram', 'runtime']
    })
  })

  it('invalidates targeted mail queries for AI state events', () => {
    const queryClient = { invalidateQueries: vi.fn() }

    handleRealtimeEvent(
      {
        id: '42',
        event: 'event',
        data: JSON.stringify({
          position: 42,
          event: {
            event_type: 'mail.ai_state.changed',
            subject: { id: 'msg:1' }
          }
        })
      },
      queryClient
    )

    expect(queryClient.invalidateQueries).toHaveBeenCalledTimes(3)
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['communications-message']
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['communications-ai-state']
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['communications-mail-list']
    })
  })

  it('patches cached AI state for AI state realtime events', () => {
    const setQueryData = vi.fn((queryKey, updater) =>
      typeof updater === 'function' ? updater(undefined) : updater
    )
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockReturnValue([]),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: '52',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'mail.ai_state.changed',
            payload: {
              message_id: 'msg-1',
              ai_state: 'PROCESSING',
              review_required: false,
              failed: false
            }
          }
        })
      },
      queryClient
    )

    expect(setQueryData).toHaveBeenCalledWith(
      ['communications-ai-state', 'msg-1'],
      expect.any(Function)
    )
    expect(setQueryData.mock.results[0]?.value).toMatchObject({
      message_id: 'msg-1',
      ai_state: 'PROCESSING',
      review_reason: null,
      last_error: null
    })
  })

  it('invalidates saved-search queries for saved search events', () => {
    const queryClient = { invalidateQueries: vi.fn() }

    handleRealtimeEvent(
      {
        id: '43',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'mail.saved_search.updated'
          }
        })
      },
      queryClient
    )

    expect(queryClient.invalidateQueries).toHaveBeenCalledOnce()
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['communications-saved-searches']
    })
  })

  it('invalidates only sync status queries for mail sync progress events', () => {
    const queryClient = { invalidateQueries: vi.fn() }

    handleRealtimeEvent(
      {
        id: '44',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'mail.sync.progress'
          }
        })
      },
      queryClient
    )

    expect(queryClient.invalidateQueries).toHaveBeenCalledOnce()
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['communications-sync-statuses']
    })
  })

  it('invalidates targeted message queries for mail message action events', () => {
    const queryClient = { invalidateQueries: vi.fn() }

    handleRealtimeEvent(
      {
        id: '45',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'mail.message.archived'
          }
        })
      },
      queryClient
    )

    expect(queryClient.invalidateQueries).toHaveBeenCalledTimes(7)
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['communications-message']
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['communications-mail-list']
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['communications-state-counts']
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['communications-threads']
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['communications-saved-searches']
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['communications-folders']
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['communications-folder-messages']
    })
  })

  it('patches cached mail list and message detail for local message action events', () => {
    const mailListKey = ['communications-mail-list', undefined, undefined, undefined, undefined, 'active']
    const detailKey = ['communications-message', 'msg-1'] as const
    const mailListData = {
      pages: [
        {
          items: [
            {
              message_id: 'msg-1',
              workflow_state: 'new',
              local_state: 'active',
              message_metadata: {}
            }
          ],
          next_cursor: null,
          has_more: false
        }
      ],
      pageParams: [null]
    }
    const detailData = {
      message: {
        message_id: 'msg-1',
        workflow_state: 'new',
        local_state: 'active',
        message_metadata: {}
      },
      attachments: []
    }
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockReturnValue([[mailListKey, mailListData]]),
      setQueryData: vi.fn((queryKey, updater) => {
        if (queryKey === detailKey) {
          return typeof updater === 'function' ? updater(detailData) : updater
        }
        return typeof updater === 'function' ? updater(mailListData) : updater
      })
    }

    handleRealtimeEvent(
      {
        id: '45',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'mail.message.read',
            payload: {
              action: 'mark_read',
              message_ids: ['msg-1']
            }
          }
        })
      },
      queryClient
    )

    expect(queryClient.getQueriesData).toHaveBeenCalledWith({
      queryKey: ['communications-mail-list']
    })
    expect(queryClient.setQueryData).toHaveBeenCalledWith(mailListKey, expect.any(Function))
    expect(queryClient.setQueryData).toHaveBeenCalledWith(detailKey, expect.any(Function))
  })

})
