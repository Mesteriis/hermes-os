import type {
  TelegramChat,
  TelegramMessage
} from '../../../shared/communications/types/telegram'
import type { MessageDeliveryState } from '@/shared/ui'
import type { WhatsappWebMessage } from '../../../shared/communications/types/whatsapp'
import type {
  MessengerChannelKind,
  MessengerConversationKind,
  MessengerConversationModel,
  MessengerInspectorModel,
  MessengerListItemModel,
  MessengerMessageModel,
  MessengerWorkflowState
} from '../components/messengers/messengerElements'
import type { CommunicationProviderConversation } from '../types/providerChannels'

export function telegramMessengerListItem(
  chat: TelegramChat,
  selectedChatId: string,
  avatarSrc?: string
): MessengerListItemModel {
  const dialogState = telegramDialogState(chat.metadata)

  return {
    id: chat.telegram_chat_id,
    channelKind: 'telegram',
    accountId: chat.account_id,
    accountLabel: chat.account_id,
    conversationKind: telegramConversationKind(chat.chat_kind),
    title: chat.title,
    subtitle: chat.username ?? chat.provider_chat_id,
    preview: telegramLastMessagePreview(chat.metadata),
    timestampLabel: messageTimeLabel(chat.last_message_at ?? chat.updated_at),
    workflowState: dialogState.workflowState ?? messengerWorkflowState(chat.sync_state),
    unreadCount: dialogState.hideUnreadCounter ? undefined : dialogState.unreadCount,
    mentionCount: dialogState.mentionCount,
    muted: dialogState.muted,
    pinned: dialogState.pinned,
    selected: chat.telegram_chat_id === selectedChatId,
    labels: [chat.sync_state],
    profile: avatarSrc
      ? { displayName: chat.title, src: avatarSrc }
      : undefined,
  }
}

function telegramLastMessagePreview(metadata: Record<string, unknown>): string {
  const preview = metadata.last_message_preview
  return typeof preview === 'string' ? preview.trim() : ''
}

function telegramDialogState(metadata: Record<string, unknown>): {
  mentionCount: number | undefined
  hideUnreadCounter: boolean
  muted: boolean
  pinned: boolean
  unreadCount: number | undefined
  workflowState: MessengerWorkflowState | undefined
} {
  const isArchived = metadata.is_archived === true
  const isMuted = metadata.is_muted === true

  return {
    hideUnreadCounter: metadata.hide_unread_counter === true,
    mentionCount: nonNegativeInteger(
      metadata.provider_unread_mention_count ?? metadata.unread_mention_count
    ),
    muted: isMuted,
    pinned: metadata.is_pinned === true,
    unreadCount: nonNegativeInteger(
      metadata.provider_unread_count ?? metadata.unread_count
    ),
    workflowState: isArchived ? 'archived' : isMuted ? 'muted' : undefined,
  }
}

function nonNegativeInteger(value: unknown): number | undefined {
  return typeof value === 'number' && Number.isInteger(value) && value > 0
    ? value
    : undefined
}

export function whatsappMessengerListItem(
  conversation: CommunicationProviderConversation,
  selectedProviderChatId: string
): MessengerListItemModel {
  return {
    id: conversation.provider_chat_id,
    channelKind: 'whatsapp',
    accountId: conversation.account_id,
    accountLabel: conversation.account_id,
    conversationKind: providerConversationKind(conversation.chat_kind),
    title: conversation.title,
    subtitle: conversation.provider_chat_id,
    preview: conversation.sync_state ?? conversation.provider_chat_id,
    timestampLabel: messageTimeLabel(conversation.last_message_at ?? conversation.updated_at),
    workflowState: messengerWorkflowState(conversation.sync_state),
    selected: conversation.provider_chat_id === selectedProviderChatId,
    labels: conversation.sync_state ? [conversation.sync_state] : []
  }
}

export function telegramMessengerConversation(
  chat: TelegramChat | null,
  messages: readonly TelegramMessage[],
  selectedMessageId = ''
): MessengerConversationModel {
  if (!chat) return emptyMessengerConversation('telegram')

  return {
    id: chat.telegram_chat_id,
    channelKind: 'telegram',
    kind: telegramConversationKind(chat.chat_kind),
    title: chat.title,
    subtitle: chat.username ?? chat.provider_chat_id,
    workflowState: messengerWorkflowState(chat.sync_state),
    participantsLabel: chat.chat_kind,
    lastSeenLabel: chat.last_message_at ? messageTimeLabel(chat.last_message_at) : undefined,
    facts: [
      { label: 'account', value: chat.account_id },
      { label: 'provider chat', value: chat.provider_chat_id },
      { label: 'sync', value: chat.sync_state }
    ],
    messages: messages.map((message) => telegramMessengerMessage(message, selectedMessageId)),
    draftPreview: ''
  }
}

export function whatsappMessengerConversation(
  conversation: CommunicationProviderConversation | null,
  messages: readonly WhatsappWebMessage[]
): MessengerConversationModel {
  if (!conversation) return emptyMessengerConversation('whatsapp')

  return {
    id: conversation.provider_chat_id,
    channelKind: 'whatsapp',
    kind: providerConversationKind(conversation.chat_kind),
    title: conversation.title,
    subtitle: conversation.provider_chat_id,
    workflowState: messengerWorkflowState(conversation.sync_state),
    participantsLabel: conversation.chat_kind ?? 'conversation',
    lastSeenLabel: conversation.last_message_at ? messageTimeLabel(conversation.last_message_at) : undefined,
    facts: [
      { label: 'account', value: conversation.account_id },
      { label: 'provider chat', value: conversation.provider_chat_id },
      { label: 'sync', value: conversation.sync_state ?? 'projected' }
    ],
    messages: messages.map(whatsappMessengerMessage),
    draftPreview: ''
  }
}

export function messengerInspectorModel(
  channelKind: MessengerChannelKind,
  conversation: MessengerConversationModel
): MessengerInspectorModel {
  const projectedMessageCount = conversation.messages.length

  return {
    intelligence: {
      score: projectedMessageCount,
      maxScore: Math.max(projectedMessageCount, 1),
      label: 'Projected messages',
      summary: projectedMessageCount > 0
        ? 'Projected provider messages are available as Communications evidence.'
        : 'No projected messages are available for this provider conversation.',
      checks: [
        {
          id: 'provider-evidence',
          label: 'Provider evidence',
          description: `${projectedMessageCount} ${channelKind} messages read from Communications projections.`,
          icon: 'tabler:database',
          tone: projectedMessageCount > 0 ? 'success' : 'neutral'
        }
      ]
    },
    entityGroups: [],
    suggestedActions: [],
    relatedContext: []
  }
}

function telegramMessengerMessage(
  message: TelegramMessage,
  selectedMessageId: string
): MessengerMessageModel {
  const delivery = messageDeliveryPresentation(message.delivery_state)

  return {
    id: message.message_id,
    author: message.sender_display_name ?? message.sender,
    body: message.text,
    timestamp: messageTimeLabel(message.occurred_at ?? message.projected_at),
    direction: messageDirection(message.delivery_state),
    pending: delivery.pending,
    deliveryStatus: delivery.status,
    deliveryStatusLabel: delivery.label,
    meta: message.provider_message_id,
    selected: message.message_id === selectedMessageId,
    attachments: telegramMessageAttachments(message),
    reactions: telegramMessageReactions(message.metadata),
  }
}

function telegramMessageReactions(metadata: Record<string, unknown>): MessengerMessageModel['reactions'] {
  const summary = metadata.reaction_summary
  if (!isRecord(summary) || !Array.isArray(summary.reactions)) return undefined

  return summary.reactions.flatMap((reaction) => {
    if (!isRecord(reaction)) return []
    const emoji = stringValue(reaction.reaction_emoji)
    const count = nonNegativeInteger(reaction.count)
    if (!emoji || !count) return []

    return [{
      emoji,
      count,
      active: reaction.is_chosen === true,
    }]
  })
}

function telegramMessageAttachments(message: TelegramMessage): MessengerMessageModel['attachments'] {
  const metadata = message.metadata
  const attachments = metadata.attachments
  if (!Array.isArray(attachments)) return undefined

  return attachments.flatMap((attachment, index) => {
    if (!isRecord(attachment)) return []

    const id = stringValue(
      attachment.attachment_id ?? attachment.provider_attachment_id ?? attachment.id
    )
    if (!id) return []

    const attachmentType = stringValue(attachment.attachment_type ?? attachment.kind) ?? 'file'
    const contentType = stringValue(attachment.content_type ?? attachment.mime_type)
    const downloadState = stringValue(attachment.download_state) ?? 'remote'
    const tdlibFileId = positiveInteger(attachment.tdlib_file_id)
    const filename = stringValue(attachment.filename ?? attachment.file_name)
      ?? `${attachmentType}-${index + 1}`

    return [{
      id,
      name: filename,
      meta: [attachmentType, contentType, downloadState].filter(Boolean).join(' · '),
      icon: telegramAttachmentIcon(attachmentType),
      downloadable: tdlibFileId !== undefined,
      providerAttachmentId: stringValue(attachment.provider_attachment_id),
      providerMessageId: message.provider_message_id,
      tdlibFileId,
      contentType,
    }]
  })
}

function telegramAttachmentIcon(attachmentType: string): string {
  const icons: Record<string, string> = {
    animation: 'tabler:movie',
    audio: 'tabler:player-play',
    document: 'tabler:file-description',
    photo: 'tabler:photo',
    sticker: 'tabler:mood-smile',
    video: 'tabler:video',
    video_note: 'tabler:circle-play',
    voice: 'tabler:microphone',
  }

  return icons[attachmentType] ?? 'tabler:paperclip'
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null
}

function stringValue(value: unknown): string | undefined {
  return typeof value === 'string' && value.trim() ? value.trim() : undefined
}

function positiveInteger(value: unknown): number | undefined {
  return typeof value === 'number' && Number.isSafeInteger(value) && value > 0
    ? value
    : undefined
}

function whatsappMessengerMessage(message: WhatsappWebMessage): MessengerMessageModel {
  const delivery = messageDeliveryPresentation(message.delivery_state)

  return {
    id: message.message_id,
    author: message.sender_display_name ?? message.sender,
    body: message.text,
    timestamp: messageTimeLabel(message.occurred_at ?? message.projected_at),
    direction: messageDirection(message.delivery_state),
    pending: delivery.pending,
    deliveryStatus: delivery.status,
    deliveryStatusLabel: delivery.label,
    meta: message.provider_message_id,
    selected: false
  }
}

function emptyMessengerConversation(channelKind: MessengerChannelKind): MessengerConversationModel {
  return {
    id: `${channelKind}:empty`,
    channelKind,
    kind: 'direct',
    title: channelKind === 'telegram' ? 'Telegram' : 'WhatsApp',
    subtitle: 'No conversation selected',
    workflowState: 'waiting',
    participantsLabel: '0 participants',
    facts: [],
    messages: [],
    draftPreview: ''
  }
}

function telegramConversationKind(kind: TelegramChat['chat_kind']): MessengerConversationKind {
  if (kind === 'private' || kind === 'bot') return 'direct'
  if (kind === 'channel') return 'channel'
  return 'group'
}

function providerConversationKind(kind: string | undefined): MessengerConversationKind {
  if (kind === 'private' || kind === 'direct') return 'direct'
  if (kind === 'channel') return 'channel'
  return 'group'
}

function messengerWorkflowState(syncState: string | null | undefined): MessengerWorkflowState {
  if (syncState === 'degraded' || syncState === 'error') return 'needs_action'
  if (syncState === 'syncing') return 'waiting'
  return 'reviewed'
}

function messageDirection(deliveryState: string): MessengerMessageModel['direction'] {
  if (messageDeliveryPresentation(deliveryState).status) {
    return 'outbound'
  }
  return 'inbound'
}

function messageDeliveryPresentation(deliveryState: string): {
  status: MessageDeliveryState | undefined
  label: string | undefined
  pending: boolean
} {
  switch (deliveryState) {
    case 'queued':
    case 'scheduled':
      return { status: 'queued', label: undefined, pending: true }
    case 'sent':
      return { status: 'sent', label: undefined, pending: false }
    case 'delivered':
      return { status: 'delivered', label: undefined, pending: false }
    case 'read':
      return { status: 'read', label: undefined, pending: false }
    case 'failed':
    case 'send_failed':
      return { status: 'failed', label: undefined, pending: false }
    case 'send_blocked':
      return { status: 'failed', label: 'Blocked', pending: false }
    case 'send_dry_run':
      return { status: 'queued', label: 'Dry run', pending: false }
    default:
      return { status: undefined, label: undefined, pending: false }
  }
}

function messageTimeLabel(value: string | null): string {
  if (!value) return 'No activity'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return value
  return date.toLocaleString(undefined, {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  })
}
