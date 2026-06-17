import { afterEach, describe, expect, it, vi } from 'vitest'
import { WebSocketClient } from './WebSocketClient'

type ListenerMap = Record<string, Array<(event?: { data?: unknown }) => void>>

class FakeWebSocket {
  static instances: FakeWebSocket[] = []

  url: string
  listeners: ListenerMap = {}
  closed = false

  constructor(url: string) {
    this.url = url
    FakeWebSocket.instances.push(this)
  }

  addEventListener(type: string, listener: (event?: { data?: unknown }) => void): void {
    this.listeners[type] ??= []
    this.listeners[type].push(listener)
  }

  close(): void {
    this.closed = true
  }

  emit(type: string, data?: unknown): void {
    for (const listener of this.listeners[type] ?? []) {
      listener(data === undefined ? undefined : { data })
    }
  }
}

describe('WebSocketClient', () => {
  afterEach(() => {
    FakeWebSocket.instances = []
    vi.unstubAllGlobals()
  })

  it('forwards websocket lagged payloads as replay-gap events instead of unknown-message errors', () => {
    vi.stubGlobal('WebSocket', FakeWebSocket as unknown as typeof WebSocket)

    const onMessage = vi.fn()
    const onError = vi.fn()
    const client = new WebSocketClient({
      url: 'ws://127.0.0.1:8080/api/events/ws',
      secret: 'test-secret',
      lastEventId: '41',
      onMessage,
      onError
    })

    client.connect()

    const socket = FakeWebSocket.instances[0]
    expect(socket.url).toContain('after_position=41')
    socket.emit('message', JSON.stringify({ type: 'lagged', data: { skipped: 6 } }))

    expect(onError).not.toHaveBeenCalled()
    expect(onMessage).toHaveBeenCalledWith({
      id: '41',
      event: 'lagged',
      data: JSON.stringify({ skipped: 6 })
    })
  })
})
