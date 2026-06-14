<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from '../../../platform/i18n'
import CalendarToolbar from '../components/CalendarToolbar.vue'
import CalendarWeekGrid from '../components/CalendarWeekGrid.vue'
import CalendarUpcoming from '../components/CalendarUpcoming.vue'
import CalendarSourceStatus from '../components/CalendarSourceStatus.vue'
import {
  useCalendarEventsQuery,
  useCalendarAccountsQuery
} from '../queries/useCalendarEventsQuery'
import { useCalendarStore, getWeekStart, getWeekColumns, filterWeekEvents, formatEventDateTime, eventTypeLabel } from '../stores/calendar'
import {
  fetchCalendarSources,
  fetchWeeklyBrief,
  fetchEventBrief,
  fetchEventAgenda,
  searchCalendarEvents,
  createCalendarEvent,
  fetchEventContextPack
} from '../api/calendar'
import Icon from '../../../shared/ui/Icon.vue'
import Button from '../../../shared/ui/Button.vue'
import type { CalendarEvent, CalendarSource, WeeklyBrief } from '../types/calendar'

const { t } = useI18n()
const store = useCalendarStore()

// TanStack Query data
const { data: accountsData, isLoading: isAccountsLoading } = useCalendarAccountsQuery()
const { data: eventsData, isLoading: isEventsLoading, refetch: refetchEvents } = useCalendarEventsQuery(200)

const calendarAccounts = computed(() => accountsData.value ?? [])
const calendarEvents = computed(() => eventsData.value ?? [])

// Calendar sources loaded manually (depends on accounts)
const calendarSources = ref<CalendarSource[]>([])

// Search results
const searchResults = ref<CalendarEvent[]>([])

// Week calculation
const weekStart = ref(getWeekStart())
const weekColumns = computed(() => getWeekColumns(weekStart.value))
const filteredEvents = computed(() => filterWeekEvents(calendarEvents.value, weekStart.value))

// Event detail state
const eventContext = ref<Record<string, unknown> | null>(null)

// Derived
const isLoading = computed(() => isAccountsLoading || isEventsLoading)
const displayEvents = computed(() =>
  searchResults.value.length > 0 ? searchResults.value : filteredEvents.value
)

async function loadSources() {
  const results: CalendarSource[] = []
  for (const acct of calendarAccounts.value) {
    try {
      const res = await fetchCalendarSources(acct.account_id)
      results.push(...res.items)
    } catch (_) { /* sources optional */ }
  }
  calendarSources.value = results
}

async function loadWeeklyBrief() {
  try {
    const brief = await fetchWeeklyBrief()
    store.setWeeklyBrief(brief as unknown as WeeklyBrief)
  } catch (_) {
    store.setWeeklyBrief(null)
  }
}

async function handleSearch() {
  if (!store.searchQuery.trim()) {
    searchResults.value = []
    return
  }
  try {
    const res = await searchCalendarEvents(store.searchQuery)
    searchResults.value = (res.results as CalendarEvent[]) || []
  } catch (_) {
    searchResults.value = []
  }
}

async function handlePrepareEvent(evt: CalendarEvent) {
  store.selectEvent(evt)
  try {
    const [ctx, brief, agenda] = await Promise.all([
      fetchEventContextPack(evt.event_id),
      fetchEventBrief(evt.event_id),
      fetchEventAgenda(evt.event_id)
    ])
    eventContext.value = ctx as unknown as Record<string, unknown>
    store.setEventBrief(brief)
    store.setEventAgenda(agenda as unknown as Record<string, unknown> | null)
  } catch (_) {
    eventContext.value = null
    store.setEventBrief(null)
    store.setEventAgenda(null)
  }
}

async function handleCreateEvent() {
  if (!store.newEventTitle || !store.newEventStart || !store.newEventEnd) return
  try {
    await createCalendarEvent({
      title: store.newEventTitle,
      start_at: new Date(store.newEventStart).toISOString(),
      end_at: new Date(store.newEventEnd).toISOString(),
      event_type: store.newEventType
    })
    store.resetNewEventForm()
    refetchEvents()
  } catch (e) {
    store.setCalendarError(e instanceof Error ? e.message : 'Create failed')
  }
}

async function handleRefreshAll() {
  refetchEvents()
  loadSources()
  loadWeeklyBrief()
}

onMounted(() => {
  loadSources()
  loadWeeklyBrief()
})
</script>

<template>
  <section class="calendar-page">
    <CalendarToolbar
      @search-calendar="handleSearch"
      @load-calendar="refetchEvents"
      @load-weekly-brief="loadWeeklyBrief"
      @refresh-all="handleRefreshAll"
    />

    <!-- New Event Form -->
    <div v-if="store.showNewEventForm" class="panel new-event-form">
      <h3>{{ t('New Event') }}</h3>
      <div class="form-row">
        <input
          type="text"
          :placeholder="t('Event title')"
          v-model="store.newEventTitle"
        />
        <select v-model="store.newEventType">
          <option
            v-for="opt in ['meeting', 'focus', 'deadline', 'personal', 'travel', 'tax', 'review', 'planning']"
            :key="opt"
            :value="opt"
          >{{ eventTypeLabel(opt) }}</option>
        </select>
      </div>
      <div class="form-row">
        <input type="datetime-local" v-model="store.newEventStart" />
        <span>&rarr;</span>
        <input type="datetime-local" v-model="store.newEventEnd" />
      </div>
      <div class="form-actions">
        <Button variant="default" @click="handleCreateEvent">{{ t('Create') }}</Button>
        <Button variant="ghost" @click="store.resetNewEventForm()">{{ t('Cancel') }}</Button>
      </div>
    </div>

    <!-- Filter bar -->
    <div class="filter-bar">
      <span>{{ calendarAccounts.length }} {{ t('accounts') }} &middot; {{ calendarEvents.length }} {{ t('events') }}</span>
      <span v-if="store.calendarError" class="error-text">{{ store.calendarError }}</span>
      <span v-if="searchResults.length > 0" class="search-hint">
        {{ t('Search') }}: {{ searchResults.length }} {{ t('results for') }} "{{ store.searchQuery }}"
      </span>
    </div>

    <!-- Main layout -->
    <div class="calendar-layout">
      <CalendarWeekGrid
        :week-columns="weekColumns"
        :calendar-search-results="searchResults"
        :filtered-events="filteredEvents"
        :is-calendar-loading="isAccountsLoading || isEventsLoading"
        :calendar-accounts="calendarAccounts"
        :selected-event="store.selectedEvent"
        :on-prepare-event="handlePrepareEvent"
      />
      <aside class="stacked-rail">
        <!-- Weekly Brief -->
        <div class="panel info-card">
          <h2>
            {{ t('Weekly Brief') }}
            <Button variant="ghost" size="sm" @click="loadWeeklyBrief">
              <Icon icon="tabler:refresh" :size="12" />
            </Button>
          </h2>
          <template v-if="store.weeklyBrief">
            <div class="metric-grid tiny">
              <article class="metric-card">
                <span>{{ t('Events') }}</span>
                <strong>{{ store.weeklyBrief.upcoming_events_this_week || 0 }}</strong>
              </article>
              <article class="metric-card">
                <span>{{ t('Overdue') }}</span>
                <strong>{{ store.weeklyBrief.overdue_deadlines || 0 }}</strong>
              </article>
              <article class="metric-card">
                <span>{{ t('No Notes') }}</span>
                <strong>{{ store.weeklyBrief.past_events_without_notes || 0 }}</strong>
              </article>
            </div>
          </template>
          <p v-else class="muted">{{ t('Click refresh to load') }}</p>
        </div>

        <CalendarUpcoming
          :calendar-events="calendarEvents"
          @prepare-event="handlePrepareEvent"
        />

        <!-- Event Detail -->
        <div v-if="store.selectedEvent" class="panel info-card event-detail">
          <h2>
            {{ store.selectedEvent.title }}
            <Button variant="ghost" size="sm" @click="store.selectEvent(null)">
              <Icon icon="tabler:x" :size="14" />
            </Button>
          </h2>
          <div class="event-meta">
            <span><Icon icon="tabler:clock" :size="14" /> {{ formatEventDateTime(store.selectedEvent.start_at) }}</span>
            <span v-if="store.selectedEvent.location">
              <Icon icon="tabler:map-pin" :size="14" /> {{ store.selectedEvent.location }}
            </span>
            <span :class="['chip', store.selectedEvent.status]">{{ store.selectedEvent.status }}</span>
          </div>
          <div v-if="store.eventBrief" class="brief-section">
            <h4>{{ t('Brief') }}</h4>
            <div v-if="(store.eventBrief.participants as any[])?.length" class="brief-participants">
              <span
                v-for="(p, idx) in (store.eventBrief.participants as any[])"
                :key="idx"
                class="participant-chip"
              >{{ p.name || p.email }}</span>
            </div>
            <p v-if="(store.eventBrief.context as any)?.summary" class="muted">
              {{ (store.eventBrief.context as any).summary }}
            </p>
          </div>
          <div v-if="store.eventAgenda" class="brief-section">
            <h4>{{ t('Agenda') }}</h4>
            <ul v-if="store.eventAgenda.suggested_agenda" class="agenda-list">
              <li v-for="(item, idx) in store.eventAgenda.suggested_agenda" :key="idx">{{ item }}</li>
            </ul>
          </div>
          <div class="event-actions">
            <Button variant="default" size="sm" @click="store.selectedEvent && handlePrepareEvent(store.selectedEvent)">
              <Icon icon="tabler:brain" :size="14" /> {{ t('Prepare') }}
            </Button>
            <Button variant="ghost" size="sm" @click="store.selectEvent(null)">
              <Icon icon="tabler:check" :size="14" /> {{ t('Complete') }}
            </Button>
          </div>
        </div>

        <CalendarSourceStatus
          :calendar-sources="calendarSources"
          :calendar-accounts="calendarAccounts"
        />
      </aside>
    </div>
  </section>
</template>
