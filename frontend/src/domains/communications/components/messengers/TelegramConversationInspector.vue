<script setup lang="ts">
import { computed } from 'vue'
import { Icon } from '@/shared/ui'
import type { TelegramChat } from '@/shared/communications/types/telegram'
import type { TelegramConversationRuntimeActionRunner } from '@/shared/communications/types/telegramRuntimeActions'
import { useTelegramConversationInspectorController } from '../../queries/useTelegramConversationInspectorController'

const props = defineProps<{
  telegramChat: TelegramChat
  runtimeActionRunner?: TelegramConversationRuntimeActionRunner
}>()

const controller = useTelegramConversationInspectorController(
  computed(() => props.telegramChat),
  props.runtimeActionRunner,
)

const {
  detailQuery,
  foldersQuery,
  membersQuery,
  mediaQuery,
  pinnedMessagesQuery,
  topicsQuery,
  topicSearch,
  messageSearch,
  topicSearchQuery,
  messageSearchQuery,
  createTopicMutation,
  closeTopicMutation,
  historyPolicyMutation,
  readReceiptPolicyMutation,
  unreadCounterPolicyMutation,
  newTopicTitle,
  providerFolderId,
  actionError,
  actionStatus,
  mediaPreviewQuery,
  providerFolders,
  selectedFolderId,
  handleFullHistoryChange,
  handleReadReceiptReportsChange,
  handleUnreadCounterChange,
  handleSetPreviewAttachment,
  handleDownloadMedia,
  handleSyncMembers,
  handleJoinConversation,
  handleLeaveConversation,
  handleCreateTopic,
  handleToggleTopic,
  handleAddFolder,
  handleRemoveFolder,
  handleReplaceFolder,
} = controller
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
        <button type="button" @click="handleSyncMembers">Sync members</button>
        <button type="button" @click="handleJoinConversation">Join</button>
        <button type="button" @click="handleLeaveConversation">Leave</button>
      </div>
    </section>

    <section class="telegram-conversation-inspector__section">
      <h3>Topics</h3>
      <input v-model="newTopicTitle" type="text" placeholder="New topic title" />
      <button type="button" @click="handleCreateTopic">Create topic</button>
      <input v-model="topicSearch" type="search" placeholder="Search topics" />
      <p v-if="!topicsQuery.data.value?.items.length">No projected topics.</p>
      <p v-for="topic in topicsQuery.data.value?.items ?? []" :key="topic.topic_id">
        {{ topic.title }}
        <button type="button" @click="handleToggleTopic(topic.topic_id, topic.is_closed)">{{ topic.is_closed ? 'Reopen' : 'Close' }}</button>
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
        <button v-if="media.attachment_id" type="button" @click="handleSetPreviewAttachment(media.attachment_id)">Preview</button>
        <button v-if="media.tdlib_file_id != null" type="button" @click="handleDownloadMedia(media)">Download</button>
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
        <button type="button" :disabled="selectedFolderId() == null" @click="handleAddFolder">Add</button>
        <button type="button" :disabled="selectedFolderId() == null" @click="handleRemoveFolder">Remove</button>
        <button type="button" :disabled="selectedFolderId() == null" @click="handleReplaceFolder">Replace</button>
      </div>
    </section>

    <p v-if="actionStatus" class="telegram-conversation-inspector__success" role="status">{{ actionStatus }}</p>
    <p v-if="actionError" class="telegram-conversation-inspector__error" role="alert">{{ actionError }}</p>
  </section>
</template>
