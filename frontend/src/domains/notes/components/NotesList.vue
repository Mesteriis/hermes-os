<script setup lang="ts">
import { ref, computed } from 'vue'
import { useVirtualizer } from '@tanstack/vue-virtual'
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type { NoteItem } from '../types/notes'

const { t } = useI18n()

const props = defineProps<{
  notes: NoteItem[]
  searchQuery: string
}>()

const emit = defineEmits<{
  'update:search-query': [value: string]
}>()

const parentRef = ref<HTMLDivElement | null>(null)

const virtualOptions = computed(() => ({
  count: props.notes.length,
  getScrollElement: () => parentRef.value,
  estimateSize: () => 100,
  overscan: 5
}))

const virtualizer = useVirtualizer(virtualOptions)

const virtualItems = computed(() => virtualizer.value.getVirtualItems())
const totalSize = computed(() => virtualizer.value.getTotalSize())
</script>

<template>
  <div class="widget-frame notes-list-panel">
    <div class="notes-main-list">
      <label class="local-search">
        <Icon icon="tabler:search" :size="17" />
        <input
          :placeholder="t('Search notes...')"
          :value="searchQuery"
          @input="emit('update:search-query', ($event.target as HTMLInputElement).value)"
        />
      </label>
      <div ref="parentRef" class="notes-scroll-container">
        <div v-if="notes.length === 0" class="muted p-4">{{ t('No notes found') }}</div>
        <div v-else :style="{ height: `${totalSize}px` }">
          <article
            v-for="vitem in virtualItems"
            :key="String(vitem.key)"
            class="note-card"
            :style="{ transform: `translateY(${vitem.start}px)`, height: `${vitem.size}px` }"
          >
            <span class="round-icon">
              <Icon :icon="notes[vitem.index].icon" :size="22" />
            </span>
            <div>
              <strong>{{ notes[vitem.index].title }}</strong>
              <p>{{ notes[vitem.index].body }}</p>
              <div class="note-meta">
                <span>{{ notes[vitem.index].source }}</span>
                <em>{{ notes[vitem.index].tag }}</em>
                <time>{{ notes[vitem.index].time }}</time>
              </div>
            </div>
          </article>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.notes-scroll-container {
  flex: 1;
  overflow-y: auto;
}
</style>
