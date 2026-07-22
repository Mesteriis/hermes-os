import { describe, expect, it } from 'vitest'
import {
  channelMessageAuthor,
  channelMessageMeta,
  channelMessageTimestamp,
} from './channelMessagePresentation'

describe('channel message presentation', () => {
  it('hides message metadata for system messages', () => {
    const message = { direction: 'system' as const, author: 'Hermes', timestamp: 'now', meta: 'system' }

    expect(channelMessageAuthor(message)).toBeUndefined()
    expect(channelMessageTimestamp(message)).toBeUndefined()
    expect(channelMessageMeta(message)).toBeUndefined()
  })

  it('preserves message metadata for regular messages', () => {
    const message = { direction: 'inbound' as const, author: 'Alice', timestamp: 'now', meta: 'email' }

    expect(channelMessageAuthor(message)).toBe('Alice')
    expect(channelMessageTimestamp(message)).toBe('now')
    expect(channelMessageMeta(message)).toBe('email')
  })
})
