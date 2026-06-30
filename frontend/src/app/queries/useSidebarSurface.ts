import { useNavigationStore } from '../../shared/stores/navigation'
import { useSidebarStore } from '../../shared/stores/sidebar'

export function useSidebarSurface() {
  const nav = useNavigationStore()
  const sidebar = useSidebarStore()

  function handleSelectItem(viewId: string): void {
    nav.navigateTo(viewId as Parameters<typeof nav.navigateTo>[0])
  }

  function handleToggleGroup(groupId: string): void {
    nav.toggleSidebarGroup(groupId)
  }

  function handleToggleRail(): void {
    nav.toggleSidebarRail()
  }

  function handleSettings(): void {
    nav.navigateTo('settings')
  }

  function isItemActive(itemViewId: string): boolean {
    if (nav.currentView === 'communications') {
      return itemViewId === 'communications'
    }
    return nav.currentView === itemViewId
  }

  function isCommunicationItemActive(sectionId?: string): boolean {
    if (!sectionId || nav.currentView !== 'communications') return false
    return nav.activeCommunicationSection === sectionId
  }

  return {
    nav,
    sidebar,
    handleSelectItem,
    handleToggleGroup,
    handleToggleRail,
    handleSettings,
    isItemActive,
    isCommunicationItemActive
  }
}
