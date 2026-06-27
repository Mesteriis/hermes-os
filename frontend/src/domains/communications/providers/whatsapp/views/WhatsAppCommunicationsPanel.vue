<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import { useI18n } from '../../../../../platform/i18n'
import Icon from '../../../../../shared/ui/Icon.vue'
import { useAttachmentPreviewQuery } from '../../../queries/useCommunicationsQuery'
import WhatsAppCommunicationsChatPane from './WhatsAppCommunicationsChatPane.vue'
import WhatsAppCommunicationsDetailPane from './WhatsAppCommunicationsDetailPane.vue'
import type {
  WhatsappWebMediaItem,
} from '../../../../../shared/communications/types/whatsapp'
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
} from '../../../queries/whatsappBusinessQueries'
import {
  firstPreviewableMediaAttachmentId,
  mediaAttachmentId,
  type WhatsAppPanelMessage,
} from './WhatsAppCommunicationsPanel.helpers'

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
</script>

<template>
  <section class="whatsapp-communications-panel communications-page">
    <header class="view-header">
      <div class="view-title-with-icon">
        <span class="hero-mark small">
          <Icon icon="tabler:brand-whatsapp" width="28" height="28" />
        </span>
        <div>
          <h1>{{ t('WhatsApp') }}</h1>
          <p>{{ t('Projected Communication conversations, messages and members') }}</p>
        </div>
      </div>
      <div class="header-actions">
        <div class="thread-actions">
          <button
            type="button"
            :class="{ active: browserMode === 'timeline' }"
            @click="setBrowserMode('timeline')"
          >
            <Icon icon="tabler:messages" width="14" height="14" />{{ t('Timeline') }}
          </button>
          <button
            type="button"
            :class="{ active: browserMode === 'media' }"
            @click="setBrowserMode('media')"
          >
            <Icon icon="tabler:photo" width="14" height="14" />{{ t('Media') }}
          </button>
        </div>
        <label class="provider-search">
          <Icon icon="tabler:search" width="16" height="16" />
          <input v-model="searchText" type="search" :placeholder="t('Search messages and media')" />
        </label>
        <label v-if="browserMode === 'media'" class="runtime-field compact media-filter">
          <span>{{ t('Kind') }}</span>
          <select v-model="mediaKindFilter">
            <option value="all">{{ t('All media') }}</option>
            <option value="image">{{ t('Images') }}</option>
            <option value="video">{{ t('Videos') }}</option>
            <option value="audio">{{ t('Audio') }}</option>
            <option value="document">{{ t('Documents') }}</option>
          </select>
        </label>
        <button
          type="button"
          class="primary-button"
          :disabled="conversationsQuery.isFetching.value || messagesQuery.isFetching.value"
          @click="() => {
            void conversationsQuery.refetch()
            void conversationDetailQuery.refetch()
            void messagesQuery.refetch()
            void pinnedMessagesQuery.refetch()
            void membersQuery.refetch()
            void mediaQuery.refetch()
          }"
        >
          <Icon icon="tabler:refresh" width="16" height="16" />{{ t('Refresh') }}
        </button>
      </div>
    </header>

    <p v-if="actionMessage" class="setup-state success">{{ actionMessage }}</p>
    <p v-if="actionError" class="inline-error">{{ actionError }}</p>

    <div class="three-pane communications-grid whatsapp-grid">
      <section class="panel conversation-list">
        <header class="provider-panel-header">
          <h2>{{ t('Conversations') }}</h2>
        </header>
        <div class="provider-list-scroll">
          <button
            v-for="conversation in conversations"
            :key="conversation.conversation_id ?? conversation.provider_chat_id"
            type="button"
            class="provider-row"
            :class="{ active: selectedConversationId === conversation.provider_chat_id }"
            @click="selectConversation(conversation.provider_chat_id)"
          >
            <strong>{{ conversation.title }}</strong>
            <span>{{ conversation.metadata?.provider_label ?? conversation.chat_kind ?? t('conversation') }}</span>
          </button>
          <div v-if="conversationsQuery.isLoading.value" class="empty-panel">
            {{ t('Loading WhatsApp conversations...') }}
          </div>
          <div v-if="!conversations.length" class="empty-panel">{{ t('No projected WhatsApp conversations yet.') }}</div>
        </div>
      </section>

      <WhatsAppCommunicationsChatPane
        v-model:draft-text="draftText"
        :selected-conversation="selectedConversation"
        :browser-mode="browserMode"
        :selected-messages="selectedMessages"
        :media-items="mediaItems"
        :empty-state-message="emptyStateMessage"
        :is-busy="isBusy"
        :is-conversation-unread="isConversationUnread"
        :is-conversation-muted="isConversationMuted"
        :is-conversation-archived="isConversationArchived"
        :is-conversation-pinned="isConversationPinned"
        @toggle-unread="toggleConversationUnread"
        @toggle-mute="toggleConversationMute"
        @toggle-archive="toggleConversationArchive"
        @toggle-pin="toggleConversationPin"
        @send-message="sendMessage"
        @set-message-ref="setMessageElementRef"
        @reply="replyToMessage"
        @forward="beginForwardMessage"
        @edit="beginEditMessage"
        @delete="deleteMessage"
        @select-media="selectMediaItem"
        @jump-to-message="jumpToMessage"
        @add-reaction="addReaction"
        @remove-reaction="removeReaction"
      />

      <WhatsAppCommunicationsDetailPane
        v-model:edit-draft-text="editDraftText"
        v-model:forward-target-filter="forwardTargetFilter"
        v-model:forward-target-conversation-id="forwardTargetConversationId"
        :selected-conversation="selectedConversation"
        :is-status-feed-conversation="isStatusFeedConversation"
        :is-conversation-unread="isConversationUnread"
        :is-conversation-archived="isConversationArchived"
        :is-conversation-muted="isConversationMuted"
        :is-conversation-pinned="isConversationPinned"
        :participant-count="conversationMetaNumber('participant_count') ?? members.length"
        :editing-message="editingMessage"
        :forwarding-message="forwardingMessage"
        :forward-target-conversations="forwardTargetConversations"
        :members="members"
        :pinned-messages="pinnedMessages"
        :media-items="mediaItems"
        :selected-media-item="selectedMediaItem"
        :media-preview="mediaPreview"
        :media-preview-error="mediaPreviewError"
        :is-media-preview-fetching="mediaPreviewQuery.isFetching.value"
        :is-busy="isBusy"
        @confirm-edit="confirmEditMessage"
        @cancel-edit="cancelEditMessage"
        @confirm-forward="confirmForwardMessage"
        @cancel-forward="cancelForwardMessage"
        @select-media="selectMediaItem"
        @clear-media-preview="clearMediaPreview"
        @jump-to-message="jumpToMessage"
      />
    </div>
  </section>
</template>

<style scoped src="./WhatsAppCommunicationsPanel.css"></style>
