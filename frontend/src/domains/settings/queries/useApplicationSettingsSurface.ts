import { computed, ref } from 'vue'
import { useApplicationSettingsQuery, groupSettingsByCategory, useSaveApplicationSettingMutation } from './useSettingsQuery'
import { useSettingsStore } from '../stores/settings'
import type { ApplicationSetting } from '../types/settings'
import { isPublicApplicationSetting, settingHasChanged } from './applicationSettingsPredicates'
import {
  categoryLabel,
  settingAllowedValues,
  settingControlType,
  settingDraftValue,
  settingMetadataFlag,
  settingMetadataText,
} from './applicationSettingsPresentation'
import { coerceApplicationSettingValue } from './applicationSettingsValue'
import { saveApplicationSettingValue } from './applicationSettingsActions'

export function useApplicationSettingsSurface() {
  const store = useSettingsStore()
  const saveSetting = useSaveApplicationSettingMutation()
  const { data: appSettingsData, isLoading } = useApplicationSettingsQuery()

  const settingDrafts = ref<Record<string, string>>({})
  const savingSettingKey = ref<string | null>(null)

  const applicationSettings = computed(() => {
    return (appSettingsData.value?.items ?? []).filter(isPublicApplicationSetting)
  })
  const allApplicationSettings = computed(() => appSettingsData.value?.items ?? [])
  const settingsByCategory = computed(() => groupSettingsByCategory(applicationSettings.value))

  const settingDraftValueForSurface = (setting: ApplicationSetting): string =>
    settingDraftValue(setting, settingDrafts.value)
  const settingHasChangedForSurface = (setting: ApplicationSetting): boolean =>
    settingHasChanged(setting, settingDrafts.value)

  async function handleSave(setting: ApplicationSetting) {
    const draftValue = settingDrafts.value[setting.setting_key]
    const valueToSave = draftValue !== undefined
      ? coerceApplicationSettingValue(draftValue, setting.value_kind)
      : setting.value
    await saveApplicationSettingValue(setting.setting_key, setting.label, valueToSave, {
      save: (variables) => saveSetting.mutateAsync(variables),
      clearMessages: () => store.clearMessages(),
      setActionMessage: (message) => store.setActionMessage(message),
      setError: (message) => store.setError(message),
      setSavingKey: (key) => { savingSettingKey.value = key }
    })
  }

  function handleInput(setting: ApplicationSetting, value: string) {
    settingDrafts.value[setting.setting_key] = value
  }

  return {
    applicationSettings,
    allApplicationSettings,
    isLoading,
    savingSettingKey,
    settingDrafts,
    settingsByCategory,
    categoryLabel,
    handleInput,
    handleSave,
    settingAllowedValues,
    settingControlType,
    settingDraftValue: settingDraftValueForSurface,
    settingHasChanged: settingHasChangedForSurface,
    settingMetadataFlag,
    settingMetadataText
  }
}
