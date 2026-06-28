import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  ProviderCallListResponse,
  ProviderCallTranscriptResponse,
} from '../types/communications'

export async function fetchProviderCalls(
  accountId?: string,
  limit = 50,
  provider?: string
): Promise<ProviderCallListResponse> {
  const params = new URLSearchParams()
  params.set('limit', String(limit))
  if (accountId?.trim()) params.set('account_id', accountId.trim())
  if (provider?.trim()) params.set('provider', provider.trim())
  return ApiClient.instance.get<ProviderCallListResponse>(
    `/api/v1/calls?${params.toString()}`,
    'Provider calls request failed'
  )
}

export async function fetchProviderCallTranscript(
  callId: string
): Promise<ProviderCallTranscriptResponse> {
  return ApiClient.instance.get<ProviderCallTranscriptResponse>(
    `/api/v1/calls/${encodeURIComponent(callId)}/transcript`,
    'Provider call transcript request failed'
  )
}
