import type { TelegramChat } from '@/shared/communications/types/telegram'
import type {
  TelegramConversationRuntimeAction,
  TelegramConversationRuntimeActionRunner,
} from '@/shared/communications/types/telegramRuntimeActions'

type RuntimeRequest = Parameters<TelegramConversationRuntimeActionRunner>[0]
type RuntimeExtras = Partial<Omit<RuntimeRequest, 'action' | 'accountId' | 'providerChatId' | 'telegramChatId'>>

type TelegramMediaDownloadSource = {
  provider_message_id: string
  tdlib_file_id: number
  provider_attachment_id?: string | null
  file_name?: string | null
  mime_type?: string | null
}

export function telegramConversationCommandId(): string {
  return crypto.randomUUID()
}

export function buildTelegramRuntimeActionRequest(
  chat: TelegramChat,
  action: TelegramConversationRuntimeAction,
  extras: RuntimeExtras = {}
): RuntimeRequest {
  return {
    action,
    accountId: chat.account_id,
    providerChatId: chat.provider_chat_id,
    telegramChatId: chat.telegram_chat_id,
    ...extras,
  }
}

export function buildTelegramTopicCreateRequest(
  chat: TelegramChat,
  title: string,
  commandId: string
): {
  conversationId: string
  request: {
    command_id: string
    account_id: string
    provider_chat_id: string
    title: string
  }
} {
  return {
    conversationId: chat.telegram_chat_id,
    request: {
      command_id: commandId,
      account_id: chat.account_id,
      provider_chat_id: chat.provider_chat_id,
      title,
    },
  }
}

export function buildTelegramTopicCloseRequest(
  chat: TelegramChat,
  topicId: string,
  isClosed: boolean,
  commandId: string
): {
  topicId: string
  request: {
    command_id: string
    account_id: string
    provider_chat_id: string
    is_closed: boolean
  }
} {
  return {
    topicId,
    request: {
      command_id: commandId,
      account_id: chat.account_id,
      provider_chat_id: chat.provider_chat_id,
      is_closed: !isClosed,
    },
  }
}

export function selectedTelegramProviderFolderId(folderId: number | null): number | undefined {
  return folderId ?? undefined
}

export function buildTelegramMediaDownloadExtras(
  media: TelegramMediaDownloadSource
): RuntimeExtras {
  return {
    providerMessageId: media.provider_message_id,
    tdlibFileId: media.tdlib_file_id,
    providerAttachmentId: media.provider_attachment_id ?? undefined,
    filename: media.file_name ?? undefined,
    contentType: media.mime_type ?? undefined,
  }
}

export function buildTelegramFolderActionExtras(
  folderId: number | undefined
): RuntimeExtras {
  return {
    providerFolderId: folderId,
  }
}

export function buildTelegramFolderReassignExtras(
  folderId: number | undefined
): RuntimeExtras {
  return {
    providerFolderIds: folderId == null ? [] : [folderId],
  }
}

export function buildTelegramHistoryPolicyRequest(chat: TelegramChat, enabled: boolean): {
  telegramChatId: string
  accountId: string
  providerChatId: string
  enabled: boolean
} {
  return {
    telegramChatId: chat.telegram_chat_id,
    accountId: chat.account_id,
    providerChatId: chat.provider_chat_id,
    enabled,
  }
}

export function buildTelegramReadReceiptPolicyRequest(chat: TelegramChat, enabled: boolean): {
  telegramChatId: string
  accountId: string
  providerChatId: string
  enabled: boolean
} {
  return buildTelegramHistoryPolicyRequest(chat, enabled)
}

export function buildTelegramUnreadCounterPolicyRequest(chat: TelegramChat, hidden: boolean): {
  telegramChatId: string
  accountId: string
  providerChatId: string
  hideUnreadCounter: boolean
} {
  return {
    telegramChatId: chat.telegram_chat_id,
    accountId: chat.account_id,
    providerChatId: chat.provider_chat_id,
    hideUnreadCounter: hidden,
  }
}
