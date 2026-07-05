import type { TreeSelectOption, UtilityTone } from '@/shared/ui'
import type {
  CommunicationChannelInspectorActionItem,
  CommunicationChannelInspectorContextItem,
  CommunicationChannelInspectorEntityGroup,
  CommunicationChannelInspectorIntelligence,
  CommunicationChannelInspectorSemanticFact,
  CommunicationChannelInspectorTopic
} from '../communicationDomainElements'

export type CommunicationCallItemModel = {
  id: string
  providerKind: CommunicationCallProviderKind
  kind: CommunicationCallKind
  dateGroupLabel: string
  sortKey: string
  title: string
  subtitle: string
  providerLabel: string
  participantsLabel: string
  startedAtLabel: string
  durationLabel: string
  state: CommunicationCallState
  summary: string
  avatarLabel: string
  recordingCount?: number
  transcriptStateLabel?: string
  recurrenceLabel?: string
  unreadCount?: number
  selected?: boolean
}

export type CommunicationCallDateGroupModel = {
  id: string
  label: string
  calls: CommunicationCallItemModel[]
}

export type CommunicationPermanentCallLinkModel = {
  id: string
  providerKind: CommunicationCallProviderKind
  providerLabel: string
  title: string
  description: string
  href: string
  statusLabel?: string
  tone?: UtilityTone
}

export type CommunicationCallKind = 'scheduled' | 'recurring' | 'room' | 'recording'
export type CommunicationCallProviderKind =
  | 'phone'
  | 'zoom'
  | 'telemost'
  | 'zulip'
  | 'google_meet'
  | 'teams'
  | 'whatsapp'
  | 'telegram'
export type CommunicationCallState =
  | 'live'
  | 'missed'
  | 'completed'
  | 'scheduled'
  | 'recurring'
  | 'recording'
  | 'transcribing'

export type CommunicationCallMomentModel = {
  id: string
  timestamp: string
  speaker: string
  text: string
  tone?: UtilityTone
  evidenceLabel?: string
}

export type CommunicationCallRecordingModel = {
  id: string
  title: string
  meta: string
  statusLabel: string
  icon: string
  tone?: UtilityTone
}

export type CommunicationCallActionModel = {
  id: string
  label: string
  description?: string
  icon: string
  tone?: UtilityTone
  disabled?: boolean
  contract?: string
}

export type CommunicationCallActionGroupModel = {
  id: string
  title: string
  icon: string
  menuLabel: string
  actions: readonly CommunicationCallActionModel[]
}

export type CommunicationCallInspectorModel = {
  intelligence: CommunicationChannelInspectorIntelligence
  entityGroups: readonly CommunicationChannelInspectorEntityGroup[]
  topics: readonly CommunicationChannelInspectorTopic[]
  semanticFacts: readonly CommunicationChannelInspectorSemanticFact[]
  suggestedActions: readonly CommunicationChannelInspectorActionItem[]
  relatedContext: readonly CommunicationChannelInspectorContextItem[]
}

export type CommunicationCallActiveModel = {
  id: string
  title: string
  subtitle: string
  statusLabel: string
  statusTone: UtilityTone
  providerKind: CommunicationCallProviderKind
  providerLabel: string
  startedAtLabel: string
  durationLabel: string
  participantCountLabel: string
  recurrenceLabel?: string
  recordingStatusLabel: string
  transcriptStatusLabel: string
  summary: string
  recordings: readonly CommunicationCallRecordingModel[]
  moments: readonly CommunicationCallMomentModel[]
}

export type CommunicationCallsSurfaceModel = {
  title: string
  providerValue: string
  providerOptions: TreeSelectOption[]
  permanentMeetings: readonly CommunicationPermanentCallLinkModel[]
  calls: readonly CommunicationCallItemModel[]
  activeCall: CommunicationCallActiveModel
  actionGroups: readonly CommunicationCallActionGroupModel[]
  inspector: CommunicationCallInspectorModel
}

export function communicationCallStateTone(state: CommunicationCallItemModel['state']): UtilityTone {
  if (state === 'missed') return 'danger'
  if (state === 'recording' || state === 'transcribing') return 'warning'
  if (state === 'completed' || state === 'live') return 'success'
  if (state === 'recurring') return 'accent'
  return 'info'
}

export function createCommunicationCallDateGroups(
  calls: readonly CommunicationCallItemModel[]
): CommunicationCallDateGroupModel[] {
  const groups: CommunicationCallDateGroupModel[] = []
  const groupByLabel = new Map<string, CommunicationCallDateGroupModel>()
  const sortedCalls = [...calls].sort((left, right) => right.sortKey.localeCompare(left.sortKey))

  for (const call of sortedCalls) {
    let group = groupByLabel.get(call.dateGroupLabel)
    if (!group) {
      group = {
        id: call.dateGroupLabel,
        label: call.dateGroupLabel,
        calls: []
      }
      groupByLabel.set(call.dateGroupLabel, group)
      groups.push(group)
    }

    group.calls.push(call)
  }

  return groups
}

export function communicationCallProviderIconName(providerKind: CommunicationCallProviderKind): string {
  if (providerKind === 'zoom') return 'tabler:brand-zoom'
  if (providerKind === 'zulip') return 'tabler:messages'
  if (providerKind === 'google_meet') return 'tabler:brand-google'
  if (providerKind === 'teams') return 'tabler:brand-teams'
  if (providerKind === 'telemost') return 'tabler:video'
  if (providerKind === 'whatsapp') return 'tabler:brand-whatsapp'
  if (providerKind === 'telegram') return 'tabler:brand-telegram'
  return 'tabler:phone-call'
}

export function communicationCallProviderLabel(providerKind: CommunicationCallProviderKind): string {
  if (providerKind === 'zoom') return 'Zoom'
  if (providerKind === 'zulip') return 'Zulip'
  if (providerKind === 'google_meet') return 'Google Meet'
  if (providerKind === 'teams') return 'Microsoft Teams'
  if (providerKind === 'telemost') return 'Yandex Telemost'
  if (providerKind === 'whatsapp') return 'WhatsApp'
  if (providerKind === 'telegram') return 'Telegram'
  return 'Phone'
}

export function communicationCallKindIconName(kind: CommunicationCallKind): string {
  if (kind === 'recurring') return 'tabler:repeat'
  if (kind === 'room') return 'tabler:door'
  if (kind === 'recording') return 'tabler:player-record'
  return 'tabler:calendar-time'
}

export function communicationCallStateLabel(state: CommunicationCallState): string {
  if (state === 'live') return 'Live'
  if (state === 'missed') return 'Missed'
  if (state === 'completed') return 'Completed'
  if (state === 'scheduled') return 'Scheduled'
  if (state === 'recurring') return 'Recurring'
  if (state === 'recording') return 'Recording'
  return 'Transcribing'
}
