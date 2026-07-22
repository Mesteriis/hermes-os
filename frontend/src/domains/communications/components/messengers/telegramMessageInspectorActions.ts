import type { TelegramMessage } from '@/shared/communications/types/telegram'
import type {
  TelegramDeleteRequest,
  TelegramEditRequest,
  TelegramRestoreVisibilityRequest,
} from '@/shared/communications/types/telegramLifecycleRequests'
import type { TelegramReactionRequest } from '@/shared/communications/types/telegram'

export function telegramMessageCommandId(): string {
  return typeof crypto?.randomUUID === 'function'
    ? crypto.randomUUID()
    : `telegram-${Date.now()}-${Math.random().toString(36).slice(2)}`
}

export function buildTelegramEditRequest(
  message: TelegramMessage,
  newText: string,
  commandId: string
): TelegramEditRequest & { message_id: string } {
  return {
    message_id: message.message_id,
    command_id: commandId,
    account_id: message.account_id,
    provider_chat_id: message.provider_chat_id ?? '',
    provider_message_id: message.provider_message_id,
    new_text: newText,
  }
}

export function buildTelegramDeleteRequest(
  message: TelegramMessage,
  commandId: string
): TelegramDeleteRequest & { message_id: string } {
  return {
    message_id: message.message_id,
    command_id: commandId,
    account_id: message.account_id,
    provider_chat_id: message.provider_chat_id ?? '',
    provider_message_id: message.provider_message_id,
    reason_class: 'deleted_by_owner',
    actor_class: 'owner',
    is_provider_delete: true,
  }
}

export function buildTelegramRestoreRequest(
  message: TelegramMessage,
  commandId: string
): TelegramRestoreVisibilityRequest & { message_id: string } {
  return {
    message_id: message.message_id,
    command_id: commandId,
    account_id: message.account_id,
    provider_chat_id: message.provider_chat_id ?? '',
    provider_message_id: message.provider_message_id,
    reason: 'owner_requested_restore',
  }
}

export function buildTelegramMarkReadRequest(message: TelegramMessage): {
  message_id: string
  account_id: string
  provider_chat_id: string
} {
  return {
    message_id: message.message_id,
    account_id: message.account_id,
    provider_chat_id: message.provider_chat_id ?? '',
  }
}

export function buildTelegramPinRequest(message: TelegramMessage): { message_id: string } {
  return { message_id: message.message_id }
}

export function buildTelegramReactionRequest(
  message: TelegramMessage,
  reactionEmoji: string
): TelegramReactionRequest {
  return {
    account_id: message.account_id,
    provider_chat_id: message.provider_chat_id ?? '',
    provider_message_id: message.provider_message_id,
    reaction_emoji: reactionEmoji.trim(),
  }
}

export function buildTelegramReactionMutationRequest(
  message: TelegramMessage,
  reactionEmoji: string
): { messageId: string; request: TelegramReactionRequest } {
  return {
    messageId: message.message_id,
    request: buildTelegramReactionRequest(message, reactionEmoji),
  }
}

export function buildTelegramReplyRequest(
  message: TelegramMessage,
  text: string
): { message_id: string; text: string } {
  return { message_id: message.message_id, text }
}

export function buildTelegramForwardRequest(
  message: TelegramMessage,
  providerChatId: string
): { message_id: string; provider_chat_id: string } {
  return { message_id: message.message_id, provider_chat_id: providerChatId }
}
