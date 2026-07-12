import { ApiClient } from '../../platform/api/ApiClient'

export type MailProviderCommandDiagnostic = {
  command_id: string
  account_id: string
  command_kind: string
  message_id: string | null
  status: string
  retry_count: number
  max_retries: number
  reconciliation_status: string
  next_attempt_at: string | null
  last_attempt_at: string | null
  dead_lettered_at: string | null
  last_error: string | null
  created_at: string
  updated_at: string
}

export type MailProviderCommandDiagnostics = {
  items: MailProviderCommandDiagnostic[]
  counts: Array<{ status: string; count: number }>
}

export type MailProviderCommandRetryResponse = Pick<
  MailProviderCommandDiagnostic,
  'command_id' | 'status' | 'retry_count' | 'max_retries' | 'reconciliation_status' | 'next_attempt_at'
>

export async function fetchMailProviderCommandDiagnostics(
  accountId: string,
  status?: string
): Promise<MailProviderCommandDiagnostics> {
  const params = new URLSearchParams({ account_id: accountId, limit: '50' })
  if (status?.trim()) params.set('status', status.trim())
  return ApiClient.instance.get<MailProviderCommandDiagnostics>(
    `/api/v1/communications/provider-commands/diagnostics?${params.toString()}`,
    'Mail provider command diagnostics request failed'
  )
}

export async function retryMailProviderCommand(
  commandId: string
): Promise<MailProviderCommandRetryResponse> {
  return ApiClient.instance.post<MailProviderCommandRetryResponse>(
    `/api/v1/communications/provider-commands/${encodeURIComponent(commandId)}/retry`,
    {},
    'Mail provider command retry request failed'
  )
}
