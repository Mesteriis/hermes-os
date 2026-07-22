export type MessengerConversationRuntimeAction =
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

export type MessengerConversationRuntimeActionRunner = (
  action: MessengerConversationRuntimeAction,
  options?: {
    providerChatId?: string
    messageId?: string
    caption?: string
    file?: File
  },
) => Promise<void>
