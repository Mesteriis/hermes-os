import { computed, nextTick, ref, watch } from 'vue'
import { useI18n } from '../../../platform/i18n'
import { useAttachmentPreviewQuery } from './useCommunicationsQuery'
import {
  useWhatsappBusinessConversationsQuery,
  useAddWhatsappReactionMutation,
  useArchiveWhatsappConversationMutation,
  useMarkWhatsappConversationReadMutation,
  useMarkWhatsappConversationUnreadMutation,
  useMuteWhatsappConversationMutation,
  useWhatsappConversationDetailQuery,
  useWhatsappConversationMembersQuery,
  useWhatsappBusinessMessagesQuery,
  useDeleteWhatsappMessageMutation,
  useEditWhatsappMessageMutation,
  useForwardWhatsappMessageMutation,
  useRemoveWhatsappReactionMutation,
  useWhatsappMediaSearchQuery,
  useWhatsappMessageSearchQuery,
  useWhatsappPinnedMessagesQuery,
  usePinWhatsappConversationMutation,
  useReplyWhatsappMessageMutation,
  useSendWhatsappMessageMutation,
  useUnarchiveWhatsappConversationMutation,
  useUnmuteWhatsappConversationMutation,
  useUnpinWhatsappConversationMutation,
} from './whatsappBusinessQueries'
import type { WhatsappWebMediaItem } from '../../../shared/communications/types/whatsapp'
import {
  firstPreviewableMediaAttachmentId,
  mediaAttachmentId,
  type WhatsAppPanelMessage,
} from './useWhatsappCommunicationsPresentation'

export function useWhatsappCommunicationsPanelSurface() {
  const { t } = useI18n()

  const selectedConversationId = ref('')
  const draftText = ref('')
  const searchText = ref('')
  const browserMode = ref<'timeline' | 'media'>('timeline')
  const mediaKindFilter = ref<'all' | 'image' | 'video' | 'audio' | 'document'>('all')
  const selectedMediaAttachmentId = ref<string | null>(null)
  const actionMessage = ref('')
  const actionError = ref('')
  const editingMessageId = ref<string | null>(null)
  const editDraftText = ref('')
  const forwardingMessageId = ref<string | null>(null)
  const forwardTargetConversationId = ref('')
  const forwardTargetFilter = ref('')
  const messageElementMap = new Map<string, HTMLElement>()

  const conversationsQuery = useWhatsappBusinessConversationsQuery(undefined, 200)
  const conversations = computed(() => conversationsQuery.data.value ?? [])
  const selectedConversationSummary = computed(
    () =>
      conversations.value.find(
        (conversation) => conversation.provider_chat_id === selectedConversationId.value
      ) ?? null
  )
  const conversationDetailQuery = useWhatsappConversationDetailQuery(
    () => selectedConversationSummary.value?.conversation_id ?? null
  )
  const selectedConversation = computed(
    () => conversationDetailQuery.data.value ?? selectedConversationSummary.value
  )
  const messagesQuery = useWhatsappBusinessMessagesQuery(
    () => selectedConversation.value?.account_id ?? null,
    () => selectedConversation.value?.provider_chat_id ?? null,
    200
  )
  const searchQuery = useWhatsappMessageSearchQuery({
    q: searchText,
    accountId: () => selectedConversation.value?.account_id ?? null,
    providerChatId: () => selectedConversation.value?.provider_chat_id ?? null,
    limit: 50,
  })
  const pinnedMessagesQuery = useWhatsappPinnedMessagesQuery({
    conversationId: () => selectedConversation.value?.conversation_id ?? null,
    limit: 20,
  })
  const membersQuery = useWhatsappConversationMembersQuery(
    () => selectedConversation.value?.conversation_id ?? null,
    50
  )
  const mediaQuery = useWhatsappMediaSearchQuery({
    q: () => searchText.value.trim() || undefined,
    accountId: () => selectedConversation.value?.account_id ?? null,
    providerChatId: () => selectedConversation.value?.provider_chat_id ?? null,
    kind: () => (mediaKindFilter.value === 'all' ? undefined : mediaKindFilter.value),
    limit: 20,
  })

  const sendMutation = useSendWhatsappMessageMutation()
  const replyMutation = useReplyWhatsappMessageMutation()
  const forwardMutation = useForwardWhatsappMessageMutation()
  const editMutation = useEditWhatsappMessageMutation()
  const deleteMutation = useDeleteWhatsappMessageMutation()
  const pinConversationMutation = usePinWhatsappConversationMutation()
  const unpinConversationMutation = useUnpinWhatsappConversationMutation()
  const archiveConversationMutation = useArchiveWhatsappConversationMutation()
  const unarchiveConversationMutation = useUnarchiveWhatsappConversationMutation()
  const muteConversationMutation = useMuteWhatsappConversationMutation()
  const unmuteConversationMutation = useUnmuteWhatsappConversationMutation()
  const markConversationReadMutation = useMarkWhatsappConversationReadMutation()
  const markConversationUnreadMutation = useMarkWhatsappConversationUnreadMutation()
  const addReactionMutation = useAddWhatsappReactionMutation()
  const removeReactionMutation = useRemoveWhatsappReactionMutation()

  const messages = computed(() => messagesQuery.data.value ?? [])
  const selectedMessages = computed(() =>
    searchText.value.trim() ? searchQuery.data.value?.items ?? [] : messages.value
  )
  const pinnedMessages = computed(() => pinnedMessagesQuery.data.value?.items ?? [])
  const members = computed(() => membersQuery.data.value ?? [])
  const mediaItems = computed(() => mediaQuery.data.value?.items ?? [])
  const selectedMediaItem = computed(
    () => mediaItems.value.find((item) => item.attachment_id === selectedMediaAttachmentId.value) ?? null
  )
  const mediaPreviewQuery = useAttachmentPreviewQuery(
    () => selectedMediaAttachmentId.value,
    () => Boolean(selectedMediaAttachmentId.value)
  )
  const mediaPreview = computed(() => mediaPreviewQuery.data.value ?? null)
  const mediaPreviewError = computed(() => {
    const error = mediaPreviewQuery.error.value
    if (!error) return ''
    return error instanceof Error ? error.message : t('Media preview failed')
  })
  const isStatusFeedConversation = computed(() =>
    selectedConversation.value?.provider_chat_id === 'status-feed' ||
    selectedConversation.value?.chat_kind === 'status_feed' ||
    Boolean(selectedConversation.value?.metadata?.is_status_feed)
  )
  const isSearchActive = computed(() => searchText.value.trim().length >= 2)
  const emptyStateMessage = computed(() => {
    if (browserMode.value === 'media') {
      return isSearchActive.value ? t('No WhatsApp media search matches.') : t('No projected media yet.')
    }
    return searchText.value.trim() ? t('No WhatsApp search matches.') : t('Select a WhatsApp conversation.')
  })
  const editingMessage = computed(
    () => messages.value.find((message) => message.message_id === editingMessageId.value) ?? null
  )
  const forwardingMessage = computed(
    () => messages.value.find((message) => message.message_id === forwardingMessageId.value) ?? null
  )
  const forwardTargetConversations = computed(() => {
    const query = forwardTargetFilter.value.trim().toLowerCase()
    return conversations.value.filter((conversation) => {
      if (!query) return true
      return (
        conversation.title.toLowerCase().includes(query) ||
        conversation.provider_chat_id.toLowerCase().includes(query) ||
        conversation.account_id.toLowerCase().includes(query)
      )
    })
  })
  const isBusy = computed(() =>
    sendMutation.isPending.value ||
    replyMutation.isPending.value ||
    forwardMutation.isPending.value ||
    editMutation.isPending.value ||
    deleteMutation.isPending.value ||
    pinConversationMutation.isPending.value ||
    unpinConversationMutation.isPending.value ||
    archiveConversationMutation.isPending.value ||
    unarchiveConversationMutation.isPending.value ||
    muteConversationMutation.isPending.value ||
    unmuteConversationMutation.isPending.value ||
    markConversationReadMutation.isPending.value ||
    markConversationUnreadMutation.isPending.value ||
    addReactionMutation.isPending.value ||
    removeReactionMutation.isPending.value
  )
  const isConversationPinned = computed(() =>
    Boolean(selectedConversation.value?.metadata?.is_pinned ?? selectedConversation.value?.metadata?.pinned)
  )
  const isConversationArchived = computed(() =>
    Boolean(selectedConversation.value?.metadata?.is_archived ?? selectedConversation.value?.metadata?.archived)
  )
  const isConversationMuted = computed(() =>
    Boolean(selectedConversation.value?.metadata?.is_muted ?? selectedConversation.value?.metadata?.muted)
  )
  const isConversationUnread = computed(() =>
    Boolean(
      selectedConversation.value?.metadata?.is_unread ??
      ((conversationMetaNumber('unread_count') ?? 0) > 0)
    )
  )

  watch(
    conversations,
    (items) => {
      if (!items.some((conversation) => conversation.provider_chat_id === selectedConversationId.value)) {
        selectedConversationId.value = items[0]?.provider_chat_id ?? ''
      }
    },
    { immediate: true }
  )

  watch(mediaItems, (items) => {
    if (!items.length) {
      selectedMediaAttachmentId.value = null
      return
    }
    if (
      selectedMediaAttachmentId.value &&
      items.some((item) => item.attachment_id === selectedMediaAttachmentId.value)
    ) {
      return
    }
    selectedMediaAttachmentId.value = firstPreviewableMediaAttachmentId(items)
  })

  function selectConversation(providerChatId: string) {
    selectedConversationId.value = providerChatId
  }

  function conversationMetaNumber(key: string): number | null {
    const value = selectedConversation.value?.metadata?.[key]
    return typeof value === 'number' ? value : null
  }

  function selectMediaItem(item: WhatsappWebMediaItem): void {
    selectedMediaAttachmentId.value = mediaAttachmentId(item)
  }

  function clearMediaPreview(): void {
    selectedMediaAttachmentId.value = null
  }

  function requireProviderChatId(message: WhatsAppPanelMessage): string | null {
    const providerChatId = message.provider_chat_id ?? message.conversation_id
    if (providerChatId) return providerChatId
    actionError.value = t('Message is missing provider conversation metadata')
    return null
  }

  function requireProviderMessageId(message: WhatsAppPanelMessage): string | null {
    const providerMessageId = message.provider_message_id ?? message.provider_record_id
    if (providerMessageId) return providerMessageId
    actionError.value = t('Message is missing provider message metadata')
    return null
  }

  async function sendMessage() {
    const conversation = selectedConversation.value
    const text = draftText.value.trim()
    if (!conversation || !text || isBusy.value) return
    actionMessage.value = ''
    actionError.value = ''
    try {
      const result = await sendMutation.mutateAsync({
        account_id: conversation.account_id,
        provider_chat_id: conversation.provider_chat_id,
        text,
      })
      draftText.value = ''
      actionMessage.value = `WhatsApp message ${result.status}`
    } catch (error) {
      actionError.value = error instanceof Error ? error.message : String(error)
    }
  }

  async function replyToMessage(message: WhatsAppPanelMessage) {
    const text = draftText.value.trim()
    if (!text || isBusy.value) return
    actionMessage.value = ''
    actionError.value = ''
    try {
      const result = await replyMutation.mutateAsync({
        message_id: message.message_id,
        text,
      })
      draftText.value = ''
      actionMessage.value = `WhatsApp reply ${result.status}`
    } catch (error) {
      actionError.value = error instanceof Error ? error.message : String(error)
    }
  }

  function beginEditMessage(message: WhatsAppPanelMessage) {
    if (isBusy.value) return
    editingMessageId.value = message.message_id
    editDraftText.value = message.text ?? message.body_text_preview ?? ''
    actionMessage.value = ''
    actionError.value = ''
  }

  function cancelEditMessage() {
    editingMessageId.value = null
    editDraftText.value = ''
  }

  function beginForwardMessage(message: WhatsAppPanelMessage) {
    if (isBusy.value) return
    forwardingMessageId.value = message.message_id
    forwardTargetFilter.value = ''
    forwardTargetConversationId.value =
      conversations.value.find(
        (conversation) => conversation.provider_chat_id !== message.provider_chat_id
      )?.provider_chat_id ?? ''
  }

  function cancelForwardMessage() {
    forwardingMessageId.value = null
    forwardTargetConversationId.value = ''
    forwardTargetFilter.value = ''
  }

  async function confirmForwardMessage() {
    const message = forwardingMessage.value
    const targetConversationId = forwardTargetConversationId.value.trim()
    if (!message || !targetConversationId || isBusy.value) return
    actionMessage.value = ''
    actionError.value = ''
    try {
      const result = await forwardMutation.mutateAsync({
        message_id: message.message_id,
        provider_chat_id: targetConversationId,
      })
      actionMessage.value = `WhatsApp forward ${result.status}`
      cancelForwardMessage()
    } catch (error) {
      actionError.value = error instanceof Error ? error.message : String(error)
    }
  }

  async function confirmEditMessage() {
    const message = editingMessage.value
    const nextText = editDraftText.value.trim()
    if (!message || !nextText || isBusy.value) return
    const providerChatId = requireProviderChatId(message)
    const providerMessageId = requireProviderMessageId(message)
    if (!providerChatId || !providerMessageId) return
    actionMessage.value = ''
    actionError.value = ''
    try {
      await editMutation.mutateAsync({
        message_id: message.message_id,
        account_id: message.account_id,
        provider_chat_id: providerChatId,
        provider_message_id: providerMessageId,
        new_text: nextText,
      })
      actionMessage.value = t('Message edited')
      cancelEditMessage()
    } catch (error) {
      actionError.value = error instanceof Error ? error.message : String(error)
    }
  }

  async function deleteMessage(message: WhatsAppPanelMessage) {
    if (isBusy.value) return
    const providerChatId = requireProviderChatId(message)
    const providerMessageId = requireProviderMessageId(message)
    if (!providerChatId || !providerMessageId) return
    actionMessage.value = ''
    actionError.value = ''
    try {
      await deleteMutation.mutateAsync({
        message_id: message.message_id,
        account_id: message.account_id,
        provider_chat_id: providerChatId,
        provider_message_id: providerMessageId,
        reason_class: 'deleted_by_owner',
        actor_class: 'owner',
        is_provider_delete: false,
      })
      actionMessage.value = t('Message deleted locally')
    } catch (error) {
      actionError.value = error instanceof Error ? error.message : String(error)
    }
  }

  async function toggleConversationPin() {
    const conversationId = selectedConversation.value?.conversation_id
    if (!conversationId || isBusy.value) return
    actionMessage.value = ''
    actionError.value = ''
    try {
      const result = isConversationPinned.value
        ? await unpinConversationMutation.mutateAsync({ conversation_id: conversationId })
        : await pinConversationMutation.mutateAsync({ conversation_id: conversationId })
      actionMessage.value = result.active ? t('Conversation pinned') : t('Conversation unpinned')
    } catch (error) {
      actionError.value = error instanceof Error ? error.message : String(error)
    }
  }

  async function toggleConversationArchive() {
    const conversationId = selectedConversation.value?.conversation_id
    if (!conversationId || isBusy.value) return
    actionMessage.value = ''
    actionError.value = ''
    try {
      const result = isConversationArchived.value
        ? await unarchiveConversationMutation.mutateAsync({ conversation_id: conversationId })
        : await archiveConversationMutation.mutateAsync({ conversation_id: conversationId })
      actionMessage.value = result.active ? t('Conversation archived') : t('Conversation unarchived')
    } catch (error) {
      actionError.value = error instanceof Error ? error.message : String(error)
    }
  }

  async function toggleConversationMute() {
    const conversationId = selectedConversation.value?.conversation_id
    if (!conversationId || isBusy.value) return
    actionMessage.value = ''
    actionError.value = ''
    try {
      const result = isConversationMuted.value
        ? await unmuteConversationMutation.mutateAsync({ conversation_id: conversationId })
        : await muteConversationMutation.mutateAsync({ conversation_id: conversationId })
      actionMessage.value = result.active ? t('Conversation muted') : t('Conversation unmuted')
    } catch (error) {
      actionError.value = error instanceof Error ? error.message : String(error)
    }
  }

  async function toggleConversationUnread() {
    const conversationId = selectedConversation.value?.conversation_id
    if (!conversationId || isBusy.value) return
    actionMessage.value = ''
    actionError.value = ''
    try {
      const result = isConversationUnread.value
        ? await markConversationReadMutation.mutateAsync({ conversation_id: conversationId })
        : await markConversationUnreadMutation.mutateAsync({ conversation_id: conversationId })
      actionMessage.value = result.active ? t('Conversation marked unread') : t('Conversation marked read')
    } catch (error) {
      actionError.value = error instanceof Error ? error.message : String(error)
    }
  }

  async function addReaction(message: WhatsAppPanelMessage, reactionEmoji: string) {
    const providerChatId = requireProviderChatId(message)
    const providerMessageId = requireProviderMessageId(message)
    if (!providerChatId || !providerMessageId || isBusy.value) return
    actionMessage.value = ''
    actionError.value = ''
    try {
      const result = await addReactionMutation.mutateAsync({
        message_id: message.message_id,
        request: {
          account_id: message.account_id,
          provider_chat_id: providerChatId,
          provider_message_id: providerMessageId,
          reaction_emoji: reactionEmoji,
          sender_id: message.sender,
          sender_display_name: message.sender_display_name,
        },
      })
      actionMessage.value = `WhatsApp reaction ${result.status}`
    } catch (error) {
      actionError.value = error instanceof Error ? error.message : String(error)
    }
  }

  async function removeReaction(message: WhatsAppPanelMessage, reactionEmoji: string) {
    const providerChatId = requireProviderChatId(message)
    const providerMessageId = requireProviderMessageId(message)
    if (!providerChatId || !providerMessageId || isBusy.value) return
    actionMessage.value = ''
    actionError.value = ''
    try {
      const result = await removeReactionMutation.mutateAsync({
        message_id: message.message_id,
        request: {
          account_id: message.account_id,
          provider_chat_id: providerChatId,
          provider_message_id: providerMessageId,
          reaction_emoji: reactionEmoji,
          sender_id: message.sender,
          sender_display_name: message.sender_display_name,
        },
      })
      actionMessage.value = `WhatsApp reaction ${result.status}`
    } catch (error) {
      actionError.value = error instanceof Error ? error.message : String(error)
    }
  }

  function setMessageElementRef(messageId: string, element: unknown) {
    if (element instanceof HTMLElement) {
      messageElementMap.set(messageId, element)
      return
    }
    messageElementMap.delete(messageId)
  }

  async function jumpToMessage(messageId: string) {
    searchText.value = ''
    await nextTick()
    await nextTick()
    const element = messageElementMap.get(messageId)
    if (!element) {
      actionError.value = t('Message is not loaded in the current timeline')
      return
    }
    element.scrollIntoView({ behavior: 'smooth', block: 'center' })
    element.classList.add('message-bubble--flash')
    window.setTimeout(() => {
      element.classList.remove('message-bubble--flash')
    }, 1400)
  }

  function setBrowserMode(mode: 'timeline' | 'media') {
    browserMode.value = mode
  }

  async function refreshPanel() {
    await Promise.all([
      conversationsQuery.refetch(),
      conversationDetailQuery.refetch(),
      messagesQuery.refetch(),
      pinnedMessagesQuery.refetch(),
      membersQuery.refetch(),
      mediaQuery.refetch(),
    ])
  }

  return {
    actionError,
    actionMessage,
    addReaction,
    beginEditMessage,
    beginForwardMessage,
    browserMode,
    cancelEditMessage,
    cancelForwardMessage,
    clearMediaPreview,
    confirmEditMessage,
    confirmForwardMessage,
    conversations,
    conversationsQuery,
    conversationMetaNumber,
    deleteMessage,
    draftText,
    editDraftText,
    editingMessage,
    emptyStateMessage,
    forwardTargetConversationId,
    forwardTargetConversations,
    forwardTargetFilter,
    forwardingMessage,
    isBusy,
    isConversationArchived,
    isConversationMuted,
    isConversationPinned,
    isConversationUnread,
    isStatusFeedConversation,
    jumpToMessage,
    mediaItems,
    mediaKindFilter,
    mediaPreview,
    mediaPreviewError,
    mediaPreviewQuery,
    mediaQuery,
    members,
    membersQuery,
    messagesQuery,
    pinnedMessages,
    pinnedMessagesQuery,
    refreshPanel,
    removeReaction,
    replyToMessage,
    searchText,
    selectConversation,
    selectedConversation,
    selectedConversationId,
    selectedMediaItem,
    selectedMessages,
    sendMessage,
    setBrowserMode,
    setMessageElementRef,
    selectMediaItem,
    t,
    toggleConversationArchive,
    toggleConversationMute,
    toggleConversationPin,
    toggleConversationUnread,
  }
}

export type WhatsappCommunicationsPanelSurface = ReturnType<typeof useWhatsappCommunicationsPanelSurface>
