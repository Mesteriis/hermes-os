<script setup lang="ts">
import Icon from '../../../shared/ui/Icon.vue'
import { useI18n } from '../../../platform/i18n'
import SignalHubSourcesTab from './SignalHubSourcesTab.vue'
import SignalHubProfilesPoliciesTab from './SignalHubProfilesPoliciesTab.vue'
import SignalHubOperationsTab from './SignalHubOperationsTab.vue'
import { useSignalHubSettingsController } from './useSignalHubSettingsController'

const { t } = useI18n()
const state = useSignalHubSettingsController()
</script>

<template>
  <div class="settings-page">
    <section class="panel settings-list-panel settings-primary-pane">
      <header class="panel-title-row">
        <div>
          <h2>{{ t('Signal Hub') }}</h2>
          <p>{{ t('Source control, recovery fixture and signal runtime state.') }}</p>
        </div>
        <button
          type="button"
          class="hermes-btn hermes-btn--outline"
          :disabled="state.isRestoringFixture.value"
          @click="state.handleRestoreFixture"
        >
          <Icon icon="tabler:restore" />
          {{ state.isRestoringFixture.value ? t('Restoring') : t('Restore Fixture') }}
        </button>
      </header>

      <div class="signal-tabs" role="tablist" :aria-label="t('Signal Hub sections')">
        <button
          v-for="tab in state.tabs"
          :key="tab.id"
          type="button"
          :class="{ active: state.activeTab.value === tab.id }"
          @click="state.activeTab.value = tab.id"
        >
          <Icon :icon="tab.icon" />
          <span>{{ t(tab.label) }}</span>
        </button>
      </div>

      <div class="signal-summary-strip">
        <div><strong>{{ state.sources.value.length }}</strong><span>{{ t('Sources') }}</span></div>
        <div><strong>{{ state.enabledCount.value }}</strong><span>{{ t('Enabled') }}</span></div>
        <div>
          <strong>{{ state.activeRuntimeCount.value }}/{{ state.runtimeCount.value }}</strong>
          <span>{{ t('Runtime') }}</span>
        </div>
        <div><strong>{{ state.replayCount.value }}</strong><span>{{ t('Replay') }}</span></div>
        <div><strong>{{ state.connectedCount.value }}</strong><span>{{ t('Connected') }}</span></div>
        <div><strong>{{ state.unhealthyCount.value }}</strong><span>{{ t('Attention') }}</span></div>
        <div><strong>{{ state.replayPendingCount.value }}</strong><span>{{ t('Replay Queue') }}</span></div>
        <div>
          <strong>{{ state.activeProfile.value?.display_name ?? t('None') }}</strong>
          <span>{{ t('Active Profile') }}</span>
        </div>
      </div>

      <SignalHubSourcesTab v-if="state.activeTab.value === 'sources'" :state="state" />
      <SignalHubProfilesPoliciesTab
        v-else-if="state.activeTab.value === 'profiles' || state.activeTab.value === 'policies'"
        :state="state"
      />
      <SignalHubOperationsTab v-else :state="state" />
    </section>
  </div>
</template>

<style src="./SignalHubSettings.css"></style>
