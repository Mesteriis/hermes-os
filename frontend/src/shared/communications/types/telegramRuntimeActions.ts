export type TelegramConversationRuntimeAction =
  | 'archive'
  | 'download_media'
  | 'folder_add'
  | 'folder_remove'
  | 'folder_reassign'
  | 'join'
  | 'leave'
  | 'mark_read'
  | 'mark_unread'
  | 'mute'
  | 'pin'
  | 'sync_latest'
  | 'sync_members'
  | 'sync_older'
  | 'sync_full'
  | 'unarchive'
  | 'unmute'
  | 'unpin'
  | 'upload_media'

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
