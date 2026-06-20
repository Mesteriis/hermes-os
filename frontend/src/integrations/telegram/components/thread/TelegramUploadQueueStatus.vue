<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '../../../../platform/i18n'
import Icon from '../../../../shared/ui/Icon.vue'
import { useTelegramCommandRetryMutation, useTelegramCommandsQuery } from '../../queries/useTelegramLifecycleQuery'
import { telegramCommandAuditState, telegramCommandRetrySummary } from '../../stores/telegramCommandAudit'
import { telegramUploadCommandTitle, telegramUploadQueueCommands } from '../../stores/telegramUploadQueue'
import type { TelegramProviderWriteCommand } from '../../types/telegram'

const { t } = useI18n()

const props = defineProps<{
  selectedAccountId: string | null
  selectedProviderChatId: string | null
}>()

const commandsQuery = useTelegramCommandsQuery(
  computed(() => props.selectedAccountId),
  20,
  computed(() => Boolean(props.selectedAccountId && props.selectedProviderChatId)),
  {
    providerChatId: computed(() => props.selectedProviderChatId),
    commandKinds: computed(() => ['send_media']),
  }
)
const retryMutation = useTelegramCommandRetryMutation()

const uploadCommands = computed(() =>
  telegramUploadQueueCommands(
    commandsQuery.data.value ?? [],
    props.selectedProviderChatId,
    3
  )
)

function canRetry(command: TelegramProviderWriteCommand): boolean {
  return command.status === 'dead_letter' || command.status === 'failed'
}

function formatDate(value: string | null | undefined): string {
  if (!value) return '—'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return '—'
  return new Intl.DateTimeFormat('en', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  }).format(date)
}
</script>

<template>
  <div v-if="uploadCommands.length > 0" class="telegram-upload-queue" aria-live="polite">
    <article
      v-for="command in uploadCommands"
      :key="command.command_id"
      class="telegram-upload-queue__item"
      :class="{ 'telegram-upload-queue__item--dead-letter': telegramCommandAuditState(command).is_dead_letter }"
    >
      <div class="telegram-upload-queue__row">
        <div class="telegram-upload-queue__title">
          <Icon icon="tabler:paperclip" width="14" height="14" />
          <strong>{{ telegramUploadCommandTitle(command) }}</strong>
        </div>
        <span class="telegram-upload-queue__state">
          {{ t(telegramCommandAuditState(command).label) }}
        </span>
      </div>
      <small>{{ telegramCommandAuditState(command).detail }}</small>
      <small>{{ telegramCommandRetrySummary(command) }}</small>
      <small v-if="command.next_attempt_at">
        {{ t('Next attempt') }}: {{ formatDate(command.next_attempt_at) }}
      </small>
      <button
        v-if="canRetry(command)"
        type="button"
        class="telegram-upload-queue__retry"
        :disabled="retryMutation.isPending.value"
        @click="retryMutation.mutate(command.command_id)"
      >
        <Icon icon="tabler:refresh" width="13" height="13" />
        {{ t('Retry upload') }}
      </button>
    </article>
  </div>
</template>

<style scoped>
.telegram-upload-queue {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 8px 12px 0;
  background: var(--color-surface, #fff);
}

.telegram-upload-queue__item {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 8px 10px;
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 8px;
  background: var(--color-bg, #fafafa);
}

.telegram-upload-queue__item--dead-letter {
  border-color: color-mix(in srgb, var(--color-danger, #b42318) 55%, var(--color-border, #e0e0e0));
}

.telegram-upload-queue__row,
.telegram-upload-queue__title {
  display: flex;
  align-items: center;
  gap: 6px;
}

.telegram-upload-queue__row {
  justify-content: space-between;
}

.telegram-upload-queue__title strong,
.telegram-upload-queue__state,
.telegram-upload-queue__item small,
.telegram-upload-queue__retry {
  font-size: 11px;
}

.telegram-upload-queue__title strong {
  color: var(--color-text, #333);
  word-break: break-word;
}

.telegram-upload-queue__state,
.telegram-upload-queue__item small {
  color: var(--color-text-secondary, #777);
}

.telegram-upload-queue__retry {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  width: fit-content;
  margin-top: 2px;
  padding: 0;
  border: none;
  background: transparent;
  color: var(--color-accent, #2563eb);
  cursor: pointer;
}

.telegram-upload-queue__retry:disabled {
  opacity: 0.6;
  cursor: default;
}
</style>
