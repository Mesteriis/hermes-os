import type { TelegramChat } from './types/telegram'

type TelegramChatMetadata = TelegramChat['metadata'] & {
  tdlib_chat_positions?: unknown
  provider_folder_ids?: unknown
  provider_folder_id?: unknown
}

export function telegramChatFolderIds(chat: TelegramChat): number[] {
  const metadata = chat.metadata as TelegramChatMetadata
  const positions = metadata.tdlib_chat_positions
  const positionFolderIds =
    positions && typeof positions === 'object' && Array.isArray((positions as { folder_ids?: unknown[] }).folder_ids)
      ? (positions as { folder_ids: unknown[] }).folder_ids.filter(
          (value): value is number => typeof value === 'number'
        )
      : []
  if (positionFolderIds.length > 0) {
    return positionFolderIds
  }

  const metadataFolderIds = Array.isArray(metadata.provider_folder_ids)
    ? metadata.provider_folder_ids.filter((value): value is number => typeof value === 'number')
    : []
  if (metadataFolderIds.length > 0) {
    return metadataFolderIds
  }

  return typeof metadata.provider_folder_id === 'number' ? [metadata.provider_folder_id] : []
}
