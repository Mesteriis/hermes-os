import type {
  SignalHubConnection,
  SignalHubCapability,
  SignalHubHealth,
  SignalHubPolicyMode,
  SignalHubPolicyScope,
  SignalHubProfilePolicy,
  SignalHubRuntimeState,
  SignalHubSource
} from '../types/signalHub'

type Translator = (value: string) => string

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

export function sourceIcon(source: SignalHubSource): string {
  const icons: Record<string, string> = {
    browser: 'tabler:browser',
    calendar: 'tabler:calendar',
    filesystem: 'tabler:folder',
    fixture: 'tabler:test-pipe',
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
