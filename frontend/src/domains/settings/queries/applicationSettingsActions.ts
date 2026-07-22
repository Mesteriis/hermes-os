import type { ApplicationSettingValue } from '../types/settings'

interface SaveApplicationSettingDependencies {
  save: (variables: { settingKey: string; value: ApplicationSettingValue }) => Promise<unknown>
  clearMessages: () => void
  setActionMessage: (message: string) => void
  setError: (message: string) => void
  setSavingKey: (key: string | null) => void
}

export async function saveApplicationSettingValue(
  settingKey: string,
  label: string,
  value: ApplicationSettingValue,
  dependencies: SaveApplicationSettingDependencies
): Promise<void> {
  dependencies.setSavingKey(settingKey)
  dependencies.clearMessages()
  try {
    await dependencies.save({ settingKey, value })
    dependencies.setActionMessage(`Saved ${label}`)
  } catch (error) {
    dependencies.setError(error instanceof Error ? error.message : 'Failed to save setting')
  } finally {
    dependencies.setSavingKey(null)
  }
}
