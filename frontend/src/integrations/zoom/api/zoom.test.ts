import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api/ApiClient'
import {
  authorizeZoomServerToServer,
  bridgeZoomMeeting,
  bridgeZoomRecording,
  bridgeZoomTranscript,
  cleanupZoomRetention,
  completeZoomOAuth,
  fetchZoomCallTranscript,
  fetchZoomAccounts,
  fetchZoomAuditEvents,
  fetchZoomCapabilities,
  fetchZoomRecordingImports,
  fetchZoomProviderCalls,
  fetchZoomRuntimeStatus,
  fetchZoomWebhookSubscriptionStatus,
  importZoomTranscriptFile,
  maintainZoomTokens,
  reconcileZoomWebhookSubscription,
  refreshZoomToken,
  removeZoomRecordingImport,
  removeZoomRuntime,
  removeZoomWebhookSubscription,
  setupZoomFixtureAccount,
  setupZoomLiveAccount,
  syncZoomRecordings,
  startZoomOAuth,
  startZoomRuntime,
  stopZoomRuntime,
} from './zoom'

describe('zoom integration API', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    ApiClient.resetForTests()
  })

  it('builds expected account and runtime routes', async () => {
    const ok = (body: unknown) =>
      new Response(JSON.stringify(body), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(ok({ version: '1.0', runtime_mode: 'fixture_plus_blocked_live', capabilities: [], planned_features: [], unsupported_features: [] }))
      .mockResolvedValueOnce(ok({ items: [] }))
      .mockResolvedValueOnce(ok({ account: { account_id: 'zoom-fixture-1' } }))
      .mockResolvedValueOnce(ok({ account: { account_id: 'zoom-live-1' } }))
      .mockResolvedValueOnce(ok({
        setup_id: 'zoom-oauth-setup-1',
        authorization_url: 'https://zoom.example.test/oauth/authorize',
        state: 'zoom-oauth-state',
        redirect_uri: 'http://127.0.0.1:8080/zoom/callback',
      }))
      .mockResolvedValueOnce(ok({
        account_id: 'zoom-live-1',
        provider_kind: 'zoom_user',
        auth_shape: 'oauth_user',
        lifecycle_state: 'authorized',
        runtime_kind: 'zoom_live_authorized_runtime',
        token_secret_ref: 'secret:zoom-token',
        client_secret_ref: 'secret:zoom-client-secret',
        secret_kind: 'oauth_token',
        store_kind: 'host_vault',
        authorized_at: '2026-06-27T10:00:00Z',
      }))
      .mockResolvedValueOnce(ok({
        account_id: 'zoom-s2s-1',
        provider_kind: 'zoom_server_to_server',
        auth_shape: 'server_to_server',
        lifecycle_state: 'authorized',
        runtime_kind: 'zoom_live_authorized_runtime',
        token_secret_ref: 'secret:zoom-s2s-token',
        client_secret_ref: 'secret:zoom-s2s-client-secret',
        secret_kind: 'oauth_token',
        store_kind: 'host_vault',
        authorized_at: '2026-06-27T10:00:00Z',
      }))
      .mockResolvedValueOnce(ok({
        account_id: 'zoom-live-1',
        provider_kind: 'zoom_user',
        auth_shape: 'oauth_user',
        token_secret_ref: 'secret:zoom-token',
        refreshed: true,
        refresh_strategy: 'oauth_refresh_token',
        status: 'refreshed',
        expires_at: '2026-06-27T11:00:00Z',
        checked_at: '2026-06-27T10:00:00Z',
        secret_kind: 'oauth_token',
        store_kind: 'host_vault',
      }))
      .mockResolvedValueOnce(ok({
        checked_count: 1,
        refreshed_count: 1,
        skipped_count: 0,
        failed_count: 0,
        refresh_expiring_within_seconds: 300,
        checked_at: '2026-06-27T10:00:00Z',
        items: [
          {
            account_id: 'zoom-live-1',
            provider_kind: 'zoom_user',
            auth_shape: 'oauth_user',
            status: 'refreshed',
            refreshed: true,
            expires_at: '2026-06-27T11:00:00Z',
          },
        ],
      }))
      .mockResolvedValueOnce(ok({
        account_id: 'zoom-live-1',
        user_id: 'me',
        from: '2026-06-01',
        to: '2026-06-30',
        meetings_seen: 1,
        meetings_recorded: 1,
        recordings_recorded: 2,
        media_downloads_recorded: 1,
        transcripts_recorded: 1,
        failures: [],
      }))
      .mockResolvedValueOnce(ok({
        account_id: 'zoom-live-1',
        provider_kind: 'zoom_user',
        auth_shape: 'oauth_user',
        checked_at: '2026-06-27T10:00:00Z',
        managed_subscription_id: 'zoom-subscription-1',
        subscriptions: [],
      }))
      .mockResolvedValueOnce(ok({
        account_id: 'zoom-live-1',
        provider_kind: 'zoom_user',
        auth_shape: 'oauth_user',
        status: 'created',
        checked_at: '2026-06-27T10:00:00Z',
        subscription: {
          subscription_id: 'zoom-subscription-1',
          subscription_name: 'Hermes Zoom Runtime',
          endpoint_url: 'https://hermes.example.test/api/v1/integrations/zoom/runtime-bridge/webhooks',
          event_types: ['meeting.started'],
        },
      }))
      .mockResolvedValueOnce(ok({
        account_id: 'zoom-live-1',
        provider_kind: 'zoom_user',
        auth_shape: 'oauth_user',
        removed: true,
        checked_at: '2026-06-27T10:00:00Z',
        subscription_id: 'zoom-subscription-1',
      }))
      .mockResolvedValueOnce(ok({ account_id: 'zoom-live-1', status: 'blocked' }))
      .mockResolvedValueOnce(ok({ account_id: 'zoom-live-1', status: 'blocked' }))
      .mockResolvedValueOnce(ok({ account_id: 'zoom-live-1', status: 'stopped' }))
      .mockResolvedValueOnce(ok({ account_id: 'zoom-live-1', removed: true }))
      .mockResolvedValueOnce(ok({ account_id: 'zoom-live-1', items: [] }))
      .mockResolvedValueOnce(ok({ account_id: 'zoom-live-1', items: [] }))
      .mockResolvedValueOnce(ok({
        account_id: 'zoom-live-1',
        attachment_id: 'attachment-1',
        blob_id: 'blob-1',
        removed: true,
        blob_metadata_removed: true,
        blob_file_removed: true,
        removed_at: '2026-06-27T10:00:00Z',
      }))
      .mockResolvedValueOnce(ok({
        account_id: 'zoom-live-1',
        checked_at: '2026-06-27T10:00:00Z',
        recordings_removed: 1,
        transcripts_removed: 1,
        items: [],
      }))
      .mockResolvedValueOnce(ok({ account_id: 'zoom-live-1', items: [] }))
    vi.stubGlobal('fetch', fetchMock)

    await fetchZoomCapabilities()
    await fetchZoomAccounts(true)
    await setupZoomFixtureAccount({
      account_id: 'zoom-fixture-1',
      display_name: 'Zoom Fixture',
      external_account_id: 'zoom-fixture-external',
    })
    await setupZoomLiveAccount({
      account_id: 'zoom-live-1',
      display_name: 'Zoom Live',
      external_account_id: 'zoom-live-external',
      auth_shape: 'oauth_user',
      client_id: 'zoom-client-id',
      token_secret_ref: 'secret:zoom-token',
      client_secret_ref: 'secret:zoom-client-secret',
      webhook_secret_ref: 'secret:zoom-webhook-secret',
    })
    await startZoomOAuth({
      account_id: 'zoom-live-1',
      display_name: 'Zoom Live',
      external_account_id: 'zoom-live-external',
      client_id: 'zoom-client-id',
      client_secret: 'zoom-client-secret',
      redirect_uri: 'http://127.0.0.1:8080/zoom/callback',
      scopes: ['meeting:read', 'recording:read'],
    })
    await completeZoomOAuth({
      setup_id: 'zoom-oauth-setup-1',
      state: 'zoom-oauth-state',
      authorization_code: 'zoom-auth-code',
    })
    await authorizeZoomServerToServer({
      account_id: 'zoom-s2s-1',
      client_id: 'zoom-s2s-client-id',
      client_secret: 'zoom-s2s-client-secret',
      zoom_account_id: 'zoom-provider-account',
    })
    await refreshZoomToken({ account_id: 'zoom-live-1', force: true })
    await maintainZoomTokens({ account_id: 'zoom-live-1', refresh_expiring_within_seconds: 300 })
    await syncZoomRecordings({
      account_id: 'zoom-live-1',
      from: '2026-06-01',
      to: '2026-06-30',
      page_size: 30,
      max_meetings: 100,
    })
    await fetchZoomWebhookSubscriptionStatus('zoom-live-1', 'https://api.zoom.example.test/v2')
    await reconcileZoomWebhookSubscription({
      account_id: 'zoom-live-1',
      endpoint_url: 'https://hermes.example.test/api/v1/integrations/zoom/runtime-bridge/webhooks',
      event_types: ['meeting.started'],
      api_base_url: 'https://api.zoom.example.test/v2',
    })
    await removeZoomWebhookSubscription({
      account_id: 'zoom-live-1',
      subscription_id: 'zoom-subscription-1',
      api_base_url: 'https://api.zoom.example.test/v2',
    })
    await fetchZoomRuntimeStatus(' zoom-live-1 ')
    await startZoomRuntime({ account_id: 'zoom-live-1', force: true })
    await stopZoomRuntime({ account_id: 'zoom-live-1', reason: 'test' })
    await removeZoomRuntime({ account_id: 'zoom-live-1', reason: 'test cleanup' })
    await fetchZoomRecordingImports(' zoom-live-1 ', 12)
    await fetchZoomAuditEvents(' zoom-live-1 ', 10)
    await removeZoomRecordingImport(' zoom-live-1 ', ' attachment-1 ', {
      reason: 'operator_removed_recording_import',
    })
    await cleanupZoomRetention(' zoom-live-1 ', {
      remove_recordings: true,
      remove_transcripts: true,
      limit: 100,
    })
    await fetchZoomRecordingImports(' zoom-live-1 ', 12)

    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/integrations/zoom/capabilities')
    expect(fetchMock.mock.calls[1][0]).toContain('/api/v1/integrations/zoom/accounts?include_removed=true')
    expect(fetchMock.mock.calls[2][0]).toContain('/api/v1/integrations/zoom/fixtures/accounts')
    expect(fetchMock.mock.calls[3][0]).toContain('/api/v1/integrations/zoom/accounts')
    expect(fetchMock.mock.calls[4][0]).toContain('/api/v1/integrations/zoom/oauth/start')
    expect(fetchMock.mock.calls[5][0]).toContain('/api/v1/integrations/zoom/oauth/complete')
    expect(fetchMock.mock.calls[6][0]).toContain(
      '/api/v1/integrations/zoom/oauth/server-to-server/authorize'
    )
    expect(fetchMock.mock.calls[7][0]).toContain('/api/v1/integrations/zoom/oauth/refresh')
    expect(fetchMock.mock.calls[8][0]).toContain('/api/v1/integrations/zoom/oauth/maintenance')
    expect(fetchMock.mock.calls[9][0]).toContain('/api/v1/integrations/zoom/provider-sync/recordings')
    expect(fetchMock.mock.calls[10][0]).toContain(
      '/api/v1/integrations/zoom/webhook-subscriptions/status?account_id=zoom-live-1&api_base_url=https%3A%2F%2Fapi.zoom.example.test%2Fv2'
    )
    expect(fetchMock.mock.calls[11][0]).toContain(
      '/api/v1/integrations/zoom/webhook-subscriptions/reconcile'
    )
    expect(fetchMock.mock.calls[12][0]).toContain(
      '/api/v1/integrations/zoom/webhook-subscriptions/remove'
    )
    expect(fetchMock.mock.calls[13][0]).toContain(
      '/api/v1/integrations/zoom/runtime/status?account_id=zoom-live-1'
    )
    expect(fetchMock.mock.calls[14][0]).toContain('/api/v1/integrations/zoom/runtime/start')
    expect(fetchMock.mock.calls[15][0]).toContain('/api/v1/integrations/zoom/runtime/stop')
    expect(fetchMock.mock.calls[16][0]).toContain('/api/v1/integrations/zoom/runtime/remove')
    expect(fetchMock.mock.calls[17][0]).toContain(
      '/api/v1/integrations/zoom/accounts/zoom-live-1/recording-imports?limit=12'
    )
    expect(fetchMock.mock.calls[18][0]).toContain(
      '/api/v1/integrations/zoom/accounts/zoom-live-1/audit-events?limit=10'
    )
    expect(fetchMock.mock.calls[19][0]).toContain(
      '/api/v1/integrations/zoom/accounts/zoom-live-1/recording-imports/attachment-1/remove'
    )
    expect(fetchMock.mock.calls[20][0]).toContain(
      '/api/v1/integrations/zoom/accounts/zoom-live-1/retention/prune'
    )
    expect(fetchMock.mock.calls[21][0]).toContain(
      '/api/v1/integrations/zoom/accounts/zoom-live-1/recording-imports?limit=12'
    )
    expect(JSON.parse(fetchMock.mock.calls[3][1].body as string)).toMatchObject({
      auth_shape: 'oauth_user',
      client_id: 'zoom-client-id',
      token_secret_ref: 'secret:zoom-token',
      client_secret_ref: 'secret:zoom-client-secret',
      webhook_secret_ref: 'secret:zoom-webhook-secret',
    })
    expect(JSON.parse(fetchMock.mock.calls[4][1].body as string)).toMatchObject({
      account_id: 'zoom-live-1',
      client_id: 'zoom-client-id',
      client_secret: 'zoom-client-secret',
      redirect_uri: 'http://127.0.0.1:8080/zoom/callback',
    })
    expect(JSON.parse(fetchMock.mock.calls[5][1].body as string)).toMatchObject({
      setup_id: 'zoom-oauth-setup-1',
      state: 'zoom-oauth-state',
      authorization_code: 'zoom-auth-code',
    })
    expect(JSON.parse(fetchMock.mock.calls[6][1].body as string)).toMatchObject({
      account_id: 'zoom-s2s-1',
      client_id: 'zoom-s2s-client-id',
      client_secret: 'zoom-s2s-client-secret',
      zoom_account_id: 'zoom-provider-account',
    })
    expect(JSON.parse(fetchMock.mock.calls[7][1].body as string)).toMatchObject({
      account_id: 'zoom-live-1',
      force: true,
    })
    expect(JSON.parse(fetchMock.mock.calls[8][1].body as string)).toMatchObject({
      account_id: 'zoom-live-1',
      refresh_expiring_within_seconds: 300,
    })
    expect(JSON.parse(fetchMock.mock.calls[9][1].body as string)).toMatchObject({
      account_id: 'zoom-live-1',
      from: '2026-06-01',
      to: '2026-06-30',
      page_size: 30,
      max_meetings: 100,
    })
    expect(JSON.parse(fetchMock.mock.calls[11][1].body as string)).toMatchObject({
      account_id: 'zoom-live-1',
      endpoint_url: 'https://hermes.example.test/api/v1/integrations/zoom/runtime-bridge/webhooks',
      event_types: ['meeting.started'],
      api_base_url: 'https://api.zoom.example.test/v2',
    })
    expect(JSON.parse(fetchMock.mock.calls[12][1].body as string)).toMatchObject({
      account_id: 'zoom-live-1',
      subscription_id: 'zoom-subscription-1',
      api_base_url: 'https://api.zoom.example.test/v2',
    })
    expect(JSON.parse(fetchMock.mock.calls[20][1].body as string)).toMatchObject({
      remove_recordings: true,
      remove_transcripts: true,
      limit: 100,
    })
    expect(JSON.parse(fetchMock.mock.calls[19][1].body as string)).toMatchObject({
      reason: 'operator_removed_recording_import',
    })
  })

  it('builds expected runtime bridge routes', async () => {
    const ok = (body: unknown) =>
      new Response(JSON.stringify(body), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(ok({ call_id: 'zoom-call-1', status: 'recorded' }))
      .mockResolvedValueOnce(ok({ recording_id: 'recording-1', status: 'recorded' }))
      .mockResolvedValueOnce(ok({ transcript_id: 'transcript-1', status: 'recorded' }))
      .mockResolvedValueOnce(ok({
        transcript_id: 'transcript-file-1',
        status: 'recorded',
        import_format: 'webvtt',
        parsed_segment_count: 1,
      }))
    vi.stubGlobal('fetch', fetchMock)

    await bridgeZoomMeeting({
      account_id: 'zoom-1',
      meeting_id: 'meeting-1',
      metadata: { safe: true },
    })
    await bridgeZoomRecording({
      account_id: 'zoom-1',
      meeting_id: 'meeting-1',
      recording: { recording_id: 'recording-1' },
    })
    await bridgeZoomTranscript({
      transcript_id: 'transcript-1',
      account_id: 'zoom-1',
      meeting_id: 'meeting-1',
      transcript_text: 'hello',
      segments: [],
    })
    await importZoomTranscriptFile({
      transcript_id: 'transcript-file-1',
      account_id: 'zoom-1',
      meeting_id: 'meeting-1',
      file_name: 'meeting.vtt',
      content_type: 'text/vtt',
      file_text: 'WEBVTT\n\n00:00:00.000 --> 00:00:01.000\nhello',
    })

    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/integrations/zoom/runtime-bridge/meetings')
    expect(fetchMock.mock.calls[1][0]).toContain('/api/v1/integrations/zoom/runtime-bridge/recordings')
    expect(fetchMock.mock.calls[2][0]).toContain('/api/v1/integrations/zoom/runtime-bridge/transcripts')
    expect(fetchMock.mock.calls[3][0]).toContain('/api/v1/integrations/zoom/runtime-bridge/transcript-files')
    expect(fetchMock.mock.calls.every((call) => call[1].method === 'POST')).toBe(true)
  })

  it('builds expected observed call routes', async () => {
    const ok = (body: unknown) =>
      new Response(JSON.stringify(body), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(ok({ items: [] }))
      .mockResolvedValueOnce(ok({ transcript: null }))
    vi.stubGlobal('fetch', fetchMock)

    await fetchZoomProviderCalls(' zoom-1 ', 12)
    await fetchZoomCallTranscript('call-1')

    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/calls?limit=12&provider=zoom&account_id=zoom-1')
    expect(fetchMock.mock.calls[1][0]).toContain('/api/v1/calls/call-1/transcript')
    expect(fetchMock.mock.calls[0][1].method).toBe('GET')
    expect(fetchMock.mock.calls[1][1].method).toBe('GET')
  })
})
