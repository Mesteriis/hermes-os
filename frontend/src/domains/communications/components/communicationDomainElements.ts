import type { CommunicationTone, EntityIconKind, MessageDirection, ProviderIconKind, StatusIconKind, TreeSelectOption, UtilityTone } from '@/shared/ui'
import type { AttachmentScanStatus } from '../types/attachments'
import type { CommunicationOutboxItem, CommunicationThreadSummary } from '../types/communications'
import { outboxStatusPresentation } from './outboxStatus'
export type {
  CommunicationCallActionGroupModel,
  CommunicationCallActionModel,
  CommunicationCallActiveModel,
  CommunicationCallDateGroupModel,
  CommunicationCallInspectorModel,
  CommunicationCallItemModel,
  CommunicationCallKind,
  CommunicationCallMomentModel,
  CommunicationCallProviderKind,
  CommunicationCallRecordingModel,
  CommunicationCallsSurfaceModel,
  CommunicationCallState,
  CommunicationPermanentCallLinkModel
} from './calls/callElements'
export {
  communicationCallKindIconName,
  communicationCallProviderIconName,
  communicationCallProviderLabel,
  communicationCallStateLabel,
  communicationCallStateTone,
  createCommunicationCallDateGroups
} from './calls/callElements'

export type CommunicationChannelId = 'mail' | 'telegram' | 'whatsapp'
export type CommunicationChannelProviderKind = 'zulip' | 'slack' | 'discord' | 'mattermost'
export type CommunicationInboxKind = 'email' | 'direct_chat' | 'group_chat' | 'channel'
export type CommunicationSurfaceStatus = 'active' | 'partial' | 'facade' | 'blocked'
export type CommunicationCapabilityStatus = 'available' | 'partial' | 'facade' | 'blocked'
export type CommunicationCardSignalTone = 'accent' | 'info' | 'success' | 'warning' | 'danger'

export type CommunicationMetricItem = {
  label: string
  value: string | number
  tone?: UtilityTone
}

export type CommunicationChannelSurfaceCardModel = {
  channelId: CommunicationChannelId
  label: string
  status: CommunicationSurfaceStatus
  description: string
  accountCountLabel?: string
  lastActivityLabel?: string
  metricItems?: readonly CommunicationMetricItem[]
  capabilityLabels?: readonly string[]
}

export type CommunicationCapabilityCardModel = {
  id: string
  title: string
  description: string
  icon: string
  status: CommunicationCapabilityStatus
  surfaceLabel: string
  metricItems?: readonly CommunicationMetricItem[]
}

export type CommunicationThreadSignalCardModel = {
  thread: CommunicationThreadSummary
  channelKind?: string
  participantPreview?: string
  preview?: string
}

export type CommunicationInboxItemModel = {
  id: string
  kind: CommunicationInboxKind
  channelKind: string
  title: string
  subtitle: string
  preview: string
  timestamp: string
  workflowState: string
  unreadCount?: number
  hasOpenAction?: boolean
  hasAttachments?: boolean
  muted?: boolean
  selected?: boolean
}

export type CommunicationConversationAttachmentModel = {
  id: string
  name: string
  meta: string
  icon: string
  tone?: UtilityTone
  scanStatus?: AttachmentScanStatus
}

export type CommunicationEmailQuotedOriginalModel = {
  author: string
  timestamp: string
  subject: string
  body: string
}

export type CommunicationMessageTranslationModel = {
  text: string
  target: string
  model?: string
}

export type CommunicationMessageAttributeModel = {
  id: string
  label: string
  value: string | number
  tone?: UtilityTone
  mono?: boolean
}

export type CommunicationMessageAttributeGroupModel = {
  id: string
  title: string
  items: readonly CommunicationMessageAttributeModel[]
}

export type CommunicationMessageActionModel = {
  id: string
  label: string
  description: string
  icon: string
  tone?: UtilityTone
  contract?: string
  disabled?: boolean
}

export type CommunicationMessageActionGroupModel = {
  id: string
  title: string
  actions: readonly CommunicationMessageActionModel[]
}

export type CommunicationConversationMessageModel = {
  id: string
  author: string
  body: string
  bodyFormat?: 'plain' | 'html'
  bodyHtml?: string
  bodyHtmlSanitized?: boolean
  timestamp: string
  direction: MessageDirection
  subject?: string
  fromLabel?: string
  toLabel?: string
  ccLabel?: string
  bccLabel?: string
  replyToLabel?: string
  meta?: string
  tone?: CommunicationTone
  attachments?: readonly CommunicationConversationAttachmentModel[]
  translation?: CommunicationMessageTranslationModel
  quotedOriginal?: CommunicationEmailQuotedOriginalModel
  labels?: readonly string[]
  markers?: readonly CommunicationMessageAttributeModel[]
  evidenceItems?: readonly CommunicationMessageAttributeModel[]
  hermesEntities?: readonly CommunicationHermesEntityModel[]
  attributeGroups?: readonly CommunicationMessageAttributeGroupModel[]
  actionGroups?: readonly CommunicationMessageActionGroupModel[]
}

export type CommunicationConversationModel = {
  id: string
  channelKind: string
  title: string
  subtitle: string
  workflowState: string
  facts: readonly CommunicationMetricItem[]
  messages: readonly CommunicationConversationMessageModel[]
  draftPreview: string
  replyOriginal?: CommunicationEmailQuotedOriginalModel
}

export type CommunicationHermesEntityModel = {
  id: string
  entity: EntityIconKind
  title: string
  description: string
  evidenceLabel: string
  tone?: UtilityTone
}

export type CommunicationHermesInspectorSectionModel = {
  id: string
  title: string
  items: readonly CommunicationHermesEntityModel[]
}

export type CommunicationChannelRoomModel = {
  id: string
  providerKind: CommunicationChannelProviderKind
  accountId: string
  label: string
  description: string
  topicCountLabel?: string
  lastActivityLabel?: string
  unreadCount?: number
  mentionCount?: number
  selected?: boolean
}

export type CommunicationChannelDirectChatModel = {
  id: string
  providerKind: CommunicationChannelProviderKind
  accountId: string
  label: string
  description: string
  avatarLabel: string
  kindLabel?: string
  lastActivityLabel?: string
  unreadCount?: number
  mentionCount?: number
  selected?: boolean
}

export type CommunicationChannelDirectFolderModel = {
  id: string
  label: string
  description?: string
  expanded?: boolean
  chats: readonly CommunicationChannelDirectChatModel[]
}

export type CommunicationChannelTopicModel = {
  id: string
  label: string
  summary: string
  messageCountLabel: string
  selected?: boolean
  tone?: UtilityTone
}

export type CommunicationChannelComposerCapabilityModel = {
  id: string
  label: string
  icon: string
  disabled?: boolean
}

export type CommunicationChannelActionModel = {
  id: string
  label: string
  description: string
  icon: string
  tone?: UtilityTone
  contract?: string
  disabled?: boolean
}

export type CommunicationChannelActionGroupModel = {
  id: string
  title: string
  menuLabel: string
  icon: string
  tone?: UtilityTone
  actions: readonly CommunicationChannelActionModel[]
}

export type CommunicationChannelInspectorCheck = {
  id: string
  label: string
  description: string
  tone?: UtilityTone
  icon?: string
}

export type CommunicationChannelInspectorIntelligence = {
  score: number
  maxScore: number
  label: string
  summary: string
  checks: readonly CommunicationChannelInspectorCheck[]
}

export type CommunicationChannelInspectorEntityItem = {
  id: string
  entity: EntityIconKind
  title: string
  description: string
  evidenceLabel?: string
  tone?: UtilityTone
}

export type CommunicationChannelInspectorEntityGroup = {
  id: string
  title: string
  items: readonly CommunicationChannelInspectorEntityItem[]
}

export type CommunicationChannelInspectorTopic = {
  id: string
  label: string
  tone?: UtilityTone
}

export type CommunicationChannelInspectorSemanticFact = {
  id: string
  label: string
  value: string
  tone?: UtilityTone
}

export type CommunicationChannelInspectorActionItem = {
  id: string
  label: string
  description: string
  icon: string
  tone?: UtilityTone
  contract?: string
}

export type CommunicationChannelInspectorContextItem = {
  id: string
  title: string
  description: string
  icon: string
  tone?: UtilityTone
}

export type CommunicationChannelInspectorModel = {
  intelligence: CommunicationChannelInspectorIntelligence
  entityGroups: readonly CommunicationChannelInspectorEntityGroup[]
  topics: readonly CommunicationChannelInspectorTopic[]
  semanticFacts: readonly CommunicationChannelInspectorSemanticFact[]
  suggestedActions: readonly CommunicationChannelInspectorActionItem[]
  relatedContext: readonly CommunicationChannelInspectorContextItem[]
}

export type CommunicationChannelWorkspaceModel = {
  title: string
  subtitle?: string
  providerValue: string
  providerOptions: TreeSelectOption[]
  activeProviderKind: CommunicationChannelProviderKind
  activeProviderLabel: string
  activeAccountLabel: string
  rooms: readonly CommunicationChannelRoomModel[]
  directChatFolders: readonly CommunicationChannelDirectFolderModel[]
  activeRoomLabel: string
  activeRoomDescription: string
  activeTopicLabel: string
  topics: readonly CommunicationChannelTopicModel[]
  messages: readonly CommunicationConversationMessageModel[]
  composerPlaceholder: string
  composerCapabilities: readonly CommunicationChannelComposerCapabilityModel[]
  actionGroups: readonly CommunicationChannelActionGroupModel[]
  inspector: CommunicationChannelInspectorModel
}

export type CommunicationInboxItemPresentation = {
  channelIcon: ProviderIconKind
  channelLabel: string
  kindLabel: string
  status: CommunicationStatusPresentation
  signal: boolean
  signalTone: CommunicationCardSignalTone
}

export type CommunicationStatusPresentation = {
  label: string
  badgeTone: UtilityTone
  statusIcon: StatusIconKind
  signalTone: CommunicationCardSignalTone
}

export type CommunicationThreadCardPresentation = {
  status: CommunicationStatusPresentation
  channelIcon: ProviderIconKind
  channelLabel: string
  signal: boolean
  signalTone: CommunicationCardSignalTone
  facts: readonly CommunicationMetricItem[]
}

export type CommunicationOutboxCardPresentation = {
  title: string
  detail: string
  badgeTone: UtilityTone
  statusIcon: StatusIconKind
  canUndo: boolean
}

const surfaceStatus: Record<CommunicationSurfaceStatus, CommunicationStatusPresentation> = {
  active: {
    label: 'Active',
    badgeTone: 'success',
    statusIcon: 'success',
    signalTone: 'success'
  },
  partial: {
    label: 'Partial',
    badgeTone: 'warning',
    statusIcon: 'warning',
    signalTone: 'warning'
  },
  facade: {
    label: 'Facade',
    badgeTone: 'info',
    statusIcon: 'idle',
    signalTone: 'info'
  },
  blocked: {
    label: 'Blocked',
    badgeTone: 'danger',
    statusIcon: 'danger',
    signalTone: 'danger'
  }
}

const capabilityStatus: Record<CommunicationCapabilityStatus, CommunicationStatusPresentation> = {
  available: {
    label: 'Available',
    badgeTone: 'success',
    statusIcon: 'success',
    signalTone: 'success'
  },
  partial: {
    label: 'Partial',
    badgeTone: 'warning',
    statusIcon: 'warning',
    signalTone: 'warning'
  },
  facade: {
    label: 'Facade',
    badgeTone: 'info',
    statusIcon: 'idle',
    signalTone: 'info'
  },
  blocked: {
    label: 'Blocked',
    badgeTone: 'danger',
    statusIcon: 'danger',
    signalTone: 'danger'
  }
}

const workflowStatus: Record<string, CommunicationStatusPresentation> = {
  new: {
    label: 'New',
    badgeTone: 'info',
    statusIcon: 'active',
    signalTone: 'info'
  },
  needs_action: {
    label: 'Needs action',
    badgeTone: 'warning',
    statusIcon: 'warning',
    signalTone: 'warning'
  },
  waiting: {
    label: 'Waiting',
    badgeTone: 'neutral',
    statusIcon: 'idle',
    signalTone: 'accent'
  },
  reviewed: {
    label: 'Reviewed',
    badgeTone: 'success',
    statusIcon: 'success',
    signalTone: 'success'
  },
  done: {
    label: 'Done',
    badgeTone: 'success',
    statusIcon: 'success',
    signalTone: 'success'
  },
  archived: {
    label: 'Archived',
    badgeTone: 'neutral',
    statusIcon: 'idle',
    signalTone: 'accent'
  },
  muted: {
    label: 'Muted',
    badgeTone: 'neutral',
    statusIcon: 'offline',
    signalTone: 'accent'
  },
  spam: {
    label: 'Spam',
    badgeTone: 'danger',
    statusIcon: 'danger',
    signalTone: 'danger'
  }
}

const inboxKindLabels: Record<CommunicationInboxKind, string> = {
  email: 'Email',
  direct_chat: 'Chat',
  group_chat: 'Group',
  channel: 'Channel'
}

export function communicationSurfaceStatusPresentation(
  status: CommunicationSurfaceStatus
): CommunicationStatusPresentation {
  return surfaceStatus[status]
}

export function communicationCapabilityStatusPresentation(
  status: CommunicationCapabilityStatus
): CommunicationStatusPresentation {
  return capabilityStatus[status]
}

export function communicationChannelProviderIcon(channelKind?: string): ProviderIconKind {
  if (channelKind === 'telegram') return 'telegram'
  if (channelKind === 'whatsapp') return 'whatsapp'
  if (channelKind === 'mail' || channelKind === 'email') return 'mail'
  return 'generic'
}

export function communicationChannelLabel(channelKind?: string): string {
  if (channelKind === 'telegram') return 'Telegram'
  if (channelKind === 'whatsapp') return 'WhatsApp'
  if (channelKind === 'mail' || channelKind === 'email') return 'Mail'
  return 'Communication'
}

export function communicationChannelProviderIconName(providerKind: CommunicationChannelProviderKind): string {
  if (providerKind === 'slack') return 'tabler:brand-slack'
  if (providerKind === 'discord') return 'tabler:brand-discord'
  if (providerKind === 'mattermost') return 'tabler:message-circle-cog'
  return 'tabler:messages'
}

export function communicationChannelProviderLabel(providerKind: CommunicationChannelProviderKind): string {
  if (providerKind === 'slack') return 'Slack'
  if (providerKind === 'discord') return 'Discord'
  if (providerKind === 'mattermost') return 'Mattermost'
  return 'Zulip'
}

export function communicationChannelDirectChatCount(
  folders: readonly CommunicationChannelDirectFolderModel[]
): number {
  let total = 0
  for (const folder of folders) {
    total += folder.chats.length
  }
  return total
}

export function communicationConversationIsEmail(channelKind?: string): boolean {
  return channelKind === 'mail' || channelKind === 'email'
}

export function communicationWorkflowStatusPresentation(workflowState: string): CommunicationStatusPresentation {
  return workflowStatus[workflowState] ?? workflowStatus.new
}

export function communicationInboxItemPresentation(
  item: CommunicationInboxItemModel
): CommunicationInboxItemPresentation {
  const status = communicationWorkflowStatusPresentation(item.workflowState)

  return {
    channelIcon: communicationChannelProviderIcon(item.channelKind),
    channelLabel: communicationChannelLabel(item.channelKind),
    kindLabel: inboxKindLabels[item.kind],
    status,
    signal: Boolean(item.hasOpenAction || item.unreadCount),
    signalTone: item.hasOpenAction ? 'warning' : status.signalTone
  }
}

export function communicationThreadCardPresentation(
  model: CommunicationThreadSignalCardModel
): CommunicationThreadCardPresentation {
  const status = communicationWorkflowStatusPresentation(model.thread.dominant_workflow_state)

  return {
    status,
    channelIcon: communicationChannelProviderIcon(model.channelKind),
    channelLabel: communicationChannelLabel(model.channelKind),
    signal: model.thread.has_open_action,
    signalTone: model.thread.has_open_action ? 'warning' : status.signalTone,
    facts: [
      { label: 'messages', value: model.thread.message_count },
      { label: 'participants', value: model.thread.participant_count },
      { label: 'attachments', value: model.thread.has_attachments ? 'yes' : 'no' }
    ]
  }
}

export function communicationOutboxCardPresentation(
  item: CommunicationOutboxItem,
  now: Date
): CommunicationOutboxCardPresentation {
  const status = outboxStatusPresentation(item, now)

  return {
    title: status.title,
    detail: status.detail,
    badgeTone: outboxToneToBadgeTone(status.tone),
    statusIcon: outboxToneToStatusIcon(status.tone),
    canUndo: status.canUndo
  }
}

function outboxToneToBadgeTone(tone: string): UtilityTone {
  if (tone === 'success') return 'success'
  if (tone === 'warning') return 'warning'
  if (tone === 'danger') return 'danger'
  return 'neutral'
}

function outboxToneToStatusIcon(tone: string): StatusIconKind {
  if (tone === 'success') return 'success'
  if (tone === 'warning') return 'warning'
  if (tone === 'danger') return 'danger'
  return 'idle'
}
