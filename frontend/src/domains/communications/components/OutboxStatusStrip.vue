<script setup lang="ts">
import { computed } from 'vue'
import Icon from '../../../shared/ui/Icon.vue'
import type { EmailOutboxItem } from '../types/communications'
import {
  outboxStatusPresentation,
  visibleOutboxStatusItems
} from './outboxStatus'

const props = defineProps<{
  items: EmailOutboxItem[]
  isLoading: boolean
  isLoadingMore: boolean
  hasMore: boolean
  isUndoing: boolean
  errorMessage: string
}>()

const emit = defineEmits<{
  undo: [outboxId: string]
  loadMore: []
  prefetchMore: []
}>()

const statusItems = computed(() =>
  visibleOutboxStatusItems(props.items).map((item) => ({
    item,
    presentation: outboxStatusPresentation(item)
  }))
)
</script>

<template>
  <section v-if="isLoading || errorMessage || statusItems.length || hasMore" class="outbox-status-strip" aria-label="Outbox delivery status">
    <div v-if="isLoading" class="outbox-status-skeleton" />
    <div v-else-if="errorMessage" class="outbox-status-error">
      <Icon icon="tabler:alert-circle" />
      <span>{{ errorMessage }}</span>
    </div>
    <div v-else class="outbox-status-items">
      <article
        v-for="{ item, presentation } in statusItems"
        :key="item.outbox_id"
        class="outbox-status-item"
        :class="`tone-${presentation.tone}`"
      >
        <Icon :icon="presentation.icon" class="outbox-status-icon" />
        <div class="outbox-status-copy">
          <div class="outbox-status-line">
            <span class="outbox-status-title">{{ presentation.title }}</span>
            <span class="outbox-status-subject">{{ item.subject || '(No subject)' }}</span>
          </div>
          <div class="outbox-status-detail">{{ presentation.detail }}</div>
        </div>
        <button
          v-if="presentation.canUndo"
          class="outbox-status-undo"
          type="button"
          :disabled="isUndoing"
          title="Undo send"
          @click="emit('undo', item.outbox_id)"
        >
          <Icon icon="tabler:arrow-back-up" />
        </button>
      </article>
      <button
        v-if="hasMore"
        class="outbox-status-more"
        type="button"
        :disabled="isLoadingMore"
        title="Load more delivery records"
        @mouseenter="emit('prefetchMore')"
        @focus="emit('prefetchMore')"
        @click="emit('loadMore')"
      >
        <Icon :icon="isLoadingMore ? 'tabler:loader-2' : 'tabler:chevron-right'" />
      </button>
    </div>
  </section>
</template>

<style scoped>
.outbox-status-strip {
  flex: 0 0 auto;
  padding: 0.5rem;
  border-right: 1px solid var(--hh-border, #e5e7eb);
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 82%, transparent);
  backdrop-filter: blur(var(--hh-panel-blur));
}

.outbox-status-items {
  display: flex;
  gap: 0.5rem;
  overflow-x: auto;
  padding-bottom: 0.125rem;
}

.outbox-status-item {
  display: grid;
  grid-template-columns: auto minmax(10rem, 1fr) auto;
  align-items: center;
  gap: 0.5rem;
  min-width: min(22rem, 100%);
  max-width: 26rem;
  padding: 0.5rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 8px;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 88%, transparent);
  box-shadow: 0 10px 30px color-mix(in srgb, var(--hh-shadow, #0f172a) 9%, transparent);
}

.outbox-status-icon {
  width: 1rem;
  height: 1rem;
}

.outbox-status-copy {
  min-width: 0;
}

.outbox-status-line {
  display: flex;
  gap: 0.375rem;
  min-width: 0;
  align-items: baseline;
}

.outbox-status-title {
  flex: 0 0 auto;
  font-size: 0.75rem;
  font-weight: 700;
  color: var(--hh-text-primary, #1f2937);
}

.outbox-status-subject {
  overflow: hidden;
  color: var(--hh-text-primary, #1f2937);
  font-size: 0.75rem;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.outbox-status-detail {
  overflow: hidden;
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.6875rem;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.outbox-status-undo {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 1.75rem;
  height: 1.75rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 6px;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 90%, transparent);
  color: var(--hh-text-primary, #1f2937);
  cursor: pointer;
}

.outbox-status-undo:hover:not(:disabled) {
  background: var(--hh-bg-hover, #f3f4f6);
}

.outbox-status-undo:disabled {
  cursor: wait;
  opacity: 0.65;
}

.outbox-status-more {
  display: inline-flex;
  flex: 0 0 auto;
  align-items: center;
  justify-content: center;
  width: 2.25rem;
  min-width: 2.25rem;
  height: 2.75rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 8px;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 88%, transparent);
  color: var(--hh-text-primary, #1f2937);
  cursor: pointer;
}

.outbox-status-more:hover:not(:disabled) {
  background: var(--hh-bg-hover, #f3f4f6);
}

.outbox-status-more:disabled {
  cursor: wait;
  opacity: 0.65;
}

.outbox-status-error {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  color: var(--hh-danger, #b91c1c);
  font-size: 0.75rem;
}

.outbox-status-skeleton {
  width: min(24rem, 100%);
  height: 2.75rem;
  border-radius: 8px;
  background: linear-gradient(
    90deg,
    color-mix(in srgb, var(--hh-bg-muted, #f3f4f6) 70%, transparent),
    color-mix(in srgb, var(--hh-bg-primary, #ffffff) 72%, transparent),
    color-mix(in srgb, var(--hh-bg-muted, #f3f4f6) 70%, transparent)
  );
  background-size: 200% 100%;
  animation: outbox-status-pulse 1.4s ease-in-out infinite;
}

.tone-success .outbox-status-icon {
  color: var(--hh-success, #047857);
}

.tone-warning .outbox-status-icon {
  color: var(--hh-warning, #b45309);
}

.tone-danger .outbox-status-icon {
  color: var(--hh-danger, #b91c1c);
}

.tone-muted .outbox-status-icon {
  color: var(--hh-text-secondary, #6b7280);
}

@keyframes outbox-status-pulse {
  0% {
    background-position: 200% 0;
  }

  100% {
    background-position: -200% 0;
  }
}
</style>
