import type { TreeSelectOption, UtilityTone } from '@/shared/ui'
import type {
  CommunicationCapabilityKind,
  CommunicationCapabilitySurfaceStatus,
  CommunicationSubSurface,
  CommunicationSurfaceCapability
} from '../../queries/communicationChannelSurface'
import type {
  CommunicationChannelActionGroupModel,
  CommunicationChannelActionModel,
  CommunicationChannelComposerCapabilityModel
} from '../communicationDomainElements'

export function channelProviderOptionsFromSubSurfaces(
  subSurfaces: readonly CommunicationSubSurface[]
): TreeSelectOption[] {
  const channelSurfaces = subSurfaces.filter((surface) => (
    surface.channelId === 'zulip'
    || surface.channelId === 'slack'
    || surface.channelId === 'discord'
    || surface.channelId === 'mattermost'
  ))

  return [
    {
      value: 'channels-providers',
      label: 'Channel providers',
      description: 'Zulip now; Slack, Discord and Mattermost later',
      icon: 'tabler:messages',
      children: channelSurfaces.map((surface) => ({
        value: `channels:${surface.channelId}`,
        label: surface.labelKey,
        description: surface.capabilityNotes[0],
        icon: channelProviderIcon(surface.channelId),
        disabled: surface.status === 'facade'
      }))
    },
    {
      value: 'channels-saved-filters',
      label: 'Saved filters',
      description: 'Reusable channel search surfaces',
      icon: 'tabler:filter-star',
      children: [
        {
          value: 'channels:filters:mentions',
          label: 'Mentions and review',
          description: 'Mentions, open candidates and owner review',
          icon: 'tabler:at'
        },
        {
          value: 'channels:filters:evidence',
          label: 'Evidence threads',
          description: 'Topics with source evidence or accepted signals',
          icon: 'tabler:shield-check'
        }
      ]
    }
  ]
}

export function channelActionGroupsFromSubSurface(
  surface: CommunicationSubSurface
): CommunicationChannelActionGroupModel[] {
  return surface.capabilityGroups.flatMap((group) => {
    const actions = group.capabilities
      .filter((capability) => channelCapabilityIsAction(capability.kind))
      .map((capability) => channelActionFromCapability(capability))

    if (actions.length === 0) {
      return []
    }

    return [{
      id: group.id,
      title: group.labelKey,
      menuLabel: group.menuLabelKey,
      icon: group.icon,
      tone: channelCapabilityTone(group.status),
      actions
    }]
  })
}

export function channelComposerCapabilitiesFromSubSurface(
  surface: CommunicationSubSurface
): CommunicationChannelComposerCapabilityModel[] {
  return surface.capabilityGroups.flatMap((group) => group.capabilities
    .filter((capability) => capability.kind === 'composer')
    .map((capability) => ({
      id: capability.id,
      label: capability.labelKey,
      icon: capability.icon,
      disabled: capability.disabled || capability.status === 'facade' || capability.status === 'blocked'
    })))
}

function channelActionFromCapability(capability: CommunicationSurfaceCapability): CommunicationChannelActionModel {
  return {
    id: capability.id,
    label: capability.labelKey,
    description: capability.descriptionKey,
    icon: capability.icon,
    tone: channelCapabilityTone(capability.status),
    contract: capability.contract,
    disabled: capability.disabled || capability.status === 'facade' || capability.status === 'blocked'
  }
}

function channelCapabilityIsAction(kind: CommunicationCapabilityKind): boolean {
  return kind === 'command' || kind === 'projection' || kind === 'runtime' || kind === 'inspector'
}

function channelCapabilityTone(status: CommunicationCapabilitySurfaceStatus): UtilityTone | undefined {
  if (status === 'available') return 'success'
  if (status === 'partial') return 'warning'
  if (status === 'blocked') return 'danger'
  return undefined
}

function channelProviderIcon(channelId: CommunicationSubSurface['channelId']): string {
  if (channelId === 'slack') return 'tabler:brand-slack'
  if (channelId === 'discord') return 'tabler:brand-discord'
  if (channelId === 'mattermost') return 'tabler:message-circle-cog'
  return 'tabler:messages'
}
