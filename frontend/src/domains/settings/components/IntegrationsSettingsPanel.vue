<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import IntegrationConnectionWizard from '../../../shared/integrationSetup/IntegrationConnectionWizard.vue'
import Icon from '../../../shared/ui/Icon.vue'
import { useIntegrationsSettingsPanelController } from '../queries/useIntegrationsSettingsPanelController'
import type { useIntegrationsSettingsSurface } from '../queries/useIntegrationsSettingsSurface'

type IntegrationsSettingsSurface = ReturnType<typeof useIntegrationsSettingsSurface>

const props = defineProps<{
  surface: IntegrationsSettingsSurface
}>()

const { t } = useI18n()

const {
  accountGroups,
  hasAccounts,
  selectedAccountSummary,
  activeMailAction,
  handleAddAccount,
  handleSelectAccount,
  handleToggleSelectedAccount,
  handleUpdateSelectedAccountLabel,
  handleSaveSelectedAccountLabel,
  handleToggleSelectedService,
  handleRunSelectedServiceNow,
  handleRunSelectedServiceModeAction,
  handleOpenCredentialRecovery,
  handleOpenSelectedServiceSetup,
  handleExportAccount,
  handleLogoutAccount,
  handleDeleteAccount,
  handleCloseConnectWizard,
  isConnectWizardOpen,
  connectWizardProviderId,
  connectWizardSelectedAccount,
} = useIntegrationsSettingsPanelController({ surface: props.surface })
</script>

<template>
  <section class="settings-section settings-accounts-section">
    <header class="settings-section-toolbar">
      <div>
        <h3>{{ t('Accounts') }}</h3>
        <p>{{ t('Provider records from backend settings and integration APIs.') }}</p>
      </div>
      <button type="button" class="primary-button" @click="handleAddAccount">
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
            @click="handleSelectAccount(row.account.account_id)"
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

        <div v-if="!hasAccounts.value" class="settings-empty-state">
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
                @change="handleToggleSelectedAccount"
              >
              <span />
              <strong>{{ selectedAccountSummary.accountToggleLabel }}</strong>
            </label>
          </header>

          <form class="settings-account-label-form" @submit.prevent="handleSaveSelectedAccountLabel">
            <label>
              <span>{{ t('Account label') }}</span>
              <input
                type="text"
                :value="selectedAccountSummary.labelDraft"
                :disabled="selectedAccountSummary.labelSaving"
                autocomplete="off"
                @input="handleUpdateSelectedAccountLabel"
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
                  @change="handleToggleSelectedService(service, $event)"
                >
                <span />
                <strong>{{ service.statusText }}</strong>
              </label>
              <button
                v-if="service.canRunNow"
                type="button"
                class="secondary-button settings-service-row__action"
                :disabled="service.isBusy"
                @click="handleRunSelectedServiceNow(service)"
              >
                <Icon icon="tabler:refresh" />
                {{ service.isBusy ? t('Syncing') : service.runNowLabel }}
              </button>
              <button
                v-if="service.canRunModeAction"
                type="button"
                class="secondary-button settings-service-row__action"
                :disabled="service.isBusy"
                @click="handleRunSelectedServiceModeAction(service)"
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
              @click="handleOpenCredentialRecovery"
            >
              <Icon icon="tabler:key" />
              {{ t('Восстановить аккаунт') }}
            </button>
            <button
              type="button"
              class="secondary-button"
              :disabled="!selectedAccountSummary.canOpenSetupAction"
              :title="selectedAccountSummary.canOpenSetupAction ? selectedAccountSummary.selectedInspectorActionLabel : t('No self-serve setup route for this provider yet')"
              @click="handleOpenSelectedServiceSetup"
            >
              <Icon icon="tabler:route" />
              {{ selectedAccountSummary.selectedInspectorActionLabel }}
            </button>
            <button
              v-if="selectedAccountSummary.canManageMail"
              type="button"
              class="secondary-button"
              :disabled="activeMailAction.value === selectedAccountSummary.account.account_id"
              @click="handleExportAccount(selectedAccountSummary.account.account_id)"
            >
              <Icon icon="tabler:download" />
              {{ t('Export') }}
            </button>
            <button
              v-if="selectedAccountSummary.canManageMail"
              type="button"
              class="secondary-button"
              :disabled="activeMailAction.value === selectedAccountSummary.account.account_id"
              @click="handleLogoutAccount(selectedAccountSummary.account.account_id)"
            >
              <Icon icon="tabler:logout" />
              {{ t('Logout') }}
            </button>
            <button
              v-if="selectedAccountSummary.canManageMail"
              type="button"
              class="danger-button"
              :disabled="activeMailAction.value === selectedAccountSummary.account.account_id"
              @click="handleDeleteAccount(selectedAccountSummary.account.account_id)"
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

    <IntegrationConnectionWizard
      :default-provider-id="connectWizardProviderId.value"
      :open="isConnectWizardOpen.value"
      :selected-account="connectWizardSelectedAccount.value"
      @close="handleCloseConnectWizard"
    />
  </section>
</template>
