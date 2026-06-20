import type { TelegramProviderWriteCommand } from '../types/telegram'

const HIDDEN_UPLOAD_STATUSES = new Set(['completed', 'cancelled'])

export function telegramUploadQueueCommands(
  commands: TelegramProviderWriteCommand[],
  providerChatId: string | null,
  limit = 3
): TelegramProviderWriteCommand[] {
  if (!providerChatId) return []

  return commands
    .filter(
      (command) =>
        command.command_kind === 'send_media' &&
        command.provider_chat_id === providerChatId &&
        !HIDDEN_UPLOAD_STATUSES.has(command.status)
    )
    .sort((left, right) => uploadQueueRecencyKey(right).localeCompare(uploadQueueRecencyKey(left)))
    .slice(0, Math.max(0, limit))
}

export function telegramUploadCommandTitle(command: TelegramProviderWriteCommand): string {
  const filename = typeof command.payload.filename === 'string' ? command.payload.filename.trim() : ''
  if (filename) return filename

  const attachmentId =
    typeof command.payload.attachment_id === 'string' ? command.payload.attachment_id.trim() : ''
  if (attachmentId) return attachmentId

  const blobId = typeof command.payload.blob_id === 'string' ? command.payload.blob_id.trim() : ''
  if (blobId) return `blob ${blobId}`

  return 'Media upload'
}

function uploadQueueRecencyKey(command: TelegramProviderWriteCommand): string {
  return command.updated_at || command.happened_at || command.created_at || ''
}
