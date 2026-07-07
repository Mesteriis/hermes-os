<script setup lang="ts">
import { computed, ref, watch, type Ref } from 'vue'
import { useI18n } from '../../../platform/i18n'
import Button from '../../../shared/ui/Button.vue'
import Icon from '../../../shared/ui/Icon.vue'
import SearchableSelect from '../../../shared/ui/SearchableSelect.vue'
import Steps from '../../../shared/ui/Steps.vue'
import type { StepsItem } from '../../../shared/ui/Steps.types'
import type { AISettingsSurface } from '../queries/useAISettingsSurface'
import type { AiModelCatalogItem, AiProviderAccount } from '../types/aiControlCenter'

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

const wizardOpen = computed({
  get: () => props.open,
  set: (value: boolean) => {
    emit('update:open', value)
  },
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

const nextLabel = computed(() => {
  if (currentStep.value === 1) return t('Подключить')
  if (currentStep.value === 2) return t('Проверить')
  return t('Готово')
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
  currentStep.value = 1
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
  if (currentStep.value > 1) {
    currentStep.value -= 1
  }
}

async function goNext(): Promise<void> {
  if (currentStep.value === 1) {
    await submitConnection()
    return
  }
  if (currentStep.value === 2) {
    await verifyConnection()
    return
  }
  closeWizard()
}

async function submitConnection(): Promise<void> {
  const provider = await props.surface.handleCreateApiProvider()
  if (!provider) return

  connectedProviderId.value = provider.provider_id
  verificationStatus.value = ''
  verificationMessage.value = ''
  currentStep.value = 2
}

async function verifyConnection(): Promise<void> {
  const provider = connectedProvider.value
  if (!provider) return

  const response = await props.surface.handleTestProvider(provider)
  if (!response) return

  verificationStatus.value = response.status
  verificationMessage.value = response.message
  if (response.status === 'ok') {
    currentStep.value = 3
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

watch(() => props.open, (isOpen) => {
  if (isOpen) resetWizard()
})
</script>

<template>
  <Steps
    v-model:open="wizardOpen"
    v-model:step="currentStep"
    :title="t('Подключение AI провайдера')"
    :description="connectedProvider?.display_name ?? t('AI провайдер')"
    :steps="wizardSteps"
    :step-count="3"
    :busy="surface.isBusy.value"
    :next-label="nextLabel"
    :finish-label="t('Готово')"
    :previous-label="t('Назад')"
    :steps-label="t('Шаги мастера')"
    size="lg"
  >
    <template #step-1>
      <section class="settings-ai-wizard-panel">
        <SearchableSelect
          class="settings-ai-preset-select"
          :model-value="surface.activeApiPresetKey.value"
          :options="surface.apiPresetOptions.value"
          :placeholder="t('Выберите провайдера')"
          :search-placeholder="t('Поиск')"
          :empty-label="t('Ничего не найдено')"
          :clearable="false"
          :aria-label="t('AI провайдер')"
          @update:model-value="selectApiProvider"
        />

        <div class="settings-form-grid settings-ai-wizard-form">
          <label>
            <span>{{ t('Название') }}</span>
            <input
              type="text"
              autocomplete="off"
              :value="surface.apiDisplayName.value"
              @input="updateText(surface.apiDisplayName, $event)"
            >
          </label>
          <label>
            <span>{{ t('Ключ провайдера') }}</span>
            <input
              type="text"
              autocomplete="off"
              :value="surface.apiProviderKey.value"
              @input="updateText(surface.apiProviderKey, $event)"
            >
          </label>
          <label class="settings-ai-wizard-form__wide">
            <span>{{ t('URL API') }}</span>
            <input
              type="url"
              autocomplete="off"
              :value="surface.apiBaseUrl.value"
              @input="updateText(surface.apiBaseUrl, $event)"
            >
          </label>
          <label class="settings-ai-wizard-form__wide">
            <span>{{ t('API-токен') }}</span>
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
          <strong>{{ t('Разрешить приватный контекст') }}</strong>
        </label>
      </section>
    </template>

    <template #step-2>
      <section class="settings-ai-wizard-panel">
        <div class="settings-ai-wizard-status">
          <Icon icon="tabler:heartbeat" />
          <span>
            <strong>{{ connectedProvider?.display_name ?? t('Провайдер') }}</strong>
            <small>{{ verificationMessage || t('Готов к проверке') }}</small>
          </span>
        </div>
        <div v-if="verificationMessage" class="settings-ai-wizard-result" :class="`is-${verificationStatus}`">
          <strong>{{ verificationStatus }}</strong>
          <span>{{ verificationMessage }}</span>
        </div>
      </section>
    </template>

    <template #step-3>
      <section class="settings-ai-wizard-panel">
        <div class="settings-ai-wizard-toolbar">
          <div>
            <strong>{{ connectedProvider?.display_name ?? t('Провайдер') }}</strong>
            <small>{{ connectedProviderModels.length }} {{ t('моделей') }}</small>
          </div>
          <Button
            variant="secondary"
            icon="tabler:refresh"
            :disabled="surface.isBusy.value || !connectedProvider"
            @click="syncModels"
          >
            {{ t('Синхронизировать модели') }}
          </Button>
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
          <strong>{{ t('Моделей пока нет') }}</strong>
        </div>
      </section>
    </template>

    <template #footer>
      <div class="hermes-steps__dock">
        <Button
          class="hermes-steps__dock-back"
          variant="ghost"
          icon="tabler:arrow-left"
          :aria-label="t('Назад')"
          :disabled="currentStep === 1 || surface.isBusy.value"
          @click="goBack"
        />
        <nav class="hermes-steps__dots" :aria-label="t('Шаги мастера')">
          <button
            v-for="item in wizardSteps"
            :key="item.title"
            class="hermes-steps__dot"
            :class="{ 'hermes-steps__dot--active': wizardSteps.indexOf(item) + 1 === currentStep }"
            type="button"
            :aria-current="wizardSteps.indexOf(item) + 1 === currentStep ? 'step' : undefined"
            :aria-label="item.title"
            :disabled="surface.isBusy.value"
            @click="currentStep = wizardSteps.indexOf(item) + 1"
          />
        </nav>
        <Button
          class="hermes-steps__dock-next"
          :loading="surface.isBusy.value"
          @click="goNext"
        >
          {{ nextLabel }}
        </Button>
      </div>
    </template>
  </Steps>
</template>
