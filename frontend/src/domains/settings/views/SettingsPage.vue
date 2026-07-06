<script setup lang="ts">
import { watch } from 'vue'
import { useI18n } from '../../../platform/i18n'
import IntegrationConnectionWizard from '../../../shared/integrationSetup/IntegrationConnectionWizard.vue'
import Icon from '../../../shared/ui/Icon.vue'
import { useToast } from '../../../shared/ui'
import AISettingsPanel from '../components/AISettingsPanel.vue'
import { useSettingsPageSurface } from '../queries/useSettingsPageSurface'
import type { AccountServiceRow } from '../queries/useIntegrationsSettingsSurface'
import type { ApplicationSetting } from '../types/settings'

const { t } = useI18n()
const toast = useToast()
const {
  aiSettings,
  applicationSettings,
  integrationsSettings,
  languageSettings,
  realtimeStatus,
  settingsOverviewCards,
  settingsTreeGroups,
  store,
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

function updateSelectedAccountLabel(event: Event) {
  integrationsSettings.handleSelectedAccountLabelInput(eventValue(event))
}

function saveSelectedAccountLabel() {
  void integrationsSettings.handleSaveSelectedAccountLabel()
}

</script>

<template>
  <section class="settings-page">
    <section class="settings-status-strip" :aria-label="t('Settings overview')">
      <article
        v-for="card in settingsOverviewCards"
        :key="card.id"
        class="settings-status-tile"
        :class="`tone-${card.tone}`"
      >
        <Icon :icon="card.icon" />
        <span>{{ t(card.label) }}</span>
        <strong>{{ card.value }}</strong>
        <button
          v-if="card.id === 'realtime' && realtimeStatus.canTriggerReconnect"
          type="button"
          class="icon-button"
          :aria-label="t('Reconnect')"
          @click="realtimeStatus.requestReconnect()"
        >
          <Icon icon="tabler:refresh" />
        </button>
      </article>
    </section>

    <div class="settings-workbench">
      <nav class="settings-tree" :aria-label="t('Settings sections')">
        <header class="settings-tree-header">
          <span>{{ t('Settings') }}</span>
          <strong>{{ t('Control Center') }}</strong>
        </header>

        <section
          v-for="group in settingsTreeGroups"
          :key="group.label"
          class="settings-tree-group"
        >
          <h2>{{ t(group.label) }}</h2>
          <button
            v-for="item in group.items"
            :key="item.id"
            type="button"
            :class="{ active: store.selectedSection === item.id }"
            @click="store.selectSection(item.id)"
          >
            <Icon class="tree-icon" :icon="item.icon" />
            <span class="settings-tree-copy">
              <strong>{{ t(item.label) }}</strong>
              <small>{{ t(item.description) }}</small>
            </span>
            <em v-if="item.meta">{{ item.meta }}</em>
          </button>
        </section>
      </nav>

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
                  <Icon :icon="row.icon" />
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
                  <div>
                    <span>{{ selectedAccountSummary.providerLabel }}</span>
                    <h3>{{ selectedAccountSummary.displayName }}</h3>
                    <p>{{ selectedAccountSummary.email || selectedAccountSummary.account.external_account_id }}</p>
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
                  </article>
                </section>

                <footer class="settings-account-actions">
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
                    {{ t('Delete') }}
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
          class="settings-section"
        >
          <header class="settings-section-toolbar">
            <div>
              <h3>{{ t('Signal Hub') }}</h3>
              <p>{{ t('Signal source controls stay under Settings; provider account setup stays in Accounts.') }}</p>
            </div>
          </header>
          <div class="settings-note-panel">
            <Icon icon="tabler:database-import" />
            <div>
              <strong>{{ t('Signal Hub contracts are preserved') }}</strong>
              <p>{{ t('Profiles, source controls, runtime health and replay flows continue to live in the Settings domain surfaces.') }}</p>
            </div>
          </div>
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
