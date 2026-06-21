<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from '../../../../../platform/i18n'
import Icon from '../../../../../shared/ui/Icon.vue'
import type { TelegramChat, TelegramMessage } from '../../../../../shared/communications/types/telegram'
import {
  useDeleteTelegramMessageMutation,
  useEditTelegramMessageMutation,
  usePinTelegramMessageMutation,
  useReplyTelegramMessageMutation,
  useSendTelegramMessageMutation,
  useTelegramChatsQuery,
  useTelegramMessageSearchQuery,
  useTelegramMessagesQuery,
} from '../../../queries/telegramBusinessQueries'

const { t } = useI18n()
const selectedConversationId = ref('')
const draftText = ref('')
const searchText = ref('')
const actionMessage = ref('')
const actionError = ref('')

const chatsQuery = useTelegramChatsQuery(undefined, 500)
const chats = computed(() => chatsQuery.data.value ?? [])
const selectedChat = computed<TelegramChat | null>(
  () => chats.value.find((chat) => chat.provider_chat_id === selectedConversationId.value) ?? chats.value[0] ?? null
)
const messagesQuery = useTelegramMessagesQuery(
  () => selectedChat.value?.account_id ?? null,
  () => selectedChat.value?.provider_chat_id ?? null,
  100
)
const searchQuery = useTelegramMessageSearchQuery({
  q: searchText,
  accountId: () => selectedChat.value?.account_id ?? null,
  providerChatId: () => selectedChat.value?.provider_chat_id ?? null,
  limit: 50,
})
const sendMutation = useSendTelegramMessageMutation()
const replyMutation = useReplyTelegramMessageMutation()
const editMutation = useEditTelegramMessageMutation()
const deleteMutation = useDeleteTelegramMessageMutation()
const pinMutation = usePinTelegramMessageMutation()
const messages = computed(() => messagesQuery.data.value ?? [])
const visibleMessages = computed(() =>
  searchText.value.trim()
    ? (searchQuery.data.value?.items ?? [])
    : messages.value
)
const isBusy = computed(() =>
  sendMutation.isPending.value ||
  replyMutation.isPending.value ||
  editMutation.isPending.value ||
  deleteMutation.isPending.value ||
  pinMutation.isPending.value
)

watch(
  chats,
  (items) => {
    if (!items.length) {
      selectedConversationId.value = ''
      return
    }
    if (!items.some((item) => item.provider_chat_id === selectedConversationId.value)) {
      selectedConversationId.value = items[0]?.provider_chat_id ?? ''
    }
  },
  { immediate: true }
)

function messageTime(message: TelegramMessage): string {
  const value = message.occurred_at ?? message.projected_at
  if (!value) return ''
  const date = new Date(value)
  return Number.isNaN(date.getTime())
    ? ''
    : new Intl.DateTimeFormat('en', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' }).format(date)
}

function messagePreview(chat: TelegramChat): string {
  const latest = messages.value
    .filter((message) => message.provider_chat_id === chat.provider_chat_id)
    .at(-1)
  return latest?.text || chat.sync_state
}

function requireProviderChatId(message: TelegramMessage): string | null {
  if (message.provider_chat_id) return message.provider_chat_id
  actionError.value = t('Message is missing provider conversation metadata')
  return null
}

async function sendMessage() {
  const chat = selectedChat.value
  const text = draftText.value.trim()
  if (!chat || !text || isBusy.value) return
  actionMessage.value = ''
  actionError.value = ''
  try {
    const result = await sendMutation.mutateAsync({
      account_id: chat.account_id,
      provider_chat_id: chat.provider_chat_id,
      text,
    })
    draftText.value = ''
    actionMessage.value = `Telegram message ${result.status}`
  } catch (error) {
    actionError.value = error instanceof Error ? error.message : String(error)
  }
}

async function replyToMessage(message: TelegramMessage) {
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
    actionMessage.value = `Telegram reply ${result.status}`
  } catch (error) {
    actionError.value = error instanceof Error ? error.message : String(error)
  }
}

async function editMessage(message: TelegramMessage) {
  const nextText = window.prompt(t('Edit message'), message.text)
  if (nextText === null || !nextText.trim() || isBusy.value) return
  const providerChatId = requireProviderChatId(message)
  if (!providerChatId) return
  try {
    await editMutation.mutateAsync({
      message_id: message.message_id,
      account_id: message.account_id,
      provider_chat_id: providerChatId,
      provider_message_id: message.provider_message_id,
      new_text: nextText.trim(),
    })
    actionMessage.value = t('Message edited')
  } catch (error) {
    actionError.value = error instanceof Error ? error.message : String(error)
  }
}

async function deleteMessage(message: TelegramMessage) {
  if (isBusy.value) return
  const providerChatId = requireProviderChatId(message)
  if (!providerChatId) return
  try {
    await deleteMutation.mutateAsync({
      message_id: message.message_id,
      account_id: message.account_id,
      provider_chat_id: providerChatId,
      provider_message_id: message.provider_message_id,
      reason_class: 'deleted_by_owner',
      actor_class: 'owner',
      is_provider_delete: false,
    })
    actionMessage.value = t('Message deleted locally')
  } catch (error) {
    actionError.value = error instanceof Error ? error.message : String(error)
  }
}

async function togglePin(message: TelegramMessage) {
  if (isBusy.value) return
  const providerChatId = requireProviderChatId(message)
  if (!providerChatId) return
  const isPinned = !(message.metadata?.is_pinned === true || message.metadata?.pinned === true)
  try {
    await pinMutation.mutateAsync({
      message_id: message.message_id,
      account_id: message.account_id,
      provider_chat_id: providerChatId,
      provider_message_id: message.provider_message_id,
      is_pinned: isPinned,
    })
    actionMessage.value = isPinned ? t('Message pinned') : t('Message unpinned')
  } catch (error) {
    actionError.value = error instanceof Error ? error.message : String(error)
  }
}
</script>

<template>
  <section class="telegram-communications-panel communications-page">
    <header class="view-header">
      <div class="view-title-with-icon">
        <span class="hero-mark small">
          <Icon icon="tabler:brand-telegram" width="28" height="28" />
        </span>
        <div>
          <h1>{{ t('Telegram') }}</h1>
          <p>{{ t('Projected Communication conversations and messages') }}</p>
        </div>
      </div>
      <label class="provider-search">
        <Icon icon="tabler:search" width="16" height="16" />
        <input v-model="searchText" type="search" :placeholder="t('Search messages')" />
      </label>
    </header>

    <p v-if="actionMessage" class="setup-state success">{{ actionMessage }}</p>
    <p v-if="actionError" class="inline-error">{{ actionError }}</p>

    <div class="three-pane communications-grid telegram-grid">
      <section class="panel conversation-list">
        <header class="provider-panel-header">
          <h2>{{ t('Conversations') }}</h2>
          <button type="button" :disabled="chatsQuery.isFetching.value" @click="chatsQuery.refetch()">
            <Icon icon="tabler:refresh" width="16" height="16" />
          </button>
        </header>
        <div class="provider-list-scroll">
          <div v-if="chatsQuery.isLoading.value" class="empty-panel">{{ t('Loading Telegram conversations...') }}</div>
          <button
            v-for="chat in chats"
            :key="chat.provider_chat_id"
            type="button"
            class="provider-row"
            :class="{ active: selectedConversationId === chat.provider_chat_id }"
            @click="selectedConversationId = chat.provider_chat_id"
          >
            <strong>{{ chat.title }}</strong>
            <span>{{ messagePreview(chat) }}</span>
          </button>
        </div>
      </section>

      <section class="panel chat-pane">
        <header class="provider-thread-header">
          <div>
            <h2>{{ selectedChat?.title ?? t('No conversation selected') }}</h2>
            <p>{{ selectedChat?.account_id ?? '' }}</p>
          </div>
        </header>
        <div class="message-scroll">
          <article v-for="message in visibleMessages" :key="message.message_id" class="message-bubble">
            <header>
              <strong>{{ message.sender_display_name ?? message.sender }}</strong>
              <time>{{ messageTime(message) }}</time>
            </header>
            <p>{{ message.text }}</p>
            <footer>
              <button type="button" :disabled="isBusy || !draftText.trim()" @click="replyToMessage(message)">
                <Icon icon="tabler:message-reply" width="14" height="14" />{{ t('Reply') }}
              </button>
              <button type="button" :disabled="isBusy" @click="editMessage(message)">
                <Icon icon="tabler:edit" width="14" height="14" />{{ t('Edit') }}
              </button>
              <button type="button" :disabled="isBusy" @click="togglePin(message)">
                <Icon icon="tabler:pin" width="14" height="14" />{{ t('Pin') }}
              </button>
              <button type="button" :disabled="isBusy" @click="deleteMessage(message)">
                <Icon icon="tabler:trash" width="14" height="14" />{{ t('Delete') }}
              </button>
            </footer>
          </article>
          <div v-if="!visibleMessages.length" class="empty-panel">{{ t('No projected Telegram messages yet.') }}</div>
        </div>
        <form class="provider-inline-form" @submit.prevent="sendMessage">
          <input v-model="draftText" type="text" :placeholder="t('Write a message')" autocomplete="off" />
          <button type="submit" :disabled="isBusy || !selectedChat || !draftText.trim()">
            <Icon icon="tabler:send" width="16" height="16" />{{ t('Send') }}
          </button>
        </form>
      </section>
    </div>
  </section>
</template>

<style scoped>
.telegram-communications-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}
.view-header,
.view-title-with-icon,
.provider-search,
.provider-panel-header,
.provider-thread-header,
.message-bubble header,
.message-bubble footer,
.provider-inline-form {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}
.view-header {
  justify-content: space-between;
  padding: 0.75rem 1rem;
  border-bottom: 1px solid var(--hh-border, #d9e2ec);
}
.provider-search,
.provider-inline-form {
  border: 1px solid var(--hh-border, #d9e2ec);
  border-radius: 8px;
  padding: 0.4rem 0.6rem;
  background: var(--hh-bg-primary, #fff);
}
.provider-search input,
.provider-inline-form input {
  border: 0;
  outline: 0;
  min-width: 220px;
  background: transparent;
  color: inherit;
}
.provider-panel-header,
.provider-thread-header {
  justify-content: space-between;
  padding: 0.75rem;
  border-bottom: 1px solid var(--hh-border, #d9e2ec);
}
.provider-list-scroll,
.message-scroll {
  flex: 1;
  overflow: auto;
  min-height: 0;
}
.provider-row {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  width: 100%;
  padding: 0.7rem 0.85rem;
  border: 0;
  border-bottom: 1px solid var(--hh-border, #eef2f6);
  background: transparent;
  color: inherit;
  text-align: left;
  cursor: pointer;
}
.provider-row.active,
.provider-row:hover {
  background: var(--hh-bg-muted, #f5f8fb);
}
.provider-row span,
.provider-thread-header p,
.message-bubble time {
  color: var(--hh-text-muted, #667085);
  font-size: 0.78rem;
}
.message-bubble {
  margin: 0.75rem;
  padding: 0.75rem;
  border: 1px solid var(--hh-border, #d9e2ec);
  border-radius: 8px;
  background: var(--hh-bg-primary, #fff);
}
.message-bubble header,
.message-bubble footer {
  justify-content: space-between;
}
.message-bubble footer {
  justify-content: flex-start;
}
.message-bubble button,
.provider-panel-header button,
.provider-inline-form button {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  border: 1px solid var(--hh-border, #d9e2ec);
  border-radius: 6px;
  background: var(--hh-bg-primary, #fff);
  color: inherit;
  padding: 0.35rem 0.55rem;
  cursor: pointer;
}
.provider-inline-form {
  margin: 0.75rem;
}
.provider-inline-form input {
  flex: 1;
}
.setup-state,
.inline-error,
.empty-panel {
  padding: 0.75rem;
}
.success {
  color: #206a3a;
}
.inline-error {
  color: #b42318;
}
</style>
