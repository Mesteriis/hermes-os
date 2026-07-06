<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from '../../../platform/i18n'
import Dialog from '../../../shared/ui/Dialog.vue'
import Icon from '../../../shared/ui/Icon.vue'
import SearchInput from '../../../shared/ui/SearchInput.vue'
import type { AISettingsSurface, AiModelRouteRow } from '../queries/useAISettingsSurface'
import type { AiModelCatalogItem, AiProviderAccount, AiProviderPreset } from '../types/aiControlCenter'
import AIProviderConnectionWizard from './AIProviderConnectionWizard.vue'

type AiSettingsTab = 'providers' | 'models' | 'routes'

interface AiProviderModelGroup {
  provider: AiProviderAccount
  models: AiModelCatalogItem[]
  availableCount: number
}

interface AiProviderListItem {
  id: string
  icon: string
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

interface AiModelCapabilityBadge {
  key: string
  label: string
  muted: boolean
}

interface AiModelRuntimeFact {
  key: string
  label: string
  value: string
}

const props = defineProps<{
  surface: AISettingsSurface
}>()
const { t } = useI18n()
const activeTab = ref<AiSettingsTab>('providers')
const activeModelProviderId = ref<string | null>(null)
const modelCatalogSearch = ref('')
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
  if (!query) return group.models
  return group.models.filter((model) => modelMatchesSearch(model, group.provider, query))
})

function providerGroupId(providerKind: string): 'local' | 'cli' | 'api' {
  if (providerKind === 'built_in') return 'local'
  if (providerKind === 'cli') return 'cli'
  return 'api'
}

function providerIcon(providerKind: string): string {
  if (providerKind === 'built_in') return 'tabler:device-desktop'
  if (providerKind === 'cli') return 'tabler:terminal-2'
  return 'tabler:cloud-cog'
}

function providerListItemForProvider(provider: AiProviderAccount): AiProviderListItem {
  const modelCount = providerModelCount(provider.provider_id)
  return {
    id: provider.provider_id,
    icon: providerIcon(provider.provider_kind),
    title: provider.display_name,
    subtitle: `${provider.provider_key} · ${provider.status} · ${provider.consent_state}`,
    badge: provider.provider_kind,
    metric: modelCount > 0 ? `${modelCount} ${t('models')}` : t('No models synced'),
    provider,
    preset: null,
  }
}

function providerListItemForPreset(preset: AiProviderPreset): AiProviderListItem {
  return {
    id: `${preset.provider_kind}:${preset.provider_key}`,
    icon: providerIcon(preset.provider_kind),
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

function selectModelProvider(providerId: string): void {
  activeModelProviderId.value = providerId
  modelCatalogSearch.value = ''
}

function modelMatchesSearch(
  model: AiModelCatalogItem,
  provider: AiProviderAccount,
  query: string
): boolean {
  return modelSearchTokens(model, provider).some((token) => token.toLowerCase().includes(query))
}

function modelSearchTokens(model: AiModelCatalogItem, provider: AiProviderAccount): string[] {
  const details = metadataRecord(model.metadata, 'runtime_details') ?? metadataRecord(model.metadata, 'details')
  return [
    provider.display_name,
    provider.provider_key,
    provider.provider_kind,
    model.display_name,
    model.model_key,
    model.category,
    model.privacy,
    ...model.capabilities,
    ...metadataStringArray(model.metadata, 'runtime_capabilities'),
    ...providerMetadataCapabilities(model.metadata),
    metadataString(model.metadata, 'owned_by'),
    metadataString(model.metadata, 'source'),
    metadataString(model.metadata, 'capability_source'),
    metadataString(details, 'family'),
    metadataString(details, 'format'),
    metadataString(details, 'parameter_size'),
    metadataString(details, 'quantization_level'),
  ].filter((token) => token.trim().length > 0)
}

function modelCapabilityBadges(model: AiModelCatalogItem): AiModelCapabilityBadge[] {
  const badges = new Map<string, AiModelCapabilityBadge>()
  const capabilitySource = metadataString(model.metadata, 'capability_source')
  const modelCapabilitiesAreInferred = capabilitySource === 'hermes_model_key_heuristic'

  for (const capability of metadataStringArray(model.metadata, 'runtime_capabilities')) {
    addCapabilityBadge(badges, capability, false)
  }
  for (const capability of providerMetadataCapabilities(model.metadata)) {
    addCapabilityBadge(badges, capability, false)
  }
  for (const capability of model.capabilities) {
    addCapabilityBadge(badges, capability, modelCapabilitiesAreInferred)
  }

  if (badges.size === 0) {
    return [
      {
        key: 'runtime-not-reported',
        label: t('Runtime did not report capabilities'),
        muted: true,
      },
    ]
  }

  return [...badges.values()]
}

function addCapabilityBadge(
  badges: Map<string, AiModelCapabilityBadge>,
  capability: string,
  muted: boolean
): void {
  const normalized = normalizedCapabilityKey(capability)
  if (!normalized) return
  const existing = badges.get(normalized)
  if (existing && (!existing.muted || muted)) return
  badges.set(normalized, {
    key: normalized,
    label: capabilityLabel(normalized, capability),
    muted,
  })
}

function normalizedCapabilityKey(capability: string): string {
  const normalized = capability.trim().replace(/_/g, '-').toLowerCase()
  if (!normalized) return ''
  if (['completion', 'completions', 'chat-completion', 'chat-completions', 'text-generation', 'text'].includes(normalized)) {
    return 'chat'
  }
  if (['embedding', 'embeddings'].includes(normalized)) return 'embeddings'
  if (['image', 'images', 'image-input'].includes(normalized)) return 'vision'
  if (['audio-input', 'audio-output'].includes(normalized)) return 'audio'
  if (['tool', 'tools', 'tool-use', 'function-calling'].includes(normalized)) return 'tools'
  return normalized
}

function capabilityLabel(capability: string, fallback: string): string {
  const labels: Record<string, string> = {
    audio: t('Audio'),
    chat: t('Chat'),
    embeddings: t('Embeddings'),
    extraction: t('Extraction'),
    multimodal: t('Multimodal'),
    reasoning: t('Reasoning'),
    routing: t('Routing'),
    summarization: t('Summaries'),
    tools: t('Tools'),
    vision: t('Vision'),
  }
  return labels[capability] ?? fallback
}

function modelRuntimeFacts(model: AiModelCatalogItem): AiModelRuntimeFact[] {
  const facts: AiModelRuntimeFact[] = []
  const details = metadataRecord(model.metadata, 'runtime_details') ?? metadataRecord(model.metadata, 'details')
  const infoSummary = metadataRecord(model.metadata, 'model_info_summary')
  const owner = metadataString(model.metadata, 'owned_by')
  const source = metadataString(model.metadata, 'source')
  const capabilitySource = metadataString(model.metadata, 'capability_source')
  const contextWindow = model.context_window ?? metadataNumber(infoSummary, 'context_window')
  const embeddingDimension = model.embedding_dimension ?? metadataNumber(infoSummary, 'embedding_dimension')

  if (contextWindow) {
    facts.push({ key: 'context', label: t('Context'), value: `${contextWindow} ctx` })
  }
  if (embeddingDimension) {
    facts.push({ key: 'embedding', label: t('Embedding'), value: `${embeddingDimension} dim` })
  }
  addRuntimeFact(facts, 'family', t('Family'), metadataString(details, 'family'))
  addRuntimeFact(facts, 'format', t('Format'), metadataString(details, 'format'))
  addRuntimeFact(facts, 'parameters', t('Parameters'), metadataString(details, 'parameter_size'))
  addRuntimeFact(facts, 'quantization', t('Quantization'), metadataString(details, 'quantization_level'))
  addRuntimeFact(facts, 'owner', t('Owner'), owner)
  addRuntimeFact(facts, 'source', t('Source'), source)
  addRuntimeFact(facts, 'capability-source', t('Capability source'), capabilitySource)

  return facts
}

function addRuntimeFact(
  facts: AiModelRuntimeFact[],
  key: string,
  label: string,
  value: string
): void {
  if (!value) return
  facts.push({ key, label, value })
}

function providerMetadataCapabilities(metadata: Record<string, unknown>): string[] {
  const providerMetadata = metadataRecord(metadata, 'provider_metadata')
  if (!providerMetadata) return []
  return [
    ...metadataStringArray(providerMetadata, 'capabilities'),
    ...metadataStringArray(providerMetadata, 'modalities'),
    ...metadataStringArray(providerMetadata, 'input_modalities'),
    ...metadataStringArray(providerMetadata, 'output_modalities'),
    ...metadataStringArray(providerMetadata, 'supported_features'),
  ]
}

function metadataRecord(
  metadata: Record<string, unknown> | null,
  key: string
): Record<string, unknown> | null {
  if (!metadata) return null
  const value = metadata[key]
  if (typeof value !== 'object' || value === null || Array.isArray(value)) return null
  return value as Record<string, unknown>
}

function metadataString(metadata: Record<string, unknown> | null, key: string): string {
  if (!metadata) return ''
  const value = metadata[key]
  return typeof value === 'string' ? value : ''
}

function metadataNumber(metadata: Record<string, unknown> | null, key: string): number | null {
  if (!metadata) return null
  const value = metadata[key]
  return typeof value === 'number' ? value : null
}

function metadataStringArray(metadata: Record<string, unknown> | null, key: string): string[] {
  if (!metadata) return []
  const value = metadata[key]
  if (!Array.isArray(value)) return []
  return value.filter((item): item is string => typeof item === 'string' && item.trim().length > 0)
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
                <Icon :icon="item.icon" />
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
                <span class="settings-ai-provider-detail-header__icon">
                  <Icon :icon="providerIcon(surface.selectedProvider.value.provider_kind)" />
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
                <Icon :icon="providerIcon(group.provider.provider_kind)" />
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
                  <button
                    type="button"
                    class="secondary-button"
                    :disabled="surface.isBusy.value"
                    @click="syncModels(selectedModelGroup.provider)"
                  >
                    <Icon icon="tabler:refresh" />
                    {{ t('Sync models') }}
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
                        v-for="capability in modelCapabilityBadges(model)"
                        :key="`${model.provider_id}:${model.model_key}:${capability.key}`"
                        :class="{ 'is-muted': capability.muted }"
                      >
                        {{ capability.label }}
                      </span>
                    </div>
                    <div
                      v-if="modelRuntimeFacts(model).length"
                      class="settings-ai-model-facts"
                    >
                      <span
                        v-for="fact in modelRuntimeFacts(model)"
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
                <span>{{ t('Clear the search or choose another provider.') }}</span>
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
                  v-for="capability in modelCapabilityBadges(model)"
                  :key="`${model.provider_id}:${model.model_key}:picker:${capability.key}`"
                  :class="{ 'is-muted': capability.muted }"
                >
                  {{ capability.label }}
                </span>
              </div>
              <div
                v-if="modelRuntimeFacts(model).length"
                class="settings-ai-model-facts"
              >
                <span
                  v-for="fact in modelRuntimeFacts(model)"
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
