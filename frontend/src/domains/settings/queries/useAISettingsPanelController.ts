import { computed, ref } from 'vue'
import type { AISettingsSurface } from './useAISettingsSurface'
import type { AiModelRouteRow } from './aiSettingsPresentation'
import type { AiProviderAccount } from '../types/aiControlCenter'
import {
  aiProviderListSelection,
  buildAiSettingsTabs,
  buildAiProviderListGroups,
  buildAiSelectedProviderRows,
  countProviderRoutes,
  type AiDetailRow,
  type AiProviderListGroup,
  type AiProviderListItem,
} from '../components/aiSettingsPanelPresentation'
import { countAvailableModels } from '../components/aiModelCatalogPresentation'
import { useI18n } from '../../../platform/i18n'

export function useAISettingsPanelController(options: {
  surface: AISettingsSurface
}) {
  type AiSettingsTab = 'providers' | 'models' | 'routes' | 'stats'

  const activeTab = ref<AiSettingsTab>('providers')
  const isProviderWizardOpen = ref(false)
  const isModelPickerOpen = ref(false)

  const { t } = useI18n()
  const tabs = computed(() => buildAiSettingsTabs({
    providers: options.surface.providers.value.length,
    models: options.surface.models.value.length,
    routes: options.surface.routeRows.value.length,
    stats: options.surface.usageStats.value?.totals.request_count ?? 0,
  }, t))

  const providerListGroups = computed<AiProviderListGroup[]>(() => buildAiProviderListGroups(
    options.surface.providers.value,
    options.surface.localPresets.value,
    options.surface.models.value,
    options.surface.providerForPreset,
    t
  ))

  const selectedProviderRows = computed<AiDetailRow[]>(() => buildAiSelectedProviderRows(
    options.surface.selectedProvider.value,
    options.surface.providerBaseUrl,
    t
  ))

  const selectedAvailableModelCount = computed(() => countAvailableModels(options.surface.selectedProviderModels.value))
  const selectedRouteCount = computed(() => countProviderRoutes(
    options.surface.selectedProvider.value?.provider_id ?? null,
    options.surface.routes.value
  ))

  function eventValue(event: Event): string {
    return event.target instanceof HTMLInputElement || event.target instanceof HTMLSelectElement
      ? event.target.value
      : ''
  }

  function eventChecked(event: Event): boolean {
    return event.target instanceof HTMLInputElement ? event.target.checked : false
  }

  function handleProviderToggle(provider: AiProviderAccount, event: Event): void {
    void options.surface.handleToggleProvider(provider, eventChecked(event))
  }

  function handleProviderConsent(provider: AiProviderAccount, event: Event): void {
    void options.surface.handleGrantConsent(provider, eventChecked(event))
  }

  function handleProviderTest(provider: AiProviderAccount): void {
    void options.surface.handleTestProvider(provider)
  }

  function handleSyncProviderModels(provider: AiProviderAccount): void {
    void options.surface.handleSyncModels(provider)
  }

  function handleTestSelectedProvider(): void {
    const provider = options.surface.selectedProvider.value
    if (!provider) return
    handleProviderTest(provider)
  }

  function handleSyncSelectedProviderModels(): void {
    const provider = options.surface.selectedProvider.value
    if (!provider) return
    handleSyncProviderModels(provider)
  }

  function handleToggleSelectedProvider(event: Event): void {
    const provider = options.surface.selectedProvider.value
    if (!provider) return
    handleProviderToggle(provider, event)
  }

  function handleUpdateSelectedProviderConsent(event: Event): void {
    const provider = options.surface.selectedProvider.value
    if (!provider) return
    handleProviderConsent(provider, event)
  }

  function handleUpdateRoute(row: AiModelRouteRow, event: Event): void {
    void options.surface.handleRouteSelection(row.slot.slot, eventValue(event))
  }

  function handleRefreshModelRoutes(): void {
    void options.surface.handleRefreshModelRoutes()
  }

  function handleSelectProviderListItem(item: AiProviderListItem): void {
    const selection = aiProviderListSelection(item)
    if (!selection) return

    if (selection.kind === 'provider') {
      options.surface.selectProvider(selection.providerId)
      return
    }

    void options.surface.handleConnectLocalPreset(selection.preset)
  }

  function handleRefreshLocalAuth(): void {
    void options.surface.handleRefreshLocalAuth()
  }

  function handleOpenLocalAuthCallback(): void {
    void options.surface.handleOpenLocalAuthCallback()
  }

  function handleSetActiveTab(tabId: AiSettingsTab): void {
    activeTab.value = tabId
  }

  function handleOpenProviderWizard(): void {
    isProviderWizardOpen.value = true
  }

  function handleProviderWizardOpen(value: boolean): void {
    if (value) {
      handleOpenProviderWizard()
      return
    }

    handleCloseProviderWizard()
  }

  function handleCloseProviderWizard(): void {
    isProviderWizardOpen.value = false
  }

  function handleOpenModelPicker(): void {
    isModelPickerOpen.value = true
  }

  function handleSetModelPickerOpen(open: boolean): void {
    isModelPickerOpen.value = open
  }

  return {
    t,
    activeTab,
    isProviderWizardOpen,
    isModelPickerOpen,
    tabs,
    providerListGroups,
    selectedProviderRows,
    selectedAvailableModelCount,
    selectedRouteCount,
    handleProviderToggle: handleProviderToggle,
    handleProviderConsent: handleProviderConsent,
    handleTestSelectedProvider,
    handleSyncSelectedProviderModels,
    handleToggleSelectedProvider,
    handleUpdateSelectedProviderConsent,
    handleProviderTest,
    handleSyncProviderModels,
    handleUpdateRoute,
    handleRefreshModelRoutes,
    handleSelectProviderListItem,
    handleRefreshLocalAuth,
    handleOpenLocalAuthCallback,
    handleSetActiveTab,
    handleProviderWizardOpen,
    handleOpenProviderWizard,
    handleCloseProviderWizard,
    handleOpenModelPicker,
    handleSetModelPickerOpen,
  }
}
