<script setup lang="ts">
import { computed } from 'vue'
import Button from '../../../shared/ui/primitives/Button.vue'
import Icon from '../../../shared/ui/Icon.vue'
import SearchableSelect from '../../../shared/ui/SearchableSelect.vue'
import Steps from '../../../shared/ui/Steps.vue'
import type { AISettingsSurface } from '../queries/useAISettingsSurface'
import { modelDetail } from './aiModelCatalogPresentation'
import { useAIProviderConnectionWizardController } from '../queries/useAIProviderConnectionWizardController'

const props = withDefaults(defineProps<{
  open?: boolean
  surface: AISettingsSurface
}>(), {
  open: false,
})

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const wizardOpen = computed({
  get: () => props.open,
  set: (value: boolean) => {
    emit('update:open', value)
  },
})

const controller = useAIProviderConnectionWizardController({
  open: computed(() => props.open),
  surface: props.surface,
  close: () => emit('update:open', false),
})
const {
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
  handleUpdateApiProviderDisplayName,
  handleUpdateApiProviderKey,
  handleUpdateApiBaseUrl,
  handleUpdateApiToken,
  handleUpdateApiConsent,
  handleSelectApiProvider,
  handleGoBack,
  handleGoNext,
  handleSyncModels,
  handleToggleModelFromEvent,
  handleSetCurrentStep,
} = controller
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
          @update:model-value="handleSelectApiProvider"
        />

        <div class="settings-form-grid settings-ai-wizard-form">
          <label>
            <span>{{ t('Название') }}</span>
            <input
              type="text"
              autocomplete="off"
              :value="surface.apiDisplayName.value"
            @input="handleUpdateApiProviderDisplayName"
            >
          </label>
          <label>
            <span>{{ t('Ключ провайдера') }}</span>
            <input
              type="text"
              autocomplete="off"
              :value="surface.apiProviderKey.value"
              @input="handleUpdateApiProviderKey"
            >
          </label>
          <label class="settings-ai-wizard-form__wide">
            <span>{{ t('URL API') }}</span>
            <input
              type="url"
              autocomplete="off"
              :value="surface.apiBaseUrl.value"
              @input="handleUpdateApiBaseUrl"
            >
          </label>
          <label class="settings-ai-wizard-form__wide">
            <span>{{ t('API-токен') }}</span>
            <input
              type="password"
              autocomplete="off"
              :value="surface.apiToken.value"
              @input="handleUpdateApiToken"
            >
          </label>
        </div>

        <label class="settings-switch">
          <input
            type="checkbox"
            :checked="surface.apiConsent.value"
            @change="handleUpdateApiConsent"
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
            <small>{{ connectedModels.length }} {{ t('моделей') }}</small>
          </div>
          <Button
            variant="secondary"
            icon="tabler:refresh"
            :disabled="surface.isBusy.value || !connectedProvider"
            @click="handleSyncModels"
          >
            {{ t('Синхронизировать модели') }}
          </Button>
        </div>

        <div v-if="syncMessage" class="settings-ai-wizard-result" :class="`is-${syncStatus}`">
          <strong>{{ syncStatus }}</strong>
          <span>{{ syncMessage }}</span>
        </div>

        <div v-if="connectedModels.length" class="settings-ai-wizard-model-list">
          <label
            v-for="model in connectedModels"
            :key="`${model.provider_id}:${model.model_key}`"
            class="settings-ai-wizard-model"
          >
            <input
              type="checkbox"
              :checked="model.is_available"
              :disabled="surface.isBusy.value"
              @change="handleToggleModelFromEvent(model, $event)"
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
          @click="handleGoBack"
        />
        <nav class="hermes-steps__dots" :aria-label="t('Шаги мастера')">
          <button
            v-for="(item, index) in wizardSteps"
            :key="item.title"
            class="hermes-steps__dot"
            :class="{ 'hermes-steps__dot--active': index + 1 === currentStep }"
            type="button"
            :aria-current="index + 1 === currentStep ? 'step' : undefined"
            :aria-label="item.title"
            :disabled="surface.isBusy.value"
            @click="handleSetCurrentStep(index + 1)"
          />
        </nav>
        <Button
          class="hermes-steps__dock-next"
          :loading="surface.isBusy.value"
          @click="handleGoNext"
        >
          {{ nextLabel }}
        </Button>
      </div>
    </template>
  </Steps>
</template>
