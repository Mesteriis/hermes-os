import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  MailReadReceipt,
  NewMailReadReceipt
} from '../types/readReceipts'

export async function recordReadReceipt(
  request: NewMailReadReceipt
): Promise<MailReadReceipt> {
  return ApiClient.instance.post<MailReadReceipt>(
    '/api/v1/communications/read-receipts',
    request,
    'Read receipt recording failed'
  )
}
