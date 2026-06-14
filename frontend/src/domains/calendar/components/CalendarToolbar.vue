<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import { useCalendarStore } from '../stores/calendar'
import type { CalendarViewMode } from '../types/calendar'

const { t } = useI18n()
const store = useCalendarStore()

const emit = defineEmits<{
  (e: 'search-calendar'): void
  (e: 'load-calendar'): void
  (e: 'load-weekly-brief'): void
  (e: 'refresh-all'): void
}>()

function setMode(mode: CalendarViewMode) {
  store.setViewMode(mode)
}
</script>

<template>
  <div class="widget-frame">
    <div class="view-header">
      <div class="view-title-with-icon">
        <span class="hero-mark small"><Icon icon="tabler:calendar" :size="28" /></span>
        <div>
          <h1>{{ t('Calendar') }}</h1>
          <p>{{ t('All your events from connected calendars') }}</p>
        </div>
      </div>
      <div class="search-bar">
        <input
          type="text"
          :placeholder="t('Search events...')"
          :value="store.searchQuery"
          @input="store.setSearchQuery(($event.target as HTMLInputElement).value); emit('search-calendar')"
        />
      </div>
      <div class="section-tabs pill-tabs">
        <button
          v-for="mode in (['day', 'week', 'month', 'agenda'] as CalendarViewMode[])"
          :key="mode"
          type="button"
          :class="['pill-tab', { active: store.viewMode === mode }]"
          @click="setMode(mode)"
        >{{ t(mode.charAt(0).toUpperCase() + mode.slice(1)) }}</button>
      </div>
      <button type="button" class="primary-button" @click="store.toggleNewEventForm()">
        <Icon icon="tabler:plus" :size="16" /> {{ t('New Event') }}
      </button>
      <button type="button" class="ghost-button" @click="emit('refresh-all')" :title="t('Refresh')">
        <Icon icon="tabler:refresh" :size="16" />
      </button>
    </div>
  </div>
</template>
