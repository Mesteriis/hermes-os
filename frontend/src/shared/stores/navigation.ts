import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'

export type PrimaryNavId =
  | 'home'
  | 'communications'
  | 'timeline'
  | 'persons'
  | 'projects'
  | 'tasks'
  | 'calendar'
  | 'documents'
  | 'notes'
  | 'knowledge'
  | 'review'
  | 'agents'

export type CommunicationSectionId =
  | 'unified'
  | 'inbox'
  | 'waiting'
  | 'needs_reply'
  | 'mentions'
  | 'mail'
  | 'telegram'
  | 'whatsapp'
  | 'calls'
  | 'meetings'

export type SidebarViewId = PrimaryNavId | 'telegram' | 'whatsapp' | 'settings' | 'organizations'
export type AppViewId = PrimaryNavId | 'settings' | 'organizations'
type RouteViewId = AppViewId
type RouteSectionQuery = string | null | Array<string | null> | undefined

const communicationSectionIds: CommunicationSectionId[] = [
  'unified',
  'inbox',
  'waiting',
  'needs_reply',
  'mentions',
  'mail',
  'telegram',
  'whatsapp',
  'calls',
  'meetings'
]

function communicationSectionViewId(sectionId: CommunicationSectionId): SidebarViewId {
  if (sectionId === 'telegram' || sectionId === 'whatsapp') {
    return sectionId
  }
  return 'communications'
}

function communicationSectionFromQuery(section: RouteSectionQuery): CommunicationSectionId | null {
  const value = Array.isArray(section) ? section[0] : section
  if (!value) return null
  return communicationSectionIds.includes(value as CommunicationSectionId)
    ? value as CommunicationSectionId
    : null
}

type ViewCopy = {
  title: string
  subtitle: string
  search?: string
  icon: string
}

const viewCopy: Record<string, ViewCopy> = {
  home: { title: 'Home', subtitle: 'Dashboard', search: 'Search home…', icon: 'tabler:home' },
  communications: { title: 'Communications', subtitle: 'Unified inbox', search: 'Search communications…', icon: 'tabler:messages' },
  timeline: { title: 'Timeline', subtitle: 'Activity stream', search: 'Search timeline…', icon: 'tabler:timeline-event' },
  persons: { title: 'Persons', subtitle: 'Persona intelligence', search: 'Search persons…', icon: 'tabler:user' },
  projects: { title: 'Projects', subtitle: 'Projects and initiatives', search: 'Search projects…', icon: 'tabler:briefcase' },
  tasks: { title: 'Tasks', subtitle: 'Task management', search: 'Search tasks…', icon: 'tabler:checkbox' },
  calendar: { title: 'Calendar', subtitle: 'Schedule and events', search: 'Search calendar…', icon: 'tabler:calendar' },
  documents: { title: 'Documents', subtitle: 'Document management', search: 'Search documents…', icon: 'tabler:file-text' },
  notes: { title: 'Notes', subtitle: 'Notes and memos', search: 'Search notes…', icon: 'tabler:notes' },
  knowledge: { title: 'Knowledge Graph', subtitle: 'Graph exploration', search: 'Search knowledge graph…', icon: 'tabler:share' },
  review: { title: 'Review', subtitle: 'Review queue', search: 'Search review…', icon: 'tabler:clipboard-check' },
  settings: { title: 'Settings', subtitle: 'Application settings', search: 'Search settings…', icon: 'tabler:settings' },
  agents: { title: 'AI Agents', subtitle: 'AI control center', search: '', icon: 'tabler:sparkles' },
  organizations: { title: 'Organizations', subtitle: 'Organizations workspace', search: 'Search organizations…', icon: 'tabler:building-community' },
  telegram: { title: 'Telegram', subtitle: 'Telegram messages', search: 'Search Telegram…', icon: 'tabler:brand-telegram' },
  whatsapp: { title: 'WhatsApp', subtitle: 'WhatsApp messages', search: 'Search WhatsApp…', icon: 'tabler:brand-whatsapp' }
}

export const useNavigationStore = defineStore('navigation', () => {
  const router = useRouter()

  const currentView = ref<AppViewId>('home')
  const activeCommunicationSection = ref<CommunicationSectionId>('unified')
  const isSidebarRail = ref(false)
  const isUserMenuOpen = ref(false)
  const expandedSidebarGroupIds = ref<string[]>(['communications'])
  const activeSidebarRailGroupId = ref<string | null>(null)

  const activeWorkspaceView = computed<SidebarViewId>(() => {
    if (currentView.value === 'communications') {
      return communicationSectionViewId(activeCommunicationSection.value)
    }
    return currentView.value as SidebarViewId
  })

  const activeView = computed<ViewCopy | null>(() => {
    return viewCopy[currentView.value] ?? null
  })

  const shellViewClass = computed<string>(() => {
    return `view-${currentView.value}`
  })

  function navigateTo(viewId: AppViewId): void {
    currentView.value = viewId
    activeSidebarRailGroupId.value = null
    router.push(`/${viewId}`)
  }

  function navigateToCommunicationSection(sectionId: CommunicationSectionId): void {
    currentView.value = 'communications'
    activeCommunicationSection.value = sectionId
    router.push({ name: 'communications', query: { section: sectionId } })
  }

  function syncFromRoute(viewId: RouteViewId, sectionQuery?: RouteSectionQuery): void {
    currentView.value = viewId
    if (viewId !== 'communications') {
      activeSidebarRailGroupId.value = null
      return
    }

    activeCommunicationSection.value = communicationSectionFromQuery(sectionQuery) ?? 'unified'
  }

  function toggleUserMenu(): void {
    isUserMenuOpen.value = !isUserMenuOpen.value
  }

  function closeUserMenu(): void {
    isUserMenuOpen.value = false
  }

  function toggleSidebarRail(): void {
    isSidebarRail.value = !isSidebarRail.value
  }

  function toggleSidebarGroup(groupId: string): void {
    const index = expandedSidebarGroupIds.value.indexOf(groupId)
    if (index >= 0) {
      expandedSidebarGroupIds.value.splice(index, 1)
    } else {
      expandedSidebarGroupIds.value.push(groupId)
    }
  }

  function setActiveSidebarRailGroup(groupId: string | null): void {
    activeSidebarRailGroupId.value = groupId
  }

  return {
    currentView,
    activeCommunicationSection,
    isSidebarRail,
    isUserMenuOpen,
    expandedSidebarGroupIds,
    activeSidebarRailGroupId,
    activeWorkspaceView,
    activeView,
    shellViewClass,
    navigateTo,
    navigateToCommunicationSection,
    syncFromRoute,
    toggleUserMenu,
    closeUserMenu,
    toggleSidebarRail,
    toggleSidebarGroup,
    setActiveSidebarRailGroup
  }
})
