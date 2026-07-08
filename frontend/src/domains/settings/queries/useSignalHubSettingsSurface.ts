import { computed, ref } from 'vue'
import {
  useApplySignalHubProfileMutation,
  useCreateSignalHubProfileMutation,
  useCreateSignalHubConnectionMutation,
  useCreateSignalHubPolicyMutation,
  useRemoveSignalHubProfileMutation,
  useCreateSignalHubReplayRequestMutation,
  useDisableSignalHubMutation,
  useDisableSignalHubSourceMutation,
  useEnableSignalHubMutation,
  useEnableSignalHubSourceMutation,
  useSignalHubCapabilitiesQuery,
  useMuteSignalHubMutation,
  usePauseSignalHubMutation,
  useRemoveSignalHubConnectionMutation,
  useResumeSignalHubMutation,
  useRunSignalHubHealthCheckMutation,
  useSignalHubConnectionsQuery,
  useSignalHubHealthQuery,
  useSignalHubProfilesQuery,
  useSignalHubReplayRequestsQuery,
  useSignalHubRuntimeStatesQuery,
  useSignalHubPoliciesQuery,
  useSignalHubSourcesQuery,
  useUnmuteSignalHubMutation,
  useUpdateSignalHubConnectionMutation,
  useUpdateSignalHubProfileMutation,
  useUpdateSignalHubRuntimeStateMutation
} from './useSignalHubQuery'
import {
  buildSignalHubReplayRequest,
  describeSignalHubReplayRequest,
  type SignalHubReplaySelectorMode
} from '../lib/signalHubReplay'
import type {
  SignalHubProfile,
  SignalHubProfilePolicy,
  SignalHubPolicyMode,
  SignalHubPolicyScope,
  SignalHubRuntimeState
} from '../types/signalHub'
import {
  buildSignalConsumerGraphRoute,
  buildSignalGraphTabs,
  buildSignalInventoryTabs,
  buildSignalInventoryRow,
  sourceControlState,
  SIGNAL_HUB_REPLAY_PROJECTION_TARGETS
} from '../components/signalHubSettingsPresentation'

export type SignalHubTab =
  | 'sources'
  | 'profiles'
  | 'connections'
  | 'runtime'
  | 'policies'
  | 'health'
  | 'replay'

export type SignalHubSettingsView = 'graph' | 'inventory'

export function useSignalHubSettingsSurface() {
  const { data: sourcesData, isLoading } = useSignalHubSourcesQuery()
  const { data: capabilitiesData } = useSignalHubCapabilitiesQuery()
  const { data: connectionsData } = useSignalHubConnectionsQuery()
  const { data: profilesData } = useSignalHubProfilesQuery()
  const { data: runtimeData } = useSignalHubRuntimeStatesQuery()
  const { data: healthData } = useSignalHubHealthQuery()
  const { data: replayData } = useSignalHubReplayRequestsQuery()
  const { data: policiesData } = useSignalHubPoliciesQuery()
  const applyProfile = useApplySignalHubProfileMutation()
  const createProfile = useCreateSignalHubProfileMutation()
  const runHealthCheck = useRunSignalHubHealthCheckMutation()
  const createConnection = useCreateSignalHubConnectionMutation()
  const createPolicy = useCreateSignalHubPolicyMutation()
  const enableSource = useEnableSignalHubSourceMutation()
  const disableSource = useDisableSignalHubSourceMutation()
  const enableSignals = useEnableSignalHubMutation()
  const disableSignals = useDisableSignalHubMutation()
  const muteSignals = useMuteSignalHubMutation()
  const unmuteSignals = useUnmuteSignalHubMutation()
  const pauseSignals = usePauseSignalHubMutation()
  const resumeSignals = useResumeSignalHubMutation()
  const createReplayRequest = useCreateSignalHubReplayRequestMutation()
  const updateConnection = useUpdateSignalHubConnectionMutation()
  const removeConnection = useRemoveSignalHubConnectionMutation()
  const updateProfile = useUpdateSignalHubProfileMutation()
  const removeProfile = useRemoveSignalHubProfileMutation()
  const updateRuntime = useUpdateSignalHubRuntimeStateMutation()

  const activeTab = ref<SignalHubTab>('sources')
  const activeSignalView = ref<SignalHubSettingsView>('graph')
  const selectedSourceCode = ref<string | null>(null)
  const selectedProfileCode = ref<string | null>(null)
  const selectedGraphSourceCode = ref('all')
  const selectedInventorySourceCode = ref('all')
  const sourceSearch = ref('')
  const sourceCategory = ref('all')
  const policyScope = ref<SignalHubPolicyScope>('event_pattern')
  const policyMode = ref<SignalHubPolicyMode>('paused')
  const policySourceCode = ref('system')
  const policyConnectionId = ref('')
  const policyEventPattern = ref('signal.raw.*')
  const policyReason = ref('owner policy')
  const connectionSourceCode = ref('telegram')
  const connectionDisplayName = ref('Primary Connection')
  const connectionProfile = ref('default')
  const profileCodeInput = ref('')
  const profileDisplayNameInput = ref('')
  const profileDescriptionInput = ref('')
  const profilePolicyScope = ref<SignalHubPolicyScope>('source')
  const profilePolicyMode = ref<SignalHubPolicyMode>('muted')
  const profilePolicySourceCode = ref('telegram')
  const profilePolicyConnectionId = ref('')
  const profilePolicyEventPattern = ref('signal.raw.*')
  const profilePolicyReason = ref('profile policy')
  const profileDraftPolicies = ref<SignalHubProfilePolicy[]>([])
  const replaySourceCode = ref('telegram')
  const replayConnectionId = ref('')
  const replayEventPattern = ref('signal.raw.telegram.*')
  const replayTargetConsumer = ref('')
  const replayTargetProjection = ref('')
  const replaySelectorMode = ref<SignalHubReplaySelectorMode>('all')
  const replayFromPosition = ref('')
  const replayToPosition = ref('')
  const replayFromTime = ref('')
  const replayToTime = ref('')

  const tabs: Array<{ id: SignalHubTab; label: string; icon: string }> = [
    { id: 'sources', label: 'Sources', icon: 'tabler:database-import' },
    { id: 'profiles', label: 'Profiles', icon: 'tabler:layout-dashboard' },
    { id: 'connections', label: 'Connections', icon: 'tabler:plug-connected' },
    { id: 'runtime', label: 'Runtime', icon: 'tabler:player-play' },
    { id: 'policies', label: 'Policies', icon: 'tabler:shield-cog' },
    { id: 'health', label: 'Health', icon: 'tabler:activity-heartbeat' },
    { id: 'replay', label: 'Replay', icon: 'tabler:player-track-next' }
  ]

  const allSources = computed(() => sourcesData.value?.items ?? [])
  const sources = computed(() =>
    allSources.value.filter((source) => source.category !== 'test')
  )
  const policies = computed(() => policiesData.value?.items ?? [])
  const profiles = computed(() => profilesData.value?.items ?? [])
  const capabilityItems = computed(() => capabilitiesData.value?.items ?? [])
  const connections = computed(() => connectionsData.value?.items ?? [])
  const runtimeStates = computed(() => runtimeData.value?.items ?? [])
  const healthItems = computed(() => healthData.value?.items ?? [])
  const replayRequests = computed(() => replayData.value?.items ?? [])
  const replayTargetConsumers = computed(() =>
    Array.from(
      new Set(
        runtimeStates.value
          .map((runtime) => runtime.runtime_kind.trim())
          .filter((runtimeKind) => runtimeKind.length > 0)
      )
    ).sort()
  )
  const replayTargetProjections = computed(() =>
    Array.from(
      new Set([
        ...SIGNAL_HUB_REPLAY_PROJECTION_TARGETS,
        ...replayRequests.value
          .map((request) => request.target_projection?.trim() ?? '')
          .filter((target) => target.length > 0)
      ])
    ).sort()
  )
  const categories = computed(() => {
    const values = new Set(sources.value.map((source) => source.category))
    return ['all', ...Array.from(values).sort()]
  })
  const filteredSources = computed(() => {
    const search = sourceSearch.value.trim().toLowerCase()
    return sources.value.filter((source) => {
      const matchesCategory =
        sourceCategory.value === 'all' || source.category === sourceCategory.value
      const matchesSearch =
        search.length === 0 ||
        source.code.toLowerCase().includes(search) ||
        source.display_name.toLowerCase().includes(search)
      return matchesCategory && matchesSearch
    })
  })
  const connectionCapableSources = computed(() =>
    sources.value.filter((source) => source.supports_connections)
  )
  const policyScopeConnections = computed(() =>
    connections.value.filter((connection) =>
      policyScope.value === 'connection' && policySourceCode.value.trim().length > 0
        ? connection.source_code === policySourceCode.value
        : true
    )
  )
  const profileScopeConnections = computed(() =>
    connections.value.filter((connection) =>
      profilePolicyScope.value === 'connection' && profilePolicySourceCode.value.trim().length > 0
        ? connection.source_code === profilePolicySourceCode.value
        : true
    )
  )
  const replayScopedConnections = computed(() =>
    connections.value.filter((connection) =>
      replaySourceCode.value.trim().length > 0 ? connection.source_code === replaySourceCode.value : true
    )
  )
  const selectedSource = computed(() => {
    const selectedCode = selectedSourceCode.value
    if (!selectedCode) return filteredSources.value[0] ?? null
    return sources.value.find((source) => source.code === selectedCode) ?? null
  })
  const selectedProfile = computed(() => {
    const selectedCode = selectedProfileCode.value
    if (!selectedCode) return null
    return profiles.value.find((profile) => profile.code === selectedCode) ?? null
  })
  const selectedSourceCapabilities = computed(() =>
    selectedSource.value
      ? capabilityItems.value.filter(
          (capability) =>
            capability.source_code === selectedSource.value?.code && capability.connection_id === null
        )
      : []
  )
  const enabledCount = computed(
    () => sources.value.filter((source) => sourceControlState(policies.value, source) === 'running').length
  )
  const runtimeCount = computed(() => sources.value.filter((source) => source.supports_runtime).length)
  const activeRuntimeCount = computed(
    () => runtimeStates.value.filter((runtime) => runtime.state === 'running').length
  )
  const replayCount = computed(() => sources.value.filter((source) => source.supports_replay).length)
  const connectedCount = computed(
    () => connections.value.filter((connection) => connection.status === 'connected').length
  )
  const activeProfile = computed(() => profiles.value.find((profile) => profile.is_active) ?? null)
  const unhealthyCount = computed(() => healthItems.value.filter((item) => item.level !== 'healthy').length)
  const replayPendingCount = computed(
    () =>
      replayRequests.value.filter(
        (request) => request.status !== 'completed' && request.status !== 'failed'
      ).length
  )
  const signalConsumerGraph = computed(() =>
    allSources.value.map((source) =>
      buildSignalConsumerGraphRoute(source, policies.value, runtimeStates.value, replayRequests.value)
    )
  )
  const graphSourceCodes = computed(() => new Set(signalConsumerGraph.value.map((route) => route.source.code)))
  const graphSourceTabs = computed(() => buildSignalGraphTabs(signalConsumerGraph.value))
  const filteredSignalConsumerGraph = computed(() => {
    const selectedSource = selectedGraphSourceCode.value
    if (selectedSource === 'all' || !graphSourceCodes.value.has(selectedSource)) {
      return signalConsumerGraph.value
    }
    return signalConsumerGraph.value.filter((route) => route.source.code === selectedSource)
  })
  const signalInventoryRows = computed(() =>
    allSources.value.map((source) =>
      buildSignalInventoryRow(
        source,
        policies.value,
        connections.value,
        runtimeStates.value,
        healthItems.value,
        capabilityItems.value,
        replayRequests.value
      )
    )
  )
  const inventorySourceCodes = computed(() => new Set(signalInventoryRows.value.map((row) => row.source.code)))
  const inventorySourceTabs = computed(() => buildSignalInventoryTabs(signalInventoryRows.value))
  const filteredSignalInventoryRows = computed(() => {
    const selectedSource = selectedInventorySourceCode.value
    if (selectedSource === 'all' || !inventorySourceCodes.value.has(selectedSource)) {
      return signalInventoryRows.value
    }
    return signalInventoryRows.value.filter((row) => row.source.code === selectedSource)
  })
  const signalViewTabs = computed(() => [
    { id: 'graph' as const, label: 'Graph', count: signalConsumerGraph.value.length },
    { id: 'inventory' as const, label: 'Inventory', count: signalInventoryRows.value.length }
  ])
  const isApplyingProfile = computed(() => applyProfile.isPending.value)
  const isSavingProfile = computed(() => createProfile.isPending.value || updateProfile.isPending.value)
  const isRemovingProfile = computed(() => removeProfile.isPending.value)
  const isRunningHealthCheck = computed(() => runHealthCheck.isPending.value)
  const isCreatingConnection = computed(() => createConnection.isPending.value)
  const isCreatingPolicy = computed(() => createPolicy.isPending.value)
  const isUpdatingSignalControls = computed(
    () =>
      enableSource.isPending.value ||
      disableSource.isPending.value ||
      muteSignals.isPending.value ||
      unmuteSignals.isPending.value ||
      pauseSignals.isPending.value ||
      resumeSignals.isPending.value
  )
  const isCreatingReplayRequest = computed(() => createReplayRequest.isPending.value)
  const isUpdatingConnection = computed(() => updateConnection.isPending.value || removeConnection.isPending.value)
  const isUpdatingRuntime = computed(() => updateRuntime.isPending.value)

  async function handleApplyProfile(profileCode: string) {
    await applyProfile.mutateAsync(profileCode)
  }

  function resetProfileEditor() {
    selectedProfileCode.value = null
    profileCodeInput.value = ''
    profileDisplayNameInput.value = ''
    profileDescriptionInput.value = ''
    profileDraftPolicies.value = []
    profilePolicyScope.value = 'source'
    profilePolicyMode.value = 'muted'
    profilePolicySourceCode.value = connectionCapableSources.value[0]?.code ?? 'telegram'
    profilePolicyConnectionId.value = ''
    profilePolicyEventPattern.value = 'signal.raw.*'
    profilePolicyReason.value = 'profile policy'
  }

  function handleSelectProfile(profile: SignalHubProfile) {
    selectedProfileCode.value = profile.code
    profileCodeInput.value = profile.code
    profileDisplayNameInput.value = profile.display_name
    profileDescriptionInput.value = profile.description
    profileDraftPolicies.value = profile.source_policies.map((policy) => ({ ...policy }))
  }

  function handleSelectGraphSource(sourceCode: string) {
    selectedGraphSourceCode.value = sourceCode
  }

  function handleSelectInventorySource(sourceCode: string) {
    selectedInventorySourceCode.value = sourceCode
  }

  function handleSelectSignalView(view: SignalHubSettingsView) {
    activeSignalView.value = view
  }

  function addDraftProfilePolicy() {
    profileDraftPolicies.value = [
      ...profileDraftPolicies.value,
      {
        scope: profilePolicyScope.value,
        source_code:
          profilePolicyScope.value === 'source' || profilePolicyScope.value === 'connection'
            ? profilePolicySourceCode.value
            : null,
        connection_id:
          profilePolicyScope.value === 'connection' ? profilePolicyConnectionId.value || null : null,
        event_pattern:
          profilePolicyScope.value === 'event_pattern' ? profilePolicyEventPattern.value : null,
        mode: profilePolicyMode.value,
        reason: profilePolicyReason.value
      }
    ]
  }

  function removeDraftProfilePolicy(index: number) {
    profileDraftPolicies.value = profileDraftPolicies.value.filter((_, policyIndex) => policyIndex !== index)
  }

  async function handleSaveProfile() {
    const request = {
      display_name: profileDisplayNameInput.value,
      description: profileDescriptionInput.value,
      source_policies: profileDraftPolicies.value
    }

    if (selectedProfile.value) {
      await updateProfile.mutateAsync({ profileCode: selectedProfile.value.code, request })
      return
    }

    await createProfile.mutateAsync({ code: profileCodeInput.value, ...request })
  }

  async function handleRemoveProfile(profileCode: string) {
    await removeProfile.mutateAsync(profileCode)
    resetProfileEditor()
  }

  async function handleRunHealthCheck(sourceCode: string, connectionId?: string | null) {
    await runHealthCheck.mutateAsync({ source_code: sourceCode, connection_id: connectionId ?? null })
  }

  async function handleCreatePolicy() {
    const request = {
      scope: policyScope.value,
      source_code:
        policyScope.value === 'source' || policyScope.value === 'connection' ? policySourceCode.value : null,
      connection_id: policyScope.value === 'connection' ? policyConnectionId.value : null,
      event_pattern: policyScope.value === 'event_pattern' ? policyEventPattern.value : null,
      reason: policyReason.value
    }

    if (policyMode.value === 'paused') return pauseSignals.mutateAsync(request)
    if (policyMode.value === 'muted') return muteSignals.mutateAsync(request)
    if (policyMode.value === 'disabled' && policyScope.value === 'source') {
      return disableSource.mutateAsync(policySourceCode.value)
    }
    if (policyMode.value === 'disabled') return disableSignals.mutateAsync(request)

    await createPolicy.mutateAsync({ ...request, mode: policyMode.value })
  }

  async function handleCreateConnection() {
    await createConnection.mutateAsync({
      source_code: connectionSourceCode.value,
      display_name: connectionDisplayName.value,
      status: 'connected',
      profile: connectionProfile.value,
      settings: {}
    })
  }

  async function handleCreateReplayRequest() {
    await createReplayRequest.mutateAsync(
      buildSignalHubReplayRequest({
        source_code: replaySourceCode.value,
        connection_id: replayConnectionId.value,
        event_pattern: replayEventPattern.value,
        target_consumer: replayTargetConsumer.value,
        target_projection: replayTargetProjection.value,
        selector_mode: replaySelectorMode.value,
        from_position: replayFromPosition.value,
        to_position: replayToPosition.value,
        from_time: replayFromTime.value,
        to_time: replayToTime.value
      })
    )
  }

  async function handleSetConnectionStatus(connectionId: string, status: string) {
    await updateConnection.mutateAsync({ connectionId, request: { status } })
  }

  async function handleRemoveConnection(connectionId: string) {
    await removeConnection.mutateAsync(connectionId)
  }

  async function handleSetRuntimeState(runtime: SignalHubRuntimeState, state: string) {
    await updateRuntime.mutateAsync({
      source_code: runtime.source_code,
      runtime_kind: runtime.runtime_kind,
      state,
      metadata: runtime.metadata
    })
  }

  async function handleEnableSource(sourceCode: string) {
    await enableSource.mutateAsync(sourceCode)
  }

  async function handleDisableSource(sourceCode: string) {
    await disableSource.mutateAsync(sourceCode)
  }

  async function handlePauseSourceSignals(sourceCode: string) {
    await pauseSignals.mutateAsync({
      scope: 'source',
      source_code: sourceCode,
      reason: 'settings signal inventory pause'
    })
  }

  async function handleResumeSourceSignals(sourceCode: string) {
    await resumeSignals.mutateAsync({
      scope: 'source',
      source_code: sourceCode,
      reason: 'settings signal inventory resume'
    })
  }

  async function handleMuteSourceSignals(sourceCode: string) {
    await muteSignals.mutateAsync({
      scope: 'source',
      source_code: sourceCode,
      reason: 'settings signal inventory mute'
    })
  }

  async function handleUnmuteSourceSignals(sourceCode: string) {
    await unmuteSignals.mutateAsync({
      scope: 'source',
      source_code: sourceCode,
      reason: 'settings signal inventory unmute'
    })
  }

  async function handleClearPolicy(policy: {
    scope: SignalHubPolicyScope
    mode: SignalHubPolicyMode
    source_code: string | null
    connection_id: string | null
    event_pattern: string | null
    reason: string
  }) {
    const request = {
      scope: policy.scope,
      source_code: policy.source_code,
      connection_id: policy.connection_id,
      event_pattern: policy.event_pattern,
      reason: policy.reason
    }
    if (policy.mode === 'paused') return resumeSignals.mutateAsync(request)
    if (policy.mode === 'muted') return unmuteSignals.mutateAsync(request)
    if (policy.mode === 'disabled' && policy.scope === 'source' && policy.source_code) {
      return enableSource.mutateAsync(policy.source_code)
    }
    if (policy.mode === 'disabled') return enableSignals.mutateAsync(request)
  }

  return {
    activeTab,
    activeProfile,
    activeRuntimeCount,
    activeSignalView,
    addDraftProfilePolicy,
    applyProfile,
    capabilitiesData,
    capabilityItems,
    categories,
    connectionCapableSources,
    connectionDisplayName,
    connectionProfile,
    connectionSourceCode,
    connections,
    connectedCount,
    createReplayRequest,
    describeSignalHubReplayRequest,
    enabledCount,
    filteredSignalConsumerGraph,
    filteredSignalInventoryRows,
    filteredSources,
    graphSourceTabs,
    handleApplyProfile,
    handleClearPolicy,
    handleCreateConnection,
    handleCreatePolicy,
    handleCreateReplayRequest,
    handleDisableSource,
    handleEnableSource,
    handleMuteSourceSignals,
    handlePauseSourceSignals,
    handleRemoveConnection,
    handleRemoveProfile,
    handleRunHealthCheck,
    handleSaveProfile,
    handleSelectGraphSource,
    handleSelectInventorySource,
    handleSelectProfile,
    handleSelectSignalView,
    handleSetConnectionStatus,
    handleSetRuntimeState,
    handleResumeSourceSignals,
    handleUnmuteSourceSignals,
    healthItems,
    isApplyingProfile,
    isCreatingConnection,
    isCreatingPolicy,
    isCreatingReplayRequest,
    isLoading,
    isRemovingProfile,
    isRunningHealthCheck,
    isSavingProfile,
    isUpdatingConnection,
    isUpdatingRuntime,
    isUpdatingSignalControls,
    inventorySourceTabs,
    policies,
    policyConnectionId,
    policyEventPattern,
    policyMode,
    policyReason,
    policyScope,
    policyScopeConnections,
    policySourceCode,
    profileCodeInput,
    profileDescriptionInput,
    profileDisplayNameInput,
    profileDraftPolicies,
    profilePolicyConnectionId,
    profilePolicyEventPattern,
    profilePolicyMode,
    profilePolicyReason,
    profilePolicyScope,
    profilePolicySourceCode,
    profileScopeConnections,
    profiles,
    replayConnectionId,
    replayCount,
    replayData,
    replayEventPattern,
    replayFromPosition,
    replayFromTime,
    replayPendingCount,
    replayRequests,
    replayScopedConnections,
    replaySelectorMode,
    replaySourceCode,
    replayTargetConsumer,
    replayTargetConsumers,
    replayTargetProjections,
    replayTargetProjection,
    replayToPosition,
    replayToTime,
    removeDraftProfilePolicy,
    resetProfileEditor,
    runtimeCount,
    runtimeStates,
    selectedProfile,
    selectedProfileCode,
    selectedGraphSourceCode,
    selectedInventorySourceCode,
    selectedSource,
    selectedSourceCapabilities,
    selectedSourceCode,
    signalConsumerGraph,
    signalInventoryRows,
    signalViewTabs,
    sourceCategory,
    sourceSearch,
    sources,
    tabs,
    unhealthyCount
  }
}

export type SignalHubSettingsSurface = ReturnType<typeof useSignalHubSettingsSurface>
