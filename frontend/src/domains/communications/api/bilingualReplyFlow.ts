import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  BilingualReplyFlowRequest,
  BilingualReplyFlowResponse
} from '../types/bilingualReplyFlow'

export async function prepareBilingualReplyFlow(
  messageId: string,
  request: BilingualReplyFlowRequest
): Promise<BilingualReplyFlowResponse> {
  return ApiClient.instance.post<BilingualReplyFlowResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/bilingual-reply-flow`,
    request,
    'Bilingual reply flow preparation failed'
  )
}
