<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useVirtualizer } from '@tanstack/vue-virtual'
import MailListItem from './MailListItem.vue'
import type { CommunicationMessageSummary } from '../types/communications'

const props = defineProps<{
  messages: CommunicationMessageSummary[]
  selectedIndex: number
  isLoading: boolean
}>()

const emit = defineEmits<{
  select: [index: number]
}>()

const parentRef = ref<HTMLDivElement | null>(null)

const virtualOptions = computed(() => ({
  count: props.messages.length,
  getScrollElement: () => parentRef.value,
  estimateSize: () => 72,
  overscan: 5
}))

const virtualizer = useVirtualizer(virtualOptions)

const virtualItems = computed(() => virtualizer.value.getVirtualItems())
const totalSize = computed(() => virtualizer.value.getTotalSize())

function handleSelect(index: number) {
  emit('select', index)
}

// Scroll selected item into view
watch(() => props.selectedIndex, (idx) => {
  if (idx >= 0) {
    virtualizer.value.scrollToIndex(idx, { align: 'center' })
  }
})
</script>

<template>
  <div ref="parentRef" class="mail-list-container">
    <div v-if="isLoading" class="mail-list-loading">
      <span>Loading messages...</span>
    </div>
    <div v-else-if="messages.length === 0" class="mail-list-empty">
      <span>No messages found</span>
    </div>
    <div
      v-else
      class="mail-list-virtual"
      :style="{ height: `${totalSize}px` }"
    >
      <div
        v-for="vitem in virtualItems"
        :key="String(vitem.key)"
        class="mail-list-row"
        :style="{
          transform: `translateY(${vitem.start}px)`,
          height: `${vitem.size}px`
        }"
      >
        <MailListItem
          :message="messages[vitem.index]"
          :is-selected="vitem.index === selectedIndex"
          @select="handleSelect(vitem.index)"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
.mail-list-container {
  height: 100%;
  overflow-y: auto;
  overflow-x: hidden;
  position: relative;
}

.mail-list-loading,
.mail-list-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  padding: 2rem;
  font-size: 0.875rem;
  color: var(--hh-text-secondary, #6b7280);
}

.mail-list-virtual {
  position: relative;
  width: 100%;
}

.mail-list-row {
  position: absolute;
  left: 0;
  right: 0;
  overflow: hidden;
}
</style>
