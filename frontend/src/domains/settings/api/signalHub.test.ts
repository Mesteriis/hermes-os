import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api/ApiClient'
import { resetSignalHubConnectClientForTests } from '../../../platform/connect/signalHubClient'
import {
  applySignalHubProfile,
  fetchSignalHubCapabilities,
  createSignalHubProfile,
  createSignalHubReplayRequest,
  createSignalHubPolicy,
  createSignalHubConnection,
  disableSignalHubSignals,
  disableSignalHubSource,
  emitSignalHubFixtureSignal,
  enableSignalHubSignals,
  enableSignalHubSource,
  fetchSignalHubFixtureSources,
  fetchSignalHubProfiles,
  fetchSignalHubConnections,
  fetchSignalHubHealth,
  fetchSignalHubPolicies,
  fetchSignalHubReplayRequests,
  fetchSignalHubRuntimeStates,
  muteSignalHubSignals,
  pauseSignalHubSignals,
  fetchSignalHubSources,
  removeSignalHubConnection,
  removeSignalHubProfile,
  resumeSignalHubSignals,
  restoreSignalHubSystemFixture,
  runSignalHubHealthCheck,
  unmuteSignalHubSignals,
  updateSignalHubConnection,
  updateSignalHubProfile,
  updateSignalHubRuntimeState
} from './signalHub'

describe('Signal Hub settings API', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
    resetSignalHubConnectClientForTests()
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    resetSignalHubConnectClientForTests()
    ApiClient.resetForTests()
  })

  it('fetches Signal Hub sources through the protected API client', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ items: [] }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await fetchSignalHubSources()

    expect(fetchMock).toHaveBeenCalledOnce()
    const [url, options] = fetchMock.mock.calls[0]
    expect(url).toBe('http://127.0.0.1:8080/hermes.signal_hub.v1.SignalHubService/ListSources')
    expect(options.method).toBe('POST')
    expect(new Headers(options.headers).get('X-Hermes-Secret')).toBe('test-secret')
  })

  it('lists Signal Hub capabilities through ConnectRPC', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({
        items: [{
          id: 'cap-1',
          sourceCode: 'telegram',
          capability: 'runtime.replay',
          state: 'available',
          reason: 'source events can be replayed from durable Signal Hub history',
          requiresConfirmation: false,
          actionClass: 'local_write',
          updatedAt: '2026-06-23T00:00:00Z'
        }]
      }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    const response = await fetchSignalHubCapabilities()

    expect(response.items).toEqual([{
      id: 'cap-1',
      source_code: 'telegram',
      connection_id: null,
      capability: 'runtime.replay',
      state: 'available',
      reason: 'source events can be replayed from durable Signal Hub history',
      requires_confirmation: false,
      action_class: 'local_write',
      updated_at: '2026-06-23T00:00:00Z'
    }])
    expect(fetchMock.mock.calls[0][0]).toBe(
      'http://127.0.0.1:8080/hermes.signal_hub.v1.SignalHubService/ListCapabilities'
    )
  })

  it('runs source and scoped signal control commands through ConnectRPC', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ sourceCode: 'telegram', clearedCount: 1 }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ sourceCode: 'telegram', policyId: 'policy-1' }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ policyId: 'policy-2' }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ clearedCount: 2 }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ policyId: 'policy-3' }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ clearedCount: 2 }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ policyId: 'policy-4' }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ clearedCount: 3 }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    await enableSignalHubSource('telegram')
    await disableSignalHubSource('telegram')
    await disableSignalHubSignals({ scope: 'event_pattern', event_pattern: 'signal.raw.telegram.*' })
    await enableSignalHubSignals({ scope: 'event_pattern', event_pattern: 'signal.raw.telegram.*' })
    await muteSignalHubSignals({ scope: 'event_pattern', event_pattern: 'signal.raw.telegram.*' })
    await unmuteSignalHubSignals({ scope: 'event_pattern', event_pattern: 'signal.raw.telegram.*' })
    await pauseSignalHubSignals({ scope: 'global' })
    await resumeSignalHubSignals({ scope: 'global' })

    expect(fetchMock.mock.calls[0][0]).toBe(
      'http://127.0.0.1:8080/hermes.signal_hub.v1.SignalHubService/EnableSource'
    )
    expect(fetchMock.mock.calls[1][0]).toBe(
      'http://127.0.0.1:8080/hermes.signal_hub.v1.SignalHubService/DisableSource'
    )
    expect(fetchMock.mock.calls[2][0]).toBe(
      'http://127.0.0.1:8080/hermes.signal_hub.v1.SignalHubService/DisableSignals'
    )
    expect(fetchMock.mock.calls[3][0]).toBe(
      'http://127.0.0.1:8080/hermes.signal_hub.v1.SignalHubService/EnableSignals'
    )
    expect(fetchMock.mock.calls[4][0]).toBe(
      'http://127.0.0.1:8080/hermes.signal_hub.v1.SignalHubService/MuteSignals'
    )
    expect(fetchMock.mock.calls[5][0]).toBe(
      'http://127.0.0.1:8080/hermes.signal_hub.v1.SignalHubService/UnmuteSignals'
    )
    expect(fetchMock.mock.calls[6][0]).toBe(
      'http://127.0.0.1:8080/hermes.signal_hub.v1.SignalHubService/PauseSignals'
    )
    expect(fetchMock.mock.calls[7][0]).toBe(
      'http://127.0.0.1:8080/hermes.signal_hub.v1.SignalHubService/ResumeSignals'
    )
    expect(JSON.parse(decodeBody(fetchMock.mock.calls[2][1].body))).toMatchObject({
      scope: 'event_pattern',
      eventPattern: 'signal.raw.telegram.*'
    })
    expect(JSON.parse(decodeBody(fetchMock.mock.calls[4][1].body))).toMatchObject({
      scope: 'event_pattern',
      eventPattern: 'signal.raw.telegram.*'
    })
  })

  it('restores the system fixture through the Signal Hub recovery endpoint', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ sourcesCreated: 13, sourcesRepaired: 0, profilesCreated: 4, profilesRepaired: 0 }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    const report = await restoreSignalHubSystemFixture()

    expect(report.sources_created).toBe(13)
    expect(report.profiles_created).toBe(4)
    expect(fetchMock).toHaveBeenCalledOnce()
    const [url, options] = fetchMock.mock.calls[0]
    expect(url).toBe('http://127.0.0.1:8080/hermes.signal_hub.v1.SignalHubService/RestoreSystemFixture')
    expect(options.method).toBe('POST')
  })

  it('lists Signal Hub fixture sources through ConnectRPC', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({
        items: [{
          fixtureId: 'fixture_basic_message',
          sourceCode: 'fixture',
          eventType: 'signal.raw.fixture.message.observed',
          correlationId: 'fixture-basic-message',
          occurredAt: '2026-01-01T00:00:00Z',
          summary: 'Fixture message'
        }]
      }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    const response = await fetchSignalHubFixtureSources()

    expect(response.items).toEqual([{
      fixture_id: 'fixture_basic_message',
      source_code: 'fixture',
      event_type: 'signal.raw.fixture.message.observed',
      correlation_id: 'fixture-basic-message',
      occurred_at: '2026-01-01T00:00:00Z',
      summary: 'Fixture message'
    }])
    expect(fetchMock).toHaveBeenCalledOnce()
    expect(fetchMock.mock.calls[0][0]).toBe('http://127.0.0.1:8080/hermes.signal_hub.v1.SignalHubService/ListFixtureSources')
  })

  it('creates, updates, applies and removes Signal Hub custom profiles through ConnectRPC', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          item: {
            id: 'profile-custom-1',
            code: 'quiet_hours',
            displayName: 'Quiet Hours',
            description: 'Mute noisy sources overnight.',
            policyCount: 2,
            sourcePolicies: [
              { scope: 'source', sourceCode: 'telegram', mode: 'muted', reason: 'night mute' },
              { scope: 'source', sourceCode: 'mail', mode: 'paused', reason: 'overnight pause' }
            ],
            isSystem: false,
            isActive: false,
            createdAt: '2026-06-23T00:00:00Z',
            updatedAt: '2026-06-23T00:00:00Z'
          }
        }), { status: 200, headers: { 'Content-Type': 'application/json' } })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          item: {
            id: 'profile-custom-1',
            code: 'quiet_hours',
            displayName: 'Quiet Hours',
            description: 'Updated description.',
            policyCount: 1,
            sourcePolicies: [
              { scope: 'event_pattern', eventPattern: 'signal.raw.mail.*', mode: 'muted', reason: 'mail quiet hours' }
            ],
            isSystem: false,
            isActive: false,
            createdAt: '2026-06-23T00:00:00Z',
            updatedAt: '2026-06-23T00:10:00Z'
          }
        }), { status: 200, headers: { 'Content-Type': 'application/json' } })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          item: {
            id: 'profile-custom-1',
            code: 'quiet_hours',
            displayName: 'Quiet Hours',
            description: 'Updated description.',
            policyCount: 1,
            sourcePolicies: [
              { scope: 'event_pattern', eventPattern: 'signal.raw.mail.*', mode: 'muted', reason: 'mail quiet hours' }
            ],
            isSystem: false,
            isActive: true,
            createdAt: '2026-06-23T00:00:00Z',
            updatedAt: '2026-06-23T00:10:00Z'
          }
        }), { status: 200, headers: { 'Content-Type': 'application/json' } })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          item: {
            id: 'profile-custom-1',
            code: 'quiet_hours',
            displayName: 'Quiet Hours',
            description: 'Updated description.',
            policyCount: 1,
            sourcePolicies: [
              { scope: 'event_pattern', eventPattern: 'signal.raw.mail.*', mode: 'muted', reason: 'mail quiet hours' }
            ],
            isSystem: false,
            isActive: false,
            createdAt: '2026-06-23T00:00:00Z',
            updatedAt: '2026-06-23T00:10:00Z'
          }
        }), { status: 200, headers: { 'Content-Type': 'application/json' } })
      )
    vi.stubGlobal('fetch', fetchMock)

    const created = await createSignalHubProfile({
      code: 'quiet_hours',
      display_name: 'Quiet Hours',
      description: 'Mute noisy sources overnight.',
      source_policies: [
        { scope: 'source', source_code: 'telegram', connection_id: null, event_pattern: null, mode: 'muted', reason: 'night mute' },
        { scope: 'source', source_code: 'mail', connection_id: null, event_pattern: null, mode: 'paused', reason: 'overnight pause' }
      ]
    })
    const updated = await updateSignalHubProfile('quiet_hours', {
      description: 'Updated description.',
      source_policies: [
        { scope: 'event_pattern', source_code: null, connection_id: null, event_pattern: 'signal.raw.mail.*', mode: 'muted', reason: 'mail quiet hours' }
      ]
    })
    const applied = await applySignalHubProfile('quiet_hours')
    const removed = await removeSignalHubProfile('quiet_hours')

    expect(created.source_policies).toHaveLength(2)
    expect(updated.source_policies[0].event_pattern).toBe('signal.raw.mail.*')
    expect(applied.is_active).toBe(true)
    expect(removed.code).toBe('quiet_hours')
    expect(fetchMock.mock.calls[0][0]).toBe(
      'http://127.0.0.1:8080/hermes.signal_hub.v1.SignalHubService/CreateProfile'
    )
    expect(fetchMock.mock.calls[1][0]).toBe(
      'http://127.0.0.1:8080/hermes.signal_hub.v1.SignalHubService/UpdateProfile'
    )
    expect(fetchMock.mock.calls[2][0]).toBe(
      'http://127.0.0.1:8080/hermes.signal_hub.v1.SignalHubService/ApplyProfile'
    )
    expect(fetchMock.mock.calls[3][0]).toBe(
      'http://127.0.0.1:8080/hermes.signal_hub.v1.SignalHubService/RemoveProfile'
    )
    expect(JSON.parse(decodeBody(fetchMock.mock.calls[1][1].body))).toMatchObject({
      code: 'quiet_hours',
      updateSourcePolicies: true,
      sourcePolicies: [{ eventPattern: 'signal.raw.mail.*', mode: 'muted' }]
    })
  })

  it('fetches Signal Hub connections, runtimes, health and replay through protected endpoints', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          items: [{
            id: 'conn-1',
            sourceCode: 'mail',
            displayName: 'Primary Mail',
            status: 'connected',
            profile: 'default',
            secretRef: 'secret:mail:primary',
            connectedAt: '2026-06-23T00:00:00Z',
            lastSeenAt: '2026-06-23T00:10:00Z',
            lastSignalAt: '2026-06-23T00:09:00Z',
            lastSyncAt: '2026-06-23T00:08:00Z',
            createdAt: '2026-06-22T00:00:00Z',
            updatedAt: '2026-06-23T00:10:00Z',
            settingsJson: '{"account_id":"mail-primary","provider_kind":"gmail"}'
          }]
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          items: [{
            id: 'runtime-1',
            sourceCode: 'mail',
            connectionId: 'conn-1',
            runtimeKind: 'mail_background_sync',
            state: 'paused',
            metadataJson: '{"scope":"scheduler"}',
            updatedAt: '2026-06-23T00:00:00Z',
            lastStartedAt: '2026-06-22T23:00:00Z',
            lastStoppedAt: '2026-06-22T23:30:00Z',
            lastHeartbeatAt: '2026-06-22T23:29:30Z',
            lastErrorAt: '2026-06-22T23:28:00Z',
            lastErrorCode: 'MAIL_TIMEOUT',
            lastErrorMessageRedacted: 'background sync timed out'
          }]
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          items: [{
            id: 'health-1',
            sourceCode: 'mail',
            connectionId: 'conn-1',
            level: 'degraded',
            summary: 'Mail provider needs retry',
            lastOkAt: '2026-06-22T22:00:00Z',
            lastFailureAt: '2026-06-22T23:28:00Z',
            failureCount: 3,
            consecutiveFailureCount: 2,
            nextRetryAt: '2026-06-23T00:15:00Z',
            updatedAt: '2026-06-23T00:00:00Z',
            evidenceJson: '{"error_code":"MAIL_TIMEOUT","retries_remaining":2}'
          }]
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          items: [{
            id: 'replay-1',
            sourceCode: 'telegram',
            eventPattern: 'signal.raw.telegram.*',
            fromPosition: '10',
            toPosition: '20',
            fromTime: '2026-06-23T00:00:00Z',
            toTime: '2026-06-23T01:00:00Z',
            targetProjection: 'timeline_event_log',
            status: 'queued',
            requestedBy: 'hermes-frontend',
            requestedAt: '2026-06-23T00:00:00Z',
            replayedCount: 0,
            metadataJson: '{"requested_from":"ui"}'
          }]
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    const connections = await fetchSignalHubConnections()
    const runtimes = await fetchSignalHubRuntimeStates()
    const health = await fetchSignalHubHealth()
    const replay = await fetchSignalHubReplayRequests()

    expect(fetchMock).toHaveBeenCalledTimes(4)
    expect(fetchMock.mock.calls[0][0]).toBe('http://127.0.0.1:8080/hermes.signal_hub.v1.SignalHubService/ListConnections')
    expect(fetchMock.mock.calls[1][0]).toBe('http://127.0.0.1:8080/hermes.signal_hub.v1.SignalHubService/ListRuntimeStates')
    expect(fetchMock.mock.calls[2][0]).toBe('http://127.0.0.1:8080/hermes.signal_hub.v1.SignalHubService/ListHealth')
    expect(fetchMock.mock.calls[3][0]).toBe('http://127.0.0.1:8080/hermes.signal_hub.v1.SignalHubService/ListReplayRequests')
    expect(connections.items[0].settings).toEqual({
      account_id: 'mail-primary',
      provider_kind: 'gmail'
    })
    expect(runtimes.items[0].last_started_at).toBe('2026-06-22T23:00:00Z')
    expect(runtimes.items[0].last_error_code).toBe('MAIL_TIMEOUT')
    expect(runtimes.items[0].last_error_message_redacted).toBe('background sync timed out')
    expect(health.items[0].evidence).toEqual({
      error_code: 'MAIL_TIMEOUT',
      retries_remaining: 2
    })
    expect(replay.items[0].from_position).toBe(10n)
    expect(replay.items[0].to_position).toBe(20n)
    expect(replay.items[0].from_time).toBe('2026-06-23T00:00:00Z')
    expect(replay.items[0].to_time).toBe('2026-06-23T01:00:00Z')
    expect(replay.items[0].target_projection).toBe('timeline_event_log')
  })

  it('runs Signal Hub health checks through ConnectRPC', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(
        JSON.stringify({
          item: {
            id: 'health-1',
            sourceCode: 'system',
            connectionId: null,
            level: 'healthy',
            summary: 'System runtime is healthy',
            lastOkAt: '2026-06-23T00:00:00Z',
            failureCount: 0,
            consecutiveFailureCount: 0,
            updatedAt: '2026-06-23T00:00:00Z'
          }
        }),
        {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        }
      )
    )
    vi.stubGlobal('fetch', fetchMock)

    const result = await runSignalHubHealthCheck({ source_code: 'system' })

    expect(result.source_code).toBe('system')
    expect(result.level).toBe('healthy')
    const [url, options] = fetchMock.mock.calls[0]
    expect(url).toBe('http://127.0.0.1:8080/hermes.signal_hub.v1.SignalHubService/RunHealthCheck')
    expect(options.method).toBe('POST')
    expect(JSON.parse(decodeBody(options.body))).toMatchObject({
      sourceCode: 'system'
    })
  })

  it('creates Signal Hub replay requests through ConnectRPC', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(
        JSON.stringify({
          item: {
            id: 'replay-1',
            sourceCode: 'telegram',
            eventPattern: 'signal.raw.telegram.*',
            fromPosition: '10',
            toPosition: '20',
            status: 'queued',
            requestedBy: 'hermes-frontend',
            requestedAt: '2026-06-23T00:00:00Z',
            replayedCount: 0,
            metadataJson: '{"requested_from":"settings_signal_hub"}'
          }
        }),
        {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        }
      )
    )
    vi.stubGlobal('fetch', fetchMock)

    await createSignalHubReplayRequest({
      source_code: 'telegram',
      event_pattern: 'signal.raw.telegram.*',
      from_position: 10,
      to_position: 20,
      target_consumer: 'signal_hub_raw_signal_dispatcher',
      metadata: { requested_from: 'settings_signal_hub' }
    })

    expect(fetchMock).toHaveBeenCalledOnce()
    const [url, options] = fetchMock.mock.calls[0]
    expect(url).toBe('http://127.0.0.1:8080/hermes.signal_hub.v1.SignalHubService/RequestReplay')
    expect(options.method).toBe('POST')
    expect(JSON.parse(decodeBody(options.body))).toMatchObject({
      sourceCode: 'telegram',
      eventPattern: 'signal.raw.telegram.*',
      fromPosition: '10',
      toPosition: '20',
      targetConsumer: 'signal_hub_raw_signal_dispatcher'
    })
  })

  it('creates connection-scoped replay requests through ConnectRPC', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(
        JSON.stringify({
          item: {
            id: 'replay-2',
            sourceCode: 'mail',
            connectionId: 'conn-1',
            eventPattern: 'signal.raw.mail.*',
            status: 'queued',
            requestedBy: 'hermes-frontend',
            requestedAt: '2026-06-23T00:00:00Z',
            replayedCount: 0,
            metadataJson: '{"requested_from":"settings_signal_hub"}'
          }
        }),
        {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        }
      )
    )
    vi.stubGlobal('fetch', fetchMock)

    await createSignalHubReplayRequest({
      source_code: 'mail',
      connection_id: 'conn-1',
      event_pattern: 'signal.raw.mail.*',
      metadata: { requested_from: 'settings_signal_hub' }
    })

    const [, options] = fetchMock.mock.calls[0]
    expect(JSON.parse(decodeBody(options.body))).toMatchObject({
      sourceCode: 'mail',
      connectionId: 'conn-1',
      eventPattern: 'signal.raw.mail.*'
    })
  })

  it('lists and applies Signal Hub profiles through ConnectRPC', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            items: [
              {
                id: 'profile-testing',
                code: 'testing',
                displayName: 'Testing',
                description: 'Real sources muted; fixture enabled.',
                policyCount: 11,
                isSystem: true,
                isActive: false,
                createdAt: '2026-06-23T00:00:00Z',
                updatedAt: '2026-06-23T00:00:00Z'
              }
            ]
          }),
          {
            status: 200,
            headers: { 'Content-Type': 'application/json' }
          }
        )
      )
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            item: {
              id: 'profile-testing',
              code: 'testing',
              displayName: 'Testing',
              description: 'Real sources muted; fixture enabled.',
              policyCount: 11,
              isSystem: true,
              isActive: true,
              createdAt: '2026-06-23T00:00:00Z',
              updatedAt: '2026-06-23T00:00:00Z'
            }
          }),
          {
            status: 200,
            headers: { 'Content-Type': 'application/json' }
          }
        )
      )
    vi.stubGlobal('fetch', fetchMock)

    const profiles = await fetchSignalHubProfiles()
    const applied = await applySignalHubProfile('testing')

    expect(profiles.items[0].code).toBe('testing')
    expect(applied.code).toBe('testing')
    expect(applied.is_active).toBe(true)
    expect(fetchMock.mock.calls[0][0]).toBe(
      'http://127.0.0.1:8080/hermes.signal_hub.v1.SignalHubService/ListProfiles'
    )
    expect(fetchMock.mock.calls[1][0]).toBe(
      'http://127.0.0.1:8080/hermes.signal_hub.v1.SignalHubService/ApplyProfile'
    )
    expect(JSON.parse(decodeBody(fetchMock.mock.calls[1][1].body))).toMatchObject({
      code: 'testing'
    })
  })

  it('emits Signal Hub fixture signals through ConnectRPC', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(
        JSON.stringify({
          fixtureId: 'fixture_basic_message',
          rawEventId: 'evt_signal_fixture_1',
          eventType: 'signal.raw.fixture.message.observed',
          sourceCode: 'fixture',
          correlationId: 'fixture-basic-message'
        }),
        {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        }
      )
    )
    vi.stubGlobal('fetch', fetchMock)

    const result = await emitSignalHubFixtureSignal('fixture_basic_message')

    expect(result.fixture_id).toBe('fixture_basic_message')
    expect(result.source_code).toBe('fixture')
    expect(fetchMock).toHaveBeenCalledOnce()
    const [url, options] = fetchMock.mock.calls[0]
    expect(url).toBe(
      'http://127.0.0.1:8080/hermes.signal_hub.v1.SignalHubService/EmitFixtureSignal'
    )
    expect(options.method).toBe('POST')
    expect(JSON.parse(decodeBody(options.body))).toMatchObject({
      fixtureId: 'fixture_basic_message'
    })
  })

  it('creates Signal Hub policies through the protected API client', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ id: 'policy-1' }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await createSignalHubPolicy({
      scope: 'event_pattern',
      event_pattern: 'signal.raw.*',
      mode: 'paused',
      reason: 'maintenance'
    })

    expect(fetchMock).toHaveBeenCalledOnce()
    const [url, options] = fetchMock.mock.calls[0]
    expect(url).toBe('http://127.0.0.1:8080/hermes.signal_hub.v1.SignalHubService/CreatePolicy')
    expect(options.method).toBe('POST')
    expect(JSON.parse(decodeBody(options.body))).toMatchObject({
      scope: 'event_pattern',
      eventPattern: 'signal.raw.*',
      mode: 'paused'
    })
  })

  it('creates connection-scoped Signal Hub policies through ConnectRPC', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ id: 'policy-connection-1' }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await createSignalHubPolicy({
      scope: 'connection',
      source_code: 'mail',
      connection_id: 'conn-1',
      mode: 'muted',
      reason: 'connection maintenance'
    })

    const [, options] = fetchMock.mock.calls[0]
    expect(JSON.parse(decodeBody(options.body))).toMatchObject({
      scope: 'connection',
      sourceCode: 'mail',
      connectionId: 'conn-1',
      mode: 'muted'
    })
  })

  it('fetches Signal Hub policies through ConnectRPC', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ items: [] }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await fetchSignalHubPolicies()

    expect(fetchMock).toHaveBeenCalledOnce()
    expect(fetchMock.mock.calls[0][0]).toBe(
      'http://127.0.0.1:8080/hermes.signal_hub.v1.SignalHubService/ListPolicies'
    )
    expect(fetchMock.mock.calls[0][1].method).toBe('POST')
  })

  it('updates Signal Hub runtime states through the protected API client', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(
        JSON.stringify({
          id: 'runtime-1',
          sourceCode: 'system',
          connectionId: null,
          runtimeKind: 'signal_hub_raw_signal_dispatcher',
          state: 'paused',
          metadataJson: '{"scope":"consumer"}',
          updatedAt: '2026-06-22T00:00:00Z',
          lastStartedAt: '2026-06-21T23:55:00Z',
          lastHeartbeatAt: '2026-06-21T23:59:00Z'
        }),
        {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        }
      )
    )
    vi.stubGlobal('fetch', fetchMock)

    await updateSignalHubRuntimeState({
      source_code: 'system',
      runtime_kind: 'signal_hub_raw_signal_dispatcher',
      state: 'paused',
      metadata: { scope: 'consumer' }
    })

    expect(fetchMock).toHaveBeenCalledOnce()
    const [url, options] = fetchMock.mock.calls[0]
    expect(url).toBe('http://127.0.0.1:8080/hermes.signal_hub.v1.SignalHubService/UpdateRuntimeState')
    expect(options.method).toBe('POST')
    expect(JSON.parse(decodeBody(options.body))).toMatchObject({
      sourceCode: 'system',
      runtimeKind: 'signal_hub_raw_signal_dispatcher',
      state: 'paused',
      metadataJson: '{"scope":"consumer"}'
    })
  })

  it('creates, updates and removes Signal Hub connections through the protected API client', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ item: { id: 'conn-1', sourceCode: 'telegram', displayName: 'Personal Telegram', status: 'connected', createdAt: '2026-06-22T00:00:00Z', updatedAt: '2026-06-22T00:00:00Z' } }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ item: { id: 'conn-1', sourceCode: 'telegram', displayName: 'Personal Telegram', status: 'paused', createdAt: '2026-06-22T00:00:00Z', updatedAt: '2026-06-22T00:00:00Z' } }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ item: { id: 'conn-1', sourceCode: 'telegram', displayName: 'Personal Telegram', status: 'removed', createdAt: '2026-06-22T00:00:00Z', updatedAt: '2026-06-22T00:00:00Z' } }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    await createSignalHubConnection({
      source_code: 'telegram',
      display_name: 'Personal Telegram',
      status: 'connected',
      profile: 'default',
      settings: { account_id: 'telegram-main', provider_kind: 'telegram_user' }
    })
    await updateSignalHubConnection('conn-1', {
      status: 'paused',
      profile: 'maintenance',
      settings: { account_id: 'telegram-main', provider_kind: 'telegram_user', runtime: 'fixture' }
    })
    await removeSignalHubConnection('conn-1')

    expect(fetchMock).toHaveBeenCalledTimes(3)
    expect(fetchMock.mock.calls[0][0]).toBe('http://127.0.0.1:8080/hermes.signal_hub.v1.SignalHubService/CreateConnection')
    expect(fetchMock.mock.calls[0][1].method).toBe('POST')
    expect(JSON.parse(decodeBody(fetchMock.mock.calls[0][1].body))).toMatchObject({
      sourceCode: 'telegram',
      displayName: 'Personal Telegram',
      settingsJson: '{"account_id":"telegram-main","provider_kind":"telegram_user"}'
    })
    expect(fetchMock.mock.calls[1][0]).toBe('http://127.0.0.1:8080/hermes.signal_hub.v1.SignalHubService/UpdateConnection')
    expect(fetchMock.mock.calls[1][1].method).toBe('POST')
    expect(JSON.parse(decodeBody(fetchMock.mock.calls[1][1].body))).toMatchObject({
      id: 'conn-1',
      status: 'paused',
      profile: 'maintenance',
      settingsJson: '{"account_id":"telegram-main","provider_kind":"telegram_user","runtime":"fixture"}'
    })
    expect(fetchMock.mock.calls[2][0]).toBe('http://127.0.0.1:8080/hermes.signal_hub.v1.SignalHubService/RemoveConnection')
    expect(fetchMock.mock.calls[2][1].method).toBe('POST')
    expect(JSON.parse(decodeBody(fetchMock.mock.calls[2][1].body))).toMatchObject({
      id: 'conn-1'
    })
  })
})

function decodeBody(body: unknown): string {
  if (typeof body === 'string') {
    return body
  }

  if (body instanceof Uint8Array) {
    return new TextDecoder().decode(body)
  }

  if (body instanceof ArrayBuffer) {
    return new TextDecoder().decode(new Uint8Array(body))
  }

  throw new Error(`Unsupported request body type: ${String(body)}`)
}
