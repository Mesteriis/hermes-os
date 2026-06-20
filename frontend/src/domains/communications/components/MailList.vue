<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useVirtualizer } from '@tanstack/vue-virtual'
import MailListItem from './MailListItem.vue'
import { useCommunicationMessagePrefetch } from '../queries/mailPrefetch'
import type { CommunicationMessageSummary } from '../types/communications'

const props = defineProps<{
  messages: CommunicationMessageSummary[]
  selectedIndex: number
  selectedMessageIds: string[]
  isLoading: boolean
  hasNextPage: boolean
  isFetchingNextPage: boolean
}>()

const emit = defineEmits<{
  select: [index: number]
  toggleSelection: [messageId: string, extendRange: boolean]
  selectVisible: [messageIds: string[]]
  clearSelection: []
  loadMore: []
}>()

const parentRef = ref<HTMLDivElement | null>(null)
const prefetchCommunicationMessage = useCommunicationMessagePrefetch()

const virtualOptions = computed(() => ({
  count: props.messages.length,
  getScrollElement: () => parentRef.value,
  estimateSize: () => 72,
  overscan: 5
}))

const virtualizer = useVirtualizer(virtualOptions)

const virtualItems = computed(() => virtualizer.value.getVirtualItems())
const totalSize = computed(() => virtualizer.value.getTotalSize())
const selectedMessageIdSet = computed(() => new Set(props.selectedMessageIds))
const visibleMessageIds = computed(() => props.messages.map((message) => message.message_id))

function handleSelect(index: number) {
  emit('select', index)
}

function handlePrefetch(messageId: string) {
  void prefetchCommunicationMessage(messageId)
}

function handleScroll() {
  const el = parentRef.value
  if (!el || !props.hasNextPage || props.isFetchingNextPage) return
  if (el.scrollTop + el.clientHeight >= el.scrollHeight - 360) {
    emit('loadMore')
  }
}

function handleKeydown(event: KeyboardEvent) {
  if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'a') {
    if (visibleMessageIds.value.length === 0) return
    event.preventDefault()
    emit('selectVisible', visibleMessageIds.value)
    return
  }

  if (event.key === 'Escape') {
    if (props.selectedMessageIds.length === 0) return
    event.preventDefault()
    emit('clearSelection')
    return
  }

  const current = props.messages[props.selectedIndex]
  if (!current) return

  if (event.code === 'Space') {
    event.preventDefault()
    emit('toggleSelection', current.message_id, event.shiftKey)
    return
  }

  if (!event.shiftKey) return
  const offset = event.key === 'ArrowDown' ? 1 : event.key === 'ArrowUp' ? -1 : 0
  if (offset === 0) return

  const nextIndex = props.selectedIndex + offset
  const next = props.messages[nextIndex]
  if (!next) return
  event.preventDefault()
  emit('select', nextIndex)
  emit('toggleSelection', next.message_id, true)
}

// Scroll selected item into view
watch(() => props.selectedIndex, (idx) => {
  if (idx >= 0) {
    virtualizer.value.scrollToIndex(idx, { align: 'center' })
  }
})
</script>

<template>
  <div
    ref="parentRef"
    class="mail-list-container"
    tabindex="0"
    role="listbox"
    aria-multiselectable="true"
    @keydown="handleKeydown"
    @scroll="handleScroll"
  >
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
          :is-checked="selectedMessageIdSet.has(messages[vitem.index].message_id)"
          :selected-message-ids="selectedMessageIds"
          @select="handleSelect(vitem.index)"
          @toggle-selection="emit('toggleSelection', messages[vitem.index].message_id, $event)"
          @prefetch="handlePrefetch(messages[vitem.index].message_id)"
        />
      </div>
    </div>
    <div v-if="isFetchingNextPage" class="mail-list-page-loading">
      <span>Loading more...</span>
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
.mail-list-empty,
.mail-list-page-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 2rem;
  font-size: 0.875rem;
  color: var(--hh-text-secondary, #6b7280);
}

.mail-list-loading,
.mail-list-empty {
  height: 100%;
}

.mail-list-page-loading {
  min-height: 3rem;
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
