<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '../../../../platform/i18n'
import Icon from '../../../../shared/ui/Icon.vue'
import type { TelegramMessage, TelegramReactionSummary } from '../../types/telegram'
import TelegramRawEvidencePanel from './TelegramRawEvidencePanel.vue'
import {
  buildTelegramCustomReactionEvidence,
  buildTelegramMessageLinkEvidence,
  buildTelegramStructuredEvidence,
  matchesTelegramReferenceQuery,
} from './telegramReferenceMetadataEvidence'

const { t } = useI18n()

const props = defineProps<{
  messageId: string
  isOpen: boolean
  currentMessage: TelegramMessage
  reactionSummary: TelegramReactionSummary | null
  referenceQuery: string
}>()

const messageLink = computed(() => buildTelegramMessageLinkEvidence(props.currentMessage))
const structuredEvidence = computed(() => buildTelegramStructuredEvidence(props.currentMessage, t))
const customReactionSummary = computed(() => buildTelegramCustomReactionEvidence(props.currentMessage))
const isRawEvidenceMatch = computed(() =>
  matchesTelegramReferenceQuery(props.referenceQuery, props.currentMessage.raw_record_id, props.currentMessage.provider_message_id)
)
const filteredStructuredEvidence = computed(() =>
  structuredEvidence.value.filter((item) =>
    matchesTelegramReferenceQuery(props.referenceQuery, item.title, item.key, ...item.lines)
  )
)
const filteredCustomReactionSummary = computed(() =>
  customReactionSummary.value.filter((item) =>
    matchesTelegramReferenceQuery(props.referenceQuery, item.customEmojiId, item.count)
  )
)
const isMessageLinkMatch = computed(() =>
  messageLink.value ? matchesTelegramReferenceQuery(props.referenceQuery, messageLink.value.href, messageLink.value.kind) : false
)
</script>

<template>
  <div v-if="isRawEvidenceMatch" class="telegram-source-evidence__group">
    <TelegramRawEvidencePanel :message-id="messageId" :is-open="isOpen" />
  </div>

  <div v-if="messageLink && isMessageLinkMatch" class="telegram-source-evidence__group">
    <strong>{{ t('Message Link') }}</strong>
    <a class="telegram-source-evidence__item telegram-source-evidence__link" :href="messageLink.href" target="_blank" rel="noreferrer">
      <Icon icon="tabler:external-link" width="14" height="14" />
      <div>
        <p>{{ t('Open provider permalink') }}</p>
        <small>{{ messageLink.kind }}</small>
        <small>{{ messageLink.href }}</small>
      </div>
    </a>
  </div>

  <div v-if="filteredStructuredEvidence.length > 0" class="telegram-source-evidence__group">
    <strong>{{ t('Structured Evidence') }}</strong>
    <article v-for="item in filteredStructuredEvidence" :key="item.key" class="telegram-source-evidence__item">
      <Icon :icon="item.icon" width="14" height="14" />
      <div>
        <p>{{ item.title }}</p>
        <small v-for="line in item.lines" :key="line">{{ line }}</small>
      </div>
    </article>
  </div>

  <div
    v-if="(reactionSummary?.reactions.length ?? 0) > 0 || filteredCustomReactionSummary.length > 0"
    class="telegram-source-evidence__group"
  >
    <strong>{{ t('Reactions') }}</strong>
    <div class="telegram-source-evidence__chips">
      <span
        v-for="group in reactionSummary?.reactions ?? []"
        :key="group.reaction_emoji"
        class="telegram-source-evidence__chip"
        :title="group.senders.join(', ')"
      >
        {{ group.reaction_emoji }} {{ group.count }}
      </span>
      <span
        v-for="group in filteredCustomReactionSummary"
        :key="group.customEmojiId"
        class="telegram-source-evidence__chip"
        :title="t('Custom Telegram reaction evidence')"
      >
        {{ t('Custom') }} #{{ group.customEmojiId }} {{ group.count }}
      </span>
    </div>
  </div>
</template>

<style scoped>
.telegram-source-evidence__group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.telegram-source-evidence__group strong {
  font-size: 11px;
  color: var(--color-text, #333);
}
.telegram-source-evidence__chips {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.telegram-source-evidence__chip {
  padding: 2px 6px;
  border-radius: 999px;
  background: var(--color-surface, #fff);
  border: 1px solid var(--color-border, #e0e0e0);
  font-size: 11px;
}
.telegram-source-evidence__item {
  display: flex;
  gap: 8px;
  align-items: flex-start;
  font-size: 11px;
  color: var(--color-text, #333);
}
.telegram-source-evidence__link {
  text-decoration: none;
}
.telegram-source-evidence__item p,
.telegram-source-evidence__item small {
  margin: 0;
}
.telegram-source-evidence__item small {
  color: var(--color-text-secondary, #777);
}
</style>
