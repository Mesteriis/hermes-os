<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import type { CalendarEvent, CalendarAccount } from '../types/calendar'
import { eventTypeTone, formatEventTime, formatEventDayShort } from '../stores/calendar'

const { t } = useI18n()

const props = defineProps<{
  weekColumns: string[]
  calendarSearchResults: CalendarEvent[]
  filteredEvents: CalendarEvent[]
  isCalendarLoading: boolean
  calendarAccounts: CalendarAccount[]
  selectedEvent: CalendarEvent | null
  onPrepareEvent: (evt: CalendarEvent) => void
}>()

function handleEventClick(evt: CalendarEvent) {
  props.onPrepareEvent(evt)
}
</script>

<template>
  <section class="panel week-board">
    <div class="week-header">
      <strong v-for="(day, i) in weekColumns" :key="i">{{ day }}</strong>
    </div>
    <div class="event-list">
      <div v-if="isCalendarLoading" class="loading-state">{{ t('Loading events...') }}</div>
      <div
        v-else-if="(calendarSearchResults.length > 0 ? calendarSearchResults : filteredEvents).length === 0"
        class="empty-state"
      >{{ t('No events') }}</div>
      <template v-else>
        <div
          v-for="evt in (calendarSearchResults.length > 0 ? calendarSearchResults : filteredEvents)"
          :key="evt.event_id"
          :class="['event-row', eventTypeTone(evt.event_type)]"
          role="button"
          tabindex="0"
          @click="handleEventClick(evt)"
          @keydown.enter="handleEventClick(evt)"
        >
          <span class="event-day">{{ formatEventDayShort(evt.start_at) }}</span>
          <span class="event-time">
            {{ formatEventTime(evt.start_at) }} - {{ formatEventTime(evt.end_at) }}
          </span>
          <strong>{{ evt.title }}</strong>
          <span class="event-type-chip">{{ evt.event_type || t('event') }}</span>
          <em v-if="evt.importance_score && evt.importance_score > 0.5" class="importance-dot high"></em>
          <em v-if="evt.readiness_score != null && evt.readiness_score < 0.5" class="importance-dot warn"></em>
        </div>
      </template>
    </div>
    <footer class="source-footer">
      <span v-for="acct in calendarAccounts" :key="acct.account_id" class="source-badge">{{ acct.account_name }}</span>
    </footer>
  </section>
</template>
