import type { TelegramMessage } from '../types/telegram'

export type TelegramMentionProjection = {
  count: number
  mentions: string[]
  detected_by: string
}

function metadataNumber(value: unknown): number {
  return typeof value === 'number' && Number.isFinite(value) ? Math.max(0, Math.trunc(value)) : 0
}

function metadataStringArray(value: unknown): string[] {
  return Array.isArray(value)
    ? value.filter((item): item is string => typeof item === 'string' && item.trim().length > 0)
    : []
}

export function telegramMessageMentionProjection(message: TelegramMessage): TelegramMentionProjection {
  const mentions = metadataStringArray(message.metadata.mentions)
  const count = Math.max(metadataNumber(message.metadata.mention_count), mentions.length)
  const detectedBy =
    typeof message.metadata.mentions_detected_by === 'string'
      ? message.metadata.mentions_detected_by
      : 'unknown'

  return {
    count,
    mentions,
    detected_by: detectedBy,
  }
}

export function telegramMessageMentionLabel(message: TelegramMessage): string {
  const projection = telegramMessageMentionProjection(message)
  if (projection.count === 0) return ''
  const mentionList = projection.mentions.slice(0, 3).join(', ')
  const overflow = projection.mentions.length > 3 ? ` +${projection.mentions.length - 3}` : ''
  return mentionList ? `${mentionList}${overflow}` : `${projection.count} mentions`
}
