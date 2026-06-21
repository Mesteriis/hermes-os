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
  return ApiClient.instance.post<TelegramLifecycleResponse>(
    `/api/v1/communications/provider-messages/${encodeURIComponent(params.message_id)}/edit`,
    request,
    'Telegram message edit failed'
  )
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
  return ApiClient.instance.post<TelegramLifecycleResponse>(
    `/api/v1/communications/provider-messages/${encodeURIComponent(params.message_id)}/delete`,
    request,
    'Telegram message delete failed'
  )
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
  return ApiClient.instance.post<TelegramLifecycleResponse>(
    `/api/v1/communications/provider-messages/${encodeURIComponent(params.message_id)}/restore-visibility`,
    request,
    'Telegram message restore failed'
  )
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
  return ApiClient.instance.post<TelegramLifecycleResponse>(
    `/api/v1/communications/provider-messages/${encodeURIComponent(params.message_id)}/pin`,
    request,
    'Telegram message pin failed'
  )
}

export async function markTelegramMessageRead(params: {
  message_id: string
  account_id: string
  provider_chat_id: string
}): Promise<TelegramChatActionResponse> {
  return ApiClient.instance.post<TelegramChatActionResponse>(
    `/api/v1/communications/provider-messages/${encodeURIComponent(params.message_id)}/mark-read`,
    {
      account_id: params.account_id,
      provider_chat_id: params.provider_chat_id,
    },
    'Telegram message mark read failed'
  )
}

export async function fetchTelegramMessageVersions(
  messageId: string
): Promise<TelegramMessageVersionListResponse> {
  return ApiClient.instance.get<TelegramMessageVersionListResponse>(
    `/api/v1/communications/provider-messages/${encodeURIComponent(messageId)}/versions`,
    'Telegram message versions request failed'
  )
}

export async function fetchTelegramMessageTombstones(
  messageId: string
): Promise<TelegramMessageTombstoneListResponse> {
  return ApiClient.instance.get<TelegramMessageTombstoneListResponse>(
    `/api/v1/communications/provider-messages/${encodeURIComponent(messageId)}/tombstones`,
    'Telegram message tombstones request failed'
  )
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
  return ApiClient.instance.get<TelegramReplyChainResponse>(
    `/api/v1/communications/provider-messages/${encodeURIComponent(messageId)}/reply-chain`,
    'Telegram reply chain request failed'
  )
}

export async function fetchTelegramForwardChain(
  messageId: string
): Promise<TelegramForwardChainResponse> {
  return ApiClient.instance.get<TelegramForwardChainResponse>(
    `/api/v1/communications/provider-messages/${encodeURIComponent(messageId)}/forward-chain`,
    'Telegram forward chain request failed'
  )
}

export async function addTelegramReaction(
  messageId: string,
  request: TelegramReactionRequest
): Promise<TelegramReactionResponse> {
  const payload: TelegramReactionRequest = {
    ...request,
    command_id: request.command_id ?? newCommandId()
  }
  return ApiClient.instance.post<TelegramReactionResponse>(
    `/api/v1/communications/provider-messages/${encodeURIComponent(messageId)}/reactions`,
    payload,
    'Telegram reaction add failed'
  )
}

export async function removeTelegramReaction(
  messageId: string,
  request: TelegramReactionRequest
): Promise<TelegramReactionResponse> {
  const commandId = request.command_id ?? newCommandId()
  const params = new URLSearchParams({
    account_id: request.account_id,
    provider_chat_id: request.provider_chat_id,
    provider_message_id: request.provider_message_id,
    reaction_emoji: request.reaction_emoji,
    sender_id: request.sender_id,
    command_id: commandId,
  })
  if (request.sender_display_name) {
    params.set('sender_display_name', request.sender_display_name)
  }
  return ApiClient.instance.delete<TelegramReactionResponse>(
    `/api/v1/communications/provider-messages/${encodeURIComponent(messageId)}/reactions?${params.toString()}`,
    'Telegram reaction remove failed'
  )
}

export async function fetchTelegramReactions(
  messageId: string
): Promise<TelegramReactionListResponse> {
  return ApiClient.instance.get<TelegramReactionListResponse>(
    `/api/v1/communications/provider-messages/${encodeURIComponent(messageId)}/reactions`,
    'Telegram reactions request failed'
  )
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
  return ApiClient.instance.post<TelegramManualSendResponse>(
    `/api/v1/communications/provider-messages/${encodeURIComponent(params.message_id)}/reply`,
    request,
    'Telegram message reply failed'
  )
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
  return ApiClient.instance.post<TelegramManualSendResponse>(
    `/api/v1/communications/provider-messages/${encodeURIComponent(params.message_id)}/forward`,
    request,
    'Telegram message forward failed'
  )
}
