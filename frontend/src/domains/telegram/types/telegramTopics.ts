// Forum topic types (P1) — split to keep telegram.ts under the 700-line SRP limit.

export type TelegramTopic = {
  topic_id: string
  telegram_chat_id: string
  account_id: string
  provider_topic_id: number
  provider_chat_id: string
  title: string
  icon_emoji: string | null
  is_pinned: boolean
  is_closed: boolean
  unread_count: number
  last_message_at: string | null
  metadata: Record<string, unknown>
  created_at: string
  updated_at: string
}

export type TelegramTopicListResponse = {
  telegram_chat_id: string
  items: TelegramTopic[]
}
