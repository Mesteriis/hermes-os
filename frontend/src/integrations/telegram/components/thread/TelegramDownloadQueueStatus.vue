<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '../../../../platform/i18n'
import Icon from '../../../../shared/ui/Icon.vue'
import { telegramAttachmentReadiness } from '../../stores/telegramMediaSearch'
import { telegramDownloadQueueItems, telegramDownloadQueueTitle } from '../../stores/telegramDownloadQueue'
import type { TelegramAttachmentHint } from '../../types/telegram'

const { t } = useI18n()

const emit = defineEmits<{
  downloadMedia: [attachment: TelegramAttachmentHint]
}>()

const props = defineProps<{
  fileHints: TelegramAttachmentHint[]
  voiceHints: TelegramAttachmentHint[]
  isTelegramActionSubmitting: boolean
}>()

const downloadItems = computed(() =>
  telegramDownloadQueueItems(props.fileHints, props.voiceHints, 4)
)

function attachmentReadiness(attachment: TelegramAttachmentHint) {
  return telegramAttachmentReadiness(attachment)
}

function canRetry(attachment: TelegramAttachmentHint): boolean {
  return attachmentReadiness(attachment).action_label === 'Retry download'
}
</script>

<template>
  <div v-if="downloadItems.length > 0" class="telegram-download-queue" aria-live="polite">
    <article
      v-for="attachment in downloadItems"
      :key="attachment.id"
      class="telegram-download-queue__item"
      :class="{ 'telegram-download-queue__item--failed': attachmentReadiness(attachment).label === 'Download failed' }"
    >
      <div class="telegram-download-queue__row">
        <div class="telegram-download-queue__title">
          <Icon icon="tabler:download" width="14" height="14" />
          <strong>{{ telegramDownloadQueueTitle(attachment) }}</strong>
        </div>
        <span class="telegram-download-queue__state">
          {{ t(attachmentReadiness(attachment).label) }}
        </span>
      </div>
      <small>{{ attachmentReadiness(attachment).detail }}</small>
      <button
        v-if="canRetry(attachment)"
        type="button"
        class="telegram-download-queue__retry"
        :disabled="isTelegramActionSubmitting"
        @click="emit('downloadMedia', attachment)"
      >
        <Icon icon="tabler:refresh" width="13" height="13" />
        {{ t('Retry download') }}
      </button>
    </article>
  </div>
</template>

<style scoped>
.telegram-download-queue {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 8px;
}

.telegram-download-queue__item {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 8px 10px;
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 8px;
  background: var(--color-bg, #fafafa);
}

.telegram-download-queue__item--failed {
  border-color: color-mix(in srgb, var(--color-danger, #b42318) 55%, var(--color-border, #e0e0e0));
}

.telegram-download-queue__row,
.telegram-download-queue__title {
  display: flex;
  align-items: center;
  gap: 6px;
}

.telegram-download-queue__row {
  justify-content: space-between;
}

.telegram-download-queue__title strong,
.telegram-download-queue__state,
.telegram-download-queue__item small,
.telegram-download-queue__retry {
  font-size: 11px;
}

.telegram-download-queue__title strong {
  color: var(--color-text, #333);
  word-break: break-word;
}

.telegram-download-queue__state,
.telegram-download-queue__item small {
  color: var(--color-text-secondary, #777);
}

.telegram-download-queue__retry {
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

.telegram-download-queue__retry:disabled {
  opacity: 0.6;
  cursor: default;
}
</style>
