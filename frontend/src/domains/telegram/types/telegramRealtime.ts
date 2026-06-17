// ADR-0091 Telegram realtime event surface.
// Keep this separate from telegram.ts so the core domain type file stays below
// the repository line-count limit while realtime coverage grows.

export type TelegramRealtimeEventType =
  | 'telegram.sync.started'
  | 'telegram.sync.progress'
  | 'telegram.sync.completed'
  | 'telegram.sync.failed'
  | 'telegram.message.created'
  | 'telegram.message.updated'
  | 'telegram.message.edited'
  | 'telegram.message.deleted'
  | 'telegram.message.tombstoned'
  | 'telegram.message.visibility_restored'
  | 'telegram.reaction.changed'
  | 'telegram.chat.updated'
  | 'telegram.chat.pinned'
  | 'telegram.chat.archived'
  | 'telegram.chat.muted'
  | 'telegram.typing.changed'
  | 'telegram.topic.updated'
  | 'telegram.participant.updated'
  | 'telegram.media.download.started'
  | 'telegram.media.download.progress'
  | 'telegram.media.download.failed'
  | 'telegram.media.downloaded'
  | 'telegram.media.upload.started'
  | 'telegram.media.upload.progress'
  | 'telegram.media.upload.failed'
  | 'telegram.media.upload.completed'
  | 'telegram.command.status_changed'
  | 'telegram.command.reconciled'

export type TelegramRealtimeEvent = {
  event_type: TelegramRealtimeEventType
  event_id: string
  occurred_at: string
  subject: { id: string; kind: string }
  payload: Record<string, unknown>
}

export type TelegramRealtimeMessage =
  | { type: 'event'; data: TelegramRealtimeEvent }
  | { type: 'lagged'; data: { skipped: number } }
