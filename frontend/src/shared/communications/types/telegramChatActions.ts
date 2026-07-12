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

export type TelegramChatHistoryPolicyRequest = {
  account_id: string
  provider_chat_id: string
  full_history_sync_enabled: boolean
}

export type TelegramChatReadReceiptPolicyRequest = {
  account_id: string
  provider_chat_id: string
  read_receipt_reports_enabled: boolean
}

export type TelegramChatUnreadCounterPolicyRequest = {
  account_id: string
  provider_chat_id: string
  hide_unread_counter: boolean
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
