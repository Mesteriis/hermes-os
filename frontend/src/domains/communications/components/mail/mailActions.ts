import type { NavigationItem } from '@/shared/ui/Navigation.types'
import type {
  CommunicationMessageActionGroupModel,
  CommunicationMessageActionModel
} from '../communicationDomainElements'

type Translate = (key: string) => string

type MailActionGroupDefinition = {
  id: string
  label: string
  menuLabel: string
  icon: string
  tone?: CommunicationMessageActionModel['tone']
  actionIds: readonly string[]
}

export type MailActionMenuGroup = {
  id: string
  label: string
  menuLabel: string
  icon: string
  tone?: CommunicationMessageActionModel['tone']
  items: NavigationItem[]
}

export type MailActionToolbarControl =
  | {
      kind: 'button'
      id: string
      label: string
      icon: string
      tone?: CommunicationMessageActionModel['tone']
      disabled?: boolean
    }
  | {
      kind: 'split'
      id: string
      label: string
      menuLabel: string
      icon: string
      tone?: CommunicationMessageActionModel['tone']
      disabled?: boolean
      items: NavigationItem[]
    }

type MailActionSectionId = 'routing' | 'intelligence' | 'evidence' | 'danger'

type MailActionSectionDefinition = {
  id: MailActionSectionId
  label: string
  groupIds: readonly string[]
}

export type MailActionToolbarSection = {
  id: MailActionSectionId
  label: string
  groups: MailActionMenuGroup[]
}

const mailActionSectionDefinitions: readonly MailActionSectionDefinition[] = [
  {
    id: 'routing',
    label: 'State and filing',
    groupIds: ['state', 'organization']
  },
  {
    id: 'intelligence',
    label: 'Hermes and creation',
    groupIds: ['hermes', 'create']
  },
  {
    id: 'evidence',
    label: 'Evidence checks',
    groupIds: ['evidence']
  },
  {
    id: 'danger',
    label: 'Danger zone',
    groupIds: ['danger']
  }
]

const mailActionResponseControlDefinitions = [
  {
    kind: 'split',
    id: 'reply',
    actionId: 'reply',
    menuLabel: 'Open reply actions',
    itemIds: ['ai-reply', 'ai-reply-variants', 'bilingual-reply-flow', 'smart-cc']
  },
  {
    kind: 'button',
    id: 'reply-all',
    actionId: 'reply-all'
  },
  {
    kind: 'split',
    id: 'forward',
    actionId: 'forward',
    menuLabel: 'Open forwarding actions',
    itemIds: ['forward-eml', 'redirect']
  }
] as const

const mailActionGroupDefinitions: readonly MailActionGroupDefinition[] = [
  {
    id: 'reply',
    label: 'Reply actions',
    menuLabel: 'Open reply actions',
    icon: 'tabler:corner-up-left',
    actionIds: ['reply', 'reply-all', 'ai-reply', 'ai-reply-variants', 'bilingual-reply-flow', 'smart-cc']
  },
  {
    id: 'forwarding',
    label: 'Forwarding actions',
    menuLabel: 'Open forwarding actions',
    icon: 'tabler:send',
    actionIds: ['forward', 'forward-eml', 'redirect']
  },
  {
    id: 'state',
    label: 'State and routing',
    menuLabel: 'Open state actions',
    icon: 'tabler:activity',
    tone: 'info',
    actionIds: [
      'mark-read',
      'mark-unread',
      'mark-spam',
      'archive',
      'restore-trash',
      'bulk-actions',
      'archive-response',
      'snooze',
      'mute'
    ]
  },
  {
    id: 'organization',
    label: 'Message organization',
    menuLabel: 'Open organization actions',
    icon: 'tabler:folder-symlink',
    actionIds: ['pin', 'important', 'add-label', 'remove-label', 'move-folder', 'copy-folder']
  },
  {
    id: 'hermes',
    label: 'Hermes actions',
    menuLabel: 'Open Hermes actions',
    icon: 'tabler:sparkles',
    tone: 'accent',
    actionIds: ['analyze', 'explain', 'update-ai-state', 'translate', 'extract-tasks', 'extract-notes']
  },
  {
    id: 'create',
    label: 'Create from message',
    menuLabel: 'Open creation actions',
    icon: 'tabler:circle-plus',
    tone: 'success',
    actionIds: ['create-task', 'create-document', 'create-event', 'create-persona']
  },
  {
    id: 'evidence',
    label: 'Evidence and safety',
    menuLabel: 'Open evidence actions',
    icon: 'tabler:shield-check',
    tone: 'warning',
    actionIds: ['auth-check', 'spf-dkim', 'language', 'export-md', 'export-eml', 'export-json']
  },
  {
    id: 'danger',
    label: 'Danger zone',
    menuLabel: 'Open destructive actions',
    icon: 'tabler:trash',
    tone: 'danger',
    actionIds: ['trash', 'delete-provider']
  }
]

export function mailActionMenuGroups(
  actionGroups: readonly CommunicationMessageActionGroupModel[] | undefined,
  translate: Translate
): MailActionMenuGroup[] {
  const actionsById = indexMailActions(actionGroups)

  return mailActionGroupDefinitions
    .map((group) => ({
      id: group.id,
      label: translate(group.label),
      menuLabel: translate(group.menuLabel),
      icon: group.icon,
      tone: group.tone,
      items: group.actionIds.flatMap((actionId) => {
        const action = actionsById.get(actionId)
        return action ? [mailActionNavigationItem(action, translate)] : []
      })
    }))
    .filter((group) => group.items.length > 0)
}

export function mailActionToolbarSections(
  actionGroups: readonly CommunicationMessageActionGroupModel[] | undefined,
  translate: Translate
): MailActionToolbarSection[] {
  const groupsById = new Map(mailActionMenuGroups(actionGroups, translate).map((group) => [group.id, group]))

  return mailActionSectionDefinitions
    .map((section) => ({
      id: section.id,
      label: translate(section.label),
      groups: section.groupIds.flatMap((groupId) => {
        const group = groupsById.get(groupId)
        return group ? [group] : []
      })
    }))
    .filter((section) => section.groups.length > 0)
}

export function mailActionResponseControls(
  actionGroups: readonly CommunicationMessageActionGroupModel[] | undefined,
  translate: Translate
): MailActionToolbarControl[] {
  const actionsById = indexMailActions(actionGroups)

  return mailActionResponseControlDefinitions.flatMap<MailActionToolbarControl>((definition) => {
    const action = actionsById.get(definition.actionId)

    if (!action) {
      return []
    }

    if (definition.kind === 'button') {
      return [{
        kind: 'button',
        id: definition.id,
        label: translate(action.label),
        icon: action.icon,
        tone: action.tone,
        disabled: action.disabled
      } satisfies MailActionToolbarControl]
    }

    return [{
      kind: 'split',
      id: definition.id,
      label: translate(action.label),
      menuLabel: translate(definition.menuLabel),
      icon: action.icon,
      tone: action.tone,
      disabled: action.disabled,
      items: definition.itemIds.flatMap((actionId) => {
        const menuAction = actionsById.get(actionId)
        return menuAction ? [mailActionNavigationItem(menuAction, translate)] : []
      })
    } satisfies MailActionToolbarControl]
  })
}

function indexMailActions(
  actionGroups: readonly CommunicationMessageActionGroupModel[] | undefined
): Map<string, CommunicationMessageActionModel> {
  const actionsById = new Map<string, CommunicationMessageActionModel>()

  for (const group of actionGroups ?? []) {
    for (const action of group.actions) {
      actionsById.set(action.id, action)
    }
  }

  return actionsById
}

function mailActionNavigationItem(
  action: CommunicationMessageActionModel,
  translate: Translate
): NavigationItem {
  return {
    id: action.id,
    label: translate(action.label),
    icon: action.icon,
    disabled: action.disabled
  }
}
