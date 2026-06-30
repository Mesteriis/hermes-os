import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api/ApiClient'
import {
  enqueueZulipDirectUploadCommand,
  enqueueZulipStreamUploadCommand,
  enqueueZulipUploadCommand,
  setupZulipBotAccount,
} from './zulip'

describe('zulip integration API', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    ApiClient.resetForTests()
  })

  it('posts bot account setup and upload command routes without transforming payloads', async () => {
    const ok = (body: unknown) =>
      new Response(JSON.stringify(body), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(ok({
        account_id: 'zulip-live-1',
        provider_kind: 'zulip_bot',
        display_name: 'Zulip Live',
        external_account_id: 'bot@example.test',
        base_url: 'https://zulip.example.test',
        credential_binding: {
          secret_purpose: 'zulip_api_key',
          secret_ref: 'secret:provider-account:zulip-live-1:zulip_api_key',
          secret_kind: 'api_token',
          store_kind: 'host_vault',
        },
      }))
      .mockResolvedValueOnce(ok({
        command_id: 'cmd-stream',
        account_id: 'zulip-live-1',
        channel_kind: 'zulip',
        command_kind: 'send_stream_message_with_upload',
        idempotency_key: 'stream-key',
        status: 'pending',
        reconciliation_status: 'not_observed',
        provider_conversation_id: 'Hermes Lab/Tasks',
        payload: {},
      }))
      .mockResolvedValueOnce(ok({
        command_id: 'cmd-direct',
        account_id: 'zulip-live-1',
        channel_kind: 'zulip',
        command_kind: 'send_direct_message_with_upload',
        idempotency_key: 'direct-key',
        status: 'pending',
        reconciliation_status: 'not_observed',
        provider_conversation_id: null,
        payload: {},
      }))
      .mockResolvedValueOnce(ok({
        command_id: 'cmd-upload',
        account_id: 'zulip-live-1',
        channel_kind: 'zulip',
        command_kind: 'upload_file',
        idempotency_key: 'upload-key',
        status: 'pending',
        reconciliation_status: 'not_observed',
        provider_conversation_id: null,
        payload: {},
      }))
    vi.stubGlobal('fetch', fetchMock)

    await setupZulipBotAccount({
      account_id: 'zulip-live-1',
      display_name: 'Zulip Live',
      external_account_id: 'bot@example.test',
      base_url: 'https://zulip.example.test',
      api_key: 'provider-token-from-user',
    })
    await enqueueZulipStreamUploadCommand('zulip-live-1', {
      stream: 'Hermes Lab',
      topic: 'Tasks',
      content: 'Traceable message',
      attachment_id: 'attachment-1',
      blob_id: 'blob:v1:attachment-1',
      filename: 'evidence.txt',
      idempotency_key: 'stream-key',
    })
    await enqueueZulipDirectUploadCommand('zulip-live-1', {
      recipients: ['alice@example.test', '123'],
      content: 'Direct trace',
      attachment_id: 'attachment-2',
      idempotency_key: 'direct-key',
    })
    await enqueueZulipUploadCommand('zulip-live-1', {
      blob_id: 'blob:v1:upload-only',
      filename: 'upload-only.txt',
      idempotency_key: 'upload-key',
    })

    expect(fetchMock).toHaveBeenCalledTimes(4)
    expect(fetchMock.mock.calls[0][0]).toBe('http://127.0.0.1:8080/api/v1/integrations/zulip/accounts')
    expect(fetchMock.mock.calls[1][0]).toContain('/api/v1/integrations/zulip/accounts/zulip-live-1/commands/stream-upload')
    expect(fetchMock.mock.calls[2][0]).toContain('/api/v1/integrations/zulip/accounts/zulip-live-1/commands/direct-upload')
    expect(fetchMock.mock.calls[3][0]).toContain('/api/v1/integrations/zulip/accounts/zulip-live-1/commands/upload')
    expect(fetchMock.mock.calls.map((call) => call[1].method)).toEqual(['POST', 'POST', 'POST', 'POST'])
    expect(JSON.parse(fetchMock.mock.calls[0][1].body as string)).toEqual({
      account_id: 'zulip-live-1',
      display_name: 'Zulip Live',
      external_account_id: 'bot@example.test',
      base_url: 'https://zulip.example.test',
      api_key: 'provider-token-from-user',
    })
    expect(JSON.parse(fetchMock.mock.calls[1][1].body as string)).toEqual({
      stream: 'Hermes Lab',
      topic: 'Tasks',
      content: 'Traceable message',
      attachment_id: 'attachment-1',
      blob_id: 'blob:v1:attachment-1',
      filename: 'evidence.txt',
      idempotency_key: 'stream-key',
    })
    expect(JSON.parse(fetchMock.mock.calls[2][1].body as string).recipients).toEqual([
      'alice@example.test',
      '123',
    ])
    expect(JSON.parse(fetchMock.mock.calls[3][1].body as string)).toEqual({
      blob_id: 'blob:v1:upload-only',
      filename: 'upload-only.txt',
      idempotency_key: 'upload-key',
    })
  })
})
