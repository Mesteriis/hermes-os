import { describe, expect, it } from 'vitest'
import {
  acceptedSignalPattern,
  buildSignalConsumerGraphRoute,
  buildSignalGraphTabs,
  buildSignalInventoryTabs,
  buildSignalInventoryRow,
  COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER,
  rawSignalPattern,
  SIGNAL_HUB_RAW_SIGNAL_CONSUMER,
  sourceControlState
} from './signalHubRoutePresentation'
import type {
  SignalHubCapability,
  SignalHubConnection,
  SignalHubHealth,
  SignalHubPolicy,
  SignalHubReplayRequest,
  SignalHubRuntimeState,
  SignalHubSource
} from '../types/signalHub'

describe('Signal Hub settings presentation', () => {
  it('derives signal patterns and source-control priority', () => {
    const source = signalSource('telegram', 'communications')
    const policies: SignalHubPolicy[] = [
      signalPolicy('source', 'telegram', 'muted'),
      signalPolicy('source', 'telegram', 'paused'),
      signalPolicy('global', null, 'disabled')
    ]

    expect(rawSignalPattern('telegram')).toBe('signal.raw.telegram.*')
    expect(acceptedSignalPattern('telegram')).toBe('signal.accepted.telegram.*')
    expect(sourceControlState(policies, source)).toBe('disabled')
  })

  it('builds documented and replay-backed consumer graph routes', () => {
    const source = signalSource('telegram', 'communications')
    const replayRequests: SignalHubReplayRequest[] = [
      replayRequest({
        source_code: 'telegram',
        target_projection: 'communication_messages'
      })
    ]

    const route = buildSignalConsumerGraphRoute(source, [], [], replayRequests)

    expect(route.raw_pattern).toBe('signal.raw.telegram.*')
    expect(route.accepted_pattern).toBe('signal.accepted.telegram.*')
    expect(route.targets).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          id: SIGNAL_HUB_RAW_SIGNAL_CONSUMER,
          kind: 'consumer',
          evidence: 'documented'
        }),
        expect.objectContaining({
          id: COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER,
          kind: 'consumer',
          evidence: 'documented'
        }),
        expect.objectContaining({
          id: 'communication_messages',
          kind: 'projection',
          evidence: 'replay'
        })
      ])
    )
  })

  it('builds graph tabs from visible source routes', () => {
    const aiRoute = buildSignalConsumerGraphRoute(signalSource('ai', 'intelligence'), [], [], [])
    const browserRoute = buildSignalConsumerGraphRoute(signalSource('browser', 'capture'), [], [], [])

    expect(buildSignalGraphTabs([aiRoute, browserRoute])).toEqual([
      { id: 'all', label: 'All', count: 2 },
      { id: 'ai', label: 'ai', count: 1 },
      { id: 'browser', label: 'browser', count: 1 }
    ])
  })

  it('builds inventory rows with policies, health, runtime and capability facts', () => {
    const source = signalSource('mail', 'communications')
    const policies = [signalPolicy('event_pattern', null, 'paused', 'signal.raw.mail.*')]
    const connections = [signalConnection('mail')]
    const runtimes = [runtimeState('mail', 'mail_background_sync')]
    const health = [
      healthItem('mail', 'healthy', 'OK'),
      healthItem('mail', 'degraded', 'Mail retry scheduled')
    ]
    const capabilities = [capability('mail', 'runtime.pause')]

    const row = buildSignalInventoryRow(
      source,
      policies,
      connections,
      runtimes,
      health,
      capabilities,
      []
    )

    expect(row.active_policies).toHaveLength(1)
    expect(row.connection_count).toBe(1)
    expect(row.runtime_states).toHaveLength(1)
    expect(row.capabilities[0].capability).toBe('runtime.pause')
    expect(row.health?.level).toBe('degraded')
    expect(buildSignalInventoryTabs([row])).toEqual([
      { id: 'all', label: 'All', count: 1 },
      { id: 'mail', label: 'mail', count: 1 }
    ])
  })
})

function signalSource(code: string, category: string): SignalHubSource {
  return {
    id: `source-${code}`,
    code,
    display_name: code,
    category,
    source_kind: 'provider',
    default_enabled: true,
    supports_connections: true,
    supports_runtime: true,
    supports_replay: true,
    supports_pause: true,
    supports_mute: true,
    capability_schema_version: 1,
    created_at: '2026-06-23T00:00:00Z',
    updated_at: '2026-06-23T00:00:00Z'
  }
}

function signalPolicy(
  scope: SignalHubPolicy['scope'],
  sourceCode: string | null,
  mode: SignalHubPolicy['mode'],
  eventPattern: string | null = null
): SignalHubPolicy {
  return {
    scope,
    source_code: sourceCode,
    connection_id: null,
    event_pattern: eventPattern,
    mode,
    reason: 'test policy',
    expires_at: null
  }
}

function signalConnection(sourceCode: string): SignalHubConnection {
  return {
    id: `connection-${sourceCode}`,
    source_code: sourceCode,
    display_name: `${sourceCode} connection`,
    status: 'connected',
    profile: null,
    settings: {},
    secret_ref: null,
    connected_at: null,
    last_seen_at: null,
    last_signal_at: null,
    last_sync_at: null,
    created_at: '2026-06-23T00:00:00Z',
    updated_at: '2026-06-23T00:00:00Z'
  }
}

function runtimeState(sourceCode: string, runtimeKind: string): SignalHubRuntimeState {
  return {
    id: `runtime-${runtimeKind}`,
    source_code: sourceCode,
    connection_id: null,
    runtime_kind: runtimeKind,
    state: 'running',
    last_started_at: null,
    last_stopped_at: null,
    last_heartbeat_at: null,
    last_error_at: null,
    last_error_code: null,
    last_error_message_redacted: null,
    metadata: {},
    updated_at: '2026-06-23T00:00:00Z'
  }
}

function healthItem(sourceCode: string, level: string, summary: string): SignalHubHealth {
  return {
    id: `health-${sourceCode}-${level}`,
    source_code: sourceCode,
    connection_id: null,
    level,
    summary,
    last_ok_at: null,
    last_failure_at: null,
    failure_count: level === 'healthy' ? 0 : 1,
    consecutive_failure_count: level === 'healthy' ? 0 : 1,
    next_retry_at: null,
    evidence: {},
    updated_at: level === 'healthy' ? '2026-06-23T00:00:00Z' : '2026-06-23T00:01:00Z'
  }
}

function capability(sourceCode: string, capabilityCode: string): SignalHubCapability {
  return {
    id: `capability-${capabilityCode}`,
    source_code: sourceCode,
    connection_id: null,
    capability: capabilityCode,
    state: 'available',
    reason: null,
    requires_confirmation: false,
    action_class: 'admin',
    updated_at: '2026-06-23T00:00:00Z'
  }
}

function replayRequest(
  overrides: Partial<SignalHubReplayRequest>
): SignalHubReplayRequest {
  return {
    id: 'replay-1',
    source_code: null,
    connection_id: null,
    event_pattern: null,
    from_position: null,
    to_position: null,
    from_time: null,
    to_time: null,
    target_consumer: null,
    target_projection: null,
    status: 'queued',
    requested_by: 'test',
    requested_at: '2026-06-23T00:00:00Z',
    started_at: null,
    completed_at: null,
    last_error_redacted: null,
    replayed_count: 0,
    metadata: {},
    ...overrides
  }
}
