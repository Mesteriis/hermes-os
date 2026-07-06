<script setup lang="ts">
import { computed, ref, watch, type Ref } from 'vue'
import { useI18n } from '../../../platform/i18n'
import Dialog from '../../../shared/ui/Dialog.vue'
import Icon from '../../../shared/ui/Icon.vue'
import SearchableSelect from '../../../shared/ui/SearchableSelect.vue'
import type { AISettingsSurface } from '../queries/useAISettingsSurface'
import type { AiModelCatalogItem, AiProviderAccount } from '../types/aiControlCenter'

type WizardStep = 'connection' | 'verification' | 'models'

interface WizardStepDefinition {
  id: WizardStep
  label: string
}

const props = withDefaults(defineProps<{
  open?: boolean
  surface: AISettingsSurface
}>(), {
  open: false,
})

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const { t } = useI18n()
const step = ref<WizardStep>('connection')
const connectedProviderId = ref<string | null>(null)
const verificationStatus = ref('')
const verificationMessage = ref('')
const syncStatus = ref('')
const syncMessage = ref('')

const stepDefinitions: readonly WizardStepDefinition[] = [
  { id: 'connection', label: 'Connection parameters' },
  { id: 'verification', label: 'Verification' },
  { id: 'models', label: 'Model access' },
]

const currentStepIndex = computed(() => {
  for (let index = 0; index < stepDefinitions.length; index += 1) {
    if (stepDefinitions[index]?.id === step.value) return index
  }

  return 0
})

const connectedProvider = computed<AiProviderAccount | null>(() => {
  for (const provider of props.surface.providers.value) {
    if (provider.provider_id === connectedProviderId.value) return provider
  }

  return null
})

const connectedProviderModels = computed<AiModelCatalogItem[]>(() => {
  const providerId = connectedProviderId.value
  const models: AiModelCatalogItem[] = []
  if (!providerId) return models

  for (const model of props.surface.models.value) {
    if (model.provider_id === providerId) models.push(model)
  }

  return models
})

function eventValue(event: Event): string {
  return event.target instanceof HTMLInputElement || event.target instanceof HTMLSelectElement
    ? event.target.value
    : ''
}

function eventChecked(event: Event): boolean {
  return event.target instanceof HTMLInputElement ? event.target.checked : false
}

function updateText(target: Ref<string>, event: Event): void {
  target.value = eventValue(event)
}

function updateBoolean(target: Ref<boolean>, event: Event): void {
  target.value = eventChecked(event)
}

function selectApiProvider(providerKey: string): void {
  props.surface.handlePresetSelect(providerKey)
}

function resetWizard(): void {
  step.value = 'connection'
  connectedProviderId.value = null
  verificationStatus.value = ''
  verificationMessage.value = ''
  syncStatus.value = ''
  syncMessage.value = ''
}

function closeWizard(): void {
  emit('update:open', false)
}

function goBack(): void {
  if (step.value === 'models') {
    step.value = 'verification'
    return
  }
  if (step.value === 'verification') {
    step.value = 'connection'
  }
}

async function submitConnection(): Promise<void> {
  const provider = await props.surface.handleCreateApiProvider()
  if (!provider) return

  connectedProviderId.value = provider.provider_id
  verificationStatus.value = ''
  verificationMessage.value = ''
  step.value = 'verification'
}

async function verifyConnection(): Promise<void> {
  const provider = connectedProvider.value
  if (!provider) return

  const response = await props.surface.handleTestProvider(provider)
  if (!response) return

  verificationStatus.value = response.status
  verificationMessage.value = response.message
  if (response.status === 'ok') {
    step.value = 'models'
  }
}

async function syncModels(): Promise<void> {
  const provider = connectedProvider.value
  if (!provider) return

  const response = await props.surface.handleSyncModels(provider)
  if (!response) return

  syncStatus.value = response.status
  syncMessage.value = response.message
}

function toggleModel(model: AiModelCatalogItem, event: Event): void {
  void props.surface.handleModelAvailability(model, eventChecked(event))
}

function modelDetail(model: AiModelCatalogItem): string {
  const details = [`${model.model_key}`, model.category, model.privacy]
  if (model.context_window) {
    details.push(`${model.context_window} ctx`)
  }
  if (model.embedding_dimension) {
    details.push(`${model.embedding_dimension} dim`)
  }
  return details.join(' · ')
}

function stepTone(definition: WizardStepDefinition, index: number): string {
  if (definition.id === step.value) return 'active'
  if (index < currentStepIndex.value) return 'done'
  return 'pending'
}

watch(() => props.open, (isOpen) => {
  if (isOpen) resetWizard()
})
</script>

<template>
  <Dialog
    :open="open"
    :title="t('Connect AI provider')"
    :description="t('Add an OpenAI-compatible provider, verify it and choose which models Hermes may use.')"
    :close-label="t('Close wizard')"
    content-class="settings-ai-wizard-dialog"
    @update:open="(value) => emit('update:open', value)"
  >
    <nav class="settings-ai-wizard-steps" :aria-label="t('Connection wizard steps')">
      <span
        v-for="(definition, index) in stepDefinitions"
        :key="definition.id"
        class="settings-ai-wizard-step"
        :class="`is-${stepTone(definition, index)}`"
      >
        <Icon :icon="index < currentStepIndex ? 'tabler:check' : 'tabler:circle-number-' + (index + 1)" />
        <span>{{ t(definition.label) }}</span>
      </span>
    </nav>

    <section v-if="step === 'connection'" class="settings-ai-wizard-panel">
      <SearchableSelect
        class="settings-ai-preset-select"
        :model-value="surface.activeApiPresetKey.value"
        :options="surface.apiPresetOptions.value"
        :placeholder="t('Choose API provider')"
        :search-placeholder="t('Search providers...')"
        :empty-label="t('No providers found')"
        :clearable="false"
        :aria-label="t('AI provider')"
        @update:model-value="selectApiProvider"
      />

      <div class="settings-form-grid settings-ai-wizard-form">
        <label>
          <span>{{ t('Display name') }}</span>
          <input
            type="text"
            autocomplete="off"
            :value="surface.apiDisplayName.value"
            @input="updateText(surface.apiDisplayName, $event)"
          >
        </label>
        <label>
          <span>{{ t('Provider key') }}</span>
          <input
            type="text"
            autocomplete="off"
            :value="surface.apiProviderKey.value"
            @input="updateText(surface.apiProviderKey, $event)"
          >
        </label>
        <label class="settings-ai-wizard-form__wide">
          <span>{{ t('Base URL') }}</span>
          <input
            type="url"
            autocomplete="off"
            :value="surface.apiBaseUrl.value"
            @input="updateText(surface.apiBaseUrl, $event)"
          >
        </label>
        <label class="settings-ai-wizard-form__wide">
          <span>{{ t('API token') }}</span>
          <input
            type="password"
            autocomplete="off"
            :value="surface.apiToken.value"
            @input="updateText(surface.apiToken, $event)"
          >
        </label>
      </div>

      <label class="settings-switch">
        <input
          type="checkbox"
          :checked="surface.apiConsent.value"
          @change="updateBoolean(surface.apiConsent, $event)"
        >
        <span />
        <strong>{{ t('Allow remote private context') }}</strong>
      </label>
    </section>

    <section v-else-if="step === 'verification'" class="settings-ai-wizard-panel">
      <div class="settings-ai-wizard-status">
        <Icon icon="tabler:heartbeat" />
        <span>
          <strong>{{ connectedProvider?.display_name ?? t('Provider') }}</strong>
          <small>{{ t('Verify that consent and Host Vault API key binding are ready before syncing models.') }}</small>
        </span>
      </div>
      <div v-if="verificationMessage" class="settings-ai-wizard-result" :class="`is-${verificationStatus}`">
        <strong>{{ verificationStatus }}</strong>
        <span>{{ verificationMessage }}</span>
      </div>
    </section>

    <section v-else class="settings-ai-wizard-panel">
      <div class="settings-ai-wizard-toolbar">
        <div>
          <strong>{{ connectedProvider?.display_name ?? t('Provider') }}</strong>
          <small>{{ t('Sync the provider catalog, then choose which models are available in Hermes.') }}</small>
        </div>
        <button
          type="button"
          class="secondary-button"
          :disabled="surface.isBusy.value || !connectedProvider"
          @click="syncModels"
        >
          <Icon icon="tabler:refresh" />
          {{ t('Sync models') }}
        </button>
      </div>

      <div v-if="syncMessage" class="settings-ai-wizard-result" :class="`is-${syncStatus}`">
        <strong>{{ syncStatus }}</strong>
        <span>{{ syncMessage }}</span>
      </div>

      <div v-if="connectedProviderModels.length" class="settings-ai-wizard-model-list">
        <label
          v-for="model in connectedProviderModels"
          :key="`${model.provider_id}:${model.model_key}`"
          class="settings-ai-wizard-model"
        >
          <input
            type="checkbox"
            :checked="model.is_available"
            :disabled="surface.isBusy.value"
            @change="toggleModel(model, $event)"
          >
          <span>
            <strong>{{ model.display_name }}</strong>
            <small>{{ modelDetail(model) }}</small>
          </span>
        </label>
      </div>

      <div v-else class="settings-empty-state">
        <Icon icon="tabler:list-search" />
        <strong>{{ t('No models synced') }}</strong>
      </div>
    </section>

    <template #footer>
      <button type="button" class="secondary-button" @click="closeWizard">
        {{ step === 'models' ? t('Done') : t('Cancel') }}
      </button>
      <button
        v-if="step !== 'connection'"
        type="button"
        class="secondary-button"
        :disabled="surface.isBusy.value"
        @click="goBack"
      >
        {{ t('Back') }}
      </button>
      <button
        v-if="step === 'connection'"
        type="button"
        class="primary-button"
        :disabled="surface.isBusy.value"
        @click="submitConnection"
      >
        <Icon icon="tabler:arrow-right" />
        {{ t('Continue') }}
      </button>
      <button
        v-else-if="step === 'verification'"
        type="button"
        class="primary-button"
        :disabled="surface.isBusy.value || !connectedProvider"
        @click="verifyConnection"
      >
        <Icon icon="tabler:heartbeat" />
        {{ t('Verify connection') }}
      </button>
    </template>
  </Dialog>
</template>
