import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  TelegramCommandListResponse,
  TelegramProviderWriteCommand,
} from '../types/telegram'

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
