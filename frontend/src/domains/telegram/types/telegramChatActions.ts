export type TelegramChatActionRequest = {
  account_id: string
  provider_chat_id: string
}

export type TelegramChatActionResponse = {
  telegram_chat_id: string
  action: string
  status: string
  metadata: Record<string, unknown>
}

export type TelegramChatLifecycleCommandResponse = {
  telegram_chat_id: string | null
  provider_chat_id: string
  action: string
  status: string
  command_id: string
}
