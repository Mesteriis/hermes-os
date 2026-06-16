// Lifecycle write-command request types (ADR-0091).
// Split from telegram.ts to stay under the 700-line SRP limit.

type TombstoneReasonClass =
  | 'deleted_by_owner'
  | 'deleted_by_counterparty'
  | 'deleted_by_provider'
  | 'moderation_removed'
  | 'account_removed'
  | 'retention_policy'
  | 'unknown'

type TombstoneActorClass = 'owner' | 'provider' | 'automation' | 'system' | 'unknown'

export type TelegramEditRequest = {
  command_id: string
  account_id: string
  provider_chat_id: string
  provider_message_id: string
  new_text: string
}

export type TelegramReplyRequest = {
  command_id: string
  account_id: string
  provider_chat_id: string
  reply_to_provider_message_id: string
  text: string
}

export type TelegramDeleteRequest = {
  command_id: string
  account_id: string
  provider_chat_id: string
  provider_message_id: string
  reason_class: TombstoneReasonClass
  actor_class: TombstoneActorClass
  is_provider_delete: boolean
}

export type TelegramRestoreVisibilityRequest = {
  command_id: string
  account_id: string
  provider_chat_id: string
  provider_message_id: string
  reason: string
}

export type TelegramPinRequest = {
  command_id: string
  account_id: string
  provider_chat_id: string
  provider_message_id: string
  is_pinned: boolean
}
