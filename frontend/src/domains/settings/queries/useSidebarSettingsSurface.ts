import { computed } from 'vue'
import { useI18n } from '../../../platform/i18n'
import { useSidebarStore } from '../../../shared/stores/sidebar'
import { useSaveFrontendSidebarMutation } from './useSettingsQuery'
import { useSettingsStore } from '../stores/settings'
import { saveSidebarSettings } from './sidebarSettingsActions'
import { buildSidebarRuleSummaries } from './sidebarSettingsPresentation'

export function useSidebarSettingsSurface() {
  const { t } = useI18n()
  const store = useSettingsStore()
  const sidebar = useSidebarStore()
  const saveSidebarSettingsMutation = useSaveFrontendSidebarMutation()

  const sidebarRuleSummaries = computed(() => buildSidebarRuleSummaries(t))

  function handleAddSidebarGroup() {
    sidebar.addSidebarGroup(store.newSidebarGroupLabel)
    store.updateNewSidebarGroupLabel('')
  }

  async function handleSaveSidebar() {
    await saveSidebarSettings(sidebar.sidebarSettingValue, {
      saveSidebarSettings: (settings) => saveSidebarSettingsMutation.mutateAsync(settings),
      applySidebarSettings: () => sidebar.setSidebarSettings(sidebar.effectiveSidebarSettings),
      setSaving: (value) => { store.isSidebarSettingsSaving = value },
      clearError: () => { store.sidebarError = '' },
      setError: (message) => { store.sidebarError = message },
      setActionMessage: (message) => store.setActionMessage(message),
      savingMessage: t('Sidebar saved'),
      errorMessage: (error) => error instanceof Error ? error.message : t('Failed to save sidebar')
    })
  }

  return {
    sidebar,
    store,
    sidebarRuleSummaries,
    handleAddSidebarGroup,
    handleSaveSidebar
  }
}
