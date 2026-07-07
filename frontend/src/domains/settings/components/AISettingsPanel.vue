<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from '../../../platform/i18n'
import Dialog from '../../../shared/ui/Dialog.vue'
import Icon from '../../../shared/ui/Icon.vue'
import type { AISettingsSurface, AiModelRouteRow } from '../queries/useAISettingsSurface'
import type { AiModelCatalogItem, AiProviderAccount, AiProviderPreset } from '../types/aiControlCenter'
import AIModelCatalogPanel from './AIModelCatalogPanel.vue'
import AIProviderConnectionWizard from './AIProviderConnectionWizard.vue'
import { modelCapabilityBadges, modelDetail, modelRuntimeFacts } from './aiModelCatalogPresentation'
import { aiProviderBrand, providerBrandClass } from './providerBranding'

type AiSettingsTab = 'providers' | 'models' | 'routes'

interface AiProviderListItem {
  id: string
  icon: string
  iconTone: string
  title: string
  subtitle: string
  badge: string
  metric: string
  provider: AiProviderAccount | null
  preset: AiProviderPreset | null
}

interface AiProviderListGroup {
  id: string
  label: string
  items: AiProviderListItem[]
}

interface AiDetailRow {
  label: string
  value: string
}

const props = defineProps<{
  surface: AISettingsSurface
}>()
const { t } = useI18n()
const activeTab = ref<AiSettingsTab>('providers')
const isProviderWizardOpen = ref(false)
const isModelPickerOpen = ref(false)

const tabs = computed(() => [
  {
    id: 'providers' as const,
    icon: 'tabler:plug-connected',
    label: t('Provider setup'),
    count: props.surface.providers.value.length,
  },
  {
    id: 'models' as const,
    icon: 'tabler:list-search',
    label: t('Model catalog'),
    count: props.surface.models.value.length,
  },
  {
    id: 'routes' as const,
    icon: 'tabler:route',
    label: t('Action routing'),
    count: props.surface.routeRows.value.length,
  },
])

const providerListGroups = computed<AiProviderListGroup[]>(() => {
  const localItems: AiProviderListItem[] = []
  const cliItems: AiProviderListItem[] = []
  const apiItems: AiProviderListItem[] = []

  for (const provider of props.surface.providers.value) {
    const group = providerGroupId(provider.provider_kind)
    const item = providerListItemForProvider(provider)
    if (group === 'local') localItems.push(item)
    if (group === 'cli') cliItems.push(item)
    if (group === 'api') apiItems.push(item)
  }

  for (const preset of props.surface.localPresets.value) {
    if (props.surface.providerForPreset(preset)) continue
    const group = providerGroupId(preset.provider_kind)
    const item = providerListItemForPreset(preset)
    if (group === 'local') localItems.push(item)
    if (group === 'cli') cliItems.push(item)
    if (group === 'api') apiItems.push(item)
  }

  return [
    { id: 'local', label: t('Local runtimes'), items: localItems },
    { id: 'cli', label: t('CLI providers'), items: cliItems },
    { id: 'api', label: t('Remote APIs'), items: apiItems },
  ]
})

const selectedProviderRows = computed<AiDetailRow[]>(() => {
  const provider = props.surface.selectedProvider.value
  if (!provider) return []
  return [
    { label: t('Provider ID'), value: provider.provider_id },
    { label: t('Provider key'), value: provider.provider_key },
    { label: t('Provider kind'), value: provider.provider_kind },
    { label: t('Base URL'), value: props.surface.providerBaseUrl(provider) || t('No base URL') },
    { label: t('Consent'), value: provider.consent_state },
  ]
})

const selectedAvailableModelCount = computed(() => {
  let count = 0
  for (const model of props.surface.selectedProviderModels.value) {
    if (model.is_available) count += 1
  }
  return count
})

const selectedRouteCount = computed(() => {
  const provider = props.surface.selectedProvider.value
  if (!provider) return 0
  let count = 0
  for (const route of props.surface.routes.value) {
    if (route.provider_id === provider.provider_id) count += 1
  }
  return count
})

const modelPickerDescription = computed(() => {
  const provider = props.surface.selectedProvider.value
  if (!provider) return t('Select a provider before choosing models.')
  return `${provider.display_name} · ${selectedAvailableModelCount.value}/${props.surface.selectedProviderModels.value.length} ${t('available')}`
})

function providerGroupId(providerKind: string): 'local' | 'cli' | 'api' {
  if (providerKind === 'built_in') return 'local'
  if (providerKind === 'cli') return 'cli'
  return 'api'
}

function providerIcon(providerKind: string, providerKey?: string): string {
  return aiProviderBrand(providerKind, providerKey).icon
}

function providerIconTone(providerKind: string, providerKey?: string): string {
  return providerBrandClass(aiProviderBrand(providerKind, providerKey))
}

function providerListItemForProvider(provider: AiProviderAccount): AiProviderListItem {
  const modelCount = providerModelCount(provider.provider_id)
  const brand = aiProviderBrand(provider.provider_kind, provider.provider_key)
  return {
    id: provider.provider_id,
    icon: brand.icon,
    iconTone: providerBrandClass(brand),
    title: provider.display_name,
    subtitle: `${provider.provider_key} · ${provider.status} · ${provider.consent_state}`,
    badge: provider.provider_kind,
    metric: modelCount > 0 ? `${modelCount} ${t('models')}` : t('No models synced'),
    provider,
    preset: null,
  }
}

function providerListItemForPreset(preset: AiProviderPreset): AiProviderListItem {
  const brand = aiProviderBrand(preset.provider_kind, preset.provider_key)
  return {
    id: `${preset.provider_kind}:${preset.provider_key}`,
    icon: brand.icon,
    iconTone: providerBrandClass(brand),
    title: preset.display_name,
    subtitle: `${preset.privacy} · ${preset.capabilities.join(', ')}`,
    badge: t('Preset'),
    metric: t('Connect'),
    provider: null,
    preset,
  }
}

function providerModelCount(providerId: string): number {
  let count = 0
  for (const model of props.surface.models.value) {
    if (model.provider_id === providerId) count += 1
  }
  return count
}

function eventValue(event: Event): string {
  return event.target instanceof HTMLInputElement || event.target instanceof HTMLSelectElement
    ? event.target.value
    : ''
}

function eventChecked(event: Event): boolean {
  return event.target instanceof HTMLInputElement ? event.target.checked : false
}

function toggleProvider(provider: AiProviderAccount, event: Event) {
  void props.surface.handleToggleProvider(provider, eventChecked(event))
}

function updateConsent(provider: AiProviderAccount, event: Event) {
  void props.surface.handleGrantConsent(provider, eventChecked(event))
}

function testProvider(provider: AiProviderAccount) {
  void props.surface.handleTestProvider(provider)
}

function syncModels(provider: AiProviderAccount) {
  void props.surface.handleSyncModels(provider)
}

function updateRoute(row: AiModelRouteRow, event: Event) {
  void props.surface.handleRouteSelection(row.slot.slot, eventValue(event))
}

function refreshModelRoutes() {
  void props.surface.handleRefreshModelRoutes()
}

function toggleModelAvailability(model: AiModelCatalogItem, event: Event) {
  void props.surface.handleModelAvailability(model, eventChecked(event))
}

function selectProviderListItem(item: AiProviderListItem): void {
  if (item.provider) {
    props.surface.selectProvider(item.provider.provider_id)
    return
  }
  if (item.preset) {
    void props.surface.handleConnectLocalPreset(item.preset)
  }
}

function refreshLocalAuth() {
  void props.surface.handleRefreshLocalAuth()
}

</script>

<template>
  <section class="settings-section settings-ai-section">
    <header class="settings-section-toolbar">
      <div>
        <h3>{{ t('AI Control Center') }}</h3>
        <p>{{ t('AI providers, OpenAI-compatible APIs, model catalog and action routing.') }}</p>
      </div>
    </header>

    <nav
      class="settings-ai-tabs"
      :aria-label="t('AI Control Center sections')"
      role="tablist"
    >
      <button
        v-for="tab in tabs"
        :id="`settings-ai-tab-${tab.id}`"
        :key="tab.id"
        type="button"
        role="tab"
        class="settings-ai-tab"
        :class="{ active: activeTab === tab.id }"
        :aria-selected="activeTab === tab.id"
        :aria-controls="`settings-ai-panel-${tab.id}`"
        @click="activeTab = tab.id"
      >
        <Icon :icon="tab.icon" />
        <span>{{ tab.label }}</span>
        <strong>{{ tab.count }}</strong>
      </button>
    </nav>

    <section
      v-if="activeTab === 'providers'"
      id="settings-ai-panel-providers"
      class="settings-ai-tab-panel settings-ai-tab-panel--providers"
      role="tabpanel"
      aria-labelledby="settings-ai-tab-providers"
    >
      <div class="settings-ai-provider-workbench">
        <aside class="settings-ai-provider-sidebar" :aria-label="t('Provider inventory')">
          <header class="settings-ai-provider-sidebar__header">
            <div>
              <span>{{ t('Provider inventory') }}</span>
              <strong>{{ surface.providers.value.length }}</strong>
            </div>
            <button
              type="button"
              class="secondary-button"
              :disabled="surface.isBusy.value"
              @click="isProviderWizardOpen = true"
            >
              <Icon icon="tabler:plug-connected" />
              {{ t('Connect provider') }}
            </button>
          </header>

          <p class="settings-ai-provider-sidebar__hint">
            {{ t('Local runtimes, CLI tools and remote APIs are managed in one provider list.') }}
          </p>

          <div class="settings-ai-provider-groups">
            <section
              v-for="group in providerListGroups"
              v-show="group.items.length"
              :key="group.id"
              class="settings-ai-provider-group"
            >
              <h4>
                <span>{{ group.label }}</span>
                <strong>{{ group.items.length }}</strong>
              </h4>

              <button
                v-for="item in group.items"
                :key="item.id"
                type="button"
                class="settings-ai-provider-row"
                :class="{
                  active: item.provider?.provider_id === surface.selectedProviderId.value,
                  'is-preset': !item.provider,
                }"
                @click="selectProviderListItem(item)"
              >
                <i class="settings-provider-icon" :class="item.iconTone" aria-hidden="true">
                  <Icon :icon="item.icon" />
                </i>
                <span>
                  <strong>{{ item.title }}</strong>
                  <small>{{ item.subtitle }}</small>
                </span>
                <em>{{ item.badge }}</em>
                <small class="settings-ai-provider-row__metric">{{ item.metric }}</small>
              </button>
            </section>
          </div>

          <div v-if="!surface.providers.value.length && !surface.isLoading.value" class="settings-empty-state">
            <Icon icon="tabler:sparkles-off" />
            <strong>{{ t('No AI providers yet') }}</strong>
            <span>{{ t('Create an OpenAI-compatible provider or seed a local runtime from backend presets.') }}</span>
          </div>
        </aside>

        <section class="settings-ai-provider-detail-pane">
          <template v-if="surface.selectedProvider.value">
            <header class="settings-ai-provider-detail-header">
              <div>
                <span
                  class="settings-ai-provider-detail-header__icon settings-provider-icon settings-provider-icon--lg"
                  :class="providerIconTone(
                    surface.selectedProvider.value.provider_kind,
                    surface.selectedProvider.value.provider_key
                  )"
                >
                  <Icon
                    :icon="providerIcon(
                      surface.selectedProvider.value.provider_kind,
                      surface.selectedProvider.value.provider_key
                    )"
                  />
                </span>
                <div>
                  <span>{{ t('Provider detail') }}</span>
                  <h3>{{ surface.selectedProvider.value.display_name }}</h3>
                  <p>
                    {{ surface.selectedProvider.value.provider_key }}
                    ·
                    {{ surface.selectedProvider.value.provider_kind }}
                    ·
                    {{ surface.selectedProvider.value.status }}
                  </p>
                </div>
              </div>

              <div class="settings-ai-provider-detail-header__actions">
                <button
                  type="button"
                  class="secondary-button"
                  :disabled="surface.isBusy.value"
                  @click="isModelPickerOpen = true"
                >
                  <Icon icon="tabler:list-check" />
                  {{ t('Choose models') }}
                </button>
                <button
                  type="button"
                  class="secondary-button"
                  :disabled="surface.isBusy.value"
                  @click="testProvider(surface.selectedProvider.value)"
                >
                  <Icon icon="tabler:heartbeat" />
                  {{ t('Test') }}
                </button>
                <button
                  type="button"
                  class="secondary-button"
                  :disabled="surface.isBusy.value"
                  @click="syncModels(surface.selectedProvider.value)"
                >
                  <Icon icon="tabler:refresh" />
                  {{ t('Sync models') }}
                </button>
              </div>
            </header>

            <div class="settings-ai-detail-metrics">
              <article>
                <span>{{ t('Runtime state') }}</span>
                <strong>{{ surface.selectedProvider.value.status }}</strong>
              </article>
              <article>
                <span>{{ t('Models available') }}</span>
                <strong>{{ selectedAvailableModelCount }}/{{ surface.selectedProviderModels.value.length }}</strong>
              </article>
              <article>
                <span>{{ t('Routes') }}</span>
                <strong>{{ selectedRouteCount }}</strong>
              </article>
            </div>

            <section class="settings-ai-detail-section">
              <header>
                <h4>{{ t('Connection') }}</h4>
              </header>
              <dl class="settings-ai-detail-list">
                <div
                  v-for="row in selectedProviderRows"
                  :key="row.label"
                >
                  <dt>{{ row.label }}</dt>
                  <dd>{{ row.value }}</dd>
                </div>
              </dl>
            </section>

            <section class="settings-ai-detail-section">
              <header>
                <h4>{{ t('Capability access') }}</h4>
              </header>
              <div class="settings-ai-detail-controls">
                <label class="settings-switch">
                  <input
                    type="checkbox"
                    :checked="surface.selectedProvider.value.status !== 'disabled'"
                    :disabled="surface.isBusy.value"
                    @change="toggleProvider(surface.selectedProvider.value, $event)"
                  >
                  <span />
                  <strong>{{ t('Enabled') }}</strong>
                </label>
                <label
                  v-if="surface.selectedProvider.value.provider_kind === 'api'"
                  class="settings-switch"
                >
                  <input
                    type="checkbox"
                    :checked="surface.selectedProvider.value.consent_state === 'granted'"
                    :disabled="surface.isBusy.value"
                    @change="updateConsent(surface.selectedProvider.value, $event)"
                  >
                  <span />
                  <strong>{{ t('Remote context') }}</strong>
                </label>
              </div>
              <div class="settings-ai-capability-row">
                <span
                  v-for="capability in surface.selectedProvider.value.capabilities"
                  :key="`${surface.selectedProvider.value.provider_id}:${capability}`"
                >
                  {{ capability }}
                </span>
              </div>
            </section>

            <section
              v-if="surface.activeLocalAuth.value"
              class="settings-ai-local-auth"
              :class="`is-${surface.activeLocalAuth.value.status}`"
            >
              <div>
                <strong>{{ surface.activeLocalAuth.value.display_name ?? surface.activeLocalAuth.value.provider_key }}</strong>
                <small>{{ surface.activeLocalAuth.value.message }}</small>
                <code v-if="surface.activeLocalAuth.value.login_command">
                  {{ surface.activeLocalAuth.value.login_command }}
                </code>
              </div>
              <div class="settings-account-actions">
                <button
                  type="button"
                  class="secondary-button"
                  :disabled="surface.isBusy.value"
                  @click="surface.handleOpenLocalAuthCallback"
                >
                  <Icon icon="tabler:external-link" />
                  {{ t('Open callback') }}
                </button>
                <button
                  type="button"
                  class="secondary-button"
                  :disabled="surface.isBusy.value"
                  @click="refreshLocalAuth"
                >
                  <Icon icon="tabler:refresh" />
                  {{ t('Refresh') }}
                </button>
              </div>
            </section>
          </template>

          <div v-else class="settings-empty-state">
            <Icon icon="tabler:pointer" />
            <strong>{{ t('Select provider') }}</strong>
          </div>
        </section>
      </div>
    </section>

    <section
      v-else-if="activeTab === 'models'"
      id="settings-ai-panel-models"
      class="settings-ai-tab-panel settings-ai-tab-panel--models"
      role="tabpanel"
      aria-labelledby="settings-ai-tab-models"
    >
      <AIModelCatalogPanel :surface="surface" />
    </section>

    <section
      v-else
      id="settings-ai-panel-routes"
      class="settings-ai-tab-panel settings-ai-tab-panel--routes"
      role="tabpanel"
      aria-labelledby="settings-ai-tab-routes"
    >
      <section class="settings-ai-route-board">
        <header class="settings-ai-route-board__header">
          <div>
            <span>{{ t('Model routing') }}</span>
            <strong>{{ surface.routeRows.value.length }}</strong>
          </div>
          <div class="settings-ai-route-board__header-actions">
            <small>{{ t('Choose which model handles translation, analysis, extraction, replies and embeddings.') }}</small>
            <button
              type="button"
              class="secondary-button"
              :disabled="surface.isBusy.value"
              @click="refreshModelRoutes"
            >
              <Icon icon="tabler:refresh" />
              {{ t('Refresh models') }}
            </button>
          </div>
        </header>

        <div class="settings-ai-route-list">
          <article
            v-for="row in surface.routeRows.value"
            :key="row.slot.slot"
            class="settings-ai-route-row"
          >
            <div>
              <strong>{{ row.label }}</strong>
              <small>{{ row.description }}</small>
              <span class="settings-ai-route-row__meta">
                <code>{{ row.slot.slot }}</code>
                <em>{{ row.options.length }} {{ t('model options') }}</em>
              </span>
            </div>
            <select
              :value="row.selectedValue"
              :disabled="surface.isBusy.value || row.options.length === 0"
              @change="updateRoute(row, $event)"
            >
              <option value="" disabled>{{ t('Select model') }}</option>
              <option
                v-for="option in row.options"
                :key="option.value"
                :value="option.value"
              >
                {{ option.label }} · {{ option.detail }}
              </option>
            </select>
          </article>
        </div>
      </section>
    </section>

    <AIProviderConnectionWizard
      v-model:open="isProviderWizardOpen"
      :surface="surface"
    />

    <Dialog
      :open="isModelPickerOpen"
      :title="t('Choose models for Hermes')"
      :description="modelPickerDescription"
      :close-label="t('Close model picker')"
      content-class="settings-ai-model-picker-dialog"
      @update:open="(value) => { isModelPickerOpen = value }"
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
            @click="syncModels(surface.selectedProvider.value)"
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
          @click="isModelPickerOpen = false"
        >
          {{ t('Done') }}
        </button>
      </template>
    </Dialog>
  </section>
</template>
