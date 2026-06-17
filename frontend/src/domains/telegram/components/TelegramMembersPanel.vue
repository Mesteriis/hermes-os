<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import Button from '../../../shared/ui/Button.vue'
import { useTelegramCommandRetryMutation, useTelegramCommandsQuery } from '../queries/useTelegramLifecycleQuery'
import { useJoinTelegramChatMutation, useLeaveTelegramChatMutation } from '../queries/useTelegramParticipantLifecycleQuery'
import { useSyncTelegramChatMembersMutation, useTelegramChatMembersQuery } from '../queries/useTelegramMembersQuery'
import { telegramCommandAuditState } from '../stores/telegramCommandAudit'
import {
  telegramParticipantLifecycleCommands,
  telegramParticipantLifecycleTitle,
} from '../stores/telegramParticipantLifecycle'
import type { TelegramCapabilitiesResponse, TelegramChatMember, TelegramOperationCapability } from '../types/telegram'
import type { TelegramProviderWriteCommand } from '../types/telegram'

const { t } = useI18n()

const props = defineProps<{
  telegramChatId: string | null
  accountId: string | null
  providerChatId: string | null
  capabilities: TelegramCapabilitiesResponse | null
}>()

const memberSearchQuery = ref('')
const memberRoleFilter = ref('')
const syncMembersMutation = useSyncTelegramChatMembersMutation()
const joinChatMutation = useJoinTelegramChatMutation()
const leaveChatMutation = useLeaveTelegramChatMutation()
const retryMutation = useTelegramCommandRetryMutation()
const membersQuery = useTelegramChatMembersQuery(
  () => props.telegramChatId,
  () => 50,
  () => memberSearchQuery.value,
  () => memberRoleFilter.value
)
const commandsQuery = useTelegramCommandsQuery(
  () => props.accountId,
  10,
  () => Boolean(props.accountId && props.providerChatId),
  {
    providerChatId: () => props.providerChatId,
    commandKinds: () => ['join', 'leave'],
  }
)

const chatMembers = computed<TelegramChatMember[]>(() => membersQuery.data.value ?? [])
const lifecycleCommands = computed(() =>
  telegramParticipantLifecycleCommands(
    commandsQuery.data.value ?? [],
    props.providerChatId,
    2
  )
)
const hasActiveMemberFilters = computed(
  () => memberSearchQuery.value.trim().length > 0 || memberRoleFilter.value.trim().length > 0
)

const hasProviderRoster = computed(() =>
  chatMembers.value.some((member) => member.source === 'tdlib' || member.source === 'bot_api')
)

const syncLabel = computed(() => {
  if (syncMembersMutation.isPending.value) return t('Syncing members')
  return t('Sync provider roster')
})

const participantActionPending = computed(
  () => joinChatMutation.isPending.value || leaveChatMutation.isPending.value
)

function capability(operation: string): TelegramOperationCapability | undefined {
  return props.capabilities?.capabilities.find((item) => item.operation === operation)
}

function capabilityEnabled(operation: string): boolean {
  return capability(operation)?.status === 'available'
}

function capabilityTitle(operation: string, fallback: string): string {
  const item = capability(operation)
  return item ? `${fallback}: ${item.status} - ${item.reason}` : fallback
}

function memberName(member: TelegramChatMember): string {
  return member.sender_display_name ?? member.username ?? member.provider_member_id
}

function memberRole(member: TelegramChatMember): string {
  if (member.is_owner) return t('Owner')
  if (member.is_admin) return t('Admin')
  return member.role ?? t('Member')
}

function sourceLabel(member: TelegramChatMember): string {
  if (member.source === 'tdlib') return 'TDLib'
  if (member.source === 'bot_api') return 'Bot API'
  return t('message heuristic')
}

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

function permissionSummary(member: TelegramChatMember): string {
  const entries = Object.entries(member.permissions ?? {})
    .filter(([, value]) => typeof value === 'boolean')
    .map(([key, value]) => `${key.replace(/^can_/, '')}:${value ? 'yes' : 'no'}`)
  return entries.length ? entries.join(' | ') : '-'
}

function requestNextPage() {
  if (!membersQuery.hasNextPage.value || membersQuery.isFetchingNextPage.value) return
  void membersQuery.fetchNextPage()
}

function syncMembers() {
  if (!props.telegramChatId || syncMembersMutation.isPending.value) return
  syncMembersMutation.mutate(props.telegramChatId)
}

function joinChat() {
  if (!props.accountId || !props.providerChatId || participantActionPending.value) return
  joinChatMutation.mutate({
    telegramChatId: props.telegramChatId,
    accountId: props.accountId,
    providerChatId: props.providerChatId,
  })
}

function leaveChat() {
  if (!props.telegramChatId || !props.accountId || !props.providerChatId || participantActionPending.value) return
  leaveChatMutation.mutate({
    telegramChatId: props.telegramChatId,
    accountId: props.accountId,
    providerChatId: props.providerChatId,
  })
}

function canRetryLifecycleCommand(command: TelegramProviderWriteCommand): boolean {
  return command.status === 'dead_letter' || command.status === 'failed'
}
</script>

<template>
  <div class="telegram-members-panel">
    <div class="telegram-members-panel__toolbar">
      <label class="telegram-member-search">
        <Icon icon="tabler:search" width="15" height="15" />
        <input
          v-model="memberSearchQuery"
          type="search"
          :placeholder="t('Search provider members')"
        />
      </label>
      <label class="telegram-member-role">
        <Icon icon="tabler:shield" width="15" height="15" />
        <input
          v-model="memberRoleFilter"
          type="search"
          :placeholder="t('Role filter')"
        />
      </label>
      <Button
        variant="outline"
        size="sm"
        :disabled="!telegramChatId || syncMembersMutation.isPending.value"
        @click="syncMembers"
      >
        <Icon icon="tabler:refresh" />
        {{ syncLabel }}
      </Button>
    </div>

    <div class="telegram-members-panel__actions">
      <Button
        variant="outline"
        size="sm"
        :disabled="!accountId || !providerChatId || participantActionPending || !capabilityEnabled('participants.join')"
        :title="capabilityTitle('participants.join', t('Join chat'))"
        @click="joinChat"
      >
        <Icon icon="tabler:user-plus" />
        {{ t('Join chat') }}
      </Button>
      <Button
        variant="outline"
        size="sm"
        :disabled="!telegramChatId || !accountId || !providerChatId || participantActionPending || !capabilityEnabled('participants.leave')"
        :title="capabilityTitle('participants.leave', t('Leave chat'))"
        @click="leaveChat"
      >
        <Icon icon="tabler:user-minus" />
        {{ t('Leave chat') }}
      </Button>
    </div>

    <p v-if="syncMembersMutation.error.value" class="telegram-members-panel__error">
      {{ syncMembersMutation.error.value.message }}
    </p>
    <p v-if="joinChatMutation.error.value" class="telegram-members-panel__error">
      {{ joinChatMutation.error.value.message }}
    </p>
    <p v-if="leaveChatMutation.error.value" class="telegram-members-panel__error">
      {{ leaveChatMutation.error.value.message }}
    </p>
    <p v-if="commandsQuery.error.value" class="telegram-members-panel__error">
      {{ commandsQuery.error.value.message }}
    </p>
    <p v-if="membersQuery.error.value" class="telegram-members-panel__error">
      {{ membersQuery.error.value.message }}
    </p>
    <p v-if="chatMembers.length > 0 && !hasProviderRoster" class="telegram-members-panel__hint">
      {{ t('Showing message-sender fallback until provider roster is synced.') }}
    </p>

    <div
      v-if="lifecycleCommands.length > 0"
      class="telegram-members-panel__lifecycle"
      aria-live="polite"
    >
      <article
        v-for="command in lifecycleCommands"
        :key="command.command_id"
        class="telegram-members-panel__lifecycle-item"
      >
        <div class="telegram-members-panel__lifecycle-row">
          <strong>{{ t(telegramParticipantLifecycleTitle(command)) }}</strong>
          <span>{{ t(telegramCommandAuditState(command).label) }}</span>
        </div>
        <small>{{ telegramCommandAuditState(command).detail }}</small>
        <small>{{ formatDate(command.updated_at || command.happened_at) }}</small>
        <button
          v-if="canRetryLifecycleCommand(command)"
          type="button"
          class="telegram-members-panel__retry"
          :disabled="retryMutation.isPending.value"
          @click="retryMutation.mutate(command.command_id)"
        >
          <Icon icon="tabler:refresh" width="13" height="13" />
          {{ t('Retry command') }}
        </button>
      </article>
    </div>

    <div v-if="membersQuery.isLoading.value" class="telegram-inspector-placeholder">
      {{ t('Loading provider roster...') }}
    </div>
    <div v-else-if="chatMembers.length === 0" class="telegram-inspector-placeholder">
      {{ hasActiveMemberFilters
        ? t('No provider members match this search.')
        : t('Members will appear after provider roster sync or selected-chat history sync.') }}
    </div>
    <article
      v-for="member in chatMembers"
      :key="member.provider_member_id"
      class="telegram-rail-card telegram-member-card"
    >
      <div class="telegram-member-card__main">
        <strong>{{ memberName(member) }}</strong>
        <p>{{ member.provider_member_id }}</p>
        <div class="telegram-member-card__badges">
          <span>{{ sourceLabel(member) }}</span>
          <span>{{ memberRole(member) }}</span>
          <span v-if="member.status">{{ member.status }}</span>
        </div>
        <small>{{ permissionSummary(member) }}</small>
      </div>
      <div class="telegram-member-side">
        <b>{{ member.source === 'message_heuristic' ? member.message_count : 'provider' }}</b>
        <small>{{ formatDate(member.observed_at ?? member.last_message_at) }}</small>
      </div>
    </article>
    <Button
      v-if="membersQuery.hasNextPage.value"
      variant="outline"
      size="sm"
      :disabled="membersQuery.isFetchingNextPage.value"
      @click="requestNextPage"
    >
      <Icon icon="tabler:chevrons-down" />
      {{ membersQuery.isFetchingNextPage.value ? t('Loading members') : t('Load more members') }}
    </Button>
  </div>
</template>

<style scoped>
.telegram-members-panel,
.telegram-member-card__main {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.telegram-members-panel__toolbar {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 140px) auto;
  gap: 8px;
  align-items: center;
}
.telegram-members-panel__actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
.telegram-members-panel__lifecycle {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.telegram-members-panel__lifecycle-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 8px 10px;
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 8px;
  background: var(--color-bg, #fafafa);
}
.telegram-members-panel__lifecycle-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}
.telegram-member-search {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 999px;
  background: var(--color-surface, #fff);
  color: var(--color-text-secondary, #777);
}
.telegram-member-role {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 999px;
  background: var(--color-surface, #fff);
  color: var(--color-text-secondary, #777);
}
.telegram-member-search input {
  flex: 1;
  min-width: 0;
  border: none;
  background: transparent;
  font: inherit;
  color: var(--color-text, #333);
  outline: none;
}
.telegram-member-role input {
  flex: 1;
  min-width: 0;
  border: none;
  background: transparent;
  font: inherit;
  color: var(--color-text, #333);
  outline: none;
}
.telegram-members-panel__hint,
.telegram-members-panel__error,
.telegram-members-panel__lifecycle-item small,
.telegram-members-panel__lifecycle-row span {
  margin: 0;
  font-size: 11px;
  color: var(--color-text-secondary, #777);
}
.telegram-members-panel__lifecycle-row strong {
  font-size: 12px;
  color: var(--color-text, #333);
}
.telegram-members-panel__error {
  color: var(--color-danger, #b42318);
}
.telegram-members-panel__retry {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  width: fit-content;
  padding: 0;
  border: none;
  background: transparent;
  color: var(--color-accent, #2563eb);
  font-size: 11px;
  cursor: pointer;
}
.telegram-members-panel__retry:disabled {
  opacity: 0.6;
  cursor: default;
}
.telegram-member-card {
  display: flex;
  justify-content: space-between;
  gap: 12px;
}
.telegram-member-card strong {
  display: block;
  font-size: 13px;
}
.telegram-member-card p,
.telegram-member-card small,
.telegram-member-side small {
  margin: 0;
  font-size: 11px;
  color: var(--color-text-secondary, #777);
}
.telegram-member-card__badges {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}
.telegram-member-card__badges span {
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 999px;
  padding: 2px 6px;
  font-size: 10px;
  color: var(--color-text-secondary, #777);
}
.telegram-member-side {
  text-align: right;
}
</style>
