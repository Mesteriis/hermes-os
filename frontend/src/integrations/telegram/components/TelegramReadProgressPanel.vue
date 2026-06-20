<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '../../../platform/i18n'
import { useTelegramCommandsQuery } from '../queries/useTelegramLifecycleQuery'
import {
  telegramCommandAuditState,
  telegramCommandSubject,
} from '../stores/telegramCommandAudit'
import {
  telegramChatMentionCountValue,
  telegramChatUnreadCount,
} from '../stores/telegram'
import {
  telegramLatestReadableProviderMessageId,
  telegramThreadReadProgress,
} from '../stores/telegramReadProgress'
import type { TelegramChat, TelegramMessage } from '../types/telegram'

const { t } = useI18n()

const props = defineProps<{
  selectedChat: TelegramChat | null
  selectedMessages: TelegramMessage[]
}>()

const commandsQuery = useTelegramCommandsQuery(
  computed(() => props.selectedChat?.account_id ?? null),
  20,
  computed(() => Boolean(props.selectedChat)),
  {
    providerChatId: computed(() => props.selectedChat?.provider_chat_id ?? null),
    commandKinds: computed(() => ['mark_read', 'mark_unread']),
  }
)

const readProgress = computed(() => {
  if (!props.selectedChat) return null
  return telegramThreadReadProgress(props.selectedChat, props.selectedMessages)
})

const latestVisibleProviderMessageId = computed(() => {
  if (!props.selectedChat) return null
  return telegramLatestReadableProviderMessageId(props.selectedChat, props.selectedMessages)
})

const readCommands = computed(() => {
  if (!props.selectedChat) return []
  return (commandsQuery.data.value ?? [])
    .filter(
      (command) =>
        command.provider_chat_id === props.selectedChat?.provider_chat_id &&
        (command.command_kind === 'mark_read' || command.command_kind === 'mark_unread')
    )
    .slice(0, 4)
})

const readStatusLabel = computed(() => {
  if (!props.selectedChat) return t('No chat selected')
  if (telegramChatUnreadCount(props.selectedChat) === 0) return t('Projection is fully read')
  if (readProgress.value?.hasUnreadAfterBoundary) return t('Unread messages remain after the provider boundary')
  if (readProgress.value?.lastReadProviderMessageId) return t('Unread state exists outside the loaded thread window')
  return t('Provider read boundary has not been observed yet')
})

function formatBoundaryLabel(messageId: string | null): string {
  if (!messageId) return t('Outside the loaded thread window')
  const message = props.selectedMessages.find((item) => item.message_id === messageId)
  if (!message) return t('Outside the loaded thread window')
  const sender = message.sender_display_name ?? message.sender
  return `${sender} · ${message.provider_message_id}`
}
</script>

<template>
  <article class="telegram-rail-card telegram-read-progress">
    <header class="telegram-read-progress__header">
      <div>
        <h3>{{ t('Read Progress') }}</h3>
        <p>{{ t('Provider-observed boundary and recent read commands for this chat.') }}</p>
      </div>
      <span class="telegram-read-progress__summary">
        {{ selectedChat ? telegramChatUnreadCount(selectedChat) : 0 }} {{ t('unread') }}
      </span>
    </header>

    <dl v-if="selectedChat" class="telegram-read-progress__facts">
      <div>
        <dt>{{ t('Provider boundary') }}</dt>
        <dd>{{ readProgress?.lastReadProviderMessageId ?? '—' }}</dd>
      </div>
      <div>
        <dt>{{ t('Loaded boundary') }}</dt>
        <dd>{{ formatBoundaryLabel(readProgress?.lastReadMessageId ?? null) }}</dd>
      </div>
      <div>
        <dt>{{ t('Latest visible message') }}</dt>
        <dd>{{ latestVisibleProviderMessageId ?? '—' }}</dd>
      </div>
      <div>
        <dt>{{ t('Mentions') }}</dt>
        <dd>{{ telegramChatMentionCountValue(selectedChat) }}</dd>
      </div>
    </dl>
    <p v-else class="telegram-read-progress__empty">
      {{ t('Select a Telegram chat to inspect read progress.') }}
    </p>

    <p class="telegram-read-progress__status">
      {{ readStatusLabel }}
    </p>

    <div v-if="selectedChat && readCommands.length > 0" class="telegram-read-progress__commands">
      <article
        v-for="command in readCommands"
        :key="command.command_id"
        class="telegram-read-progress__command"
      >
        <div class="telegram-read-progress__command-row">
          <strong>{{ telegramCommandSubject(command) }}</strong>
          <span
            class="telegram-read-progress__state"
            :class="`telegram-read-progress__state--${telegramCommandAuditState(command).tone}`"
          >
            {{ t(telegramCommandAuditState(command).label) }}
          </span>
        </div>
        <small>{{ telegramCommandAuditState(command).detail }}</small>
      </article>
    </div>
  </article>
</template>

<style scoped>
.telegram-read-progress,
.telegram-read-progress__commands {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.telegram-read-progress__header,
.telegram-read-progress__command-row {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;
}

.telegram-read-progress__header h3,
.telegram-read-progress__header p,
.telegram-read-progress__status,
.telegram-read-progress__empty,
.telegram-read-progress__command small {
  margin: 0;
}

.telegram-read-progress__header p,
.telegram-read-progress__status,
.telegram-read-progress__empty,
.telegram-read-progress__facts dt,
.telegram-read-progress__facts dd,
.telegram-read-progress__command small {
  font-size: 11px;
  color: var(--color-text-secondary, #777);
}

.telegram-read-progress__summary {
  font-size: 11px;
  font-weight: 600;
  color: var(--color-text, #333);
}

.telegram-read-progress__facts {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px 12px;
  margin: 0;
}

.telegram-read-progress__facts div {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.telegram-read-progress__facts dd {
  margin: 0;
  color: var(--color-text, #333);
  word-break: break-word;
}

.telegram-read-progress__status {
  padding: 8px 10px;
  border-radius: 8px;
  background: color-mix(in srgb, var(--color-surface-hover, #f5f7fb) 78%, transparent);
}

.telegram-read-progress__command {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding-top: 8px;
  border-top: 1px solid var(--color-border, #e0e0e0);
}

.telegram-read-progress__command:first-child {
  padding-top: 0;
  border-top: none;
}

.telegram-read-progress__command strong {
  font-size: 12px;
  color: var(--color-text, #333);
}

.telegram-read-progress__state {
  padding: 2px 7px;
  border-radius: 999px;
  border: 1px solid var(--color-border, #e0e0e0);
  background: var(--color-bg, #fafafa);
  color: var(--color-text-secondary, #777);
  font-size: 10px;
  white-space: nowrap;
}

.telegram-read-progress__state--progress {
  color: var(--color-primary, #1565c0);
  border-color: color-mix(in srgb, var(--color-primary, #1565c0) 45%, transparent);
  background: color-mix(in srgb, var(--color-primary, #1565c0) 10%, transparent);
}

.telegram-read-progress__state--success {
  color: var(--color-success, #137333);
  border-color: color-mix(in srgb, var(--color-success, #137333) 40%, transparent);
  background: color-mix(in srgb, var(--color-success, #137333) 10%, transparent);
}

.telegram-read-progress__state--warning,
.telegram-read-progress__state--danger {
  color: var(--color-danger, #b42318);
  border-color: color-mix(in srgb, var(--color-danger, #b42318) 45%, transparent);
  background: color-mix(in srgb, var(--color-danger, #b42318) 10%, transparent);
}
</style>
