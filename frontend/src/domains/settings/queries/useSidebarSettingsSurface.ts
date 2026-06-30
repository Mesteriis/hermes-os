import { computed } from 'vue'
import { useI18n } from '../../../platform/i18n'
import { useSidebarStore } from '../../../shared/stores/sidebar'
import { useSaveFrontendSidebarMutation } from './useSettingsQuery'
import { useSettingsStore } from '../stores/settings'

export function useSidebarSettingsSurface() {
  const { t } = useI18n()
  const store = useSettingsStore()
  const sidebar = useSidebarStore()
  const saveSidebarSettings = useSaveFrontendSidebarMutation()

  const sidebarRuleSummaries = computed(() => [
    { text: t('Default keeps the current sidebar order'), badge: t('Preset') },
    { text: t('Communications sources stay nested'), badge: t('Context') },
    { text: t('Hidden domains stay recoverable here'), badge: t('Safe') },
    { text: t('Settings store no message content'), badge: t('Privacy') }
  ])

  function handleAddSidebarGroup() {
    sidebar.addSidebarGroup(store.newSidebarGroupLabel)
    store.updateNewSidebarGroupLabel('')
  }

  async function handleSaveSidebar() {
    store.isSidebarSettingsSaving = true
    store.sidebarError = ''
    try {
      await saveSidebarSettings.mutateAsync(sidebar.sidebarSettingValue)
      sidebar.setSidebarSettings(sidebar.effectiveSidebarSettings)
      store.setActionMessage(t('Sidebar saved'))
    } catch (error) {
      store.sidebarError = error instanceof Error ? error.message : t('Failed to save sidebar')
    } finally {
      store.isSidebarSettingsSaving = false
    }
  }

  return {
    sidebar,
    store,
    sidebarRuleSummaries,
    handleAddSidebarGroup,
    handleSaveSidebar
  }
}
