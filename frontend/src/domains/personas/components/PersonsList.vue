<script setup lang="ts">
import { ref, computed } from 'vue'
import { useVirtualizer } from '@tanstack/vue-virtual'
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type { PersonItem } from '../types/persona'

const { t } = useI18n()

const props = defineProps<{
  personList: PersonItem[]
  selectedPersonIndex: number
}>()

const emit = defineEmits<{
  selectPerson: [index: number]
}>()

const parentRef = ref<HTMLDivElement | null>(null)

const virtualOptions = computed(() => ({
  count: props.personList.length,
  getScrollElement: () => parentRef.value,
  estimateSize: () => 70,
  overscan: 5
}))

const virtualizer = useVirtualizer(virtualOptions)

const virtualItems = computed(() => virtualizer.value.getVirtualItems())
const totalSize = computed(() => virtualizer.value.getTotalSize())
</script>

<template>
  <div class="widget-frame" data-widget-id="persons-list">
    <section class="panel persons-list-panel">
      <header>
        <div>
          <h1>{{ t('Persons') }}</h1>
          <p>{{ personList.length }} {{ t('persons') }}</p>
        </div>
        <button type="button" class="primary-button" disabled>{{ t('New Person') }}</button>
      </header>
      <div class="filter-tabs compact">
        <button type="button" class="active">{{ t('All') }}</button>
        <button type="button" disabled>{{ t('People') }} <em>532</em></button>
        <button type="button" disabled>{{ t('Companies') }} <em>110</em></button>
      </div>
      <label class="local-search">
        <Icon icon="tabler:search" :size="17" />
        <input :placeholder="t('Search persons...')" />
      </label>
      <div ref="parentRef" class="persons-scroll-container">
        <div v-if="personList.length === 0" class="muted p-4">{{ t('No persons found') }}</div>
        <div v-else :style="{ height: `${totalSize}px` }">
          <button
            v-for="vitem in virtualItems"
            :key="personList[vitem.index].person_id"
            type="button"
            class="person-row"
            :class="{ active: selectedPersonIndex === vitem.index }"
            :style="{ transform: `translateY(${vitem.start}px)`, height: `${vitem.size}px` }"
            @click="emit('selectPerson', vitem.index)"
          >
            <span class="round-icon ghost"><Icon icon="tabler:user" :size="20" /></span>
            <span>
              <strong>{{ personList[vitem.index].name }}</strong>
              <small>{{ personList[vitem.index].role }}</small>
              <em>{{ personList[vitem.index].company }}</em>
            </span>
            <small>{{ personList[vitem.index].status ?? personList[vitem.index].channel ?? t('Email') }}</small>
          </button>
        </div>
      </div>
    </section>
  </div>
</template>

<style scoped>
.persons-scroll-container {
  flex: 1;
  overflow-y: auto;
}
.persons-list-panel {
  display: flex;
  flex-direction: column;
  padding: 12px;
  height: 100%;
}
.persons-list-panel header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}
.persons-list-panel .person-row {
  position: relative;
  display: grid;
  grid-template-columns: 44px 1fr auto;
  gap: 10px;
  align-items: center;
  width: 100%;
  min-height: var(--hh-widget-card-compact);
  border: 1px solid transparent;
  border-radius: var(--hh-radius-md);
  background: transparent;
  color: #e6f7f5;
  padding: 9px 10px;
  text-align: left;
  cursor: pointer;
}
.persons-list-panel .person-row.active {
  border-color: rgba(45, 240, 206, 0.24);
  background: rgba(25, 109, 100, 0.24);
}
.persons-list-panel .person-row strong {
  display: block;
  color: var(--hh-color-text-bright);
  font-size: 14px;
  font-weight: 560;
}
.persons-list-panel .person-row small,
.persons-list-panel .person-row em {
  display: block;
  margin-top: 5px;
  overflow: hidden;
  font-size: 11px;
  font-style: normal;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
