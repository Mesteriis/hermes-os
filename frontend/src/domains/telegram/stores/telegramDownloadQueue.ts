import { telegramAttachmentReadiness } from './telegramMediaSearch'
import type { TelegramAttachmentHint } from '../types/telegram'

export function telegramDownloadQueueItems(
  fileHints: TelegramAttachmentHint[],
  voiceHints: TelegramAttachmentHint[],
  limit = 4
): TelegramAttachmentHint[] {
  const seen = new Set<string>()
  const ordered = [...fileHints, ...voiceHints]

  return ordered
    .filter((attachment) => {
      if (seen.has(attachment.id)) return false
      seen.add(attachment.id)

      const readiness = telegramAttachmentReadiness(attachment)
      return readiness.label === 'Download in progress' || readiness.label === 'Download failed'
    })
    .slice(0, Math.max(0, limit))
}

export function telegramDownloadQueueTitle(attachment: TelegramAttachmentHint): string {
  const name = attachment.fileName.trim()
  if (name) return name

  const providerAttachmentId = attachment.providerAttachmentId.trim()
  if (providerAttachmentId) return providerAttachmentId

  if (attachment.tdlibFileId !== null) return `TDLib file ${attachment.tdlibFileId}`
  return 'Telegram media'
}
