import type { TelegramTopic } from '../types/telegramTopics'

export function telegramTopicStateLabel(topic: TelegramTopic): string {
  const states = []
  if (topic.is_pinned) states.push('Pinned')
  if (topic.is_closed) states.push('Closed')
  if (topic.unread_count > 0) states.push(`${topic.unread_count} unread`)
  return states.length > 0 ? states.join(' · ') : 'Open'
}

export function telegramTopicActivityLabel(topic: TelegramTopic): string {
  if (!topic.last_message_at) return 'No projected activity'
  const date = new Date(topic.last_message_at)
  if (Number.isNaN(date.getTime())) return 'No projected activity'
  return new Intl.DateTimeFormat('en', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  }).format(date)
}

export function telegramTopicProviderLabel(topic: TelegramTopic): string {
  return `Topic ${topic.provider_topic_id} · ${telegramTopicActivityLabel(topic)}`
}
