import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { loadFrontendConfig } from '../../../platform/config/env'
import { useI18n } from '../../../platform/i18n'
import { useSettingsStore } from '../stores/settings'
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
  useCreateAiProviderMutation,
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

const DEFAULT_OPENAI_BASE_URL = 'https://api.openai.com/v1'
const ROUTE_OPTION_SEPARATOR = '|'
const LOCAL_AUTH_POLL_INTERVAL_MS = 2500

type AiProviderAuthFlow = AiProviderAuthStartResponse | AiProviderAuthStatusResponse

const DEFAULT_API_PROVIDER_PRESETS: AiProviderPreset[] = [
  apiProviderPreset('raw', 'Raw OpenAI-compatible API', null, [
    'chat',
    'reasoning',
    'summarization',
    'embeddings',
    'extraction',
  ]),
  apiProviderPreset('openai', 'OpenAI', 'https://api.openai.com/v1', [
    'chat',
    'reasoning',
    'embeddings',
  ]),
  apiProviderPreset('deepseek', 'DeepSeek', 'https://api.deepseek.com/v1', [
    'chat',
    'reasoning',
  ]),
  apiProviderPreset('openrouter', 'OpenRouter', 'https://openrouter.ai/api/v1', [
    'chat',
    'reasoning',
    'routing',
  ]),
  apiProviderPreset('groq', 'Groq', 'https://api.groq.com/openai/v1', [
    'chat',
    'reasoning',
  ]),
  apiProviderPreset('together', 'Together AI', 'https://api.together.xyz/v1', [
    'chat',
    'reasoning',
    'embeddings',
  ]),
  apiProviderPreset('fireworks', 'Fireworks AI', 'https://api.fireworks.ai/inference/v1', [
    'chat',
    'reasoning',
    'embeddings',
  ]),
  apiProviderPreset('mistral', 'Mistral AI', 'https://api.mistral.ai/v1', [
    'chat',
    'reasoning',
    'embeddings',
  ]),
  apiProviderPreset('xai', 'xAI', 'https://api.x.ai/v1', [
    'chat',
    'reasoning',
  ]),
  apiProviderPreset(
    'gemini-openai',
    'Google Gemini OpenAI-compatible',
    'https://generativelanguage.googleapis.com/v1beta/openai',
    ['chat', 'reasoning', 'embeddings']
  ),
  apiProviderPreset('perplexity', 'Perplexity', 'https://api.perplexity.ai', [
    'chat',
    'reasoning',
  ]),
  apiProviderPreset('nvidia-nim', 'NVIDIA NIM', 'https://integrate.api.nvidia.com/v1', [
    'chat',
    'reasoning',
    'embeddings',
  ]),
  apiProviderPreset('cerebras', 'Cerebras', 'https://api.cerebras.ai/v1', [
    'chat',
    'reasoning',
  ]),
  apiProviderPreset('lm-studio', 'LM Studio', 'http://127.0.0.1:1234/v1', [
    'chat',
    'local_runtime',
  ]),
  apiProviderPreset('vllm-local', 'vLLM local', 'http://127.0.0.1:8000/v1', [
    'chat',
    'local_runtime',
  ]),
  apiProviderPreset('omniroute', 'OmniRoute', 'https://ai.sh-inc.ru/v1', [
    'chat',
    'embeddings',
    'routing',
  ]),
]

const SLOT_LABELS: Record<string, string> = {
  default_chat: 'Translation and general chat',
  reasoning: 'Reasoning',
  summarization: 'Summaries',
  mail_intelligence: 'Mail analysis',
  reply_draft: 'Reply drafts',
  extraction: 'Extraction and categorization',
  embeddings: 'Embeddings',
  meeting_prep: 'Meeting preparation',
}

const SLOT_DESCRIPTIONS: Record<string, string> = {
  default_chat: 'Default text generation, translation and short assistant actions.',
  reasoning: 'Deep reasoning and multi-step analysis.',
  summarization: 'Summaries for messages, documents and context packs.',
  mail_intelligence: 'Mail triage, sentiment, urgency and signal extraction.',
  reply_draft: 'Drafting replies for mail and messaging flows.',
  extraction: 'Entity extraction, classification and categorization.',
  embeddings: 'Semantic index embeddings. Dimension must match backend requirements.',
  meeting_prep: 'Meeting prep, agendas and follow-up intelligence.',
}

export function useAISettingsSurface() {
  const { t } = useI18n()
  const store = useSettingsStore()
  const frontendConfig = loadFrontendConfig()
  const overviewQuery = useAiSettingsOverviewQuery()
  const createProvider = useCreateAiProviderMutation()
  const updateProvider = useUpdateAiProviderMutation()
  const syncModels = useSyncAiProviderModelsMutation()
  const testProvider = useTestAiProviderMutation()
  const updateModelAvailability = useUpdateAiModelAvailabilityMutation()
  const updateConsent = useUpdateAiProviderConsentMutation()
  const updateRoute = useUpdateAiModelRouteMutation()
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
  let localAuthPollId: number | null = null

  const overview = computed(() => overviewQuery.data.value ?? null)
  const providers = computed(() => overview.value?.providers ?? [])
  const models = computed(() => overview.value?.models ?? [])
  const routes = computed(() => overview.value?.routes ?? [])
  const capabilitySlots = computed(() => overview.value?.capability_slots ?? [])
  const providerPresets = computed(() => overview.value?.provider_presets ?? [])
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

  async function handleRouteSelection(slot: string, value: string) {
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
  })

  const isBusy = computed(() =>
    createProvider.isPending.value ||
    updateProvider.isPending.value ||
    syncModels.isPending.value ||
    testProvider.isPending.value ||
    updateModelAvailability.isPending.value ||
    updateConsent.isPending.value ||
    updateRoute.isPending.value ||
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
    handleModelAvailability,
    handleOpenLocalAuthCallback,
    handlePresetSelect,
    handleRefreshLocalAuth,
    handleRefreshModelRoutes,
    handleRouteSelection,
    handleSyncModels,
    handleTestProvider,
    handleToggleProvider,
    isBusy,
    isLoading: overviewQuery.isLoading,
    localPresets,
    models,
    overview,
    providers,
    providerBaseUrl,
    providerForPreset,
    routeRows,
    routes,
    selectProvider,
    selectedProvider,
    selectedProviderId,
    selectedProviderModels,
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

function apiProviderPreset(
  providerKey: string,
  displayName: string,
  baseUrl: string | null,
  capabilities: string[]
): AiProviderPreset {
  return {
    provider_kind: 'api',
    provider_key: providerKey,
    display_name: displayName,
    privacy: 'remote',
    base_url: baseUrl,
    command_preset: null,
    capabilities,
  }
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
  if (
    typeof error === 'object' &&
    error !== null &&
    'message' in error &&
    typeof error.message === 'string'
  ) {
    return error.message
  }
  return fallback
}
