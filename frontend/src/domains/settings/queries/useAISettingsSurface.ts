import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { loadFrontendConfig } from '../../../platform/config/env'
import { useI18n } from '../../../platform/i18n'
import { useSettingsStore } from '../stores/settings'
import {
  DEFAULT_API_PROVIDER_PRESETS,
  DEFAULT_OPENAI_BASE_URL,
} from './aiSettingsCatalog'
import {
  errorMessage,
  modelStateKey,
  mergedApiPresets,
  buildAiModelRouteRows,
  modelDownloadProgressLabel as formatModelDownloadProgressLabel,
  modelRouteUsageCount as countModelRouteUsage,
  providerBaseUrl as resolveProviderBaseUrl,
} from './aiSettingsPresentation'
import {
  metadataBoolean,
  providerIsBuiltInOllama
} from './aiSettingsPredicates'
import type {
  AiModelCatalogItem,
  AiProviderAuthStartResponse,
  AiProviderAuthStatusResponse,
  AiProviderAccount,
  AiProviderCommandResponse,
  AiProviderPreset,
} from '../types/aiControlCenter'
import type { SelectOption } from '../../../shared/ui/Selection.types'
import {
  useAiSettingsOverviewQuery,
  useAiHubUsageStatsQuery,
  useCreateAiProviderMutation,
  useDeleteAiModelRouteMutation,
  useDownloadAiModelMutation,
  useFetchAiProviderAuthStatusMutation,
  useStartAiProviderAuthMutation,
  useSyncAiProviderModelsMutation,
  useTestAiProviderMutation,
  useUpdateAiModelAvailabilityMutation,
  useUpdateAiModelRouteMutation,
  useUpdateAiProviderConsentMutation,
  useUpdateAiProviderMutation,
} from './useAISettingsQuery'
import {
  syncAiProviderModels,
  testAiProvider,
  toggleAiProvider,
  updateAiProviderConsent
} from './aiProviderActions'
import { updateAiModelAvailability } from './aiModelActions'
import { updateAiRouteSelection } from './aiRouteActions'
import { downloadAiModel } from './aiModelDownloadActions'
import {
  buildAiProviderAuthStartRequest,
  buildAiProviderCallbackUrl
} from './aiProviderAuthRequests'
import { buildAiApiProviderCreateRequest } from './aiProviderRequests'
import {
  refreshAiProviderAuthStatus,
  startAiProviderAuth
} from './aiProviderAuthActions'
import { parseAiProviderCallbackMessage } from './aiProviderCallback'


const LOCAL_AUTH_POLL_INTERVAL_MS = 2500
const DOWNLOAD_PROGRESS_MAX_BEFORE_COMPLETION = 92

type AiProviderAuthFlow = AiProviderAuthStartResponse | AiProviderAuthStatusResponse

export function useAISettingsSurface() {
  const { t } = useI18n()
  const store = useSettingsStore()
  const frontendConfig = loadFrontendConfig()
  const overviewQuery = useAiSettingsOverviewQuery()
  const usageStatsQuery = useAiHubUsageStatsQuery(24)
  const createProvider = useCreateAiProviderMutation()
  const updateProvider = useUpdateAiProviderMutation()
  const syncModels = useSyncAiProviderModelsMutation()
  const testProvider = useTestAiProviderMutation()
  const updateModelAvailability = useUpdateAiModelAvailabilityMutation()
  const downloadModel = useDownloadAiModelMutation()
  const updateConsent = useUpdateAiProviderConsentMutation()
  const updateRoute = useUpdateAiModelRouteMutation()
  const deleteRoute = useDeleteAiModelRouteMutation()
  const startProviderAuth = useStartAiProviderAuthMutation()
  const fetchProviderAuthStatus = useFetchAiProviderAuthStatusMutation()

  const selectedProviderId = ref<string | null>(null)
  const apiDisplayName = ref('OpenAI')
  const apiProviderKey = ref('openai')
  const apiBaseUrl = ref(DEFAULT_OPENAI_BASE_URL)
  const apiToken = ref('')
  const apiConsent = ref(true)
  const activeApiPresetKey = ref('openai')
  const activeLocalAuth = ref<AiProviderAuthFlow | null>(null)
  const downloadingModelProgress = ref<Record<string, number>>({})
  let localAuthPollId: number | null = null
  const modelDownloadTimers = new Map<string, number>()

  const overview = computed(() => overviewQuery.data.value ?? null)
  const providers = computed(() => overview.value?.providers ?? [])
  const models = computed(() => overview.value?.models ?? [])
  const routes = computed(() => overview.value?.routes ?? [])
  const capabilitySlots = computed(() => overview.value?.capability_slots ?? [])
  const providerPresets = computed(() => overview.value?.provider_presets ?? [])
  const usageStats = computed(() => usageStatsQuery.data.value ?? null)
  const providerUsageRows = computed(() => usageStats.value?.providers ?? [])
  const hourlyUsageRows = computed(() => usageStats.value?.hourly ?? [])
  const selectedProvider = computed(() => {
    return providers.value.find((provider) => provider.provider_id === selectedProviderId.value) ?? null
  })
  const selectedProviderModels = computed(() => {
    const provider = selectedProvider.value
    if (!provider) return []
    return models.value.filter((model) => model.provider_id === provider.provider_id)
  })
  const routeRows = computed(() => buildAiModelRouteRows(
    capabilitySlots.value,
    routes.value,
    providers.value,
    models.value,
    t
  ))
  const apiPresets = computed(() => mergedApiPresets(DEFAULT_API_PROVIDER_PRESETS, providerPresets.value))
  const apiPresetOptions = computed<SelectOption[]>(() =>
    apiPresets.value.map((preset) => ({
      value: preset.provider_key,
      label: preset.display_name,
      description: preset.base_url ?? t('Custom endpoint'),
      icon: preset.provider_key === 'raw' ? 'tabler:braces' : 'tabler:plug-connected',
    }))
  )
  const localPresets = computed(() =>
    providerPresets.value.filter((preset) => preset.provider_kind !== 'api')
  )

  watch(
    providers,
    (items) => {
      if (!selectedProviderId.value && items.length > 0) {
        selectedProviderId.value = items[0].provider_id
      }
      if (
        selectedProviderId.value &&
        items.length > 0 &&
        !items.some((provider) => provider.provider_id === selectedProviderId.value)
      ) {
        selectedProviderId.value = items[0].provider_id
      }
    },
    { immediate: true }
  )

  function selectProvider(providerId: string) {
    selectedProviderId.value = providerId
  }

  function providerForPreset(preset: AiProviderPreset): AiProviderAccount | null {
    return providers.value.find((provider) =>
      provider.provider_kind === preset.provider_kind && provider.provider_key === preset.provider_key
    ) ?? null
  }

  function applyPreset(preset: AiProviderPreset) {
    apiDisplayName.value = preset.display_name
    apiProviderKey.value = preset.provider_key
    apiBaseUrl.value = preset.base_url ?? ''
    apiConsent.value = preset.privacy === 'remote'
    activeApiPresetKey.value = preset.provider_key
  }

  function handlePresetSelect(providerKey: string) {
    const preset = apiPresets.value.find((item) => item.provider_key === providerKey)
    if (!preset) return

    applyPreset(preset)
  }

  async function handleCreateApiProvider(): Promise<AiProviderAccount | null> {
    const displayName = apiDisplayName.value.trim()
    const providerKey = apiProviderKey.value.trim()
    const baseUrl = apiBaseUrl.value.trim()
    const token = apiToken.value.trim()

    if (!displayName || !providerKey || !baseUrl) {
      store.setError(t('Display name, provider key and base URL are required.'))
      return null
    }
    if (!token) {
      store.setError(t('API token is required for a usable OpenAI-compatible provider.'))
      return null
    }

    try {
      const provider = await createProvider.mutateAsync(buildAiApiProviderCreateRequest({
        providerKey,
        displayName,
        baseUrl,
        token,
        remoteContextConsent: apiConsent.value
      }))
      apiToken.value = ''
      selectedProviderId.value = provider.provider_id
      await overviewQuery.refetch()
      store.setActionMessage(t('AI provider connected'))
      return provider
    } catch (error) {
      store.setError(errorMessage(error, t('AI provider create failed')))
      return null
    }
  }

  async function handleConnectLocalPreset(preset: AiProviderPreset) {
    await startAiProviderAuth(preset, {
      startAuth: (selectedPreset) => startProviderAuth.mutateAsync(buildAiProviderAuthStartRequest(
        selectedPreset,
        aiProviderCallbackBaseUrl()
      )),
      setActiveAuth: (response) => { activeLocalAuth.value = response },
      setSelectedProvider: (providerId) => { selectedProviderId.value = providerId },
      stopPolling: stopLocalAuthPolling,
      startPolling: startLocalAuthPolling,
      setActionMessage: (message) => store.setActionMessage(message),
      setError: (message) => store.setError(message),
      t
    })
  }

  async function handleRefreshLocalAuth() {
    const setupId = activeLocalAuth.value?.setup_id
    if (!setupId) return
    await refreshLocalAuthStatus(setupId, true)
  }

  function handleOpenLocalAuthCallback() {
    const callbackUrl = activeLocalAuth.value?.callback_url
    if (!callbackUrl) return
    if (typeof window !== 'undefined' && typeof window.open === 'function') {
      window.open(callbackUrl, '_blank', 'noopener,noreferrer')
    }
  }

  async function handleToggleProvider(provider: AiProviderAccount, enabled: boolean) {
    await toggleAiProvider(provider, enabled, {
      updateProvider: (variables) => updateProvider.mutateAsync(variables),
      setSelectedProvider: (providerId) => { selectedProviderId.value = providerId },
      setActionMessage: (message) => store.setActionMessage(message),
      setError: (message) => store.setError(message),
      t
    })
  }

  async function handleGrantConsent(provider: AiProviderAccount, consented: boolean) {
    await updateAiProviderConsent(provider, consented, {
      updateConsent: (variables) => updateConsent.mutateAsync(variables),
      setSelectedProvider: (providerId) => { selectedProviderId.value = providerId },
      setActionMessage: (message) => store.setActionMessage(message),
      setError: (message) => store.setError(message),
      t
    })
  }

  async function handleTestProvider(provider: AiProviderAccount): Promise<AiProviderCommandResponse | null> {
    return testAiProvider(provider, {
      execute: (providerId) => testProvider.mutateAsync(providerId),
      setActionMessage: (message) => store.setActionMessage(message),
      setError: (message) => store.setError(message),
      t
    })
  }

  async function handleSyncModels(provider: AiProviderAccount): Promise<AiProviderCommandResponse | null> {
    return syncAiProviderModels(provider, {
      execute: (providerId) => syncModels.mutateAsync(providerId),
      refreshOverview: () => overviewQuery.refetch(),
      setActionMessage: (message) => store.setActionMessage(message),
      setError: (message) => store.setError(message),
      t
    })
  }

  async function handleModelAvailability(
    model: AiModelCatalogItem,
    isAvailable: boolean
  ): Promise<AiModelCatalogItem | null> {
    return updateAiModelAvailability(model, isAvailable, {
      updateAvailability: (request) => updateModelAvailability.mutateAsync(request),
      refreshOverview: () => overviewQuery.refetch(),
      setActionMessage: (message) => store.setActionMessage(message),
      setError: (message) => store.setError(message),
      t
    })
  }

  async function handleModelDownload(model: AiModelCatalogItem): Promise<AiModelCatalogItem | null> {
    const key = modelStateKey(model)
    return downloadAiModel(model, key, {
      isDownloading: (modelKey) => downloadingModelProgress.value[modelKey] !== undefined,
      startProgress: startModelDownloadProgress,
      finishProgress: finishModelDownloadProgress,
      clearProgress: clearModelDownloadProgress,
      download: (request) => downloadModel.mutateAsync(request),
      refreshOverview: () => overviewQuery.refetch(),
      setActionMessage: (message) => store.setActionMessage(message),
      setError: (message) => store.setError(message),
      t
    })
  }

  async function handleRouteSelection(slot: string, value: string) {
    await updateAiRouteSelection(slot, value, {
      updateRoute: (variables) => updateRoute.mutateAsync(variables),
      deleteRoute: (routeSlot) => deleteRoute.mutateAsync(routeSlot),
      refreshOverview: () => overviewQuery.refetch(),
      setActionMessage: (message) => store.setActionMessage(message),
      setError: (message) => store.setError(message),
      t
    })
  }

  async function handleRefreshModelRoutes() {
    await overviewQuery.refetch()
    store.setActionMessage(t('AI model list refreshed'))
  }

  async function handleRefreshUsageStats() {
    await usageStatsQuery.refetch()
    store.setActionMessage(t('AI Hub usage stats refreshed'))
  }

  function modelRequiresDownload(model: AiModelCatalogItem): boolean {
    return metadataBoolean(model.metadata, 'pull_required') && providerIsBuiltInOllama(model.provider_id)
  }

  function modelDownloadProgressValue(model: AiModelCatalogItem): number | null {
    const progress = downloadingModelProgress.value[modelStateKey(model)]
    return typeof progress === 'number' ? progress : null
  }

  function modelDownloadProgressLabel(model: AiModelCatalogItem): string | null {
    const progress = modelDownloadProgressValue(model)
    return formatModelDownloadProgressLabel(progress, t)
  }

  function modelIsDownloading(model: AiModelCatalogItem): boolean {
    return modelDownloadProgressValue(model) !== null
  }

  function modelRouteUsageCount(model: AiModelCatalogItem): number {
    return countModelRouteUsage(model, routes.value)
  }

  function modelIsUsedByRoute(model: AiModelCatalogItem): boolean {
    return modelRouteUsageCount(model) > 0
  }

  function providerBaseUrl(provider: AiProviderAccount): string {
    return resolveProviderBaseUrl(provider)
  }

  async function refreshLocalAuthStatus(setupId: string, notify: boolean) {
    await refreshAiProviderAuthStatus(setupId, notify, {
      isPending: () => fetchProviderAuthStatus.isPending.value,
      fetchStatus: (authSetupId) => fetchProviderAuthStatus.mutateAsync(authSetupId),
      setActiveAuth: (response) => { activeLocalAuth.value = response },
      setSelectedProvider: (providerId) => { selectedProviderId.value = providerId },
      stopPolling: stopLocalAuthPolling,
      setActionMessage: (message) => store.setActionMessage(message),
      setError: (message) => store.setError(message),
      t
    })
  }

  function startLocalAuthPolling(setupId: string) {
    stopLocalAuthPolling()
    if (typeof window === 'undefined') return
    localAuthPollId = window.setInterval(() => {
      void refreshLocalAuthStatus(setupId, false)
    }, LOCAL_AUTH_POLL_INTERVAL_MS)
  }

  function stopLocalAuthPolling() {
    if (typeof window === 'undefined' || localAuthPollId === null) return
    window.clearInterval(localAuthPollId)
    localAuthPollId = null
  }

  function startModelDownloadProgress(key: string) {
    clearModelDownloadProgress(key)
    downloadingModelProgress.value = {
      ...downloadingModelProgress.value,
      [key]: 8,
    }
    if (typeof window === 'undefined') return
    const timerId = window.setInterval(() => {
      const current = downloadingModelProgress.value[key]
      if (typeof current !== 'number') return
      const next = Math.min(current + Math.max(3, Math.round((100 - current) / 6)), DOWNLOAD_PROGRESS_MAX_BEFORE_COMPLETION)
      downloadingModelProgress.value = {
        ...downloadingModelProgress.value,
        [key]: next,
      }
    }, 500)
    modelDownloadTimers.set(key, timerId)
  }

  function finishModelDownloadProgress(key: string) {
    if (typeof window !== 'undefined') {
      const timerId = modelDownloadTimers.get(key)
      if (timerId !== undefined) {
        window.clearInterval(timerId)
        modelDownloadTimers.delete(key)
      }
      downloadingModelProgress.value = {
        ...downloadingModelProgress.value,
        [key]: 100,
      }
      window.setTimeout(() => {
        clearModelDownloadProgress(key)
      }, 400)
      return
    }
    clearModelDownloadProgress(key)
  }

  function clearModelDownloadProgress(key: string) {
    if (typeof window !== 'undefined') {
      const timerId = modelDownloadTimers.get(key)
      if (timerId !== undefined) {
        window.clearInterval(timerId)
        modelDownloadTimers.delete(key)
      }
    }
    if (downloadingModelProgress.value[key] === undefined) return
    const next = { ...downloadingModelProgress.value }
    delete next[key]
    downloadingModelProgress.value = next
  }

  function aiProviderCallbackBaseUrl(): string {
    return buildAiProviderCallbackUrl(frontendConfig.apiBaseUrl)
  }

  function handleProviderCallbackMessage(event: MessageEvent) {
    const callback = parseAiProviderCallbackMessage(event.data)
    if (!callback) return
    if (callback.providerId) selectedProviderId.value = callback.providerId
    stopLocalAuthPolling()
    void overviewQuery.refetch()
    store.setActionMessage(t('AI provider connected'))
  }

  onMounted(() => {
    if (typeof window !== 'undefined') {
      window.addEventListener('message', handleProviderCallbackMessage)
    }
  })

  onUnmounted(() => {
    if (typeof window !== 'undefined') {
      window.removeEventListener('message', handleProviderCallbackMessage)
    }
    stopLocalAuthPolling()
    if (typeof window !== 'undefined') {
      for (const timerId of modelDownloadTimers.values()) {
        window.clearInterval(timerId)
      }
    }
    modelDownloadTimers.clear()
  })

  const isBusy = computed(() =>
    createProvider.isPending.value ||
    updateProvider.isPending.value ||
    syncModels.isPending.value ||
    testProvider.isPending.value ||
    updateModelAvailability.isPending.value ||
    downloadModel.isPending.value ||
    updateConsent.isPending.value ||
    updateRoute.isPending.value ||
    deleteRoute.isPending.value ||
    startProviderAuth.isPending.value ||
    fetchProviderAuthStatus.isPending.value
  )

  return {
    apiBaseUrl,
    apiConsent,
    apiDisplayName,
    activeApiPresetKey,
    activeLocalAuth,
    apiPresetOptions,
    apiPresets,
    apiProviderKey,
    apiToken,
    capabilitySlots,
    handleCreateApiProvider,
    handleConnectLocalPreset,
    handleGrantConsent,
    handleModelDownload,
    handleModelAvailability,
    handleOpenLocalAuthCallback,
    handlePresetSelect,
    handleRefreshLocalAuth,
    handleRefreshModelRoutes,
    handleRefreshUsageStats,
    handleRouteSelection,
    handleSyncModels,
    handleTestProvider,
    handleToggleProvider,
    hourlyUsageRows,
    isBusy,
    isLoading: overviewQuery.isLoading,
    localPresets,
    modelDownloadProgressValue,
    modelDownloadProgressLabel,
    modelIsDownloading,
    modelIsUsedByRoute,
    modelRequiresDownload,
    modelRouteUsageCount,
    models,
    overview,
    providers,
    providerBaseUrl,
    providerForPreset,
    providerUsageRows,
    routeRows,
    routes,
    selectProvider,
    selectedProvider,
    selectedProviderId,
    selectedProviderModels,
    usageStats,
    usageStatsQuery,
  }
}

export type AISettingsSurface = ReturnType<typeof useAISettingsSurface>
