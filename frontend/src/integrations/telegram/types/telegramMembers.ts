export type TelegramChatMember = {
  sender_id: string
  sender_display_name: string | null
  message_count: number
  last_message_at: string | null
  source: 'tdlib' | 'bot_api' | 'message_heuristic'
  provider_member_id: string
  username: string | null
  role: string | null
  status: string | null
  is_admin: boolean
  is_owner: boolean
  permissions: Record<string, unknown>
  observed_at: string | null
}

export type TelegramChatMemberListResponse = {
  items: TelegramChatMember[]
  next_cursor: string | null
}

export type TelegramChatMembersSyncResponse = {
  telegram_chat_id: string
  synced_count: number
  items: TelegramChatMember[]
}
