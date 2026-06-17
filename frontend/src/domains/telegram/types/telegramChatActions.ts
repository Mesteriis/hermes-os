export type TelegramChatActionRequest = {
  account_id: string
  provider_chat_id: string
  last_read_inbox_provider_message_id?: string
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

export type TelegramChatFolderReassignRequest = {
  account_id: string
  provider_chat_id: string
  target_provider_folder_ids: number[]
}

export type TelegramChatFolderReassignResponse = {
  telegram_chat_id: string
  provider_chat_id: string
  action: string
  status: string
  command_ids: string[]
  added_provider_folder_ids: number[]
  removed_provider_folder_ids: number[]
}
