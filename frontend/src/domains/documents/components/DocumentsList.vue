<script setup lang="ts">
import { ref, computed } from 'vue'
import { useVirtualizer } from '@tanstack/vue-virtual'
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type { DocDisplayItem } from '../types/documents'

const { t } = useI18n()

const props = defineProps<{
  documents: DocDisplayItem[]
  searchQuery: string
  activeFilter: string
}>()

const emit = defineEmits<{
  'update:search-query': [value: string]
  'update:active-filter': [value: string]
}>()

const parentRef = ref<HTMLDivElement | null>(null)

const virtualOptions = computed(() => ({
  count: props.documents.length,
  getScrollElement: () => parentRef.value,
  estimateSize: () => 80,
  overscan: 5
}))

const virtualizer = useVirtualizer(virtualOptions)

const virtualItems = computed(() => virtualizer.value.getVirtualItems())
const totalSize = computed(() => virtualizer.value.getTotalSize())
</script>

<template>
  <div class="widget-frame documents-list-panel">
    <div class="document-main-list">
      <div class="document-filter-bar">
        <div class="segmented">
          <button
            type="button"
            :class="['segmented', { active: activeFilter === 'all' }]"
            @click="emit('update:active-filter', 'all')"
          >{{ t('All') }}</button>
          <button
            type="button"
            :class="['segmented', { active: activeFilter === 'shared' }]"
            @click="emit('update:active-filter', 'shared')"
          >{{ t('Shared') }}</button>
          <button
            type="button"
            :class="['segmented', { active: activeFilter === 'recent' }]"
            @click="emit('update:active-filter', 'recent')"
          >{{ t('Recent') }}</button>
        </div>
        <label class="local-search">
          <Icon icon="tabler:search" :size="17" />
          <input
            :placeholder="t('Search documents...')"
            :value="searchQuery"
            @input="emit('update:search-query', ($event.target as HTMLInputElement).value)"
          />
        </label>
      </div>
      <div ref="parentRef" class="documents-scroll-container">
        <div v-if="documents.length === 0" class="muted p-4">{{ t('No documents found') }}</div>
        <div v-else :style="{ height: `${totalSize}px` }">
          <article
            v-for="vitem in virtualItems"
            :key="String(vitem.key)"
            class="document-row"
            :style="{ transform: `translateY(${vitem.start}px)`, height: `${vitem.size}px` }"
          >
            <span :class="['round-icon', documents[vitem.index].tone]">
              <Icon :icon="documents[vitem.index].icon" :size="20" />
            </span>
            <div>
              <strong>{{ documents[vitem.index].name }}</strong>
              <small>{{ documents[vitem.index].source }} &middot; {{ documents[vitem.index].project }} &middot; {{ documents[vitem.index].type }} &middot; {{ documents[vitem.index].size }}</small>
            </div>
            <time>{{ documents[vitem.index].date }}</time>
          </article>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.documents-scroll-container {
  flex: 1;
  overflow-y: auto;
}
</style>
