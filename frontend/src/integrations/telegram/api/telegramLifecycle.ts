import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  TelegramChatActionResponse,
  TelegramCommandListResponse,
  TelegramDeleteRequest,
  TelegramEditRequest,
  TelegramForwardChainResponse,
  TelegramForwardRequest,
  TelegramLifecycleResponse,
  TelegramManualSendResponse,
  TelegramReplyChainResponse,
  TelegramReplyRequest,
  TelegramPinRequest,
  TelegramProviderWriteCommand,
  TelegramMessageTombstoneListResponse,
  TelegramMessageVersionListResponse,
  TelegramReactionListResponse,
  TelegramReactionRequest,
  TelegramReactionResponse,
  TelegramRestoreVisibilityRequest,
} from '../types/telegram'
import {
  addTelegramBusinessReaction,
  deleteTelegramBusinessMessage,
  editTelegramBusinessMessage,
  fetchTelegramBusinessForwardChain,
  fetchTelegramBusinessMessageTombstones,
  fetchTelegramBusinessMessageVersions,
  fetchTelegramBusinessReactions,
  fetchTelegramBusinessReplyChain,
  forwardTelegramBusinessMessage,
  markTelegramBusinessMessageRead,
  pinTelegramBusinessMessage,
  removeTelegramBusinessReaction,
  replyToTelegramBusinessMessage,
  restoreTelegramBusinessMessageVisibility,
} from '../../../shared/communications/telegramBusinessApi'

function newCommandId(): string {
  return `cmd_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`
}

export async function editTelegramMessage(params: {
  message_id: string
  account_id: string
  provider_chat_id: string
  provider_message_id: string
  new_text: string
}): Promise<TelegramLifecycleResponse> {
  const request: TelegramEditRequest = {
    command_id: newCommandId(),
    account_id: params.account_id,
    provider_chat_id: params.provider_chat_id,
    provider_message_id: params.provider_message_id,
    new_text: params.new_text,
  }
  return editTelegramBusinessMessage({ message_id: params.message_id, ...request })
}

export async function deleteTelegramMessage(params: {
  message_id: string
  account_id: string
  provider_chat_id: string
  provider_message_id: string
  reason_class?: string
  is_provider_delete?: boolean
}): Promise<TelegramLifecycleResponse> {
  const request: TelegramDeleteRequest = {
    command_id: newCommandId(),
    account_id: params.account_id,
    provider_chat_id: params.provider_chat_id,
    provider_message_id: params.provider_message_id,
    reason_class: (params.reason_class as TelegramDeleteRequest['reason_class']) ?? 'deleted_by_owner',
    actor_class: 'owner',
    is_provider_delete: params.is_provider_delete ?? false,
  }
  return deleteTelegramBusinessMessage({ message_id: params.message_id, ...request })
}

export async function restoreTelegramMessageVisibility(params: {
  message_id: string
  account_id: string
  provider_chat_id: string
  provider_message_id: string
  reason?: string
}): Promise<TelegramLifecycleResponse> {
  const request: TelegramRestoreVisibilityRequest = {
    command_id: newCommandId(),
    account_id: params.account_id,
    provider_chat_id: params.provider_chat_id,
    provider_message_id: params.provider_message_id,
    reason: params.reason ?? 'manual_restore',
  }
  return restoreTelegramBusinessMessageVisibility({ message_id: params.message_id, ...request })
}

export async function pinTelegramMessage(params: {
  message_id: string
  account_id: string
  provider_chat_id: string
  provider_message_id: string
  is_pinned: boolean
}): Promise<TelegramLifecycleResponse> {
  const request: TelegramPinRequest = {
    command_id: newCommandId(),
    account_id: params.account_id,
    provider_chat_id: params.provider_chat_id,
    provider_message_id: params.provider_message_id,
    is_pinned: params.is_pinned,
  }
  return pinTelegramBusinessMessage({ message_id: params.message_id, ...request })
}

export async function markTelegramMessageRead(params: {
  message_id: string
  account_id: string
  provider_chat_id: string
}): Promise<TelegramChatActionResponse> {
  return markTelegramBusinessMessageRead(params) as Promise<TelegramChatActionResponse>
}

export async function fetchTelegramMessageVersions(
  messageId: string
): Promise<TelegramMessageVersionListResponse> {
  return fetchTelegramBusinessMessageVersions(messageId)
}

export async function fetchTelegramMessageTombstones(
  messageId: string
): Promise<TelegramMessageTombstoneListResponse> {
  return fetchTelegramBusinessMessageTombstones(messageId)
}

export async function fetchTelegramCommands(
  accountId: string,
  limit = 50,
  options?: {
    providerChatId?: string | null
    providerMessageId?: string | null
    commandKinds?: string[]
  }
): Promise<TelegramCommandListResponse> {
  const params = new URLSearchParams({ account_id: accountId, limit: String(limit) })
  if (options?.providerChatId?.trim()) {
    params.set('provider_chat_id', options.providerChatId.trim())
  }
  if (options?.providerMessageId?.trim()) {
    params.set('provider_message_id', options.providerMessageId.trim())
  }
  const commandKinds = (options?.commandKinds ?? [])
    .map((value) => value.trim())
    .filter((value) => value.length > 0)
  if (commandKinds.length > 0) {
    params.set('command_kinds', commandKinds.join(','))
  }
  return ApiClient.instance.get<TelegramCommandListResponse>(
    `/api/v1/integrations/telegram/commands?${params.toString()}`,
    'Telegram commands request failed'
  )
}

export async function retryTelegramCommand(
  commandId: string
): Promise<TelegramProviderWriteCommand> {
  return ApiClient.instance.post<TelegramProviderWriteCommand>(
    `/api/v1/integrations/telegram/commands/${encodeURIComponent(commandId)}/retry`,
    {},
    'Telegram command retry failed'
  )
}

export async function fetchTelegramReplyChain(
  messageId: string
): Promise<TelegramReplyChainResponse> {
  return fetchTelegramBusinessReplyChain(messageId)
}

export async function fetchTelegramForwardChain(
  messageId: string
): Promise<TelegramForwardChainResponse> {
  return fetchTelegramBusinessForwardChain(messageId)
}

export async function addTelegramReaction(
  messageId: string,
  request: TelegramReactionRequest
): Promise<TelegramReactionResponse> {
  const payload: TelegramReactionRequest = {
    ...request,
    command_id: request.command_id ?? newCommandId()
  }
  return addTelegramBusinessReaction(messageId, payload)
}

export async function removeTelegramReaction(
  messageId: string,
  request: TelegramReactionRequest
): Promise<TelegramReactionResponse> {
  const commandId = request.command_id ?? newCommandId()
  return removeTelegramBusinessReaction(messageId, {
    ...request,
    command_id: commandId,
  })
}

export async function fetchTelegramReactions(
  messageId: string
): Promise<TelegramReactionListResponse> {
  return fetchTelegramBusinessReactions(messageId)
}

export async function replyToTelegramMessage(params: {
  message_id: string
  account_id: string
  provider_chat_id: string
  reply_to_provider_message_id: string
  text: string
}): Promise<TelegramManualSendResponse> {
  const request: TelegramReplyRequest = {
    command_id: newCommandId(),
    account_id: params.account_id,
    provider_chat_id: params.provider_chat_id,
    reply_to_provider_message_id: params.reply_to_provider_message_id,
    text: params.text,
  }
  return replyToTelegramBusinessMessage({
    message_id: params.message_id,
    text: request.text,
  })
}

export async function forwardTelegramMessage(params: {
  message_id: string
  account_id: string
  provider_chat_id: string
  from_provider_chat_id: string
  from_provider_message_id: string
}): Promise<TelegramManualSendResponse> {
  const request: TelegramForwardRequest = {
    command_id: newCommandId(),
    account_id: params.account_id,
    provider_chat_id: params.provider_chat_id,
    from_provider_chat_id: params.from_provider_chat_id,
    from_provider_message_id: params.from_provider_message_id,
  }
  return forwardTelegramBusinessMessage({
    message_id: params.message_id,
    provider_chat_id: request.provider_chat_id,
  })
}
