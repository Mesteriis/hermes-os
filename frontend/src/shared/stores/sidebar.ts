import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

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
  | 'event-tracing'
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

export type CommunicationSidebarSectionId = Extract<
  CommunicationSectionId,
  'mail' | 'telegram' | 'whatsapp' | 'calls' | 'meetings'
>
export type CommunicationSidebarItemId = `communications.${CommunicationSidebarSectionId}`
export type SidebarPrimaryItemId = Exclude<PrimaryNavId, 'communications'>
export type SidebarItemId = SidebarPrimaryItemId | CommunicationSidebarItemId
export type SidebarRootItemId = SidebarPrimaryItemId | `group:${string}`

export type PrimaryWorkspaceNavItem = {
  id: PrimaryNavId
  label: string
  icon: string
}

export type CommunicationSection = {
  id: CommunicationSectionId
  label: string
  icon: string
  group: 'overview' | 'workflow' | 'sources'
}

export type SidebarNavGroup = {
  id: string
  label: string
  icon: string
  itemIds: SidebarItemId[]
  separatorBeforeItemIds: SidebarItemId[]
}

export type SidebarSettings = {
  schemaVersion: 3
  rootItemIds: SidebarRootItemId[]
  groups: SidebarNavGroup[]
  hiddenItemIds: SidebarItemId[]
}

export type ResolvedSidebarItem = {
  itemId: SidebarItemId
  label: string
  icon: string
  isCommunication: boolean
  sectionId?: CommunicationSidebarSectionId
}

export type ResolvedSidebarRootEntry =
  | {
      kind: 'item'
      rootId: SidebarPrimaryItemId
      item: ResolvedSidebarItem
    }
  | {
      kind: 'group'
      rootId: `group:${string}`
      group: SidebarNavGroup & { items: ResolvedSidebarItem[] }
    }

const SIDEBAR_SETTINGS_SCHEMA_VERSION = 3 as const

export const primaryWorkspaceNav: PrimaryWorkspaceNavItem[] = [
  { id: 'home', label: 'Home', icon: 'tabler:home' },
  { id: 'communications', label: 'Communications', icon: 'tabler:messages' },
  { id: 'timeline', label: 'Timeline', icon: 'tabler:timeline-event' },
  { id: 'persons', label: 'Persons', icon: 'tabler:user' },
  { id: 'projects', label: 'Projects', icon: 'tabler:briefcase' },
  { id: 'tasks', label: 'Tasks', icon: 'tabler:checkbox' },
  { id: 'calendar', label: 'Calendar', icon: 'tabler:calendar' },
  { id: 'documents', label: 'Documents', icon: 'tabler:file-text' },
  { id: 'notes', label: 'Notes', icon: 'tabler:notes' },
  { id: 'knowledge', label: 'Knowledge Graph', icon: 'tabler:share' },
  { id: 'review', label: 'Review', icon: 'tabler:clipboard-check' },
  { id: 'event-tracing', label: 'Event Traces', icon: 'tabler:route' },
  { id: 'agents', label: 'AI Agents', icon: 'tabler:sparkles' }
]

export const communicationSections: CommunicationSection[] = [
  { id: 'unified', label: 'Unified', icon: 'tabler:sparkles', group: 'overview' },
  { id: 'inbox', label: 'Inbox', icon: 'tabler:mail', group: 'workflow' },
  { id: 'waiting', label: 'Waiting', icon: 'tabler:clock-hour-4', group: 'workflow' },
  { id: 'needs_reply', label: 'Needs Reply', icon: 'tabler:message-reply', group: 'workflow' },
  { id: 'mentions', label: 'Mentions', icon: 'tabler:at', group: 'workflow' },
  { id: 'mail', label: 'Mail', icon: 'tabler:mail', group: 'sources' },
  { id: 'telegram', label: 'Telegram', icon: 'tabler:brand-telegram', group: 'sources' },
  { id: 'whatsapp', label: 'WhatsApp', icon: 'tabler:brand-whatsapp', group: 'sources' },
  { id: 'calls', label: 'Calls', icon: 'tabler:phone', group: 'sources' },
  { id: 'meetings', label: 'Meetings', icon: 'tabler:calendar-event', group: 'sources' }
]

const communicationSidebarSectionIds: CommunicationSidebarSectionId[] = [
  'mail', 'telegram', 'whatsapp', 'calls', 'meetings'
]

function communicationSidebarItemId(sectionId: CommunicationSidebarSectionId): CommunicationSidebarItemId {
  return `communications.${sectionId}`
}

function normalizeGroupId(value: string): string {
  return value.toLowerCase().replace(/[^a-z0-9-]+/g, '-').replace(/^-+|-+$/g, '')
}

function sidebarGroupRootId(groupId: string): `group:${string}` {
  return `group:${normalizeGroupId(groupId)}`
}

function defaultSidebarSettings(): SidebarSettings {
  const communicationSectionsInSidebar = communicationSections.filter((s) =>
    communicationSidebarSectionIds.includes(s.id as CommunicationSidebarSectionId)
  ) as Array<CommunicationSection & { id: CommunicationSidebarSectionId }>

  const defaultCommunicationsGroup: SidebarNavGroup = {
    id: 'communications',
    label: 'Communications',
    icon: 'tabler:messages',
    itemIds: [
      ...communicationSectionsInSidebar.map((s) => communicationSidebarItemId(s.id)),
      'timeline' as SidebarPrimaryItemId
    ],
    separatorBeforeItemIds: []
  }

  const communicationGroupPrimaryItemIds: SidebarPrimaryItemId[] = ['timeline']

  const defaultRootItemIds: SidebarRootItemId[] = primaryWorkspaceNav.flatMap((item) =>
    item.id === 'communications'
      ? [sidebarGroupRootId(defaultCommunicationsGroup.id)]
      : !communicationGroupPrimaryItemIds.includes(item.id as SidebarPrimaryItemId)
        ? [item.id as SidebarPrimaryItemId]
        : []
  )

  return {
    schemaVersion: SIDEBAR_SETTINGS_SCHEMA_VERSION,
    rootItemIds: defaultRootItemIds,
    groups: [defaultCommunicationsGroup],
    hiddenItemIds: []
  }
}

function resolveSidebarItem(itemId: SidebarItemId): ResolvedSidebarItem | null {
  // Check if it's a primary nav item
  const primaryItem = primaryWorkspaceNav.find((p) => p.id === itemId)
  if (primaryItem) {
    return {
      itemId: itemId as SidebarPrimaryItemId,
      label: primaryItem.label,
      icon: primaryItem.icon,
      isCommunication: false
    }
  }

  // Check if it's a communication sidebar item
  if (itemId.startsWith('communications.')) {
    const sectionId = itemId.slice('communications.'.length) as CommunicationSidebarSectionId
    const section = communicationSections.find((s) => s.id === sectionId)
    if (section && communicationSidebarSectionIds.includes(sectionId)) {
      return {
        itemId,
        label: section.label,
        icon: section.icon,
        isCommunication: true,
        sectionId
      }
    }
  }

  return null
}

export const useSidebarStore = defineStore('sidebar', () => {
  const sidebarSettings = ref<SidebarSettings>(defaultSidebarSettings())
  const sidebarDraft = ref<SidebarSettings | null>(null)

  const effectiveSidebarSettings = computed<SidebarSettings>(() => {
    return sidebarDraft.value ?? sidebarSettings.value
  })

  const sidebarRootEntries = computed<ResolvedSidebarRootEntry[]>(() => {
    const entries: ResolvedSidebarRootEntry[] = []
    const settings = effectiveSidebarSettings.value

    for (const rootId of settings.rootItemIds) {
      // Check if it's a primary item
      if (!rootId.startsWith('group:')) {
        const primaryId = rootId as SidebarPrimaryItemId
        const resolved = resolveSidebarItem(primaryId)
        if (resolved) {
          entries.push({ kind: 'item', rootId: primaryId, item: resolved })
        }
        continue
      }

      // It's a group
      const groupId = rootId.slice('group:'.length)
      const group = settings.groups.find((g) => normalizeGroupId(g.id) === groupId)
      if (!group) continue

      const items: ResolvedSidebarItem[] = []
      for (const itemId of group.itemIds) {
        if (!settings.hiddenItemIds.includes(itemId)) {
          const resolved = resolveSidebarItem(itemId)
          if (resolved) items.push(resolved)
        }
      }

      entries.push({
        kind: 'group',
        rootId: rootId as `group:${string}`,
        group: { ...group, items }
      })
    }

    return entries
  })

  const sidebarHiddenNavItems = computed<SidebarItemId[]>(() => {
    return effectiveSidebarSettings.value.hiddenItemIds
  })

  function setSidebarSettings(settings: SidebarSettings): void {
    sidebarSettings.value = settings
  }

  function updateSidebarDraft(update: (draft: SidebarSettings) => SidebarSettings): void {
    if (!sidebarDraft.value) {
      sidebarDraft.value = JSON.parse(JSON.stringify(sidebarSettings.value))
    }
    const draft = sidebarDraft.value as SidebarSettings
    sidebarDraft.value = update(draft)
  }

  function resetSidebarSettingsToDefault(): void {
    sidebarDraft.value = defaultSidebarSettings()
  }

  function cancelSidebarSettingsEditing(): void {
    sidebarDraft.value = null
  }

  function sidebarConfigItem(itemId: SidebarItemId): { id: SidebarItemId; label: string; icon: string } | null {
    const resolved = resolveSidebarItem(itemId)
    if (resolved) {
      return { id: resolved.itemId, label: resolved.label, icon: resolved.icon }
    }
    return null
  }

  function sidebarGroupIdFromLabel(label: string): string {
    return label.toLowerCase().replace(/[^a-z0-9-]+/g, '-').replace(/^-+|-+$/g, '')
  }

  function addSidebarGroup(): void {
    updateSidebarDraft((draft) => {
      const groupCount = draft.groups.filter((g) => g.id.startsWith('group-')).length + 1
      const label = `Group ${groupCount}`
      const id = `group-${groupCount}`
      return {
        ...draft,
        groups: [...draft.groups, { id, label, icon: 'tabler:folder', itemIds: [], separatorBeforeItemIds: [] }],
        rootItemIds: [...draft.rootItemIds, `group:${id}`]
      }
    })
  }

  function removeSidebarGroup(groupId: string): void {
    updateSidebarDraft((draft) => {
      const normalized = normalizeGroupId(groupId)
      return {
        ...draft,
        groups: draft.groups.filter((g) => normalizeGroupId(g.id) !== normalized),
        rootItemIds: draft.rootItemIds.filter((id) => {
          if (id.startsWith('group:')) {
            return normalizeGroupId(id.slice('group:'.length)) !== normalized
          }
          return true
        })
      }
    })
  }

  function moveSidebarGroup(groupId: string, direction: -1 | 1): void {
    updateSidebarDraft((draft) => {
      const normalized = normalizeGroupId(groupId)
      const idx = draft.groups.findIndex((g) => normalizeGroupId(g.id) === normalized)
      if (idx < 0) return draft
      const newIdx = idx + direction
      if (newIdx < 0 || newIdx >= draft.groups.length) return draft
      const groups = [...draft.groups]
      const [moved] = groups.splice(idx, 1)
      groups.splice(newIdx, 0, moved)
      return { ...draft, groups }
    })
  }

  function moveSidebarRootItem(rootId: string, direction: -1 | 1): void {
    updateSidebarDraft((draft) => {
      const idx = draft.rootItemIds.indexOf(rootId as SidebarRootItemId)
      if (idx < 0) return draft
      const newIdx = idx + direction
      if (newIdx < 0 || newIdx >= draft.rootItemIds.length) return draft
      const items = [...draft.rootItemIds]
      const [moved] = items.splice(idx, 1)
      items.splice(newIdx, 0, moved)
      return { ...draft, rootItemIds: items }
    })
  }

  function moveSidebarItem(itemId: SidebarItemId, direction: -1 | 1): void {
    updateSidebarDraft((draft) => {
      const groups = draft.groups.map((group) => {
        const idx = group.itemIds.indexOf(itemId)
        if (idx < 0) return group
        const newIdx = idx + direction
        if (newIdx < 0 || newIdx >= group.itemIds.length) return group
        const items = [...group.itemIds]
        const [moved] = items.splice(idx, 1)
        items.splice(newIdx, 0, moved)
        return { ...group, itemIds: items }
      })
      return { ...draft, groups }
    })
  }

  function moveSidebarItemToGroup(itemId: SidebarItemId, targetGroupId: string): void {
    updateSidebarDraft((draft) => {
      const normalizedTarget = normalizeGroupId(targetGroupId)
      let sourceItemRemoved = false

      const groups = draft.groups.map((group) => {
        const normalized = normalizeGroupId(group.id)
        if (normalized === normalizedTarget && sourceItemRemoved) {
          return { ...group, itemIds: [...group.itemIds, itemId] }
        }
        if (group.itemIds.includes(itemId) && !sourceItemRemoved) {
          sourceItemRemoved = true
          return { ...group, itemIds: group.itemIds.filter((id) => id !== itemId) }
        }
        return group
      })

      if (!sourceItemRemoved) {
        // Item was a root item, move to group
        return {
          ...draft,
          rootItemIds: draft.rootItemIds.filter((id) => id !== itemId),
          groups
        }
      }

      // Find target and add item
      const finalGroups = groups.map((group) => {
        if (normalizeGroupId(group.id) === normalizedTarget) {
          return { ...group, itemIds: [...group.itemIds, itemId] }
        }
        return group
      })

      return { ...draft, groups: finalGroups }
    })
  }

  function toggleSidebarGroupSeparator(groupId: string, itemId: SidebarItemId): void {
    updateSidebarDraft((draft) => ({
      ...draft,
      groups: draft.groups.map((group) => {
        if (normalizeGroupId(group.id) !== normalizeGroupId(groupId)) return group
        const hasSeparator = group.separatorBeforeItemIds.includes(itemId)
        return {
          ...group,
          separatorBeforeItemIds: hasSeparator
            ? group.separatorBeforeItemIds.filter((id) => id !== itemId)
            : [...group.separatorBeforeItemIds, itemId]
        }
      })
    }))
  }

  function toggleSidebarItemHidden(itemId: SidebarItemId): void {
    updateSidebarDraft((draft) => {
      const isHidden = draft.hiddenItemIds.includes(itemId)
      return {
        ...draft,
        hiddenItemIds: isHidden
          ? draft.hiddenItemIds.filter((id) => id !== itemId)
          : [...draft.hiddenItemIds, itemId]
      }
    })
  }

  function updateSidebarGroupLabel(groupId: string, label: string): void {
    updateSidebarDraft((draft) => ({
      ...draft,
      groups: draft.groups.map((group) =>
        normalizeGroupId(group.id) === normalizeGroupId(groupId) ? { ...group, label } : group
      )
    }))
  }

  return {
    sidebarSettings,
    sidebarDraft,
    effectiveSidebarSettings,
    sidebarRootEntries,
    sidebarHiddenNavItems,
    setSidebarSettings,
    updateSidebarDraft,
    resetSidebarSettingsToDefault,
    cancelSidebarSettingsEditing,
    sidebarConfigItem,
    sidebarGroupIdFromLabel,
    addSidebarGroup,
    removeSidebarGroup,
    moveSidebarGroup,
    moveSidebarRootItem,
    moveSidebarItem,
    moveSidebarItemToGroup,
    toggleSidebarGroupSeparator,
    toggleSidebarItemHidden,
    updateSidebarGroupLabel
  }
})
