<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import Button from '../../../shared/ui/Button.vue'
import { useJoinTelegramChatMutation, useLeaveTelegramChatMutation } from '../queries/useTelegramParticipantLifecycleQuery'
import { useSyncTelegramChatMembersMutation } from '../queries/useTelegramQuery'
import type { TelegramCapabilitiesResponse, TelegramChatMember, TelegramOperationCapability } from '../types/telegram'

const { t } = useI18n()

const props = defineProps<{
  telegramChatId: string | null
  accountId: string | null
  providerChatId: string | null
  chatMembers: TelegramChatMember[]
  capabilities: TelegramCapabilitiesResponse | null
}>()

const memberSearchQuery = ref('')
const syncMembersMutation = useSyncTelegramChatMembersMutation()
const joinChatMutation = useJoinTelegramChatMutation()
const leaveChatMutation = useLeaveTelegramChatMutation()

const filteredChatMembers = computed(() => {
  const query = memberSearchQuery.value.trim().toLowerCase()
  if (!query) return props.chatMembers
  return props.chatMembers.filter((member) =>
    [
      member.sender_display_name ?? '',
      member.sender_id,
      member.provider_member_id,
      member.username ?? '',
      member.role ?? '',
      member.status ?? '',
      member.source,
    ]
      .join(' ')
      .toLowerCase()
      .includes(query)
  )
})

const hasProviderRoster = computed(() =>
  props.chatMembers.some((member) => member.source === 'tdlib' || member.source === 'bot_api')
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
  if (!value) return '-'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return '-'
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
</script>

<template>
  <div class="telegram-members-panel">
    <div class="telegram-members-panel__toolbar">
      <label v-if="chatMembers.length > 0" class="telegram-member-search">
        <Icon icon="tabler:search" width="15" height="15" />
        <input
          v-model="memberSearchQuery"
          type="search"
          :placeholder="t('Search provider members')"
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
    <p v-if="chatMembers.length > 0 && !hasProviderRoster" class="telegram-members-panel__hint">
      {{ t('Showing message-sender fallback until provider roster is synced.') }}
    </p>

    <div v-if="chatMembers.length === 0" class="telegram-inspector-placeholder">
      {{ t('Members will appear after provider roster sync or selected-chat history sync.') }}
    </div>
    <div v-else-if="filteredChatMembers.length === 0" class="telegram-inspector-placeholder">
      {{ t('No provider members match this search.') }}
    </div>
    <article
      v-for="member in filteredChatMembers"
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
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 8px;
  align-items: center;
}
.telegram-members-panel__actions {
  display: flex;
  flex-wrap: wrap;
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
.telegram-member-search input {
  flex: 1;
  min-width: 0;
  border: none;
  background: transparent;
  font: inherit;
  color: var(--color-text, #333);
  outline: none;
}
.telegram-members-panel__hint,
.telegram-members-panel__error {
  margin: 0;
  font-size: 11px;
  color: var(--color-text-secondary, #777);
}
.telegram-members-panel__error {
  color: var(--color-danger, #b42318);
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
