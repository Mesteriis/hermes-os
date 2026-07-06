import { computed, ref } from 'vue'
import { useApplicationSettingsQuery, groupSettingsByCategory, useSaveApplicationSettingMutation } from './useSettingsQuery'
import { useSettingsStore } from '../stores/settings'
import type { ApplicationSetting } from '../types/settings'

export function useApplicationSettingsSurface() {
  const store = useSettingsStore()
  const saveSetting = useSaveApplicationSettingMutation()
  const { data: appSettingsData, isLoading } = useApplicationSettingsQuery()

  const settingDrafts = ref<Record<string, string>>({})
  const savingSettingKey = ref<string | null>(null)

  const applicationSettings = computed(() => {
    return (appSettingsData.value?.items ?? []).filter(isPublicApplicationSetting)
  })
  const settingsByCategory = computed(() => groupSettingsByCategory(applicationSettings.value))

  function settingDraftValue(setting: ApplicationSetting): string {
    const draft = settingDrafts.value[setting.setting_key]
    if (draft !== undefined) return draft
    return String(setting.value ?? '')
  }

  function settingHasChanged(setting: ApplicationSetting): boolean {
    const draft = settingDrafts.value[setting.setting_key]
    if (draft === undefined) return false
    return draft !== String(setting.value)
  }

  function settingControlType(setting: ApplicationSetting): string {
    const allowedValues = setting.metadata?.allowed_values
    if (Array.isArray(allowedValues) && allowedValues.length > 0) return 'select'
    if (setting.value_kind === 'boolean') return 'checkbox'
    if (setting.value_kind === 'integer') return 'number'
    return 'text'
  }

  function settingAllowedValues(setting: ApplicationSetting): string[] {
    const values = setting.metadata?.allowed_values
    return Array.isArray(values) ? values.map(String) : []
  }

  function settingMetadataFlag(setting: ApplicationSetting, key: string): boolean {
    return Boolean(setting.metadata?.[key])
  }

  function settingMetadataText(setting: ApplicationSetting, key: string): string {
    const value = setting.metadata?.[key]
    return typeof value === 'string' ? value : ''
  }

  function categoryLabel(category: string): string {
    const labels: Record<string, string> = {
      general: 'General',
      frontend: 'Interface',
      ai: 'AI',
      privacy: 'Privacy',
      notifications: 'Notifications'
    }
    return labels[category] ?? category
  }

  async function handleSave(setting: ApplicationSetting) {
    savingSettingKey.value = setting.setting_key
    store.clearMessages()
    try {
      const draftValue = settingDrafts.value[setting.setting_key]
      const valueToSave = draftValue !== undefined ? coerceValue(draftValue, setting.value_kind) : setting.value
      await saveSetting.mutateAsync({ settingKey: setting.setting_key, value: valueToSave })
      store.setActionMessage(`Saved ${setting.label}`)
    } catch (error) {
      store.setError(error instanceof Error ? error.message : 'Failed to save setting')
    } finally {
      savingSettingKey.value = null
    }
  }

  function handleInput(setting: ApplicationSetting, value: string) {
    settingDrafts.value[setting.setting_key] = value
  }

  return {
    applicationSettings,
    isLoading,
    savingSettingKey,
    settingDrafts,
    settingsByCategory,
    categoryLabel,
    handleInput,
    handleSave,
    settingAllowedValues,
    settingControlType,
    settingDraftValue,
    settingHasChanged,
    settingMetadataFlag,
    settingMetadataText
  }
}

function isPublicApplicationSetting(setting: ApplicationSetting): boolean {
  return setting.category !== 'ai' && !setting.setting_key.startsWith('ai.')
}

function coerceValue(draft: string, kind: string): ApplicationSetting['value'] {
  switch (kind) {
    case 'boolean':
      return draft === 'true'
    case 'integer':
      return parseInt(draft, 10) || 0
    case 'json':
      try {
        return JSON.parse(draft)
      } catch {
        return draft
      }
    default:
      return draft
  }
}
