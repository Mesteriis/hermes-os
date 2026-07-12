<script setup lang="ts">
import { computed, ref } from 'vue'
import { Icon } from '@/shared/ui'
import type { TelegramChat } from '@/shared/communications/types/telegram'
import type { TelegramConversationRuntimeAction, TelegramConversationRuntimeActionRunner } from '@/shared/communications/types/telegramRuntimeActions'
import { telegramProviderFolders } from '../../queries/telegramWorkspacePresentation'
import {
  useCloseTelegramTopicMutation,
  useCreateTelegramTopicMutation,
  useTelegramChatDetailQuery,
  useTelegramChatFoldersQuery,
  useTelegramChatMembersQuery,
  useTelegramAttachmentPreviewQuery,
  useTelegramMediaSearchQuery,
  useTelegramMessageSearchQuery,
  useTelegramPinnedMessagesQuery,
  useTelegramTopicsQuery,
  useTelegramTopicSearchQuery,
  useUpdateTelegramChatHistoryPolicyMutation,
  useUpdateTelegramChatReadReceiptPolicyMutation,
  useUpdateTelegramChatUnreadCounterPolicyMutation,
} from '../../queries/telegramBusinessQueries'

const props = defineProps<{
  telegramChat: TelegramChat
  runtimeActionRunner?: TelegramConversationRuntimeActionRunner
}>()

const chatId = computed(() => props.telegramChat.telegram_chat_id)
const accountId = computed(() => props.telegramChat.account_id)
const providerChatId = computed(() => props.telegramChat.provider_chat_id)
const detailQuery = useTelegramChatDetailQuery(chatId)
const foldersQuery = useTelegramChatFoldersQuery(accountId)
const membersQuery = useTelegramChatMembersQuery(chatId, 30)
const mediaQuery = useTelegramMediaSearchQuery({ accountId, providerChatId, limit: () => 30 })
const pinnedMessagesQuery = useTelegramPinnedMessagesQuery({ telegramChatId: chatId, limit: () => 30 })
const topicsQuery = useTelegramTopicsQuery(chatId, 30)
const topicSearch = ref('')
const messageSearch = ref('')
const topicSearchQuery = useTelegramTopicSearchQuery(chatId, topicSearch, 30)
const messageSearchQuery = useTelegramMessageSearchQuery({ q: messageSearch, accountId, providerChatId, limit: () => 30 })
const createTopicMutation = useCreateTelegramTopicMutation()
const closeTopicMutation = useCloseTelegramTopicMutation()
const historyPolicyMutation = useUpdateTelegramChatHistoryPolicyMutation()
const readReceiptPolicyMutation = useUpdateTelegramChatReadReceiptPolicyMutation()
const unreadCounterPolicyMutation = useUpdateTelegramChatUnreadCounterPolicyMutation()
const newTopicTitle = ref('')
const providerFolderId = ref<number | null>(null)
const actionError = ref('')
const actionStatus = ref('')
const selectedMediaAttachmentId = ref<string | null>(null)
const mediaPreviewQuery = useTelegramAttachmentPreviewQuery(selectedMediaAttachmentId)
const providerFolders = computed(() =>
  telegramProviderFolders(foldersQuery.data.value ?? [])
)

async function runAction(
  action: TelegramConversationRuntimeAction,
  extras: Partial<Parameters<TelegramConversationRuntimeActionRunner>[0]> = {}
): Promise<void> {
  if (!props.runtimeActionRunner) return
  actionError.value = ''
  actionStatus.value = ''
  try {
    await props.runtimeActionRunner({
      action,
      accountId: props.telegramChat.account_id,
      providerChatId: props.telegramChat.provider_chat_id,
      telegramChatId: props.telegramChat.telegram_chat_id,
      ...extras,
    })
    actionStatus.value = `Telegram ${action.replaceAll('_', ' ')} command queued.`
  } catch (error) {
    actionError.value = error instanceof Error ? error.message : 'Telegram provider command failed.'
  }
}

function selectedFolderId(): number | undefined {
  return providerFolderId.value ?? undefined
}

function topicCommandId(): string {
  return crypto.randomUUID()
}

async function createTopic(): Promise<void> {
  const title = newTopicTitle.value.trim()
  if (!title) return
  await createTopicMutation.mutateAsync({
    conversationId: props.telegramChat.telegram_chat_id,
    request: {
      command_id: topicCommandId(),
      account_id: props.telegramChat.account_id,
      provider_chat_id: props.telegramChat.provider_chat_id,
      title,
    },
  })
  newTopicTitle.value = ''
}

async function toggleTopic(topicId: string, isClosed: boolean): Promise<void> {
  await closeTopicMutation.mutateAsync({
    topicId,
    request: {
      command_id: topicCommandId(),
      account_id: props.telegramChat.account_id,
      provider_chat_id: props.telegramChat.provider_chat_id,
      is_closed: !isClosed,
    },
  })
}

async function toggleFullHistory(enabled: boolean): Promise<void> {
  actionError.value = ''
  try {
    await historyPolicyMutation.mutateAsync({
      telegramChatId: props.telegramChat.telegram_chat_id,
      accountId: props.telegramChat.account_id,
      providerChatId: props.telegramChat.provider_chat_id,
      enabled,
    })
    if (enabled) {
      await runAction('sync_full')
    } else {
      actionStatus.value = 'Full Telegram history is disabled for this chat.'
    }
  } catch (error) {
    actionError.value = error instanceof Error ? error.message : 'Telegram history policy update failed.'
  }
}

function handleFullHistoryChange(event: Event): void {
  void toggleFullHistory((event.target as HTMLInputElement).checked)
}

async function toggleReadReceiptReports(enabled: boolean): Promise<void> {
  actionError.value = ''
  try {
    await readReceiptPolicyMutation.mutateAsync({
      telegramChatId: props.telegramChat.telegram_chat_id,
      accountId: props.telegramChat.account_id,
      providerChatId: props.telegramChat.provider_chat_id,
      enabled,
    })
    actionStatus.value = enabled
      ? 'Telegram read reports are enabled for this chat.'
      : 'Telegram read reports are disabled for this chat. Hermes will keep read state locally.'
  } catch (error) {
    actionError.value = error instanceof Error ? error.message : 'Telegram read receipt policy update failed.'
  }
}

function handleReadReceiptReportsChange(event: Event): void {
  void toggleReadReceiptReports((event.target as HTMLInputElement).checked)
}

async function toggleUnreadCounter(hidden: boolean): Promise<void> {
  actionError.value = ''
  try {
    await unreadCounterPolicyMutation.mutateAsync({
      telegramChatId: props.telegramChat.telegram_chat_id,
      accountId: props.telegramChat.account_id,
      providerChatId: props.telegramChat.provider_chat_id,
      hideUnreadCounter: hidden,
    })
    actionStatus.value = hidden
      ? 'Unread counter is hidden for this chat.'
      : 'Unread counter is visible for this chat.'
  } catch (error) {
    actionError.value = error instanceof Error ? error.message : 'Telegram unread counter policy update failed.'
  }
}

function handleUnreadCounterChange(event: Event): void {
  void toggleUnreadCounter((event.target as HTMLInputElement).checked)
}
</script>

<template>
  <section class="telegram-conversation-inspector" aria-label="Telegram conversation resources">
    <header class="telegram-conversation-inspector__header">
      <Icon icon="tabler:brand-telegram" size="1rem" />
      <div>
        <strong>Telegram conversation</strong>
        <span>{{ detailQuery.data.value?.username ?? telegramChat.provider_chat_id }}</span>
      </div>
    </header>

	<section class="telegram-conversation-inspector__section">
	  <h3>History</h3>
	  <label v-if="telegramChat.chat_kind !== 'private'">
	    <input
	      type="checkbox"
	      :checked="telegramChat.metadata.full_history_sync_enabled === true"
	      :disabled="historyPolicyMutation.isPending.value"
	      @change="handleFullHistoryChange"
	    />
	    Load full history for this group or forum
	  </label>
	  <p v-else>Private chat history is loaded in full automatically.</p>
	</section>

    <section class="telegram-conversation-inspector__section">
      <h3>Privacy</h3>
      <label>
        <input
          type="checkbox"
          :checked="telegramChat.metadata.read_receipt_reports_enabled !== false"
          :disabled="readReceiptPolicyMutation.isPending.value"
          @change="handleReadReceiptReportsChange"
        />
        Send read reports to Telegram for this chat
      </label>
      <p>When disabled, Hermes marks messages read only in its local state.</p>
      <p>Telegram does not expose a delivery-receipt suppression control to TDLib.</p>
    </section>

    <section class="telegram-conversation-inspector__section">
      <h3>List appearance</h3>
      <label>
        <input
          type="checkbox"
          :checked="telegramChat.metadata.hide_unread_counter === true"
          :disabled="unreadCounterPolicyMutation.isPending.value"
          @change="handleUnreadCounterChange"
        />
        Hide unread counter for this chat
      </label>
      <p>The unread state is retained locally; only its list badge is hidden.</p>
    </section>

	<section class="telegram-conversation-inspector__section">
      <h3>Participants</h3>
      <p v-if="!membersQuery.data.value?.length">No projected participants.</p>
      <p v-for="member in membersQuery.data.value ?? []" :key="member.provider_member_id">
        {{ member.sender_display_name ?? member.sender_id }} · {{ member.role ?? member.status ?? 'member' }}
      </p>
      <div class="telegram-conversation-inspector__actions">
        <button type="button" @click="runAction('sync_members')">Sync members</button>
        <button type="button" @click="runAction('join')">Join</button>
        <button type="button" @click="runAction('leave')">Leave</button>
      </div>
    </section>

    <section class="telegram-conversation-inspector__section">
      <h3>Topics</h3>
      <input v-model="newTopicTitle" type="text" placeholder="New topic title" />
      <button type="button" @click="createTopic">Create topic</button>
      <input v-model="topicSearch" type="search" placeholder="Search topics" />
      <p v-if="!topicsQuery.data.value?.items.length">No projected topics.</p>
      <p v-for="topic in topicsQuery.data.value?.items ?? []" :key="topic.topic_id">
        {{ topic.title }}
        <button type="button" @click="toggleTopic(topic.topic_id, topic.is_closed)">{{ topic.is_closed ? 'Reopen' : 'Close' }}</button>
      </p>
      <p v-for="topic in topicSearchQuery.data.value?.items ?? []" :key="`search-${topic.topic_id}`">
        Search: {{ topic.title }}
      </p>
    </section>

    <section class="telegram-conversation-inspector__section">
      <h3>Pinned messages</h3>
      <p v-if="!pinnedMessagesQuery.data.value?.items.length">No pinned messages.</p>
      <p v-for="message in pinnedMessagesQuery.data.value?.items ?? []" :key="message.message_id">
        {{ message.sender_display_name ?? message.sender }} · {{ message.text }}
      </p>
    </section>

    <section class="telegram-conversation-inspector__section">
      <h3>Media</h3>
      <p v-if="!mediaQuery.data.value?.items.length">No projected media.</p>
      <p v-for="media in mediaQuery.data.value?.items ?? []" :key="media.attachment_id ?? media.provider_attachment_id ?? media.provider_message_id">
        {{ media.file_name }} · {{ media.download_state }}
        <button v-if="media.attachment_id" type="button" @click="selectedMediaAttachmentId = media.attachment_id">Preview</button>
        <button v-if="media.tdlib_file_id != null" type="button" @click="runAction('download_media', { providerMessageId: media.provider_message_id, tdlibFileId: media.tdlib_file_id, providerAttachmentId: media.provider_attachment_id ?? undefined, filename: media.file_name, contentType: media.mime_type ?? undefined })">Download</button>
      </p>
      <article v-if="mediaPreviewQuery.data.value" class="telegram-conversation-inspector__preview">
        <strong>{{ mediaPreviewQuery.data.value.filename ?? 'Attachment preview' }}</strong>
        <img v-if="mediaPreviewQuery.data.value.preview_kind === 'image' && mediaPreviewQuery.data.value.data_url" :src="mediaPreviewQuery.data.value.data_url" :alt="mediaPreviewQuery.data.value.filename ?? 'Telegram attachment preview'" />
        <audio v-else-if="mediaPreviewQuery.data.value.preview_kind === 'audio' && mediaPreviewQuery.data.value.data_url" :src="mediaPreviewQuery.data.value.data_url" controls />
        <video v-else-if="mediaPreviewQuery.data.value.preview_kind === 'video' && mediaPreviewQuery.data.value.data_url" :src="mediaPreviewQuery.data.value.data_url" controls />
        <p v-else>{{ mediaPreviewQuery.data.value.text }}</p>
      </article>
    </section>

    <section class="telegram-conversation-inspector__section">
      <h3>Message search</h3>
      <input v-model="messageSearch" type="search" placeholder="Search projected messages" />
      <p v-for="message in messageSearchQuery.data.value?.items ?? []" :key="`search-${message.message_id}`">
        {{ message.sender_display_name ?? message.sender }} · {{ message.text }}
      </p>
    </section>

    <section class="telegram-conversation-inspector__section">
      <h3>Telegram folders</h3>
      <select v-model="providerFolderId" :disabled="!providerFolders.length">
        <option :value="null" disabled>Select a Telegram folder</option>
        <option v-for="folder in providerFolders" :key="folder.id" :value="folder.provider_folder_id">
          {{ folder.label }}
        </option>
      </select>
      <div class="telegram-conversation-inspector__actions">
        <button type="button" :disabled="selectedFolderId() == null" @click="runAction('folder_add', { providerFolderId: selectedFolderId() })">Add</button>
        <button type="button" :disabled="selectedFolderId() == null" @click="runAction('folder_remove', { providerFolderId: selectedFolderId() })">Remove</button>
        <button type="button" :disabled="selectedFolderId() == null" @click="runAction('folder_reassign', { providerFolderIds: selectedFolderId() == null ? [] : [selectedFolderId() as number] })">Replace</button>
      </div>
    </section>

    <p v-if="actionStatus" class="telegram-conversation-inspector__success" role="status">{{ actionStatus }}</p>
    <p v-if="actionError" class="telegram-conversation-inspector__error" role="alert">{{ actionError }}</p>
  </section>
</template>
