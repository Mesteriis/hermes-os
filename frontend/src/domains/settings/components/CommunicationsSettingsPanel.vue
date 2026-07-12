<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type { MailProviderSemanticRole } from '../../../shared/mailSync/providerResources'
import type { CommunicationsSettingsSurface } from '../queries/useCommunicationsSettingsSurface'

const props = defineProps<{
  surface: CommunicationsSettingsSurface
}>()

const { t } = useI18n()

function eventValue(event: Event): string {
  return event.target instanceof HTMLInputElement
    || event.target instanceof HTMLSelectElement
    || event.target instanceof HTMLTextAreaElement
    ? event.target.value
    : ''
}

function eventChecked(event: Event): boolean {
  return event.target instanceof HTMLInputElement ? event.target.checked : false
}

function commandCount(status: string): number {
  return props.surface.commandDiagnostics.value?.counts.find((item) => item.status === status)?.count ?? 0
}

function formatTimestamp(value: string | null): string {
  if (!value) return '—'
  const date = new Date(value)
  return Number.isFinite(date.getTime()) ? date.toLocaleString() : value
}

const semanticRoles: Array<{ value: MailProviderSemanticRole; label: string }> = [
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

function semanticRoleValue(event: Event): MailProviderSemanticRole | null {
  const value = event.target instanceof HTMLSelectElement ? event.target.value : ''
  return semanticRoles.some((role) => role.value === value) ? value as MailProviderSemanticRole : null
}

function localFolderValue(event: Event): string | null {
  const value = event.target instanceof HTMLSelectElement ? event.target.value : ''
  return value || null
}

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
            @input="surface.updateDegradationThreshold(eventValue($event))"
          />
        </label>
        <p>{{ t(surface.degradationThresholdSetting.value.description) }}</p>
        <button
          type="button"
          class="primary-button"
          :disabled="!surface.degradationThresholdSetting.value.is_editable || !surface.degradationThresholdSetting.value || !surface.degradationThresholdDraft.value"
          @click="surface.saveDegradationThreshold()"
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
            @change="surface.updateTelegramReadReceiptReports(eventChecked($event))"
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
          @click="surface.selectMailAccount(account.account_id)"
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
                @change="surface.toggleSelectedMailSync(eventChecked($event))"
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
              <input type="number" min="1" :value="surface.batchSizeDraft.value" @input="surface.batchSizeDraft.value = eventValue($event)" />
            </label>
            <label>
              <span>{{ t('Poll interval (seconds)') }}</span>
              <input type="number" min="1" :value="surface.pollIntervalDraft.value" @input="surface.pollIntervalDraft.value = eventValue($event)" />
            </label>
          </div>
          <button type="button" class="primary-button" :disabled="surface.syncSaving.value" @click="surface.saveSelectedMailSyncSettings()">
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
                  @change="surface.updateSelectedMailContentEgress('body', eventChecked($event))"
                />
                <span>{{ t('Message body') }}</span>
              </label>
              <label class="settings-switch">
                <input
                  type="checkbox"
                  :checked="surface.selectedContentEgress.value.attachments"
                  :disabled="surface.contentEgressSaving.value"
                  @change="surface.updateSelectedMailContentEgress('attachments', eventChecked($event))"
                />
                <span>{{ t('Attachments') }}</span>
              </label>
              <label class="settings-switch">
                <input
                  type="checkbox"
                  :checked="surface.selectedContentEgress.value.extracted_text"
                  :disabled="surface.contentEgressSaving.value"
                  @change="surface.updateSelectedMailContentEgress('extracted_text', eventChecked($event))"
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
                  @click="surface.selectSensitiveForwardingPolicy(policy.policy_id)"
                >
                  <Icon :icon="policy.enabled ? 'tabler:shield-check' : 'tabler:shield-off'" />
                  <span><strong>{{ policy.name }}</strong><small>{{ policy.minimum_severity }} · {{ policy.fixed_recipients.length }} recipients</small></span>
                </button>
                <button type="button" class="secondary-button" @click="surface.createSensitiveForwardingPolicy()">
                  <Icon icon="tabler:plus" />
                  {{ t('New policy') }}
                </button>
              </div>

              <div class="settings-communications-fields">
                <label>
                  <span>{{ t('Policy name') }}</span>
                  <input :value="surface.sensitiveForwardingDraft.value.name" @input="surface.updateSensitiveForwardingDraft({ name: eventValue($event) })" />
                </label>
                <label>
                  <span>{{ t('Delivery account') }}</span>
                  <select :value="surface.sensitiveForwardingDraft.value.delivery_account_id" @change="surface.updateSensitiveForwardingDraft({ delivery_account_id: eventValue($event) })">
                    <option v-for="account in surface.mailAccounts.value" :key="account.account_id" :value="account.account_id">{{ account.display_name }}</option>
                  </select>
                </label>
                <label>
                  <span>{{ t('Fixed recipients') }}</span>
                  <input :value="surface.sensitiveForwardingDraft.value.fixed_recipients.join(', ')" :placeholder="t('security@example.com, owner@example.com')" @input="surface.updateSensitiveForwardingRecipients(eventValue($event))" />
                </label>
                <label>
                  <span>{{ t('Minimum severity') }}</span>
                  <select :value="surface.sensitiveForwardingDraft.value.minimum_severity" @change="surface.updateSensitiveForwardingDraft({ minimum_severity: eventValue($event) as 'low' | 'medium' | 'high' | 'critical' })">
                    <option value="low">low</option><option value="medium">medium</option><option value="high">high</option><option value="critical">critical</option>
                  </select>
                </label>
                <label>
                  <span>{{ t('Maximum sends per hour') }}</span>
                  <input type="number" min="1" :value="surface.sensitiveForwardingDraft.value.max_sends_per_hour" @input="surface.updateSensitiveForwardingDraft({ max_sends_per_hour: Number(eventValue($event)) })" />
                </label>
                <label>
                  <span>{{ t('Quiet hours start (UTC)') }}</span>
                  <input type="time" :value="surface.sensitiveForwardingQuietHour('start')" @input="surface.updateSensitiveForwardingQuietHours(eventValue($event), surface.sensitiveForwardingQuietHour('end'))" />
                </label>
                <label>
                  <span>{{ t('Quiet hours end (UTC)') }}</span>
                  <input type="time" :value="surface.sensitiveForwardingQuietHour('end')" @input="surface.updateSensitiveForwardingQuietHours(surface.sensitiveForwardingQuietHour('start'), eventValue($event))" />
                </label>
                <label>
                  <span>{{ t('Policy expiry (UTC)') }}</span>
                  <input type="datetime-local" :value="surface.sensitiveForwardingExpiryValue()" @input="surface.updateSensitiveForwardingExpiry(eventValue($event))" />
                </label>
                <label class="settings-switch">
                  <input type="checkbox" :checked="surface.sensitiveForwardingDraft.value.enabled" @change="surface.updateSensitiveForwardingDraft({ enabled: eventChecked($event) })" />
                  <span>{{ t('Policy enabled') }}</span>
                </label>
                <label class="settings-switch">
                  <input type="checkbox" :checked="surface.sensitiveForwardingDraft.value.include_message_body" @change="surface.updateSensitiveForwardingDraft({ include_message_body: eventChecked($event) })" />
                  <span>{{ t('Include message body when source content access is enabled') }}</span>
                </label>
                <label class="settings-switch">
                  <input type="checkbox" :checked="surface.sensitiveForwardingDraft.value.include_attachments" @change="surface.updateSensitiveForwardingDraft({ include_attachments: eventChecked($event) })" />
                  <span>{{ t('Include clean attachments when source attachment access is enabled') }}</span>
                </label>
              </div>
              <label class="settings-communications-policy-template">
                <span>{{ t('Notification subject template') }}</span>
                <input :value="surface.sensitiveForwardingDraft.value.subject_template" @input="surface.updateSensitiveForwardingDraft({ subject_template: eventValue($event) })" />
              </label>
              <label class="settings-communications-policy-template">
                <span>{{ t('Notification body template') }}</span>
                <textarea rows="3" :value="surface.sensitiveForwardingDraft.value.body_template" @input="surface.updateSensitiveForwardingDraft({ body_template: eventValue($event) })" />
              </label>
              <div class="settings-communications-policy-actions">
                <button type="button" class="primary-button" :disabled="surface.sensitiveForwardingSaving.value" @click="surface.saveSensitiveForwardingPolicy()">
                  {{ t('Save sensitive forwarding policy') }}
                </button>
                <button
                  v-if="surface.selectedSensitiveForwardingPolicyId.value"
                  type="button"
                  class="secondary-button"
                  :disabled="surface.sensitiveForwardingDeleting.value"
                  @click="surface.removeSelectedSensitiveForwardingPolicy()"
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
              @change="surface.updateProviderResourceRole(resource, semanticRoleValue($event))"
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
              @change="surface.updateProviderResourceLocalFolder(resource, localFolderValue($event))"
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

    <section class="settings-communications-panel settings-command-diagnostics">
      <header>
        <div>
          <span>{{ t('Provider commands') }}</span>
          <strong>{{ t('Mail command queue diagnostics') }}</strong>
        </div>
        <button
          type="button"
          class="secondary-button"
          :disabled="surface.commandDiagnosticsRefreshing.value || !surface.selectedMailAccount.value"
          @click="surface.refreshCommandDiagnostics()"
        >
          <Icon :icon="surface.commandDiagnosticsRefreshing.value ? 'tabler:loader-2' : 'tabler:refresh'" />
          {{ t('Refresh') }}
        </button>
      </header>

      <div class="settings-command-diagnostics__counts" aria-label="Mail provider command status counts">
        <span v-for="status in ['queued', 'executing', 'retrying', 'completed', 'dead_letter']" :key="status">
          <strong>{{ commandCount(status) }}</strong>
          {{ status }}
        </span>
      </div>

      <label class="settings-command-diagnostics__filter">
        <span>{{ t('Status filter') }}</span>
        <select v-model="surface.commandDiagnosticsStatus.value">
          <option value="">{{ t('All statuses') }}</option>
          <option value="queued">queued</option>
          <option value="executing">executing</option>
          <option value="retrying">retrying</option>
          <option value="completed">completed</option>
          <option value="dead_letter">dead_letter</option>
        </select>
      </label>

      <div v-if="surface.commandDiagnosticsLoading.value" class="settings-empty-state">
        <Icon icon="tabler:loader-2" />
        <strong>{{ t('Loading command diagnostics') }}</strong>
      </div>
      <div v-else-if="!surface.commandDiagnostics.value?.items.length" class="settings-empty-state">
        <Icon icon="tabler:circle-check" />
        <strong>{{ t('No provider commands match this filter') }}</strong>
      </div>
      <div v-else class="settings-command-diagnostics__table-wrap">
        <table class="settings-command-diagnostics__table">
          <thead>
            <tr>
              <th>{{ t('Command') }}</th>
              <th>{{ t('Status') }}</th>
              <th>{{ t('Attempts') }}</th>
              <th>{{ t('Reconciliation') }}</th>
              <th>{{ t('Updated') }}</th>
              <th>{{ t('Last error') }}</th>
              <th>{{ t('Recovery') }}</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="command in surface.commandDiagnostics.value.items" :key="command.command_id">
              <td><strong>{{ command.command_kind }}</strong><small>{{ command.command_id }}</small></td>
              <td><span :data-status="command.status">{{ command.status }}</span></td>
              <td>{{ command.retry_count }} / {{ command.max_retries }}</td>
              <td>{{ command.reconciliation_status }}</td>
              <td><time :datetime="command.updated_at">{{ formatTimestamp(command.updated_at) }}</time></td>
              <td>{{ command.last_error || '—' }}</td>
              <td>
                <button
                  v-if="command.status === 'dead_letter'"
                  type="button"
                  class="secondary-button"
                  :disabled="surface.commandDiagnosticsRetrying.value"
                  @click="surface.retryMailProviderCommand(command.command_id)"
                >
                  <Icon :icon="surface.commandDiagnosticsRetrying.value ? 'tabler:loader-2' : 'tabler:refresh'" />
                  {{ t('Retry') }}
                </button>
                <span v-else>—</span>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </section>
  </section>
</template>
