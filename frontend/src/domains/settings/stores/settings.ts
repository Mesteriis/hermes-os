import { defineStore, getActivePinia } from 'pinia'
import { ref, computed } from 'vue'
import { useNotificationsStore, type NotificationItem } from '../../../shared/stores/notifications'

export type SettingsSection =
  | 'accounts'
  | 'language'
  | 'application'
  | 'signal-hub'
  | 'ai'

let settingsNotificationCounter = 0

export const useSettingsStore = defineStore('settings-ui', () => {
  // --- UI state ---
  const selectedSection = ref<SettingsSection>('accounts')
  const actionMessage = ref('')
  const errorMessage = ref('')

  // --- Sidebar editing state ---
  const isSidebarSettingsSaving = ref(false)
  const sidebarError = ref('')
  const newSidebarGroupLabel = ref('')

  // --- Selected integration ---
  const selectedIntegrationId = ref<string | null>(null)

  // --- Computed ---
  const hasSidebarChanges = computed(() => false) // Placeholder — will connect to sidebar store later

  // --- Actions ---
  function selectSection(section: SettingsSection) {
    selectedSection.value = section
    actionMessage.value = ''
    errorMessage.value = ''
  }

  function setActionMessage(msg: string) {
    actionMessage.value = msg
    errorMessage.value = ''
    publishSettingsNotification('success', 'Settings action completed', msg)
  }

  function setError(msg: string) {
    errorMessage.value = msg
    actionMessage.value = ''
    publishSettingsNotification('danger', 'Settings action failed', msg)
  }

  function clearMessages() {
    actionMessage.value = ''
    errorMessage.value = ''
  }

  function selectIntegration(id: string | null) {
    selectedIntegrationId.value = id
  }

  function updateNewSidebarGroupLabel(label: string) {
    newSidebarGroupLabel.value = label
  }

  function publishSettingsNotification(
    tone: NonNullable<NotificationItem['tone']>,
    title: string,
    body: string
  ): void {
    if (!body.trim()) return
    if (!getActivePinia()) return

    useNotificationsStore().addNotification({
      id: `settings-${Date.now()}-${++settingsNotificationCounter}`,
      title,
      body,
      icon: tone === 'success' ? 'tabler:check' : 'tabler:alert-circle',
      tone,
      sourceLabel: 'Settings',
      time: new Date(),
      targetView: 'settings',
      targetId: selectedSection.value,
      dedupeKey: `settings:${tone}:${selectedSection.value}:${body}`,
    })
  }

  return {
    selectedSection,
    actionMessage,
    errorMessage,
    isSidebarSettingsSaving,
    sidebarError,
    newSidebarGroupLabel,
    selectedIntegrationId,
    hasSidebarChanges,
    selectSection,
    setActionMessage,
    setError,
    clearMessages,
    selectIntegration,
    updateNewSidebarGroupLabel
  }
})
