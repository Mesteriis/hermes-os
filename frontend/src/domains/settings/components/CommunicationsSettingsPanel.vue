<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type { CommunicationsSettingsSurface } from '../queries/useCommunicationsSettingsSurface'
import {
  MAX_MAIL_POLL_INTERVAL_SECONDS,
  MIN_MAIL_POLL_INTERVAL_SECONDS,
  MAX_MAIL_SYNC_WINDOWS,
  MAX_MAIL_BATCH_SIZE,
} from '../../../shared/mailSync/types'
import { useCommunicationsSettingsPanelController } from '../queries/useCommunicationsSettingsPanelController'

const props = defineProps<{
  surface: CommunicationsSettingsSurface
}>()

const { t } = useI18n()

const {
  handleDegradationThresholdInput,
  handleTelegramReadReceiptReportsChange,
  handleSelectMailAccount,
  handleSelectedMailSyncToggle,
  handleBatchSizeDraftInput,
  handlePollIntervalDraftInput,
  handleWindowsDraftInput,
  handleContentEgressBodyToggle,
  handleContentEgressAttachmentsToggle,
  handleContentEgressExtractedTextToggle,
  handlePolicySelection,
  handleNewPolicy,
  handlePolicyNameInput,
  handleDeliveryAccountInput,
  handleRecipientInput,
  handleSeverityInput,
  handleMaxSendsInput,
  handleQuietHoursStartInput,
  handleQuietHoursEndInput,
  handleExpiryInput,
  handlePolicyEnabledInput,
  handleIncludeMessageBodyInput,
  handleIncludeAttachmentsInput,
  handleSubjectTemplateInput,
  handleBodyTemplateInput,
  handleResourceRoleInput,
  handleResourceLocalFolderInput,
  handleSaveDegradationThreshold,
  handleSaveSelectedMailSyncSettings,
  handleSaveSensitiveForwardingPolicy,
  handleRemoveSelectedSensitiveForwardingPolicy,
} = useCommunicationsSettingsPanelController({
  surface: props.surface,
})

const semanticRoles: Array<{ value: string; label: string }> = [
  { value: 'inbox', label: 'Inbox' },
  { value: 'sent', label: 'Sent' },
  { value: 'drafts', label: 'Drafts' },
  { value: 'archive', label: 'Archive' },
  { value: 'trash', label: 'Trash' },
  { value: 'junk', label: 'Junk' },
  { value: 'all', label: 'All mail' },
  { value: 'flagged', label: 'Flagged' },
  { value: 'important', label: 'Important' },
  { value: 'user', label: 'User label' },
]

function mappingSourceLabel(source: string): string {
  return source === 'manual' ? t('Manual override') : t('Discovered')
}
</script>

<template>
  <section class="settings-section settings-communications-section">
    <header class="settings-section-toolbar">
      <div>
        <h3>{{ t('Communications') }}</h3>
        <p>{{ t('Provider reliability policy and mail synchronization settings.') }}</p>
      </div>
    </header>

    <nav class="settings-communications-tabs" :aria-label="t('Communications settings')">
      <button type="button" class="settings-communications-tab active" aria-current="page">
        <Icon icon="tabler:mail" />
        {{ t('Mail') }}
      </button>
    </nav>

    <section class="settings-communications-panel">
      <header>
        <div>
          <span>{{ t('Reliability policy') }}</span>
          <strong>{{ t('Provider degradation') }}</strong>
        </div>
        <small>{{ t('A successful or skipped run clears the consecutive failure counter.') }}</small>
      </header>

      <div v-if="surface.degradationThresholdSetting.value" class="settings-communications-policy">
        <label>
          <span>{{ t('Failures before degradation') }}</span>
          <input
            type="number"
            min="1"
            max="10"
            :value="surface.degradationThresholdDraft.value"
            @input="handleDegradationThresholdInput"
          />
        </label>
        <p>{{ t(surface.degradationThresholdSetting.value.description) }}</p>
        <button
          type="button"
          class="primary-button"
          :disabled="!surface.degradationThresholdSetting.value.is_editable || !surface.degradationThresholdSetting.value || !surface.degradationThresholdDraft.value"
          @click="handleSaveDegradationThreshold()"
        >
          {{ t('Save policy') }}
        </button>
      </div>
    </section>

    <section v-if="surface.telegramReadReceiptReportsSetting.value" class="settings-communications-panel">
      <header>
        <div>
          <span>{{ t('Telegram privacy') }}</span>
          <strong>{{ t('Read reports') }}</strong>
        </div>
        <small>{{ t('A chat can override this default from its inspector.') }}</small>
      </header>
      <div class="settings-communications-policy">
        <label class="settings-switch">
          <input
            type="checkbox"
            :checked="surface.telegramReadReceiptReportsSetting.value.value === true"
            :disabled="!surface.telegramReadReceiptReportsSetting.value.is_editable"
            @change="handleTelegramReadReceiptReportsChange"
          />
          <span>{{ t('Send read reports to Telegram') }}</span>
        </label>
        <p>{{ t(surface.telegramReadReceiptReportsSetting.value.description) }}</p>
        <p>{{ t('Telegram delivery receipts are provider-managed and cannot be suppressed through TDLib.') }}</p>
      </div>
    </section>

    <section class="settings-communications-mail-grid">
      <aside class="settings-communications-panel settings-communications-accounts">
        <header>
          <div>
            <span>{{ t('Mail accounts') }}</span>
            <strong>{{ t('Provider sync') }}</strong>
          </div>
        </header>
        <div v-if="surface.mailAccounts.value.length === 0" class="settings-empty-state">
          <Icon icon="tabler:mail-off" />
          <strong>{{ t('No mail accounts connected') }}</strong>
        </div>
        <button
          v-for="account in surface.mailAccounts.value"
          :key="account.account_id"
          type="button"
          class="settings-choice"
          :class="{ active: surface.selectedMailAccount.value?.account_id === account.account_id }"
          @click="handleSelectMailAccount(account.account_id)"
        >
          <Icon icon="tabler:mail" />
          <span>
            <strong>{{ account.display_name }}</strong>
            <small>{{ account.provider_kind }}</small>
          </span>
        </button>
      </aside>

      <section class="settings-communications-panel settings-communications-detail">
        <template v-if="surface.selectedMailAccount.value && surface.selectedSyncSettings.value">
          <header>
            <div>
              <span>{{ t('Mail') }}</span>
              <strong>{{ surface.selectedMailAccount.value.display_name }}</strong>
            </div>
            <label class="settings-switch">
              <input
                type="checkbox"
                :checked="surface.selectedSyncSettings.value.sync_enabled"
                :disabled="surface.syncSaving.value"
                @change="handleSelectedMailSyncToggle"
              />
              <span>{{ surface.selectedSyncSettings.value.sync_enabled ? t('Sync enabled') : t('Sync paused') }}</span>
            </label>
          </header>

          <dl v-if="surface.selectedSyncStatus.value" class="settings-communications-facts">
            <div><dt>{{ t('Current status') }}</dt><dd>{{ surface.selectedSyncStatus.value.status }}</dd></div>
            <div><dt>{{ t('Consecutive failures') }}</dt><dd>{{ surface.selectedSyncStatus.value.consecutive_failures }}</dd></div>
          </dl>

          <div class="settings-communications-fields">
            <label>
              <span>{{ t('Batch size') }}</span>
              <input
                type="number"
                min="1"
                :max="MAX_MAIL_BATCH_SIZE"
                :value="surface.batchSizeDraft.value"
                @input="handleBatchSizeDraftInput"
              />
            </label>
            <label>
              <span>{{ t('Sync windows') }}</span>
              <input
                type="number"
                min="1"
                :max="MAX_MAIL_SYNC_WINDOWS"
                :value="surface.windowsDraft.value"
                @input="handleWindowsDraftInput"
              />
            </label>
            <label>
              <span>{{ t('Poll interval (seconds)') }}</span>
              <input
                type="number"
                :min="MIN_MAIL_POLL_INTERVAL_SECONDS"
                :max="MAX_MAIL_POLL_INTERVAL_SECONDS"
                :value="surface.pollIntervalDraft.value"
                @input="handlePollIntervalDraftInput"
              />
            </label>
          </div>
          <button
            type="button"
            class="primary-button"
            :disabled="surface.syncSaving.value"
            @click="handleSaveSelectedMailSyncSettings()"
          >
            {{ t('Save mail settings') }}
          </button>

          <section class="settings-communications-egress" aria-labelledby="mail-content-egress-title">
            <header>
              <div>
                <span>{{ t('External content access') }}</span>
                <strong id="mail-content-egress-title">{{ t('Content egress') }}</strong>
              </div>
              <small>{{ t('Off by default. Enable only for approved automation on this account.') }}</small>
            </header>
            <div v-if="surface.contentEgressLoading.value" class="settings-empty-state">
              <Icon icon="tabler:loader-2" />
              <strong>{{ t('Loading content access settings') }}</strong>
            </div>
            <div v-else-if="surface.selectedContentEgress.value" class="settings-communications-egress__switches">
              <label class="settings-switch">
                <input
                  type="checkbox"
                  :checked="surface.selectedContentEgress.value.body"
                  :disabled="surface.contentEgressSaving.value"
                  @change="handleContentEgressBodyToggle"
                />
                <span>{{ t('Message body') }}</span>
              </label>
              <label class="settings-switch">
                <input
                  type="checkbox"
                  :checked="surface.selectedContentEgress.value.attachments"
                  :disabled="surface.contentEgressSaving.value"
                  @change="handleContentEgressAttachmentsToggle"
                />
                <span>{{ t('Attachments') }}</span>
              </label>
              <label class="settings-switch">
                <input
                  type="checkbox"
                  :checked="surface.selectedContentEgress.value.extracted_text"
                  :disabled="surface.contentEgressSaving.value"
                  @change="handleContentEgressExtractedTextToggle"
                />
                <span>{{ t('Extracted text') }}</span>
              </label>
            </div>
          </section>

          <section class="settings-communications-egress" aria-labelledby="sensitive-forwarding-title">
            <header>
              <div>
                <span>{{ t('Automation policy') }}</span>
                <strong id="sensitive-forwarding-title">{{ t('Sensitive forwarding') }}</strong>
              </div>
              <small>{{ t('Disabled by default. Source body and clean attachments require separate policy opt-ins plus source-account content permissions; unsafe attachments are always withheld.') }}</small>
            </header>
            <div v-if="surface.sensitiveForwardingPoliciesLoading.value" class="settings-empty-state">
              <Icon icon="tabler:loader-2" />
              <strong>{{ t('Loading sensitive forwarding policies') }}</strong>
            </div>
            <template v-else>
              <div class="settings-communications-policy-list">
                <button
                  v-for="policy in surface.sensitiveForwardingPolicies.value"
                  :key="policy.policy_id"
                  type="button"
                  class="settings-choice"
                  :class="{ active: surface.selectedSensitiveForwardingPolicyId.value === policy.policy_id }"
                  @click="handlePolicySelection(policy.policy_id)"
                >
                  <Icon :icon="policy.enabled ? 'tabler:shield-check' : 'tabler:shield-off'" />
                  <span><strong>{{ policy.name }}</strong><small>{{ policy.minimum_severity }} · {{ policy.fixed_recipients.length }} recipients</small></span>
                </button>
                <button type="button" class="secondary-button" @click="handleNewPolicy">
                  <Icon icon="tabler:plus" />
                  {{ t('New policy') }}
                </button>
              </div>

              <div class="settings-communications-fields">
                <label>
                  <span>{{ t('Policy name') }}</span>
                  <input :value="surface.sensitiveForwardingDraft.value.name" @input="handlePolicyNameInput" />
                </label>
                <label>
                  <span>{{ t('Delivery account') }}</span>
                  <select :value="surface.sensitiveForwardingDraft.value.delivery_account_id" @change="handleDeliveryAccountInput">
                    <option v-for="account in surface.mailAccounts.value" :key="account.account_id" :value="account.account_id">{{ account.display_name }}</option>
                  </select>
                </label>
                <label>
                  <span>{{ t('Fixed recipients') }}</span>
                  <input :value="surface.sensitiveForwardingDraft.value.fixed_recipients.join(', ')" :placeholder="t('security@example.com, owner@example.com')" @input="handleRecipientInput" />
                </label>
                <label>
                  <span>{{ t('Minimum severity') }}</span>
                  <select :value="surface.sensitiveForwardingDraft.value.minimum_severity" @change="handleSeverityInput">
                    <option value="low">low</option><option value="medium">medium</option><option value="high">high</option><option value="critical">critical</option>
                  </select>
                </label>
                <label>
                  <span>{{ t('Maximum sends per hour') }}</span>
                  <input type="number" min="1" :value="surface.sensitiveForwardingDraft.value.max_sends_per_hour" @input="handleMaxSendsInput" />
                </label>
                <label>
                  <span>{{ t('Quiet hours start (UTC)') }}</span>
                  <input type="time" :value="surface.sensitiveForwardingQuietHour('start')" @input="handleQuietHoursStartInput" />
                </label>
                <label>
                  <span>{{ t('Quiet hours end (UTC)') }}</span>
                  <input type="time" :value="surface.sensitiveForwardingQuietHour('end')" @input="handleQuietHoursEndInput" />
                </label>
                <label>
                  <span>{{ t('Policy expiry (UTC)') }}</span>
                  <input type="datetime-local" :value="surface.sensitiveForwardingExpiryValue()" @input="handleExpiryInput" />
                </label>
                <label class="settings-switch">
                  <input type="checkbox" :checked="surface.sensitiveForwardingDraft.value.enabled" @change="handlePolicyEnabledInput" />
                  <span>{{ t('Policy enabled') }}</span>
                </label>
                <label class="settings-switch">
                  <input type="checkbox" :checked="surface.sensitiveForwardingDraft.value.include_message_body" @change="handleIncludeMessageBodyInput" />
                  <span>{{ t('Include message body when source content access is enabled') }}</span>
                </label>
                <label class="settings-switch">
                  <input type="checkbox" :checked="surface.sensitiveForwardingDraft.value.include_attachments" @change="handleIncludeAttachmentsInput" />
                  <span>{{ t('Include clean attachments when source attachment access is enabled') }}</span>
                </label>
              </div>
              <label class="settings-communications-policy-template">
                <span>{{ t('Notification subject template') }}</span>
                <input :value="surface.sensitiveForwardingDraft.value.subject_template" @input="handleSubjectTemplateInput" />
              </label>
              <label class="settings-communications-policy-template">
                <span>{{ t('Notification body template') }}</span>
                <textarea rows="3" :value="surface.sensitiveForwardingDraft.value.body_template" @input="handleBodyTemplateInput" />
              </label>
              <div class="settings-communications-policy-actions">
                <button type="button" class="primary-button" :disabled="surface.sensitiveForwardingSaving.value" @click="handleSaveSensitiveForwardingPolicy">
                  {{ t('Save sensitive forwarding policy') }}
                </button>
                <button
                  v-if="surface.selectedSensitiveForwardingPolicyId.value"
                  type="button"
                  class="secondary-button"
                  :disabled="surface.sensitiveForwardingDeleting.value"
                  @click="handleRemoveSelectedSensitiveForwardingPolicy"
                >
                  {{ t('Delete policy') }}
                </button>
              </div>
            </template>
          </section>
        </template>
        <div v-else class="settings-empty-state">
          <Icon :icon="surface.isLoading.value ? 'tabler:loader-2' : 'tabler:mail-off'" />
          <strong>{{ surface.isLoading.value ? t('Loading mail settings') : t('Select a mail account') }}</strong>
        </div>
      </section>
    </section>

    <section class="settings-communications-panel settings-provider-resources">
      <header>
        <div>
          <span>{{ t('Provider folders & labels') }}</span>
          <strong>{{ t('Mail provider mapping') }}</strong>
        </div>
        <small>{{ t('Set only Inbox, Sent, Drafts, Trash, Archive and Junk when they match the provider folder. Leave marketing/category labels unassigned.') }}</small>
      </header>

      <div v-if="surface.providerResourcesLoading.value" class="settings-empty-state">
        <Icon icon="tabler:loader-2" />
        <strong>{{ t('Loading provider folders and labels') }}</strong>
      </div>
      <div v-else-if="surface.providerResources.value.length === 0" class="settings-empty-state">
        <Icon icon="tabler:folders-off" />
        <strong>{{ t('No provider folders or labels discovered yet') }}</strong>
      </div>
      <div v-else class="settings-provider-resources__list">
        <article v-for="resource in surface.providerResources.value" :key="resource.mapping_id" class="settings-provider-resource">
          <div class="settings-provider-resource__identity">
            <Icon :icon="resource.resource_kind === 'label' ? 'tabler:tag' : 'tabler:folder'" />
            <span>
              <strong>{{ resource.display_name }}</strong>
              <small>{{ resource.resource_kind }} · {{ mappingSourceLabel(resource.mapping_source) }}</small>
            </span>
          </div>
          <label>
            <span>{{ t('Role') }}</span>
            <select
              :value="resource.semantic_role ?? ''"
              :disabled="!resource.writable || surface.providerResourcesSaving.value"
              @change="handleResourceRoleInput(resource, $event)"
            >
              <option value="">{{ t('Unassigned') }}</option>
              <option v-for="role in semanticRoles" :key="role.value" :value="role.value">{{ t(role.label) }}</option>
            </select>
          </label>
          <label>
            <span>{{ t('Local folder') }}</span>
            <select
              :value="resource.local_folder_id ?? ''"
              :disabled="!resource.writable || surface.providerResourcesSaving.value || surface.localFoldersLoading.value"
              @change="handleResourceLocalFolderInput(resource, $event)"
            >
              <option value="">{{ t('No local folder') }}</option>
              <option v-for="folder in surface.localFolders.value" :key="folder.folder_id" :value="folder.folder_id">
                {{ folder.name }}
              </option>
            </select>
          </label>
          <small v-if="!resource.writable">{{ t('Read-only provider resource') }}</small>
        </article>
      </div>
    </section>

  </section>
</template>
