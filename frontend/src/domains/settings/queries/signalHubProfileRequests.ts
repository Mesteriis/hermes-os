import type {
  SignalHubProfileCreateRequest,
  SignalHubProfilePolicy,
  SignalHubPolicyMode,
  SignalHubPolicyScope
} from '../types/signalHub'

export interface SignalHubProfilePolicyDraft {
  scope: SignalHubPolicyScope
  sourceCode: string
  connectionId: string
  eventPattern: string
  mode: SignalHubPolicyMode
  reason: string
}

export function buildSignalHubProfilePolicy(
  draft: SignalHubProfilePolicyDraft
): SignalHubProfilePolicy {
  return {
    scope: draft.scope,
    source_code: draft.scope === 'source' || draft.scope === 'connection' ? draft.sourceCode : null,
    connection_id: draft.scope === 'connection' ? draft.connectionId || null : null,
    event_pattern: draft.scope === 'event_pattern' ? draft.eventPattern : null,
    mode: draft.mode,
    reason: draft.reason
  }
}

export function buildSignalHubProfileSaveRequest(
  displayName: string,
  description: string,
  sourcePolicies: SignalHubProfilePolicy[]
): Pick<SignalHubProfileCreateRequest, 'display_name' | 'description' | 'source_policies'> {
  return {
    display_name: displayName,
    description,
    source_policies: sourcePolicies
  }
}
