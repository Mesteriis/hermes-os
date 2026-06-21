<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from '../../../../platform/i18n'
import Icon from '../../../../shared/ui/Icon.vue'
import { telegramTopicProviderLabel, telegramTopicStateLabel } from '../../stores/telegramTopicProjection'
import { useTelegramAccountCapabilitiesQuery } from '../../queries/useTelegramQuery'
import {
  useTelegramTopicsQuery,
  useTelegramTopicSearchQuery,
} from '../../../../shared/communications/telegramBusinessQueries'
import {
  useCreateTelegramTopicMutation,
  useToggleTelegramTopicClosedMutation,
} from '../../queries/useTelegramTopicLifecycleQuery'

const { t } = useI18n()

const emit = defineEmits<{
  selectTopic: [topicId: string]
}>()

const props = defineProps<{
  accountId: string | null | undefined
  telegramChatId: string | null | undefined
  providerChatIdHint: string | null | undefined
  isTelegramActionSubmitting: boolean
}>()

const topicSearchQuery = ref('')
const newTopicTitle = ref('')
const { data: topicsData, isLoading: topicsLoading } = useTelegramTopicsQuery(
  computed(() => props.telegramChatId)
)
const capabilityQuery = useTelegramAccountCapabilitiesQuery(computed(() => props.accountId))
const { data: topicSearchData } = useTelegramTopicSearchQuery(
  computed(() => props.telegramChatId),
  topicSearchQuery
)
const createTopicMutation = useCreateTelegramTopicMutation()
const toggleTopicClosedMutation = useToggleTelegramTopicClosedMutation()
const displayedTopics = computed(() =>
  topicSearchQuery.value.trim() ? (topicSearchData.value?.items ?? []) : (topicsData.value?.items ?? [])
)
const topicCapabilities = computed(() => capabilityQuery.data.value?.capabilities ?? [])
const canCreateTopics = computed(() =>
  topicCapabilities.value.some((capability) => capability.operation === 'topics.create' && capability.status === 'available')
)
const canToggleTopics = computed(() =>
  topicCapabilities.value.some((capability) => capability.operation === 'topics.close' && capability.status === 'available')
)
const topicMutationPending = computed(() =>
  createTopicMutation.isPending.value || toggleTopicClosedMutation.isPending.value
)
const topicActionDisabled = computed(() =>
  props.isTelegramActionSubmitting || topicMutationPending.value
)

async function submitTopicCreate() {
  const title = newTopicTitle.value.trim()
  const telegramChatId = props.telegramChatId?.trim()
  const accountId = props.accountId?.trim()
  const providerChatId = props.providerChatIdHint?.trim()
    ?? displayedTopics.value[0]?.provider_chat_id
    ?? topicsData.value?.items[0]?.provider_chat_id

  if (!title || !telegramChatId || !accountId || !providerChatId || !canCreateTopics.value) return

  await createTopicMutation.mutateAsync({
    telegramChatId,
    accountId,
    providerChatId,
    title,
  })
  newTopicTitle.value = ''
}

async function toggleTopicClosed(topicId: string, providerChatId: string, isClosed: boolean) {
  const telegramChatId = props.telegramChatId?.trim()
  const accountId = props.accountId?.trim()
  if (!telegramChatId || !accountId || !providerChatId || !canToggleTopics.value) return

  await toggleTopicClosedMutation.mutateAsync({
    topicId,
    telegramChatId,
    accountId,
    providerChatId,
    isClosed,
  })
}
</script>

<template>
  <div class="telegram-topic-toolbar">
    <input
      v-model="newTopicTitle"
      type="text"
      :placeholder="t('New topic title')"
      class="telegram-topic-create-input"
      :disabled="topicActionDisabled || !canCreateTopics"
      @keyup.enter="submitTopicCreate"
    />
    <button
      type="button"
      class="telegram-topic-action-button"
      :disabled="topicActionDisabled || !canCreateTopics || !newTopicTitle.trim()"
      :title="canCreateTopics ? t('Create topic') : t('Topic create requires QR-authorized TDLib runtime.')"
      @click="submitTopicCreate"
    >
      <Icon icon="tabler:plus" width="16" height="16" />
    </button>
  </div>
  <div class="telegram-topic-search-bar">
    <input
      v-model="topicSearchQuery"
      type="search"
      :placeholder="t('Search topics…')"
      class="telegram-topic-search-input"
    />
  </div>
  <div v-if="topicsLoading && !topicSearchQuery.trim()" class="empty-panel fill">
    {{ t('Loading topics…') }}
  </div>
  <div v-else-if="displayedTopics.length === 0" class="empty-panel fill">
    {{ topicSearchQuery.trim() ? t('No topics match your search.') : t('No forum topics found for this chat.') }}
  </div>
  <div v-else class="telegram-topic-list">
    <article
      v-for="topic in displayedTopics"
      :key="topic.topic_id"
      class="telegram-topic-card"
      @click="emit('selectTopic', topic.topic_id)"
    >
      <span class="telegram-topic-card__icon">
        <template v-if="topic.icon_emoji">{{ topic.icon_emoji }}</template>
        <Icon v-else icon="tabler:message-circle" width="16" height="16" />
      </span>
      <div class="telegram-topic-card__body">
        <strong>{{ topic.title }}</strong>
        <small>{{ telegramTopicStateLabel(topic) }}</small>
        <small>{{ telegramTopicProviderLabel(topic) }}</small>
      </div>
      <button
        type="button"
        class="telegram-topic-card__action"
        :disabled="topicActionDisabled || !canToggleTopics"
        :title="topic.is_closed ? t('Reopen topic') : t('Close topic')"
        @click.stop="toggleTopicClosed(topic.topic_id, topic.provider_chat_id, !topic.is_closed)"
      >
        <Icon :icon="topic.is_closed ? 'tabler:lock-open-2' : 'tabler:lock'" width="15" height="15" />
      </button>
      <span v-if="topic.unread_count > 0" class="telegram-topic-card__badge">{{ topic.unread_count }}</span>
      <Icon v-if="topic.is_pinned" icon="tabler:pin" width="13" height="13" class="telegram-topic-card__pin" />
      <Icon icon="tabler:chevron-right" width="16" height="16" class="telegram-topic-card__arrow" />
    </article>
  </div>
</template>

<style scoped>
.empty-panel.fill {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  font-size: 13px;
  color: var(--color-text-secondary, #999);
}
.telegram-topic-toolbar,
.telegram-topic-search-bar {
  display: flex;
  gap: 8px;
  padding-top: 8px;
}
.telegram-topic-create-input,
.telegram-topic-search-input {
  flex: 1;
  min-width: 0;
  border: 1px solid var(--color-border, #d0d5dd);
  border-radius: 6px;
  padding: 8px 10px;
  font-size: 12px;
  background: var(--color-surface, #fff);
}
.telegram-topic-action-button,
.telegram-topic-card__action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--color-border, #d0d5dd);
  background: var(--color-surface, #fff);
  color: var(--color-text-secondary, #667085);
  border-radius: 6px;
  cursor: pointer;
}
.telegram-topic-action-button {
  width: 34px;
  height: 34px;
}
.telegram-topic-search-bar {
  padding: 6px 8px 2px;
}
.telegram-topic-search-input {
  width: 100%;
  padding: 5px 8px;
  border: 1px solid var(--color-border, #ddd);
  border-radius: 4px;
  font-size: 12px;
  background: var(--color-bg, #fff);
  color: var(--color-text, #333);
}
.telegram-topic-search-input:focus {
  outline: none;
  border-color: var(--color-primary, #2196f3);
}
.telegram-topic-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 6px 0;
}
.telegram-topic-card {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 10px;
  border-radius: 8px;
  cursor: pointer;
  font-size: 13px;
  border: 1px solid var(--color-border, #e8ecf0);
  background: var(--color-surface, #fff);
}
.telegram-topic-card:hover {
  background: var(--color-primary-subtle, #e3f2fd);
}
.telegram-topic-card__icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  font-size: 16px;
  flex-shrink: 0;
}
.telegram-topic-card__body {
  flex: 1;
  min-width: 0;
}
.telegram-topic-card__body strong {
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.telegram-topic-card__body small {
  font-size: 10px;
  color: var(--color-text-secondary, #999);
}
.telegram-topic-card__action {
  width: 28px;
  height: 28px;
}
.telegram-topic-card__badge {
  min-width: 18px;
  height: 18px;
  padding: 0 5px;
  border-radius: 999px;
  background: var(--color-primary, #0066cc);
  color: #fff;
  font-size: 10px;
  font-weight: 600;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
.telegram-topic-card__pin {
  color: var(--color-text-secondary, #aaa);
  flex-shrink: 0;
}
.telegram-topic-card__arrow {
  color: var(--color-text-secondary, #bbb);
  flex-shrink: 0;
}
</style>
