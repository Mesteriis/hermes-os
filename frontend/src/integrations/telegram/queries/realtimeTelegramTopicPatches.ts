import type { TelegramTopic, TelegramTopicListResponse } from '../types/telegramTopics'
import { isRecord, stringValue } from '../../../domains/communications/queries/realtimePatchShared'
import type { TelegramEventPayload } from './realtimeTelegramPatchShared'

export function patchTelegramTopicList(
  queryKey: readonly unknown[],
  response: TelegramTopicListResponse | undefined,
  eventType: string,
  payload: TelegramEventPayload | undefined
): TelegramTopicListResponse | undefined {
  if (!response || eventType !== 'telegram.topic.updated') return response
  if (!isTopicListQueryKey(queryKey)) return response

  const topic = telegramTopicSnapshot(payload?.topic)
  if (!topic || response.telegram_chat_id !== topic.telegram_chat_id) return response

  const query = topicSearchQuery(queryKey)
  if (query && !topic.title.toLowerCase().includes(query)) {
    const nextItems = response.items.filter((item) => item.topic_id !== topic.topic_id)
    return nextItems.length === response.items.length ? response : { ...response, items: nextItems }
  }

  const limit = typeof queryKey[queryKey.length - 1] === 'number'
    ? queryKey[queryKey.length - 1] as number
    : null
  const nextItems = [topic, ...response.items.filter((item) => item.topic_id !== topic.topic_id)]
  nextItems.sort((left, right) => topicSortKey(right).localeCompare(topicSortKey(left)))

  return {
    ...response,
    items: typeof limit === 'number' ? nextItems.slice(0, limit) : nextItems,
  }
}

function isTopicListQueryKey(queryKey: readonly unknown[]): boolean {
  if (queryKey[0] !== 'integrations' || queryKey[1] !== 'telegram') return false
  return queryKey[2] === 'topics' || queryKey[2] === 'topic-search'
}

function topicSearchQuery(queryKey: readonly unknown[]): string {
  if (queryKey[2] !== 'topic-search') return ''
  return typeof queryKey[4] === 'string' ? queryKey[4].trim().toLowerCase() : ''
}

function telegramTopicSnapshot(value: unknown): TelegramTopic | null {
  if (!isRecord(value)) return null
  const topicId = stringValue(value.topic_id)
  const telegramChatId = stringValue(value.telegram_chat_id)
  const accountId = stringValue(value.account_id)
  const providerChatId = stringValue(value.provider_chat_id)
  const title = stringValue(value.title)
  const createdAt = stringValue(value.created_at)
  const updatedAt = stringValue(value.updated_at)
  const providerTopicId = typeof value.provider_topic_id === 'number' ? value.provider_topic_id : null
  if (!topicId || !telegramChatId || !accountId || !providerChatId || !title || !createdAt || !updatedAt || providerTopicId === null) {
    return null
  }

  return {
    topic_id: topicId,
    telegram_chat_id: telegramChatId,
    account_id: accountId,
    provider_topic_id: providerTopicId,
    provider_chat_id: providerChatId,
    title,
    icon_emoji: stringValue(value.icon_emoji),
    is_pinned: value.is_pinned === true,
    is_closed: value.is_closed === true,
    unread_count: typeof value.unread_count === 'number' ? value.unread_count : 0,
    last_message_at: stringValue(value.last_message_at),
    metadata: isRecord(value.metadata) ? value.metadata : {},
    created_at: createdAt,
    updated_at: updatedAt,
  }
}

function topicSortKey(topic: TelegramTopic): string {
  return `${topic.is_pinned ? '1' : '0'}:${topic.last_message_at ?? topic.updated_at}`
}
