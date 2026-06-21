import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  CommunicationReadReceipt,
  NewCommunicationReadReceipt
} from '../types/readReceipts'

export async function recordReadReceipt(
  request: NewCommunicationReadReceipt
): Promise<CommunicationReadReceipt> {
  return ApiClient.instance.post<CommunicationReadReceipt>(
    '/api/v1/communications/read-receipts',
    request,
    'Read receipt recording failed'
  )
}
