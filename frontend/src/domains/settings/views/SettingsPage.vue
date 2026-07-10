<script setup lang="ts">
import { watch } from 'vue'
import { useI18n } from '../../../platform/i18n'
import IntegrationConnectionWizard from '../../../shared/integrationSetup/IntegrationConnectionWizard.vue'
import Icon from '../../../shared/ui/Icon.vue'
import { useToast } from '../../../shared/ui'
import AISettingsPanel from '../components/AISettingsPanel.vue'
import BackgroundJobsSettingsPanel from '../components/BackgroundJobsSettingsPanel.vue'
import MaintenanceSettingsPanel from '../components/MaintenanceSettingsPanel.vue'
import SettingsNavigationTree from '../components/SettingsNavigationTree.vue'
import SettingsOverviewStrip from '../components/SettingsOverviewStrip.vue'
import TraceLogsSettingsPanel from '../components/TraceLogsSettingsPanel.vue'
import {
  healthTone,
  signalTargetTone,
  sourceIcon,
  sourceStateTone
} from '../components/signalHubSettingsPresentation'
import type { SignalRouteTargetKind } from '../components/signalHubSettingsPresentation'
import { useSettingsPageSurface } from '../queries/useSettingsPageSurface'
import type { AccountServiceRow } from '../queries/useIntegrationsSettingsSurface'
import type { ApplicationSetting } from '../types/settings'

const { t } = useI18n()
const toast = useToast()
const {
  aiSettings,
  applicationSettings,
  backgroundJobsSettings,
  integrationsSettings,
  languageSettings,
  maintenanceSettings,
  realtimeStatus,
  settingsOverviewCards,
  settingsTreeGroups,
  signalHubSettings,
  store,
  traceLogsSettings,
} = useSettingsPageSurface()

const accountGroups = integrationsSettings.groups
const selectedAccountSummary = integrationsSettings.selectedAccountSummary
const settingsByCategory = applicationSettings.settingsByCategory

watch(() => store.actionMessage, (message) => {
  if (message) toast.success(t('Settings action completed'), message)
})

watch(() => store.errorMessage, (message) => {
  if (message) toast.error(t('Settings action failed'), message)
})

function eventValue(event: Event): string {
  return event.target instanceof HTMLInputElement || event.target instanceof HTMLSelectElement || event.target instanceof HTMLTextAreaElement
    ? event.target.value
    : ''
}

function eventChecked(event: Event): boolean {
  return event.target instanceof HTMLInputElement ? event.target.checked : false
}

function updateSettingDraft(setting: ApplicationSetting, event: Event) {
  applicationSettings.handleInput(setting, eventValue(event))
}

function updateBooleanSettingDraft(setting: ApplicationSetting, event: Event) {
  applicationSettings.handleInput(setting, eventChecked(event) ? 'true' : 'false')
}

function updateSelectedAccount(event: Event) {
  void integrationsSettings.handleToggleSelectedAccount(eventChecked(event))
}

function updateSelectedService(service: AccountServiceRow, event: Event) {
  void integrationsSettings.handleToggleSelectedService(service.id, eventChecked(event))
}

function runSelectedServiceNow(service: AccountServiceRow) {
  void integrationsSettings.handleRunSelectedServiceNow(service.id)
}

function runSelectedServiceModeAction(service: AccountServiceRow) {
  if (service.id === 'contacts') {
    void integrationsSettings.handleEnableSelectedContactsBidirectional()
  }
}

function updateSelectedAccountLabel(event: Event) {
  integrationsSettings.handleSelectedAccountLabelInput(eventValue(event))
}

function saveSelectedAccountLabel() {
  void integrationsSettings.handleSaveSelectedAccountLabel()
}

function signalTargetIcon(kind: SignalRouteTargetKind): string {
  return kind === 'projection' ? 'tabler:chart-dots' : 'tabler:route'
}

</script>

<template>
  <section class="settings-page">
    <SettingsOverviewStrip
      :can-reconnect="realtimeStatus.canTriggerReconnect"
      :cards="settingsOverviewCards"
      @reconnect="realtimeStatus.requestReconnect()"
    />

    <div class="settings-workbench">
      <SettingsNavigationTree
        :groups="settingsTreeGroups"
        :selected-section="store.selectedSection"
        @select-section="store.selectSection"
      />

      <main class="settings-workbench-content">
        <section
          v-if="store.selectedSection === 'accounts'"
          class="settings-section settings-accounts-section"
        >
          <header class="settings-section-toolbar">
            <div>
              <h3>{{ t('Accounts') }}</h3>
              <p>{{ t('Provider records from backend settings and integration APIs.') }}</p>
            </div>
            <button type="button" class="primary-button" @click="integrationsSettings.openConnectWizard('mail')">
              <Icon icon="tabler:plus" />
              {{ t('Add account') }}
            </button>
          </header>

          <div class="settings-accounts-grid">
            <section class="settings-account-list" :aria-label="t('Accounts')">
              <template v-for="group in accountGroups" :key="group.label">
                <button
                  v-for="row in group.items"
                  :key="row.account.account_id"
                  type="button"
                  class="settings-account-row"
                  :class="{ active: row.isSelected }"
                  @click="integrationsSettings.selectIntegration(row.account.account_id)"
                >
                  <i class="settings-provider-icon" :class="row.iconTone" aria-hidden="true">
                    <Icon :icon="row.icon" />
                  </i>
                  <span>
                    <strong>{{ row.displayName }}</strong>
                    <small>{{ row.providerLabel }} · {{ row.flowLabel }}</small>
                  </span>
                  <em :class="`is-${row.statusClass}`">{{ row.statusText }}</em>
                </button>
              </template>

              <div v-if="!integrationsSettings.hasAccounts.value" class="settings-empty-state">
                <Icon icon="tabler:id-off" />
                <strong>{{ t('No accounts yet') }}</strong>
                <span>{{ t('Start the wizard to create a backend-managed account setup.') }}</span>
              </div>
            </section>

            <section class="settings-account-detail">
              <template v-if="selectedAccountSummary">
                <header class="settings-account-detail__header">
                  <div class="settings-account-detail__identity">
                    <i
                      class="settings-provider-icon settings-provider-icon--lg"
                      :class="selectedAccountSummary.iconTone"
                      aria-hidden="true"
                    >
                      <Icon :icon="selectedAccountSummary.icon" />
                    </i>
                    <div>
                      <span>{{ selectedAccountSummary.providerLabel }}</span>
                      <h3>{{ selectedAccountSummary.displayName }}</h3>
                      <p>{{ selectedAccountSummary.email || selectedAccountSummary.account.external_account_id }}</p>
                    </div>
                  </div>
                  <label class="settings-switch" :title="selectedAccountSummary.accountToggleHelp">
                    <input
                      type="checkbox"
                      :checked="selectedAccountSummary.accountEnabled"
                      :disabled="!selectedAccountSummary.canManageMail"
                      @change="updateSelectedAccount"
                    >
                    <span />
                    <strong>{{ selectedAccountSummary.accountToggleLabel }}</strong>
                  </label>
                </header>

                <form class="settings-account-label-form" @submit.prevent="saveSelectedAccountLabel">
                  <label>
                    <span>{{ t('Account label') }}</span>
                    <input
                      type="text"
                      :value="selectedAccountSummary.labelDraft"
                      :disabled="selectedAccountSummary.labelSaving"
                      autocomplete="off"
                      @input="updateSelectedAccountLabel"
                    >
                  </label>
                  <button
                    type="submit"
                    class="secondary-button"
                    :disabled="!selectedAccountSummary.labelDirty || selectedAccountSummary.labelSaving"
                  >
                    <Icon icon="tabler:device-floppy" />
                    {{ selectedAccountSummary.labelSaving ? t('Saving') : t('Save') }}
                  </button>
                </form>

                <section class="settings-service-list" :aria-label="t('Services')">
                  <h4>{{ t('Services') }}</h4>
                  <article
                    v-for="service in selectedAccountSummary.services"
                    :key="service.id"
                    class="settings-service-row"
                    :class="{ disabled: !service.canToggle }"
                  >
                    <Icon :icon="service.icon" />
                    <span>
                      <strong>{{ service.label }}</strong>
                      <small>{{ service.detail }}</small>
                    </span>
                    <label class="settings-switch" :title="service.disabledReason || service.statusText">
                      <input
                        type="checkbox"
                        :checked="service.enabled"
                        :disabled="!service.canToggle || service.isBusy"
                        @change="updateSelectedService(service, $event)"
                      >
                      <span />
                      <strong>{{ service.statusText }}</strong>
                    </label>
                    <button
                      v-if="service.canRunNow"
                      type="button"
                      class="secondary-button settings-service-row__action"
                      :disabled="service.isBusy"
                      @click="runSelectedServiceNow(service)"
                    >
                      <Icon icon="tabler:refresh" />
                      {{ service.isBusy ? t('Syncing') : service.runNowLabel }}
                    </button>
                    <button
                      v-if="service.canRunModeAction"
                      type="button"
                      class="secondary-button settings-service-row__action"
                      :disabled="service.isBusy"
                      @click="runSelectedServiceModeAction(service)"
                    >
                      <Icon icon="tabler:arrows-exchange" />
                      {{ service.modeActionLabel }}
                    </button>
                  </article>
                </section>

                <footer class="settings-account-actions">
                  <button
                    v-if="selectedAccountSummary.canRecoverExpiredCredential"
                    type="button"
                    class="secondary-button"
                    @click="integrationsSettings.openCredentialRecovery()"
                  >
                    <Icon icon="tabler:key" />
                    {{ t('Восстановить аккаунт') }}
                  </button>
                  <button
                    type="button"
                    class="secondary-button"
                    :disabled="!selectedAccountSummary.canOpenSetupAction"
                    :title="selectedAccountSummary.canOpenSetupAction ? selectedAccountSummary.selectedInspectorActionLabel : t('No self-serve setup route for this provider yet')"
                    @click="integrationsSettings.openConnectWizard()"
                  >
                    <Icon icon="tabler:route" />
                    {{ selectedAccountSummary.selectedInspectorActionLabel }}
                  </button>
                  <button
                    v-if="selectedAccountSummary.canManageMail"
                    type="button"
                    class="secondary-button"
                    :disabled="integrationsSettings.activeMailAction.value === selectedAccountSummary.account.account_id"
                    @click="integrationsSettings.handleExport(selectedAccountSummary.account.account_id)"
                  >
                    <Icon icon="tabler:download" />
                    {{ t('Export') }}
                  </button>
                  <button
                    v-if="selectedAccountSummary.canManageMail"
                    type="button"
                    class="secondary-button"
                    :disabled="integrationsSettings.activeMailAction.value === selectedAccountSummary.account.account_id"
                    @click="integrationsSettings.handleLogout(selectedAccountSummary.account.account_id)"
                  >
                    <Icon icon="tabler:logout" />
                    {{ t('Logout') }}
                  </button>
                  <button
                    v-if="selectedAccountSummary.canManageMail"
                    type="button"
                    class="danger-button"
                    :disabled="integrationsSettings.activeMailAction.value === selectedAccountSummary.account.account_id"
                    @click="integrationsSettings.handleDelete(selectedAccountSummary.account.account_id)"
                  >
                    <Icon icon="tabler:trash" />
                    {{ t('Delete permanently') }}
                  </button>
                </footer>
              </template>

              <div v-else class="settings-empty-state">
                <Icon icon="tabler:cursor-text" />
                <strong>{{ t('Select an account') }}</strong>
                <span>{{ t('Choose a provider record to inspect services and available backend actions.') }}</span>
              </div>
            </section>
          </div>
        </section>

        <section
          v-else-if="store.selectedSection === 'application'"
          class="settings-section"
        >
          <header class="settings-section-toolbar">
            <div>
              <h3>{{ t('Application Settings') }}</h3>
              <p>{{ t('Declared backend settings registry, grouped by category.') }}</p>
            </div>
          </header>

          <div v-if="applicationSettings.isLoading.value" class="settings-empty-state">
            <Icon icon="tabler:loader-2" />
            <strong>{{ t('Loading settings') }}</strong>
          </div>

          <template v-else>
            <section
              v-for="(settings, category) in settingsByCategory"
              :key="category"
              class="settings-registry-group"
            >
              <h4>{{ t(applicationSettings.categoryLabel(String(category))) }}</h4>
              <article v-for="setting in settings" :key="setting.setting_key" class="settings-registry-row">
                <div>
                  <strong>{{ setting.label }}</strong>
                  <small>{{ setting.description }}</small>
                  <code>{{ setting.setting_key }}</code>
                </div>

                <select
                  v-if="applicationSettings.settingControlType(setting) === 'select'"
                  :value="applicationSettings.settingDraftValue(setting)"
                  :disabled="!setting.is_editable"
                  @change="updateSettingDraft(setting, $event)"
                >
                  <option
                    v-for="value in applicationSettings.settingAllowedValues(setting)"
                    :key="value"
                    :value="value"
                  >
                    {{ value }}
                  </option>
                </select>

                <label v-else-if="applicationSettings.settingControlType(setting) === 'checkbox'" class="settings-switch">
                  <input
                    type="checkbox"
                    :checked="applicationSettings.settingDraftValue(setting) === 'true'"
                    :disabled="!setting.is_editable"
                    @change="updateBooleanSettingDraft(setting, $event)"
                  >
                  <span />
                  <strong>{{ applicationSettings.settingDraftValue(setting) }}</strong>
                </label>

                <input
                  v-else-if="applicationSettings.settingControlType(setting) === 'number'"
                  type="number"
                  :value="applicationSettings.settingDraftValue(setting)"
                  :disabled="!setting.is_editable"
                  @input="updateSettingDraft(setting, $event)"
                >

                <textarea
                  v-else-if="setting.value_kind === 'json'"
                  :value="applicationSettings.settingDraftValue(setting)"
                  :disabled="!setting.is_editable"
                  rows="3"
                  @input="updateSettingDraft(setting, $event)"
                />

                <input
                  v-else
                  type="text"
                  :value="applicationSettings.settingDraftValue(setting)"
                  :disabled="!setting.is_editable"
                  @input="updateSettingDraft(setting, $event)"
                >

                <button
                  type="button"
                  class="secondary-button"
                  :disabled="!setting.is_editable || !applicationSettings.settingHasChanged(setting) || applicationSettings.savingSettingKey.value === setting.setting_key"
                  @click="applicationSettings.handleSave(setting)"
                >
                  <Icon icon="tabler:device-floppy" />
                  {{ t('Save') }}
                </button>
              </article>
            </section>
          </template>
        </section>

        <BackgroundJobsSettingsPanel
          v-else-if="store.selectedSection === 'background-jobs'"
          :surface="backgroundJobsSettings"
        />

        <TraceLogsSettingsPanel
          v-else-if="store.selectedSection === 'logs-traces'"
          :surface="traceLogsSettings"
        />

        <MaintenanceSettingsPanel v-else-if="store.selectedSection === 'maintenance'" :surface="maintenanceSettings" />

        <section
          v-else-if="store.selectedSection === 'language'"
          class="settings-section"
        >
          <header class="settings-section-toolbar">
            <div>
              <h3>{{ t('Language') }}</h3>
              <p>{{ t('Locale preference is stored through the settings registry.') }}</p>
            </div>
          </header>
          <div class="settings-form-grid">
            <button
              v-for="localeOption in languageSettings.localeOptions"
              :key="localeOption.value"
              type="button"
              class="settings-choice"
              :class="{ active: languageSettings.currentLocale.value === localeOption.value }"
              :disabled="languageSettings.isBusy.value"
              @click="languageSettings.handleLocaleChange(localeOption.value)"
            >
              <Icon icon="tabler:language" />
              <strong>{{ localeOption.label }}</strong>
            </button>
          </div>
        </section>

        <section
          v-else-if="store.selectedSection === 'signal-hub'"
          class="settings-section settings-signal-section"
        >
          <header class="settings-section-toolbar">
            <div>
              <h3>{{ t('Signal Hub') }}</h3>
              <p>{{ t('Signal source controls stay under Settings; provider account setup stays in Accounts.') }}</p>
            </div>
          </header>

          <div v-if="signalHubSettings.isLoading.value" class="settings-empty-state">
            <Icon icon="tabler:loader-2" />
            <strong>{{ t('Loading Signal Hub') }}</strong>
          </div>

          <template v-else>
            <section class="settings-signal-summary" :aria-label="t('Signal Hub summary')">
              <article class="settings-signal-summary-tile">
                <span>{{ t('Sources') }}</span>
                <strong>{{ signalHubSettings.signalInventoryRows.value.length }}</strong>
              </article>
              <article class="settings-signal-summary-tile">
                <span>{{ t('Running') }}</span>
                <strong>{{ signalHubSettings.enabledCount.value }}</strong>
              </article>
              <article class="settings-signal-summary-tile">
                <span>{{ t('Connected') }}</span>
                <strong>{{ signalHubSettings.connectedCount.value }}</strong>
              </article>
              <article class="settings-signal-summary-tile">
                <span>{{ t('Attention') }}</span>
                <strong>{{ signalHubSettings.unhealthyCount.value + signalHubSettings.replayPendingCount.value }}</strong>
              </article>
            </section>

            <section
              class="settings-signal-panel"
              :aria-label="t(signalHubSettings.activeSignalView.value === 'graph' ? 'Signal consumer graph' : 'Signal inventory')"
            >
              <header class="settings-signal-panel__header">
                <div>
                  <span>{{ t(signalHubSettings.activeSignalView.value === 'graph' ? 'Graph' : 'Inventory') }}</span>
                  <strong>{{ t(signalHubSettings.activeSignalView.value === 'graph' ? 'Signals and consumers' : 'All signals') }}</strong>
                </div>
                <small>{{ t(signalHubSettings.activeSignalView.value === 'graph' ? 'Raw and accepted signal routes from the current Signal Hub surface.' : 'Source-scoped pause, mute, disable and resume controls.') }}</small>
                <nav class="settings-signal-view-tabs" :aria-label="t('Signal Hub views')">
                  <button
                    v-for="view in signalHubSettings.signalViewTabs.value"
                    :key="view.id"
                    type="button"
                    class="settings-signal-view-tab"
                    :class="{ active: signalHubSettings.activeSignalView.value === view.id }"
                    :aria-pressed="signalHubSettings.activeSignalView.value === view.id"
                    @click="signalHubSettings.handleSelectSignalView(view.id)"
                  >
                    <Icon :icon="view.id === 'graph' ? 'tabler:route' : 'tabler:table'" />
                    <span>{{ t(view.label) }}</span>
                    <strong>{{ view.count }}</strong>
                  </button>
                </nav>
              </header>

              <nav
                v-if="signalHubSettings.activeSignalView.value === 'graph'"
                class="settings-signal-category-tabs"
                :aria-label="t('Signal categories')"
              >
                <button
                  v-for="category in signalHubSettings.graphSourceTabs.value"
                  :key="category.id"
                  type="button"
                  class="settings-signal-category-tab"
                  :class="{ active: signalHubSettings.selectedGraphSourceCode.value === category.id }"
                  :aria-pressed="signalHubSettings.selectedGraphSourceCode.value === category.id"
                  @click="signalHubSettings.handleSelectGraphSource(category.id)"
                >
                  <span>{{ t(category.label) }}</span>
                  <strong>{{ category.count }}</strong>
                </button>
              </nav>

              <nav v-else class="settings-signal-category-tabs" :aria-label="t('Signal inventory categories')">
                <button
                  v-for="category in signalHubSettings.inventorySourceTabs.value"
                  :key="category.id"
                  type="button"
                  class="settings-signal-category-tab"
                  :class="{ active: signalHubSettings.selectedInventorySourceCode.value === category.id }"
                  :aria-pressed="signalHubSettings.selectedInventorySourceCode.value === category.id"
                  @click="signalHubSettings.handleSelectInventorySource(category.id)"
                >
                  <span>{{ t(category.label) }}</span>
                  <strong>{{ category.count }}</strong>
                </button>
              </nav>

              <div v-if="signalHubSettings.activeSignalView.value === 'graph'" class="settings-signal-graph">
                <article
                  v-for="route in signalHubSettings.filteredSignalConsumerGraph.value"
                  :key="route.source.code"
                  class="settings-signal-route"
                >
                  <div class="settings-signal-node settings-signal-node--source">
                    <i class="settings-provider-icon" aria-hidden="true">
                      <Icon :icon="sourceIcon(route.source)" />
                    </i>
                    <span>
                      <strong>{{ route.source.display_name }}</strong>
                      <small>{{ route.source.code }}</small>
                    </span>
                    <em :class="`is-${sourceStateTone(route.state)}`">{{ t(route.state) }}</em>
                  </div>

                  <div class="settings-signal-node settings-signal-node--patterns">
                    <code>{{ route.raw_pattern }}</code>
                    <code>{{ route.accepted_pattern }}</code>
                  </div>

                  <div class="settings-signal-node settings-signal-node--targets">
                    <span
                      v-for="target in route.targets"
                      :key="`${target.kind}:${target.id}`"
                      class="settings-signal-chip"
                      :class="[`is-${target.kind}`, `tone-${signalTargetTone(target)}`]"
                    >
                      <Icon :icon="signalTargetIcon(target.kind)" />
                      {{ target.label }}
                    </span>
                  </div>
                </article>
              </div>

              <div v-else class="settings-signal-table-scroll">
                <table class="settings-signal-table">
                  <thead>
                    <tr>
                      <th scope="col">{{ t('Signal') }}</th>
                      <th scope="col">{{ t('State') }}</th>
                      <th scope="col">{{ t('Consumed by') }}</th>
                      <th scope="col">{{ t('Health') }}</th>
                      <th scope="col">{{ t('Controls') }}</th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr
                      v-for="row in signalHubSettings.filteredSignalInventoryRows.value"
                      :key="row.source.code"
                    >
                      <td>
                        <div class="settings-signal-identity">
                          <i class="settings-provider-icon" aria-hidden="true">
                            <Icon :icon="sourceIcon(row.source)" />
                          </i>
                          <span>
                            <strong>{{ row.source.display_name }}</strong>
                            <small>{{ row.raw_pattern }}</small>
                          </span>
                        </div>
                      </td>
                      <td>
                        <span class="settings-signal-state" :class="`is-${sourceStateTone(row.state)}`">
                          {{ t(row.state) }}
                        </span>
                        <small>{{ row.active_policies.length }} {{ t('policies') }}</small>
                      </td>
                      <td>
                        <div class="settings-signal-chip-list">
                          <span
                            v-for="target in row.targets"
                            :key="`${target.kind}:${target.id}`"
                            class="settings-signal-chip"
                            :class="[`is-${target.kind}`, `tone-${signalTargetTone(target)}`]"
                          >
                            {{ target.label }}
                          </span>
                        </div>
                      </td>
                      <td>
                        <div class="settings-signal-health">
                          <span
                            class="settings-signal-state"
                            :class="`is-${healthTone(row.health?.level ?? 'unknown')}`"
                          >
                            {{ t(row.health?.level ?? 'unknown') }}
                          </span>
                          <small>{{ row.health?.summary ?? t('No health history') }}</small>
                          <small>
                            {{ row.connection_count }} {{ t('connections') }} ·
                            {{ row.runtime_states.length }} {{ t('runtimes') }} ·
                            {{ row.capabilities.length }} {{ t('capabilities') }}
                          </small>
                        </div>
                      </td>
                      <td>
                        <div class="settings-signal-controls">
                          <button type="button" class="icon-button" :title="t('Pause')" :aria-label="`${t('Pause')} ${row.source.display_name}`" :disabled="!row.source.supports_pause || row.state === 'paused' || signalHubSettings.isUpdatingSignalControls.value" @click="signalHubSettings.handlePauseSourceSignals(row.source.code)">
                            <Icon icon="tabler:player-pause" />
                          </button>
                          <button type="button" class="icon-button" :title="t('Resume')" :aria-label="`${t('Resume')} ${row.source.display_name}`" :disabled="row.state !== 'paused' || signalHubSettings.isUpdatingSignalControls.value" @click="signalHubSettings.handleResumeSourceSignals(row.source.code)">
                            <Icon icon="tabler:player-play" />
                          </button>
                          <button type="button" class="icon-button" :title="t('Mute')" :aria-label="`${t('Mute')} ${row.source.display_name}`" :disabled="!row.source.supports_mute || row.state === 'muted' || signalHubSettings.isUpdatingSignalControls.value" @click="signalHubSettings.handleMuteSourceSignals(row.source.code)">
                            <Icon icon="tabler:volume-off" />
                          </button>
                          <button type="button" class="icon-button" :title="t('Unmute')" :aria-label="`${t('Unmute')} ${row.source.display_name}`" :disabled="row.state !== 'muted' || signalHubSettings.isUpdatingSignalControls.value" @click="signalHubSettings.handleUnmuteSourceSignals(row.source.code)">
                            <Icon icon="tabler:volume" />
                          </button>
                          <button type="button" class="icon-button danger-icon-button" :title="t('Disable')" :aria-label="`${t('Disable')} ${row.source.display_name}`" :disabled="row.state === 'disabled' || signalHubSettings.isUpdatingSignalControls.value" @click="signalHubSettings.handleDisableSource(row.source.code)">
                            <Icon icon="tabler:circle-off" />
                          </button>
                          <button type="button" class="icon-button" :title="t('Enable')" :aria-label="`${t('Enable')} ${row.source.display_name}`" :disabled="(row.state !== 'disabled' && row.state !== 'off') || signalHubSettings.isUpdatingSignalControls.value" @click="signalHubSettings.handleEnableSource(row.source.code)">
                            <Icon icon="tabler:power" />
                          </button>
                        </div>
                      </td>
                    </tr>
                  </tbody>
                </table>
              </div>
            </section>
          </template>
        </section>

        <AISettingsPanel
          v-else-if="store.selectedSection === 'ai'"
          :surface="aiSettings"
        />
      </main>
    </div>

    <IntegrationConnectionWizard
      :default-provider-id="integrationsSettings.connectWizardProviderId.value"
      :open="integrationsSettings.isConnectWizardOpen.value"
      :selected-account="integrationsSettings.connectWizardSelectedAccount.value"
      @close="integrationsSettings.closeConnectWizard"
    />
  </section>
</template>
