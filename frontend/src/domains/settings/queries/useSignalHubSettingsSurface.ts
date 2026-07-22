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
  activeProfile as activeSignalHubProfile,
  buildSignalConsumerGraph,
  buildSignalInventory,
  countActiveRuntimeStates,
  countConnectedConnections,
  countPendingReplayRequests,
  countReplaySources,
  countRunningSources,
  countRuntimeSources,
  countUnhealthyHealthItems,
  filterConnectionsForScope,
  filterConnectionsForSource,
  filterSignalConsumerGraph,
  filterSignalInventory,
  filterSignalSources,
  findSelectedProfile,
  findSelectedSource,
  graphTabs,
  inventoryTabs,
  replayTargetConsumers,
  replayTargetProjections,
  SIGNAL_HUB_TABS,
  signalCategories,
  signalViewTabs as buildSignalViewTabs,
  sourceCapabilities,
  visibleSignalSources,
  type SignalHubTab
} from './signalHubSettingsSelectors'
import {
  buildSignalHubSourcePolicyRequest,
  clearSignalHubPolicy,
  createSignalHubPolicy,
  type SignalHubPolicyDraft
} from './signalHubPolicyActions'
import {
  buildSignalHubProfilePolicy,
  buildSignalHubProfileSaveRequest
} from './signalHubProfileRequests'
import {
  removeSignalHubProfile,
  saveSignalHubProfile
} from './signalHubProfileActions'
import {
  buildSignalHubConnectionCreateRequest,
  buildSignalHubConnectionStatusRequest,
  buildSignalHubHealthCheckRequest,
  buildSignalHubRuntimeStateRequest
} from './signalHubOperationalRequests'

export type { SignalHubTab } from './signalHubSettingsSelectors'
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

  const allSources = computed(() => sourcesData.value?.items ?? [])
  const sources = computed(() => visibleSignalSources(allSources.value))
  const policies = computed(() => policiesData.value?.items ?? [])
  const profiles = computed(() => profilesData.value?.items ?? [])
  const capabilityItems = computed(() => capabilitiesData.value?.items ?? [])
  const connections = computed(() => connectionsData.value?.items ?? [])
  const runtimeStates = computed(() => runtimeData.value?.items ?? [])
  const healthItems = computed(() => healthData.value?.items ?? [])
  const replayRequests = computed(() => replayData.value?.items ?? [])
  const replayTargetConsumerOptions = computed(() => replayTargetConsumers(runtimeStates.value))
  const replayTargetProjectionOptions = computed(() => replayTargetProjections(replayRequests.value))
  const categories = computed(() => signalCategories(sources.value))
  const filteredSources = computed(() =>
    filterSignalSources(sources.value, sourceSearch.value, sourceCategory.value)
  )
  const connectionCapableSources = computed(() =>
    sources.value.filter((source) => source.supports_connections)
  )
  const policyScopeConnections = computed(() =>
    filterConnectionsForScope(connections.value, policyScope.value, policySourceCode.value)
  )
  const profileScopeConnections = computed(() =>
    filterConnectionsForScope(connections.value, profilePolicyScope.value, profilePolicySourceCode.value)
  )
  const replayScopedConnections = computed(() =>
    filterConnectionsForSource(connections.value, replaySourceCode.value)
  )
  const selectedSource = computed(() => {
    const selectedCode = selectedSourceCode.value
    return findSelectedSource(sources.value, filteredSources.value, selectedCode)
  })
  const selectedProfile = computed(() => {
    const selectedCode = selectedProfileCode.value
    return findSelectedProfile(profiles.value, selectedCode)
  })
  const selectedSourceCapabilities = computed(() =>
    sourceCapabilities(capabilityItems.value, selectedSource.value)
  )
  const enabledCount = computed(() => countRunningSources(sources.value, policies.value))
  const runtimeCount = computed(() => countRuntimeSources(sources.value))
  const activeRuntimeCount = computed(() => countActiveRuntimeStates(runtimeStates.value))
  const replayCount = computed(() => countReplaySources(sources.value))
  const connectedCount = computed(() => countConnectedConnections(connections.value))
  const activeProfile = computed(() => activeSignalHubProfile(profiles.value))
  const unhealthyCount = computed(() => countUnhealthyHealthItems(healthItems.value))
  const replayPendingCount = computed(() => countPendingReplayRequests(replayRequests.value))
  const signalConsumerGraph = computed(() =>
    buildSignalConsumerGraph(allSources.value, policies.value, runtimeStates.value, replayRequests.value)
  )
  const graphSourceTabs = computed(() => graphTabs(signalConsumerGraph.value))
  const filteredSignalConsumerGraph = computed(() =>
    filterSignalConsumerGraph(signalConsumerGraph.value, selectedGraphSourceCode.value)
  )
  const signalInventoryRows = computed(() =>
    buildSignalInventory(
      allSources.value,
      policies.value,
      connections.value,
      runtimeStates.value,
      healthItems.value,
      capabilityItems.value,
      replayRequests.value
    )
  )
  const inventorySourceTabs = computed(() => inventoryTabs(signalInventoryRows.value))
  const filteredSignalInventoryRows = computed(() =>
    filterSignalInventory(signalInventoryRows.value, selectedInventorySourceCode.value)
  )
  const signalViewTabs = computed(() =>
    buildSignalViewTabs(signalConsumerGraph.value, signalInventoryRows.value)
  )
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
      buildSignalHubProfilePolicy({
        scope: profilePolicyScope.value,
        sourceCode: profilePolicySourceCode.value,
        connectionId: profilePolicyConnectionId.value,
        eventPattern: profilePolicyEventPattern.value,
        mode: profilePolicyMode.value,
        reason: profilePolicyReason.value
      })
    ]
  }

  function removeDraftProfilePolicy(index: number) {
    profileDraftPolicies.value = profileDraftPolicies.value.filter((_, policyIndex) => policyIndex !== index)
  }

  async function handleSaveProfile() {
    const request = buildSignalHubProfileSaveRequest(
      profileDisplayNameInput.value,
      profileDescriptionInput.value,
      profileDraftPolicies.value
    )

    await saveSignalHubProfile(selectedProfile.value?.code ?? null, profileCodeInput.value, request, {
      update: (variables) => updateProfile.mutateAsync(variables),
      create: (createRequest) => createProfile.mutateAsync(createRequest)
    })
  }

  async function handleRemoveProfile(profileCode: string) {
    await removeSignalHubProfile(profileCode, {
      remove: (code) => removeProfile.mutateAsync(code),
      resetEditor: resetProfileEditor
    })
  }

  async function handleRunHealthCheck(sourceCode: string, connectionId?: string | null) {
    await runHealthCheck.mutateAsync(buildSignalHubHealthCheckRequest(sourceCode, connectionId))
  }

  async function handleCreatePolicy() {
    const request: SignalHubPolicyDraft = {
      scope: policyScope.value,
      source_code:
        policyScope.value === 'source' || policyScope.value === 'connection' ? policySourceCode.value : null,
      connection_id: policyScope.value === 'connection' ? policyConnectionId.value : null,
      event_pattern: policyScope.value === 'event_pattern' ? policyEventPattern.value : null,
      reason: policyReason.value
    }

    await createSignalHubPolicy(request, policyMode.value, {
      pause: (draft) => pauseSignals.mutateAsync(draft),
      mute: (draft) => muteSignals.mutateAsync(draft),
      disableSource: (sourceCode) => disableSource.mutateAsync(sourceCode),
      disable: (draft) => disableSignals.mutateAsync(draft),
      create: (draft) => createPolicy.mutateAsync(draft)
    })
  }

  async function handleCreateConnection() {
    await createConnection.mutateAsync(buildSignalHubConnectionCreateRequest(
      connectionSourceCode.value,
      connectionDisplayName.value,
      connectionProfile.value
    ))
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
    await updateConnection.mutateAsync({
      connectionId,
      request: buildSignalHubConnectionStatusRequest(status)
    })
  }

  async function handleRemoveConnection(connectionId: string) {
    await removeConnection.mutateAsync(connectionId)
  }

  async function handleSetRuntimeState(runtime: SignalHubRuntimeState, state: string) {
    await updateRuntime.mutateAsync(buildSignalHubRuntimeStateRequest(runtime, state))
  }

  async function handleEnableSource(sourceCode: string) {
    await enableSource.mutateAsync(sourceCode)
  }

  async function handleDisableSource(sourceCode: string) {
    await disableSource.mutateAsync(sourceCode)
  }

  async function handlePauseSourceSignals(sourceCode: string) {
    await pauseSignals.mutateAsync(buildSignalHubSourcePolicyRequest(
      sourceCode, 'settings signal inventory pause'
    ))
  }

  async function handleResumeSourceSignals(sourceCode: string) {
    await resumeSignals.mutateAsync(buildSignalHubSourcePolicyRequest(
      sourceCode, 'settings signal inventory resume'
    ))
  }

  async function handleMuteSourceSignals(sourceCode: string) {
    await muteSignals.mutateAsync(buildSignalHubSourcePolicyRequest(
      sourceCode, 'settings signal inventory mute'
    ))
  }

  async function handleUnmuteSourceSignals(sourceCode: string) {
    await unmuteSignals.mutateAsync(buildSignalHubSourcePolicyRequest(
      sourceCode, 'settings signal inventory unmute'
    ))
  }

  async function handleClearPolicy(policy: SignalHubPolicyDraft & { mode: SignalHubPolicyMode }) {
    await clearSignalHubPolicy(policy, {
      resume: (draft) => resumeSignals.mutateAsync(draft),
      unmute: (draft) => unmuteSignals.mutateAsync(draft),
      enableSource: (sourceCode) => enableSource.mutateAsync(sourceCode),
      enable: (draft) => enableSignals.mutateAsync(draft)
    })
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
    replayTargetConsumers: replayTargetConsumerOptions,
    replayTargetProjections: replayTargetProjectionOptions,
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
    tabs: SIGNAL_HUB_TABS,
    unhealthyCount
  }
}

export type SignalHubSettingsSurface = ReturnType<typeof useSignalHubSettingsSurface>
