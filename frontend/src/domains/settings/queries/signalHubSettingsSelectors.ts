import type {
  SignalHubCapability,
  SignalHubConnection,
  SignalHubHealth,
  SignalHubPolicy,
  SignalHubProfile,
  SignalHubReplayRequest,
  SignalHubRuntimeState,
  SignalHubSource
} from '../types/signalHub'
import {
  buildSignalConsumerGraphRoute,
  buildSignalGraphTabs,
  buildSignalInventoryRow,
  buildSignalInventoryTabs,
  sourceControlState,
  SIGNAL_HUB_REPLAY_PROJECTION_TARGETS,
  type SignalConsumerGraphRoute,
  type SignalInventoryRow,
  type SignalSourceTab
} from '../components/signalHubRoutePresentation'

export type SignalHubTab =
  | 'sources'
  | 'profiles'
  | 'connections'
  | 'runtime'
  | 'policies'
  | 'health'
  | 'replay'

export const SIGNAL_HUB_TABS: Array<{ id: SignalHubTab; label: string; icon: string }> = [
  { id: 'sources', label: 'Sources', icon: 'tabler:database-import' },
  { id: 'profiles', label: 'Profiles', icon: 'tabler:layout-dashboard' },
  { id: 'connections', label: 'Connections', icon: 'tabler:plug-connected' },
  { id: 'runtime', label: 'Runtime', icon: 'tabler:player-play' },
  { id: 'policies', label: 'Policies', icon: 'tabler:shield-cog' },
  { id: 'health', label: 'Health', icon: 'tabler:activity-heartbeat' },
  { id: 'replay', label: 'Replay', icon: 'tabler:player-track-next' }
]

export function visibleSignalSources(sources: SignalHubSource[]): SignalHubSource[] {
  return sources.filter((source) => source.category !== 'test')
}

export function replayTargetConsumers(runtimeStates: SignalHubRuntimeState[]): string[] {
  return uniqueSorted(runtimeStates.map((runtime) => runtime.runtime_kind.trim()))
}

export function replayTargetProjections(replayRequests: SignalHubReplayRequest[]): string[] {
  return uniqueSorted([
    ...SIGNAL_HUB_REPLAY_PROJECTION_TARGETS,
    ...replayRequests.map((request) => request.target_projection?.trim() ?? '')
  ])
}

export function signalCategories(sources: SignalHubSource[]): string[] {
  return ['all', ...uniqueSorted(sources.map((source) => source.category))]
}

export function filterSignalSources(
  sources: SignalHubSource[],
  searchValue: string,
  category: string
): SignalHubSource[] {
  const search = searchValue.trim().toLowerCase()
  return sources.filter((source) => {
    const matchesCategory = category === 'all' || source.category === category
    const matchesSearch =
      search.length === 0 ||
      source.code.toLowerCase().includes(search) ||
      source.display_name.toLowerCase().includes(search)
    return matchesCategory && matchesSearch
  })
}

export function filterConnectionsForScope(
  connections: SignalHubConnection[],
  scope: string,
  sourceCode: string
): SignalHubConnection[] {
  return connections.filter((connection) =>
    scope === 'connection' && sourceCode.trim().length > 0
      ? connection.source_code === sourceCode
      : true
  )
}

export function filterConnectionsForSource(
  connections: SignalHubConnection[],
  sourceCode: string
): SignalHubConnection[] {
  return connections.filter((connection) =>
    sourceCode.trim().length > 0 ? connection.source_code === sourceCode : true
  )
}

export function findSelectedSource(
  sources: SignalHubSource[],
  filteredSources: SignalHubSource[],
  selectedCode: string | null
): SignalHubSource | null {
  if (!selectedCode) return filteredSources[0] ?? null
  return sources.find((source) => source.code === selectedCode) ?? null
}

export function findSelectedProfile(
  profiles: SignalHubProfile[],
  selectedCode: string | null
): SignalHubProfile | null {
  if (!selectedCode) return null
  return profiles.find((profile) => profile.code === selectedCode) ?? null
}

export function sourceCapabilities(
  capabilities: SignalHubCapability[],
  selectedSource: SignalHubSource | null
): SignalHubCapability[] {
  if (!selectedSource) return []
  return capabilities.filter(
    (capability) => capability.source_code === selectedSource.code && capability.connection_id === null
  )
}

export function countRunningSources(sources: SignalHubSource[], policies: SignalHubPolicy[]): number {
  return sources.filter((source) => sourceControlState(policies, source) === 'running').length
}

export function countRuntimeSources(sources: SignalHubSource[]): number {
  return sources.filter((source) => source.supports_runtime).length
}

export function countActiveRuntimeStates(runtimeStates: SignalHubRuntimeState[]): number {
  return runtimeStates.filter((runtime) => runtime.state === 'running').length
}

export function countReplaySources(sources: SignalHubSource[]): number {
  return sources.filter((source) => source.supports_replay).length
}

export function countConnectedConnections(connections: SignalHubConnection[]): number {
  return connections.filter((connection) => connection.status === 'connected').length
}

export function activeProfile(profiles: SignalHubProfile[]): SignalHubProfile | null {
  return profiles.find((profile) => profile.is_active) ?? null
}

export function countUnhealthyHealthItems(healthItems: SignalHubHealth[]): number {
  return healthItems.filter((item) => item.level !== 'healthy').length
}

export function countPendingReplayRequests(replayRequests: SignalHubReplayRequest[]): number {
  return replayRequests.filter(
    (request) => request.status !== 'completed' && request.status !== 'failed'
  ).length
}

export function buildSignalConsumerGraph(
  sources: SignalHubSource[],
  policies: SignalHubPolicy[],
  runtimeStates: SignalHubRuntimeState[],
  replayRequests: SignalHubReplayRequest[]
): SignalConsumerGraphRoute[] {
  return sources.map((source) =>
    buildSignalConsumerGraphRoute(source, policies, runtimeStates, replayRequests)
  )
}

export function filterSignalConsumerGraph(
  routes: SignalConsumerGraphRoute[],
  selectedSourceCode: string
): SignalConsumerGraphRoute[] {
  if (selectedSourceCode === 'all' || !routes.some((route) => route.source.code === selectedSourceCode)) {
    return routes
  }
  return routes.filter((route) => route.source.code === selectedSourceCode)
}

export function buildSignalInventory(
  sources: SignalHubSource[],
  policies: SignalHubPolicy[],
  connections: SignalHubConnection[],
  runtimeStates: SignalHubRuntimeState[],
  healthItems: SignalHubHealth[],
  capabilities: SignalHubCapability[],
  replayRequests: SignalHubReplayRequest[]
): SignalInventoryRow[] {
  return sources.map((source) =>
    buildSignalInventoryRow(
      source,
      policies,
      connections,
      runtimeStates,
      healthItems,
      capabilities,
      replayRequests
    )
  )
}

export function filterSignalInventory(
  rows: SignalInventoryRow[],
  selectedSourceCode: string
): SignalInventoryRow[] {
  if (selectedSourceCode === 'all' || !rows.some((row) => row.source.code === selectedSourceCode)) {
    return rows
  }
  return rows.filter((row) => row.source.code === selectedSourceCode)
}

export function signalViewTabs(
  graph: SignalConsumerGraphRoute[],
  inventory: SignalInventoryRow[]
): Array<{ id: 'graph' | 'inventory'; label: string; count: number }> {
  return [
    { id: 'graph', label: 'Graph', count: graph.length },
    { id: 'inventory', label: 'Inventory', count: inventory.length }
  ]
}

export function graphTabs(routes: SignalConsumerGraphRoute[]): SignalSourceTab[] {
  return buildSignalGraphTabs(routes)
}

export function inventoryTabs(rows: SignalInventoryRow[]): SignalSourceTab[] {
  return buildSignalInventoryTabs(rows)
}

function uniqueSorted(values: string[]): string[] {
  return Array.from(new Set(values.filter((value) => value.length > 0))).sort()
}
