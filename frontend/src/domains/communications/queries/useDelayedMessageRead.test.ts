import { effectScope, nextTick, ref } from 'vue'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { useDelayedMessageRead } from './useDelayedMessageRead'

describe('useDelayedMessageRead', () => {
  afterEach(() => vi.useRealTimers())

  it('marks the opened message as read after two seconds', async () => {
    vi.useFakeTimers()
    const messageId = ref('message-1')
    const markRead = vi.fn().mockResolvedValue(undefined)
    const scope = effectScope()

    scope.run(() => useDelayedMessageRead(messageId, markRead))
    await vi.advanceTimersByTimeAsync(1999)
    expect(markRead).not.toHaveBeenCalled()

    await vi.advanceTimersByTimeAsync(1)
    expect(markRead).toHaveBeenCalledWith('message-1')
    scope.stop()
  })

  it('does not mark a message that was replaced before the delay elapsed', async () => {
    vi.useFakeTimers()
    const messageId = ref('message-1')
    const markRead = vi.fn().mockResolvedValue(undefined)
    const scope = effectScope()

    scope.run(() => useDelayedMessageRead(messageId, markRead))
    await vi.advanceTimersByTimeAsync(1000)
    messageId.value = 'message-2'
    await nextTick()

    await vi.advanceTimersByTimeAsync(1000)
    expect(markRead).not.toHaveBeenCalled()
    await vi.advanceTimersByTimeAsync(1000)
    expect(markRead).toHaveBeenCalledTimes(1)
    expect(markRead).toHaveBeenCalledWith('message-2')
    scope.stop()
  })

  it('cancels the delayed mark when the owning view is disposed', async () => {
    vi.useFakeTimers()
    const messageId = ref('message-1')
    const markRead = vi.fn().mockResolvedValue(undefined)
    const scope = effectScope()

    scope.run(() => useDelayedMessageRead(messageId, markRead))
    scope.stop()
    await vi.advanceTimersByTimeAsync(2000)

    expect(markRead).not.toHaveBeenCalled()
  })

  it('reports a provider synchronization failure without an unhandled rejection', async () => {
    vi.useFakeTimers()
    const messageId = ref('message-1')
    const markRead = vi.fn().mockRejectedValue(new Error('provider unavailable'))
    const onError = vi.fn()
    const scope = effectScope()

    scope.run(() => useDelayedMessageRead(messageId, markRead, onError))
    await vi.advanceTimersByTimeAsync(2000)

    expect(onError).toHaveBeenCalledWith(expect.any(Error))
    scope.stop()
  })
})
