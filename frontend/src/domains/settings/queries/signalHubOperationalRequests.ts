import type {
  SignalHubConnectionCreateRequest,
  SignalHubConnectionUpdateRequest,
  SignalHubHealthCheckRequest,
  SignalHubRuntimeState,
  SignalHubRuntimeStateRequest
} from '../types/signalHub'

export function buildSignalHubConnectionCreateRequest(
  sourceCode: string,
  displayName: string,
  profile: string
): SignalHubConnectionCreateRequest {
  return {
    source_code: sourceCode,
    display_name: displayName,
    status: 'connected',
    profile,
    settings: {}
  }
}

export function buildSignalHubConnectionStatusRequest(
  status: string
): SignalHubConnectionUpdateRequest {
  return { status }
}

export function buildSignalHubRuntimeStateRequest(
  runtime: SignalHubRuntimeState,
  state: string
): SignalHubRuntimeStateRequest {
  return {
    source_code: runtime.source_code,
    runtime_kind: runtime.runtime_kind,
    state,
    metadata: runtime.metadata
  }
}

export function buildSignalHubHealthCheckRequest(
  sourceCode: string,
  connectionId: string | null | undefined
): SignalHubHealthCheckRequest {
  return {
    source_code: sourceCode,
    connection_id: connectionId ?? null
  }
}
