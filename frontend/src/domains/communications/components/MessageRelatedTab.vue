<script setup lang="ts">
import Icon from '../../../shared/ui/Icon.vue'
import Button from '../../../shared/ui/Button.vue'
import type { MailMessageDetailResponse } from '../types/communications'

const props = defineProps<{
  detail: MailMessageDetailResponse | null
}>()

const emit = defineEmits<{
  togglePin: []
  toggleImportant: []
  mute: []
  exportMd: []
}>()
</script>

<template>
  <div class="related-tab">
    <div v-if="!detail" class="no-data">No message selected</div>
    <div v-else class="related-actions">
      <div class="actions-group">
        <h4 class="group-title">Message Actions</h4>
        <Button variant="outline" size="sm" @click="emit('togglePin')">
          <Icon icon="tabler:pin" /> Pin
        </Button>
        <Button variant="outline" size="sm" @click="emit('toggleImportant')">
          <Icon icon="tabler:star" /> Important
        </Button>
        <Button variant="outline" size="sm" @click="emit('mute')">
          <Icon icon="tabler:bell-off" /> Mute
        </Button>
        <Button variant="outline" size="sm" @click="emit('exportMd')">
          <Icon icon="tabler:download" /> Export
        </Button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.related-tab {
  padding: 0.75rem;
}

.related-actions {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.actions-group {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.group-title {
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--hh-text-secondary, #6b7280);
  margin: 0;
}

.no-data {
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.875rem;
  padding: 2rem;
  text-align: center;
}
</style>
