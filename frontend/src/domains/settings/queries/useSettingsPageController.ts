import type { SettingsSection } from '../stores/settings'
import { useSettingsPageSurface } from './useSettingsPageSurface'

export function useSettingsPageController() {
  const surface = useSettingsPageSurface()
  const {
    aiSettings,
    applicationSettings,
    applicationSettingsCount,
    backgroundJobsSettings,
    communicationsSettings,
    integrationCount,
    integrationsSettings,
    languageSettings,
    maintenanceSettings,
    realtimeStatus,
    selectedTreeItem,
    settingsOverviewCards,
    settingsTreeGroups,
    signalHubSettings,
    store,
    traceLogsSettings,
  } = surface

  function selectSection(section: SettingsSection): void {
    store.selectSection(section)
  }

  function requestRealtimeReconnect(): void {
    realtimeStatus.requestReconnect()
  }

  return {
    aiSettings,
    applicationSettings,
    applicationSettingsCount,
    backgroundJobsSettings,
    communicationsSettings,
    integrationCount,
    integrationsSettings,
    languageSettings,
    maintenanceSettings,
    realtimeStatus,
    settingsOverviewCards,
    settingsTreeGroups,
    signalHubSettings,
    store,
    selectSection,
    requestRealtimeReconnect,
    actionMessage: store.actionMessage,
    errorMessage: store.errorMessage,
    traceLogsSettings,
    selectedSection: store.selectedSection,
    selected: selectedTreeItem,
  }
}
