<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type { CommunicationsSettingsSurface } from '../queries/useCommunicationsSettingsSurface'

const props = defineProps<{
  surface: CommunicationsSettingsSurface
}>()

const { t } = useI18n()

function eventValue(event: Event): string {
  return event.target instanceof HTMLInputElement ? event.target.value : ''
}

function eventChecked(event: Event): boolean {
  return event.target instanceof HTMLInputElement ? event.target.checked : false
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
        </template>
        <div v-else class="settings-empty-state">
          <Icon :icon="surface.isLoading.value ? 'tabler:loader-2' : 'tabler:mail-off'" />
          <strong>{{ surface.isLoading.value ? t('Loading mail settings') : t('Select a mail account') }}</strong>
        </div>
      </section>
    </section>
  </section>
</template>
