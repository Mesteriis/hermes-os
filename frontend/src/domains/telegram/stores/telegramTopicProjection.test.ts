import { describe, expect, it } from 'vitest'
import {
  telegramTopicActivityLabel,
  telegramTopicProviderLabel,
  telegramTopicStateLabel,
} from './telegramTopicProjection'
import type { TelegramTopic } from '../types/telegramTopics'

function topic(overrides: Partial<TelegramTopic>): TelegramTopic {
  return {
    topic_id: 'topic-1',
    telegram_chat_id: 'chat-1',
    account_id: 'acct-1',
    provider_topic_id: 42,
    provider_chat_id: 'provider-chat-1',
    title: 'Architecture',
    icon_emoji: null,
    is_pinned: false,
    is_closed: false,
    unread_count: 0,
    last_message_at: null,
    metadata: {},
    created_at: '2026-06-17T10:00:00Z',
    updated_at: '2026-06-17T10:00:00Z',
    ...overrides,
  }
}

describe('telegram topic projection', () => {
  it('summarizes pinned, closed and unread state from projected topic rows', () => {
    expect(
      telegramTopicStateLabel(
        topic({
          is_pinned: true,
          is_closed: true,
          unread_count: 7,
        })
      )
    ).toBe('Pinned · Closed · 7 unread')
  })

  it('keeps open topics explicit when no state flags are projected', () => {
    expect(telegramTopicStateLabel(topic({}))).toBe('Open')
  })

  it('surfaces provider topic id with projected activity time', () => {
    const projectedTopic = topic({ last_message_at: '2026-06-17T12:34:00Z' })

    expect(telegramTopicActivityLabel(projectedTopic)).toMatch(/Jun 17/)
    expect(telegramTopicProviderLabel(projectedTopic)).toMatch(/^Topic 42 · /)
  })

  it('handles topics with no projected last activity', () => {
    expect(telegramTopicProviderLabel(topic({ last_message_at: null }))).toBe(
      'Topic 42 · No projected activity'
    )
  })
})
