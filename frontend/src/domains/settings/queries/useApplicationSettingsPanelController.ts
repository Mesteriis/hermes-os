import type { useApplicationSettingsSurface } from './useApplicationSettingsSurface'

type ApplicationSettingsSurface = ReturnType<typeof useApplicationSettingsSurface>

export function useApplicationSettingsPanelController(options: {
  surface: ApplicationSettingsSurface
}) {
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

  function handleSettingInput(setting: Parameters<ApplicationSettingsSurface['handleInput']>[0], event: Event): void {
    options.surface.handleInput(setting, eventValue(event))
  }

  function handleSettingBooleanInput(setting: Parameters<ApplicationSettingsSurface['handleInput']>[0], event: Event): void {
    options.surface.handleInput(setting, eventChecked(event) ? 'true' : 'false')
  }

  function handleSaveSetting(setting: Parameters<ApplicationSettingsSurface['handleSave']>[0]): void {
    void options.surface.handleSave(setting)
  }

  return {
    categoryLabel: options.surface.categoryLabel,
    isLoading: options.surface.isLoading,
    settingAllowedValues: options.surface.settingAllowedValues,
    settingsByCategory: options.surface.settingsByCategory,
    settingControlType: options.surface.settingControlType,
    settingDraftValue: options.surface.settingDraftValue,
    settingHasChanged: options.surface.settingHasChanged,
    savingSettingKey: options.surface.savingSettingKey,
    handleSettingBooleanInput,
    handleSettingInput,
    handleSaveSetting,
  }
}
