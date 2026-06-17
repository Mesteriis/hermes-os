<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type { TelegramProviderWriteCommand } from '../types/telegram'
import { useTelegramCommandRetryMutation, useTelegramCommandsQuery } from '../queries/useTelegramLifecycleQuery'
import {
  telegramCommandAuditState,
  telegramCommandRetrySummary,
  telegramCommandSubject
} from '../stores/telegramCommandAudit'

const { t } = useI18n()

const props = defineProps<{
  selectedAccountId: string | null
  selectedProviderChatId: string | null
}>()

const searchQuery = ref('')
const currentChatOnly = ref(true)
const commandsQuery = useTelegramCommandsQuery(
  computed(() => props.selectedAccountId),
  20,
  true,
  {
    providerChatId: computed(() =>
      currentChatOnly.value ? props.selectedProviderChatId : null
    ),
  }
)
const retryMutation = useTelegramCommandRetryMutation()
const commands = computed(() => commandsQuery.data.value ?? [])
const filteredCommands = computed(() => {
  const query = searchQuery.value.trim().toLowerCase()
  return commands.value.filter((command) => {
    if (
      currentChatOnly.value &&
      props.selectedProviderChatId &&
      command.provider_chat_id !== props.selectedProviderChatId
    ) {
      return false
    }
    if (!query) return true
    return [
      command.command_kind,
      command.status,
      command.provider_chat_id,
      command.provider_message_id ?? '',
      command.capability_state,
      command.action_class,
    ]
      .join(' ')
      .toLowerCase()
      .includes(query)
  })
})

function formatDate(value: string | null | undefined): string {
  if (!value) return '—'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return '—'
  return new Intl.DateTimeFormat('en', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  }).format(date)
}

function commandTitle(command: TelegramProviderWriteCommand): string {
  return [command.command_kind, command.status].join(' · ')
}

function commandStateClass(command: TelegramProviderWriteCommand): string {
  return `telegram-command-audit__state--${telegramCommandAuditState(command).tone}`
}

function canRetry(command: TelegramProviderWriteCommand): boolean {
  return command.status === 'dead_letter' || command.status === 'failed'
}
</script>

<template>
  <article class="telegram-rail-card telegram-command-audit">
    <header class="telegram-command-audit__header">
      <div>
        <h3>{{ t('Recent Commands') }}</h3>
        <p>{{ t('Durable Telegram command rows for the selected account.') }}</p>
      </div>
      <label class="telegram-command-audit__toggle">
        <input v-model="currentChatOnly" type="checkbox" />
        <span>{{ t('Current chat only') }}</span>
      </label>
    </header>

    <label v-if="commands.length > 0" class="telegram-command-audit__search">
      <Icon icon="tabler:search" width="15" height="15" />
      <input
        v-model="searchQuery"
        type="search"
        :placeholder="t('Search command rows')"
      />
    </label>

    <div v-if="!selectedAccountId" class="telegram-call-placeholder">
      {{ t('Select a Telegram account to inspect command audit rows.') }}
    </div>
    <div v-else-if="commandsQuery.isLoading.value" class="telegram-call-placeholder">
      {{ t('Loading Telegram command audit...') }}
    </div>
    <div v-else-if="commands.length === 0" class="telegram-call-placeholder">
      {{ t('No Telegram command rows projected for this account yet.') }}
    </div>
    <div v-else-if="filteredCommands.length === 0" class="telegram-call-placeholder">
      {{ t('No Telegram command rows match this filter.') }}
    </div>
    <div v-else class="telegram-command-audit__list">
      <article
        v-for="command in filteredCommands"
        :key="command.command_id"
        class="telegram-command-audit__item"
        :class="{ 'telegram-command-audit__item--dead-letter': telegramCommandAuditState(command).is_dead_letter }"
      >
        <div class="telegram-command-audit__row">
          <strong>{{ commandTitle(command) }}</strong>
          <small>{{ formatDate(command.happened_at) }}</small>
        </div>
        <p>{{ telegramCommandSubject(command) }}</p>
        <div class="telegram-command-audit__badges">
          <span class="telegram-command-audit__state" :class="commandStateClass(command)">
            {{ t(telegramCommandAuditState(command).label) }}
          </span>
          <span>{{ telegramCommandRetrySummary(command) }}</span>
        </div>
        <small>
          {{ command.capability_state }} · {{ command.action_class }} · {{ command.confirmation_decision }}
        </small>
        <small>{{ t('Reconciliation') }}: {{ command.reconciliation_status }}</small>
        <small>{{ telegramCommandAuditState(command).detail }}</small>
        <button
          v-if="canRetry(command)"
          type="button"
          class="telegram-command-audit__retry"
          :disabled="retryMutation.isPending.value"
          @click="retryMutation.mutate(command.command_id)"
        >
          <Icon icon="tabler:refresh" width="14" height="14" />
          {{ t('Retry command') }}
        </button>
      </article>
    </div>
  </article>
</template>

<style scoped>
.telegram-command-audit,
.telegram-command-audit__list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.telegram-command-audit__header,
.telegram-command-audit__row,
.telegram-command-audit__toggle,
.telegram-command-audit__search,
.telegram-command-audit__badges {
  display: flex;
  align-items: center;
  gap: 8px;
}

.telegram-command-audit__header {
  justify-content: space-between;
  align-items: flex-start;
}

.telegram-command-audit__header h3,
.telegram-command-audit__header p,
.telegram-command-audit__item p,
.telegram-command-audit__item small {
  margin: 0;
}

.telegram-command-audit__header p,
.telegram-command-audit__item small,
.telegram-command-audit__toggle,
.telegram-command-audit__search {
  font-size: 11px;
  color: var(--color-text-secondary, #777);
}

.telegram-command-audit__search {
  padding: 8px 10px;
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 999px;
  background: var(--color-surface, #fff);
}

.telegram-command-audit__search input {
  flex: 1;
  border: none;
  background: transparent;
  font: inherit;
  color: var(--color-text, #333);
  outline: none;
}

.telegram-command-audit__item {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 10px 12px;
  border-radius: 10px;
  border: 1px solid var(--color-border, #e0e0e0);
  background: var(--color-surface, #fff);
}

.telegram-command-audit__item--dead-letter {
  border-color: color-mix(in srgb, var(--color-danger, #b42318) 55%, var(--color-border, #e0e0e0));
}

.telegram-command-audit__row {
  justify-content: space-between;
}

.telegram-command-audit__item strong,
.telegram-command-audit__item p {
  font-size: 12px;
  color: var(--color-text, #333);
  word-break: break-word;
}

.telegram-command-audit__badges {
  flex-wrap: wrap;
  font-size: 11px;
  color: var(--color-text-secondary, #777);
}

.telegram-command-audit__state {
  padding: 2px 7px;
  border-radius: 999px;
  border: 1px solid var(--color-border, #e0e0e0);
  background: var(--color-bg, #fafafa);
  color: var(--color-text-secondary, #777);
}

.telegram-command-audit__state--danger {
  border-color: color-mix(in srgb, var(--color-danger, #b42318) 55%, transparent);
  background: color-mix(in srgb, var(--color-danger, #b42318) 12%, transparent);
  color: var(--color-danger, #b42318);
}

.telegram-command-audit__state--warning {
  border-color: color-mix(in srgb, var(--color-warning, #b54708) 55%, transparent);
  background: color-mix(in srgb, var(--color-warning, #b54708) 12%, transparent);
  color: var(--color-warning, #b54708);
}

.telegram-command-audit__state--progress {
  border-color: color-mix(in srgb, var(--color-accent, #2563eb) 45%, transparent);
  background: color-mix(in srgb, var(--color-accent, #2563eb) 10%, transparent);
  color: var(--color-accent, #2563eb);
}

.telegram-command-audit__state--success {
  border-color: color-mix(in srgb, var(--color-success, #027a48) 45%, transparent);
  background: color-mix(in srgb, var(--color-success, #027a48) 10%, transparent);
  color: var(--color-success, #027a48);
}

.telegram-command-audit__retry {
  display: inline-flex;
  align-items: center;
  align-self: flex-start;
  gap: 5px;
  border: 1px solid var(--color-border, #d6dce3);
  border-radius: 999px;
  background: var(--color-surface, #fff);
  color: var(--color-text, #333);
  padding: 5px 9px;
  cursor: pointer;
}

.telegram-command-audit__retry:disabled {
  cursor: not-allowed;
  opacity: 0.55;
}
</style>
