<script setup lang="ts">
import Icon from '../../../shared/ui/Icon.vue'
import SearchInput from '../../../shared/ui/SearchInput.vue'
import type { AISettingsSurface } from '../queries/useAISettingsSurface'
import {
  modelCapabilityBadges,
  modelDetail,
  modelRuntimeFacts,
} from './aiModelCatalogPresentation'
import { providerIcon, providerIconTone } from './aiModelCatalogPanelPresentation'
import { useAIModelCatalogController } from '../queries/useAIModelCatalogController'

const props = defineProps<{
  surface: AISettingsSurface
}>()

const {
  t,
  providerModelGroups,
  selectedModelGroup,
  selectedModelGroupModels,
  handleAvailableModelsFilterChange,
  handleSyncModels,
  handleSelectModelProvider,
  handleDownloadModel,
  handleToggleModelAvailability,
  modelProgress,
  modelProgressLabel,
  modelCatalogSearch,
  showAvailableModelsOnly,
} = useAIModelCatalogController({
  surface: props.surface,
})
</script>

<template>
  <div v-if="providerModelGroups.length" class="settings-ai-model-catalog">
    <aside class="settings-ai-model-provider-tabs" :aria-label="t('Model providers')">
        <button
          v-for="group in providerModelGroups"
          :key="group.provider.provider_id"
          type="button"
          class="settings-ai-model-provider-tab"
          :class="{ active: selectedModelGroup?.provider.provider_id === group.provider.provider_id }"
          @click="handleSelectModelProvider(group.provider.provider_id)"
      >
        <i
          class="settings-provider-icon"
          :class="providerIconTone(group.provider.provider_kind, group.provider.provider_key)"
          aria-hidden="true"
        >
          <Icon :icon="providerIcon(group.provider.provider_kind, group.provider.provider_key)" />
        </i>
        <span>
          <strong>{{ group.provider.display_name }}</strong>
          <small>{{ group.provider.provider_key }} · {{ group.provider.status }}</small>
        </span>
        <em>{{ group.availableCount }}/{{ group.models.length }}</em>
      </button>
    </aside>

    <section v-if="selectedModelGroup" class="settings-ai-model-browser">
      <header class="settings-ai-model-browser__header">
        <div>
          <span>{{ selectedModelGroup.provider.provider_kind }}</span>
          <strong>{{ selectedModelGroup.provider.display_name }}</strong>
          <small>
            {{ selectedModelGroup.availableCount }}/{{ selectedModelGroup.models.length }}
            {{ t('available') }}
          </small>
        </div>
        <div class="settings-ai-model-browser__actions">
          <div class="settings-ai-model-search">
            <SearchInput
              v-model="modelCatalogSearch"
              :aria-label="t('Search models')"
              :clear-label="t('Clear model search')"
              :placeholder="t('Search models')"
            />
          </div>
          <label
            class="settings-ai-model-availability-filter"
            :title="t('Show only available models')"
          >
            <input
              type="checkbox"
              :aria-label="t('Show only available models')"
              :checked="showAvailableModelsOnly"
              @change="handleAvailableModelsFilterChange"
            >
          </label>
          <button
            type="button"
            class="icon-button"
            :title="t('Sync models')"
            :aria-label="t('Sync models')"
            :disabled="surface.isBusy.value"
            @click="handleSyncModels"
          >
            <Icon icon="tabler:refresh" />
          </button>
        </div>
      </header>

      <div
        v-if="selectedModelGroup.models.length && selectedModelGroupModels.length"
        class="settings-ai-model-list settings-ai-model-list--catalog"
      >
        <article
          v-for="model in selectedModelGroupModels"
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
                :key="`${model.provider_id}:${model.model_key}:${capability.key}`"
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
                :key="`${model.provider_id}:${model.model_key}:${fact.key}`"
              >
                <strong>{{ fact.label }}</strong>
                {{ fact.value }}
              </span>
            </div>
          </div>
        </article>
      </div>

      <div v-else-if="selectedModelGroup.models.length" class="settings-empty-state">
        <Icon icon="tabler:search-off" />
        <strong>{{ t('No matching models') }}</strong>
        <span>{{ t('Clear the search, disable the availability filter or choose another provider.') }}</span>
      </div>

      <div v-else class="settings-empty-state">
        <Icon icon="tabler:list-search" />
        <strong>{{ t('No models synced') }}</strong>
      </div>
    </section>
  </div>

  <div v-else-if="!surface.isLoading.value" class="settings-empty-state">
    <Icon icon="tabler:sparkles-off" />
    <strong>{{ t('No AI providers yet') }}</strong>
  </div>
</template>
