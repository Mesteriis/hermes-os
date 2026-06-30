import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  ZulipAccountSetupRequest,
  ZulipAccountSetupResponse,
  ZulipCommandEnqueueResponse,
  ZulipDirectUploadCommandRequest,
  ZulipStreamUploadCommandRequest,
  ZulipUploadCommandRequest,
} from '../types/zulip'

export async function setupZulipBotAccount(
  request: ZulipAccountSetupRequest
): Promise<ZulipAccountSetupResponse> {
  return ApiClient.instance.post<ZulipAccountSetupResponse>(
    '/api/v1/integrations/zulip/accounts',
    request,
    'Zulip bot account setup failed'
  )
}

export async function enqueueZulipStreamUploadCommand(
  accountId: string,
  request: ZulipStreamUploadCommandRequest
): Promise<ZulipCommandEnqueueResponse> {
  return ApiClient.instance.post<ZulipCommandEnqueueResponse>(
    `/api/v1/integrations/zulip/accounts/${encodeURIComponent(accountId.trim())}/commands/stream-upload`,
    request,
    'Zulip stream upload command enqueue failed'
  )
}

export async function enqueueZulipDirectUploadCommand(
  accountId: string,
  request: ZulipDirectUploadCommandRequest
): Promise<ZulipCommandEnqueueResponse> {
  return ApiClient.instance.post<ZulipCommandEnqueueResponse>(
    `/api/v1/integrations/zulip/accounts/${encodeURIComponent(accountId.trim())}/commands/direct-upload`,
    request,
    'Zulip direct upload command enqueue failed'
  )
}

export async function enqueueZulipUploadCommand(
  accountId: string,
  request: ZulipUploadCommandRequest
): Promise<ZulipCommandEnqueueResponse> {
  return ApiClient.instance.post<ZulipCommandEnqueueResponse>(
    `/api/v1/integrations/zulip/accounts/${encodeURIComponent(accountId.trim())}/commands/upload`,
    request,
    'Zulip upload command enqueue failed'
  )
}
