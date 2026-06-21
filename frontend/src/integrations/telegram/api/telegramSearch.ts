import { ApiClient } from '../../../platform/api/ApiClient'

export type TelegramProviderSearchCommandResponse = {
  account_id: string
  provider_chat_id?: string | null
  query: string
  limit: number
  status: string
  error?: string | null
}

export async function searchTelegramProviderMessages(params: {
  q: string
  account_id: string
  provider_chat_id?: string
  limit?: number
}): Promise<TelegramProviderSearchCommandResponse> {
  const body = {
    q: params.q.trim(),
    account_id: params.account_id.trim(),
    provider_chat_id: params.provider_chat_id?.trim(),
    limit: params.limit,
  }
  return ApiClient.instance.post<TelegramProviderSearchCommandResponse>(
    '/api/v1/integrations/telegram/provider-search',
    body,
    'Telegram provider search trigger failed'
  )
}
