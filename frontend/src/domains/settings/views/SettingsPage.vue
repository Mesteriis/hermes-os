<script setup lang="ts">
import { watch } from 'vue'
import { useI18n } from '../../../platform/i18n'
import { useToast } from '../../../shared/ui'
import AISettingsPanel from '../components/AISettingsPanel.vue'
import ApplicationSettingsPanel from '../components/ApplicationSettingsPanel.vue'
import BackgroundJobsSettingsPanel from '../components/BackgroundJobsSettingsPanel.vue'
import LanguageSettingsPanel from '../components/LanguageSettingsPanel.vue'
import IntegrationsSettingsPanel from '../components/IntegrationsSettingsPanel.vue'
import CommunicationsSettingsPanel from '../components/CommunicationsSettingsPanel.vue'
import MaintenanceSettingsPanel from '../components/MaintenanceSettingsPanel.vue'
import SignalHubSettingsPanel from '../components/SignalHubSettingsPanel.vue'
import SettingsNavigationTree from '../components/SettingsNavigationTree.vue'
import SettingsOverviewStrip from '../components/SettingsOverviewStrip.vue'
import TraceLogsSettingsPanel from '../components/TraceLogsSettingsPanel.vue'
import { useSettingsPageController } from '../queries/useSettingsPageController'

const { t } = useI18n()
const toast = useToast()
const {
  aiSettings,
  applicationSettings,
  backgroundJobsSettings,
  communicationsSettings,
  integrationsSettings,
  languageSettings,
  maintenanceSettings,
  realtimeStatus,
  settingsOverviewCards,
  settingsTreeGroups,
  signalHubSettings,
  actionMessage,
  errorMessage,
  selectedSection,
  requestRealtimeReconnect,
  selectSection,
  traceLogsSettings,
} = useSettingsPageController()

watch(() => actionMessage, (message) => {
  if (message) toast.success(t('Settings action completed'), message)
})

watch(() => errorMessage, (message) => {
  if (message) toast.error(t('Settings action failed'), message)
})

</script>

<template>
  <section class="settings-page">
    <SettingsOverviewStrip
      :can-reconnect="realtimeStatus.canTriggerReconnect"
      :cards="settingsOverviewCards"
      @reconnect="requestRealtimeReconnect"
    />

    <div class="settings-workbench">
      <SettingsNavigationTree
        :groups="settingsTreeGroups"
        :selected-section="selectedSection"
        @select-section="selectSection"
      />

      <main class="settings-workbench-content">
        <IntegrationsSettingsPanel
          v-if="selectedSection === 'accounts'"
          :surface="integrationsSettings"
        />

        <CommunicationsSettingsPanel
          v-else-if="selectedSection === 'communications'"
          :surface="communicationsSettings"
        />

        <ApplicationSettingsPanel
          v-else-if="selectedSection === 'application'"
          :surface="applicationSettings"
        />

        <BackgroundJobsSettingsPanel
          v-else-if="selectedSection === 'background-jobs'"
          :surface="backgroundJobsSettings"
        />

        <TraceLogsSettingsPanel
          v-else-if="selectedSection === 'logs-traces'"
          :surface="traceLogsSettings"
        />

        <MaintenanceSettingsPanel v-else-if="selectedSection === 'maintenance'" :surface="maintenanceSettings" />

        <LanguageSettingsPanel
          v-else-if="selectedSection === 'language'"
          :surface="languageSettings"
        />

        <SignalHubSettingsPanel
          v-else-if="selectedSection === 'signal-hub'"
          :surface="signalHubSettings"
        />

        <AISettingsPanel
          v-else-if="selectedSection === 'ai'"
          :surface="aiSettings"
        />
      </main>
    </div>

  </section>
</template>
