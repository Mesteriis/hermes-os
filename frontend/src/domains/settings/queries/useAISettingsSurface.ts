import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { loadFrontendConfig } from '../../../platform/config/env'
import { useI18n } from '../../../platform/i18n'
import { useSettingsStore } from '../stores/settings'
import {
  DEFAULT_API_PROVIDER_PRESETS,
  DEFAULT_OPENAI_BASE_URL,
  SLOT_DESCRIPTIONS,
  SLOT_LABELS,
} from './aiSettingsCatalog'
import type {
  AiCapabilitySlot,
  AiModelCatalogItem,
  AiModelRoute,
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

export interface AiModelRouteRow {
  slot: AiCapabilitySlot
  label: string
  description: string
  selectedValue: string
  selectedModelLabel: string
  options: AiModelRouteOption[]
}

export interface AiModelRouteOption {
  value: string
  label: string
  detail: string
}

const ROUTE_OPTION_SEPARATOR = '|'
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
  const routeRows = computed<AiModelRouteRow[]>(() => {
    return capabilitySlots.value.map((slot) => {
      const route = routeForSlot(slot.slot)
      const options = routeOptionsForSlot(slot)
      const selectedValue = route ? routeOptionValue(route.provider_id, route.model_key) : ''
      return {
        slot,
        label: t(SLOT_LABELS[slot.slot] ?? slot.label),
        description: t(SLOT_DESCRIPTIONS[slot.slot] ?? slot.description),
        selectedValue,
        selectedModelLabel: route ? modelLabel(route.provider_id, route.model_key) : t('Not routed'),
        options,
      }
    })
  })
  const apiPresets = computed(() => mergedApiPresets(providerPresets.value))
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
      const provider = await createProvider.mutateAsync({
        provider_kind: 'api',
        provider_key: providerKey,
        display_name: displayName,
        base_url: baseUrl,
        capabilities: ['chat', 'reasoning', 'summarization', 'embeddings'],
        enabled: true,
        remote_context_consent: apiConsent.value,
        api_key: token,
      })
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
    try {
      const response = await startProviderAuth.mutateAsync({
        provider_kind: preset.provider_kind,
        provider_key: preset.provider_key,
        display_name: preset.display_name,
        callback_url: aiProviderCallbackBaseUrl(),
      })
      activeLocalAuth.value = response
      if (response.provider) {
        selectedProviderId.value = response.provider.provider_id
        stopLocalAuthPolling()
        store.setActionMessage(t('AI provider connected'))
        return
      }
      startLocalAuthPolling(response.setup_id)
      store.setActionMessage(response.message || t('AI provider callback started'))
    } catch (error) {
      store.setError(errorMessage(error, t('AI provider callback start failed')))
    }
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
    try {
      const updated = await updateProvider.mutateAsync({
        providerId: provider.provider_id,
        request: { enabled },
      })
      selectedProviderId.value = updated.provider_id
      store.setActionMessage(enabled ? t('AI provider enabled') : t('AI provider disabled'))
    } catch (error) {
      store.setError(errorMessage(error, t('AI provider update failed')))
    }
  }

  async function handleGrantConsent(provider: AiProviderAccount, consented: boolean) {
    try {
      const updated = await updateConsent.mutateAsync({
        providerId: provider.provider_id,
        request: { consented },
      })
      selectedProviderId.value = updated.provider_id
      store.setActionMessage(consented ? t('Remote context consent granted') : t('Remote context consent revoked'))
    } catch (error) {
      store.setError(errorMessage(error, t('AI provider consent update failed')))
    }
  }

  async function handleTestProvider(provider: AiProviderAccount): Promise<AiProviderCommandResponse | null> {
    try {
      const response = await testProvider.mutateAsync(provider.provider_id)
      store.setActionMessage(response.message || t('AI provider test completed'))
      return response
    } catch (error) {
      store.setError(errorMessage(error, t('AI provider test failed')))
      return null
    }
  }

  async function handleSyncModels(provider: AiProviderAccount): Promise<AiProviderCommandResponse | null> {
    try {
      const response = await syncModels.mutateAsync(provider.provider_id)
      await overviewQuery.refetch()
      store.setActionMessage(response.message || t('AI models synchronized'))
      return response
    } catch (error) {
      store.setError(errorMessage(error, t('AI model sync failed')))
      return null
    }
  }

  async function handleModelAvailability(
    model: AiModelCatalogItem,
    isAvailable: boolean
  ): Promise<AiModelCatalogItem | null> {
    try {
      const updated = await updateModelAvailability.mutateAsync({
        provider_id: model.provider_id,
        model_key: model.model_key,
        is_available: isAvailable,
      })
      await overviewQuery.refetch()
      store.setActionMessage(isAvailable ? t('AI model enabled') : t('AI model disabled'))
      return updated
    } catch (error) {
      store.setError(errorMessage(error, t('AI model availability update failed')))
      return null
    }
  }

  async function handleModelDownload(model: AiModelCatalogItem): Promise<AiModelCatalogItem | null> {
    const key = modelStateKey(model)
    if (downloadingModelProgress.value[key] !== undefined) return null

    startModelDownloadProgress(key)
    try {
      const updated = await downloadModel.mutateAsync({
        provider_id: model.provider_id,
        model_key: model.model_key,
      })
      finishModelDownloadProgress(key)
      await overviewQuery.refetch()
      store.setActionMessage(t('AI model downloaded'))
      return updated
    } catch (error) {
      clearModelDownloadProgress(key)
      store.setError(errorMessage(error, t('AI model download failed')))
      return null
    }
  }

  async function handleRouteSelection(slot: string, value: string) {
    if (!value) {
      try {
        await deleteRoute.mutateAsync(slot)
        await overviewQuery.refetch()
        store.setActionMessage(t('AI model route cleared'))
      } catch (error) {
        store.setError(errorMessage(error, t('AI model route delete failed')))
      }
      return
    }

    const parsed = parseRouteOptionValue(value)
    if (!parsed) return

    try {
      await updateRoute.mutateAsync({
        slot,
        request: {
          provider_id: parsed.providerId,
          model_key: parsed.modelKey,
        },
      })
      await overviewQuery.refetch()
      store.setActionMessage(t('AI model route updated'))
    } catch (error) {
      store.setError(errorMessage(error, t('AI model route update failed')))
    }
  }

  async function handleRefreshModelRoutes() {
    await overviewQuery.refetch()
    store.setActionMessage(t('AI model list refreshed'))
  }

  async function handleRefreshUsageStats() {
    await usageStatsQuery.refetch()
    store.setActionMessage(t('AI Hub usage stats refreshed'))
  }

  function routeForSlot(slot: string): AiModelRoute | null {
    return routes.value.find((route) => route.capability_slot === slot) ?? null
  }

  function routeOptionsForSlot(slot: AiCapabilitySlot): AiModelRouteOption[] {
    return models.value
      .filter((model) => modelUsableForSlot(model, slot))
      .map((model) => ({
        value: routeOptionValue(model.provider_id, model.model_key),
        label: modelLabel(model.provider_id, model.model_key),
        detail: `${model.privacy} · ${model.category}`,
      }))
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
    if (progress === null) return null
    if (progress >= 100) return t('Finalizing model')
    if (progress >= 70) return t('Preparing model for routing')
    if (progress >= 30) return t('Downloading model')
    return t('Starting download')
  }

  function modelIsDownloading(model: AiModelCatalogItem): boolean {
    return modelDownloadProgressValue(model) !== null
  }

  function modelRouteUsageCount(model: AiModelCatalogItem): number {
    let count = 0
    for (const route of routes.value) {
      if (route.provider_id === model.provider_id && route.model_key === model.model_key) count += 1
    }
    return count
  }

  function modelIsUsedByRoute(model: AiModelCatalogItem): boolean {
    return modelRouteUsageCount(model) > 0
  }

  function modelLabel(providerId: string, modelKey: string): string {
    const provider = providers.value.find((item) => item.provider_id === providerId)
    const model = models.value.find((item) => item.provider_id === providerId && item.model_key === modelKey)
    const providerName = provider?.display_name ?? providerId
    const modelName = model?.display_name ?? modelKey
    return `${providerName} / ${modelName}`
  }

  function providerBaseUrl(provider: AiProviderAccount): string {
    const value = provider.config.base_url
    return typeof value === 'string' ? value : ''
  }

  async function refreshLocalAuthStatus(setupId: string, notify: boolean) {
    if (fetchProviderAuthStatus.isPending.value) return
    try {
      const response = await fetchProviderAuthStatus.mutateAsync(setupId)
      activeLocalAuth.value = response
      if (response.provider) {
        selectedProviderId.value = response.provider.provider_id
        stopLocalAuthPolling()
        store.setActionMessage(t('AI provider connected'))
        return
      }
      if (notify) {
        store.setActionMessage(response.message || t('AI provider callback still waiting'))
      }
    } catch (error) {
      stopLocalAuthPolling()
      store.setError(errorMessage(error, t('AI provider callback status failed')))
    }
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
    return `${frontendConfig.apiBaseUrl.replace(/\/+$/, '')}/api/v1/ai/provider-auth/callback`
  }

  function handleProviderCallbackMessage(event: MessageEvent) {
    const data = event.data
    if (
      typeof data !== 'object' ||
      data === null ||
      !('type' in data) ||
      data.type !== 'hermes:ai-provider-connected'
    ) {
      return
    }
    const providerId = 'providerId' in data && typeof data.providerId === 'string'
      ? data.providerId
      : null
    if (providerId) {
      selectedProviderId.value = providerId
    }
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

function routeOptionValue(providerId: string, modelKey: string): string {
  return `${encodeURIComponent(providerId)}${ROUTE_OPTION_SEPARATOR}${encodeURIComponent(modelKey)}`
}

function parseRouteOptionValue(value: string): { providerId: string; modelKey: string } | null {
  const [providerId, modelKey] = value.split(ROUTE_OPTION_SEPARATOR)
  if (!providerId || !modelKey) return null
  return {
    providerId: decodeURIComponent(providerId),
    modelKey: decodeURIComponent(modelKey),
  }
}

function modelUsableForSlot(model: AiModelCatalogItem, slot: AiCapabilitySlot): boolean {
  if (!model.is_available) return false
  if (slot.requires_embedding_dimension && model.embedding_dimension !== slot.requires_embedding_dimension) {
    return false
  }
  if (slot.slot === 'embeddings') {
    return model.category === 'embeddings' || model.capabilities.includes('embeddings')
  }
  if (model.category === 'embeddings' || model.capabilities.includes('embeddings')) return false
  if (slot.slot === 'reasoning') {
    return model.category === 'reasoning' || model.capabilities.includes('reasoning')
  }
  if (slot.slot === 'summarization') {
    return model.capabilities.includes('summarization') || model.capabilities.includes('chat')
  }
  if (slot.slot === 'extraction') {
    return model.capabilities.includes('extraction') || model.capabilities.includes('chat')
  }
  return model.capabilities.includes('chat') || model.category === 'chat' || model.category === 'reasoning'
}

function mergedApiPresets(providerPresets: AiProviderPreset[]): AiProviderPreset[] {
  const presetsByKey = new Map<string, AiProviderPreset>()
  for (const preset of DEFAULT_API_PROVIDER_PRESETS) {
    presetsByKey.set(preset.provider_key, preset)
  }
  for (const preset of providerPresets) {
    if (preset.provider_kind === 'api') {
      presetsByKey.set(preset.provider_key, preset)
    }
  }
  return Array.from(presetsByKey.values())
}

function errorMessage(error: unknown, fallback: string): string {
  if (error instanceof Error) return error.message
  const message = typeof error === 'object' && error !== null && 'message' in error ? error.message : null
  return typeof message === 'string' ? message : fallback
}

function metadataBoolean(metadata: Record<string, unknown>, key: string): boolean { return metadata[key] === true }

function modelStateKey(model: AiModelCatalogItem): string { return `${model.provider_id}:${model.model_key}` }

function providerIsBuiltInOllama(providerId: string): boolean { return providerId === 'provider:built_in:ollama' }
