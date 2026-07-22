import { computed, ref, watch, type Ref } from 'vue'
import { useI18n } from '../../../platform/i18n'
import type { StepsItem } from '../../../shared/ui/Steps.types'
import type { AISettingsSurface } from './useAISettingsSurface'
import type { AiModelCatalogItem } from '../types/aiControlCenter'
import {
  aiProviderWizardVerificationSucceeded,
  connectedProviderModels,
  findConnectedProvider,
  wizardNextLabel,
} from '../components/aiProviderConnectionWizardPresentation'

export function useAIProviderConnectionWizardController(options: {
  open: Ref<boolean>
  surface: AISettingsSurface
  close: () => void
}) {
  const { t } = useI18n()
  const currentStep = ref(1)
  const connectedProviderId = ref<string | null>(null)
  const verificationStatus = ref('')
  const verificationMessage = ref('')
  const syncStatus = ref('')
  const syncMessage = ref('')
  const wizardSteps = computed<StepsItem[]>(() => [
    { title: t('Параметры API') },
    { title: t('Проверка') },
    { title: t('Модели Hermes') },
  ])
  const connectedProvider = computed(() => findConnectedProvider(
    options.surface.providers.value,
    connectedProviderId.value,
  ))
  const connectedModels = computed(() => connectedProviderModels(
    options.surface.models.value,
    connectedProviderId.value,
  ))
  const nextLabel = computed(() => wizardNextLabel(currentStep.value, t))

  function handleSelectApiProvider(providerKey: string): void {
    options.surface.handlePresetSelect(providerKey)
  }

  function eventValue(event: Event): string {
    return event.target instanceof HTMLInputElement || event.target instanceof HTMLSelectElement
      ? event.target.value
      : ''
  }

  function eventChecked(event: Event): boolean {
    return event.target instanceof HTMLInputElement ? event.target.checked : false
  }

  function handleUpdateApiProviderDisplayName(event: Event): void {
    options.surface.apiDisplayName.value = eventValue(event)
  }

  function handleUpdateApiProviderKey(event: Event): void {
    options.surface.apiProviderKey.value = eventValue(event)
  }

  function handleUpdateApiBaseUrl(event: Event): void {
    options.surface.apiBaseUrl.value = eventValue(event)
  }

  function handleUpdateApiToken(event: Event): void {
    options.surface.apiToken.value = eventValue(event)
  }

  function handleUpdateApiConsent(event: Event): void {
    options.surface.apiConsent.value = eventChecked(event)
  }

  function resetWizard(): void {
    currentStep.value = 1
    connectedProviderId.value = null
    verificationStatus.value = ''
    verificationMessage.value = ''
    syncStatus.value = ''
    syncMessage.value = ''
  }

  function handleCloseWizard(): void {
    options.close()
  }

  function handleGoBack(): void {
    if (currentStep.value > 1) currentStep.value -= 1
  }

  function handleSetCurrentStep(step: number): void {
    if (step < 1 || step > wizardSteps.value.length) return
    currentStep.value = step
  }

  async function submitConnection(): Promise<void> {
    const provider = await options.surface.handleCreateApiProvider()
    if (!provider) return
    connectedProviderId.value = provider.provider_id
    verificationStatus.value = ''
    verificationMessage.value = ''
    currentStep.value = 2
  }

  async function verifyConnection(): Promise<void> {
    const provider = connectedProvider.value
    if (!provider) return
    const response = await options.surface.handleTestProvider(provider)
    if (!response) return
    verificationStatus.value = response.status
    verificationMessage.value = response.message
    if (aiProviderWizardVerificationSucceeded(response.status)) currentStep.value = 3
  }

  async function handleSyncModels(): Promise<void> {
    const provider = connectedProvider.value
    if (!provider) return
    const response = await options.surface.handleSyncModels(provider)
    if (!response) return
    syncStatus.value = response.status
    syncMessage.value = response.message
  }

  function handleToggleModel(model: AiModelCatalogItem, available: boolean): void {
    void options.surface.handleModelAvailability(model, available)
  }

  function handleToggleModelFromEvent(model: AiModelCatalogItem, event: Event): void {
    handleToggleModel(model, eventChecked(event))
  }

  async function handleGoNext(): Promise<void> {
    if (currentStep.value === 1) {
      await submitConnection()
      return
    }
    if (currentStep.value === 2) {
      await verifyConnection()
      return
    }
    handleCloseWizard()
  }

  watch(options.open, (isOpen) => {
    if (isOpen) resetWizard()
  })

  return {
    t,
    currentStep,
    verificationStatus,
    verificationMessage,
    syncStatus,
    syncMessage,
    wizardSteps,
    connectedProvider,
    connectedModels,
    nextLabel,
    handleSelectApiProvider,
    handleUpdateApiProviderDisplayName,
    handleUpdateApiProviderKey,
    handleUpdateApiBaseUrl,
    handleUpdateApiToken,
    handleUpdateApiConsent,
    resetWizard,
    handleCloseWizard,
    handleSetCurrentStep,
    handleGoBack,
    handleGoNext,
    handleSyncModels,
    handleToggleModelFromEvent,
    handleToggleModel,
  }
}
