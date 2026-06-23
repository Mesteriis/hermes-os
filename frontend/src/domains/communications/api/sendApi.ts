import { redirectMessageConnect, sendCommunicationConnect } from './connectCommunications'
import type { SendCommunicationRequest, SendCommunicationResponse, RedirectMessageRequest } from '../types/communications'

export async function sendEmail(request: SendCommunicationRequest): Promise<SendCommunicationResponse> {
  return sendCommunicationConnect(request)
}

export async function redirectMessage(
  messageId: string,
  request: RedirectMessageRequest
): Promise<SendCommunicationResponse> {
  return redirectMessageConnect(messageId, request)
}
