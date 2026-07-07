import { ApiClient } from '../../../platform/api/ApiClient'
import type { EmailAccountListResponse } from '../types/communications'

export async function fetchEmailAccounts(): Promise<EmailAccountListResponse> {
  return ApiClient.instance.get<EmailAccountListResponse>(
    '/api/v1/communications/email/accounts',
    'Email accounts request failed'
  )
}
