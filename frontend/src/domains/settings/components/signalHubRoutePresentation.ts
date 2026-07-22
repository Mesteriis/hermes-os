import type {
  SignalHubCapability,
  SignalHubConnection,
  SignalHubHealth,
  SignalHubPolicy,
  SignalHubPolicyMode,
  SignalHubPolicyScope,
  SignalHubReplayRequest,
  SignalHubRuntimeState,
  SignalHubSource
} from '../types/signalHub'

export type SignalRouteTargetKind = 'consumer' | 'projection'
export type SignalRouteTargetEvidence = 'documented' | 'runtime' | 'replay'

export interface SignalRouteTarget {
  id: string
  label: string
  kind: SignalRouteTargetKind
  evidence: SignalRouteTargetEvidence
}

export interface SignalConsumerGraphRoute {
  source: SignalHubSource
  state: ReturnType<typeof sourceControlState>
  raw_pattern: string
  accepted_pattern: string
  targets: SignalRouteTarget[]
}

export function signalTargetIcon(kind: SignalRouteTargetKind): string {
  return kind === 'projection' ? 'tabler:chart-dots' : 'tabler:route'
}

export interface SignalInventoryRow extends SignalConsumerGraphRoute {
  active_policies: SignalHubPolicy[]
  capabilities: SignalHubCapability[]
  connection_count: number
  health: SignalHubHealth | null
  runtime_states: SignalHubRuntimeState[]
}

export interface SignalControlAvailability {
  pauseDisabled: boolean
  resumeDisabled: boolean
  muteDisabled: boolean
  unmuteDisabled: boolean
  disableDisabled: boolean
  enableDisabled: boolean
}

export function signalControlAvailability(
  row: Pick<SignalInventoryRow, 'state'> & {
    source: Pick<SignalHubSource, 'supports_pause' | 'supports_mute'>
  },
  isBusy: boolean
): SignalControlAvailability {
  return {
    pauseDisabled: !row.source.supports_pause || row.state === 'paused' || isBusy,
    resumeDisabled: row.state !== 'paused' || isBusy,
    muteDisabled: !row.source.supports_mute || row.state === 'muted' || isBusy,
    unmuteDisabled: row.state !== 'muted' || isBusy,
    disableDisabled: row.state === 'disabled' || isBusy,
    enableDisabled: (row.state !== 'disabled' && row.state !== 'off') || isBusy,
  }
}

export interface SignalSourceTab {
  id: string
  label: string
  count: number
}

export const SIGNAL_HUB_RAW_SIGNAL_CONSUMER = 'signal_hub_raw_signal_dispatcher'
export const COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER =
  'communication_provider_observation_projection'
export const SIGNAL_HUB_REPLAY_PROJECTION_TARGETS = [
  'communication_messages',
  'timeline_event_log',
  'persona_derived_evidence',
  'project_link_review_effects'
] as const

const DOCUMENTED_SIGNAL_CONSUMERS = new Set<string>([
  SIGNAL_HUB_RAW_SIGNAL_CONSUMER,
  COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER
])

const DOCUMENTED_REPLAY_PROJECTIONS = new Set<string>(SIGNAL_HUB_REPLAY_PROJECTION_TARGETS)

export function sourceControlState(
  policies: Array<{ scope: SignalHubPolicyScope; mode: SignalHubPolicyMode; source_code: string | null }>,
  source: SignalHubSource
): 'running' | 'muted' | 'paused' | 'disabled' | 'off' {
  const relevantPolicies = policies.filter(
    (policy) =>
      (policy.scope === 'global' || policy.scope === 'source') &&
      (policy.scope !== 'source' || policy.source_code === source.code)
  )
  if (relevantPolicies.some((policy) => policy.mode === 'disabled')) return 'disabled'
  if (relevantPolicies.some((policy) => policy.mode === 'paused')) return 'paused'
  if (relevantPolicies.some((policy) => policy.mode === 'muted')) return 'muted'
  return source.default_enabled ? 'running' : 'off'
}

export function rawSignalPattern(sourceCode: string): string {
  return `signal.raw.${sourceCode}.*`
}

export function acceptedSignalPattern(sourceCode: string): string {
  return `signal.accepted.${sourceCode}.*`
}

export function signalPolicyAppliesToSource(policy: SignalHubPolicy, source: SignalHubSource): boolean {
  if (policy.scope === 'global') return true
  if (policy.scope === 'source' || policy.scope === 'connection') {
    return policy.source_code === source.code
  }
  if (policy.scope === 'event_pattern' && policy.event_pattern) {
    return eventPatternMatchesSource(policy.event_pattern, source.code)
  }
  return false
}

export function buildSignalConsumerGraphRoute(
  source: SignalHubSource,
  policies: SignalHubPolicy[],
  runtimeStates: SignalHubRuntimeState[],
  replayRequests: SignalHubReplayRequest[]
): SignalConsumerGraphRoute {
  return {
    source,
    state: sourceControlState(policies, source),
    raw_pattern: rawSignalPattern(source.code),
    accepted_pattern: acceptedSignalPattern(source.code),
    targets: signalRouteTargetsForSource(source, runtimeStates, replayRequests)
  }
}

export function buildSignalInventoryRow(
  source: SignalHubSource,
  policies: SignalHubPolicy[],
  connections: SignalHubConnection[],
  runtimeStates: SignalHubRuntimeState[],
  healthItems: SignalHubHealth[],
  capabilities: SignalHubCapability[],
  replayRequests: SignalHubReplayRequest[]
): SignalInventoryRow {
  const sourcePolicies = policies.filter((policy) => signalPolicyAppliesToSource(policy, source))
  const sourceRuntimeStates = runtimeStates.filter((runtime) => runtime.source_code === source.code)
  return {
    ...buildSignalConsumerGraphRoute(source, policies, runtimeStates, replayRequests),
    active_policies: sourcePolicies,
    capabilities: capabilities.filter((capability) => capability.source_code === source.code),
    connection_count: connections.filter((connection) => connection.source_code === source.code).length,
    health: highestPriorityHealth(healthItems.filter((item) => item.source_code === source.code)),
    runtime_states: sourceRuntimeStates
  }
}

export function buildSignalGraphTabs(routes: SignalConsumerGraphRoute[]): SignalSourceTab[] {
  return [
    { id: 'all', label: 'All', count: routes.length },
    ...routes.map((route) => ({
      id: route.source.code,
      label: route.source.display_name,
      count: route.targets.length
    }))
  ]
}

export function buildSignalInventoryTabs(rows: SignalInventoryRow[]): SignalSourceTab[] {
  return [
    { id: 'all', label: 'All', count: rows.length },
    ...rows.map((row) => ({
      id: row.source.code,
      label: row.source.display_name,
      count: 1
    }))
  ]
}

export function signalRouteTargetsForSource(
  source: SignalHubSource,
  runtimeStates: SignalHubRuntimeState[],
  replayRequests: SignalHubReplayRequest[]
): SignalRouteTarget[] {
  const targets: SignalRouteTarget[] = [
    routeTarget(SIGNAL_HUB_RAW_SIGNAL_CONSUMER, 'consumer', 'documented')
  ]

  if (source.category === 'communications') {
    targets.push(routeTarget(COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER, 'consumer', 'documented'))
  }

  for (const runtime of runtimeStates) {
    if (runtime.source_code !== source.code) continue
    const runtimeTarget = routeTargetFromRuntime(runtime)
    if (runtimeTarget) targets.push(runtimeTarget)
  }

  for (const request of replayRequests) {
    if (!replayRequestAppliesToSource(request, source)) continue
    if (request.target_consumer) {
      targets.push(routeTarget(request.target_consumer, 'consumer', 'replay'))
    }
    if (request.target_projection) {
      targets.push(routeTarget(request.target_projection, 'projection', 'replay'))
    }
  }

  return mergeRouteTargets(targets)
}

export function signalTargetTone(target: SignalRouteTarget): 'neutral' | 'good' | 'warn' {
  if (target.evidence === 'runtime') return 'good'
  if (target.evidence === 'replay') return 'warn'
  return 'neutral'
}

export function signalTargetLabel(targetId: string): string {
  return targetId.replaceAll('_', ' ')
}

function routeTarget(
  id: string,
  kind: SignalRouteTargetKind,
  evidence: SignalRouteTargetEvidence
): SignalRouteTarget {
  return { id, label: signalTargetLabel(id), kind, evidence }
}

function routeTargetFromRuntime(runtime: SignalHubRuntimeState): SignalRouteTarget | null {
  const scope = typeof runtime.metadata.scope === 'string' ? runtime.metadata.scope : null
  if (
    scope !== 'consumer' &&
    scope !== 'projection' &&
    !DOCUMENTED_SIGNAL_CONSUMERS.has(runtime.runtime_kind) &&
    !DOCUMENTED_REPLAY_PROJECTIONS.has(runtime.runtime_kind)
  ) {
    return null
  }

  const kind =
    scope === 'projection' || DOCUMENTED_REPLAY_PROJECTIONS.has(runtime.runtime_kind)
      ? 'projection'
      : 'consumer'
  return routeTarget(runtime.runtime_kind, kind, 'runtime')
}

function replayRequestAppliesToSource(request: SignalHubReplayRequest, source: SignalHubSource): boolean {
  if (request.source_code && request.source_code !== source.code) return false
  if (request.source_code === source.code) return true
  if (!request.event_pattern) return !request.source_code
  return eventPatternMatchesSource(request.event_pattern, source.code)
}

function eventPatternMatchesSource(pattern: string, sourceCode: string): boolean {
  const trimmed = pattern.trim()
  return (
    trimmed === 'signal.*' ||
    trimmed === 'signal.raw.*' ||
    trimmed === 'signal.accepted.*' ||
    trimmed.includes(`.${sourceCode}.`) ||
    trimmed.includes(`.${sourceCode}.*`)
  )
}

function mergeRouteTargets(targets: SignalRouteTarget[]): SignalRouteTarget[] {
  const evidenceRank: Record<SignalRouteTargetEvidence, number> = {
    runtime: 3,
    replay: 2,
    documented: 1
  }
  const merged = new Map<string, SignalRouteTarget>()

  for (const target of targets) {
    const key = `${target.kind}:${target.id}`
    const previous = merged.get(key)
    if (!previous || evidenceRank[target.evidence] > evidenceRank[previous.evidence]) {
      merged.set(key, target)
    }
  }

  return Array.from(merged.values()).sort((left, right) => {
    if (left.kind !== right.kind) return left.kind.localeCompare(right.kind)
    return left.label.localeCompare(right.label)
  })
}

function highestPriorityHealth(items: SignalHubHealth[]): SignalHubHealth | null {
  if (items.length === 0) return null
  const rank: Record<string, number> = {
    failed: 5,
    failing: 5,
    error: 5,
    blocked: 5,
    degraded: 4,
    warning: 3,
    disabled: 2,
    unknown: 1,
    healthy: 0
  }
  return [...items].sort((left, right) => {
    const rankDelta = (rank[right.level] ?? 1) - (rank[left.level] ?? 1)
    if (rankDelta !== 0) return rankDelta
    return right.updated_at.localeCompare(left.updated_at)
  })[0]
}
