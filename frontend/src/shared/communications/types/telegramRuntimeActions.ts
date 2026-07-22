import type { MessengerConversationRuntimeAction } from './messengerRuntimeActions'

export type { MessengerConversationRuntimeAction } from './messengerRuntimeActions'

export type TelegramConversationRuntimeAction =
  | MessengerConversationRuntimeAction

export type TelegramConversationRuntimeActionRequest = {
  action: TelegramConversationRuntimeAction
  accountId: string
  providerChatId: string
  telegramChatId: string
  lastReadInboxProviderMessageId?: string
  caption?: string
  file?: File
  providerAttachmentId?: string
  providerFolderId?: number
  providerFolderIds?: number[]
  providerMessageId?: string
  tdlibFileId?: number
  filename?: string
  contentType?: string
  historyFromMessageId?: number
}

export type TelegramConversationRuntimeActionRunner = (
  request: TelegramConversationRuntimeActionRequest
) => Promise<void>
