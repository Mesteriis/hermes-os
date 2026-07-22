<script setup lang="ts">
import Icon from '../../../shared/ui/Icon.vue'
import type { AISettingsSurface } from '../queries/useAISettingsSurface'
import AIModelCatalogPanel from './AIModelCatalogPanel.vue'
import AIModelPickerDialog from './AIModelPickerDialog.vue'
import AIProviderConnectionWizard from './AIProviderConnectionWizard.vue'
import AIUsageStatsPanel from './AIUsageStatsPanel.vue'
import { useAISettingsPanelController } from '../queries/useAISettingsPanelController'
import { providerIcon, providerIconTone } from './aiSettingsPanelPresentation'
import {
  hasAiRemoteContextConsent,
  isAiProviderEnabled,
  supportsAiRemoteContext,
} from './aiProviderAccessPresentation'

const props = defineProps<{
  surface: AISettingsSurface
}>()

const {
  t,
  activeTab,
  isProviderWizardOpen,
  isModelPickerOpen,
  tabs,
  providerListGroups,
  selectedProviderRows,
  selectedAvailableModelCount,
  selectedRouteCount,
  handleTestSelectedProvider,
  handleSyncSelectedProviderModels,
  handleToggleSelectedProvider,
  handleUpdateSelectedProviderConsent,
  handleUpdateRoute,
  handleRefreshModelRoutes,
  handleSelectProviderListItem,
  handleRefreshLocalAuth,
  handleOpenLocalAuthCallback,
  handleSetActiveTab,
  handleProviderWizardOpen,
  handleOpenProviderWizard,
  handleOpenModelPicker,
  handleSetModelPickerOpen,
} = useAISettingsPanelController({
  surface: props.surface,
})

</script>

<template>
  <section class="settings-section settings-ai-section">
    <header class="settings-section-toolbar">
      <div>
        <h3>{{ t('AI Hub') }}</h3>
        <p>{{ t('AI providers, local model downloads, model catalog and route selection.') }}</p>
      </div>
    </header>

    <nav
      class="settings-ai-tabs"
      :aria-label="t('AI Hub sections')"
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
        @click="handleSetActiveTab(tab.id)"
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
              @click="handleOpenProviderWizard"
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
                @click="handleSelectProviderListItem(item)"
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
                @click="handleOpenModelPicker"
              >
                  <Icon icon="tabler:list-check" />
                  {{ t('Choose models') }}
                </button>
                <button
                  type="button"
                  class="secondary-button"
                  :disabled="surface.isBusy.value"
                  @click="handleTestSelectedProvider"
                >
                  <Icon icon="tabler:heartbeat" />
                  {{ t('Test') }}
                </button>
                <button
                  type="button"
                  class="secondary-button"
                  :disabled="surface.isBusy.value"
                  @click="handleSyncSelectedProviderModels"
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
                    :checked="isAiProviderEnabled(surface.selectedProvider.value)"
                    :disabled="surface.isBusy.value"
                    @change="handleToggleSelectedProvider"
                  >
                  <span />
                  <strong>{{ t('Enabled') }}</strong>
                </label>
                <label
                  v-if="supportsAiRemoteContext(surface.selectedProvider.value)"
                  class="settings-switch"
                >
                  <input
                    type="checkbox"
                    :checked="hasAiRemoteContextConsent(surface.selectedProvider.value)"
                    :disabled="surface.isBusy.value"
                    @change="handleUpdateSelectedProviderConsent"
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
              @click="handleOpenLocalAuthCallback"
                >
                  <Icon icon="tabler:external-link" />
                  {{ t('Open callback') }}
                </button>
                <button
                  type="button"
                  class="secondary-button"
                  :disabled="surface.isBusy.value"
              @click="handleRefreshLocalAuth"
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
      v-else-if="activeTab === 'routes'"
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
              @click="handleRefreshModelRoutes"
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
              :disabled="surface.isBusy.value"
              @change="handleUpdateRoute(row, $event)"
            >
              <option value="">{{ t('Not routed') }}</option>
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

    <section
      v-else
      id="settings-ai-panel-stats"
      class="settings-ai-tab-panel settings-ai-tab-panel--stats"
      role="tabpanel"
      aria-labelledby="settings-ai-tab-stats"
    >
      <AIUsageStatsPanel :surface="surface" />
    </section>

    <AIProviderConnectionWizard
      :open="isProviderWizardOpen"
      @update:open="handleProviderWizardOpen"
      :surface="surface"
    />

    <AIModelPickerDialog
      :open="isModelPickerOpen"
      :surface="surface"
      @update:open="handleSetModelPickerOpen"
    />
  </section>
</template>
