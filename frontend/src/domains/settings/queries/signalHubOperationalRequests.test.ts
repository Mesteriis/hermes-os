import { describe, expect, it } from 'vitest'
import {
  buildSignalHubConnectionCreateRequest,
  buildSignalHubConnectionStatusRequest,
  buildSignalHubHealthCheckRequest,
  buildSignalHubRuntimeStateRequest
} from './signalHubOperationalRequests'
import type { SignalHubRuntimeState } from '../types/signalHub'

describe('signal hub operational requests', () => {
  it('builds a connection create request with explicit initial status', () => {
    expect(buildSignalHubConnectionCreateRequest('telegram', 'Primary', 'default')).toEqual({
      source_code: 'telegram', display_name: 'Primary', status: 'connected',
      profile: 'default', settings: {}
    })
  })

  it('preserves runtime metadata when changing runtime state', () => {
    const runtime = runtimeState()
    expect(buildSignalHubRuntimeStateRequest(runtime, 'paused')).toEqual({
      source_code: 'telegram', runtime_kind: 'ingest', state: 'paused', metadata: { owner: 'system' }
    })
    expect(buildSignalHubConnectionStatusRequest('disabled')).toEqual({ status: 'disabled' })
    expect(buildSignalHubHealthCheckRequest('telegram', undefined)).toEqual({
      source_code: 'telegram', connection_id: null
    })
  })
})

function runtimeState(): SignalHubRuntimeState {
  return {
    id: 'runtime-1', source_code: 'telegram', connection_id: null,
    runtime_kind: 'ingest', state: 'running', last_started_at: null,
    last_stopped_at: null, last_heartbeat_at: null, last_error_at: null,
    last_error_code: null, last_error_message_redacted: null,
    metadata: { owner: 'system' }, updated_at: ''
  }
}
