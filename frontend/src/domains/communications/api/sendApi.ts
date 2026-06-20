import { ApiClient } from '../../../platform/api/ApiClient'
import type { SendCommunicationRequest, SendCommunicationResponse, RedirectMessageRequest } from '../types/communications'

export async function sendEmail(request: SendCommunicationRequest): Promise<SendCommunicationResponse> {
  return ApiClient.instance.post<SendCommunicationResponse>(
    '/api/v1/communications/send',
    request,
    'Email send failed'
  )
}

export async function redirectMessage(
  messageId: string,
  request: RedirectMessageRequest
): Promise<SendCommunicationResponse> {
  return ApiClient.instance.post<SendCommunicationResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/redirect`,
    request,
    'Redirect message failed'
  )
}
