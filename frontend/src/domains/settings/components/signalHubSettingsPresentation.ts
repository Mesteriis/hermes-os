import type {
  SignalHubConnection,
  SignalHubCapability,
  SignalHubHealth,
  SignalHubPolicy,
  SignalHubPolicyMode,
  SignalHubPolicyScope,
  SignalHubProfilePolicy,
  SignalHubReplayRequest,
  SignalHubRuntimeState,
  SignalHubSource
} from '../types/signalHub'

type Translator = (value: string) => string

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

export interface SignalInventoryRow extends SignalConsumerGraphRoute {
  active_policies: SignalHubPolicy[]
  capabilities: SignalHubCapability[]
  connection_count: number
  health: SignalHubHealth | null
  runtime_states: SignalHubRuntimeState[]
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
  'person_derived_evidence',
  'project_link_review_effects'
] as const

const DOCUMENTED_SIGNAL_CONSUMERS = new Set<string>([
  SIGNAL_HUB_RAW_SIGNAL_CONSUMER,
  COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER
])

const DOCUMENTED_REPLAY_PROJECTIONS = new Set<string>(SIGNAL_HUB_REPLAY_PROJECTION_TARGETS)

export function capabilityLabels(source: SignalHubSource): string[] {
  const labels: string[] = []
  if (source.supports_connections) labels.push('Connections')
  if (source.supports_runtime) labels.push('Runtime')
  if (source.supports_replay) labels.push('Replay')
  if (source.supports_pause) labels.push('Pause')
  if (source.supports_mute) labels.push('Mute')
  return labels
}

export function capabilityTone(state: string): string {
  if (state === 'available') return 'good'
  if (state === 'degraded') return 'warn'
  if (state === 'blocked' || state === 'unsupported') return 'bad'
  return 'neutral'
}

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

  if (isCommunicationSignalSource(source)) {
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

export function sourceIcon(source: SignalHubSource): string {
  const icons: Record<string, string> = {
    browser: 'tabler:browser',
    calendar: 'tabler:calendar',
    filesystem: 'tabler:folder',
    github: 'tabler:brand-github',
    home_assistant: 'tabler:home-cog',
    ai: 'tabler:sparkles',
    mail: 'tabler:mail',
    rss: 'tabler:rss',
    system: 'tabler:settings-cog',
    telegram: 'tabler:brand-telegram',
    voice: 'tabler:microphone',
    whatsapp: 'tabler:brand-whatsapp'
  }
  return icons[source.code] ?? 'tabler:plug'
}

export function sourceIconForCode(sources: SignalHubSource[], sourceCode: string): string {
  const source = sources.find((item) => item.code === sourceCode)
  return source ? sourceIcon(source) : 'tabler:plug'
}

export function statusTone(status: string): string {
  if (status === 'connected' || status === 'ready') return 'good'
  if (status === 'paused' || status === 'degraded') return 'warn'
  if (status === 'error' || status === 'disconnected') return 'bad'
  return 'neutral'
}

export function sourceStateTone(state: string): string {
  if (state === 'running') return 'good'
  if (state === 'paused' || state === 'muted') return 'warn'
  if (state === 'disabled' || state === 'off') return 'bad'
  return 'neutral'
}

export function healthTone(level: string): string {
  if (level === 'healthy') return 'good'
  if (level === 'degraded' || level === 'warning') return 'warn'
  if (level === 'blocked' || level === 'error' || level === 'failed') return 'bad'
  return 'neutral'
}

export function runtimeTone(state: string): string {
  if (state === 'running') return 'good'
  if (state === 'paused' || state === 'muted' || state === 'stopping') return 'warn'
  if (state === 'error' || state === 'stopped') return 'bad'
  return 'neutral'
}

export function connectionLabel(
  t: Translator,
  connections: SignalHubConnection[],
  connectionId: string | null | undefined
): string {
  if (!connectionId) return t('No connection')
  const connection = connections.find((item) => item.id === connectionId)
  if (!connection) return connectionId
  return `${connection.display_name} (${connection.source_code})`
}

function summarizeObjectEntries(
  t: Translator,
  value: Record<string, unknown> | null | undefined,
  maxEntries = 3
): string[] {
  if (!value) return []
  const entries = Object.entries(value).filter(([, entryValue]) => entryValue != null)
  return entries.slice(0, maxEntries).map(([key, entryValue]) => `${key}: ${formatSummaryValue(t, entryValue)}`)
}

function formatSummaryValue(t: Translator, value: unknown): string {
  if (typeof value === 'string') return value
  if (typeof value === 'number' || typeof value === 'boolean') return String(value)
  if (Array.isArray(value)) return `${value.length} ${t('items')}`
  if (value && typeof value === 'object') return t('structured')
  return t('unknown')
}

export function formatSettingsSummary(t: Translator, connection: SignalHubConnection): string {
  const entries = summarizeObjectEntries(t, connection.settings)
  if (entries.length === 0) return t('No non-secret settings')
  const totalEntries = Object.keys(connection.settings).length
  const suffix = totalEntries > entries.length ? ` +${totalEntries - entries.length} ${t('more')}` : ''
  return `${entries.join(' • ')}${suffix}`
}

export function formatConnectionTimeline(t: Translator, connection: SignalHubConnection): string {
  const checkpoints = [
    connection.connected_at ? `${t('Connected')} ${connection.connected_at}` : null,
    connection.last_seen_at ? `${t('Seen')} ${connection.last_seen_at}` : null,
    connection.last_signal_at ? `${t('Signal')} ${connection.last_signal_at}` : null,
    connection.last_sync_at ? `${t('Sync')} ${connection.last_sync_at}` : null
  ].filter((value): value is string => Boolean(value))
  return checkpoints.length > 0 ? checkpoints.join(' • ') : t('No activity recorded')
}

export function formatRuntimeTimeline(t: Translator, runtime: SignalHubRuntimeState): string {
  const checkpoints = [
    runtime.last_started_at ? `${t('Started')} ${runtime.last_started_at}` : null,
    runtime.last_stopped_at ? `${t('Stopped')} ${runtime.last_stopped_at}` : null,
    runtime.last_heartbeat_at ? `${t('Heartbeat')} ${runtime.last_heartbeat_at}` : null
  ].filter((value): value is string => Boolean(value))
  return checkpoints.length > 0 ? checkpoints.join(' • ') : t('No runtime telemetry yet')
}

export function formatRuntimeError(t: Translator, runtime: SignalHubRuntimeState): string | null {
  if (!runtime.last_error_at && !runtime.last_error_code && !runtime.last_error_message_redacted) {
    return null
  }
  return [runtime.last_error_code ?? t('error'), runtime.last_error_message_redacted, runtime.last_error_at]
    .filter((value): value is string => Boolean(value))
    .join(' • ')
}

export function formatHealthStatus(
  t: Translator,
  connections: SignalHubConnection[],
  item: SignalHubHealth
): string {
  const fragments = [
    item.connection_id ? connectionLabel(t, connections, item.connection_id) : null,
    item.last_ok_at ? `${t('Last OK')} ${item.last_ok_at}` : null,
    item.last_failure_at ? `${t('Last failure')} ${item.last_failure_at}` : null,
    item.failure_count > 0 ? `${t('Failures')} ${item.failure_count}` : null,
    item.consecutive_failure_count > 0 ? `${t('Consecutive')} ${item.consecutive_failure_count}` : null
  ].filter((value): value is string => Boolean(value))
  return fragments.length > 0 ? fragments.join(' • ') : t('No health history')
}

export function formatHealthEvidence(t: Translator, item: SignalHubHealth): string | null {
  const entries = summarizeObjectEntries(t, item.evidence)
  if (entries.length === 0) return null
  const totalEntries = Object.keys(item.evidence).length
  const suffix = totalEntries > entries.length ? ` +${totalEntries - entries.length} ${t('more')}` : ''
  return `${entries.join(' • ')}${suffix}`
}

export function policyTargetLabel(
  t: Translator,
  connections: SignalHubConnection[],
  policy: {
    scope: SignalHubPolicyScope
    source_code: string | null
    connection_id: string | null
    event_pattern: string | null
  }
): string {
  if (policy.scope === 'connection' && policy.connection_id) {
    return connectionLabel(t, connections, policy.connection_id)
  }
  return policy.event_pattern ?? policy.source_code ?? policy.scope
}

export function profilePolicyLabel(
  t: Translator,
  connections: SignalHubConnection[],
  policy: SignalHubProfilePolicy
): string {
  if (policy.scope === 'connection' && policy.connection_id) {
    return `${policy.mode} / ${connectionLabel(t, connections, policy.connection_id)}`
  }
  return `${policy.mode} / ${policy.event_pattern ?? policy.source_code ?? policy.scope}`
}

export function capabilityLabel(capability: SignalHubCapability): string {
  return `${capability.capability} / ${capability.action_class}`
}

function isCommunicationSignalSource(source: SignalHubSource): boolean {
  return source.category === 'communications'
}

function routeTarget(
  id: string,
  kind: SignalRouteTargetKind,
  evidence: SignalRouteTargetEvidence
): SignalRouteTarget {
  return {
    id,
    label: signalTargetLabel(id),
    kind,
    evidence
  }
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
