import { getSignalHubConnectClient } from '../../../platform/connect/signalHubClient'
import type {
  SignalHubCapabilitiesResponse,
  SignalHubCapability,
  SignalHubConnectionCreateRequest,
  SignalHubConnectionResponse,
  SignalHubConnectionUpdateRequest,
  SignalHubConnectionsResponse,
  SignalHubControlRequest,
  SignalHubControlResponse,
  SignalHubCreatePolicyResponse,
  SignalHubHealthResponse,
  SignalHubHealthCheckRequest,
  SignalHubPoliciesResponse,
  SignalHubPolicyRequest,
  SignalHubProfile,
  SignalHubProfileCreateRequest,
  SignalHubProfilePolicy,
  SignalHubProfileUpdateRequest,
  SignalHubProfilesResponse,
  SignalHubReplayRequestsResponse,
  SignalHubReplayRequestCreateRequest,
  SignalHubRuntimeState,
  SignalHubRuntimeStateRequest,
  SignalHubRuntimeStatesResponse,
  SignalHubSourcesResponse
} from '../types/signalHub'

export async function fetchSignalHubSources(): Promise<SignalHubSourcesResponse> {
  const response = await getSignalHubConnectClient().listSources({})
  return {
    items: response.items.map((item) => ({
      id: item.id,
      code: item.code,
      display_name: item.displayName,
      category: item.category,
      source_kind: item.sourceKind,
      default_enabled: item.defaultEnabled,
      supports_connections: item.supportsConnections,
      supports_runtime: item.supportsRuntime,
      supports_replay: item.supportsReplay,
      supports_pause: item.supportsPause,
      supports_mute: item.supportsMute,
      capability_schema_version: item.capabilitySchemaVersion,
      created_at: item.createdAt,
      updated_at: item.updatedAt
    }))
  }
}

export async function fetchSignalHubSource(sourceCode: string): Promise<SignalHubSourcesResponse['items'][number]> {
  const response = await getSignalHubConnectClient().getSource({
    code: sourceCode
  })
  return {
    id: response.item?.id ?? '',
    code: response.item?.code ?? sourceCode,
    display_name: response.item?.displayName ?? sourceCode,
    category: response.item?.category ?? '',
    source_kind: response.item?.sourceKind ?? '',
    default_enabled: response.item?.defaultEnabled ?? false,
    supports_connections: response.item?.supportsConnections ?? false,
    supports_runtime: response.item?.supportsRuntime ?? false,
    supports_replay: response.item?.supportsReplay ?? false,
    supports_pause: response.item?.supportsPause ?? false,
    supports_mute: response.item?.supportsMute ?? false,
    capability_schema_version: response.item?.capabilitySchemaVersion ?? 1,
    created_at: response.item?.createdAt ?? '',
    updated_at: response.item?.updatedAt ?? ''
  }
}

export async function fetchSignalHubCapabilities(): Promise<SignalHubCapabilitiesResponse> {
  const response = await getSignalHubConnectClient().listCapabilities({})
  return {
    items: response.items.map(mapCapability)
  }
}

export async function fetchSignalHubProfiles(): Promise<SignalHubProfilesResponse> {
  const response = await getSignalHubConnectClient().listProfiles({})
  return {
    items: response.items.map((item) => mapProfile(item, item.code))
  }
}

export async function createSignalHubProfile(
  request: SignalHubProfileCreateRequest
): Promise<SignalHubProfile> {
  const response = await getSignalHubConnectClient().createProfile({
    code: request.code,
    displayName: request.display_name,
    description: request.description,
    sourcePolicies: request.source_policies.map(protoProfilePolicy)
  })
  return mapProfile(response.item, request.code)
}

export async function updateSignalHubProfile(
  profileCode: string,
  request: SignalHubProfileUpdateRequest
): Promise<SignalHubProfile> {
  const response = await getSignalHubConnectClient().updateProfile({
    code: profileCode,
    displayName: request.display_name,
    description: request.description,
    sourcePolicies: (request.source_policies ?? []).map(protoProfilePolicy),
    updateSourcePolicies: request.source_policies !== undefined
  })
  return mapProfile(response.item, profileCode)
}

export async function removeSignalHubProfile(profileCode: string): Promise<SignalHubProfile> {
  const response = await getSignalHubConnectClient().removeProfile({
    code: profileCode
  })
  return mapProfile(response.item, profileCode)
}

export async function applySignalHubProfile(profileCode: string): Promise<SignalHubProfilesResponse['items'][number]> {
  const response = await getSignalHubConnectClient().applyProfile({
    code: profileCode
  })
  return mapProfile(response.item, profileCode)
}

export async function fetchSignalHubConnections(): Promise<SignalHubConnectionsResponse> {
  const response = await getSignalHubConnectClient().listConnections({})
  return {
    items: response.items.map((item) => ({
      id: item.id,
      source_code: item.sourceCode,
      display_name: item.displayName,
      status: item.status,
      profile: item.profile ?? null,
      settings: parseJsonObject(item.settingsJson),
      secret_ref: item.secretRef ?? null,
      connected_at: item.connectedAt ?? null,
      last_seen_at: item.lastSeenAt ?? null,
      last_signal_at: item.lastSignalAt ?? null,
      last_sync_at: item.lastSyncAt ?? null,
      created_at: item.createdAt,
      updated_at: item.updatedAt
    }))
  }
}

export async function createSignalHubConnection(
  request: SignalHubConnectionCreateRequest
): Promise<SignalHubConnectionResponse> {
  const response = await getSignalHubConnectClient().createConnection({
    sourceCode: request.source_code,
    displayName: request.display_name,
    status: request.status,
    profile: request.profile ?? undefined,
    secretRef: request.secret_ref ?? undefined,
    settingsJson: JSON.stringify(request.settings ?? {})
  })
  return {
    item: {
      id: response.item?.id ?? '',
      source_code: response.item?.sourceCode ?? request.source_code,
      display_name: response.item?.displayName ?? request.display_name,
      status: response.item?.status ?? request.status,
      profile: response.item?.profile ?? null,
      settings: parseJsonObject(response.item?.settingsJson),
      secret_ref: response.item?.secretRef ?? null,
      connected_at: response.item?.connectedAt ?? null,
      last_seen_at: response.item?.lastSeenAt ?? null,
      last_signal_at: response.item?.lastSignalAt ?? null,
      last_sync_at: response.item?.lastSyncAt ?? null,
      created_at: response.item?.createdAt ?? '',
      updated_at: response.item?.updatedAt ?? ''
    }
  }
}

export async function updateSignalHubConnection(
  connectionId: string,
  request: SignalHubConnectionUpdateRequest
): Promise<SignalHubConnectionResponse> {
  const response = await getSignalHubConnectClient().updateConnection({
    id: connectionId,
    displayName: request.display_name,
    status: request.status,
    profile: request.profile ?? undefined,
    secretRef: request.secret_ref ?? undefined,
    settingsJson: request.settings ? JSON.stringify(request.settings) : undefined
  })
  return {
    item: {
      id: response.item?.id ?? connectionId,
      source_code: response.item?.sourceCode ?? '',
      display_name: response.item?.displayName ?? '',
      status: response.item?.status ?? request.status ?? '',
      profile: response.item?.profile ?? null,
      settings: parseJsonObject(response.item?.settingsJson),
      secret_ref: response.item?.secretRef ?? null,
      connected_at: response.item?.connectedAt ?? null,
      last_seen_at: response.item?.lastSeenAt ?? null,
      last_signal_at: response.item?.lastSignalAt ?? null,
      last_sync_at: response.item?.lastSyncAt ?? null,
      created_at: response.item?.createdAt ?? '',
      updated_at: response.item?.updatedAt ?? ''
    }
  }
}

export async function removeSignalHubConnection(
  connectionId: string
): Promise<SignalHubConnectionResponse> {
  const response = await getSignalHubConnectClient().removeConnection({
    id: connectionId
  })
  return {
    item: {
      id: response.item?.id ?? connectionId,
      source_code: response.item?.sourceCode ?? '',
      display_name: response.item?.displayName ?? '',
      status: response.item?.status ?? 'removed',
      profile: response.item?.profile ?? null,
      settings: parseJsonObject(response.item?.settingsJson),
      secret_ref: response.item?.secretRef ?? null,
      connected_at: response.item?.connectedAt ?? null,
      last_seen_at: response.item?.lastSeenAt ?? null,
      last_signal_at: response.item?.lastSignalAt ?? null,
      last_sync_at: response.item?.lastSyncAt ?? null,
      created_at: response.item?.createdAt ?? '',
      updated_at: response.item?.updatedAt ?? ''
    }
  }
}

export async function fetchSignalHubHealth(): Promise<SignalHubHealthResponse> {
  const response = await getSignalHubConnectClient().listHealth({})
  return {
    items: response.items.map((item) => ({
      id: item.id,
      source_code: item.sourceCode,
      connection_id: item.connectionId ?? null,
      level: item.level,
      summary: item.summary,
      last_ok_at: item.lastOkAt ?? null,
      last_failure_at: item.lastFailureAt ?? null,
      failure_count: item.failureCount,
      consecutive_failure_count: item.consecutiveFailureCount,
      next_retry_at: item.nextRetryAt ?? null,
      evidence: parseJsonObject(item.evidenceJson),
      updated_at: item.updatedAt
    }))
  }
}

export async function runSignalHubHealthCheck(
  request: SignalHubHealthCheckRequest
): Promise<SignalHubHealthResponse['items'][number]> {
  const response = await getSignalHubConnectClient().runHealthCheck({
    sourceCode: request.source_code,
    connectionId: request.connection_id ?? undefined
  })
  return {
    id: response.item?.id ?? '',
    source_code: response.item?.sourceCode ?? request.source_code,
    connection_id: response.item?.connectionId ?? null,
    level: response.item?.level ?? 'unknown',
    summary: response.item?.summary ?? '',
    last_ok_at: response.item?.lastOkAt ?? null,
    last_failure_at: response.item?.lastFailureAt ?? null,
    failure_count: response.item?.failureCount ?? 0,
    consecutive_failure_count: response.item?.consecutiveFailureCount ?? 0,
    next_retry_at: response.item?.nextRetryAt ?? null,
    evidence: parseJsonObject(response.item?.evidenceJson),
    updated_at: response.item?.updatedAt ?? ''
  }
}

export async function fetchSignalHubRuntimeStates(): Promise<SignalHubRuntimeStatesResponse> {
  const response = await getSignalHubConnectClient().listRuntimeStates({})
  return {
    items: response.items.map((item) => ({
      id: item.id,
      source_code: item.sourceCode,
      connection_id: item.connectionId ?? null,
      runtime_kind: item.runtimeKind,
      state: item.state,
      last_started_at: item.lastStartedAt ?? null,
      last_stopped_at: item.lastStoppedAt ?? null,
      last_heartbeat_at: item.lastHeartbeatAt ?? null,
      last_error_at: item.lastErrorAt ?? null,
      last_error_code: item.lastErrorCode ?? null,
      last_error_message_redacted: item.lastErrorMessageRedacted ?? null,
      metadata: parseJsonObject(item.metadataJson),
      updated_at: item.updatedAt
    }))
  }
}

export async function updateSignalHubRuntimeState(
  request: SignalHubRuntimeStateRequest
): Promise<SignalHubRuntimeState> {
  const response = await getSignalHubConnectClient().updateRuntimeState({
    sourceCode: request.source_code,
    runtimeKind: request.runtime_kind,
    state: request.state,
    metadataJson: JSON.stringify(request.metadata ?? {})
  })
  return {
    id: response.item?.id ?? '',
    source_code: response.item?.sourceCode ?? request.source_code,
    connection_id: response.item?.connectionId ?? null,
    runtime_kind: response.item?.runtimeKind ?? request.runtime_kind,
    state: response.item?.state ?? request.state,
    last_started_at: response.item?.lastStartedAt ?? null,
    last_stopped_at: response.item?.lastStoppedAt ?? null,
    last_heartbeat_at: response.item?.lastHeartbeatAt ?? null,
    last_error_at: response.item?.lastErrorAt ?? null,
    last_error_code: response.item?.lastErrorCode ?? null,
    last_error_message_redacted: response.item?.lastErrorMessageRedacted ?? null,
    metadata: parseJsonObject(response.item?.metadataJson ?? '{}'),
    updated_at: response.item?.updatedAt ?? ''
  }
}

export async function fetchSignalHubReplayRequests(): Promise<SignalHubReplayRequestsResponse> {
  const response = await getSignalHubConnectClient().listReplayRequests({})
  return {
    items: response.items.map((item) => ({
      id: item.id,
      source_code: item.sourceCode ?? null,
      connection_id: item.connectionId ?? null,
      event_pattern: item.eventPattern ?? null,
      from_position: item.fromPosition ?? null,
      to_position: item.toPosition ?? null,
      from_time: item.fromTime ?? null,
      to_time: item.toTime ?? null,
      target_consumer: item.targetConsumer ?? null,
      target_projection: item.targetProjection ?? null,
      status: item.status,
      requested_by: item.requestedBy,
      requested_at: item.requestedAt,
      started_at: item.startedAt ?? null,
      completed_at: item.completedAt ?? null,
      last_error_redacted: item.lastErrorRedacted ?? null,
      replayed_count: item.replayedCount,
      metadata: parseJsonObject(item.metadataJson)
    }))
  }
}

export async function createSignalHubReplayRequest(
  request: SignalHubReplayRequestCreateRequest
): Promise<{ item: SignalHubReplayRequestsResponse['items'][number] }> {
  const response = await getSignalHubConnectClient().requestReplay({
    sourceCode: request.source_code ?? undefined,
    connectionId: request.connection_id ?? undefined,
    eventPattern: request.event_pattern ?? undefined,
    fromPosition: normalizeOptionalInt64(request.from_position),
    toPosition: normalizeOptionalInt64(request.to_position),
    fromTime: request.from_time ?? undefined,
    toTime: request.to_time ?? undefined,
    targetConsumer: request.target_consumer ?? undefined,
    targetProjection: request.target_projection ?? undefined,
    metadataJson: JSON.stringify(request.metadata ?? {})
  })
  return {
    item: {
      id: response.item?.id ?? '',
      source_code: response.item?.sourceCode ?? null,
      connection_id: response.item?.connectionId ?? null,
      event_pattern: response.item?.eventPattern ?? null,
      from_position: response.item?.fromPosition ?? null,
      to_position: response.item?.toPosition ?? null,
      from_time: response.item?.fromTime ?? null,
      to_time: response.item?.toTime ?? null,
      target_consumer: response.item?.targetConsumer ?? null,
      target_projection: response.item?.targetProjection ?? null,
      status: response.item?.status ?? 'queued',
      requested_by: response.item?.requestedBy ?? 'hermes-frontend',
      requested_at: response.item?.requestedAt ?? '',
      started_at: response.item?.startedAt ?? null,
      completed_at: response.item?.completedAt ?? null,
      last_error_redacted: response.item?.lastErrorRedacted ?? null,
      replayed_count: response.item?.replayedCount ?? 0,
      metadata: parseJsonObject(response.item?.metadataJson ?? '{}')
    }
  }
}

function normalizeOptionalInt64(value: bigint | number | string | null | undefined): bigint | undefined {
  if (value === null || value === undefined || value === '') {
    return undefined
  }

  return typeof value === 'bigint' ? value : BigInt(value)
}

export async function fetchSignalHubPolicies(): Promise<SignalHubPoliciesResponse> {
  const response = await getSignalHubConnectClient().listPolicies({})
  return {
    items: response.items.map((item) => ({
      scope: item.scope as SignalHubPoliciesResponse['items'][number]['scope'],
      source_code: item.sourceCode ?? null,
      connection_id: item.connectionId ?? null,
      event_pattern: item.eventPattern ?? null,
      mode: item.mode as SignalHubPoliciesResponse['items'][number]['mode'],
      reason: item.reason,
      expires_at: item.expiresAt ?? null
    }))
  }
}

export async function createSignalHubPolicy(
  request: SignalHubPolicyRequest
): Promise<SignalHubCreatePolicyResponse> {
  const response = await getSignalHubConnectClient().createPolicy({
    scope: request.scope,
    sourceCode: request.source_code ?? undefined,
    connectionId: request.connection_id ?? undefined,
    eventPattern: request.event_pattern ?? undefined,
    mode: request.mode,
    reason: request.reason,
    expiresAt: request.expires_at ?? undefined
  })
  return {
    id: response.id
  }
}

export async function enableSignalHubSource(sourceCode: string): Promise<SignalHubControlResponse> {
  const response = await getSignalHubConnectClient().enableSource({
    sourceCode
  })
  return {
    source_code: response.sourceCode || sourceCode,
    connection_id: null,
    event_pattern: null,
    policy_id: null,
    cleared_count: response.clearedCount
  }
}

export async function disableSignalHubSource(sourceCode: string): Promise<SignalHubControlResponse> {
  const response = await getSignalHubConnectClient().disableSource({
    sourceCode
  })
  return {
    source_code: response.sourceCode || sourceCode,
    connection_id: null,
    event_pattern: null,
    policy_id: response.policyId || null,
    cleared_count: 0
  }
}

export async function disableSignalHubSignals(
  request: SignalHubControlRequest
): Promise<SignalHubControlResponse> {
  const response = await getSignalHubConnectClient().disableSignals(controlRequestBody(request))
  return {
    source_code: request.source_code ?? null,
    connection_id: request.connection_id ?? null,
    event_pattern: request.event_pattern ?? null,
    policy_id: response.policyId ?? null,
    cleared_count: 0
  }
}

export async function enableSignalHubSignals(
  request: SignalHubControlRequest
): Promise<SignalHubControlResponse> {
  const response = await getSignalHubConnectClient().enableSignals(controlRequestBody(request))
  return {
    source_code: request.source_code ?? null,
    connection_id: request.connection_id ?? null,
    event_pattern: request.event_pattern ?? null,
    policy_id: null,
    cleared_count: response.clearedCount
  }
}

function controlRequestBody(request: SignalHubControlRequest) {
  return {
    scope: request.scope,
    sourceCode: request.source_code ?? undefined,
    connectionId: request.connection_id ?? undefined,
    eventPattern: request.event_pattern ?? undefined,
    reason: request.reason ?? undefined
  }
}

export async function muteSignalHubSignals(
  request: SignalHubControlRequest
): Promise<SignalHubControlResponse> {
  const response = await getSignalHubConnectClient().muteSignals(controlRequestBody(request))
  return {
    source_code: request.source_code ?? null,
    connection_id: request.connection_id ?? null,
    event_pattern: request.event_pattern ?? null,
    policy_id: response.policyId ?? null,
    cleared_count: 0
  }
}

export async function unmuteSignalHubSignals(
  request: SignalHubControlRequest
): Promise<SignalHubControlResponse> {
  const response = await getSignalHubConnectClient().unmuteSignals(controlRequestBody(request))
  return {
    source_code: request.source_code ?? null,
    connection_id: request.connection_id ?? null,
    event_pattern: request.event_pattern ?? null,
    policy_id: null,
    cleared_count: response.clearedCount
  }
}

export async function pauseSignalHubSignals(
  request: SignalHubControlRequest
): Promise<SignalHubControlResponse> {
  const response = await getSignalHubConnectClient().pauseSignals(controlRequestBody(request))
  return {
    source_code: request.source_code ?? null,
    connection_id: request.connection_id ?? null,
    event_pattern: request.event_pattern ?? null,
    policy_id: response.policyId ?? null,
    cleared_count: 0
  }
}

export async function resumeSignalHubSignals(
  request: SignalHubControlRequest
): Promise<SignalHubControlResponse> {
  const response = await getSignalHubConnectClient().resumeSignals(controlRequestBody(request))
  return {
    source_code: request.source_code ?? null,
    connection_id: request.connection_id ?? null,
    event_pattern: request.event_pattern ?? null,
    policy_id: null,
    cleared_count: response.clearedCount
  }
}

function parseJsonObject(value: string | undefined): Record<string, unknown> {
  if (!value) return {}
  try {
    const parsed = JSON.parse(value)
    return parsed && typeof parsed === 'object' && !Array.isArray(parsed)
      ? (parsed as Record<string, unknown>)
      : {}
  } catch {
    return {}
  }
}

function mapProfile(
  item: { [key: string]: any } | undefined,
  fallbackCode: string
): SignalHubProfile {
  return {
    id: item?.id ?? '',
    code: item?.code ?? fallbackCode,
    display_name: item?.displayName ?? fallbackCode,
    description: item?.description ?? '',
    policy_count: item?.policyCount ?? 0,
    source_policies: Array.isArray(item?.sourcePolicies)
      ? item.sourcePolicies.map(mapProfilePolicy)
      : [],
    is_system: item?.isSystem ?? false,
    is_active: item?.isActive ?? false,
    created_at: item?.createdAt ?? '',
    updated_at: item?.updatedAt ?? ''
  }
}

function mapCapability(item: { [key: string]: any }): SignalHubCapability {
  return {
    id: item.id,
    source_code: item.sourceCode,
    connection_id: item.connectionId ?? null,
    capability: item.capability,
    state: item.state,
    reason: item.reason ?? null,
    requires_confirmation: item.requiresConfirmation ?? false,
    action_class: item.actionClass ?? 'read',
    updated_at: item.updatedAt ?? ''
  }
}

function mapProfilePolicy(item: { [key: string]: any }): SignalHubProfilePolicy {
  return {
    scope: item.scope,
    source_code: item.sourceCode ?? null,
    connection_id: item.connectionId ?? null,
    event_pattern: item.eventPattern ?? null,
    mode: item.mode,
    reason: item.reason
  }
}

function protoProfilePolicy(policy: SignalHubProfilePolicy) {
  return {
    scope: policy.scope,
    sourceCode: policy.source_code ?? undefined,
    connectionId: policy.connection_id ?? undefined,
    eventPattern: policy.event_pattern ?? undefined,
    mode: policy.mode,
    reason: policy.reason
  }
}
