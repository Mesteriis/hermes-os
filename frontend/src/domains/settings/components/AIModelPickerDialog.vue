<script setup lang="ts">
import Dialog from '../../../shared/ui/Dialog.vue'
import Icon from '../../../shared/ui/Icon.vue'
import type { AISettingsSurface } from '../queries/useAISettingsSurface'
import {
  modelCapabilityBadges,
  modelDetail,
  modelRuntimeFacts,
} from './aiModelCatalogPresentation'
import { useAIModelPickerController } from '../queries/useAIModelPickerController'

const props = defineProps<{
  open: boolean
  surface: AISettingsSurface
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const {
  selectedAvailableModelCount,
  modelPickerDescription,
  t,
  modelProgress,
  handleDownloadModel,
  handleSyncModels,
  handleToggleModelAvailability,
  modelProgressLabel,
} = useAIModelPickerController({
  surface: props.surface,
})

function handleUpdateOpen(value: boolean): void {
  emit('update:open', value)
}

function handleCloseDialog(): void {
  emit('update:open', false)
}
</script>

<template>
  <Dialog
    :open="open"
    :title="t('Choose models for Hermes')"
    :description="modelPickerDescription"
    :close-label="t('Close model picker')"
    content-class="settings-ai-model-picker-dialog"
    @update:open="handleUpdateOpen"
  >
    <section class="settings-ai-model-picker">
      <header
        v-if="surface.selectedProvider.value"
        class="settings-ai-model-picker__toolbar"
      >
        <div>
          <span>{{ t('Models enabled in Hermes') }}</span>
          <strong>
            {{ selectedAvailableModelCount }}/{{ surface.selectedProviderModels.value.length }}
            {{ t('available') }}
          </strong>
          <small>
            {{ t('Use the checkboxes to decide which synced models Hermes can route actions to.') }}
          </small>
        </div>
        <button
          type="button"
          class="secondary-button"
          :disabled="surface.isBusy.value"
          @click="handleSyncModels"
        >
          <Icon icon="tabler:refresh" />
          {{ t('Sync models') }}
        </button>
      </header>

      <div
        v-if="surface.selectedProviderModels.value.length"
        class="settings-ai-model-picker__list"
      >
        <article
          v-for="model in surface.selectedProviderModels.value"
          :key="`${model.provider_id}:${model.model_key}`"
          class="settings-ai-model-card"
          :class="{ 'is-unavailable': !model.is_available }"
        >
          <div class="settings-ai-model-checkbox">
            <template v-if="surface.modelIsDownloading(model)">
              <span>
                <strong>{{ t('Download queued') }}</strong>
                <small>{{ t('Hermes is pulling the model before it can be used in routing.') }}</small>
              </span>
            </template>
            <template v-else-if="!model.is_available && surface.modelRequiresDownload(model)">
              <span>
                <strong>{{ t('Not downloaded') }}</strong>
                <small>{{ t('Download this local model before Hermes can route work to it.') }}</small>
              </span>
              <button
                type="button"
                class="secondary-button"
                :disabled="surface.isBusy.value"
                @click="handleDownloadModel(model)"
              >
                <Icon icon="tabler:download" />
                {{ t('Download') }}
              </button>
            </template>
            <template v-else>
              <label class="settings-ai-model-checkbox">
                <input
                  type="checkbox"
                  :checked="model.is_available"
                  :disabled="surface.isBusy.value"
                  @change="handleToggleModelAvailability(model, $event)"
                >
                <span>
                  <strong>
                    {{
                      surface.modelIsUsedByRoute(model)
                        ? t('Used by route')
                        : model.is_available
                          ? t('Available')
                          : t('Not available')
                    }}
                  </strong>
                  <small>
                    {{
                      model.is_available
                        ? t('Available models can be selected in AI Hub routing.')
                        : t('Unavailable models stay out of route selection until enabled again.')
                    }}
                  </small>
                </span>
              </label>
            </template>
          </div>

          <div class="settings-ai-model-card__body">
            <div class="settings-ai-model-card__title">
              <strong>{{ model.display_name }}</strong>
              <small>{{ modelDetail(model) }}</small>
            </div>
            <div v-if="surface.modelIsDownloading(model)" class="settings-ai-model-progress">
              <progress
                class="settings-ai-model-progress-bar"
                max="100"
                :value="modelProgress(model)"
                :aria-label="modelProgressLabel(model)"
              />
              <small>{{ modelProgressLabel(model) }}</small>
            </div>
            <div class="settings-ai-capability-row">
              <span
                v-for="capability in modelCapabilityBadges(model, t)"
                :key="`${model.provider_id}:${model.model_key}:picker:${capability.key}`"
                :class="{ 'is-muted': capability.muted }"
              >
                {{ capability.label }}
              </span>
            </div>
            <div
              v-if="modelRuntimeFacts(model, t).length"
              class="settings-ai-model-facts"
            >
              <span
                v-for="fact in modelRuntimeFacts(model, t)"
                :key="`${model.provider_id}:${model.model_key}:picker:${fact.key}`"
              >
                <strong>{{ fact.label }}</strong>
                {{ fact.value }}
              </span>
            </div>
          </div>
        </article>
      </div>

      <div v-else class="settings-empty-state">
        <Icon icon="tabler:list-search" />
        <strong>{{ t('No models synced') }}</strong>
        <span>{{ t('Sync provider models and then enable the models Hermes should use.') }}</span>
      </div>
    </section>

    <template #footer>
      <button
        type="button"
        class="primary-button"
        @click="handleCloseDialog"
      >
        {{ t('Done') }}
      </button>
    </template>
  </Dialog>
</template>
