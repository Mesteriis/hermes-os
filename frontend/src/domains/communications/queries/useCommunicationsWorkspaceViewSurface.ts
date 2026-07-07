import { computed, ref, toValue, watch, type MaybeRefOrGetter } from 'vue'
import { getActivePinia } from 'pinia'
import {
  attachmentIcon,
  messageTime,
  senderEmail,
  senderLabel,
} from '../stores/communications'
import type {
  CommunicationAttachment,
  CommunicationMessageDetailItem,
  CommunicationMessageSummary,
  TranslationResponse,
} from '../types/communications'
import type {
  CommunicationConversationAttachmentModel,
  CommunicationConversationMessageModel,
  CommunicationConversationModel,
} from '../components/communicationDomainElements'
import type { MailListItemModel } from '../components/mail/mailElements'
import type {
  MailInspectorEntityItem,
  MailInspectorModel,
  MailInspectorSemanticFact,
  MailInspectorTopic,
} from '../components/mail/mailInspector'
import type { MessengerListItemModel } from '../components/messengers/messengerElements'
import {
  mailActionGroups,
  selectMailWorkspaceAction,
} from './communicationMailWorkspaceActions'
import { mailItem } from './communicationMailWorkspaceModels'
import {
  routeToAccountId,
  routeToChannelId,
  type PrimaryChannelId,
} from './communicationWorkspaceRoutes'
import {
  handleVisibleMailItemIdsChange as syncVisibleMailSelection,
} from './visibleMailSelection'
import {
  messengerInspectorModel,
  telegramMessengerConversation as buildTelegramMessengerConversation,
  telegramMessengerListItem,
  whatsappMessengerConversation as buildWhatsappMessengerConversation,
  whatsappMessengerListItem,
} from './communicationMessengerWorkspaceModels'
import {
  useTelegramChatsQuery,
  useTelegramMessagesQuery,
} from './telegramBusinessQueries'
import { useNotificationsStore, type NotificationItem } from '../../../shared/stores/notifications'
import { useCommunicationsPageSurface } from './useCommunicationsPageSurface'
import {
  useWhatsappBusinessConversationsQuery,
  useWhatsappBusinessMessagesQuery,
} from './whatsappBusinessQueries'

export function useCommunicationsWorkspaceViewSurface(
  selectedRouteId?: MaybeRefOrGetter<string | undefined>
) {
  const pageSurface = useCommunicationsPageSurface()
  const notificationsStore = getActivePinia() ? useNotificationsStore() : null
  const selectedTelegramChatId = ref('')
  const selectedWhatsappProviderChatId = ref('')
  const activeChannelId = computed<PrimaryChannelId>(
    () => routeToChannelId(toValue(selectedRouteId)) ?? 'mail'
  )
  const selectedRouteAccountId = computed(() =>
    routeToAccountId(toValue(selectedRouteId))
  )
  const selectedTelegramAccountId = computed(() =>
    activeChannelId.value === 'telegram'
      ? selectedRouteAccountId.value ?? undefined
      : undefined
  )
  const selectedWhatsappAccountId = computed(() =>
    activeChannelId.value === 'whatsapp'
      ? selectedRouteAccountId.value ?? null
      : null
  )

  watch(
    selectedRouteAccountId,
    (accountId) => {
      if (activeChannelId.value !== 'mail') return
      pageSurface.store.setSelectedMailAccountId(accountId ?? '')
    },
    { immediate: true }
  )

  watch(
    activeChannelId,
    (channelId) => {
      if (channelId !== 'mail') return
      pageSurface.store.setStateFilter('')
      pageSurface.store.setLocalStateFilter('all')
    },
    { immediate: true }
  )

  watch(
    pageSurface.visibleMailList,
    (messages) => {
      if (
        pageSurface.store.selectedCommunicationMessageId ||
        messages.length === 0
      )
        return
      pageSurface.handleSelectMessage(0)
    },
    { immediate: true }
  )

  watch(
    () => notificationsStore?.pendingNotificationTarget ?? null,
    (notification) => openNotificationTarget(notification),
    { immediate: true }
  )

  const mailItems = computed<MailListItemModel[]>(() =>
    pageSurface.visibleMailList.value.map((message) =>
      mailItem(message, pageSurface.store.selectedCommunicationMessageId)
    )
  )
  const mailSyncStatus = computed(() => {
    const statuses = pageSurface.store.mailSyncStatuses
    const selectedAccountId = pageSurface.store.selectedMailAccountId
    if (selectedAccountId) {
      return (
        statuses.find((status) => status.account_id === selectedAccountId) ??
        null
      )
    }

    return (
      statuses.find((status) => mailSyncStatusIsActive(status.status)) ??
      statuses[0] ??
      null
    )
  })

  const conversation = computed<CommunicationConversationModel>(() => {
    const detail = pageSurface.messageDetail.value?.message ?? null
    const attachments = pageSurface.messageDetail.value?.attachments ?? []
    const summary =
      pageSurface.store.selectedCommunication ??
      pageSurface.visibleMailList.value[0] ??
      null
    const source = detail ?? summary

    if (!source) return emptyConversation()

    return {
      id: source.conversation_id ?? source.message_id,
      channelKind: source.channel_kind,
      title: source.subject || senderLabel(source.sender),
      subtitle: senderEmail(source.sender),
      workflowState: source.workflow_state,
      facts: [
        { label: 'workflow', value: source.workflow_state },
        {
          label: 'attachments',
          value: attachmentCount(source, attachments.length),
        },
        { label: 'importance', value: source.importance_score ?? 'n/a' },
      ],
      messages: [
        conversationMessage(
          source,
          attachments,
          messageTranslation(
            pageSurface.store.mailMessageInsight,
            source.message_id
          )
        ),
      ],
      draftPreview: pageSurface.store.composeForm.body,
    }
  })

  const mailInspector = computed<MailInspectorModel>(() => {
    const detail = pageSurface.messageDetail.value?.message ?? null
    const attachments = pageSurface.messageDetail.value?.attachments ?? []
    const summary =
      pageSurface.store.selectedCommunication ??
      pageSurface.visibleMailList.value[0] ??
      null

    return mailInspectorModel(detail ?? summary, attachments.length)
  })

  const telegramChatsQuery = useTelegramChatsQuery(
    selectedTelegramAccountId,
    200
  )
  const telegramChats = computed(() => telegramChatsQuery.data.value ?? [])
  const selectedTelegramChat = computed(
    () =>
      telegramChats.value.find(
        (chat) => chat.telegram_chat_id === selectedTelegramChatId.value
      ) ?? null
  )
  const telegramMessagesQuery = useTelegramMessagesQuery(
    () => selectedTelegramChat.value?.account_id ?? null,
    () => selectedTelegramChat.value?.provider_chat_id ?? null,
    200
  )
  const telegramMessages = computed(
    () => telegramMessagesQuery.data.value ?? []
  )
  const telegramMessengerItems = computed(() =>
    telegramChats.value.map((chat) =>
      telegramMessengerListItem(chat, selectedTelegramChatId.value)
    )
  )
  const telegramMessengerConversation = computed(() =>
    buildTelegramMessengerConversation(
      selectedTelegramChat.value,
      telegramMessages.value
    )
  )
  const telegramMessengerInspector = computed(() =>
    messengerInspectorModel('telegram', telegramMessengerConversation.value)
  )

  const whatsappConversationsQuery = useWhatsappBusinessConversationsQuery(
    selectedWhatsappAccountId,
    200
  )
  const whatsappConversations = computed(
    () => whatsappConversationsQuery.data.value ?? []
  )
  const selectedWhatsappConversation = computed(
    () =>
      whatsappConversations.value.find(
        (conversation) =>
          conversation.provider_chat_id === selectedWhatsappProviderChatId.value
      ) ?? null
  )
  const whatsappMessagesQuery = useWhatsappBusinessMessagesQuery(
    () => selectedWhatsappConversation.value?.account_id ?? null,
    () => selectedWhatsappConversation.value?.provider_chat_id ?? null,
    200
  )
  const whatsappMessages = computed(
    () => whatsappMessagesQuery.data.value ?? []
  )
  const whatsappMessengerItems = computed(() =>
    whatsappConversations.value.map((conversation) =>
      whatsappMessengerListItem(
        conversation,
        selectedWhatsappProviderChatId.value
      )
    )
  )
  const whatsappMessengerConversation = computed(() =>
    buildWhatsappMessengerConversation(
      selectedWhatsappConversation.value,
      whatsappMessages.value
    )
  )
  const whatsappMessengerInspector = computed(() =>
    messengerInspectorModel('whatsapp', whatsappMessengerConversation.value)
  )

  watch(
    telegramChats,
    (items) => {
      if (
        items.some(
          (chat) => chat.telegram_chat_id === selectedTelegramChatId.value
        )
      )
        return

      selectedTelegramChatId.value = items[0]?.telegram_chat_id ?? ''
    },
    { immediate: true }
  )

  watch(
    whatsappConversations,
    (items) => {
      if (
        items.some(
          (conversation) =>
            conversation.provider_chat_id ===
            selectedWhatsappProviderChatId.value
        )
      )
        return

      selectedWhatsappProviderChatId.value = items[0]?.provider_chat_id ?? ''
    },
    { immediate: true }
  )
  return {
    activeChannelId,
    conversation,
    isMailActionRunning: computed(() => pageSurface.store.isMailActionRunning),
    mailActionError: computed(() => pageSurface.store.mailActionError),
    mailActionStatus: computed(() => pageSurface.store.mailActionStatus),
    mailInspector,
    mailItems,
    mailSyncStatus,
    pageSurface,
    refreshMail,
    selectMailAction,
    handleVisibleMailItemIdsChange,
    selectMailMessage,
    selectTelegramConversation,
    selectWhatsappConversation,
    telegramMessengerConversation,
    telegramMessengerInspector,
    telegramMessengerItems,
    whatsappMessengerConversation,
    whatsappMessengerInspector,
    whatsappMessengerItems,
    title: 'Communications',
    description:
      'Unified evidence-first workspace for mail, messenger channels, provider commands and review pressure.',
  }

  function selectTelegramConversation(item: MessengerListItemModel): void {
    selectedTelegramChatId.value = item.id
  }

  function selectWhatsappConversation(item: MessengerListItemModel): void {
    selectedWhatsappProviderChatId.value = item.id
  }

  function selectMailMessage(item: MailListItemModel): void {
    const messageIndex = pageSurface.visibleMailList.value.findIndex(
      (message) => message.message_id === item.id
    )
    if (messageIndex < 0) return
    pageSurface.handleSelectMessage(messageIndex)
  }

  function handleVisibleMailItemIdsChange(itemIds: string[]): void {
    syncVisibleMailSelection(pageSurface, itemIds)
  }

  function openNotificationTarget(notification: NotificationItem | null): void {
    if (notification?.targetView !== 'communications-mail') return

    if (notification.targetId) {
      pageSurface.store.selectMessageId(notification.targetId)
      pageSurface.store.setActiveMessageContextTab('message')
      pageSurface.store.setCommunicationMessageInsight(null)
    }

    notificationsStore?.consumePendingNotificationTarget()
  }

  function refreshMail(): void {
    void pageSurface.refetchMailList()
  }

  async function selectMailAction(actionId: string): Promise<void> {
    await selectMailWorkspaceAction(pageSurface, actionId)
  }
}

function mailSyncStatusIsActive(status: string): boolean {
  return (
    status === 'queued' ||
    status === 'running' ||
    status === 'recoverable_full_resync_needed'
  )
}

function attachmentCount(
  source: CommunicationMessageDetailItem | CommunicationMessageSummary,
  fallbackCount: number
): number {
  if ('attachment_count' in source) return source.attachment_count
  return fallbackCount
}

function conversationMessage(
  source: CommunicationMessageDetailItem | CommunicationMessageSummary,
  attachments: readonly CommunicationAttachment[],
  translation: TranslationResponse | null
): CommunicationConversationMessageModel {
  return {
    id: source.message_id,
    author: senderLabel(source.sender_display_name ?? source.sender),
    body: messageBody(source),
    bodyFormat: 'body_html' in source && source.body_html ? 'html' : 'plain',
    bodyHtml: 'body_html' in source ? source.body_html ?? undefined : undefined,
    bodyHtmlSanitized:
      'body_html' in source ? Boolean(source.body_html) : undefined,
    timestamp: messageTime(source.occurred_at ?? source.projected_at),
    direction: messageDirection(source.delivery_state),
    subject: source.subject,
    fromLabel: source.sender,
    toLabel: source.recipients.join(', '),
    meta: source.provider_record_id,
    attachments: attachments.map(conversationAttachment),
    translation:
      translation?.translated && translation.text?.trim()
        ? {
            text: translation.text.trim(),
            target: translation.target ?? 'ru',
            model: translation.model,
          }
        : undefined,
    evidenceItems: [
      {
        id: 'raw-record',
        label: 'raw record',
        value: source.raw_record_id,
        mono: true,
      },
      {
        id: 'provider-record',
        label: 'provider record',
        value: source.provider_record_id,
        mono: true,
      },
    ],
    markers: [
      { id: 'workflow', label: 'workflow', value: source.workflow_state },
      { id: 'delivery', label: 'delivery', value: source.delivery_state },
    ],
    actionGroups: mailActionGroups(source),
  }
}

function messageTranslation(
  insight: { messageId: string; translation: TranslationResponse | null } | null,
  messageId: string
): TranslationResponse | null {
  if (insight?.messageId !== messageId) return null

  return insight.translation
}

function messageBody(
  source: CommunicationMessageDetailItem | CommunicationMessageSummary
): string {
  if ('body_text' in source) return source.body_text
  return source.body_text_preview || source.ai_summary || source.subject
}

function messageDirection(
  deliveryState: string
): CommunicationConversationMessageModel['direction'] {
  if (
    deliveryState === 'sent' ||
    deliveryState === 'queued' ||
    deliveryState === 'scheduled'
  ) {
    return 'outbound'
  }
  return 'inbound'
}

function conversationAttachment(
  attachment: CommunicationAttachment
): CommunicationConversationAttachmentModel {
  return {
    id: attachment.attachment_id,
    name: attachment.filename ?? attachment.attachment_id,
    meta: `${attachment.content_type} · ${attachment.scan_status}`,
    icon: attachmentIcon(attachment.content_type),
    tone: attachment.scan_status === 'clean' ? 'success' : 'warning',
  }
}

function emptyConversation(): CommunicationConversationModel {
  return {
    id: 'empty',
    channelKind: 'mail',
    title: 'No message selected',
    subtitle: 'Import or select a communication to inspect source evidence.',
    workflowState: 'new',
    facts: [],
    messages: [],
    draftPreview: '',
  }
}

function mailInspectorModel(
  message: CommunicationMessageDetailItem | CommunicationMessageSummary | null,
  attachmentFallbackCount: number
): MailInspectorModel {
  if (!message) return emptyMailInspector()

  const attachmentTotal = attachmentCount(message, attachmentFallbackCount)
  const importanceScore = message.importance_score ?? 0

  return {
    intelligence: {
      score: importanceScore,
      maxScore: 100,
      label: 'Projected importance',
      summary:
        message.ai_summary ??
        'No backend summary is available for this message.',
      checks: [
        {
          id: 'raw-record',
          label: 'Raw record',
          description: message.raw_record_id,
          icon: 'tabler:file-database',
          tone: 'neutral',
        },
        {
          id: 'provider-record',
          label: 'Provider record',
          description: message.provider_record_id,
          icon: 'tabler:mail-code',
          tone: 'info',
        },
        {
          id: 'attachments',
          label: 'Attachments',
          description: String(attachmentTotal),
          icon: 'tabler:paperclip',
          tone: attachmentTotal > 0 ? 'success' : 'neutral',
        },
      ],
    },
    entityGroups: mailInspectorEntityGroups(message),
    topics: mailInspectorTopics(message),
    semanticFacts: mailInspectorFacts(message, attachmentTotal),
    suggestedActions: [],
    relatedContext: [],
  }
}

function emptyMailInspector(): MailInspectorModel {
  return {
    intelligence: {
      score: 0,
      maxScore: 100,
      label: 'Projected importance',
      summary: 'Select a message to inspect Communications evidence.',
      checks: [
        {
          id: 'empty',
          label: 'No message selected',
          description: 'No backend message projection is currently selected.',
          icon: 'tabler:circle-dashed',
          tone: 'neutral',
        },
      ],
    },
    entityGroups: [],
    topics: [],
    semanticFacts: [],
    suggestedActions: [],
    relatedContext: [],
  }
}

function mailInspectorEntityGroups(
  message: CommunicationMessageDetailItem | CommunicationMessageSummary
): MailInspectorModel['entityGroups'] {
  const items: MailInspectorEntityItem[] = [
    {
      id: 'raw-record',
      entity: 'document',
      title: 'Raw provider record',
      description: message.raw_record_id,
      evidenceLabel: 'Source record retained before promotion',
      tone: 'neutral',
    },
    {
      id: 'provider-record',
      entity: 'knowledge',
      title: 'Provider record',
      description: message.provider_record_id,
      evidenceLabel: message.channel_kind,
      tone: 'info',
    },
  ]

  if (message.observation_id) {
    items.push({
      id: 'observation',
      entity: 'knowledge',
      title: 'Observation',
      description: message.observation_id,
      evidenceLabel: 'Canonical observation reference',
      tone: 'info',
    })
  }

  if (message.ai_summary) {
    items.push({
      id: 'ai-summary',
      entity: 'knowledge',
      title: 'AI summary candidate',
      description: message.ai_summary,
      evidenceLabel:
        message.ai_summary_generated_at ??
        'summary generated without timestamp',
      tone: 'accent',
    })
  }

  return [
    {
      id: 'source-evidence',
      title: 'Source evidence',
      items,
    },
  ]
}

function mailInspectorTopics(
  message: CommunicationMessageDetailItem | CommunicationMessageSummary
): MailInspectorTopic[] {
  return message.ai_category
    ? [
        {
          id: 'ai-category',
          label: message.ai_category,
          tone: 'info',
        },
      ]
    : []
}

function mailInspectorFacts(
  message: CommunicationMessageDetailItem | CommunicationMessageSummary,
  attachmentTotal: number
): MailInspectorSemanticFact[] {
  const facts: MailInspectorSemanticFact[] = [
    {
      id: 'account',
      label: 'Account',
      value: message.account_id,
    },
    {
      id: 'workflow',
      label: 'Workflow',
      value: message.workflow_state,
    },
    {
      id: 'delivery',
      label: 'Delivery',
      value: message.delivery_state,
    },
    {
      id: 'attachments',
      label: 'Attachments',
      value: String(attachmentTotal),
    },
  ]

  if (message.importance_score !== null) {
    facts.push({
      id: 'importance',
      label: 'Importance',
      value: String(message.importance_score),
      tone: message.importance_score >= 75 ? 'warning' : 'neutral',
    })
  }

  if (message.occurred_at) {
    facts.push({
      id: 'occurred-at',
      label: 'Observed',
      value: messageTime(message.occurred_at),
    })
  }

  return facts
}
