<script setup lang="ts">
import Icon from '../../../shared/ui/Icon.vue'
import Button from '../../../shared/ui/Button.vue'
import type { EmailDraft, ComposeFormModel } from '../types/communications'

const props = defineProps<{
  drafts: EmailDraft[]
}>()

const emit = defineEmits<{
  openDraft: [draft: EmailDraft]
  deleteDraft: [draftId: string]
}>()
</script>

<template>
  <div v-if="drafts.length > 0" class="draft-strip">
    <div class="draft-strip-header">
      <Icon icon="tabler:edit" class="draft-strip-icon" />
      <span class="draft-strip-title">Drafts ({{ drafts.length }})</span>
    </div>
    <div class="draft-list">
      <div v-for="draft in drafts" :key="draft.draft_id" class="draft-item">
        <div class="draft-info" @click="emit('openDraft', draft)">
          <span class="draft-subject">{{ draft.subject || '(No subject)' }}</span>
          <span class="draft-recipients">{{ draft.to_recipients?.join(', ') || 'No recipients' }}</span>
        </div>
        <Button variant="ghost" size="sm" class="draft-delete-btn" @click="emit('deleteDraft', draft.draft_id)">
          <Icon icon="tabler:x" />
        </Button>
      </div>
    </div>
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
  display: flex;
  flex-direction: column;
}

.draft-item {
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
</style>
