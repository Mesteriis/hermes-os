import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export type SettingsSection =
  | 'appearance'
  | 'language'
  | 'application'
  | 'sidebar'
  | 'integrations'
  | 'signal-hub'
  | 'ai'

export const useSettingsStore = defineStore('settings-ui', () => {
  // --- UI state ---
  const selectedSection = ref<SettingsSection>('appearance')
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
  }

  function setError(msg: string) {
    errorMessage.value = msg
    actionMessage.value = ''
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
