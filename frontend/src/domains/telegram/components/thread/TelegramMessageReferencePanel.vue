<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from '../../../../platform/i18n'
import Icon from '../../../../shared/ui/Icon.vue'
import TelegramMessageSourceEvidencePanel from './TelegramMessageSourceEvidencePanel.vue'
import type {
  TelegramMessage,
  TelegramMessageReferenceSummary,
  TelegramProviderWriteCommand,
} from '../../types/telegram'
import {
  previousTelegramVersionBody,
  summarizeTelegramCommandEvidence,
  summarizeTelegramTombstoneState,
  summarizeTelegramVersionDelta,
} from './referenceEvidence'
import {
  hasTelegramSourceEvidence,
  matchesTelegramReferenceQuery,
  matchesTelegramSourceEvidence,
} from './telegramReferenceMetadataEvidence'
import {
  useTelegramCommandsQuery,
  useTelegramMessageReactionsQuery,
  useTelegramMessageTombstonesQuery,
  useTelegramMessageVersionsQuery,
} from '../../queries/useTelegramLifecycleQuery'
import { useTelegramForwardChainQuery, useTelegramReplyChainQuery } from '../../queries/useTelegramReferenceQuery'

const { t } = useI18n()

const props = defineProps<{
  messageId: string
  isOpen: boolean
  currentMessage: TelegramMessage
}>()

const emit = defineEmits<{
  openMessage: [message: TelegramMessage]
}>()

const referenceSearchQuery = ref('')

const replyChainQuery = useTelegramReplyChainQuery(
  computed(() => props.messageId),
  computed(() => props.isOpen)
)
const forwardChainQuery = useTelegramForwardChainQuery(
  computed(() => props.messageId),
  computed(() => props.isOpen)
)
const versionsQuery = useTelegramMessageVersionsQuery(
  computed(() => props.messageId),
  computed(() => props.isOpen)
)
const tombstonesQuery = useTelegramMessageTombstonesQuery(
  computed(() => props.messageId),
  computed(() => props.isOpen)
)
const reactionsQuery = useTelegramMessageReactionsQuery(
  computed(() => props.messageId),
  computed(() => props.isOpen)
)
const commandsQuery = useTelegramCommandsQuery(
  computed(() => props.currentMessage.account_id),
  25,
  computed(() => props.isOpen)
)

const replyToItems = computed(() => replyChainQuery.data.value?.reply_to ?? [])
const replies = computed(() => replyChainQuery.data.value?.replies ?? [])
const forwards = computed(() => forwardChainQuery.data.value?.forwards ?? [])
const versions = computed(() => versionsQuery.data.value?.versions ?? [])
const tombstones = computed(() => tombstonesQuery.data.value?.tombstones ?? [])
const reactions = computed(() => reactionsQuery.data.value?.reactions ?? [])
const reactionSummary = computed(() => reactionsQuery.data.value?.summary ?? null)
const hasSourceEvidence = computed(() => hasTelegramSourceEvidence(props.currentMessage, t))
const isSourceEvidenceMatch = computed(() => matchesTelegramSourceEvidence(props.currentMessage, t, referenceSearchQuery.value))
const relatedCommands = computed(() => {
  const commands = commandsQuery.data.value ?? []
  const directMatches = commands.filter((command) => command.provider_message_id === props.currentMessage.provider_message_id)
  if (directMatches.length > 0) {
    return directMatches
  }
  return commands
    .filter((command) => command.provider_chat_id === props.currentMessage.provider_chat_id)
    .slice(0, 5)
})
const isLoading = computed(
  () =>
    replyChainQuery.isLoading.value ||
    forwardChainQuery.isLoading.value ||
    versionsQuery.isLoading.value ||
    tombstonesQuery.isLoading.value ||
    reactionsQuery.isLoading.value ||
    commandsQuery.isLoading.value
)

function matchesReferenceQuery(...values: Array<string | null | undefined>): boolean {
  return matchesTelegramReferenceQuery(referenceSearchQuery.value, ...values)
}

const filteredReplyToItems = computed(() =>
  replyToItems.value.filter((item) =>
    matchesReferenceQuery(
      replyTargetTitle(item),
      replyTargetBody(item),
      item.target_provider_id
    )
  )
)

const filteredReplies = computed(() =>
  replies.value.filter((item) =>
    matchesReferenceQuery(
      replyTitle(item),
      replyBody(item),
      item.source_provider_id
    )
  )
)

const filteredForwards = computed(() =>
  forwards.value.filter((item) =>
    matchesReferenceQuery(
      forwardTitle(item),
      forwardMeta(item),
      item.source_message_summary?.text ?? null
    )
  )
)

const filteredVersions = computed(() =>
  versions.value.filter((version) =>
    matchesReferenceQuery(
      version.body_text ?? null,
      `version ${version.version_number}`,
      formatDate(version.edit_timestamp)
    )
  )
)

const filteredTombstones = computed(() =>
  tombstones.value.filter((tombstone) =>
    matchesReferenceQuery(
      tombstone.reason_class,
      tombstone.actor_class,
      formatDate(tombstone.observed_at)
    )
  )
)

const filteredCommands = computed(() =>
  relatedCommands.value.filter((command) =>
    matchesReferenceQuery(
      command.command_kind,
      command.status,
      command.action_class,
      command.capability_state,
      command.confirmation_decision,
      command.provider_message_id,
      command.provider_chat_id
    )
  )
)

function formatDate(value: string | null): string {
  if (!value) return ''
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return ''
  return new Intl.DateTimeFormat('en', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  }).format(date)
}

function replyTargetTitle(item: (typeof replyToItems.value)[number]): string {
  return item.target_message_summary?.sender_display_name
    ?? item.target_message_summary?.sender
    ?? item.target_provider_id
}

function replyTargetBody(item: (typeof replyToItems.value)[number]): string {
  return item.target_message_summary?.text || item.target_provider_id
}

function replyTitle(item: (typeof replies.value)[number]): string {
  return item.source_message_summary?.sender_display_name
    ?? item.source_message_summary?.sender
    ?? item.source_provider_id
}

function replyBody(item: (typeof replies.value)[number]): string {
  return item.source_message_summary?.text || item.source_provider_id
}

function forwardTitle(item: (typeof forwards.value)[number]): string {
  return item.forward_origin_sender_name
    ?? item.forward_origin_sender_id
    ?? item.forward_origin_message_id
    ?? item.source_message_summary?.sender_display_name
    ?? item.source_message_summary?.sender
    ?? t('Unknown origin')
}

function forwardMeta(item: (typeof forwards.value)[number]): string {
  const segments = [
    item.forward_origin_chat_id,
    item.forward_date ? formatDate(item.forward_date) : null,
  ].filter(Boolean)
  return segments.join(' · ')
}

function forwardBody(item: (typeof forwards.value)[number]): string {
  return item.source_message_summary?.text
    ?? item.forward_origin_message_id
    ?? t('No origin summary')
}

function buildFocusedMessage(
  summary: TelegramMessageReferenceSummary,
  providerChatIdFallback: string
): TelegramMessage {
  return {
    message_id: summary.message_id,
    raw_record_id: `telegram-reference:${summary.message_id}`,
    account_id: props.currentMessage.account_id,
    provider_message_id: summary.provider_message_id,
    provider_chat_id: summary.provider_chat_id ?? providerChatIdFallback,
    chat_title: summary.chat_title,
    sender: summary.sender,
    sender_display_name: summary.sender_display_name,
    text: summary.text,
    occurred_at: summary.occurred_at,
    projected_at: summary.occurred_at ?? props.currentMessage.projected_at,
    channel_kind: props.currentMessage.channel_kind,
    delivery_state: 'received',
    metadata: {},
  }
}

function openReplyTarget(item: (typeof replyToItems.value)[number]) {
  if (!item.target_message_summary) return
  emit(
    'openMessage',
    buildFocusedMessage(item.target_message_summary, item.provider_chat_id)
  )
}

function openReplySource(item: (typeof replies.value)[number]) {
  if (!item.source_message_summary) return
  emit(
    'openMessage',
    buildFocusedMessage(item.source_message_summary, item.provider_chat_id)
  )
}

function openForwardSource(item: (typeof forwards.value)[number]) {
  if (!item.source_message_summary) return
  emit(
    'openMessage',
    buildFocusedMessage(item.source_message_summary, item.provider_chat_id)
  )
}

function commandSubject(command: TelegramProviderWriteCommand): string {
  const segments = [
    command.command_kind,
    command.status,
    command.retry_count > 0 ? `${t('Retries')} ${command.retry_count}` : null,
  ].filter(Boolean)
  return segments.join(' · ')
}

function versionPreviousBody(currentIndex: number): string | null {
  return previousTelegramVersionBody(filteredVersions.value, currentIndex)
}
</script>

<template>
  <section class="telegram-reference-panel">
    <div v-if="isLoading" class="telegram-reference-panel__empty">
      {{ t('Loading Telegram references...') }}
    </div>
    <div
      v-else-if="
        replyToItems.length === 0 &&
        replies.length === 0 &&
        forwards.length === 0 &&
        versions.length === 0 &&
        tombstones.length === 0 &&
        reactions.length === 0 &&
        relatedCommands.length === 0 &&
        !hasSourceEvidence
      "
      class="telegram-reference-panel__empty"
    >
      {{ t('No lifecycle, reply, forward or reaction evidence projected for this message yet.') }}
    </div>
    <div v-else class="telegram-reference-panel__stack">
      <label class="telegram-reference-panel__search">
        <Icon icon="tabler:search" width="14" height="14" />
        <input
          v-model="referenceSearchQuery"
          type="search"
          :placeholder="t('Filter references, lifecycle and commands')"
        />
      </label>

      <TelegramMessageSourceEvidencePanel
        :message-id="messageId"
        :is-open="isOpen"
        :current-message="currentMessage"
        :reaction-summary="reactionSummary"
        :reference-query="referenceSearchQuery"
      />

      <div v-if="filteredReplyToItems.length > 0" class="telegram-reference-panel__group">
        <strong>{{ t('Replies To') }}</strong>
        <article
          v-for="item in filteredReplyToItems"
          :key="item.reply_ref_id"
          class="telegram-reference-panel__item"
          :class="{ 'telegram-reference-panel__item--action': Boolean(item.target_message_summary) }"
          @click="openReplyTarget(item)"
        >
          <Icon icon="tabler:corner-up-left" width="14" height="14" />
          <div>
            <p>{{ replyTargetTitle(item) }}</p>
            <small>{{ replyTargetBody(item) }}</small>
            <small>{{ t('Depth') }} {{ item.reply_depth }}</small>
          </div>
        </article>
      </div>

      <div v-if="filteredReplies.length > 0" class="telegram-reference-panel__group">
        <strong>{{ t('Replies') }}</strong>
        <article
          v-for="item in filteredReplies"
          :key="item.reply_ref_id"
          class="telegram-reference-panel__item"
          :class="{ 'telegram-reference-panel__item--action': Boolean(item.source_message_summary) }"
          @click="openReplySource(item)"
        >
          <Icon icon="tabler:corner-down-right" width="14" height="14" />
          <div>
            <p>{{ replyTitle(item) }}</p>
            <small>{{ replyBody(item) }}</small>
            <small>{{ t('Depth') }} {{ item.reply_depth }}</small>
          </div>
        </article>
      </div>

      <div v-if="filteredForwards.length > 0" class="telegram-reference-panel__group">
        <strong>{{ t('Forwards') }}</strong>
        <article
          v-for="item in filteredForwards"
          :key="item.forward_ref_id"
          class="telegram-reference-panel__item"
          :class="{ 'telegram-reference-panel__item--action': Boolean(item.source_message_summary) }"
          @click="openForwardSource(item)"
        >
          <Icon icon="tabler:arrow-forward-up" width="14" height="14" />
          <div>
            <p>{{ forwardTitle(item) }}</p>
            <small>{{ forwardMeta(item) || t('No origin chat') }}</small>
            <small>{{ forwardBody(item) }}</small>
          </div>
        </article>
      </div>

      <div v-if="filteredVersions.length > 0" class="telegram-reference-panel__group">
        <strong>{{ t('Edit History') }}</strong>
        <article
          v-for="(version, index) in filteredVersions"
          :key="version.version_id"
          class="telegram-reference-panel__item"
        >
          <Icon icon="tabler:pencil" width="14" height="14" />
          <div>
            <p>{{ t('Version') }} {{ version.version_number }}</p>
            <small>{{ version.body_text || t('Empty body') }}</small>
            <small>{{ summarizeTelegramVersionDelta(version, versionPreviousBody(index)) }}</small>
            <small v-if="version.source_event">{{ version.source_event }}</small>
            <small>{{ formatDate(version.edit_timestamp) }}</small>
          </div>
        </article>
      </div>

      <div v-if="filteredTombstones.length > 0" class="telegram-reference-panel__group">
        <strong>{{ t('Tombstones') }}</strong>
        <article
          v-for="tombstone in filteredTombstones"
          :key="tombstone.tombstone_id"
          class="telegram-reference-panel__item"
        >
          <Icon icon="tabler:trash" width="14" height="14" />
          <div>
            <p>{{ tombstone.reason_class }}</p>
            <small>{{ tombstone.actor_class }} · {{ formatDate(tombstone.observed_at) }}</small>
            <small>{{ summarizeTelegramTombstoneState(tombstone) }}</small>
            <small v-if="tombstone.source_event">{{ tombstone.source_event }}</small>
          </div>
        </article>
      </div>

      <div v-if="filteredCommands.length > 0" class="telegram-reference-panel__group">
        <strong>{{ t('Recent Commands') }}</strong>
        <article
          v-for="command in filteredCommands"
          :key="command.command_id"
          class="telegram-reference-panel__item"
        >
          <Icon icon="tabler:clock-bolt" width="14" height="14" />
          <div>
            <p>{{ commandSubject(command) }}</p>
            <small>{{ command.provider_message_id ?? command.provider_chat_id }}</small>
            <small>{{ summarizeTelegramCommandEvidence(command) }}</small>
            <small>{{ formatDate(command.happened_at) }}</small>
          </div>
        </article>
      </div>

      <div
        v-if="
          referenceSearchQuery.trim() &&
          filteredReplyToItems.length === 0 &&
          filteredReplies.length === 0 &&
          filteredForwards.length === 0 &&
          filteredVersions.length === 0 &&
          filteredTombstones.length === 0 &&
          filteredCommands.length === 0 &&
          !isSourceEvidenceMatch
        "
        class="telegram-reference-panel__empty"
      >
        {{ t('No reference evidence matches this filter.') }}
      </div>
    </div>
  </section>
</template>

<style scoped>
.telegram-reference-panel {
  margin-top: 8px;
  padding: 8px 10px;
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 8px;
  background: var(--color-bg, #fafafa);
}
.telegram-reference-panel__empty {
  font-size: 11px;
  color: var(--color-text-secondary, #777);
}
.telegram-reference-panel__stack,
.telegram-reference-panel__group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.telegram-reference-panel__search {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 999px;
  background: var(--color-surface, #fff);
  color: var(--color-text-secondary, #777);
}
.telegram-reference-panel__search input {
  flex: 1;
  border: none;
  background: transparent;
  font: inherit;
  color: var(--color-text, #333);
  outline: none;
}
.telegram-reference-panel__group strong {
  font-size: 11px;
  color: var(--color-text, #333);
}
.telegram-reference-panel__item {
  display: flex;
  gap: 8px;
  align-items: flex-start;
  font-size: 11px;
  color: var(--color-text, #333);
}
.telegram-reference-panel__item--action {
  cursor: pointer;
}
.telegram-reference-panel__item--action:hover {
  background: var(--color-primary-subtle, #e3f2fd);
  border-radius: 6px;
}
.telegram-reference-panel__item p,
.telegram-reference-panel__item small {
  margin: 0;
}
.telegram-reference-panel__item small {
  color: var(--color-text-secondary, #777);
}
</style>
