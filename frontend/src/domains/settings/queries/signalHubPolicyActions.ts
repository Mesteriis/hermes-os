import type {
  SignalHubPolicyMode,
  SignalHubPolicyRequest,
  SignalHubPolicyScope
} from '../types/signalHub'

export interface SignalHubPolicyDraft {
  scope: SignalHubPolicyScope
  source_code: string | null
  connection_id: string | null
  event_pattern: string | null
  reason: string
}

export function buildSignalHubSourcePolicyRequest(
  sourceCode: string,
  reason: string
): SignalHubPolicyDraft {
  return {
    scope: 'source',
    source_code: sourceCode,
    connection_id: null,
    event_pattern: null,
    reason
  }
}

interface CreatePolicyDependencies {
  pause: (request: SignalHubPolicyDraft) => Promise<unknown>
  mute: (request: SignalHubPolicyDraft) => Promise<unknown>
  disableSource: (sourceCode: string) => Promise<unknown>
  disable: (request: SignalHubPolicyDraft) => Promise<unknown>
  create: (request: SignalHubPolicyRequest) => Promise<unknown>
}

export function createSignalHubPolicy(
  draft: SignalHubPolicyDraft,
  mode: SignalHubPolicyMode,
  dependencies: CreatePolicyDependencies
): Promise<unknown> {
  if (mode === 'paused') return dependencies.pause(draft)
  if (mode === 'muted') return dependencies.mute(draft)
  if (mode === 'disabled' && draft.scope === 'source') {
    return dependencies.disableSource(draft.source_code ?? '')
  }
  if (mode === 'disabled') return dependencies.disable(draft)
  return dependencies.create({ ...draft, mode })
}

interface ClearPolicyDependencies {
  resume: (request: SignalHubPolicyDraft) => Promise<unknown>
  unmute: (request: SignalHubPolicyDraft) => Promise<unknown>
  enableSource: (sourceCode: string) => Promise<unknown>
  enable: (request: SignalHubPolicyDraft) => Promise<unknown>
}

export function clearSignalHubPolicy(
  policy: SignalHubPolicyDraft & { mode: SignalHubPolicyMode },
  dependencies: ClearPolicyDependencies
): Promise<unknown> | undefined {
  const request: SignalHubPolicyDraft = {
    scope: policy.scope,
    source_code: policy.source_code,
    connection_id: policy.connection_id,
    event_pattern: policy.event_pattern,
    reason: policy.reason
  }
  if (policy.mode === 'paused') return dependencies.resume(request)
  if (policy.mode === 'muted') return dependencies.unmute(request)
  if (policy.mode === 'disabled' && policy.scope === 'source' && policy.source_code) {
    return dependencies.enableSource(policy.source_code)
  }
  if (policy.mode === 'disabled') return dependencies.enable(request)
  return undefined
}
