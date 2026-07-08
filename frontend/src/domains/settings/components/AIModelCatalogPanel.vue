<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import SearchInput from '../../../shared/ui/SearchInput.vue'
import type { AISettingsSurface } from '../queries/useAISettingsSurface'
import type { AiModelCatalogItem, AiProviderAccount } from '../types/aiControlCenter'
import {
  modelCapabilityBadges,
  modelDetail,
  modelMatchesSearch,
  modelRuntimeFacts,
} from './aiModelCatalogPresentation'
import { aiProviderBrand, providerBrandClass } from './providerBranding'

interface AiProviderModelGroup {
  provider: AiProviderAccount
  models: AiModelCatalogItem[]
  availableCount: number
}

const props = defineProps<{
  surface: AISettingsSurface
}>()

const { t } = useI18n()
const activeModelProviderId = ref<string | null>(null)
const modelCatalogSearch = ref('')
const showAvailableModelsOnly = ref(true)

const providerModelGroups = computed<AiProviderModelGroup[]>(() => {
  const groups: AiProviderModelGroup[] = []

  for (const provider of props.surface.providers.value) {
    const models: AiModelCatalogItem[] = []
    let availableCount = 0

    for (const model of props.surface.models.value) {
      if (model.provider_id !== provider.provider_id) continue

      models.push(model)
      if (model.is_available) availableCount += 1
    }

    groups.push({ provider, models, availableCount })
  }

  return groups
})

const selectedModelGroup = computed<AiProviderModelGroup | null>(() => {
  const groups = providerModelGroups.value
  if (!groups.length) return null
  return groups.find((group) => group.provider.provider_id === activeModelProviderId.value) ?? groups[0]
})

const normalizedModelCatalogSearch = computed(() => modelCatalogSearch.value.trim().toLowerCase())

const selectedModelGroupModels = computed<AiModelCatalogItem[]>(() => {
  const group = selectedModelGroup.value
  if (!group) return []
  const query = normalizedModelCatalogSearch.value
  const models: AiModelCatalogItem[] = []
  for (const model of group.models) {
    if (showAvailableModelsOnly.value && !model.is_available) continue
    if (!query || modelMatchesSearch(model, group.provider, query)) models.push(model)
  }
  return models
})

function providerIcon(providerKind: string, providerKey?: string): string {
  return aiProviderBrand(providerKind, providerKey).icon
}

function providerIconTone(providerKind: string, providerKey?: string): string {
  return providerBrandClass(aiProviderBrand(providerKind, providerKey))
}

function eventChecked(event: Event): boolean {
  return event.target instanceof HTMLInputElement ? event.target.checked : false
}

function updateAvailableModelsFilter(event: Event): void {
  showAvailableModelsOnly.value = eventChecked(event)
}

function syncModels(provider: AiProviderAccount) {
  void props.surface.handleSyncModels(provider)
}

function toggleModelAvailability(model: AiModelCatalogItem, event: Event) {
  void props.surface.handleModelAvailability(model, eventChecked(event))
}

function selectModelProvider(providerId: string): void {
  activeModelProviderId.value = providerId
  modelCatalogSearch.value = ''
}
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
        @click="selectModelProvider(group.provider.provider_id)"
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
              @change="updateAvailableModelsFilter"
            >
          </label>
          <button
            type="button"
            class="icon-button"
            :title="t('Sync models')"
            :aria-label="t('Sync models')"
            :disabled="surface.isBusy.value"
            @click="syncModels(selectedModelGroup.provider)"
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
          <label class="settings-ai-model-checkbox">
            <input
              type="checkbox"
              :checked="model.is_available"
              :disabled="surface.isBusy.value"
              @change="toggleModelAvailability(model, $event)"
            >
            <span>
              <strong>{{ model.is_available ? t('Enabled in Hermes') : t('Disabled in Hermes') }}</strong>
              <small>{{ t('Controls whether this model appears in action routing.') }}</small>
            </span>
          </label>

          <div class="settings-ai-model-card__body">
            <div class="settings-ai-model-card__title">
              <strong>{{ model.display_name }}</strong>
              <small>{{ modelDetail(model) }}</small>
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
