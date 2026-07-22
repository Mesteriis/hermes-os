interface SaveSidebarDependencies<TSettings> {
  saveSidebarSettings: (settings: TSettings) => Promise<unknown>
  applySidebarSettings: () => void
  setSaving: (value: boolean) => void
  clearError: () => void
  setError: (message: string) => void
  setActionMessage: (message: string) => void
  savingMessage: string
  errorMessage: (error: unknown) => string
}

export async function saveSidebarSettings<TSettings>(
  settings: TSettings,
  dependencies: SaveSidebarDependencies<TSettings>
): Promise<void> {
  dependencies.setSaving(true)
  dependencies.clearError()
  try {
    await dependencies.saveSidebarSettings(settings)
    dependencies.applySidebarSettings()
    dependencies.setActionMessage(dependencies.savingMessage)
  } catch (error) {
    dependencies.setError(dependencies.errorMessage(error))
  } finally {
    dependencies.setSaving(false)
  }
}
