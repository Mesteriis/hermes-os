import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { ApplicationSetting } from '../types/settings'
import { saveApplicationSetting } from '../api/settings'

export type SettingsSection =
  | 'appearance'
  | 'language'
  | 'application'
  | 'sidebar'
  | 'integrations'
  | 'ai'

export const useSettingsStore = defineStore('settings-ui', () => {
  // --- UI state ---
  const selectedSection = ref<SettingsSection>('appearance')
  const actionMessage = ref('')
  const errorMessage = ref('')
  const savingSettingKey = ref<string | null>(null)

  // --- Drafts for application settings ---
  const settingDrafts = ref<Record<string, string>>({})

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

  function updateSettingDraft(key: string, value: string) {
    settingDrafts.value[key] = value
  }

  async function saveSetting(setting: ApplicationSetting) {
    savingSettingKey.value = setting.setting_key
    clearMessages()
    try {
      const draftValue = settingDrafts.value[setting.setting_key]
      const valueToSave = draftValue !== undefined ? coerceValue(draftValue, setting.value_kind) : setting.value
      await saveApplicationSetting(setting.setting_key, valueToSave)
      setActionMessage(`Saved ${setting.label}`)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to save setting')
    } finally {
      savingSettingKey.value = null
    }
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
    savingSettingKey,
    settingDrafts,
    isSidebarSettingsSaving,
    sidebarError,
    newSidebarGroupLabel,
    selectedIntegrationId,
    hasSidebarChanges,
    selectSection,
    setActionMessage,
    setError,
    clearMessages,
    updateSettingDraft,
    saveSetting,
    selectIntegration,
    updateNewSidebarGroupLabel
  }
})

/** Coerce a draft string value to the correct type for saving. */
function coerceValue(
  draft: string,
  kind: string
): ApplicationSetting['value'] {
  switch (kind) {
    case 'boolean':
      return draft === 'true'
    case 'integer':
      return parseInt(draft, 10) || 0
    case 'json':
      try { return JSON.parse(draft) } catch { return draft }
    default:
      return draft
  }
}
