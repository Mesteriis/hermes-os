<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import type { CalendarEvent } from '../types/calendar'
import { formatEventDate, formatEventTime } from '../stores/calendar'

const { t } = useI18n()

const props = defineProps<{
  calendarEvents: CalendarEvent[]
}>()

const emit = defineEmits<{
  (e: 'prepare-event', evt: CalendarEvent): void
}>()

function handleClick(evt: CalendarEvent) {
  emit('prepare-event', evt)
}
</script>

<template>
  <section class="panel info-card">
    <h2>{{ t('Upcoming') }}</h2>
    <p v-if="calendarEvents.length === 0" class="muted">{{ t('No upcoming events') }}</p>
    <template v-else>
      <div
        v-for="evt in calendarEvents.filter(e => new Date(e.start_at) >= new Date()).slice(0, 8)"
        :key="evt.event_id"
        class="deadline"
        role="button"
        tabindex="0"
        @click="handleClick(evt)"
        @keydown.enter="handleClick(evt)"
      >
        <span>{{ formatEventDate(evt.start_at) }} &middot; {{ evt.title }}</span>
        <time>{{ formatEventTime(evt.start_at) }}</time>
      </div>
    </template>
  </section>
</template>
