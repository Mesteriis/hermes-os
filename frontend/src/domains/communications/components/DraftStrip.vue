<script setup lang="ts">
import { computed, ref } from 'vue'
import { useVirtualizer } from '@tanstack/vue-virtual'
import Icon from '../../../shared/ui/Icon.vue'
import Button from '../../../shared/ui/Button.vue'
import type { CommunicationDraft } from '../types/communications'

const props = defineProps<{
  drafts: CommunicationDraft[]
  hasMore: boolean
  isLoadingMore: boolean
}>()

const emit = defineEmits<{
  openDraft: [draft: CommunicationDraft]
  deleteDraft: [draftId: string]
  loadMore: []
}>()

const draftScrollRef = ref<HTMLDivElement | null>(null)
const draftVirtualOptions = computed(() => ({
  count: props.drafts.length,
  getScrollElement: () => draftScrollRef.value,
  estimateSize: () => 46,
  overscan: 8
}))
const draftVirtualizer = useVirtualizer(draftVirtualOptions)
const virtualDraftRows = computed(() => draftVirtualizer.value.getVirtualItems())
const draftVirtualTotalSize = computed(() => draftVirtualizer.value.getTotalSize())
</script>

<template>
  <div v-if="drafts.length > 0" class="draft-strip">
    <div class="draft-strip-header">
      <Icon icon="tabler:edit" class="draft-strip-icon" />
      <span class="draft-strip-title">Drafts ({{ drafts.length }})</span>
    </div>
    <div ref="draftScrollRef" class="draft-list" :style="{ maxHeight: '12rem' }">
      <div class="draft-list-track" :style="{ height: `${draftVirtualTotalSize}px` }">
        <div
          v-for="virtualRow in virtualDraftRows"
          :key="String(virtualRow.key)"
          class="draft-item"
          :style="{
            height: `${virtualRow.size}px`,
            transform: `translateY(${virtualRow.start}px)`
          }"
        >
          <div class="draft-info" @click="emit('openDraft', drafts[virtualRow.index])">
            <span class="draft-subject">{{ drafts[virtualRow.index].subject || '(No subject)' }}</span>
            <span class="draft-recipients">{{ drafts[virtualRow.index].to_recipients?.join(', ') || 'No recipients' }}</span>
          </div>
          <Button
            variant="ghost"
            size="sm"
            class="draft-delete-btn"
            @click="emit('deleteDraft', drafts[virtualRow.index].draft_id)"
          >
            <Icon icon="tabler:x" />
          </Button>
        </div>
      </div>
    </div>
    <Button
      v-if="hasMore"
      class="draft-load-more"
      type="button"
      variant="ghost"
      size="sm"
      :disabled="isLoadingMore"
      @click="emit('loadMore')"
    >
      <Icon icon="tabler:chevron-down" />
      <span>{{ isLoadingMore ? 'Loading drafts...' : 'Load more drafts' }}</span>
    </Button>
  </div>
</template>

<style scoped>
.draft-strip {
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
  background: var(--hh-bg-warning-light, #fffbeb);
}

.draft-strip-header {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.375rem 0.75rem;
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--hh-text-warning, #d97706);
}

.draft-strip-icon {
  width: 14px;
  height: 14px;
}

.draft-strip-title {
  color: var(--hh-text-warning, #d97706);
}

.draft-list {
  position: relative;
  overflow: auto;
}

.draft-list-track {
  position: relative;
  width: 100%;
}

.draft-item {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  display: flex;
  align-items: center;
  padding: 0.25rem 0.75rem;
  gap: 0.5rem;
  border-top: 1px solid var(--hh-border, #e5e7eb);
}

.draft-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 0.0625rem;
  cursor: pointer;
  min-width: 0;
}

.draft-subject {
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--hh-text-primary, #1f2937);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.draft-recipients {
  font-size: 0.6875rem;
  color: var(--hh-text-tertiary, #9ca3af);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.draft-delete-btn {
  flex-shrink: 0;
}

.draft-load-more {
  width: 100%;
  justify-content: center;
  border-radius: 0;
  border-top: 1px solid var(--hh-border, #e5e7eb);
  background: color-mix(in srgb, var(--hh-bg-warning-light, #fffbeb) 78%, transparent);
  color: var(--hh-text-warning, #d97706);
  font-size: 0.75rem;
  font-weight: 600;
  gap: 0.375rem;
}

.draft-load-more:disabled {
  cursor: not-allowed;
  opacity: 0.6;
}
</style>
