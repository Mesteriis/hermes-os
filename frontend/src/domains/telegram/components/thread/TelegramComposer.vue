<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from '../../../../platform/i18n'
import Icon from '../../../../shared/ui/Icon.vue'
import TelegramSendDryRunPanel from './TelegramSendDryRunPanel.vue'
import type { TelegramMessage } from '../../types/telegram'

const { t } = useI18n()

const props = defineProps<{
  text: string
  isTelegramActionSubmitting: boolean
  selectedAccountId: string | null
  selectedProviderChatId: string | null
  replyTo?: TelegramMessage | null
}>()

const emit = defineEmits<{
  'update:text': [value: string]
  sendMessage: []
  syncHistory: []
  clearReply: []
}>()

const isEmojiTrayOpen = ref(false)
const isSendMenuOpen = ref(false)

function appendEmoji(value: string) {
  emit('update:text', `${props.text}${value}`)
  isEmojiTrayOpen.value = false
}

function submitManualSend() {
  isSendMenuOpen.value = false
  emit('sendMessage')
}
</script>

<template>
  <div class="telegram-composer-wrapper">
  <div v-if="replyTo" class="telegram-reply-banner">
    <Icon icon="tabler:corner-up-left" width="14" height="14" />
    <span class="telegram-reply-banner__sender">{{ replyTo.sender_display_name ?? replyTo.sender }}</span>
    <span class="telegram-reply-banner__text">{{ replyTo.text?.slice(0, 80) }}</span>
    <button type="button" :title="t('Cancel reply')" @click="emit('clearReply')">
      <Icon icon="tabler:x" width="14" height="14" />
    </button>
  </div>
  <form class="telegram-compose-bar" @submit.prevent="submitManualSend">
    <button type="button" disabled :title="t('Attachment upload is not available in this slice')">
      <Icon icon="tabler:paperclip" width="18" height="18" />
    </button>
    <textarea
      :value="text"
      rows="1"
      :placeholder="t('Write a message...')"
      autocomplete="off"
      @input="emit('update:text', ($event.target as HTMLTextAreaElement).value)"
    ></textarea>
    <div class="telegram-compose-menu">
      <button type="button" :title="t('Emoji')" @click="isEmojiTrayOpen = !isEmojiTrayOpen">
        <Icon icon="tabler:mood-smile" width="18" height="18" />
      </button>
      <div v-if="isEmojiTrayOpen" class="telegram-emoji-popover">
        <button
          v-for="emoji in ['👍', '🔥', '🎉', '✅', '🙏']"
          :key="emoji"
          type="button"
          @click="appendEmoji(emoji)"
        >
          {{ emoji }}
        </button>
      </div>
    </div>
    <button type="button" disabled :title="t('Voice messages require media runtime')">
      <Icon icon="tabler:microphone" width="18" height="18" />
    </button>
    <button
      type="submit"
      class="send"
      :disabled="isTelegramActionSubmitting || !text.trim()"
      :title="t('Send')"
    >
      <Icon icon="tabler:send" width="18" height="18" />
    </button>
    <div class="telegram-compose-menu">
      <button
        type="button"
        class="send-more"
        :title="t('More')"
        @click="isSendMenuOpen = !isSendMenuOpen"
      >
        <Icon icon="tabler:chevron-down" width="17" height="17" />
      </button>
      <div v-if="isSendMenuOpen" class="command-popover telegram-send-popover">
        <button
          type="button"
          :disabled="isTelegramActionSubmitting || !text.trim()"
          @click="submitManualSend"
        >
          <Icon icon="tabler:send" width="15" height="15" />{{ t('Send now') }}
        </button>
        <button
          type="button"
          :disabled="isTelegramActionSubmitting"
          @click="isSendMenuOpen = false; emit('syncHistory')"
        >
          <Icon icon="tabler:history" width="15" height="15" />{{ t('Sync History') }}
        </button>
        <TelegramSendDryRunPanel
          :accountId="selectedAccountId"
          :providerChatId="selectedProviderChatId"
          :text="text"
        />
      </div>
    </div>
  </form>
  </div>
</template>

<style scoped>
.telegram-compose-bar {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 8px 12px;
  border-top: 1px solid var(--color-border, #e0e0e0);
  background: var(--color-surface, #fff);
}
.telegram-compose-bar textarea {
  flex: 1;
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 8px;
  padding: 8px 12px;
  font-size: 12px;
  font-family: inherit;
  resize: none;
  outline: none;
  min-height: 36px;
  color: var(--color-text, #333);
  background: var(--color-bg, #f9f9f9);
}
.telegram-compose-bar button {
  border: none;
  background: transparent;
  cursor: pointer;
  padding: 6px;
  color: var(--color-text-secondary, #777);
  border-radius: 4px;
}
.telegram-compose-bar button:hover:not(:disabled) {
  background: var(--color-bg, #f5f5f5);
}
.telegram-compose-bar button.send {
  background: var(--color-primary, #0066cc);
  color: #fff;
  border-radius: 8px;
  padding: 8px;
}
.telegram-compose-bar button.send:disabled {
  opacity: 0.5;
}
.telegram-compose-menu {
  position: relative;
}
.telegram-emoji-popover,
.command-popover {
  position: absolute;
  bottom: 100%;
  right: 0;
  background: var(--color-surface, #fff);
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0,0,0,0.1);
  margin-bottom: 4px;
  z-index: 10;
}
.telegram-emoji-popover {
  display: flex;
  gap: 2px;
  padding: 6px;
}
.telegram-emoji-popover button {
  padding: 4px 8px;
  font-size: 18px;
}
.command-popover {
  min-width: 140px;
  padding: 4px;
}
.telegram-send-popover {
  min-width: 280px;
}
.command-popover button {
  display: flex;
  align-items: center;
  gap: 6px;
  width: 100%;
  padding: 6px 10px;
  font-size: 11px;
  color: var(--color-text, #333);
}
.command-popover button:hover,
.telegram-emoji-popover button:hover {
  background: var(--color-bg, #f5f5f5);
}
.telegram-composer-wrapper {
  display: flex;
  flex-direction: column;
}
.telegram-reply-banner {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 14px;
  background: var(--color-primary-subtle, #e3f2fd);
  border-top: 1px solid var(--color-border, #e0e0e0);
  font-size: 11px;
  color: var(--color-text-secondary, #555);
}
.telegram-reply-banner__sender {
  font-weight: 600;
  color: var(--color-primary, #0066cc);
  flex-shrink: 0;
}
.telegram-reply-banner__text {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.telegram-reply-banner button {
  border: none;
  background: transparent;
  cursor: pointer;
  padding: 2px;
  color: var(--color-text-secondary, #999);
  border-radius: 4px;
  flex-shrink: 0;
}
</style>
