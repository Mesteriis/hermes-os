import { ApiClient } from '../../../platform/api/ApiClient'
import type { SendEmailRequest, SendEmailResponse, RedirectMessageRequest } from '../types/communications'

export async function sendEmail(request: SendEmailRequest): Promise<SendEmailResponse> {
  return ApiClient.instance.post<SendEmailResponse>(
    '/api/v1/communications/send',
    request,
    'Email send failed'
  )
}

export async function redirectMessage(
  messageId: string,
  request: RedirectMessageRequest
): Promise<SendEmailResponse> {
  return ApiClient.instance.post<SendEmailResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/redirect`,
    request,
    'Redirect message failed'
  )
}
