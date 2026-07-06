import type {
  TelegramChat,
  TelegramMessage
} from '../../../shared/communications/types/telegram'
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
  selectedChatId: string
): MessengerListItemModel {
  return {
    id: chat.telegram_chat_id,
    channelKind: 'telegram',
    accountId: chat.account_id,
    accountLabel: chat.account_id,
    conversationKind: telegramConversationKind(chat.chat_kind),
    title: chat.title,
    subtitle: chat.username ?? chat.provider_chat_id,
    preview: chat.sync_state,
    timestampLabel: messageTimeLabel(chat.last_message_at ?? chat.updated_at),
    workflowState: messengerWorkflowState(chat.sync_state),
    selected: chat.telegram_chat_id === selectedChatId,
    labels: [chat.sync_state]
  }
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
  messages: readonly TelegramMessage[]
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
    messages: messages.map(telegramMessengerMessage),
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

function telegramMessengerMessage(message: TelegramMessage): MessengerMessageModel {
  return {
    id: message.message_id,
    author: message.sender_display_name ?? message.sender,
    body: message.text,
    timestamp: messageTimeLabel(message.occurred_at ?? message.projected_at),
    direction: messageDirection(message.delivery_state),
    meta: message.provider_message_id,
    selected: false
  }
}

function whatsappMessengerMessage(message: WhatsappWebMessage): MessengerMessageModel {
  return {
    id: message.message_id,
    author: message.sender_display_name ?? message.sender,
    body: message.text,
    timestamp: messageTimeLabel(message.occurred_at ?? message.projected_at),
    direction: messageDirection(message.delivery_state),
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
  if (deliveryState === 'sent' || deliveryState === 'queued' || deliveryState === 'scheduled') {
    return 'outbound'
  }
  return 'inbound'
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
